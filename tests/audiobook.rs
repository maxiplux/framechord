//! Offline audiobook contract and engine tests with a mocked speech provider.
use async_trait::async_trait;
use framechord::{
    audio, audiobook,
    contracts::*,
    engine::{self, Submit},
    error::{Error, Result},
    pricing, project,
    providers::{Adapter, Generated},
    validation,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

static ENGINE_TESTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Returns 0.1 s of audio per request and records every narrated text.
struct Fake {
    calls: AtomicUsize,
    texts: Mutex<Vec<String>>,
}
impl Fake {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            texts: Mutex::new(vec![]),
        })
    }
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}
#[async_trait]
impl Adapter for Fake {
    async fn generate(&self, e: &Effective) -> Result<Generated> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let text = e.text.clone().unwrap_or_default();
        self.texts.lock().unwrap().push(text.clone());
        if text.contains("FAIL") {
            return Err(Error::new("provider_rejected", "Mock rejection", 3));
        }
        Ok(Generated {
            bytes: audio::wav(&[0.2; 2400])?,
            format: "wav".into(),
            request_id: Some(format!("remote-{}", e.id)),
            usage: None,
        })
    }
}

fn config() -> Config {
    let mut cfg = Config::default();
    cfg.defaults.insert(
        Media::Speech,
        Selection {
            provider: Provider::Openai,
            model: "gpt-4o-mini-tts".into(),
        },
    );
    let settings = Settings {
        voice: Some("alloy".into()),
        ..Settings::default()
    };
    cfg.default_settings.insert(Media::Speech, settings);
    cfg
}

const MANUSCRIPT: &str = "---\ntitle: Test\n---\n\n# The Start\n\nFirst paragraph of the opening chapter, long enough that two paragraphs cannot share one small request.\n\nSecond paragraph of the opening chapter, which therefore becomes the second provider request.\n\n# Middle\n\nOnly one paragraph here.\n\n# The End\n\nClosing words.\n";

fn job(dir: &Path, options: AudiobookOptions) -> Job {
    fs::write(dir.join("book.md"), MANUSCRIPT).unwrap();
    serde_json::from_value(serde_json::json!({
        "schema_version": 2,
        "id": "book",
        "media": "audiobook",
        "manuscript": "book.md",
        "assembly": {"pause_ms": 500},
    }))
    .map(|mut j: Job| {
        j.audiobook = Some(options);
        j
    })
    .unwrap()
}
fn resolve(dir: &Path, options: AudiobookOptions) -> Effective {
    validation::resolve(&job(dir, options), dir, &config()).unwrap()
}
fn small(format: AudioFormat, output_mode: OutputMode) -> AudiobookOptions {
    AudiobookOptions {
        format,
        output_mode,
        // Splits the first chapter into two paragraph-aligned parts.
        max_chars_per_request: Some(200),
        title: Some("Test Book".into()),
        author: Some("A. Writer".into()),
        announce_titles: true,
        ..Default::default()
    }
}
fn leaves(book: &Effective) -> usize {
    book.turns.iter().map(|c| c.turns.len()).sum()
}

#[test]
fn manuscript_resolves_to_chapters_with_exact_text_and_one_narrator() {
    let t = tempfile::tempdir().unwrap();
    let options = AudiobookOptions {
        max_chars_per_request: Some(200),
        announce_titles: true,
        ..Default::default()
    };
    let book = resolve(t.path(), options);
    assert_eq!(book.media, Media::Audiobook);
    let ids: Vec<_> = book.turns.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["the-start", "middle", "the-end"]);
    let first: String = book.turns[0]
        .turns
        .iter()
        .map(|p| p.text.clone().unwrap())
        .collect();
    assert_eq!(
        first,
        "The Start\n\nFirst paragraph of the opening chapter, long enough that two paragraphs cannot share one small request.\n\nSecond paragraph of the opening chapter, which therefore becomes the second provider request."
    );
    assert!(book.turns.iter().all(|c| {
        c.text.is_none()
            && c.settings.voice.as_deref() == Some("alloy")
            && c.turns
                .iter()
                .all(|p| p.audiobook.is_none() && p.assembly.is_none())
    }));
    let info = book.turns[2]
        .audiobook
        .as_ref()
        .unwrap()
        .chapter
        .clone()
        .unwrap();
    assert_eq!((info.index, info.count), (3, 3));
    assert!(info.source.starts_with("book.md:"));
    let summary = audiobook::summary(
        &job(t.path(), AudiobookOptions::default()),
        t.path(),
        &config(),
        &book,
    )
    .unwrap();
    assert_eq!(summary["narrator"]["selection_source"], "speech_default");
    assert_eq!(summary["front_matter"]["lines"], "1-3");
}

