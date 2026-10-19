//! Audiobook resolution: one narrator, exact chapter text, local assembly.
//!
//! A book resolves to an `Audiobook` effective whose turns are chapters; each
//! chapter is speech whose turns are provider-sized parts. Only parts are sent
//! to providers; chapters and the book are assembled locally.
use crate::{
    contracts::*,
    error::{Error, Result},
    manuscript, providers,
    storage::*,
    validation,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

/// Request size used when a speech model has no recorded per-request limit.
///
/// Checked 2026-10-19: Gemini 3.8 Flash TTS lists an 8,192-token input limit
/// (https://ai.google.dev/gemini-api/docs/models/gemini-3.8-flash-tts) and Fish
/// documents no per-request character cap
/// (https://docs.fish.audio/api-reference/endpoint/openapi-v1/text-to-speech).
/// 4,000 characters (about five minutes of narration) stays well inside both
/// while keeping a failed or revised part cheap to repeat.
pub const DEFAULT_CHUNK_CHARS: usize = 4000;
pub const DEFAULT_MP3_KBPS: u16 = 64;
/// MPEG-2 Layer III bitrates valid at the 24 kHz working rate.
/// Only rates the bundled encoder exposes are accepted, so a job cannot pass
/// validation and then fail at assembly after paid requests.
pub const MP3_KBPS: [u16; 12] = [8, 16, 24, 32, 40, 48, 64, 80, 96, 112, 128, 160];
/// Reservation estimate shared with the engine budget.
pub const CHARS_PER_SECOND: u32 = 12;

struct Source {
    id: String,
    title: Option<String>,
    body: String,
    source: String,
    brief: Option<String>,
    settings: Settings,
}

fn label(value: &Option<String>, field: &str) -> Result<()> {
    if value.as_ref().is_some_and(|v| {
        v.trim().is_empty() || v.chars().count() > 200 || v.chars().any(char::is_control)
    }) {
        return Err(Error::input(format!(
            "{field} must be 1–200 characters without control characters"
        )));
    }
    Ok(())
}

pub fn options(job: &Job) -> Result<AudiobookOptions> {
    let options = job.audiobook.clone().unwrap_or_default();
    if let Some(kbps) = options.mp3_bitrate_kbps {
        if options.format != AudioFormat::Mp3 {
            return Err(Error::input(
                "audiobook.mp3_bitrate_kbps requires format: mp3",
            ));
        }
        if !MP3_KBPS.contains(&kbps) {
            return Err(Error::input(format!(
                "audiobook.mp3_bitrate_kbps must be one of {MP3_KBPS:?} at the 24 kHz mono working rate"
            )));
        }
    }
    if options.format == AudioFormat::Mp3 && !cfg!(feature = "mp3") {
        return Err(Error::input(
            "This build has no MP3 encoder; use audiobook.format: wav or a build with the mp3 feature",
        ));
    }
    if options.max_chars_per_request.is_some_and(|n| n < 200) {
        return Err(Error::input(
            "audiobook.max_chars_per_request must be at least 200",
        ));
    }
    label(&options.title, "audiobook.title")?;
    label(&options.author, "audiobook.author")?;
    Ok(options)
}

fn sources(job: &Job, base: &Path, max_id: usize) -> Result<Vec<Source>> {
    match (&job.manuscript, job.chapters.is_empty()) {
        (Some(_), false) | (None, true) => Err(Error::input(
            "Audiobook requires exactly one of manuscript or chapters",
        )),
        (Some(path), true) => {
            let split = job.split.clone().unwrap_or_default();
            let report = manuscript::split(&text(base, path)?, &split, max_id)?;
            Ok(report
                .chapters
                .into_iter()
                .map(|c| Source {
                    source: format!("{}:{}-{}", path.display(), c.lines.0, c.lines.1),
                    id: c.id,
                    title: c.title,
                    body: c.body,
                    brief: None,
                    settings: Settings::default(),
                })
                .collect())
        }
        (None, false) => {
            if job.split.is_some() {
                return Err(Error::input("split applies only to a manuscript"));
            }
            let mut ids = BTreeSet::new();
            job.chapters
                .iter()
                .map(|c| {
                    id(&c.id)?;
                    if c.id.contains("--") || c.id.len() > max_id {
                        return Err(Error::input(format!(
                            "Chapter ID `{}` must avoid `--` and use at most {max_id} characters with this audiobook ID",
                            c.id
                        )));
                    }
                    if !ids.insert(c.id.clone()) {
                        return Err(Error::input("Audiobook chapter IDs must be unique"));
                    }
                    label(&c.title, "Chapter title")?;
                    Ok(Source {
                        id: c.id.clone(),
                        title: c.title.clone(),
                        body: text(base, &c.text_file)?,
                        source: c.text_file.display().to_string(),
                        brief: c.brief.as_ref().map(|p| text(base, p)).transpose()?,
                        settings: c.settings.clone(),
                    })
                })
                .collect()
        }
    }
}

/// Narrator selection and where it came from; never guessed silently.
fn narrator(
    job: &Job,
    cfg: &Config,
) -> Result<(Option<Selection>, &'static str, Settings, Option<String>)> {
    if job.character.is_some() {
        return Ok((
            job.selection.clone(),
            "character",
            Settings::default(),
            None,
        ));
    }
    if let Some(selection) = &job.selection {
        return Ok((Some(selection.clone()), "job", Settings::default(), None));
    }
    if let Some(selection) = cfg.defaults.get(&Media::Audiobook) {
        return Ok((
            Some(selection.clone()),
            "audiobook_default",
            cfg.default_settings
                .get(&Media::Audiobook)
                .cloned()
                .unwrap_or_default(),
            cfg.default_locales.get(&Media::Audiobook).cloned(),
        ));
    }
    if let Some(selection) = cfg.defaults.get(&Media::Speech) {
        // Speech defaults (settings and locale) are inherited by the speech resolver.
        return Ok((
            Some(selection.clone()),
            "speech_default",
            Settings::default(),
            None,
        ));
    }
    Err(Error::input(
        "Audiobook needs a narrator: set character, selection, or configure an audiobook or speech default",
    ))
}

pub fn resolve(
    job: &Job,
    base: &Path,
    cfg: &Config,
    index: Option<&crate::provenance::Index>,
) -> Result<Effective> {
    if job.text_file.is_some()
        || !job.turns.is_empty()
        || job.spec.is_some()
        || !job.references.is_empty()
    {
        return Err(Error::input(
            "Audiobooks take text from manuscript or chapters; text_file, turns, spec and references are not used",
        ));
    }
    if job.output.is_some() {
        return Err(Error::input(
            "Audiobooks publish several files; omit output and pass --output-dir",
        ));
    }
    if job.settings.format.is_some() {
        return Err(Error::input(
            "Choose the audiobook container with audiobook.format (wav or mp3), not settings.format",
        ));
    }
    let assembly = job.assembly.clone().unwrap_or_default();
    if !assembly.turn_gain_db.is_empty() {
        return Err(Error::input(
            "turn_gain_db applies to podcasts; audiobook chapters share one loudness target",
        ));
    }
    crate::audio::validate_assembly(&assembly, &[])?;
    let options = options(job)?;
    for dep in &job.depends_on {
        id(dep)?;
    }
    let max_id = manuscript::max_chapter_id(&job.id)?;
    let sources = sources(job, base, max_id)?;
    let (selection, _, defaults, default_locale) = narrator(job, cfg)?;
    let count = sources.len() as u32;
    let mut chapters = vec![];
    for (i, source) in sources.into_iter().enumerate() {
        let spoken_title = options.announce_titles && source.title.is_some();
        let words = match (&source.title, spoken_title) {
            (Some(title), true) => format!("{title}\n\n{}", source.body),
            _ => source.body.clone(),
        };
        let mut settings = defaults.clone();
        settings.overlay(&job.settings);
        settings.overlay(&source.settings);
        let speech = Job {
            schema_version: 1,
            id: source.id.clone(),
            media: Media::Speech,
            selection: if job.character.is_some() {
                job.selection.clone()
            } else {
                selection.clone()
            },
            brief: job.brief.clone(),
            text_file: None,
            character: job.character.clone(),
            cast: vec![],
            assembly: None,
            spec: None,
            references: vec![],
            voice_samples: job.voice_samples.clone(),
            turns: vec![],
            settings,
            locale: job.locale.clone().or_else(|| default_locale.clone()),
            depends_on: vec![],
            output: None,
            manuscript: None,
            split: None,
            chapters: vec![],
            audiobook: None,
        };
        let mut chapter = validation::resolve_chapter(&speech, base, cfg, index, &words)?;
        if let Some(brief) = &source.brief {
            chapter.prompt = format!("{}\nCHAPTER DIRECTION:\n{brief}", chapter.prompt);
        }
        let limit = options
            .max_chars_per_request
            .map_or(DEFAULT_CHUNK_CHARS, |n| n as usize)
            .min(providers::speech_limit(&chapter.selection).unwrap_or(usize::MAX));
        chapter.turns.clear();
        let mut leaf = chapter.clone();
        leaf.assembly = None;
        leaf.audiobook = None;
        leaf.output = None;
        leaf.depends_on.clear();
        for (n, part) in validation::chunk_paragraphs(&words, limit)
            .into_iter()
            .enumerate()
        {
            let mut p = leaf.clone();
            p.id = format!("part-{:04}", n + 1);
            p.text = Some(part);
            chapter.turns.push(p);
        }
        chapter.text = None;
        chapter.assembly = Some(Assembly {
            pause_ms: 0,
            turn_gain_db: Default::default(),
            loudness_lufs: assembly.loudness_lufs,
        });
        chapter.audiobook = Some(AudiobookPlan {
            role: AudiobookRole::Chapter,
            output_mode: options.output_mode,
            format: options.format,
            mp3_bitrate_kbps: options.mp3_bitrate_kbps.unwrap_or(DEFAULT_MP3_KBPS),
            title: options.title.clone(),
            author: options.author.clone(),
            chapter: Some(ChapterInfo {
                index: i as u32 + 1,
                count,
                title: source.title.clone(),
                spoken_title,
                source: source.source.clone(),
                text_sha256: hash(words.as_bytes()),
                chars: words.chars().count() as u64,
            }),
            selected: None,
        });
        chapters.push(chapter);
    }
    let Some(first) = chapters.first() else {
        return Err(Error::input(
            "Audiobook has no chapters with narratable text",
        ));
    };
    let identity = |c: &Effective| {
        let mut refs: Vec<_> = c.references.iter().map(|r| r.sha256.clone()).collect();
        refs.sort();
        (
            c.selection.clone(),
            c.character_hash.clone(),
            c.settings.voice.clone(),
            c.locale.clone(),
            refs,
        )
    };
    for chapter in &chapters[1..] {
        if identity(chapter) != identity(first) {
            return Err(Error::input(format!(
                "Audiobook chapter `{}` changes the narrator voice; one narrator per audiobook",
                chapter.id
            )));
        }
    }
    let book = Effective {
        assembly: Some(Assembly {
            pause_ms: assembly.pause_ms,
            turn_gain_db: Default::default(),
            loudness_lufs: None,
        }),
        id: job.id.clone(),
        media: Media::Audiobook,
        selection: first.selection.clone(),
        prompt: String::new(),
        text: None,
        locale: first.locale.clone(),
        settings: Settings::default(),
        character_hash: first.character_hash.clone(),
        references: vec![],
        depends_on: job.depends_on.clone(),
        output: None,
        turns: chapters,
        audiobook: Some(AudiobookPlan {
            role: AudiobookRole::Book,
            output_mode: options.output_mode,
            format: options.format,
            mp3_bitrate_kbps: options.mp3_bitrate_kbps.unwrap_or(DEFAULT_MP3_KBPS),
            title: options.title.clone(),
            author: options.author.clone(),
            chapter: None,
            selected: None,
        }),
    };
    providers::validate(&book)?;
    Ok(book)
}

/// Keep only the selected chapters; the run then has no full-book assembly.
pub fn select(book: &mut Effective, ids: &[String]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    if book.media != Media::Audiobook {
        return Err(Error::input("--chapter applies only to audiobook jobs"));
    }
    let known: Vec<_> = book.turns.iter().map(|c| c.id.clone()).collect();
    for wanted in ids {
        if !known.contains(wanted) {
            return Err(Error::input(format!(
                "Unknown chapter `{wanted}`; this audiobook has: {}",
                known.join(", ")
            )));
        }
    }
    book.turns.retain(|c| ids.contains(&c.id));
    if let Some(plan) = &mut book.audiobook {
        plan.selected = Some(book.turns.iter().map(|c| c.id.clone()).collect());
    }
    Ok(())
}

fn excerpt(text: &str, from_end: bool) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len().min(80);
    let slice = if from_end {
        &chars[chars.len() - n..]
    } else {
        &chars[..n]
    };
    slice.iter().collect::<String>().replace(['\n', '\r'], " ")
}

