//! Voice previews stay candidates; saving a provider voice and binding it are explicit.
use crate::{
    contracts::*,
    engine,
    error::{Error, Result},
    operations::{self, Output, Request},
    storage::*,
    validation,
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, path::Path};
pub const DESIGN_MODELS: &[&str] = &["eleven_multilingual_ttv_v2", "eleven_ttv_v3"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub id: String,
    pub audio: Reference,
    pub language: Option<String>,
}
#[async_trait]
pub trait Adapter: Send + Sync {
    async fn design(&self, selection: &Selection, description: &str, text: &str) -> Result<Output>;
    async fn create(&self, name: &str, description: &str, candidate: &str) -> Result<Output>;
    async fn verify(&self, _voice: &str) -> Result<Output> {
        Err(Error::input(
            "This adapter does not support refreshing voice verification",
        ))
    }
    async fn clone_voice(
        &self,
        _name: &str,
        _description: &str,
        _recordings: &[Reference],
    ) -> Result<Output> {
        Err(Error::input(
            "Recorded-voice cloning is unsupported by this adapter",
        ))
    }
}
pub fn validate_design(selection: &Selection, description: &str, text: &str) -> Result<()> {
    if selection.provider != Provider::Elevenlabs
        || !DESIGN_MODELS.contains(&selection.model.as_str())
    {
        return Err(Error::input(
            "Voice design supports ElevenLabs eleven_multilingual_ttv_v2 or eleven_ttv_v3; choose explicitly",
        ));
    }
    if !(20..=1000).contains(&description.chars().count()) {
        return Err(Error::input(
            "Voice design description requires 20–1000 characters",
        ));
    }
    if !(100..=1000).contains(&text.chars().count()) {
        return Err(Error::input(
            "Exact voice preview script requires 100–1000 characters",
        ));
    }
    Ok(())
}
fn identity(path: &Path) -> Result<(Character, String, String)> {
    let c = validation::character(path)?;
    let voice = c.voice.as_ref().ok_or_else(|| {
        Error::input("Enable an explicit ElevenLabs speech profile before voice design")
    })?;
    if voice.selection.provider != Provider::Elevenlabs {
        return Err(Error::input(
            "Voice design does not replace another provider's pinned voice profile",
        ));
    }
    if !crate::providers::registry().iter().any(|cap| {
        cap.provider == voice.selection.provider
            && cap.model == voice.selection.model
            && cap.media == Media::Speech
    }) {
        return Err(Error::input(
            "Choose a supported explicit ElevenLabs speech model for the character before voice design",
        ));
    }
    let base = path.parent().unwrap();
    let description = text(base, &voice.description)?;
    let fingerprint = hash(&serde_json::to_vec(&(
        &c.id,
        text(base, &c.persona)?,
        &description,
        &voice.selection,
    ))?);
    Ok((c, description, fingerprint))
}
pub fn draft(
    root: &Path,
    character: &Path,
    selection: Selection,
    script: &str,
    run_id: Option<&str>,
    limits: Limits,
) -> Result<VoiceDesign> {
    validation::limits(&limits)?;
    let (c, description, identity_hash) = identity(character)?;
    validate_design(&selection, &description, script)?;
    let id = run_id
        .map(str::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    crate::storage::id(&id)?;
    let voice = c.voice.unwrap();
    // Reserve the maximum three previews from one design request, not three calls.
    if script.chars().count() * 3 > limits.speech_chars as usize
        || (script.chars().count() * 3).div_ceil(12) > limits.audio_seconds as usize
    {
        return Err(Error::limit(
            "Voice design budget must cover up to three previews of the exact script",
        ));
    }
    let draft = VoiceDesign {
        schema_version: 1,
        id,
        character_id: c.id,
        identity_hash,
        selection,
        speech_selection: voice.selection,
        binding_before: voice.binding,
        description,
        text: script.into(),
        limits,
        candidates: vec![],
    };
    let path = engine::run_path(root, &draft.id)?.join("voice-design.yaml");
    if path.exists() {
        let existing: VoiceDesign = yaml(&path)?;
        version(existing.schema_version)?;
        if existing.identity_hash != draft.identity_hash
            || existing.selection != draft.selection
            || existing.text != draft.text
            || serde_json::to_value(&existing.limits)? != serde_json::to_value(&draft.limits)?
        {
            return Err(Error::input(
                "Design run ID already belongs to different voice inputs or limits",
            ));
        }
        return Ok(existing);
    }
    Ok(draft)
}
pub async fn design(
    root: &Path,
    mut draft: VoiceDesign,
    retry_unknown: bool,
    adapter: &dyn Adapter,
) -> Result<VoiceDesign> {
    version(draft.schema_version)?;
    validation::limits(&draft.limits)?;
    validate_design(&draft.selection, &draft.description, &draft.text)?;
    let dir = engine::run_path(root, &draft.id)?;
    let guard = lock(&dir.join("voice-design.lock"))?;
    if !guard.try_lock_exclusive()? {
        return Err(Error::new(
            "operation_active",
            "Another process owns this voice design",
            3,
        ));
    }
    let manifest = dir.join("voice-design.yaml");
    if manifest.exists() {
        let saved: VoiceDesign = yaml(&manifest)?;
        if serde_json::to_value(&saved)? != serde_json::to_value(&draft)? {
            return Err(Error::input(
                "Voice design changed concurrently; inspect its saved record",
            ));
        }
    } else {
        save(&manifest, &draft)?;
    }
    let input = json!({"kind":"voice_design","character_id":draft.character_id,"identity_hash":draft.identity_hash,"description":draft.description,"text":draft.text});
    let reservation = Effective {
        assembly: None,
        id: "voice-design".into(),
        media: Media::Speech,
        selection: draft.selection.clone(),
        prompt: draft.description.clone(),
        text: Some(draft.text.repeat(3)),
        locale: None,
        settings: Settings::default(),
        character_hash: Some(draft.identity_hash.clone()),
        references: vec![],
        depends_on: vec![],
        output: None,
        turns: vec![],
        audiobook: None,
    };
    let operation = operations::perform(
        &dir.join("design-operation.yaml"),
        Request {
            selection: draft.selection.clone(),
            input,
            attempts: draft.limits.attempts,
            retry_unknown,
        },
        || engine::reserve(root, &draft.id, &draft.limits, Some(&reservation), false),
        || adapter.design(&draft.selection, &draft.description, &draft.text),
    )
    .await?;
    if !draft.candidates.is_empty() {
        for c in &draft.candidates {
            if hash(&fs::read(&c.artifact.path)?) != c.artifact.sha256 {
                return Err(Error::new(
                    "artifact_corrupt",
                    "Voice preview no longer matches its saved hash",
                    3,
                ));
            }
        }
        let budget: Budget = yaml(&dir.join("budget.yaml"))?;
        if budget.audio_seconds > draft.limits.audio_seconds {
            return Err(Error::limit(
                "Saved voice previews exceeded the duration budget; inspect the candidates before further operations",
            ));
        }
        return Ok(draft);
    }
    let previews: Vec<Preview> = serde_json::from_value(
        operation
            .result
            .ok_or_else(|| Error::input("Voice design operation has no previews"))?,
    )?;
    if previews.is_empty() || previews.len() > 3 {
        return Err(Error::input(
            "Voice design must return one to three previews",
        ));
    }
    let mut candidates = vec![];
    let mut ids = std::collections::BTreeSet::new();
    let mut duration = 0.0;
    for (index, p) in previews.into_iter().enumerate() {
        if p.id.is_empty() || !ids.insert(p.id.clone()) {
            return Err(Error::input(
                "Voice candidate IDs must be nonempty and distinct",
            ));
        }
        let bytes = STANDARD
            .decode(&p.audio.base64)
            .map_err(|_| Error::input("Invalid preview audio encoding"))?;
        if hash(&bytes) != p.audio.sha256 {
            return Err(Error::input(
                "Voice preview bytes do not match their snapshot hash",
            ));
        }
        let format = match p.audio.mime.as_str() {
            "audio/mpeg" | "audio/mp3" => "mp3",
            "audio/wav" | "audio/x-wav" => "wav",
            _ => return Err(Error::input("Unsupported voice preview audio format")),
        };
        let path = dir
            .join("outputs")
            .join(format!("voice-candidate-{}.{}", index + 1, format));
        let artifact = engine::artifact(&path, "voice-design", &bytes, format, None)?;
        duration += artifact.duration_seconds.unwrap_or(0.0);
        atomic(&path, &bytes, true)?;
        candidates.push(VoiceCandidate {
            id: p.id,
            artifact,
            language: p.language,
        });
    }
    draft.candidates = candidates;
    save(&manifest, &draft)?;
    // Record measured preview duration; never discard completed candidates on overrun.
    let budget_guard = lock(&dir.join("budget.lock"))?;
    budget_guard.lock_exclusive()?;
    let mut budget: Budget = yaml(&dir.join("budget.yaml"))?;
    budget.audio_seconds = budget.audio_seconds.max(duration.ceil() as u32);
    save(&dir.join("budget.yaml"), &budget)?;
    if budget.audio_seconds > draft.limits.audio_seconds {
        return Err(Error::limit(
            "Voice preview duration exceeded the budget; candidates saved and further requests blocked",
        ));
    }
    Ok(draft)
}
#[derive(Default)]
pub struct CreateOptions {
    pub bind: bool,
    pub replace_binding: bool,
    pub retry_unknown: bool,
    pub dry_run: bool,
    pub refresh_verification: bool,
}
pub async fn create(
    root: &Path,
    character: &Path,
    draft: &VoiceDesign,
    candidate: &str,
    name: &str,
    options: CreateOptions,
    adapter: &dyn Adapter,
) -> Result<serde_json::Value> {
    version(draft.schema_version)?;
    validation::limits(&draft.limits)?;
    validate_design(&draft.selection, &draft.description, &draft.text)?;
    if name.trim().is_empty() || name.chars().count() > 100 {
        return Err(Error::input(
            "Provider voice name requires 1–100 characters",
        ));
    }
    let guard = lock(&character.with_extension("lock"))?;
    guard.lock_exclusive()?;
    let (mut c, _, identity_hash) = identity(character)?;
    if c.id != draft.character_id
        || identity_hash != draft.identity_hash
        || c.voice.as_ref().unwrap().selection != draft.speech_selection
    {
        return Err(Error::input(
            "Character identity/profile changed after design; create new previews before binding",
        ));
    }
    let candidate = draft
        .candidates
        .iter()
        .find(|p| p.id == candidate)
        .ok_or_else(|| Error::input("Select a candidate ID from the saved design run"))?;
    let bytes = fs::read(&candidate.artifact.path)?;
    if hash(&bytes) != candidate.artifact.sha256 {
        return Err(Error::new(
            "artifact_corrupt",
            "Voice preview no longer matches its saved hash",
            3,
        ));
    }
    let dir = engine::run_path(root, &draft.id)?;
    let operation_path = dir.join(format!("create-{}.yaml", hash(candidate.id.as_bytes())));
    let saved = if operation_path.exists() {
        Some(yaml::<Operation>(&operation_path)?)
    } else {
        None
    };
    let prior_created = saved
        .as_ref()
        .and_then(|s| s.result.as_ref())
        .and_then(|r| r["voice_id"].as_str());
    let current = c.voice.as_ref().unwrap().binding.as_deref();
    if options.bind && current.is_some() && current != prior_created && !options.replace_binding {
        return Err(Error::input(
            "A voice is already bound; --replace-binding must be explicit before creating a replacement",
        ));
    }
    if options.dry_run {
        return Ok(
            json!({"dry_run":true,"provider":"elevenlabs","candidate":candidate.id,"name":name,"bind":options.bind,"provider_voice_created":false}),
        );
    }
    let input = json!({"kind":"voice_create","candidate":candidate.id,"name":name,"description":draft.description,"identity_hash":draft.identity_hash});
    let reservation = Effective {
        assembly: None,
        id: "voice-create".into(),
        media: Media::Text,
        selection: draft.selection.clone(),
        prompt: String::new(),
        text: None,
        locale: None,
        settings: Settings::default(),
        character_hash: None,
        references: vec![],
        depends_on: vec![],
        output: None,
        turns: vec![],
        audiobook: None,
    };
    let mut operation = operations::perform(
        &operation_path,
        Request {
            selection: draft.selection.clone(),
            input,
            attempts: draft.limits.attempts,
            retry_unknown: options.retry_unknown,
        },
        || engine::reserve(root, &draft.id, &draft.limits, Some(&reservation), false),
        || adapter.create(name, &draft.description, &candidate.id),
    )
    .await?;
    let mut result = operation
        .result
        .clone()
        .ok_or_else(|| Error::input("Create operation has no voice binding"))?;
    let voice_id = result["voice_id"]
        .as_str()
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| Error::input("Provider returned no voice binding"))?
        .to_owned();
    if options.refresh_verification {
        let _slots = engine::slots(Provider::Elevenlabs).await?;
        let verified = adapter.verify(&voice_id).await?;
        if verified.value["voice_id"] != json!(voice_id) {
            return Err(Error::input(
                "Voice verification returned a different provider binding",
            ));
        }
        result["verification"] = json!({"status":verified.value,"request_id":verified.request_id,"usage":verified.usage});
        operation.result = Some(result.clone());
        save(&operation_path, &operation)?;
    }
    let verification = result.pointer("/verification/status").unwrap_or(&result);
    if options.bind
        && verification["requires_verification"] != json!(false)
        && verification["is_verified"] != json!(true)
    {
        return Ok(
            json!({"provider_voice_created":true,"voice_id":voice_id,"bound":false,"verification_required":verification["requires_verification"],"reason":"Provider verification status must be confirmed before binding; use --refresh-verification after completing provider requirements"}),
        );
    }
    if options.bind {
        c.voice.as_mut().unwrap().binding = Some(voice_id.clone());
        save(character, &c)?;
    }
    // Selection is explicit; the preview stays a candidate until separate persona accept.
    Ok(
        json!({"provider_voice_created":true,"voice_id":voice_id,"bound":options.bind,"character":absolute(character)?,"preview_accepted":false}),
    )
}
pub fn status(root: &Path, id: &str) -> Result<serde_json::Value> {
    let dir = engine::run_path(root, id)?;
    let design: VoiceDesign = yaml(&dir.join("voice-design.yaml"))?;
    version(design.schema_version)?;
    let mut operations = vec![];
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name == "design-operation.yaml"
            || (name.starts_with("create-") && name.ends_with(".yaml"))
        {
            operations.push(crate::operations::status(&path)?);
        }
    }
    Ok(json!({"design":design,"operations":operations}))
}