#[test]
fn audiobook_contract_rejections_happen_before_any_request() {
    let t = tempfile::tempdir().unwrap();
    let cfg = config();
    let base = job(t.path(), AudiobookOptions::default());
    let check = |j: &Job, needle: &str| {
        let error = validation::resolve(j, t.path(), &cfg)
            .unwrap_err()
            .to_string();
        assert!(error.contains(needle), "{error}");
    };
    let mut v1 = base.clone();
    v1.schema_version = 1;
    v1.assembly = None;
    check(&v1, "schema_version 2");
    let mut both = base.clone();
    fs::write(t.path().join("one.md"), "Words.").unwrap();
    both.chapters = vec![Chapter {
        id: "one".into(),
        title: None,
        text_file: "one.md".into(),
        brief: None,
        settings: Settings::default(),
    }];
    check(&both, "exactly one of manuscript or chapters");
    let mut output = base.clone();
    output.output = Some("book.wav".into());
    check(&output, "--output-dir");
    let mut gain = base.clone();
    gain.assembly
        .as_mut()
        .unwrap()
        .turn_gain_db
        .insert("x".into(), 1.0);
    check(&gain, "turn_gain_db");
    let mut format = base.clone();
    format.settings.format = Some("mp3".into());
    check(&format, "audiobook.format");
    let mut bitrate = base.clone();
    bitrate.audiobook = Some(AudiobookOptions {
        format: AudioFormat::Mp3,
        mp3_bitrate_kbps: Some(57),
        ..Default::default()
    });
    check(&bitrate, "mp3_bitrate_kbps");
    // Rates the encoder cannot produce are rejected before any request.
    for kbps in [56, 144] {
        bitrate.audiobook.as_mut().unwrap().mp3_bitrate_kbps = Some(kbps);
        check(&bitrate, "mp3_bitrate_kbps");
    }
    let mut speech = base.clone();
    speech.media = Media::Speech;
    check(&speech, "only valid for audiobook");
    // One narrator: a chapter cannot switch the voice.
    let mut voices = base.clone();
    voices.manuscript = None;
    fs::write(t.path().join("two.md"), "More words.").unwrap();
    let other = Settings {
        voice: Some("verse".into()),
        ..Settings::default()
    };
    voices.chapters = vec![
        Chapter {
            id: "one".into(),
            title: None,
            text_file: "one.md".into(),
            brief: None,
            settings: Settings::default(),
        },
        Chapter {
            id: "two".into(),
            title: None,
            text_file: "two.md".into(),
            brief: None,
            settings: other,
        },
    ];
    check(&voices, "changes the narrator voice");
    let mut duplicate = voices.clone();
    duplicate.chapters[1].id = "one".into();
    duplicate.chapters[1].settings = Settings::default();
    check(&duplicate, "unique");
    let mut long = base.clone();
    long.id = "x".repeat(50);
    check(&long, "too long");
}

#[test]
fn existing_effective_contracts_serialize_without_audiobook_fields() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("s.md"), "Hello there.").unwrap();
    let speech: Job = serde_json::from_value(serde_json::json!({
        "schema_version": 1, "id": "s", "media": "speech", "text_file": "s.md"
    }))
    .unwrap();
    let e = validation::resolve(&speech, t.path(), &config()).unwrap();
    let value = serde_json::to_value(&e).unwrap();
    assert!(value.get("audiobook").is_none());
    let job_value = serde_json::to_value(&speech).unwrap();
    for key in ["manuscript", "split", "chapters", "audiobook"] {
        assert!(job_value.get(key).is_none(), "{key}");
    }
    // Packaging choices never change what a provider part is asked to read.
    let wav = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full));
    let mp3 = resolve(t.path(), small(AudioFormat::Mp3, OutputMode::Chapters));
    for (a, b) in wav.turns.iter().zip(&mp3.turns) {
        for (x, y) in a.turns.iter().zip(&b.turns) {
            assert_eq!(
                validation::fingerprint(x).unwrap(),
                validation::fingerprint(y).unwrap()
            );
        }
    }
}