fn chapter_text(chapter: &Effective) -> String {
    chapter
        .turns
        .iter()
        .filter_map(|p| p.text.as_deref())
        .collect()
}

/// Leaf totals across a book: (requests, speech characters, reserved seconds).
pub fn totals(book: &Effective) -> (u32, u32, u32) {
    let mut requests = 0;
    let mut chars = 0u32;
    let mut seconds = 0u32;
    for part in book.turns.iter().flat_map(|c| &c.turns) {
        let n = part.text.as_deref().unwrap_or("").chars().count() as u32;
        requests += 1;
        chars = chars.saturating_add(n);
        seconds = seconds.saturating_add(n.div_ceil(CHARS_PER_SECOND));
    }
    (requests, chars, seconds)
}

/// Reviewable chapter table for `validate` and dry runs.
pub fn summary(job: &Job, base: &Path, cfg: &Config, book: &Effective) -> Result<Value> {
    let plan = book.audiobook.clone();
    let (_, selection_source, _, _) = narrator(job, cfg)?;
    let mut warnings = vec![];
    let mut skipped = vec![];
    let mut front_matter = Value::Null;
    if let Some(path) = &job.manuscript {
        let report = manuscript::split(
            &text(base, path)?,
            &job.split.clone().unwrap_or_default(),
            manuscript::max_chapter_id(&job.id)?,
        )?;
        warnings = report.warnings;
        skipped = report.skipped;
        front_matter = report
            .front_matter
            .map_or(Value::Null, |(a, b)| json!({"lines": format!("{a}-{b}")}));
    } else {
        for chapter in &book.turns {
            warnings.extend(manuscript::markup_warnings(
                &chapter.id,
                &chapter_text(chapter),
            ));
        }
    }
    let chapters: Vec<Value> = book
        .turns
        .iter()
        .map(|c| {
            let info = c.audiobook.as_ref().and_then(|p| p.chapter.clone());
            let words = chapter_text(c);
            let chars = words.chars().count() as u32;
            json!({
                "index": info.as_ref().map(|i| i.index),
                "id": c.id,
                "title": info.as_ref().and_then(|i| i.title.clone()),
                "spoken_title": info.as_ref().is_some_and(|i| i.spoken_title),
                "source": info.as_ref().map(|i| i.source.clone()),
                "chars": chars,
                "parts": c.turns.len(),
                "estimated_seconds": chars.div_ceil(CHARS_PER_SECOND),
                "text_sha256": info.as_ref().map(|i| i.text_sha256.clone()),
                "opening": excerpt(&words, false),
                "closing": excerpt(&words, true),
            })
        })
        .collect();
    let (requests, chars, seconds) = totals(book);
    Ok(json!({
        "source": job.manuscript.as_ref().map_or("chapters".to_owned(), |p| p.display().to_string()),
        "split": job.manuscript.as_ref().map(|_| job.split.clone().unwrap_or_default().heading_level),
        "front_matter": front_matter,
        "output_mode": plan.as_ref().map(|p| p.output_mode),
        "format": plan.as_ref().map(|p| p.format),
        "pause_ms": book.assembly.as_ref().map_or(0, |a| a.pause_ms),
        "narrator": {
            "selection": book.selection,
            "selection_source": selection_source,
            "voice": book.settings.voice.clone().or_else(|| book.turns.first().and_then(|c| c.settings.voice.clone())),
            "character_hash": book.character_hash,
        },
        "chapters": chapters,
        "skipped": skipped,
        "warnings": warnings,
        "totals": {
            "chapters": book.turns.len(),
            "requests": requests,
            "speech_chars": chars,
            "estimated_audio_seconds": seconds,
        },
    }))
}

