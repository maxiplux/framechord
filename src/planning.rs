//! Validate model-authored bundles before reading dependencies or publishing files.
use crate::{
    contracts::*,
    error::{Error, Result},
    storage::*,
    validation,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Files {
    files: Vec<PlannedFile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlannedFile {
    path: PathBuf,
    content: String,
}

pub fn stage(content: &str, kind: &str, directory: &Path, config: &Config) -> Result<PathBuf> {
    if !matches!(kind, "character" | "job" | "infographic") {
        return Err(Error::input("Unsupported plan kind"));
    }
    if content.len() > 1024 * 1024 {
        return Err(Error::input(
            "Planner bundle exceeds 1 MiB; raw response remains in the run",
        ));
    }
    let files: Files = serde_json::from_str(content.trim()).map_err(|_| {
        Error::input("Planner output was not valid file JSON; raw response remains in the run")
    })?;
    if files.files.is_empty() || files.files.len() > 32 {
        return Err(Error::input("Planner bundle requires 1–32 authoring files"));
    }
    let mut names = BTreeSet::new();
    // Check the whole list before creating even the first file.
    for file in &files.files {
        authoring_path(&file.path)?;
        if file.path.starts_with("_inputs") {
            return Err(Error::input(
                "Planner cannot replace reserved selected assets",
            ));
        }
        if !names.insert(file.path.clone()) {
            return Err(Error::input("Planner emitted duplicate file paths"));
        }
        if !matches!(
            file.path.extension().and_then(|s| s.to_str()),
            Some("yaml" | "yml" | "md" | "txt")
        ) {
            return Err(Error::input(
                "Planner may author only YAML, Markdown, and text files",
            ));
        }
    }
    for file in files.files {
        atomic(&directory.join(file.path), file.content.as_bytes(), false)?;
    }
    let root = std::fs::canonicalize(directory)?;
    let manifest = root.join(format!("{kind}.yaml"));
    match kind {
        "character" => {
            bind_character_inputs(&root, &manifest)?;
            contained_character(&root, &manifest)?;
            validation::character(&manifest)?;
        }
        "infographic" => {
            bind_infographic_inputs(&root, &manifest)?;
            contained_infographic(&root, &manifest)?;
            validation::infographic(&manifest)?;
        }
        "job" => {
            let mut job: Job = yaml(&manifest)?;
            bind_selected_assets(&root, &mut job)?;
            save(&manifest, &job)?;
            let base = manifest.parent().unwrap();
            for member in &job.cast {
                let path = dependency(&root, base, &member.character)?;
                bind_character_inputs(&root, &path)?;
                contained_character(&root, &path)?;
                if let Some(path) = &member.direction {
                    dependency(&root, base, path)?;
                }
            }
            for sample in &job.voice_samples {
                contained_sample(&root, base, sample)?;
            }
            for path in [&job.brief, &job.text_file].into_iter().flatten() {
                dependency(&root, base, path)?;
            }
            for reference in &job.references {
                dependency(&root, base, reference.path())?;
            }
            if let Some(path) = &job.character {
                let path = dependency(&root, base, path)?;
                bind_character_inputs(&root, &path)?;
                contained_character(&root, &path)?;
            }
            if let Some(path) = &job.spec {
                let path = dependency(&root, base, path)?;
                bind_infographic_inputs(&root, &path)?;
                contained_infographic(&root, &path)?;
            }
            let sources: Vec<PathBuf> = root
                .join("_inputs/sources")
                .read_dir()
                .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
                .unwrap_or_default();
            let mut narrated = vec![];
            if let Some(path) = &job.manuscript {
                narrated.push(dependency(&root, base, path)?);
            }
            for chapter in &job.chapters {
                narrated.push(dependency(&root, base, &chapter.text_file)?);
                if let Some(path) = &chapter.brief {
                    dependency(&root, base, path)?;
                }
            }
            // With selected source texts, the narrated words must be exactly those files,
            // each narrated once and none left out.
            let selected = sources
                .iter()
                .map(std::fs::canonicalize)
                .collect::<std::io::Result<BTreeSet<_>>>()?;
            let unique: BTreeSet<_> = narrated.iter().cloned().collect();
            if job.media == Media::Audiobook
                && !selected.is_empty()
                && (unique != selected || unique.len() != narrated.len())
            {
                return Err(Error::input(
                    "Planned audiobook must narrate the selected _inputs/sources files exactly; the planner may not author book text",
                ));
            }
            for turn in &job.turns {
                for sample in &turn.voice_samples {
                    contained_sample(&root, base, sample)?;
                }
                dependency(&root, base, &turn.text_file)?;
                if let Some(path) = &turn.character {
                    let path = dependency(&root, base, path)?;
                    bind_character_inputs(&root, &path)?;
                    contained_character(&root, &path)?;
                }
            }
            if let Some(path) = &job.output {
                authoring_path(path)?;
            }
            validation::resolve(&job, base, config)?;
        }
        _ => unreachable!(),
    }
    Ok(manifest)
}

fn authoring_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(Error::input("Planner emitted a path outside its bundle"));
    }
    Ok(())
}
fn dependency(root: &Path, base: &Path, path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Err(Error::input(
            "Planner dependency must be relative and inside its bundle",
        ));
    }
    let resolved = relative(base, path)?;
    if !resolved.starts_with(root) {
        return Err(Error::input("Planner dependency escapes its bundle"));
    }
    Ok(resolved)
}
fn contained_character(root: &Path, manifest: &Path) -> Result<()> {
    let character: Character = yaml(manifest)?;
    let base = manifest.parent().unwrap();
    dependency(root, base, &character.persona)?;
    if let Some(profile) = &character.visual {
        dependency(root, base, &profile.description)?;
        for path in &profile.references {
            dependency(root, base, path)?;
        }
    }
    if let Some(profile) = &character.voice {
        dependency(root, base, &profile.description)?;
        for path in &profile.references {
            dependency(root, base, path)?;
        }
        for sample in &profile.samples {
            contained_sample(root, base, sample)?;
        }
    }
    Ok(())
}
fn contained_sample(root: &Path, base: &Path, sample: &VoiceSample) -> Result<()> {
    dependency(root, base, &sample.audio)?;
    dependency(root, base, &sample.transcript)?;
    let path = dependency(root, base, &sample.consent)?;
    let consent: VoiceConsent = yaml(&path)?;
    dependency(root, path.parent().unwrap(), &consent.evidence)?;
    Ok(())
}
fn contained_infographic(root: &Path, manifest: &Path) -> Result<()> {
    let spec: Infographic = yaml(manifest)?;
    let base = manifest.parent().unwrap();
    dependency(root, base, &spec.visual_brief)?;
    for reference in &spec.references {
        dependency(root, base, &reference.path)?;
    }
    Ok(())
}