fn chapter_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn full_wav_book_publishes_chapters_book_index_and_cue() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let book = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full));
    let parts = leaves(&book);
    assert_eq!(parts, 4);
    let out = t.path().join("out");
    let run = engine::submit(
        t.path(),
        vec![book],
        Limits::default(),
        Submit {
            output_dir: Some(out.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let fake = Fake::new();
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(
        run.state,
        State::Succeeded,
        "{:?}",
        run.jobs.iter().map(|j| &j.error).collect::<Vec<_>>()
    );
    assert_eq!(fake.calls(), parts);
    assert_eq!(
        chapter_files(&out.join("book")),
        [
            "book.chapters.json",
            "book.cue",
            "book.wav",
            "chapter-01-the-start.wav",
            "chapter-02-middle.wav",
            "chapter-03-the-end.wav",
        ]
    );
    let index: ChapterIndex =
        serde_json::from_slice(&fs::read(out.join("book/book.chapters.json")).unwrap()).unwrap();
    assert_eq!(index.chapters.len(), 3);
    assert_eq!(index.chapters[0].file, "chapter-01-the-start.wav");
    assert!((index.chapters[0].duration_seconds - 0.2).abs() < 1e-9);
    assert!((index.chapters[1].start_seconds - 0.7).abs() < 1e-9);
    let full = index.full.unwrap();
    assert_eq!(full.file, "book.wav");
    assert!((full.duration_seconds - (0.2 + 0.1 + 0.1 + 2.0 * 0.5)).abs() < 1e-9);
    assert!(
        (audio::probe(&out.join("book/book.wav")).unwrap() - full.duration_seconds).abs() < 1e-6
    );
    let cue = fs::read_to_string(out.join("book/book.cue")).unwrap();
    assert!(cue.contains("FILE \"book.wav\" WAVE"));
    assert!(cue.contains("TRACK 02 AUDIO\n    TITLE \"Middle\"\n    INDEX 01 00:00:5"));
    let summary = audiobook::run_summary(&run).unwrap();
    assert_eq!(summary[0]["complete"], true);
    assert_eq!(summary[0]["chapters"][0]["parts"], 2);
    // Book audio is the last artifact; every artifact points at its published copy.
    let record = run.jobs.iter().find(|j| j.input.id == "book").unwrap();
    assert_eq!(
        record
            .artifacts
            .iter()
            .map(|a| a.format.as_str())
            .collect::<Vec<_>>(),
        ["json", "cue", "wav"]
    );
    assert!(record.artifacts.iter().all(|a| a.path.starts_with(&out)));
    // Another full run makes no provider requests and refuses to overwrite.
    let error = engine::submit(
        t.path(),
        vec![resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full))],
        Limits::default(),
        Submit {
            output_dir: Some(out.clone()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("--overwrite"));
}

#[cfg(feature = "mp3")]
#[tokio::test]
async fn mp3_book_has_id3_chapters_matching_the_index() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let book = resolve(t.path(), small(AudioFormat::Mp3, OutputMode::Full));
    let out = t.path().join("out");
    let run = engine::submit(
        t.path(),
        vec![book],
        Limits::default(),
        Submit {
            output_dir: Some(out.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let run = engine::work(t.path(), &run.id, Fake::new(), false)
        .await
        .unwrap();
    assert_eq!(
        run.state,
        State::Succeeded,
        "{:?}",
        run.jobs.iter().map(|j| &j.error).collect::<Vec<_>>()
    );
    let dir = out.join("book");
    assert!(dir.join("chapter-02-middle.mp3").is_file());
    let tag = id3::Tag::read_from_path(dir.join("book.mp3")).unwrap();
    use id3::TagLike;
    assert_eq!(tag.album(), Some("Test Book"));
    assert_eq!(tag.artist(), Some("A. Writer"));
    let chapters: Vec<_> = tag.chapters().collect();
    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[1].start_time, 700);
    let toc: Vec<_> = tag.tables_of_contents().collect();
    assert!(toc[0].top_level && toc[0].ordered && toc[0].elements.len() == 3);
    let chapter = id3::Tag::read_from_path(dir.join("chapter-02-middle.mp3")).unwrap();
    assert_eq!(chapter.title(), Some("Middle"));
    assert_eq!(
        (chapter.track(), chapter.total_tracks()),
        (Some(2), Some(3))
    );
    let index: ChapterIndex =
        serde_json::from_slice(&fs::read(dir.join("book.chapters.json")).unwrap()).unwrap();
    assert_eq!(index.format, AudioFormat::Mp3);
    // CBR MP3 decodes to at least the PCM duration (LAME adds encoder delay/padding).
    let decoded = audio::probe(&dir.join("book.mp3")).unwrap();
    assert!(
        decoded >= index.full.unwrap().duration_seconds - 0.01,
        "{decoded}"
    );
}

#[tokio::test]
async fn chapter_subsets_then_full_run_reuse_every_part() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    project::init(t.path(), &["none".into()]).unwrap();
    let mut subset = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Chapters));
    audiobook::select(&mut subset, &["middle".into()]).unwrap();
    assert!(
        audiobook::select(&mut subset.clone(), &["nope".into()])
            .unwrap_err()
            .to_string()
            .contains("this audiobook has: middle")
    );
    let fake = Fake::new();
    let run = engine::submit(t.path(), vec![subset], Limits::default(), Submit::default()).unwrap();
    assert!(run.jobs.iter().all(|j| j.input.id != "book"));
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(run.state, State::Succeeded);
    assert_eq!(fake.calls(), 1);
    let summary = audiobook::run_summary(&run).unwrap();
    assert_eq!(summary[0]["selected"], serde_json::json!(["middle"]));
    assert_eq!(summary[0]["complete"], false);
    // The full run calls only the chapters not produced yet.
    let book = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Chapters));
    let preflight = engine::audiobook_preflight(
        t.path(),
        std::slice::from_ref(&book),
        &Limits::default(),
        false,
        None,
    )
    .unwrap()
    .check()
    .unwrap()
    .unwrap();
    assert_eq!(preflight["cached_requests"], 1);
    assert_eq!(preflight["requests"], 3);
    let run = engine::submit(t.path(), vec![book], Limits::default(), Submit::default()).unwrap();
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(run.state, State::Succeeded);
    assert_eq!(fake.calls(), 4);
    let record = run.jobs.iter().find(|j| j.input.id == "book").unwrap();
    assert_eq!(record.artifacts.len(), 1);
    assert_eq!(record.artifacts[0].format, "json");
    let index: ChapterIndex =
        serde_json::from_slice(&fs::read(&record.artifacts[0].path).unwrap()).unwrap();
    assert!(index.full.is_none());
    assert_eq!(index.chapters.len(), 3);
    // Unchanged leaves are reused once more for free.
    let again = engine::submit(
        t.path(),
        vec![resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full))],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let again = engine::work(t.path(), &again.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(again.state, State::Succeeded);
    assert_eq!(fake.calls(), 4);
}

