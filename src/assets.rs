use crate::{cli, error::Result, storage};
use std::path::Path;

pub const TEMPLATES: &[(&str, &str)] = &[
    (
        "reference-sheet.md",
        include_str!("../assets/templates/reference-sheet.md"),
    ),
    (
        "identity-checklist.md",
        include_str!("../assets/templates/identity-checklist.md"),
    ),
    (
        "cast-scene.yaml",
        include_str!("../assets/templates/cast-scene.yaml"),
    ),
    (
        "verification.yaml",
        include_str!("../assets/templates/verification.yaml"),
    ),
    (
        "recovery-evidence.md",
        include_str!("../assets/templates/recovery-evidence.md"),
    ),
    (
        "acceptance-review.yaml",
        include_str!("../assets/templates/acceptance-review.yaml"),
    ),
    (
        "project-config.yaml",
        include_str!("../assets/templates/project-config.yaml"),
    ),
    ("persona.md", include_str!("../assets/templates/persona.md")),
    ("visual.md", include_str!("../assets/templates/visual.md")),
    ("voice.md", include_str!("../assets/templates/voice.md")),
    (
        "voice-preview.md",
        include_str!("../assets/templates/voice-preview.md"),
    ),
    (
        "voice-consent.yaml",
        include_str!("../assets/templates/voice-consent.yaml"),
    ),
    (
        "voice-permission.md",
        include_str!("../assets/templates/voice-permission.md"),
    ),
    (
        "voice-sample.yaml",
        include_str!("../assets/templates/voice-sample.yaml"),
    ),
    (
        "voice-transcript.md",
        include_str!("../assets/templates/voice-transcript.md"),
    ),
    (
        "voice-clone.yaml",
        include_str!("../assets/templates/voice-clone.yaml"),
    ),
    (
        "host-limits.yaml",
        include_str!("../assets/templates/host-limits.yaml"),
    ),
    (
        "character-scene.yaml",
        include_str!("../assets/templates/character-scene.yaml"),
    ),
    (
        "character-scene.md",
        include_str!("../assets/templates/character-scene.md"),
    ),
    ("image.md", include_str!("../assets/templates/image.md")),
    ("script.md", include_str!("../assets/templates/script.md")),
    ("music.md", include_str!("../assets/templates/music.md")),
    ("sfx.md", include_str!("../assets/templates/sfx.md")),
    (
        "infographic.md",
        include_str!("../assets/templates/infographic.md"),
    ),
    (
        "infographic.yaml",
        include_str!("../assets/templates/infographic.yaml"),
    ),
    ("job.yaml", include_str!("../assets/templates/job.yaml")),
    (
        "podcast.yaml",
        include_str!("../assets/templates/podcast.yaml"),
    ),
    (
        "audiobook.yaml",
        include_str!("../assets/templates/audiobook.yaml"),
    ),
    (
        "audiobook-chapters.yaml",
        include_str!("../assets/templates/audiobook-chapters.yaml"),
    ),
    (
        "manuscript.md",
        include_str!("../assets/templates/manuscript.md"),
    ),
    (
        "narration.md",
        include_str!("../assets/templates/narration.md"),
    ),
    (
        "chapters/chapter-01.md",
        include_str!("../assets/templates/chapters/chapter-01.md"),
    ),
    (
        "chapters/chapter-02.md",
        include_str!("../assets/templates/chapters/chapter-02.md"),
    ),
    (
        "examples/narrator/character.yaml",
        include_str!("../assets/templates/examples/narrator/character.yaml"),
    ),
    (
        "examples/narrator/persona.md",
        include_str!("../assets/templates/persona.md"),
    ),
    (
        "examples/narrator/voice.md",
        include_str!("../assets/templates/voice.md"),
    ),
    (
        "examples/mascot/character.yaml",
        include_str!("../assets/templates/examples/mascot/character.yaml"),
    ),
    (
        "examples/mascot/persona.md",
        include_str!("../assets/templates/persona.md"),
    ),
    (
        "examples/mascot/visual.md",
        include_str!("../assets/templates/visual.md"),
    ),
    (
        "examples/combined/character.yaml",
        include_str!("../assets/templates/examples/combined/character.yaml"),
    ),
    (
        "examples/combined/persona.md",
        include_str!("../assets/templates/persona.md"),
    ),
    (
        "examples/combined/visual.md",
        include_str!("../assets/templates/visual.md"),
    ),
    (
        "examples/combined/voice.md",
        include_str!("../assets/templates/voice.md"),
    ),
];
pub fn templates(path: &Path) -> Result<Vec<String>> {
    templates_with(path, false)
}
fn write(path: &Path, bytes: &[u8], overwrite: bool) -> Result<bool> {
    if overwrite {
        storage::atomic(path, bytes, true)?;
        Ok(true)
    } else {
        storage::create(path, bytes)
    }
}
pub fn templates_with(path: &Path, overwrite: bool) -> Result<Vec<String>> {
    let mut existing = vec![];
    for (name, data) in TEMPLATES {
        if !write(&path.join(name), data.as_bytes(), overwrite)? {
            existing.push((*name).into());
        }
    }
    Ok(existing)
}
pub fn skill(path: &Path) -> Result<Vec<String>> {
    skill_with(path, false)
}
pub fn skill_with(path: &Path, overwrite: bool) -> Result<Vec<String>> {
    let mut existing = vec![];
    for (name, data) in [
        (
            "SKILL.md",
            include_str!("../assets/skill/SKILL.md").to_owned(),
        ),
        (
            "references/setup.md",
            include_str!("../assets/skill/references/setup.md").to_owned(),
        ),
        (
            "references/installation.md",
            include_str!("../assets/skill/references/installation.md").to_owned(),
        ),
        (
            "references/renaming.md",
            include_str!("../assets/skill/references/renaming.md").to_owned(),
        ),
        (
            "references/workflow.md",
            include_str!("../assets/skill/references/workflow.md").to_owned(),
        ),
        (
            "references/authoring.md",
            include_str!("../assets/skill/references/authoring.md").to_owned(),
        ),
        (
            "references/voices.md",
            include_str!("../assets/skill/references/voices.md").to_owned(),
        ),
        (
            "references/audiobooks.md",
            include_str!("../assets/skill/references/audiobooks.md").to_owned(),
        ),
        (
            "references/consistency.md",
            include_str!("../assets/skill/references/consistency.md").to_owned(),
        ),
        (
            "references/connections.md",
            include_str!("../docs/CONNECTIONS.md").to_owned(),
        ),
        (
            "references/remediation.md",
            include_str!("../docs/REMEDIATION.md").to_owned(),
        ),
        ("references/commands.md", cli::command_reference()),
    ] {
        if !write(&path.join(name), data.as_bytes(), overwrite)? {
            existing.push(name.to_owned());
        }
    }
    for name in cli::SCHEMAS {
        write(
            &path.join(format!("references/schemas/{name}.json")),
            serde_json::to_vec_pretty(&cli::schema(name)?)?.as_slice(),
            overwrite,
        )?;
    }
    existing.extend(templates_with(&path.join("templates"), overwrite)?);
    Ok(existing)
}
