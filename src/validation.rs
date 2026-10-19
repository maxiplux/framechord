use crate::{
    contracts::*,
    error::{Error, Result},
    providers,
    storage::*,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::{collections::BTreeSet, fs, path::Path};

pub fn config(config: &Config) -> Result<()> {
    version2(config.schema_version)?;
    limits(&config.limits)?;
    for (media, selection) in &config.defaults {
        model(selection)?;
        let target = crate::catalog::target(*media);
        let supported = if target == Media::Text {
            matches!(
                selection.provider,
                Provider::Openai | Provider::Google | Provider::Openrouter
            )
        } else {
            providers::registry().iter().any(|c| {
                c.provider == selection.provider && c.model == selection.model && c.media == target
            })
        };
        if !supported {
            return Err(Error::input(format!(
                "Unsupported default provider/model for {media:?}; inspect framechord models"
            )));
        }
    }
    for media in config
        .default_settings
        .keys()
        .chain(config.default_locales.keys())
        .collect::<BTreeSet<_>>()
    {
        let empty = Settings::default();
        let settings = config.default_settings.get(media).unwrap_or(&empty);
        let selection = config.defaults.get(media).ok_or_else(|| {
            Error::input(
                "Default settings require an explicit provider/model for the same media type",
            )
        })?;
        let target = crate::catalog::target(*media);
        let mut probe = Effective {
            assembly: None,
            id: "configuration-check".into(),
            media: target,
            selection: selection.clone(),
            prompt: String::new(),
            text: if target == Media::Speech {
                Some("Offline validation".into())
            } else {
                None
            },
            locale: config.default_locales.get(media).cloned(),
            settings: settings.clone(),
            references: vec![],
            character_hash: None,
            depends_on: vec![],
            output: None,
            turns: vec![],
            audiobook: None,
        };
        if target == Media::Speech && probe.settings.voice.is_none() {
            probe.settings.voice = Some("configuration-placeholder".into());
        }
        if probe.settings.input_fidelity.is_some() {
            probe.references.push(Reference {
                blob: None,
                origin: None,
                role: ReferenceRole::Content,
                name: "edit-only-placeholder.png".into(),
                mime: "image/png".into(),
                sha256: String::new(),
                base64: String::new(),
                transcript: None,
                consent_hash: None,
            });
        }
        providers::validate(&probe)?;
    }
    for (media, locale) in &config.default_locales {
        if !config.defaults.contains_key(media) || locale.trim().is_empty() {
            return Err(Error::input(
                "Default locales require a configured media provider/model and a nonempty locale",
            ));
        }
    }
    Ok(())
}
pub fn limits(l: &Limits) -> Result<()> {
    if [
        l.requests,
        l.images,
        l.audio_seconds,
        l.speech_chars,
        l.revisions,
        l.attempts,
    ]
    .contains(&0)
        || l.concurrency == 0
        || l.per_provider == 0
        || l.concurrency > 32
        || l.per_provider > 16
        || l.attempts > 3
    {
        return Err(Error::input(
            "Limits must be positive; at most 32 run slots, 16 provider slots, and 3 attempts are supported; actual execution also respects user host ceilings",
        ));
    }
    Ok(())
}
pub fn model(s: &Selection) -> Result<()> {
    if s.model.trim().is_empty()
        || s.model.contains("latest")
        || s.model.contains(['?', '#', '\n', '\r'])
    {
        Err(Error::input(
            "Choose an explicit model ID; empty or latest models are not allowed",
        ))
    } else {
        Ok(())
    }
}
pub fn character(path: &Path) -> Result<Character> {
    let c: Character = yaml(path)?;
    version(c.schema_version)?;
    id(&c.id)?;
    let base = path.parent().unwrap_or(Path::new("."));
    text(base, &c.persona)?;
    if let Some(v) = &c.visual {
        model(&v.selection)?;
        text(base, &v.description)?;
        let mut probe = Config::default();
        probe.defaults.insert(Media::Image, v.selection.clone());
        probe
            .default_settings
            .insert(Media::Image, v.settings.clone());
        config(&probe)?;
        for (sha, origin) in &v.origins {
            if sha != &origin.sha256 || origin.selection != v.selection {
                return Err(Error::input(
                    "Character origin metadata conflicts with its pinned model or hash",
                ));
            }
        }
        for p in &v.references {
            let mut r = reference(base, p)?;
            r.origin = v.origins.get(&r.sha256).cloned();
            crate::provenance::check(&r)?;
        }
    }
    if let Some(v) = &c.voice {
        model(&v.selection)?;
        text(base, &v.description)?;
        for p in &v.references {
            reference(base, p)?;
        }
        for sample in &v.samples {
            voice_sample(base, sample, ConsentPurpose::ReferenceSynthesis)?;
        }
        if v.binding.as_ref().is_some_and(|b| b.trim().is_empty()) {
            return Err(Error::input("Voice binding cannot be empty"));
        }
    }
    Ok(c)
}
pub fn consent(base: &Path, path: &Path, purpose: ConsentPurpose) -> Result<String> {
    let path = relative(base, path)?;
    let consent: VoiceConsent = yaml(&path)?;
    version(consent.schema_version)?;
    if !consent.granted
        || consent.subject.trim().is_empty()
        || consent.authorized_by.trim().is_empty()
        || !consent.purposes.contains(&purpose)
    {
        return Err(Error::input(
            "Voice use requires a granted consent record with subject, authorizer, and the requested purpose; do not invent consent",
        ));
    }
    let evidence = text(path.parent().unwrap(), &consent.evidence)?;
    Ok(hash(&serde_json::to_vec(&(
        consent.subject,
        consent.authorized_by,
        consent.granted,
        consent.purposes,
        evidence,
    ))?))
}
pub fn voice_sample(
    base: &Path,
    sample: &VoiceSample,
    purpose: ConsentPurpose,
) -> Result<Reference> {
    let mut r = reference(base, &sample.audio)?;
    if !r.mime.starts_with("audio/") {
        return Err(Error::input("Voice sample must be an audio recording"));
    }
    r.role = ReferenceRole::Identity;
    r.transcript = Some(text(base, &sample.transcript)?);
    r.consent_hash = Some(consent(base, &sample.consent, purpose)?);
    Ok(r)
}
pub fn reference(base: &Path, path: &Path) -> Result<Reference> {
    let bytes = fs::read(relative(base, path)?)?;
    if bytes.is_empty() || bytes.len() > 32 * 1024 * 1024 {
        return Err(Error::input("Reference must contain 1 byte to 32 MiB"));
    }
    let name = path
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or_else(|| Error::input("Reference needs a filename"))?
        .to_owned();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        _ => {
            return Err(Error::input(
                "Reference must be PNG, JPEG, WebP, WAV, MP3, or FLAC",
            ));
        }
    };
    if mime.starts_with("image/") {
        let expected = match ext.as_str() {
            "png" => image::ImageFormat::Png,
            "jpg" | "jpeg" => image::ImageFormat::Jpeg,
            _ => image::ImageFormat::WebP,
        };
        if image::guess_format(&bytes).ok() != Some(expected) {
            return Err(Error::input(
                "Reference image bytes do not match its file extension",
            ));
        }
        image::load_from_memory(&bytes)
            .map_err(|_| Error::input("Reference image cannot be decoded"))?;
    } else {
        let header_matches = match ext.as_str() {
            "wav" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE"),
            "flac" => bytes.starts_with(b"fLaC"),
            "mp3" => {
                bytes.starts_with(b"ID3")
                    || (bytes.first() == Some(&0xff)
                        && bytes.get(1).is_some_and(|b| b & 0xe0 == 0xe0))
            }
            _ => false,
        };
        if !header_matches {
            return Err(Error::input(
                "Reference audio bytes do not match its file extension",
            ));
        }
        crate::audio::decode(&bytes)
            .map_err(|_| Error::input("Reference audio cannot be decoded"))?;
    }
    Ok(Reference {
        blob: None,
        origin: None,
        transcript: None,
        consent_hash: None,
        role: ReferenceRole::Content,
        name,
        mime: mime.into(),
        sha256: hash(&bytes),
        base64: STANDARD.encode(bytes),
    })
}
pub fn infographic(path: &Path) -> Result<(Infographic, String, Vec<Reference>)> {
    let s: Infographic = yaml(path)?;
    version(s.schema_version)?;
    if [&s.title, &s.audience, &s.key_message, &s.canvas, &s.layout]
        .iter()
        .any(|v| v.trim().is_empty())
        || s.sections.is_empty()
    {
        return Err(Error::input(
            "Infographic requires title, audience, key_message, canvas, layout, and ordered sections",
        ));
    }
    if s.sections
        .iter()
        .any(|x| x.heading.trim().is_empty() || x.copy.trim().is_empty())
    {
        return Err(Error::input(
            "Each infographic section requires heading and exact copy",
        ));
    }
    let base = path.parent().unwrap_or(Path::new("."));
    let direction = text(base, &s.visual_brief)?;
    let prompt = format!(
        "Create a readable infographic. Preserve every title, label, number, and unit exactly as supplied. Do not add claims, values, or labels. Ordered sections define reading order. Use clear typography and visual hierarchy.\n\nSTRUCTURED COPY AND LAYOUT:\n{}\n\nVISUAL DIRECTION:\n{}",
        serde_json::to_string_pretty(&s)?,
        direction
    );
    let refs = s
        .references
        .iter()
        .map(|p| {
            let mut r = reference(base, &p.path)?;
            r.role = p.role;
            r.origin = p.origin.clone();
            crate::provenance::check(&r)?;
            Ok(r)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((s, prompt, refs))
}
pub fn resolve(job: &Job, base: &Path, cfg: &Config) -> Result<Effective> {
    resolve_inner(job, base, cfg, None)
}
pub fn resolve_project(root: &Path, job: &Job, base: &Path, cfg: &Config) -> Result<Effective> {
    let index = crate::provenance::Index::load(root)?;
    resolve_inner(job, base, cfg, Some(&index))
}
fn resolve_inner(
    job: &Job,
    base: &Path,
    cfg: &Config,
    index: Option<&crate::provenance::Index>,
) -> Result<Effective> {
    resolve_with(job, base, cfg, index, None)
}
/// `inline` supplies exact speech text already read from a manuscript, in place of `text_file`.
fn resolve_with(
    job: &Job,
    base: &Path,
    cfg: &Config,
    index: Option<&crate::provenance::Index>,
    inline: Option<&str>,
) -> Result<Effective> {
    version2(job.schema_version)?;
    id(&job.id)?;
    if job.schema_version == 1 && (!job.cast.is_empty() || job.assembly.is_some()) {
        return Err(Error::input("cast and assembly require schema_version 2"));
    }
    let audiobook_fields = job.manuscript.is_some()
        || job.split.is_some()
        || !job.chapters.is_empty()
        || job.audiobook.is_some();
    if job.schema_version == 1 && (job.media == Media::Audiobook || audiobook_fields) {
        return Err(Error::input("audiobook requires schema_version 2"));
    }
    if job.media != Media::Audiobook && audiobook_fields {
        return Err(Error::input(
            "manuscript, split, chapters and audiobook are only valid for audiobook jobs",
        ));
    }
    if !job.cast.is_empty() {
        return resolve_cast(job, base, cfg, index);
    }
    if job.media == Media::Audiobook {
        return crate::audiobook::resolve(job, base, cfg, index);
    }
    if let Some(assembly) = &job.assembly {
        if job.media != Media::Podcast {
            return Err(Error::input("assembly is only valid for podcasts"));
        }
        crate::audio::validate_assembly(
            assembly,
            &job.turns.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
        )?;
    }
    for dep in &job.depends_on {
        id(dep)?;
    }
    let mut selection = job
        .selection
        .clone()
        .or_else(|| cfg.defaults.get(&job.media).cloned());
    let mut settings = job.settings.clone();
    let mut visual_settings = Settings::default();
    let mut prompt = match &job.brief {
        Some(p) => text(base, p)?,
        None => String::new(),
    };
    let speech = match (inline, &job.text_file) {
        (Some(words), None) => Some(words.to_owned()),
        (None, Some(p)) => Some(text(base, p)?),
        (None, None) => None,
        (Some(_), Some(_)) => return Err(Error::input("Speech text has two sources")),
    };
    let mut references = job
        .references
        .iter()
        .map(|source| {
            let declared = match source {
                JobReference::Path(_) => None,
                JobReference::Detailed(r) => r.origin.clone(),
            };
            crate::provenance::reference(base, source.path(), source.role(), declared, index)
        })
        .collect::<Result<Vec<_>>>()?;
    if !job.voice_samples.is_empty() && job.media != Media::Speech {
        return Err(Error::input(
            "voice_samples apply to speech; podcast samples belong on individual turns",
        ));
    }
    references.extend(
        job.voice_samples
            .iter()
            .map(|sample| voice_sample(base, sample, ConsentPurpose::ReferenceSynthesis))
            .collect::<Result<Vec<_>>>()?,
    );
    let mut character_hash = None;
    if let Some(p) = &job.character {
        let path = relative(base, p)?;
        let c = character(&path)?;
        let cb = path.parent().unwrap();
        let persona = text(cb, &c.persona)?;
        let mut identity = format!("STABLE IDENTITY:\n{persona}\n");
        let pinned = if matches!(job.media, Media::Image | Media::Infographic) {
            let v = c
                .visual
                .as_ref()
                .ok_or_else(|| Error::input("Character has no visual profile"))?;
            identity += &text(cb, &v.description)?;
            visual_settings = v.settings.clone();
            for p in &v.references {
                let mut r =
                    crate::provenance::reference(cb, p, ReferenceRole::Identity, None, index)?;
                if let Some(declared) = v.origins.get(&r.sha256) {
                    crate::provenance::attach(&mut r, None, Some(declared.clone()), index)?;
                }
                if r.origin.is_none() {
                    r.origin = Some(AssetOrigin {
                        selection: v.selection.clone(),
                        settings: v.settings.clone(),
                        locale: None,
                        sha256: r.sha256.clone(),
                        artifact: None,
                    });
                }
                references.push(r);
            }
            v.selection.clone()
        } else if job.media == Media::Speech {
            let v = c
                .voice
                .as_ref()
                .ok_or_else(|| Error::input("Character has no voice profile"))?;
            identity += &text(cb, &v.description)?;
            references.extend(
                v.references
                    .iter()
                    .map(|p| {
                        let mut r = reference(cb, p)?;
                        r.role = ReferenceRole::Identity;
                        Ok(r)
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
            if let (Some(bound), Some(requested)) = (&v.binding, &settings.voice)
                && bound != requested
            {
                return Err(Error::input(
                    "Voice override conflicts with character binding; edit an explicit alternate character profile",
                ));
            }
            settings.voice = v.binding.clone().or(settings.voice);
            references.extend(
                v.samples
                    .iter()
                    .map(|sample| voice_sample(cb, sample, ConsentPurpose::ReferenceSynthesis))
                    .collect::<Result<Vec<_>>>()?,
            );
            v.selection.clone()
        } else {
            return Err(Error::input(
                "Characters apply to images, infographics, and speech; podcast characters belong to turns",
            ));
        };
        if job.selection.as_ref().is_some_and(|s| *s != pinned) {
            return Err(Error::input(
                "Provider/model override conflicts with pinned character profile",
            ));
        }
        selection = Some(pinned.clone());
        character_hash = Some(hash(&serde_json::to_vec(&(c, &identity))?));
        if job.media == Media::Speech
            && matches!(pinned.provider, Provider::Fish | Provider::Elevenlabs)
        {
            // The saved provider binding carries identity on APIs without an instruction channel.
            // Keep the readable profile in the identity hash, and reject unsupported job direction.
        } else {
            prompt = format!("{identity}\nJOB DIRECTION:\n{prompt}");
        }
    }
    if job.media == Media::Infographic {
        let path = job
            .spec
            .as_ref()
            .ok_or_else(|| Error::input("Infographic requires spec YAML"))?;
        let (_, compiled, refs) = infographic(&relative(base, path)?)?;
        prompt = format!("{prompt}\n{compiled}");
        references.extend(refs);
    } else if job.spec.is_some() {
        return Err(Error::input("spec is only valid for infographics"));
    }
    let mut source_settings = None;
    let mut source_locale = None;
    if matches!(job.media, Media::Image | Media::Infographic) {
        for r in &mut references {
            if r.origin.is_none() {
                crate::provenance::attach(r, None, None, index)?;
            }
            crate::provenance::check(r)?;
        }
        let identities = references
            .iter()
            .filter(|r| matches!(r.role, ReferenceRole::Identity))
            .collect::<Vec<_>>();
        if cfg.consistency.require_known_identity_model
            && identities.iter().any(|r| r.origin.is_none())
        {
            return Err(Error::input(
                "Identity reference has no original model metadata; select a recorded original artifact, a configured character, or declare matching origin metadata in YAML",
            ));
        }
        let mut pinned = None;
        for origin in identities.iter().filter_map(|r| r.origin.as_ref()) {
            if pinned.as_ref().is_some_and(|p| p != &origin.selection) {
                return Err(Error::input(
                    "Selected identity assets were generated with different provider/models; one job cannot silently combine model lineages",
                ));
            }
            pinned = Some(origin.selection.clone());
            if let Some(common) = &mut source_settings {
                Settings::common(common, &origin.settings);
            } else {
                source_settings = Some(origin.settings.clone());
                source_locale = origin.locale.clone();
            }
            if source_locale != origin.locale {
                source_locale = None;
            }
        }
        if let Some(pinned) = pinned {
            if job
                .selection
                .as_ref()
                .is_some_and(|requested| requested != &pinned)
                || (job.character.is_some() && selection.as_ref() != Some(&pinned))
            {
                return Err(Error::input(
                    "Requested provider/model differs from the original identity asset; reuse the original selection instead of changing the character's model",
                ));
            }
            selection = Some(pinned);
        }
    }
    let defaults_match = selection
        .as_ref()
        .is_some_and(|selected| cfg.defaults.get(&job.media) == Some(selected));
    let mut inherited = if job.media != Media::Podcast && defaults_match {
        cfg.default_settings
            .get(&job.media)
            .cloned()
            .unwrap_or_default()
    } else {
        Settings::default()
    };
    if references.is_empty() {
        inherited.input_fidelity = None;
    }
    if cfg.consistency.inherit_source_settings
        && let Some(source) = &source_settings
    {
        inherited.overlay(source);
    }
    inherited.overlay(&visual_settings);
    inherited.overlay(&settings);
    if references.is_empty() && job.settings.input_fidelity.is_none() {
        inherited.input_fidelity = None;
    }
    settings = inherited;
    let locale = job.locale.clone().or(source_locale).or_else(|| {
        if defaults_match {
            cfg.default_locales.get(&job.media).cloned()
        } else {
            None
        }
    });
    if matches!(job.media, Media::Image | Media::Infographic) {
        // Identity references take priority over style/layout/content images.
        references.sort_by_key(|r| !matches!(r.role, ReferenceRole::Identity));
        if references
            .iter()
            .any(|r| matches!(r.role, ReferenceRole::Identity))
        {
            prompt.push_str("\nIDENTITY IS AN ACCEPTANCE REQUIREMENT: preserve the exact individual in each identity reference: facial geometry, eye shape, hair silhouette, head-to-body ratio, limb lengths, material/rendering style, and distinguishing details. Change only the requested scene/action. A similar costume or palette is insufficient. Do not blend identities, redesign anatomy, change apparent age, or reinterpret the reference as inspiration.\n");
            if selection.as_ref().is_some_and(|s| {
                s.provider == Provider::Openai
                    && matches!(s.model.as_str(), "gpt-image-1" | "gpt-image-1.5")
            }) {
                match settings.input_fidelity.as_deref() {
                    None => settings.input_fidelity = Some("high".into()),
                    Some("high") => {}
                    _ => {
                        return Err(Error::input(
                            "Identity references require input_fidelity high for this model; low fidelity is not accepted",
                        ));
                    }
                }
            }
        }
    }
    if matches!(job.media, Media::Image | Media::Infographic) && !references.is_empty() {
        prompt.push_str("\nREFERENCE GUIDE (same order as attached images):\n");
        for (i, r) in references.iter().enumerate() {
            prompt += &format!(
                "Reference {} ({}): {:?}. {}\n",
                i + 1,
                r.name,
                r.role,
                match r.role {
                    ReferenceRole::Identity => "Preserve this character's identity.",
                    ReferenceRole::Style => "Borrow visual style only.",
                    ReferenceRole::Layout => "Use this layout structure only.",
                    ReferenceRole::Content => "Use the supplied content as a visual reference.",
                }
            );
        }
    }
    if job.media == Media::Speech && speech.is_none() {
        return Err(Error::input(
            "Speech requires text_file with exact spoken words",
        ));
    }
    if matches!(
        job.media,
        Media::Image | Media::Music | Media::Sfx | Media::Text
    ) && prompt.trim().is_empty()
    {
        return Err(Error::input("Generation requires a nonempty brief"));
    }
    if job.media != Media::Speech && speech.is_some() {
        return Err(Error::input("text_file is only valid for speech"));
    }
    if job.locale.as_ref().is_some_and(|l| l.trim().is_empty()) {
        return Err(Error::input("locale cannot be empty"));
    }
    let mut turns = Vec::new();
    if job.media == Media::Podcast {
        if settings != Settings::default() {
            return Err(Error::input(
                "Configure podcast voice and generation settings on each turn; top-level settings are not applied",
            ));
        }
        if job.turns.is_empty() {
            return Err(Error::input("Podcast requires ordered turns"));
        }
        for t in &job.turns {
            let mut turn_settings = if t.character.is_none()
                && t.selection
                    .as_ref()
                    .is_none_or(|selected| cfg.defaults.get(&Media::Podcast) == Some(selected))
            {
                cfg.default_settings
                    .get(&Media::Podcast)
                    .cloned()
                    .unwrap_or_default()
            } else {
                Settings::default()
            };
            turn_settings.overlay(&t.settings);
            let turn = Job {
                cast: vec![],
                assembly: None,
                voice_samples: t.voice_samples.clone(),
                schema_version: 1,
                id: t.id.clone(),
                media: Media::Speech,
                selection: t.selection.clone().or_else(|| {
                    if t.character.is_some() {
                        None
                    } else {
                        selection.clone()
                    }
                }),
                brief: job.brief.clone(),
                text_file: Some(t.text_file.clone()),
                character: t.character.clone(),
                spec: None,
                references: vec![],
                turns: vec![],
                settings: turn_settings,
                locale: locale.clone(),
                depends_on: vec![],
                output: None,
                manuscript: None,
                split: None,
                chapters: vec![],
                audiobook: None,
            };
            turns.push(resolve_inner(&turn, base, cfg, index)?);
        }
        let ids: BTreeSet<_> = turns.iter().map(|t| &t.id).collect();
        if ids.len() != turns.len() {
            return Err(Error::input("Podcast turn IDs must be unique"));
        }
    } else if !job.turns.is_empty() {
        return Err(Error::input("turns are only valid for podcast jobs"));
    }
    let selection = selection.or_else(|| turns.first().map(|turn|turn.selection.clone())).ok_or_else(|| {
        Error::input("Set provider/model explicitly, configure defaults, or specify each podcast turn's character/selection")
    })?;
    model(&selection)?;
    let mut input = Effective {
        assembly: job.assembly.clone(),
        id: job.id.clone(),
        media: job.media,
        selection,
        prompt,
        text: speech,
        locale,
        settings,
        character_hash,
        references,
        depends_on: job.depends_on.clone(),
        output: job
            .output
            .as_ref()
            .map(|p| absolute(&base.join(p)))
            .transpose()?,
        turns,
        audiobook: None,
    };
    if input.media == Media::Speech
        && let Some(limit) = providers::speech_limit(&input.selection)
        && input
            .text
            .as_ref()
            .is_some_and(|t| t.chars().count() > limit)
    {
        let source = input.text.clone().unwrap();
        for (i, part) in split_script(&source, limit).into_iter().enumerate() {
            let mut turn = input.clone();
            turn.id = format!("part-{:04}", i + 1);
            turn.text = Some(part);
            turn.turns.clear();
            turn.output = None;
            turn.depends_on.clear();
            input.turns.push(turn);
        }
    }
    providers::validate(&input)?;
    Ok(input)
}
pub fn split_script(source: &str, max_chars: usize) -> Vec<String> {
    let mut remaining = source;
    let mut parts = vec![];
    while remaining.chars().count() > max_chars {
        let end = remaining
            .char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap();
        let boundary = remaining[..end]
            .char_indices()
            .filter(|(i, c)| *i > end / 2 && c.is_whitespace())
            .map(|(i, c)| i + c.len_utf8())
            .next_back()
            .unwrap_or(end);
        parts.push(remaining[..boundary].to_owned());
        remaining = &remaining[boundary..];
    }
    if !remaining.is_empty() {
        parts.push(remaining.to_owned());
    }
    parts
}
/// Resolve one audiobook chapter as speech with exact in-memory text.
pub(crate) fn resolve_chapter(
    job: &Job,
    base: &Path,
    cfg: &Config,
    index: Option<&crate::provenance::Index>,
    words: &str,
) -> Result<Effective> {
    resolve_with(job, base, cfg, index, Some(words))
}
fn paragraph_break(s: &str) -> Option<usize> {
    s.match_indices('\n').map(|(i, _)| i).find(|&i| {
        let after = &s[i + 1..];
        after.strip_prefix('\r').unwrap_or(after).starts_with('\n')
    })
}
/// Paragraph-aligned chunks that concatenate back to `source` exactly.
///
/// Whole paragraphs are packed greedily; an oversized paragraph is cut after a
/// sentence end in the second half of the window, else at whitespace, else at
/// a character boundary. Paragraph alignment keeps an edit early in a chapter
/// from moving later boundaries, so unchanged parts stay cached.
pub fn chunk_paragraphs(source: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    let mut paragraphs = vec![];
    let mut rest = source;
    while !rest.is_empty() {
        let end = match paragraph_break(rest) {
            Some(i) => rest[i..]
                .char_indices()
                .find(|(_, c)| !matches!(c, '\n' | '\r'))
                .map_or(rest.len(), |(j, _)| i + j),
            None => rest.len(),
        };
        paragraphs.push(&rest[..end]);
        rest = &rest[end..];
    }
    let mut chunks: Vec<String> = vec![];
    let mut current = String::new();
    let mut count = 0;
    for paragraph in paragraphs {
        let len = paragraph.chars().count();
        if count + len <= max_chars {
            current.push_str(paragraph);
            count += len;
            continue;
        }
        if !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
        }
        if len <= max_chars {
            current.push_str(paragraph);
            count = len;
            continue;
        }
        let mut remaining = paragraph;
        while remaining.chars().count() > max_chars {
            let end = remaining
                .char_indices()
                .nth(max_chars)
                .map(|(i, _)| i)
                .unwrap();
            let window = &remaining[..end];
            let chars: Vec<(usize, char)> = window.char_indices().collect();
            let sentence = chars
                .windows(2)
                .filter(|w| {
                    w[0].0 > end / 2
                        && matches!(w[0].1, '.' | '!' | '?' | '…' | '。' | '！' | '？')
                        && w[1].1.is_whitespace()
                })
                .map(|w| w[1].0 + w[1].1.len_utf8())
                .next_back();
            let space = || {
                window
                    .char_indices()
                    .filter(|(i, c)| *i > end / 2 && c.is_whitespace())
                    .map(|(i, c)| i + c.len_utf8())
                    .next_back()
            };
            let boundary = sentence.or_else(space).unwrap_or(end);
            chunks.push(remaining[..boundary].to_owned());
            remaining = &remaining[boundary..];
        }
        current.push_str(remaining);
        count = remaining.chars().count();
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    // Never send a whitespace-only request: fold it into the previous chunk.
    let mut out: Vec<String> = vec![];
    for chunk in chunks {
        match out.last_mut() {
            Some(last) if chunk.trim().is_empty() => last.push_str(&chunk),
            _ => out.push(chunk),
        }
    }
    if out.len() > 1 && out[0].trim().is_empty() {
        let first = out.remove(0);
        out[0].insert_str(0, &first);
    }
    out
}
pub fn graph(jobs: &[Effective]) -> Result<()> {
    if jobs.is_empty() {
        return Err(Error::input("Batch requires at least one job"));
    }
    let mut ids = BTreeSet::new();
    for j in jobs {
        if !ids.insert(j.id.clone()) {
            return Err(Error::input("Duplicate job ID"));
        }
    }
    let mut done = BTreeSet::new();
    while done.len() < jobs.len() {
        let before = done.len();
        for j in jobs {
            if j.depends_on.iter().any(|d| !ids.contains(d)) {
                return Err(Error::input("Dependency refers to a missing job"));
            }
            if j.depends_on.iter().all(|d| done.contains(d)) {
                done.insert(j.id.clone());
            }
        }
        if done.len() == before {
            return Err(Error::input("Job dependencies contain a cycle"));
        }
    }
    let mut outputs = BTreeSet::new();
    for j in jobs {
        if let Some(p) = &j.output
            && !outputs.insert(p.clone())
        {
            return Err(Error::input("Jobs cannot share an output destination"));
        }
    }
    Ok(())
}
pub fn fingerprint(input: &Effective) -> Result<String> {
    let mut content = input.clone();
    content.id = String::new();
    content.output = None;
    content.depends_on.clear();
    Ok(hash(&serde_json::to_vec(&content)?))
}

fn resolve_cast(
    job: &Job,
    base: &Path,
    cfg: &Config,
    index: Option<&crate::provenance::Index>,
) -> Result<Effective> {
    if job.character.is_some() || !matches!(job.media, Media::Image | Media::Infographic) {
        return Err(Error::input(
            "cast applies only to images/infographics and conflicts with character",
        ));
    }
    let mut aliases = BTreeSet::new();
    let mut combined = job.clone();
    combined.cast.clear();
    let mut pinned = None;
    let mut settings = None;
    let mut identities = Vec::new();
    let mut refs = Vec::new();
    let mut hashes = Vec::new();
    for member in &job.cast {
        id(&member.alias)?;
        if !aliases.insert(&member.alias) {
            return Err(Error::input("Cast aliases must be unique"));
        }
        let profile = character(&relative(base, &member.character)?)?
            .visual
            .ok_or_else(|| Error::input("Cast character has no visual profile"))?;
        if profile.references.is_empty() {
            return Err(Error::input(
                "Each cast member needs accepted visual references; bootstrap and accept first",
            ));
        }
        let mut single = combined.clone();
        single.character = Some(member.character.clone());
        single.references.clear();
        // Resolve visual identity without duplicating the scene or infographic content.
        single.media = Media::Image;
        single.spec = None;
        single.brief = None;
        single.depends_on.clear();
        let mut input = resolve_inner(&single, base, cfg, index)?;
        if input.references.is_empty() {
            return Err(Error::input(
                "Each cast member needs accepted visual references; bootstrap and accept first",
            ));
        }
        if pinned.as_ref().is_some_and(|p| p != &input.selection) {
            return Err(Error::input(
                "Cast members require conflicting provider/models",
            ));
        }
        if settings.as_ref().is_some_and(|s| s != &profile.settings) {
            return Err(Error::input(
                "Cast members require conflicting identity settings; use compatible explicit profiles",
            ));
        }
        settings = Some(profile.settings);
        pinned = Some(input.selection.clone());
        let direction = member
            .direction
            .as_ref()
            .map(|p| text(base, p))
            .transpose()?
            .unwrap_or_default();
        for (i, r) in input.references.iter_mut().enumerate() {
            r.name = format!("{}-identity-{}", member.alias, i + 1);
        }
        hashes.push((
            member.alias.clone(),
            input.character_hash,
            direction.clone(),
        ));
        identities.push(format!(
            "CAST {}:\n{}\nSCENE DIRECTION FOR {}:\n{}",
            member.alias,
            input
                .prompt
                .split("\nJOB DIRECTION:")
                .next()
                .unwrap_or(&input.prompt),
            member.alias,
            direction
        ));
        refs.extend(input.references);
    }
    // A skeleton resolves scene settings and additional content references with the same pin.
    combined.selection = pinned;
    if let Some(mut inherited) = settings {
        inherited.overlay(&combined.settings);
        combined.settings = inherited;
    }
    for reference in &refs {
        // The final resolver needs the immutable bytes and provenance, not synthetic file paths.
        crate::provenance::check(reference)?;
    }
    // Identity edit fidelity was already validated on every member. The scene-only
    // skeleton may have no references until the resolved cast is attached below.
    if combined.references.is_empty() {
        combined.settings.input_fidelity = None;
    }
    let mut out = resolve_inner(&combined, base, cfg, index)?;
    refs.extend(out.references);
    if refs.len() > 8 {
        return Err(Error::input(
            "Combined cast exceeds the CLI limit of eight image references",
        ));
    }
    refs.sort_by_key(|r| !matches!(r.role, ReferenceRole::Identity));
    out.references = refs;
    out.character_hash = Some(hash(&serde_json::to_vec(&hashes)?));
    out.prompt = format!("{}\nSCENE:\n{}", identities.join("\n"), out.prompt);
    out.prompt.push_str("\nPreserve each selected identity exactly: facial geometry, proportions, materials, colors and distinguishing details. Do not blend or redesign characters.\nATTACHMENT ORDER:\n");
    for (i, r) in out.references.iter().enumerate() {
        out.prompt += &format!("{}: {} ({:?})\n", i + 1, r.name, r.role);
    }
    if out.selection.provider == Provider::Openai
        && matches!(
            out.selection.model.as_str(),
            "gpt-image-1" | "gpt-image-1.5"
        )
    {
        out.settings.input_fidelity = Some("high".into());
    }
    providers::validate(&out)?;
    Ok(out)
}