#[tokio::test]
async fn preflight_blocks_oversized_books_before_any_run_exists() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let book = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full));
    let limits = Limits {
        requests: 3,
        speech_chars: 100,
        ..Limits::default()
    };
    let error = engine::submit(t.path(), vec![book], limits, Submit::default()).unwrap_err();
    assert_eq!(error.exit, 6);
    let message = error.to_string();
    assert!(message.contains("needs 4 provider requests"), "{message}");
    assert!(
        message.contains("limits.requests to at least 4"),
        "{message}"
    );
    assert!(
        message.contains("limits.speech_chars to at least"),
        "{message}"
    );
    assert!(message.contains("No request was made"));
    assert!(engine::list(t.path()).unwrap().is_empty());
}

#[tokio::test]
async fn failed_part_blocks_the_book_and_keeps_other_chapters() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let mut j = job(t.path(), AudiobookOptions::default());
    fs::write(
        t.path().join("book.md"),
        "# One\nFine words.\n# Two\nFAIL here.\n# Three\nMore fine words.\n",
    )
    .unwrap();
    j.audiobook = None;
    let book = validation::resolve(&j, t.path(), &config()).unwrap();
    let out = t.path().join("out");
    let run = engine::submit(
        t.path(),
        vec![book],
        Limits::default(),
        Submit {
            output_dir: Some(out.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let fake = Fake::new();
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(run.state, State::Partial);
    let state = |id: &str| run.jobs.iter().find(|j| j.input.id == id).unwrap().state;
    assert_eq!(state("book--one"), State::Succeeded);
    assert_eq!(state("book--two"), State::Blocked);
    assert_eq!(state("book"), State::Blocked);
    assert!(out.join("book/chapter-01-one.wav").is_file());
    assert!(!out.join("book/book.wav").exists());
    let summary = audiobook::run_summary(&run).unwrap();
    assert_eq!(summary[0]["missing"], serde_json::json!(["two"]));
    // Resume retries only the failed part; successful parts are not requested again.
    let before = fake.calls();
    engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(fake.calls(), before + 1);
}

#[tokio::test]
async fn chapter_revision_regenerates_only_that_chapter_and_reassembles() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let book = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full));
    let run = engine::submit(t.path(), vec![book], Limits::default(), Submit::default()).unwrap();
    let fake = Fake::new();
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(run.state, State::Succeeded);
    assert_eq!(fake.calls(), 4);
    let chapter = run
        .jobs
        .iter()
        .find(|j| j.input.id == "book--the-start")
        .unwrap();
    let artifact = chapter.artifacts.last().unwrap().id.clone();
    let (revision, method) = framechord::cli::prepare_revision(
        t.path(),
        &artifact,
        "Slower, with more warmth.",
        None,
        None,
    )
    .unwrap();
    assert_eq!(method, "regenerate_chapter_and_assemble");
    let revised = engine::work(t.path(), &revision.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(revised.state, State::Succeeded);
    // Both parts of the revised chapter are new requests; the other chapters are cached.
    assert_eq!(fake.calls(), 6);
    let cached: Vec<_> = revised
        .jobs
        .iter()
        .filter(|j| j.cached)
        .map(|j| j.input.id.clone())
        .collect();
    assert_eq!(
        cached,
        ["book--middle--part-0001", "book--the-end--part-0001"]
    );
    assert!(
        revised
            .jobs
            .iter()
            .filter(|j| j.input.id.starts_with("book--the-start--"))
            .all(|j| j.input.prompt.contains("Slower, with more warmth."))
    );
    let error = framechord::cli::prepare_revision(
        t.path(),
        &artifact,
        "x",
        None,
        Some(framechord::cli::Target::Turn("the-start")),
    )
    .unwrap_err();
    assert!(error.to_string().contains("--chapter"));
}