/// Book-level view of a run: chapter files, states, the full book and what is missing.
pub fn run_summary(run: &Run) -> Option<Value> {
    let mut books: Vec<String> = vec![];
    for job in &run.jobs {
        let book = match job.input.audiobook.as_ref().map(|p| p.role) {
            Some(AudiobookRole::Book) => job.input.id.clone(),
            Some(AudiobookRole::Chapter) => job.input.id.split("--").next()?.to_owned(),
            None => continue,
        };
        if !books.contains(&book) {
            books.push(book);
        }
    }
    if books.is_empty() {
        return None;
    }
    let summaries: Vec<Value> = books
        .iter()
        .map(|book| {
            let record = run.jobs.iter().find(|j| &j.input.id == book);
            let prefix = format!("{book}--");
            let mut chapters: Vec<&JobRecord> = run
                .jobs
                .iter()
                .filter(|j| {
                    j.input.id.starts_with(&prefix)
                        && j.input.audiobook.as_ref().is_some_and(|p| p.role == AudiobookRole::Chapter)
                })
                .collect();
            chapters.sort_by_key(|j| j.input.audiobook.as_ref().and_then(|p| p.chapter.as_ref()).map(|c| c.index));
            let plan = chapters
                .first()
                .and_then(|c| c.input.audiobook.clone())
                .or_else(|| record.and_then(|r| r.input.audiobook.clone()));
            let rows: Vec<Value> = chapters
                .iter()
                .map(|c| {
                    let info = c.input.audiobook.as_ref().and_then(|p| p.chapter.clone());
                    let parts: Vec<_> = run
                        .jobs
                        .iter()
                        .filter(|j| j.input.id.starts_with(&format!("{}--", c.input.id)))
                        .collect();
                    let published = c.artifacts.last();
                    json!({
                        "index": info.as_ref().map(|i| i.index),
                        "id": c.input.id.trim_start_matches(&prefix),
                        "title": info.and_then(|i| i.title),
                        "state": c.state,
                        "path": published.map(|a| a.path.clone()),
                        "artifact": published.map(|a| a.id.clone()),
                        "duration_seconds": published.and_then(|a| a.duration_seconds),
                        "parts": parts.len(),
                        "cached_parts": parts.iter().filter(|p| p.cached).count(),
                        "error": c.error,
                    })
                })
                .collect();
            let missing: Vec<_> = chapters
                .iter()
                .filter(|c| c.state != State::Succeeded)
                .map(|c| c.input.id.trim_start_matches(&prefix).to_owned())
                .collect();
            let find = |format: &str| {
                record.and_then(|r| r.artifacts.iter().find(|a| a.format == format)).map(|a| a.path.clone())
            };
            let audio = record.and_then(|r| {
                r.artifacts
                    .iter()
                    .find(|a| matches!(a.format.as_str(), "wav" | "mp3"))
            });
            let complete = record.is_some_and(|r| r.state == State::Succeeded);
            let next = if record.is_none() {
                "Chapter subset: run generate again without --chapter to assemble the book; completed chapters are reused from cache"
            } else if complete {
                "Listen to the chapters before accepting the narration; files are not reviewed automatically"
            } else if missing.is_empty() {
                "Book assembly did not finish; inspect the error and resume the run"
            } else {
                "Resume the run, or fix the failed chapter and generate again; successful parts are reused from cache"
            };
            json!({
                "job": book,
                "output_mode": plan.as_ref().map(|p| p.output_mode),
                "format": plan.as_ref().map(|p| p.format),
                "selected": record.is_none().then(|| rows.iter().map(|r| r["id"].clone()).collect::<Vec<_>>()),
                "complete": complete,
                "state": record.map(|r| r.state),
                "error": record.and_then(|r| r.error.clone()),
                "full": audio.map(|a| json!({"path": a.path, "artifact": a.id, "duration_seconds": a.duration_seconds})),
                "sidecars": {"chapters_json": find("json"), "cue": find("cue")},
                "chapters": rows,
                "missing": missing,
                "next": next,
            })
        })
        .collect();
    Some(json!(summaries))
}

