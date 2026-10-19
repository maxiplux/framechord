//! Local chapter and full-book assembly. No provider calls happen here.
use crate::{
    audio,
    contracts::*,
    engine::artifact_file,
    error::{Error, Result},
    storage::*,
};
use std::path::{Path, PathBuf};

/// Assemble one audiobook chapter or book job from its completed dependencies.
pub fn run(dir: &Path, run: &Run, mut job: JobRecord) -> JobRecord {
    let cancelled = || dir.join("cancel").exists();
    job.artifacts.clear();
    let result = match job.input.audiobook.as_ref().map(|p| p.role) {
        Some(AudiobookRole::Chapter) => chapter(dir, run, &mut job, &cancelled),
        Some(AudiobookRole::Book) => book(dir, run, &mut job, &cancelled),
        None => Err(Error::new("assembly", "Not an audiobook job", 3)),
    };
    match result {
        Ok(()) => {
            job.state = State::Succeeded;
            job.error = None;
        }
        Err(e) if e.code == "cancelled" => {
            job.state = State::Cancelled;
            job.error = Some(e.to_string());
        }
        Err(e) => {
            job.state = State::Failed;
            job.error = Some(e.to_string());
        }
    }
    let _ = crate::progress::save(
        dir,
        &job.input.id,
        if job.state == State::Succeeded {
            crate::progress::Stage::Complete
        } else {
            crate::progress::Stage::Failed
        },
    );
    job
}

fn dependency<'a>(run: &'a Run, job: &JobRecord, child: &str) -> Result<&'a JobRecord> {
    let name = format!("{}--{child}", job.input.id);
    run.jobs
        .iter()
        .find(|r| r.input.id == name && r.state == State::Succeeded)
        .filter(|r| !r.artifacts.is_empty())
        .ok_or_else(|| Error::new("assembly", format!("{name} has no successful artifact"), 3))
}

#[cfg(feature = "mp3")]
fn tag(
    plan: &AudiobookPlan,
    title: &str,
    track: Option<(u32, u32)>,
    chapters: &[(String, u32, u32)],
) -> Result<Vec<u8>> {
    use id3::TagLike;
    let mut tag = id3::Tag::new();
    tag.set_title(title);
    if let Some(book) = &plan.title {
        tag.set_album(book.as_str());
    }
    if let Some(author) = &plan.author {
        tag.set_artist(author.as_str());
    }
    tag.set_genre("Audiobook");
    if let Some((n, total)) = track {
        tag.set_track(n);
        tag.set_total_tracks(total);
    }
    let mut elements = vec![];
    for (i, (name, start, end)) in chapters.iter().enumerate() {
        let element = format!("ch{}", i + 1);
        elements.push(element.clone());
        tag.add_frame(id3::frame::Chapter {
            element_id: element,
            start_time: *start,
            end_time: *end,
            start_offset: u32::MAX,
            end_offset: u32::MAX,
            frames: vec![id3::Frame::text("TIT2", name.as_str())],
        });
    }
    if !elements.is_empty() {
        tag.add_frame(id3::frame::TableOfContents {
            element_id: "toc".into(),
            top_level: true,
            ordered: true,
            elements,
            frames: vec![],
        });
    }
    let mut out = vec![];
    tag.write_to(&mut out, id3::Version::Id3v23)
        .map_err(|_| Error::new("audio_write", "Cannot write the MP3 ID3 tag", 3))?;
    Ok(out)
}
#[cfg(not(feature = "mp3"))]
fn tag(
    _: &AudiobookPlan,
    _: &str,
    _: Option<(u32, u32)>,
    _: &[(String, u32, u32)],
) -> Result<Vec<u8>> {
    Err(Error::input("This build has no MP3 encoder"))
}

fn chapter_name(input: &Effective) -> String {
    let info = input.audiobook.as_ref().and_then(|p| p.chapter.as_ref());
    info.and_then(|i| i.title.clone())
        .unwrap_or_else(|| input.id.rsplit("--").next().unwrap_or(&input.id).to_owned())
}

/// Copy each artifact to its destination, updating the recorded path.
fn deliver(run: &Run, artifact: &mut Artifact, dest: Option<&Path>) -> Result<()> {
    if let Some(dest) = dest {
        copy_atomic(&artifact.path, dest, run.overwrite)?;
        artifact.path = absolute(dest)?;
    }
    Ok(())
}