#[test]
fn streamed_assembly_matches_in_memory_assembly() {
    let t = tempfile::tempdir().unwrap();
    let clips: Vec<(String, Vec<f32>)> = vec![
        (
            "a".into(),
            (0..2400).map(|i| (i as f32 / 40.0).sin() * 0.3).collect(),
        ),
        (
            "b".into(),
            (0..4800).map(|i| (i as f32 / 25.0).sin() * 0.9).collect(),
        ),
    ];
    let paths: Vec<PathBuf> = clips
        .iter()
        .map(|(id, samples)| {
            let p = t.path().join(format!("{id}.wav"));
            fs::write(&p, audio::wav(samples).unwrap()).unwrap();
            p
        })
        .collect();
    let decoded: Vec<(String, Vec<f32>)> = paths
        .iter()
        .zip(&clips)
        .map(|(p, (id, _))| (id.clone(), audio::mono_24k(&fs::read(p).unwrap()).unwrap()))
        .collect();
    let options = Assembly {
        pause_ms: 100,
        turn_gain_db: Default::default(),
        loudness_lufs: Some(-20.0),
    };
    let expected = audio::wav(&audio::assemble(&decoded, Some(&options)).unwrap()).unwrap();
    let target = t.path().join("streamed.wav");
    let mut writer = audio::Writer::new(Some(fs::File::create(&target).unwrap()), None).unwrap();
    audio::assemble_files(&paths, 100, Some(-20.0), &mut writer, &|| false).unwrap();
    writer.finish().unwrap();
    let streamed = fs::read(&target).unwrap();
    assert_eq!(streamed.len(), expected.len());
    let difference = streamed
        .iter()
        .zip(&expected)
        .filter(|(a, b)| a != b)
        .count();
    // Float summation order in the loudness meter may differ by one LSB on a few samples.
    assert!(difference < streamed.len() / 100, "{difference}");
    assert!(audio::wav_capacity() > 24 * 3600 * 24000);
    assert!(audio::wav_capacity() < 25 * 3600 * 24000);
}