/// Copy only caller-selected inputs before sending any planning request.
/// Return a readable inventory; never send image/audio bytes to a text planner.
pub fn prepare_inputs(
    root: &Path,
    directory: &Path,
    characters: &[PathBuf],
    assets: &[PathBuf],
) -> Result<serde_json::Value> {
    let index = crate::provenance::Index::load(root)?;
    let mut inventory = Vec::new();
    let mut ids = BTreeSet::new();
    for source in characters {
        let source = storage_path(source)?;
        let character = validation::character(&source)?;
        if !ids.insert(character.id.clone()) {
            return Err(Error::input("Selected character IDs must be unique"));
        }
        let relative = PathBuf::from("_inputs/characters").join(&character.id);
        let manifest = crate::project::bundle(&source, &directory.join(&relative), None)?;
        let mut copied: Character = yaml(&manifest)?;
        if let Some(v) = &mut copied.visual {
            let original = character.visual.as_ref().unwrap();
            for (from, to) in original.references.iter().zip(&v.references) {
                let r = crate::provenance::reference(
                    source.parent().unwrap(),
                    from,
                    ReferenceRole::Identity,
                    None,
                    Some(&index),
                )?;
                if let Some(origin) = r.origin {
                    v.origins.insert(r.sha256, origin);
                }
                validation::reference(manifest.parent().unwrap(), to)?;
            }
        }
        save(&manifest, &copied)?;
        inventory.push(serde_json::json!({"kind":"character","manifest":relative.join("character.yaml"),"definition":copied,"persona":text(source.parent().unwrap(), &character.persona)?}));
    }
    for (i, source) in assets.iter().enumerate() {
        let source = storage_path(source)?;
        let mut reference = validation::reference(source.parent().unwrap(), &source)?;
        crate::provenance::attach(&mut reference, Some(&source), None, Some(&index))?;
        let relative = PathBuf::from("_inputs/assets").join(format!(
            "{i}-{}",
            source.file_name().unwrap().to_string_lossy()
        ));
        atomic(&directory.join(&relative), &std::fs::read(&source)?, false)?;
        inventory.push(serde_json::json!({"kind":"asset","path":relative,"mime":reference.mime,"sha256":reference.sha256,"origin":reference.origin}));
    }
    let inventory = serde_json::json!(inventory);
    if !inventory.as_array().unwrap().is_empty() {
        save(&directory.join("_inputs/inventory.yaml"), &inventory)?;
    }
    Ok(inventory)
}
/// Copy exact source texts (such as a manuscript) into `_inputs/sources`.
/// The planner sees only paths, sizes and hashes, never the words themselves.
pub fn prepare_sources(
    directory: &Path,
    inventory: serde_json::Value,
    sources: &[PathBuf],
) -> Result<serde_json::Value> {
    if sources.is_empty() {
        return Ok(inventory);
    }
    let mut items = inventory.as_array().cloned().unwrap_or_default();
    for (i, source) in sources.iter().enumerate() {
        let source = storage_path(source)?;
        if !matches!(
            source.extension().and_then(|e| e.to_str()),
            Some("md" | "txt")
        ) {
            return Err(Error::input(
                "Planning sources must be Markdown or text files",
            ));
        }
        let bytes = std::fs::read(&source)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::input("Planning sources must be at most 16 MiB"));
        }
        let words = String::from_utf8(bytes)
            .map_err(|_| Error::input("Planning sources must be UTF-8 text"))?;
        if words.trim().is_empty() {
            return Err(Error::input("Planning source is empty"));
        }
        // Forward slashes on every platform: the planner writes this path into portable YAML.
        let relative = format!(
            "_inputs/sources/{i}-{}",
            source.file_name().unwrap().to_string_lossy()
        );
        atomic(&directory.join(&relative), words.as_bytes(), false)?;
        // Metadata only: no source words (headings included) reach the planner.
        items.push(serde_json::json!({
            "kind": "source",
            "path": relative,
            "sha256": crate::storage::hash(words.as_bytes()),
            "chars": words.chars().count(),
        }));
    }
    let inventory = serde_json::json!(items);
    save(&directory.join("_inputs/inventory.yaml"), &inventory)?;
    Ok(inventory)
}
fn storage_path(path: &Path) -> Result<PathBuf> {
    Ok(std::fs::canonicalize(path)?)
}