fn chapter(dir: &Path, run: &Run, job: &mut JobRecord, cancelled: &dyn Fn() -> bool) -> Result<()> {
    let plan = job.input.audiobook.clone().unwrap();
    let id = job.input.id.clone();
    let clips = job
        .input
        .turns
        .iter()
        .map(|part| {
            Ok(dependency(run, job, &part.id)?
                .artifacts
                .last()
                .unwrap()
                .path
                .clone())
        })
        .collect::<Result<Vec<PathBuf>>>()?;
    let assembly = job.input.assembly.clone().unwrap_or_default();
    let wav_path = dir.join("outputs").join(format!("{id}.wav"));
    let wav = temp_beside(&wav_path)?;
    let mp3_path = dir.join("outputs").join(format!("{id}.mp3"));
    let mp3 = match plan.format {
        AudioFormat::Mp3 => {
            let info = plan.chapter.as_ref();
            let bytes = tag(
                &plan,
                &chapter_name(&job.input),
                info.map(|i| (i.index, i.count)),
                &[],
            )?;
            Some((temp_beside(&mp3_path)?, bytes))
        }
        AudioFormat::Wav => None,
    };
    let mut writer = audio::Writer::new(
        Some(wav.reopen()?),
        mp3.as_ref()
            .map(|(file, bytes)| {
                Ok::<_, Error>((file.reopen()?, plan.mp3_bitrate_kbps, bytes.clone()))
            })
            .transpose()?,
    )?;
    audio::assemble_files(
        &clips,
        assembly.pause_ms,
        assembly.loudness_lufs,
        &mut writer,
        cancelled,
    )?;
    writer.finish()?;
    persist(wav, &wav_path, true)?;
    job.artifacts.push(artifact_file(
        &wav_path,
        &id,
        "wav",
        run.parent_artifact.clone(),
    )?);
    if let Some((file, _)) = mp3 {
        persist(file, &mp3_path, true)?;
        job.artifacts.push(artifact_file(
            &mp3_path,
            &id,
            "mp3",
            run.parent_artifact.clone(),
        )?);
    }
    let output = job.input.output.clone();
    deliver(run, job.artifacts.last_mut().unwrap(), output.as_deref())
}

/// Path written into the chapter index: a bare file name beside the index, else absolute.
fn reference(path: &Path, index: &Path) -> String {
    if path.parent() == index.parent() {
        path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        )
    } else {
        path.display().to_string()
    }
}

fn cue_time(seconds: f64) -> String {
    let frames = (seconds * 75.0).round() as u64;
    format!(
        "{:02}:{:02}:{:02}",
        frames / 75 / 60,
        frames / 75 % 60,
        frames % 75
    )
}
fn quoted(text: &str) -> String {
    text.replace('"', "'").replace(['\n', '\r'], " ")
}