#[test]
fn audiobook_parents_are_not_priced_and_parts_are_priced_once() {
    let t = tempfile::tempdir().unwrap();
    let book = resolve(t.path(), small(AudioFormat::Wav, OutputMode::Full));
    let table = pricing::bundled();
    let record = |input: Effective| JobRecord {
        fingerprint: String::new(),
        input,
        state: State::Succeeded,
        attempts: 1,
        artifacts: vec![],
        error: None,
        request_id: None,
        usage: None,
        cached: false,
        retry_not_before: None,
        duration_accounted: true,
    };
    assert_eq!(
        pricing::job_fields(&table, &record(book.clone())).1,
        "not_estimated"
    );
    assert_eq!(
        pricing::job_fields(&table, &record(book.turns[0].clone())).1,
        "not_estimated"
    );
    assert_ne!(
        pricing::job_fields(&table, &record(book.turns[0].turns[0].clone())).1,
        "not_estimated"
    );
}

#[test]
fn bundled_audiobook_templates_validate_offline() {
    let t = tempfile::tempdir().unwrap();
    project::init(t.path(), &["none".into()]).unwrap();
    let templates = t.path().join("framechord/templates");
    // Google is the default preset for new CLI projects.
    let cfg = framechord::setup::recommended();
    for name in ["audiobook.yaml", "audiobook-chapters.yaml"] {
        let j: Job = framechord::storage::yaml(&templates.join(name)).unwrap();
        let book = validation::resolve(&j, &templates, &cfg).unwrap();
        assert_eq!(book.turns.len(), 2, "{name}");
    }
}