/// Replace long narration strings in a JSON view with their size and hash.
pub fn compact(value: &mut Value) {
    match value {
        Value::Array(items) => items.iter_mut().for_each(compact),
        Value::Object(map) => {
            for (key, item) in map.iter_mut() {
                if key == "text"
                    && let Some(text) = item.as_str()
                    && text.chars().count() > 2000
                {
                    *item = json!({"text_chars": text.chars().count(), "text_sha256": hash(text.as_bytes())});
                } else {
                    compact(item);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunks_concatenate_exactly_and_stay_paragraph_aligned() {
        let paragraph = |n: usize| format!("Sentence {n} is here. Another one follows it.\n\n");
        let text: String = (0..40).map(paragraph).collect::<String>() + "End.";
        let chunks = validation::chunk_paragraphs(&text, 300);
        assert_eq!(chunks.concat(), text);
        assert!(
            chunks
                .iter()
                .all(|c| c.chars().count() <= 300 && !c.trim().is_empty())
        );
        assert!(
            chunks[..chunks.len() - 1]
                .iter()
                .all(|c| c.ends_with("\n\n"))
        );
        let edited = text.replacen("Sentence 0", "Sentence zero", 1);
        let after = validation::chunk_paragraphs(&edited, 300);
        assert_eq!(chunks[1..], after[1..]);
        let long = "word ".repeat(200) + "Final sentence. " + &"x".repeat(50);
        let pieces = validation::chunk_paragraphs(&long, 250);
        assert_eq!(pieces.concat(), long);
        assert!(pieces.iter().all(|c| c.chars().count() <= 250));
        let unbroken = "y".repeat(1000);
        assert_eq!(
            validation::chunk_paragraphs(&unbroken, 300).concat(),
            unbroken
        );
        let crlf = "One.\r\n\r\nTwo.\r\n\r\nThree.";
        assert_eq!(validation::chunk_paragraphs(crlf, 8).len(), 3);
    }
}