fn book(dir: &Path, run: &Run, job: &mut JobRecord, cancelled: &dyn Fn() -> bool) -> Result<()> {
    let plan = job.input.audiobook.clone().unwrap();
    let id = job.input.id.clone();
    let pause_ms = job.input.assembly.as_ref().map_or(0, |a| a.pause_ms);
    let pause = audio::pause_samples(pause_ms) as u64;
    let full = plan.output_mode == OutputMode::Full;
    let ext = plan.format.extension();
    // Final locations decide the relative paths written into the index.
    let index_path = match (&job.input.output, full) {
        (Some(out), true) => out.with_file_name(format!("{id}.chapters.json")),
        (Some(out), false) => out.clone(),
        (None, _) => dir.join("outputs").join(format!("{id}.chapters.json")),
    };
    let audio_path = job
        .input
        .output
        .clone()
        .filter(|_| full)
        .unwrap_or_else(|| dir.join("outputs").join(format!("{id}.{ext}")));
    let mut chapters = vec![];
    let mut wavs = vec![];
    let mut start = 0u64;
    for (n, c) in job.input.turns.iter().enumerate() {
        let record = dependency(run, job, &c.id)?;
        let wav = record
            .artifacts
            .iter()
            .find(|a| a.format == "wav")
            .ok_or_else(|| Error::new("assembly", "Chapter has no WAV intermediate", 3))?;
        let published = record.artifacts.last().unwrap();
        let samples = hound::WavReader::open(&wav.path)
            .map_err(|_| Error::new("invalid_audio", "Cannot read chapter WAV", 3))?
            .duration() as u64;
        if n > 0 {
            start += pause;
        }
        let info = c.audiobook.as_ref().and_then(|p| p.chapter.clone());
        let seconds = |s: u64| s as f64 / audio::RATE as f64;
        chapters.push(IndexedChapter {
            index: info.as_ref().map_or(n as u32 + 1, |i| i.index),
            id: c.id.clone(),
            title: info.as_ref().and_then(|i| i.title.clone()),
            file: reference(&published.path, &index_path),
            sha256: published.sha256.clone(),
            start_seconds: seconds(start),
            duration_seconds: seconds(samples),
            end_seconds: seconds(start + samples),
            source: info.as_ref().map(|i| i.source.clone()).unwrap_or_default(),
            text_sha256: info.map(|i| i.text_sha256).unwrap_or_default(),
        });
        wavs.push(wav.path.clone());
        start += samples;
    }
    let total = start;
    let mut warnings = vec![];
    let mut artifacts = vec![];
    let mut full_file = None;
    if full {
        if plan.format == AudioFormat::Wav && total > audio::wav_capacity() {
            return Err(Error::input(
                "The full book exceeds the WAV size limit (about 24.8 hours); use audiobook.format: mp3 or output_mode: chapters",
            ));
        }
        let staged = dir.join("outputs").join(format!("{id}.{ext}"));
        let file = temp_beside(&staged)?;
        let mut writer = match plan.format {
            AudioFormat::Wav => audio::Writer::new(Some(file.reopen()?), None)?,
            AudioFormat::Mp3 => {
                let marks: Vec<_> = job
                    .input
                    .turns
                    .iter()
                    .zip(&chapters)
                    .map(|(c, i)| {
                        (
                            chapter_name(c),
                            (i.start_seconds * 1000.0).round() as u32,
                            (i.end_seconds * 1000.0).round() as u32,
                        )
                    })
                    .collect();
                let title = plan.title.clone().unwrap_or_else(|| id.clone());
                let bytes = tag(&plan, &title, None, &marks)?;
                audio::Writer::new(None, Some((file.reopen()?, plan.mp3_bitrate_kbps, bytes)))?
            }
        };
        audio::concat_files(&wavs, pause_ms, &mut writer, cancelled)?;
        writer.finish()?;
        persist(file, &staged, true)?;
        let a = artifact_file(&staged, &id, ext, run.parent_artifact.clone())?;
        full_file = Some(IndexedFile {
            file: reference(&audio_path, &index_path),
            duration_seconds: total as f64 / audio::RATE as f64,
            sha256: a.sha256.clone(),
        });
        if chapters.len() <= 99 {
            let mut cue = String::new();
            if let Some(author) = &plan.author {
                cue += &format!("PERFORMER \"{}\"\n", quoted(author));
            }
            if let Some(title) = &plan.title {
                cue += &format!("TITLE \"{}\"\n", quoted(title));
            }
            cue += &format!(
                "FILE \"{}\" {}\n",
                quoted(&reference(&audio_path, &index_path)),
                if plan.format == AudioFormat::Mp3 {
                    "MP3"
                } else {
                    "WAVE"
                }
            );
            for (n, (c, i)) in job.input.turns.iter().zip(&chapters).enumerate() {
                cue += &format!(
                    "  TRACK {:02} AUDIO\n    TITLE \"{}\"\n    INDEX 01 {}\n",
                    n + 1,
                    quoted(&chapter_name(c)),
                    cue_time(i.start_seconds)
                );
            }
            let staged_cue = dir.join("outputs").join(format!("{id}.cue"));
            atomic(&staged_cue, cue.as_bytes(), true)?;
            let dest = job
                .input
                .output
                .as_ref()
                .map(|o| o.with_file_name(format!("{id}.cue")));
            artifacts.push((
                artifact_file(&staged_cue, &id, "cue", run.parent_artifact.clone())?,
                dest,
            ));
        } else {
            warnings.push("No .cue sheet: the CUE format allows at most 99 tracks".to_owned());
        }
        artifacts.push((a, job.input.output.clone()));
    }
    let index = ChapterIndex {
        schema_version: 1,
        book: id.clone(),
        title: plan.title.clone(),
        author: plan.author.clone(),
        format: plan.format,
        output_mode: plan.output_mode,
        sample_rate: audio::RATE,
        channels: 1,
        pause_ms,
        full: full_file,
        chapters,
        warnings,
    };
    let staged_index = dir.join("outputs").join(format!("{id}.chapters.json"));
    atomic(&staged_index, &serde_json::to_vec_pretty(&index)?, true)?;
    let index_dest = job.input.output.as_ref().map(|_| index_path.clone());
    artifacts.insert(
        0,
        (
            artifact_file(&staged_index, &id, "json", run.parent_artifact.clone())?,
            index_dest,
        ),
    );
    for (mut artifact, dest) in artifacts {
        deliver(run, &mut artifact, dest.as_deref())?;
        job.artifacts.push(artifact);
    }
    Ok(())
}
