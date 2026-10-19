use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use framechord::{
    assets,
    contracts::*,
    credentials,
    engine::{self, Submit},
    error::{Error, Result},
    project,
    providers::{self, Adapter, Generated},
    storage, validation,
};
use std::{
    fs,
    io::Cursor,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

static ENGINE_TESTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn png() -> Vec<u8> {
    let mut cursor = Cursor::new(vec![]);
    let image = image::RgbImage::from_pixel(2, 2, image::Rgb([20, 80, 120]));
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
    cursor.into_inner()
}
fn input(name: &str) -> Effective {
    Effective {
        assembly: None,
        id: name.into(),
        media: Media::Image,
        selection: Selection {
            provider: Provider::Openai,
            model: "gpt-image-2.5-flare".into(),
        },
        prompt: format!("A forest for {name}"),
        text: None,
        locale: Some("en-US".into()),
        settings: Settings::default(),
        character_hash: None,
        references: vec![],
        depends_on: vec![],
        output: None,
        turns: vec![],
        audiobook: None,
    }
}
struct Fake {
    calls: AtomicUsize,
    active: AtomicUsize,
    max: AtomicUsize,
    rendezvous: Option<tokio::sync::Barrier>,
}
impl Fake {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            max: AtomicUsize::new(0),
            rendezvous: None,
        }
    }
}
#[async_trait]
impl Adapter for Fake {
    async fn generate(&self, e: &Effective) -> Result<Generated> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max.fetch_max(active, Ordering::SeqCst);
        if let Some(barrier) = &self.rendezvous {
            // Require overlapping requests without relying on filesystem speed.
            tokio::time::timeout(Duration::from_secs(10), barrier.wait())
                .await
                .expect("the engine must execute two independent requests concurrently");
        }
        tokio::time::sleep(Duration::from_millis(30)).await;
        self.active.fetch_sub(1, Ordering::SeqCst);
        if e.id == "unknown" {
            return Err(Error::unknown("Ambiguous provider timeout"));
        }
        if e.id == "failed" {
            return Err(Error::new("provider_rejected", "Mock rejection", 3));
        }
        let (bytes, format) = if e.media == Media::Speech {
            (framechord::audio::wav(&[0.1; 2400])?, "wav")
        } else {
            (png(), "png")
        };
        Ok(Generated {
            bytes,
            format: format.into(),
            request_id: Some(format!("remote-{}", e.id)),
            usage: Some(serde_json::json!({"total_tokens":1})),
        })
    }
}
#[test]
fn offline_init_preserves_credentials_user_content_and_skills() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    fs::write(root.join(".env"), "PRIVATE_TEST_VALUE=keep-me\n").unwrap();
    project::init(root, &["codex".into(), "claude".into()]).unwrap();
    let config = root.join("framechord/config.yaml");
    let mut c = project::config(root).unwrap();
    c.defaults.insert(Media::Image, input("x").selection);
    storage::save(&config, &c).unwrap();
    let visual = root.join("framechord/templates/visual.md");
    fs::write(&visual, "User authored visual identity").unwrap();
    let skill = root.join(".claude/skills/framechord/SKILL.md");
    fs::write(&skill, "Modified local skill").unwrap();
    project::init(root, &["codex".into(), "claude".into()]).unwrap();
    assert_eq!(
        fs::read_to_string(root.join(".env")).unwrap(),
        format!(
            "PRIVATE_TEST_VALUE=keep-me\n{}",
            framechord::credentials::ENV_TEMPLATE
        )
    );
    assert_eq!(
        fs::read_to_string(visual).unwrap(),
        "User authored visual identity"
    );
    assert_eq!(fs::read_to_string(skill).unwrap(), "Modified local skill");
    assert!(
        project::config(root)
            .unwrap()
            .defaults
            .contains_key(&Media::Image)
    );
    assert!(
        root.join(".agents/skills/framechord/references/commands.md")
            .is_file()
    );
    assert!(
        root.join(".agents/skills/framechord/references/schemas/job.json")
            .is_file()
    );
}
#[test]
fn initialization_preserves_an_incomplete_existing_persona() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("framechord/characters/persona");
    fs::create_dir_all(&p).unwrap();
    fs::write(p.join("persona.md"), "original").unwrap();
    project::init(t.path(), &[]).unwrap();
    assert_eq!(
        fs::read_to_string(p.join("persona.md")).unwrap(),
        "original"
    );
    assert!(!p.join("character.yaml").exists());
}
#[test]
fn user_scope_install_uses_the_same_embedded_bundle() {
    let home = tempfile::tempdir().unwrap();
    project::install(home.path(), &["codex".into(), "claude".into()]).unwrap();
    assert_eq!(
        fs::read(home.path().join(".agents/skills/framechord/SKILL.md")).unwrap(),
        fs::read(home.path().join(".claude/skills/framechord/SKILL.md")).unwrap()
    );
    let export = tempfile::tempdir().unwrap();
    assets::skill(export.path()).unwrap();
    assert!(export.path().join("templates/infographic.yaml").exists());
    project::install_scoped(
        home.path(),
        &["opencode".into(), "copilot".into()],
        false,
        true,
    )
    .unwrap();
    let shared = fs::read(home.path().join(".agents/skills/framechord/SKILL.md")).unwrap();
    for native in [
        ".config/opencode/skills/framechord/SKILL.md",
        ".copilot/skills/framechord/SKILL.md",
    ] {
        assert_eq!(
            fs::read(home.path().join(native)).unwrap(),
            shared,
            "{native}"
        );
    }
}
#[test]
fn invalid_agent_target_causes_no_partial_initialization() {
    let t = tempfile::tempdir().unwrap();
    let error = project::init(t.path(), &["codex".into(), "unknown".into()]).unwrap_err();
    assert_eq!(error.code, "invalid_input");
    assert!(
        error.message.contains("cursor, gemini, opencode, copilot"),
        "{}",
        error.message
    );
    assert!(error.message.contains("none"));
    assert!(!t.path().join("framechord").exists());
}
#[test]
fn native_agent_targets_dedupe_accept_none_and_install_only_their_directories() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    let agents = ["copilot".to_string(), "cursor".into(), "copilot".into()];
    project::init(root, &agents).unwrap();
    assert!(root.join(".github/skills/framechord/SKILL.md").is_file());
    assert!(root.join(".cursor/skills/framechord/SKILL.md").is_file());
    assert!(!root.join(".agents").exists());
    assert!(!root.join(".claude").exists());
    let second = project::init(root, &agents).unwrap();
    let preserved: Vec<String> =
        serde_json::from_value(second["existing_preserved"].clone()).unwrap();
    assert_eq!(
        preserved.iter().filter(|p| p.ends_with("SKILL.md")).count(),
        2,
        "repeated targets install once: {preserved:?}"
    );
    let none = tempfile::tempdir().unwrap();
    project::init(none.path(), &["none".into()]).unwrap();
    for dir in [
        ".agents",
        ".claude",
        ".cursor",
        ".gemini",
        ".opencode",
        ".github",
    ] {
        assert!(!none.path().join(dir).exists(), "{dir} must not be created");
    }
    assert!(none.path().join("framechord/config.yaml").is_file());
    let mixed = tempfile::tempdir().unwrap();
    assert!(project::init(mixed.path(), &["none".into(), "codex".into()]).is_err());
    assert!(!mixed.path().join("framechord").exists());
    let blank = tempfile::tempdir().unwrap();
    assert!(
        project::init(blank.path(), &["".into()]).is_err(),
        "a blank target is a mistake, not a request for no skills"
    );
    assert!(!blank.path().join("framechord").exists());
}
fn make_character(root: &Path) -> std::path::PathBuf {
    let path = project::add_persona(root, "mira").unwrap();
    let mut c: Character = storage::yaml(&path).unwrap();
    c.visual = Some(Visual {
        settings: Settings::default(),
        origins: Default::default(),
        description: "visual.md".into(),
        selection: input("x").selection,
        references: vec![],
    });
    c.voice = Some(Voice {
        samples: vec![],
        description: "voice.md".into(),
        selection: Selection {
            provider: Provider::Openai,
            model: "gpt-4o-mini-tts".into(),
        },
        binding: Some("alloy".into()),
        references: vec![],
    });
    storage::save(&path, &c).unwrap();
    path
}
#[test]
fn portable_character_keeps_dependencies_and_detects_id_conflicts() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    fs::create_dir(&source).unwrap();
    let c = make_character(&source);
    let img = t.path().join("chosen.png");
    fs::write(&img, png()).unwrap();
    project::accept(&c, &img, "image").unwrap();
    let bundle = t.path().join("bundle");
    project::bundle(&c, &bundle, None).unwrap();
    fs::remove_dir_all(&source).unwrap();
    let dest = t.path().join("destination");
    let imported = project::import(&dest, &bundle, None).unwrap();
    let definition = validation::character(&imported).unwrap();
    assert_eq!(definition.voice.unwrap().binding.as_deref(), Some("alloy"));
    assert!(
        imported
            .parent()
            .unwrap()
            .join(&definition.visual.unwrap().references[0])
            .is_file()
    );
    assert!(project::import(&dest, &bundle, None).is_err());
    assert!(project::import(&dest, &bundle, Some("mira-copy")).is_ok());
}
#[test]
fn importing_escape_paths_is_rejected() {
    let t = tempfile::tempdir().unwrap();
    let c = make_character(t.path());
    let mut definition: Character = storage::yaml(&c).unwrap();
    let outside = t.path().join("outside.md");
    fs::write(&outside, "Private external dependency").unwrap();
    definition.persona = outside;
    storage::save(&c, &definition).unwrap();
    assert!(project::import(&t.path().join("dest"), c.parent().unwrap(), None).is_err());
}
#[test]
fn credential_lookup_is_root_scoped_and_environment_wins() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join(".env"), "OPENAI_API_KEY=parent-secret\n").unwrap();
    let child = t.path().join("child");
    fs::create_dir(&child).unwrap();
    assert!(credentials::resolve_with("OPENAI_API_KEY", &child, |_| None).is_err());
    fs::write(child.join(".env"), "OPENAI_API_KEY=local-secret\n").unwrap();
    assert_eq!(
        credentials::resolve_with("OPENAI_API_KEY", &child, |_| Some("process-secret".into()))
            .unwrap(),
        "process-secret"
    );
    assert_eq!(
        credentials::resolve_with("OPENAI_API_KEY", &child, |_| None).unwrap(),
        "local-secret"
    );
    let err =
        credentials::resolve_with("OPENAI_API_KEY", &child, |_| Some(String::new())).unwrap_err();
    assert!(!err.to_string().contains("secret"));
}
#[test]
fn schemas_and_parser_reject_unknown_fields_and_versions() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("config.yaml");
    fs::write(&path, "schema_version: 1\napi_key: secret\n").unwrap();
    assert!(storage::yaml::<Config>(&path).is_err());
    let cfg = Config {
        schema_version: 99,
        ..Default::default()
    };
    assert!(validation::config(&cfg).is_err());
    let schema = framechord::cli::schema("job").unwrap();
    assert_eq!(schema["additionalProperties"], false);
    assert!(schema["$schema"].as_str().unwrap().contains("2020-12"));
}
#[test]
fn capability_rejection_includes_unknown_settings_and_voice_references() {
    let mut e = input("x");
    e.selection.provider = Provider::Fish;
    assert!(providers::validate(&e).is_err());
    e = input("x");
    e.settings.seed = Some(1);
    assert!(providers::validate(&e).is_err());
    e = input("x");
    e.references.push(Reference {
        blob: None,
        origin: None,
        transcript: None,
        consent_hash: None,
        role: ReferenceRole::Content,
        name: "voice.wav".into(),
        mime: "audio/wav".into(),
        sha256: "x".into(),
        base64: String::new(),
    });
    assert!(providers::validate(&e).is_err());
}
#[test]
fn infographic_compiler_retains_exact_copy_units_and_values() {
    let t = tempfile::tempdir().unwrap();
    assets::templates(t.path()).unwrap();
    let p = t.path().join("infographic.yaml");
    let mut spec: Infographic = storage::yaml(&p).unwrap();
    spec.sections[0].values = vec![InfoValue {
        label: "Example estimate".into(),
        value: "03.50".into(),
        unit: Some("kg".into()),
        source: Some("fictional demonstration".into()),
    }];
    storage::save(&p, &spec).unwrap();
    let (_, compiled, _) = validation::infographic(&p).unwrap();
    assert!(compiled.contains("03.50"));
    assert!(compiled.contains("kg"));
    assert!(compiled.contains(&spec.sections[0].copy));
}
#[test]
fn cache_fingerprint_changes_for_content_reference_voice_and_locale() {
    let e = input("x");
    let baseline = validation::fingerprint(&e).unwrap();
    let mut altered = e.clone();
    altered.output = Some("elsewhere.png".into());
    altered.id = "other".into();
    assert_eq!(baseline, validation::fingerprint(&altered).unwrap());
    altered.prompt.push('!');
    assert_ne!(baseline, validation::fingerprint(&altered).unwrap());
    altered = e.clone();
    altered.locale = Some("es-419".into());
    assert_ne!(baseline, validation::fingerprint(&altered).unwrap());
    altered = e.clone();
    altered.references.push(Reference {
        blob: None,
        origin: None,
        transcript: None,
        consent_hash: None,
        role: ReferenceRole::Content,
        name: "face.png".into(),
        mime: "image/png".into(),
        sha256: storage::hash(&png()),
        base64: STANDARD.encode(png()),
    });
    assert_ne!(baseline, validation::fingerprint(&altered).unwrap());
    altered = e.clone();
    altered.settings.voice = Some("voice-2".into());
    assert_ne!(baseline, validation::fingerprint(&altered).unwrap());
}
#[test]
fn graph_rejects_cycles_missing_dependencies_and_output_collisions() {
    let mut a = input("a");
    let mut b = input("b");
    a.depends_on = vec!["b".into()];
    b.depends_on = vec!["a".into()];
    assert!(validation::graph(&[a.clone(), b.clone()]).is_err());
    b.depends_on.clear();
    a.depends_on = vec!["missing".into()];
    assert!(validation::graph(&[a.clone(), b.clone()]).is_err());
    a.depends_on.clear();
    a.output = Some("same.png".into());
    b.output = a.output.clone();
    assert!(validation::graph(&[a, b]).is_err());
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actual_parallel_execution_respects_limits_and_isolates_outputs() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let limits = Limits {
        images: 12,
        ..Default::default()
    };
    let run = engine::submit(
        t.path(),
        (0..6).map(|i| input(&format!("job-{i}"))).collect(),
        limits,
        Submit::default(),
    )
    .unwrap();
    let fake = Arc::new(Fake {
        rendezvous: Some(tokio::sync::Barrier::new(2)),
        ..Fake::new()
    });
    let result = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(result.state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 6);
    assert_eq!(fake.max.load(Ordering::SeqCst), 2);
    let paths: std::collections::BTreeSet<_> =
        result.jobs.iter().map(|j| &j.artifacts[0].path).collect();
    assert_eq!(paths.len(), 6);
    for j in result.jobs {
        assert_eq!(
            storage::hash(&fs::read(&j.artifacts[0].path).unwrap()),
            j.artifacts[0].sha256
        );
        assert!(j.request_id.is_some());
    }
}
#[tokio::test]
async fn dependencies_wait_for_success_and_block_on_failure() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let mut dependent = input("dependent");
    dependent.depends_on = vec!["failed".into()];
    let fake = Arc::new(Fake::new());
    let run = engine::submit(
        t.path(),
        vec![input("failed"), dependent, input("success")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let result = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(result.state, State::Partial);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    assert_eq!(result.jobs[1].state, State::Blocked);
    assert!(result.jobs[2].artifacts[0].path.is_file());
}
#[tokio::test]
async fn cache_reuses_verified_results_and_fresh_bypasses_cache() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let first = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    engine::work(t.path(), &first.id, fake.clone(), false)
        .await
        .unwrap();
    let second = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let second = engine::work(t.path(), &second.id, fake.clone(), false)
        .await
        .unwrap();
    assert!(second.jobs[0].cached);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    assert!(
        !engine::run_path(t.path(), &second.id)
            .unwrap()
            .join("budget.yaml")
            .exists()
    );
    let third = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit {
            fresh: true,
            ..Default::default()
        },
    )
    .unwrap();
    engine::work(t.path(), &third.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn corrupt_cache_artifact_is_not_reused() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let run = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let run = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    fs::write(&run.jobs[0].artifacts[0].path, b"corrupt").unwrap();
    let next = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    engine::work(t.path(), &next.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn resume_preserves_successes_and_does_not_retry_unknown_without_flag() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let run = engine::submit(
        t.path(),
        vec![input("good"), input("unknown")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let r = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(engine::exit(&r), 7);
    let id = r.jobs[0].artifacts[0].id.clone();
    let r = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    assert_eq!(r.jobs[0].artifacts[0].id, id);
    engine::work(t.path(), &run.id, fake.clone(), true)
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 3);
}
#[tokio::test]
async fn worker_loss_marks_inflight_unknown_and_does_not_submit_it() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let mut run = engine::submit(
        t.path(),
        vec![input("lost"), input("queued")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    run.jobs[0].state = State::Running;
    run.state = State::Running;
    storage::save(
        &engine::run_path(t.path(), &run.id)
            .unwrap()
            .join("run.yaml"),
        &run,
    )
    .unwrap();
    let fake = Arc::new(Fake::new());
    let result = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(result.jobs[0].state, State::Unknown);
    assert_eq!(result.jobs[1].state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn budgets_reserve_before_calls_and_limits_take_priority_over_partial() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let limits = Limits {
        images: 1,
        ..Default::default()
    };
    let run = engine::submit(
        t.path(),
        vec![input("one"), input("two")],
        limits,
        Submit::default(),
    )
    .unwrap();
    let fake = Arc::new(Fake::new());
    let r = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(engine::exit(&r), 6);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    assert_eq!(r.jobs[0].state, State::Succeeded);
    engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn revision_budget_group_is_shared_and_bounded() {
    let t = tempfile::tempdir().unwrap();
    let l = Limits {
        revisions: 1,
        ..Default::default()
    };
    let r = engine::submit(t.path(), vec![input("a")], l.clone(), Submit::default()).unwrap();
    engine::reserve(t.path(), &r.budget_group, &l, None, true).unwrap();
    assert_eq!(
        engine::reserve(t.path(), &r.budget_group, &l, None, true)
            .unwrap_err()
            .exit,
        6
    );
}
#[test]
fn submission_idempotency_detects_input_conflicts() {
    let t = tempfile::tempdir().unwrap();
    let r = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit {
            request_id: Some("request-a".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let repeat = engine::submit(
        t.path(),
        vec![input("a")],
        Limits::default(),
        Submit {
            request_id: Some("request-a".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(r.id, repeat.id);
    assert!(
        engine::submit(
            t.path(),
            vec![input("b")],
            Limits::default(),
            Submit {
                request_id: Some("request-a".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn rust_audio_assembly_has_known_duration_rate_and_channels() {
    let bytes = framechord::audio::wav(&[0.2; 4800]).unwrap();
    let (samples, rate) = framechord::audio::decode(&bytes).unwrap();
    assert_eq!(rate, 24000);
    assert_eq!(samples.len(), 4800);
    let reader = hound::WavReader::new(Cursor::new(bytes)).unwrap();
    assert_eq!(reader.spec().channels, 1);
    assert_eq!(reader.spec().bits_per_sample, 16);
    let mut bytes = Cursor::new(vec![]);
    {
        let mut w = hound::WavWriter::new(
            &mut bytes,
            hound::WavSpec {
                channels: 2,
                sample_rate: 48000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();
        for _ in 0..4800 {
            w.write_sample(1000_i16).unwrap();
            w.write_sample(3000_i16).unwrap();
        }
        w.finalize().unwrap();
    }
    let converted = framechord::audio::mono_24k(&bytes.into_inner()).unwrap();
    assert_eq!(converted.len(), 2400);
    assert!((converted[1200] - 2000.0 / 32768.0).abs() < 0.002);
}
#[tokio::test]
async fn podcast_keeps_ordered_turns_and_combined_wav() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let mut parent = input("podcast");
    parent.media = Media::Podcast;
    parent.prompt.clear();
    for name in ["alice", "bob"] {
        let mut turn = input(name);
        turn.media = Media::Speech;
        turn.selection.model = "gpt-4o-mini-tts".into();
        turn.text = Some("Hello, welcome.".into());
        turn.settings.voice = Some("alloy".into());
        parent.turns.push(turn);
    }
    let r = engine::submit(t.path(), vec![parent], Limits::default(), Submit::default()).unwrap();
    let fake = Arc::new(Fake::new());
    let r = engine::work(t.path(), &r.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(r.state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    assert_eq!(r.jobs.len(), 3);
    assert_eq!(r.jobs[0].artifacts.len(), 2);
    assert_eq!(r.jobs[2].artifacts[0].duration_seconds, Some(0.2));
}

#[test]
fn completed_character_examples_validate_and_import_as_portable_bundles() {
    let t = tempfile::tempdir().unwrap();
    assets::templates(t.path()).unwrap();
    for name in ["narrator", "mascot", "combined"] {
        let dir = t.path().join("examples").join(name);
        validation::character(&dir.join("character.yaml")).unwrap();
        project::import(&t.path().join("destination"), &dir, None).unwrap();
    }
}
#[test]
fn accepted_identity_is_reused_and_overrides_unrelated_project_defaults() {
    let t = tempfile::tempdir().unwrap();
    let path = make_character(t.path());
    let image = t.path().join("chosen.png");
    fs::write(&image, png()).unwrap();
    project::accept(&path, &image, "image").unwrap();
    let brief = t.path().join("scene.md");
    fs::write(&brief, "A new pose in a forest.").unwrap();
    let mut cfg = Config::default();
    cfg.defaults.insert(
        Media::Image,
        Selection {
            provider: Provider::Google,
            model: "gemini-nano-banana-2.1".into(),
        },
    );
    let job = Job {
        cast: vec![],
        assembly: None,
        voice_samples: vec![],
        schema_version: 1,
        id: "scene".into(),
        media: Media::Image,
        selection: None,
        brief: Some(brief),
        text_file: None,
        character: Some(path.clone()),
        spec: None,
        references: vec![],
        turns: vec![],
        settings: Settings::default(),
        locale: None,
        depends_on: vec![],
        output: None,
        manuscript: None,
        split: None,
        chapters: vec![],
        audiobook: None,
    };
    let effective = validation::resolve(&job, t.path(), &cfg).unwrap();
    assert_eq!(effective.selection.provider, Provider::Openai);
    assert_eq!(effective.references.len(), 1);
    assert!(matches!(
        effective.references[0].role,
        ReferenceRole::Identity
    ));
    assert!(effective.prompt.contains("new pose"));
    let mut conflict = job;
    conflict.selection = cfg.defaults.get(&Media::Image).cloned();
    assert!(validation::resolve(&conflict, t.path(), &cfg).is_err());
    assert_eq!(
        validation::character(&path)
            .unwrap()
            .visual
            .unwrap()
            .references
            .len(),
        1
    );
}
#[tokio::test]
async fn cached_dependencies_finish_without_requiring_a_second_resume() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let a = input("a");
    let mut b = input("b");
    b.depends_on = vec!["a".into()];
    let run = engine::submit(
        t.path(),
        vec![a.clone(), b.clone()],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    let repeat =
        engine::submit(t.path(), vec![a, b], Limits::default(), Submit::default()).unwrap();
    let repeated = engine::work(t.path(), &repeat.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(repeated.state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    assert!(repeated.jobs.iter().all(|j| j.cached));
}
#[tokio::test]
async fn revisions_preserve_original_artifacts_and_inherit_budget() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let run = engine::submit(
        t.path(),
        vec![input("image")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let original = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    let artifact = &original.jobs[0].artifacts[0];
    let old = fs::read(&artifact.path).unwrap();
    let (child, method) = framechord::cli::prepare_revision(
        t.path(),
        &artifact.id,
        "Use a lighter background.",
        None,
        None,
    )
    .unwrap();
    assert_eq!(method, "native_edit");
    assert_eq!(child.budget_group, original.budget_group);
    assert_eq!(child.jobs[0].input.references.len(), 1);
    let child = engine::work(t.path(), &child.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(child.state, State::Succeeded);
    assert_eq!(
        child.jobs[0].artifacts[0].parent.as_deref(),
        Some(artifact.id.as_str())
    );
    assert_eq!(fs::read(&artifact.path).unwrap(), old);
    let budget: Budget = storage::yaml(
        &engine::run_path(t.path(), &original.budget_group)
            .unwrap()
            .join("budget.yaml"),
    )
    .unwrap();
    assert_eq!(budget.requests, 2);
    assert_eq!(budget.revisions, 1);
}
#[tokio::test]
async fn podcast_revision_regenerates_only_the_selected_turn() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let mut parent = input("dialogue");
    parent.media = Media::Podcast;
    for name in ["alice", "bob"] {
        let mut e = input(name);
        e.media = Media::Speech;
        e.selection.model = "gpt-4o-mini-tts".into();
        e.text = Some("Hello.".into());
        e.settings.voice = Some("alloy".into());
        parent.turns.push(e);
    }
    let run = engine::submit(t.path(), vec![parent], Limits::default(), Submit::default()).unwrap();
    let original = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    let combined = &original.jobs[2].artifacts[0];
    let (revision, method) = framechord::cli::prepare_revision(
        t.path(),
        &combined.id,
        "A warmer delivery.",
        None,
        Some(framechord::cli::Target::Turn("bob")),
    )
    .unwrap();
    assert_eq!(method, "regenerate_turn_and_assemble");
    let revised = engine::work(t.path(), &revision.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(revised.state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 3);
    assert!(revised.jobs[0].cached);
    assert!(!revised.jobs[1].cached);
    assert_eq!(original.jobs[0].input.text, revised.jobs[0].input.text);
    assert_eq!(original.jobs[1].input.text, revised.jobs[1].input.text);
}
#[tokio::test]
async fn cancel_stops_queued_work_without_submitting_requests() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let run = engine::submit(
        t.path(),
        vec![input("a"), input("b")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    engine::cancel(t.path(), &run.id).unwrap();
    let cancelled = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(cancelled.state, State::Cancelled);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn stored_response_recovers_publication_without_an_extra_paid_attempt() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let mut e = input("a");
    let destination = t.path().join("destination.png");
    e.output = Some(destination.clone());
    let run = engine::submit(
        t.path(),
        vec![e],
        Limits {
            attempts: 1,
            ..Default::default()
        },
        Submit::default(),
    )
    .unwrap();
    fs::write(&destination, b"created after submission").unwrap();
    let failed = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(failed.state, State::Failed);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    fs::remove_file(&destination).unwrap();
    let recovered = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(recovered.state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        storage::hash(&fs::read(destination).unwrap()),
        recovered.jobs[0].artifacts.last().unwrap().sha256
    );
}
#[test]
fn output_directory_and_podcast_expansion_preserve_request_idempotency() {
    let t = tempfile::tempdir().unwrap();
    let e = input("image");
    let dest = t.path().join("exports");
    let first = engine::submit(
        t.path(),
        vec![e.clone()],
        Limits::default(),
        Submit {
            request_id: Some("same".into()),
            output_dir: Some(dest.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let second = engine::submit(
        t.path(),
        vec![e],
        Limits::default(),
        Submit {
            request_id: Some("same".into()),
            output_dir: Some(dest),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(first.id, second.id);
}
#[tokio::test]
async fn long_speech_is_split_exactly_and_assembled_with_no_parent_provider_call() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let script = "Clarity matters. Café 🌿. ".repeat(250);
    let source = t.path().join("article.md");
    fs::write(&source, &script).unwrap();
    let job = Job {
        cast: vec![],
        assembly: None,
        voice_samples: vec![],
        schema_version: 1,
        id: "article".into(),
        media: Media::Speech,
        selection: Some(Selection {
            provider: Provider::Openai,
            model: "gpt-4o-mini-tts".into(),
        }),
        brief: None,
        text_file: Some(source),
        character: None,
        spec: None,
        references: vec![],
        turns: vec![],
        settings: Settings {
            voice: Some("alloy".into()),
            ..Default::default()
        },
        locale: None,
        depends_on: vec![],
        output: None,
        manuscript: None,
        split: None,
        chapters: vec![],
        audiobook: None,
    };
    let effective = validation::resolve(&job, t.path(), &Config::default()).unwrap();
    assert!(effective.turns.len() > 1);
    assert_eq!(
        effective
            .turns
            .iter()
            .map(|t| t.text.as_deref().unwrap())
            .collect::<String>(),
        script
    );
    assert!(
        effective
            .turns
            .iter()
            .all(|t| t.text.as_ref().unwrap().chars().count() <= 4096)
    );
    let count = effective.turns.len();
    let fake = Arc::new(Fake::new());
    let run = engine::submit(
        t.path(),
        vec![effective],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let completed = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(completed.state, State::Succeeded);
    assert_eq!(fake.calls.load(Ordering::SeqCst), count);
    assert_eq!(completed.jobs.last().unwrap().artifacts[0].format, "wav");
}
#[test]
fn explicit_bundle_updates_replace_only_bundled_files() {
    let root = tempfile::tempdir().unwrap();
    assets::skill(root.path()).unwrap();
    let skill = root.path().join("SKILL.md");
    fs::write(&skill, "Local customization").unwrap();
    fs::write(root.path().join("personal-notes.md"), "Keep this").unwrap();
    assets::skill(root.path()).unwrap();
    assert_eq!(fs::read_to_string(&skill).unwrap(), "Local customization");
    assets::skill_with(root.path(), true).unwrap();
    assert_eq!(
        fs::read_to_string(skill).unwrap(),
        include_str!("../assets/skill/SKILL.md")
    );
    assert_eq!(
        fs::read_to_string(root.path().join("personal-notes.md")).unwrap(),
        "Keep this"
    );
}
#[tokio::test]
async fn jpeg_provider_response_preserves_original_and_publishes_png() {
    let _serial = ENGINE_TESTS.lock().await;
    struct Jpeg;
    #[async_trait]
    impl Adapter for Jpeg {
        async fn generate(&self, _: &Effective) -> Result<Generated> {
            let mut bytes = Cursor::new(vec![]);
            image::RgbImage::from_pixel(2, 2, image::Rgb([20, 80, 120]))
                .write_to(&mut bytes, image::ImageFormat::Jpeg)
                .unwrap();
            Ok(Generated {
                bytes: bytes.into_inner(),
                format: "png".into(),
                request_id: Some("jpeg-response".into()),
                usage: None,
            })
        }
    }
    let root = tempfile::tempdir().unwrap();
    let run = engine::submit(
        root.path(),
        vec![input("image")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let result = engine::work(root.path(), &run.id, Arc::new(Jpeg), false)
        .await
        .unwrap();
    assert_eq!(result.state, State::Succeeded);
    let artifacts = &result.jobs[0].artifacts;
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0].format, "jpeg");
    assert_eq!(artifacts[1].format, "png");
    assert!(
        fs::read(&artifacts[1].path)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert_eq!(artifacts[1].width, Some(2));
}

#[test]
fn planned_files_use_the_shared_validator_and_contained_dependencies() {
    let stage = tempfile::tempdir().unwrap();
    let content = serde_json::json!({"files":[
        {"path":"job.yaml","content":"schema_version: 1\nid: planned\nmedia: image\nselection:\n  provider: openai\n  model: gpt-image-1\nbrief: brief.md\n"},
        {"path":"brief.md","content":"Draw exactly three teal leaves."}
    ]}).to_string();
    let manifest =
        framechord::planning::stage(&content, "job", stage.path(), &Config::default()).unwrap();
    let job: Job = storage::yaml(&manifest).unwrap();
    let effective = validation::resolve(&job, stage.path(), &Config::default()).unwrap();
    assert_eq!(effective.prompt, "Draw exactly three teal leaves.");
    assert!(!stage.path().join("framechord/runs").exists());
}

#[test]
fn planner_rejects_escape_dependencies_and_non_authoring_files() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("outside.md"), "Private outside content").unwrap();
    let stage = root.path().join("stage");
    fs::create_dir(&stage).unwrap();
    let content = serde_json::json!({"files":[{"path":"job.yaml","content":"schema_version: 1\nid: planned\nmedia: image\nselection:\n  provider: openai\n  model: gpt-image-1\nbrief: ../outside.md\n"}]}).to_string();
    let error =
        framechord::planning::stage(&content, "job", &stage, &Config::default()).unwrap_err();
    assert!(error.to_string().contains("escapes"));
    assert!(!error.to_string().contains("Private outside content"));
    for path in [".env", "run.sh", "../outside.md", ""] {
        let stage = tempfile::tempdir().unwrap();
        let content = serde_json::json!({"files":[{"path":"brief.md","content":"Valid first file"},{"path":path,"content":"Bad file"}]}).to_string();
        assert!(
            framechord::planning::stage(&content, "job", stage.path(), &Config::default()).is_err()
        );
        assert!(!stage.path().join("brief.md").exists());
    }
}

#[test]
fn planner_checks_nested_character_dependencies_before_resolution() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("outside.md"), "Private outside content").unwrap();
    let stage = root.path().join("stage");
    fs::create_dir(&stage).unwrap();
    let content = serde_json::json!({"files":[
        {"path":"job.yaml","content":"schema_version: 1\nid: planned\nmedia: image\ncharacter: mira/character.yaml\nbrief: brief.md\n"},
        {"path":"brief.md","content":"A forest scene"},
        {"path":"mira/character.yaml","content":"schema_version: 1\nid: mira\npersona: ../../outside.md\nvisual:\n  description: visual.md\n  selection:\n    provider: openai\n    model: gpt-image-1\n"},
        {"path":"mira/visual.md","content":"Teal leaf mascot"}
    ]}).to_string();
    assert!(
        framechord::planning::stage(&content, "job", &stage, &Config::default())
            .unwrap_err()
            .to_string()
            .contains("escapes")
    );
}

#[tokio::test]
async fn cache_publication_failure_preserves_other_batch_jobs() {
    let _serial = ENGINE_TESTS.lock().await;
    let root = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let source = input("source");
    let first = engine::submit(
        root.path(),
        vec![source.clone()],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    engine::work(root.path(), &first.id, fake.clone(), false)
        .await
        .unwrap();
    let mut cached = source;
    cached.id = "cached".into();
    let destination = root.path().join("existing.png");
    cached.output = Some(destination.clone());
    let run = engine::submit(
        root.path(),
        vec![cached, input("independent")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    // The destination appears after submission, while this batch is queued.
    fs::write(&destination, b"User asset").unwrap();
    let completed = engine::work(root.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(completed.state, State::Partial);
    assert_eq!(completed.jobs[0].state, State::Failed);
    assert_eq!(completed.jobs[1].state, State::Succeeded);
    assert_eq!(fs::read(destination).unwrap(), b"User asset");
    assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn reference_validation_checks_media_bytes_before_acceptance() {
    let root = tempfile::tempdir().unwrap();
    let image = root.path().join("valid.png");
    fs::write(&image, png()).unwrap();
    assert_eq!(
        validation::reference(root.path(), Path::new("valid.png"))
            .unwrap()
            .mime,
        "image/png"
    );
    fs::write(root.path().join("wrong.jpg"), png()).unwrap();
    assert!(validation::reference(root.path(), Path::new("wrong.jpg")).is_err());
    let character = make_character(root.path());
    fs::write(root.path().join("broken.png"), b"not an image").unwrap();
    assert!(project::accept(&character, &root.path().join("broken.png"), "image").is_err());
    let saved: Character = storage::yaml(&character).unwrap();
    assert!(saved.visual.unwrap().references.is_empty());
    let audio = framechord::audio::wav(&[0.1; 2400]).unwrap();
    fs::write(root.path().join("voice.wav"), &audio).unwrap();
    assert_eq!(
        validation::reference(root.path(), Path::new("voice.wav"))
            .unwrap()
            .mime,
        "audio/wav"
    );
    fs::write(root.path().join("wrong.mp3"), &audio).unwrap();
    assert!(validation::reference(root.path(), Path::new("wrong.mp3")).is_err());
}

#[tokio::test]
async fn unsupported_ai_review_and_corrupt_manual_review_never_make_paid_requests() {
    use clap::Parser;
    let _serial = ENGINE_TESTS.lock().await;
    let root = tempfile::tempdir().unwrap();
    let fake = Arc::new(Fake::new());
    let submitted = engine::submit(
        root.path(),
        vec![input("reviewed")],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let run = engine::work(root.path(), &submitted.id, fake.clone(), false)
        .await
        .unwrap();
    let artifact = &run.jobs[0].artifacts[0];
    let config = framechord::cli::Cli::try_parse_from([
        "framechord",
        "--project",
        root.path().to_str().unwrap(),
        "review",
        "check",
        "--artifact",
        &artifact.id,
        "--ai",
        "--provider",
        "google",
        "--model",
        "gemini-3.1-flash",
    ])
    .unwrap();
    assert_eq!(
        framechord::cli::dispatch(&config).await.unwrap_err().exit,
        2
    );
    let budget: Budget = storage::yaml(
        &engine::run_path(root.path(), &run.id)
            .unwrap()
            .join("budget.yaml"),
    )
    .unwrap();
    assert_eq!(budget.requests, 1);
    let file = root.path().join("review.yaml");
    storage::save(
        &file,
        &Review {
            schema_version: 1,
            artifact: artifact.id.clone(),
            quality: Quality::Passed,
            provenance: "human".into(),
            findings: vec![],
            reviewer: None,
        },
    )
    .unwrap();
    fs::write(&artifact.path, b"changed after generation").unwrap();
    let config = framechord::cli::Cli::try_parse_from([
        "framechord",
        "--project",
        root.path().to_str().unwrap(),
        "review",
        "record",
        "--file",
        file.to_str().unwrap(),
    ])
    .unwrap();
    assert_eq!(
        framechord::cli::dispatch(&config).await.unwrap_err().code,
        "artifact_corrupt"
    );
    assert_eq!(
        engine::load(root.path(), &run.id).unwrap().jobs[0].artifacts[0].quality,
        Quality::Unchecked
    );
}

#[tokio::test]
async fn provider_reconciliation_recovers_saved_response_without_paid_retry() {
    let _serial = ENGINE_TESTS.lock().await;
    struct Pending {
        generations: AtomicUsize,
        reads: AtomicUsize,
    }
    #[async_trait]
    impl Adapter for Pending {
        async fn generate(&self, _: &Effective) -> Result<Generated> {
            self.generations.fetch_add(1, Ordering::SeqCst);
            let mut error = Error::unknown("Provider still processing");
            error.request_id = Some("interaction_saved".into());
            Err(error)
        }
        async fn reconcile(&self, _: &Effective, id: &str) -> Result<providers::Reconciled> {
            assert_eq!(id, "interaction_saved");
            let count = self.reads.fetch_add(1, Ordering::SeqCst);
            if count == 0 {
                return Ok(providers::Reconciled::Pending);
            }
            Ok(providers::Reconciled::Complete(Generated {
                bytes: png(),
                format: "png".into(),
                request_id: Some(id.into()),
                usage: None,
            }))
        }
    }
    let root = tempfile::tempdir().unwrap();
    let mut effective = input("recover");
    effective.selection = Selection {
        provider: Provider::Google,
        model: "gemini-3.1-flash-image-preview".into(),
    };
    // Use the implemented model from the registry, keeping no network dependency.
    effective.selection.model = providers::registry()
        .into_iter()
        .find(|c| c.provider == Provider::Google && c.media == Media::Image)
        .unwrap()
        .model
        .into();
    let limits = Limits {
        requests: 1,
        attempts: 1,
        ..Default::default()
    };
    let run = engine::submit(root.path(), vec![effective], limits, Submit::default()).unwrap();
    let adapter = Arc::new(Pending {
        generations: AtomicUsize::new(0),
        reads: AtomicUsize::new(0),
    });
    assert_eq!(
        engine::work(root.path(), &run.id, adapter.clone(), false)
            .await
            .unwrap()
            .state,
        State::Unknown
    );
    assert_eq!(
        engine::reconcile(root.path(), &run.id, adapter.as_ref())
            .await
            .unwrap()
            .state,
        State::Unknown
    );
    assert_eq!(
        engine::reconcile(root.path(), &run.id, adapter.as_ref())
            .await
            .unwrap()
            .state,
        State::Queued
    );
    let completed = engine::work(root.path(), &run.id, adapter.clone(), false)
        .await
        .unwrap();
    assert_eq!(completed.state, State::Succeeded);
    assert_eq!(completed.jobs[0].attempts, 1);
    assert_eq!(adapter.generations.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.reads.load(Ordering::SeqCst), 2);
}

#[test]
fn identity_references_preserve_roles_choose_high_fidelity_and_invalidate_cache() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("identity.png"), png()).unwrap();
    fs::write(root.path().join("layout.png"), png()).unwrap();
    fs::write(
        root.path().join("brief.md"),
        "The same selected characters playing together in a forest.",
    )
    .unwrap();
    let yaml = "schema_version: 1\nid: same-characters\nmedia: image\nselection: {provider: openai, model: gpt-image-1}\nbrief: brief.md\nreferences:\n  - layout.png\n  - path: identity.png\n    role: identity\n";
    fs::write(root.path().join("job.yaml"), yaml).unwrap();
    let mut job: Job = storage::yaml(&root.path().join("job.yaml")).unwrap();
    let resolved = validation::resolve(&job, root.path(), &Config::default()).unwrap();
    assert_eq!(resolved.settings.input_fidelity.as_deref(), Some("high"));
    assert!(matches!(
        resolved.references[0].role,
        ReferenceRole::Identity
    ));
    assert_eq!(resolved.references[0].name, "identity.png");
    assert!(resolved.prompt.contains("facial geometry"));
    assert!(resolved.prompt.contains("identity.png"));
    let mut altered = resolved.clone();
    altered.settings.input_fidelity = None;
    assert_ne!(
        validation::fingerprint(&resolved).unwrap(),
        validation::fingerprint(&altered).unwrap()
    );
    job.settings.input_fidelity = Some("low".into());
    assert!(validation::resolve(&job, root.path(), &Config::default()).is_err());
    job.settings.input_fidelity = None;
    job.selection.as_mut().unwrap().model = "gpt-image-1-mini".into();
    assert!(validation::resolve(&job, root.path(), &Config::default()).is_err());
    let bad = yaml.replace("role: identity", "role: imaginary");
    fs::write(root.path().join("invalid.yaml"), bad).unwrap();
    assert!(storage::yaml::<Job>(&root.path().join("invalid.yaml")).is_err());
}
#[test]
fn planned_identity_references_cannot_escape_the_authoring_bundle() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let path = outside.path().join("private.png");
    fs::write(&path, png()).unwrap();
    let job = format!(
        "schema_version: 1\nid: escape\nmedia: image\nselection: {{provider: openai, model: gpt-image-1}}\nbrief: brief.md\nreferences:\n  - path: {}\n    role: identity\n",
        path.display()
    );
    let content=serde_json::json!({"files":[{"path":"job.yaml","content":job},{"path":"brief.md","content":"A scene."}]}).to_string();
    assert!(framechord::planning::stage(&content, "job", root.path(), &Config::default()).is_err());
}

fn derived_job(path: &Path) -> Job {
    serde_json::from_value(serde_json::json!({
        "schema_version":1,"id":"derived","media":"image","brief":"brief.md",
        "references":[{"path":path,"role":"identity"}]
    }))
    .unwrap()
}
#[test]
fn initialization_applies_complete_defaults_before_creating_profiles() {
    let bundled: Config =
        serde_saphyr::from_str(include_str!("../assets/templates/project-config.yaml")).unwrap();
    validation::config(&bundled).unwrap();
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("configured");
    let mut cfg = Config::default();
    cfg.consistency.require_known_identity_model = true;
    cfg.defaults.insert(Media::Image, input("source").selection);
    cfg.default_settings.insert(
        Media::Image,
        Settings {
            input_fidelity: None,
            image_quality: Some("high".into()),
            size: Some("1536x1024".into()),
            format: Some("png".into()),
            ..Default::default()
        },
    );
    cfg.defaults.insert(
        Media::Speech,
        Selection {
            provider: Provider::Openai,
            model: "gpt-4o-mini-tts".into(),
        },
    );
    cfg.default_settings.insert(
        Media::Speech,
        Settings {
            voice: Some("coral".into()),
            format: Some("wav".into()),
            ..Default::default()
        },
    );
    cfg.default_locales.insert(Media::Speech, "en-US".into());
    cfg.limits.concurrency = 3;
    project::init_with_config(&root, &["codex".into(), "claude".into()], Some(&cfg)).unwrap();
    let c = validation::character(&project::character_path(&root, "persona").unwrap()).unwrap();
    assert_eq!(
        c.visual.unwrap().settings,
        cfg.default_settings[&Media::Image]
    );
    assert_eq!(c.voice.unwrap().binding.as_deref(), Some("coral"));
    fs::write(root.join("brief.md"), "New original").unwrap();
    let mut job = derived_job(Path::new("unused.png"));
    job.references.clear();
    let e =
        validation::resolve_project(&root, &job, &root, &project::config(&root).unwrap()).unwrap();
    assert_eq!(e.settings.size.as_deref(), Some("1536x1024"));
    assert_eq!(e.settings.input_fidelity, None); // edit-only preference is not sent for original generation
    assert_eq!(project::config(&root).unwrap().limits.concurrency, 3);
    fs::write(root.join(".env"), "TEST_PRIVATE=preserve\n").unwrap();
    project::init_with_config(&root, &[], Some(&cfg)).unwrap();
    cfg.limits.concurrency = 2;
    assert!(project::init_with_config(&root, &[], Some(&cfg)).is_err());
    assert_eq!(
        fs::read_to_string(root.join(".env")).unwrap(),
        format!(
            "TEST_PRIVATE=preserve\n{}",
            framechord::credentials::ENV_TEMPLATE
        )
    );
    assert_eq!(project::config(&root).unwrap().limits.concurrency, 3);
}
#[test]
fn incompatible_configuration_is_rejected_before_initialization_writes() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("absent");
    let mut cfg = Config::default();
    cfg.default_settings.insert(
        Media::Image,
        Settings {
            size: Some("invalid".into()),
            ..Default::default()
        },
    );
    assert!(project::init_with_config(&root, &[], Some(&cfg)).is_err());
    assert!(!root.exists());
    cfg.default_settings.clear();
    cfg.defaults.insert(
        Media::Speech,
        Selection {
            provider: Provider::Fish,
            model: "s1".into(),
        },
    );
    cfg.default_locales.insert(Media::Speech, "en-US".into());
    assert!(validation::config(&cfg).is_err());
}
#[tokio::test]
async fn original_receipt_pins_model_settings_and_survives_character_export() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    let mut original = input("original");
    original.settings.image_quality = Some("low".into());
    original.settings.size = Some("1024x1024".into());
    let run = engine::submit(
        root,
        vec![original.clone()],
        Limits::default(),
        Submit::default(),
    )
    .unwrap();
    let fake = Arc::new(Fake::new());
    let done = engine::work(root, &run.id, fake.clone(), false)
        .await
        .unwrap();
    let artifact = &done.jobs[0].artifacts[0];
    let mut cfg = Config::default();
    cfg.consistency.require_known_identity_model = true;
    cfg.defaults.insert(
        Media::Image,
        Selection {
            provider: Provider::Openai,
            model: "gpt-image-2".into(),
        },
    );
    cfg.default_settings.insert(
        Media::Image,
        Settings {
            size: Some("1536x1024".into()),
            image_quality: Some("high".into()),
            ..Default::default()
        },
    );
    project::init_with_config(root, &[], Some(&cfg)).unwrap();
    fs::write(root.join("brief.md"), "Same person playing in the forest").unwrap();
    let copy = root.join("portable-original.png");
    fs::copy(&artifact.path, &copy).unwrap();
    let mut job = derived_job(&copy);
    let e = validation::resolve_project(root, &job, root, &cfg).unwrap();
    assert_eq!(e.selection, original.selection);
    assert_eq!(e.settings.size, original.settings.size);
    assert_eq!(e.settings.image_quality, original.settings.image_quality);
    assert_eq!(e.settings.input_fidelity, None);
    assert_eq!(
        e.references[0].origin.as_ref().unwrap().artifact.as_deref(),
        Some(artifact.id.as_str())
    );
    let mut bypass = e.clone();
    bypass.selection.model = "gpt-image-2".into();
    assert!(providers::validate(&bypass).is_err());
    job.selection = cfg.defaults.get(&Media::Image).cloned();
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
    job.selection = None;
    job.settings.size = Some("1536x1024".into());
    assert_eq!(
        validation::resolve_project(root, &job, root, &cfg)
            .unwrap()
            .settings
            .size
            .as_deref(),
        Some("1536x1024")
    );
    let char_path = project::character_path(root, "persona").unwrap();
    assert!(project::accept_project(root, &char_path, &artifact.path, "image").is_err());
    let mut c: Character = storage::yaml(&char_path).unwrap();
    c.visual.as_mut().unwrap().selection = original.selection.clone();
    c.visual.as_mut().unwrap().settings = Settings::default();
    storage::save(&char_path, &c).unwrap();
    project::accept_project(root, &char_path, &artifact.path, "image").unwrap();
    let bundle = t.path().join("bundle");
    project::bundle(&char_path, &bundle, None).unwrap();
    let destination = tempfile::tempdir().unwrap();
    project::init_with_config(destination.path(), &[], Some(&cfg)).unwrap();
    let imported = project::import(destination.path(), &bundle, Some("imported")).unwrap();
    fs::remove_dir_all(root.join("framechord/runs")).unwrap();
    fs::write(destination.path().join("brief.md"), "Same original").unwrap();
    job.references.clear();
    job.character = Some(imported);
    let portable =
        validation::resolve_project(destination.path(), &job, destination.path(), &cfg).unwrap();
    assert_eq!(portable.selection, original.selection);
    assert_eq!(
        portable.references[0].origin.as_ref().unwrap().sha256,
        artifact.sha256
    );
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn receipt_lookup_detects_ambiguous_copies_rejected_sources_and_tampering() {
    let _serial = ENGINE_TESTS.lock().await;
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    let a = input("first");
    let mut b = input("second");
    b.selection.model = "gpt-image-2.5-sunburst".into();
    let run = engine::submit(root, vec![a, b], Limits::default(), Submit::default()).unwrap();
    let done = engine::work(root, &run.id, Arc::new(Fake::new()), false)
        .await
        .unwrap();
    let artifact = &done.jobs[0].artifacts[0];
    fs::write(root.join("brief.md"), "Same character").unwrap();
    let cfg = Config::default();
    let mut job = derived_job(&artifact.path);
    assert_eq!(
        validation::resolve_project(root, &job, root, &cfg)
            .unwrap()
            .selection
            .model,
        "gpt-image-2.5-flare"
    );
    let copy = root.join("copied.png");
    fs::copy(&artifact.path, &copy).unwrap();
    job = derived_job(&copy);
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
    let mut rejected = done.clone();
    rejected.jobs[0].artifacts[0].quality = Quality::NeedsRevision;
    storage::save(
        &engine::run_path(root, &done.id).unwrap().join("run.yaml"),
        &rejected,
    )
    .unwrap();
    job = derived_job(&artifact.path);
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
    let mut changed = image::RgbImage::from_pixel(2, 2, image::Rgb([200, 20, 10]));
    changed.put_pixel(0, 0, image::Rgb([1, 2, 3]));
    changed.save(&artifact.path).unwrap();
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
}
#[test]
fn strict_external_identity_requires_hashed_origin_and_rejects_mixed_models() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    fs::write(root.join("original.png"), png()).unwrap();
    fs::write(root.join("brief.md"), "Same character").unwrap();
    let mut cfg = Config::default();
    cfg.consistency.require_known_identity_model = true;
    cfg.defaults
        .insert(Media::Image, input("x").selection.clone());
    let mut job = derived_job(Path::new("original.png"));
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
    let origin = AssetOrigin {
        selection: input("x").selection,
        settings: Settings::default(),
        locale: None,
        sha256: storage::hash(&png()),
        artifact: None,
    };
    if let JobReference::Detailed(r) = &mut job.references[0] {
        r.origin = Some(origin.clone());
    }
    assert!(validation::resolve_project(root, &job, root, &cfg).is_ok());
    let mut wrong = origin.clone();
    wrong.sha256 = "incorrect".into();
    if let JobReference::Detailed(r) = &mut job.references[0] {
        r.origin = Some(wrong);
    }
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
    if let JobReference::Detailed(r) = &mut job.references[0] {
        r.origin = Some(origin.clone());
    }
    let mut other = origin;
    other.selection.model = "gpt-image-2".into();
    job.references
        .push(JobReference::Detailed(Box::new(InfoReference {
            path: "original.png".into(),
            role: ReferenceRole::Identity,
            origin: Some(other),
        })));
    assert!(validation::resolve_project(root, &job, root, &cfg).is_err());
}
#[test]
fn configured_speech_and_podcast_defaults_resolve_without_repeating_flags() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("script.txt"), "Exact words.").unwrap();
    let mut cfg = Config::default();
    let selection = Selection {
        provider: Provider::Openai,
        model: "gpt-4o-mini-tts".into(),
    };
    for media in [Media::Speech, Media::Podcast] {
        cfg.defaults.insert(media, selection.clone());
        cfg.default_settings.insert(
            media,
            Settings {
                voice: Some("coral".into()),
                format: Some("wav".into()),
                ..Default::default()
            },
        );
        cfg.default_locales.insert(media, "en-US".into());
    }
    validation::config(&cfg).unwrap();
    let speech:Job=serde_json::from_value(serde_json::json!({"schema_version":1,"id":"speech","media":"speech","text_file":"script.txt"})).unwrap();
    let e = validation::resolve(&speech, t.path(), &cfg).unwrap();
    assert_eq!(e.settings.voice.as_deref(), Some("coral"));
    assert_eq!(e.locale.as_deref(), Some("en-US"));
    let podcast:Job=serde_json::from_value(serde_json::json!({"schema_version":1,"id":"podcast","media":"podcast","turns":[{"id":"turn","text_file":"script.txt"}]})).unwrap();
    let e = validation::resolve(&podcast, t.path(), &cfg).unwrap();
    assert_eq!(e.settings, Settings::default());
    assert_eq!(e.turns[0].settings.voice.as_deref(), Some("coral"));
    let mut alternate = podcast.clone();
    alternate.turns[0].selection = Some(Selection {
        provider: Provider::Google,
        model: "gemini-3.8-flash-tts".into(),
    });
    alternate.turns[0].settings.voice = Some("Kore".into());
    let e = validation::resolve(&alternate, t.path(), &cfg).unwrap();
    assert_eq!(e.turns[0].selection.provider, Provider::Google);
    assert_eq!(e.turns[0].settings.voice.as_deref(), Some("Kore"));
}