fn bind_selected_assets(root: &Path, job: &mut Job) -> Result<()> {
    let inventory = root.join("_inputs/inventory.yaml");
    if !inventory.exists() {
        return Ok(());
    }
    let inventory: serde_json::Value = yaml(&inventory)?;
    for r in &mut job.references {
        let path = dependency(root, root, r.path())?;
        for selected in inventory
            .as_array()
            .into_iter()
            .flatten()
            .filter(|v| v["kind"] == "asset")
        {
            let selected_path =
                dependency(root, root, Path::new(selected["path"].as_str().unwrap()))?;
            if path != selected_path {
                continue;
            }
            let origin: Option<AssetOrigin> = serde_json::from_value(selected["origin"].clone())?;
            if matches!(r.role(), ReferenceRole::Identity)
                && let Some(origin) = origin
            {
                if let JobReference::Detailed(d) = r
                    && d.origin.as_ref().is_some_and(|o| {
                        o.selection != origin.selection
                            || o.sha256 != origin.sha256
                            || o.settings != origin.settings
                    })
                {
                    return Err(Error::input("Planner changed selected asset provenance"));
                }
                *r = JobReference::Detailed(Box::new(InfoReference {
                    path: r.path().to_owned(),
                    role: r.role(),
                    origin: Some(origin),
                }));
            }
        }
    }
    Ok(())
}

fn selected_origin(root: &Path, path: &Path) -> Result<Option<AssetOrigin>> {
    let inventory = root.join("_inputs/inventory.yaml");
    if !inventory.exists() {
        return Ok(None);
    }
    let values: serde_json::Value = yaml(&inventory)?;
    for v in values
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v["kind"] == "asset")
    {
        if dependency(root, root, Path::new(v["path"].as_str().unwrap()))? == path {
            return Ok(serde_json::from_value(v["origin"].clone())?);
        }
    }
    Ok(None)
}
fn bind_character_inputs(root: &Path, path: &Path) -> Result<()> {
    if path.starts_with(root.join("_inputs")) {
        return Ok(());
    }
    let mut c: Character = yaml(path)?;
    if let Some(v) = &mut c.visual {
        for reference in &v.references {
            let p = dependency(root, path.parent().unwrap(), reference)?;
            if let Some(origin) = selected_origin(root, &p)? {
                if v.selection != origin.selection {
                    return Err(Error::input(
                        "Planned character model differs from selected original asset",
                    ));
                }
                if v.origins.get(&origin.sha256).is_some_and(|o| {
                    o.selection != origin.selection || o.settings != origin.settings
                }) {
                    return Err(Error::input(
                        "Planner changed selected character provenance",
                    ));
                }
                v.origins.insert(origin.sha256.clone(), origin);
            }
        }
    }
    save(path, &c)
}
fn bind_infographic_inputs(root: &Path, path: &Path) -> Result<()> {
    let mut spec: Infographic = yaml(path)?;
    for reference in &mut spec.references {
        let p = dependency(root, path.parent().unwrap(), &reference.path)?;
        if matches!(reference.role, ReferenceRole::Identity)
            && let Some(origin) = selected_origin(root, &p)?
        {
            if reference.origin.as_ref().is_some_and(|o| {
                o.selection != origin.selection
                    || o.sha256 != origin.sha256
                    || o.settings != origin.settings
            }) {
                return Err(Error::input(
                    "Planner changed selected infographic provenance",
                ));
            }
            reference.origin = Some(origin);
        }
    }
    save(path, &spec)
}