pub fn validate_clone(manifest: &Path) -> Result<serde_json::Value> {
    let request: VoiceClone = yaml(manifest)?;
    version(request.schema_version)?;
    crate::storage::id(&request.id)?;
    if request.name.trim().is_empty()
        || request.name.chars().count() > 100
        || request.recordings.is_empty()
        || request.recordings.len() > 5
    {
        return Err(Error::input(
            "Voice cloning needs a name and one to five explicitly selected recordings",
        ));
    }
    let base = manifest.parent().unwrap_or(Path::new("."));
    let (character, _, identity_hash) = identity(&relative(base, &request.character)?)?;
    let consent_hash =
        validation::consent(base, &request.consent, ConsentPurpose::SavedVoiceClone)?;
    let mut size = 0;
    for path in &request.recordings {
        let r = validation::reference(base, path)?;
        if !r.mime.starts_with("audio/") {
            return Err(Error::input("Cloning recordings must be audio files"));
        }
        size += r.base64.len();
    }
    if size > 48 * 1024 * 1024 {
        return Err(Error::input(
            "Cloning recordings exceed the 48 MiB encoded budget",
        ));
    }
    Ok(
        json!({"request":request,"character_id":character.id,"identity_hash":identity_hash,"consent_hash":consent_hash}),
    )
}
pub async fn clone_voice(
    root: &Path,
    manifest: &Path,
    limits: Limits,
    dry_run: bool,
    retry_unknown: bool,
    refresh_verification: bool,
    adapter: &dyn Adapter,
) -> Result<serde_json::Value> {
    validate_clone(manifest)?;
    let request: VoiceClone = yaml(manifest)?;
    version(request.schema_version)?;
    crate::storage::id(&request.id)?;
    validation::limits(&limits)?;
    if request.name.trim().is_empty()
        || request.name.chars().count() > 100
        || request.recordings.is_empty()
        || request.recordings.len() > 5
    {
        return Err(Error::input(
            "Voice cloning needs a name and one to five explicitly selected recordings",
        ));
    }
    let base = manifest.parent().unwrap_or(Path::new("."));
    let character = relative(base, &request.character)?;
    let guard = lock(&character.with_extension("lock"))?;
    guard.lock_exclusive()?;
    let (mut c, description, identity_hash) = identity(&character)?;
    let consent_hash =
        validation::consent(base, &request.consent, ConsentPurpose::SavedVoiceClone)?;
    let recordings = request
        .recordings
        .iter()
        .map(|path| {
            let mut r = validation::reference(base, path)?;
            if !r.mime.starts_with("audio/") {
                return Err(Error::input("Cloning recordings must be audio files"));
            }
            r.role = ReferenceRole::Identity;
            r.consent_hash = Some(consent_hash.clone());
            Ok(r)
        })
        .collect::<Result<Vec<_>>>()?;
    if recordings.iter().map(|r| r.base64.len()).sum::<usize>() > 48 * 1024 * 1024 {
        return Err(Error::input(
            "Cloning recordings exceed the 48 MiB encoded budget",
        ));
    }
    let dir = engine::run_path(root, &request.id)?;
    let path = dir.join("clone-operation.yaml");
    let prior = if path.exists() {
        Some(yaml::<Operation>(&path)?)
    } else {
        None
    };
    let prior_id = prior
        .as_ref()
        .and_then(|o| o.result.as_ref())
        .and_then(|r| r["voice_id"].as_str());
    if request.bind
        && c.voice
            .as_ref()
            .unwrap()
            .binding
            .as_deref()
            .is_some_and(|id| Some(id) != prior_id)
        && !request.replace_binding
    {
        return Err(Error::input(
            "An existing voice binding requires explicit replace_binding before cloning a replacement",
        ));
    }
    let selection = c.voice.as_ref().unwrap().selection.clone();
    let input = json!({"kind":"voice_clone","character_id":c.id,"identity_hash":identity_hash,"name":request.name,"description":description,"recordings":recordings,"consent_hash":consent_hash});
    if dry_run {
        return Ok(
            json!({"dry_run":true,"run_id":request.id,"input":input,"bind":request.bind,"provider_voice_created":false}),
        );
    }
    let reservation = Effective {
        assembly: None,
        id: "voice-clone".into(),
        media: Media::Text,
        selection: selection.clone(),
        prompt: description.clone(),
        text: None,
        locale: None,
        settings: Settings::default(),
        character_hash: Some(identity_hash),
        references: vec![],
        depends_on: vec![],
        output: None,
        turns: vec![],
        audiobook: None,
    };
    let mut operation = operations::perform(
        &path,
        Request {
            selection,
            input,
            attempts: limits.attempts,
            retry_unknown,
        },
        || engine::reserve(root, &request.id, &limits, Some(&reservation), false),
        || adapter.clone_voice(&request.name, &description, &recordings),
    )
    .await?;
    let mut result = operation
        .result
        .clone()
        .ok_or_else(|| Error::input("Cloning operation returned no voice"))?;
    let voice=result["voice_id"].as_str().filter(|id|!id.trim().is_empty()).ok_or_else(||Error::unknown("Provider may have cloned the voice without a returned binding; reconcile before retry"))?.to_owned();
    if refresh_verification {
        let _slots = engine::slots(Provider::Elevenlabs).await?;
        let verified = adapter.verify(&voice).await?;
        if verified.value["voice_id"] != json!(voice) {
            return Err(Error::input(
                "Verification returned a different cloned binding",
            ));
        }
        result["verification"] = json!({"status":verified.value,"request_id":verified.request_id});
        operation.result = Some(result.clone());
        save(&path, &operation)?;
    }
    let verification = result.pointer("/verification/status").unwrap_or(&result);
    let ready = verification["requires_verification"] == json!(false)
        || verification["is_verified"] == json!(true);
    if request.bind && ready {
        c.voice.as_mut().unwrap().binding = Some(voice.clone());
        save(&character, &c)?;
    }
    Ok(
        json!({"run_id":request.id,"voice_id":voice,"provider_voice_created":true,"bound":request.bind&&ready,"verification_required":!ready,"source_recordings_accepted":false,"operation":absolute(&path)?}),
    )
}
pub fn clone_status(root: &Path, id: &str) -> Result<Operation> {
    operations::status(&engine::run_path(root, id)?.join("clone-operation.yaml"))
}