#[tokio::test]
async fn podcast_turn_revision_directs_every_part_of_a_long_turn() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let selection = Selection {
        provider: Provider::Openai,
        model: "gpt-4o-mini-tts".into(),
    };
    let speech = |id: &str, text: Option<&str>| Effective {
        assembly: None,
        id: id.into(),
        media: Media::Speech,
        selection: selection.clone(),
        prompt: String::new(),
        text: text.map(Into::into),
        locale: None,
        settings: Settings {
            voice: Some("alloy".into()),
            ..Settings::default()
        },
        character_hash: None,
        references: vec![],
        depends_on: vec![],
        output: None,
        turns: vec![],
        audiobook: None,
    };
    let mut long = speech("host", Some("one two"));
    long.turns = vec![
        speech("part-0001", Some("one ")),
        speech("part-0002", Some("two")),
    ];
    let mut podcast = speech("show", None);
    podcast.media = Media::Podcast;
    podcast.settings = Settings::default();
    podcast.turns = vec![long, speech("guest", Some("Hi."))];
    let run = engine::submit(
        t.path(),
        vec![podcast],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let fake = Fake::new();
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(run.state, State::Succeeded);
    let combined = run
        .jobs
        .last()
        .unwrap()
        .artifacts
        .last()
        .unwrap()
        .id
        .clone();
    let (revision, _) = framechord::cli::prepare_revision(
        t.path(),
        &combined,
        "Brighter.",
        None,
        Some(framechord::cli::Target::Turn("host")),
    )
    .unwrap();
    let parts: Vec<_> = revision
        .jobs
        .iter()
        .filter(|j| j.input.id.contains("--host--part-"))
        .collect();
    assert_eq!(parts.len(), 2);
    assert!(parts.iter().all(|j| j.input.prompt.contains("Brighter.")));
    let revised = engine::work(t.path(), &revision.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(revised.state, State::Succeeded);
    // Two new part requests; the guest turn is reused from cache.
    assert_eq!(fake.calls(), 5);
}

#[test]
fn cli_reports_chapter_table_schema_notices_and_refuses_direct_audiobooks() {
    use assert_cmd::Command;
    let t = tempfile::tempdir().unwrap();
    let bin = || {
        let mut c = Command::cargo_bin("framechord").unwrap();
        c.arg("--project").arg(t.path()).arg("--json");
        c
    };
    bin()
        .args(["init", "--agents", "none", "--non-interactive"])
        .assert()
        .success();
    let dir = t.path().join("framechord/jobs/book");
    fs::create_dir_all(&dir).unwrap();
    let templates = t.path().join("framechord/templates");
    for name in ["manuscript.md", "narration.md"] {
        fs::copy(templates.join(name), dir.join(name)).unwrap();
    }
    fs::copy(templates.join("audiobook.yaml"), dir.join("job.yaml")).unwrap();
    let out = bin()
        .args(["validate", "--kind", "job"])
        .arg(dir.join("job.yaml"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let book = &value["data"]["audiobook"];
    assert_eq!(book["totals"]["chapters"], 2);
    assert_eq!(book["chapters"][1]["id"], "chapter-two");
    assert_eq!(book["budget"]["fits"], true);
    let out = bin().args(["schema", "chapters"]).output().unwrap();
    assert!(out.status.success());
    let out = bin().arg("notices").output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("LAME"));
    let out = bin()
        .args(["generate", "audiobook", "--text", "Hello", "--dry-run"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("job manifest"));
    let out = bin()
        .args(["generate", "--job"])
        .arg(dir.join("job.yaml"))
        .args(["--chapter", "nope", "--dry-run"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .contains("this audiobook has: chapter-one, chapter-two")
    );
}

#[test]
fn planned_audiobooks_must_narrate_the_selected_source_exactly() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("book.md");
    fs::write(&source, MANUSCRIPT).unwrap();
    let plan = |job: &str, extra: &str| {
        let stage = tempfile::tempdir().unwrap();
        let inventory = framechord::planning::prepare_sources(
            stage.path(),
            serde_json::json!([]),
            std::slice::from_ref(&source),
        )
        .unwrap();
        assert_eq!(inventory[0]["path"], "_inputs/sources/0-book.md");
        assert!(inventory[0].get("content").is_none());
        assert!(inventory[0].get("headings").is_none());
        let mut files = vec![serde_json::json!({"path": "job.yaml", "content": job})];
        if !extra.is_empty() {
            files.push(serde_json::json!({"path": "chapter.md", "content": extra}));
        }
        let content = serde_json::json!({ "files": files }).to_string();
        framechord::planning::stage(&content, "job", stage.path(), &config()).map(|_| ())
    };
    plan(
        "schema_version: 2\nid: book\nmedia: audiobook\nmanuscript: _inputs/sources/0-book.md\n",
        "",
    )
    .unwrap();
    let error = plan(
        "schema_version: 2\nid: book\nmedia: audiobook\nchapters:\n  - id: one\n    text_file: chapter.md\n",
        "A paraphrase.",
    )
    .unwrap_err();
    assert!(error.to_string().contains("exactly"), "{error}");
    // Every selected source must be narrated: leaving one out is rejected.
    let second = t.path().join("two.md");
    fs::write(&second, "Second part.").unwrap();
    let stage = tempfile::tempdir().unwrap();
    framechord::planning::prepare_sources(
        stage.path(),
        serde_json::json!([]),
        &[source.clone(), second],
    )
    .unwrap();
    let partial = serde_json::json!({"files": [{"path": "job.yaml", "content":
        "schema_version: 2\nid: book\nmedia: audiobook\nchapters:\n  - id: one\n    text_file: _inputs/sources/0-book.md\n"}]})
    .to_string();
    let error = framechord::planning::stage(&partial, "job", stage.path(), &config()).unwrap_err();
    assert!(error.to_string().contains("exactly"), "{error}");
}
