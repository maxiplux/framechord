pub mod health;
use crate::{
    contracts::*,
    credentials,
    error::{Error, Result},
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

#[derive(Clone, serde::Serialize)]
pub struct Capability {
    pub provider: Provider,
    pub model: &'static str,
    pub media: Media,
    pub image_references: bool,
    pub voice_binding: bool,
    pub format: &'static str,
    pub audio_references: bool,
}
pub fn speech_limit(selection: &Selection) -> Option<usize> {
    match (selection.provider, selection.model.as_str()) {
        (Provider::Openai, _) => Some(4096),
        (Provider::Elevenlabs, "eleven_flash_v2_5") => Some(40000),
        (Provider::Elevenlabs, "eleven_multilingual_v2") => Some(10000),
        (Provider::Elevenlabs, "eleven_v3") => Some(5000),
        _ => None,
    }
}
pub fn fidelity_values(model: &str) -> &'static [&'static str] {
    match model {
        "gpt-image-1" | "gpt-image-1.5" => &["high", "low"],
        "gpt-image-1-mini" => &["low"],
        _ => &[],
    }
}
pub fn capabilities() -> Vec<Value> {
    registry()
        .into_iter()
        .map(|cap| {
            let mut value = serde_json::to_value(&cap).expect("Capability is serializable");
            let mut settings = vec!["format"];
            match cap.media {
                Media::Image if cap.provider == Provider::Openai => {
                    settings.extend(["size", "image_quality"]);
                    if !fidelity_values(cap.model).is_empty() {
                        settings.push("input_fidelity");
                    }
                }
                Media::Speech => settings.push("voice"),
                Media::Music => settings.extend(["duration_seconds", "instrumental"]),
                Media::Sfx => settings.push("duration_seconds"),
                _ => {}
            }
            value["supported_settings"] = json!(settings);
            value["input_fidelity_values"] = json!(if cap.provider == Provider::Openai
                && cap.media == Media::Image
            {
                fidelity_values(cap.model)
            } else {
                &[]
            });
            value["identity_references"] =
                json!(cap.image_references && cap.model != "gpt-image-1-mini");
            value["speech_characters_per_request"] = json!(if cap.media == Media::Speech {
                speech_limit(&Selection {
                    provider: cap.provider,
                    model: cap.model.into(),
                })
            } else {
                None
            });
            value["result_retrieval"] = json!(cap.provider == Provider::Google);
            value["saved_voice_cloning"] =
                json!(cap.media == Media::Speech && cap.provider == Provider::Elevenlabs);
            value["delivery_direction"] = json!(
                cap.media != Media::Speech
                    || matches!(cap.provider, Provider::Google)
                    || cap.provider == Provider::Openai
                        && !matches!(cap.model, "tts-1" | "tts-1-hd")
            );
            value
        })
        .collect()
}
pub fn registry() -> Vec<Capability> {
    let mut result = vec![];
    for m in [
        "gpt-image-1",
        "gpt-image-1.5",
        "gpt-image-2",
        "gpt-image-1-mini",
        "gpt-image-2.5-flare",
        "gpt-image-2.5-flare-2026-09-08",
        "gpt-image-2.5-sunburst",
        "gpt-image-2.5-sunburst-2026-09-08",
    ] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Openai,
            model: m,
            media: Media::Image,
            image_references: true,
            voice_binding: false,
            format: "png",
        });
    }
    for m in [
        "gpt-4o-mini-tts",
        "gpt-4o-mini-tts-2025-12-15",
        "tts-1",
        "tts-1-hd",
    ] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Openai,
            model: m,
            media: Media::Speech,
            image_references: false,
            voice_binding: true,
            format: "wav",
        });
    }
    for m in [
        "gemini-nano-banana-2.1",
        "gemini-3.1-flash-image",
        "gemini-3.1-flash-lite-image",
        "gemini-3-pro-image",
        "gemini-2.5-flash-image",
    ] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Google,
            model: m,
            media: Media::Image,
            image_references: true,
            voice_binding: false,
            format: "png",
        });
    }
    for model in ["gemini-3.8-flash-tts", "gemini-3.8-flash-lite-tts"] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Google,
            model,
            media: Media::Speech,
            image_references: false,
            voice_binding: true,
            format: "wav",
        });
    }
    for model in ["lyria-3.5", "lyria-3-clip-preview"] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Google,
            model,
            media: Media::Music,
            image_references: false,
            voice_binding: false,
            format: "mp3",
        });
    }
    for m in ["s1", "s2-pro", "s2.1-pro", "s2.1-pro-free", "drama-3"] {
        result.push(Capability {
            audio_references: true,
            provider: Provider::Fish,
            model: m,
            media: Media::Speech,
            image_references: false,
            voice_binding: true,
            format: "wav",
        });
    }
    for m in [
        "eleven_multilingual_v2",
        "eleven_flash_v2_5",
        "eleven_turbo_v2_5",
        "eleven_v3",
    ] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Elevenlabs,
            model: m,
            media: Media::Speech,
            image_references: false,
            voice_binding: true,
            format: "mp3",
        });
    }
    for m in ["music_v1", "music_v2", "music_v2_5"] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Elevenlabs,
            model: m,
            media: Media::Music,
            image_references: false,
            voice_binding: false,
            format: "mp3",
        });
    }
    result.push(Capability {
        audio_references: false,
        provider: Provider::Elevenlabs,
        model: "eleven_text_to_sound_v2",
        media: Media::Sfx,
        image_references: false,
        voice_binding: false,
        format: "mp3",
    });
    for model in [
        "openai/gpt-image-1",
        "openai/gpt-image-2.5-flare",
        "openai/gpt-image-2.5-sunburst",
        "google/gemini-nano-banana-2.1",
    ] {
        result.push(Capability {
            audio_references: false,
            provider: Provider::Openrouter,
            model,
            media: Media::Image,
            image_references: false,
            voice_binding: false,
            format: "png",
        });
    }
    result
}
pub fn valid_image_size(model: &str, size: &str) -> bool {
    if size == "auto" {
        return true;
    }
    if !model.starts_with("gpt-image-2.5-") {
        return ["1024x1024", "1024x1536", "1536x1024"].contains(&size);
    }
    let Some((w, h)) = size.split_once('x') else {
        return false;
    };
    let (Ok(w), Ok(h)) = (w.parse::<u64>(), h.parse::<u64>()) else {
        return false;
    };
    w > 0
        && h > 0
        && w <= 3840
        && h <= 3840
        && w % 16 == 0
        && h % 16 == 0
        && w.max(h) <= w.min(h) * 3
        && (655360..=8294400).contains(&(w * h))
}
pub fn validate(e: &Effective) -> Result<()> {
    if matches!(e.media, Media::Podcast | Media::Audiobook)
        || (e.media == Media::Speech && !e.turns.is_empty())
    {
        for t in &e.turns {
            validate(t)?;
        }
        return Ok(());
    }
    let media = if e.media == Media::Infographic {
        Media::Image
    } else {
        e.media
    };
    let cap = registry().into_iter().find(|c| {
        c.provider == e.selection.provider && c.model == e.selection.model && c.media == media
    });
    if media == Media::Text
        && matches!(
            e.selection.provider,
            Provider::Openai | Provider::Google | Provider::Openrouter
        )
    {
        if !e.references.is_empty() {
            return Err(Error::input(
                "Text planning does not accept media references; use explicit review for quality",
            ));
        }
        if e.settings != Settings::default() {
            return Err(Error::input(
                "Text planning does not accept media generation settings",
            ));
        }
        return Ok(());
    }
    let c=cap.ok_or_else(||Error::new("unsupported_capability","Provider/model/media combination is not in the implemented capability registry; inspect framechord models",2))?;
    if media == Media::Speech {
        if e.references
            .iter()
            .any(|r| !r.mime.starts_with("audio/") || !matches!(r.role, ReferenceRole::Identity))
        {
            return Err(Error::input(
                "Speech references must be accepted identity recordings or voice_samples",
            ));
        }
        if e.settings.voice.is_none()
            && (e.selection.provider != Provider::Fish
                || e.references.is_empty()
                || e.references.iter().any(|r| {
                    r.transcript.as_ref().is_none_or(|t| t.trim().is_empty())
                        || r.consent_hash.is_none()
                }))
        {
            return Err(Error::input(
                "Speech requires a saved voice binding or authorized Fish audio samples with exact transcripts",
            ));
        }
        if e.references.len() > 8
            || e.references.iter().map(|r| r.base64.len()).sum::<usize>() > 64 * 1024 * 1024
        {
            return Err(Error::input(
                "At most eight voice references and 64 MiB of encoded audio are supported",
            ));
        }
    } else if !e.references.is_empty()
        && (!c.image_references || e.references.iter().any(|r| !r.mime.starts_with("image/")))
    {
        return Err(Error::new(
            "unsupported_references",
            "This media/provider combination does not support these references; use supported image references or authorized Fish speech samples",
            2,
        ));
    }
    if media == Media::Image {
        for reference in &e.references {
            crate::provenance::check(reference)?;
            if matches!(reference.role, ReferenceRole::Identity)
                && reference
                    .origin
                    .as_ref()
                    .is_some_and(|origin| origin.selection != e.selection)
            {
                return Err(Error::input(
                    "Identity reference model differs from the original asset's recorded provider/model",
                ));
            }
        }
    }
    if let Some(fidelity) = e.settings.input_fidelity.as_deref()
        && (media != Media::Image
            || e.selection.provider != Provider::Openai
            || e.references.is_empty()
            || !fidelity_values(&e.selection.model).contains(&fidelity))
    {
        return Err(Error::input(
            "input_fidelity requires image references and a supported OpenAI image-edit model; gpt-image-2 requires this parameter to be omitted",
        ));
    }
    if matches!(media, Media::Image)
        && e.references
            .iter()
            .any(|r| matches!(r.role, ReferenceRole::Identity))
        && e.selection.provider == Provider::Openai
    {
        if e.selection.model == "gpt-image-1-mini" {
            return Err(Error::input(
                "This model only supports low-fidelity editing; identity-preserving requests require an explicitly selected capable model",
            ));
        }
        if matches!(e.selection.model.as_str(), "gpt-image-1" | "gpt-image-1.5")
            && e.settings.input_fidelity.as_deref() != Some("high")
        {
            return Err(Error::input(
                "Identity references require input_fidelity high; resolve the authored job before submission",
            ));
        }
    }
    if e.settings.seed.is_some() {
        return Err(Error::input(
            "Seed is not implemented for this adapter; omit it",
        ));
    }
    if let Some(quality) = &e.settings.image_quality
        && (!(media == Media::Image && e.selection.provider == Provider::Openai)
            || !(if e.selection.model.starts_with("gpt-image-2.5-") {
                &["low", "medium", "high", "xhigh", "max", "auto"][..]
            } else {
                &["low", "medium", "high", "auto"][..]
            })
            .contains(&quality.as_str()))
    {
        return Err(Error::input(
            "Unsupported image quality for this OpenAI image model; inspect framechord models",
        ));
    }
    if e.settings.instrumental.is_some() && media != Media::Music {
        return Err(Error::input("instrumental applies only to music"));
    }
    if e.settings.size.is_some()
        && !(media == Media::Image && e.selection.provider == Provider::Openai)
    {
        return Err(Error::input(
            "Explicit pixel size currently supported only for OpenAI images; express composition in the brief for other adapters",
        ));
    }
    if let Some(size) = &e.settings.size
        && !valid_image_size(&e.selection.model, size)
    {
        return Err(Error::input(
            "Unsupported image size: GPT Image 2.5 accepts auto or WxH with both sides divisible by 16 and at most 3840, aspect ratio at most 3:1, and 655360–8294400 pixels (for example 2048x2048); other supported image models accept 1024x1024, 1024x1536, 1536x1024, or auto",
        ));
    }
    let final_format = if media == Media::Speech {
        "wav"
    } else {
        c.format
    };
    if e.settings
        .format
        .as_ref()
        .is_some_and(|f| f != final_format)
    {
        return Err(Error::input(format!(
            "This workflow publishes {final_format}; requested format is unsupported"
        )));
    }
    if media == Media::Speech {
        if e.locale.is_some()
            && (matches!(e.selection.provider, Provider::Fish | Provider::Elevenlabs)
                || matches!(e.selection.model.as_str(), "tts-1" | "tts-1-hd"))
        {
            return Err(Error::input(
                "This adapter does not support explicit locale direction; use a compatible instruction-capable speech model, or omit locale and use the saved voice's language with exact source text",
            ));
        }
        if matches!(e.selection.provider, Provider::Fish | Provider::Elevenlabs)
            && !e.prompt.trim().is_empty()
        {
            return Err(Error::input(
                "This saved-voice speech adapter does not accept job delivery instructions yet; omit the brief or choose an adapter with an instruction channel",
            ));
        }
        if e.settings.voice.as_ref().is_some_and(|v| {
            v.is_empty()
                || v.len() > 8192
                || !v
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        }) {
            return Err(Error::input(
                "Voice binding must be a provider ID or built-in name, without URL/path characters",
            ));
        }
        let count = e.text.as_deref().unwrap_or("").chars().count();
        if let Some(limit) = speech_limit(&e.selection)
            && count > limit
        {
            return Err(Error::input(format!(
                "Speech model supports at most {limit} characters per request; resolve the authored job to split exact text before submission"
            )));
        }
        if e.settings.duration_seconds.is_some() {
            return Err(Error::input(
                "Speech duration is estimated from exact text; duration_seconds is for music and effects",
            ));
        }
        if e.settings.size.is_some() {
            return Err(Error::input("Speech does not support image size"));
        }
        if e.selection.provider == Provider::Openai
            && matches!(e.selection.model.as_str(), "tts-1" | "tts-1-hd")
            && !e.prompt.trim().is_empty()
        {
            return Err(Error::input(
                "tts-1 models do not accept voice direction; choose a gpt-4o-mini-tts model",
            ));
        }
    } else if e.settings.voice.is_some() {
        return Err(Error::input("voice applies only to speech"));
    }
    if matches!(media, Media::Music | Media::Sfx) {
        let d = e.settings.duration_seconds.ok_or_else(|| {
            Error::input(
                "Music and effects require explicit duration_seconds for bounded generation",
            )
        })?;
        let max = if media == Media::Sfx { 30 } else { 600 };
        if e.selection.provider == Provider::Google {
            if e.selection.model == "lyria-3-clip-preview" && d != 30 {
                return Err(Error::input(
                    "Lyria Clip always generates 30 seconds; set duration_seconds to 30",
                ));
            }
            if e.selection.model == "lyria-3.5" && d < 60 {
                return Err(Error::input(
                    "Lyria full songs cannot be requested as short clips; choose Lyria Clip at 30 seconds or explicitly reserve at least 60 seconds",
                ));
            }
        }
        if d == 0 || d > max || media == Media::Music && d < 3 {
            return Err(Error::input(format!(
                "Duration must be {}–{max} seconds",
                if media == Media::Music { 3 } else { 1 }
            )));
        }
        if e.selection.provider == Provider::Elevenlabs && e.prompt.chars().count() > 4100 {
            return Err(Error::input("ElevenLabs prompt exceeds 4100 characters"));
        }
    } else if media == Media::Image && e.settings.duration_seconds.is_some() {
        return Err(Error::input("Images have no duration"));
    }
    Ok(())
}

pub struct Generated {
    pub bytes: Vec<u8>,
    pub format: String,
    pub request_id: Option<String>,
    pub usage: Option<Value>,
}
#[async_trait]
pub trait Adapter: Send + Sync {
    async fn generate(&self, input: &Effective) -> Result<Generated>;
    async fn reconcile(&self, _input: &Effective, _id: &str) -> Result<Reconciled> {
        Err(Error::input(
            "This provider does not expose retrieval for this request; reconcile in its console before explicit retry",
        ))
    }
}
pub enum Reconciled {
    Complete(Generated),
    Pending,
    Failed,
}
pub struct HttpAdapter {
    root: PathBuf,
    client: reqwest::Client,
    transport: Arc<dyn Transport>,
}
pub struct WireResponse {
    pub status: u16,
    pub headers: reqwest::header::HeaderMap,
    pub bytes: Vec<u8>,
}
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, request: reqwest::Request) -> Result<WireResponse>;
}
struct HttpsTransport {
    client: reqwest::Client,
}
#[async_trait]
impl Transport for HttpsTransport {
    async fn send(&self, request: reqwest::Request) -> Result<WireResponse> {
        let mut response=self.client.execute(request).await.map_err(|_|Error::unknown("Submission interrupted. The provider may have accepted it; inspect run state before explicit retry"))?;
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let mut bytes = vec![];
        if response.status().is_success() {
            while let Some(chunk) = response.chunk().await.map_err(|_| {
                let mut e = Error::unknown("Provider response was interrupted after submission");
                e.request_id = remote_id(&headers);
                e
            })? {
                if bytes.len() + chunk.len() > 128 * 1024 * 1024 {
                    let mut e =
                        Error::unknown("Provider response exceeded 128 MiB after submission");
                    e.request_id = remote_id(&headers);
                    return Err(e);
                }
                bytes.extend_from_slice(&chunk);
            }
        }
        Ok(WireResponse {
            status,
            headers,
            bytes,
        })
    }
}
fn remote_id(headers: &reqwest::header::HeaderMap) -> Option<String> {
    ["x-request-id", "request-id", "song-id"]
        .iter()
        .find_map(|h| headers.get(*h)?.to_str().ok().map(str::to_owned))
}
#[derive(Debug, Clone)]
pub struct CatalogOptions {
    pub public: bool,
    pub page_size: usize,
    pub max_pages: usize,
    pub page_token: Option<String>,
    pub page_number: usize,
    pub search: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
}
impl Default for CatalogOptions {
    fn default() -> Self {
        Self {
            public: false,
            page_size: 20,
            max_pages: 1,
            page_token: None,
            page_number: 1,
            search: None,
            language: None,
            category: None,
        }
    }
}
impl HttpAdapter {
    pub async fn review_audio(
        &self,
        selection: &Selection,
        prompt: &str,
        bytes: &[u8],
        format: &str,
        artifact: &str,
        references: &[Reference],
    ) -> Result<crate::operations::Output> {
        crate::catalog::require_current(selection)?;
        let body = audio_review_body(selection, prompt, bytes, format, references)?;
        let key = credentials::resolve(Provider::Google, &self.root)?;
        let (response, header_id) = self
            .json(
                self.client
                    .post("https://generativelanguage.googleapis.com/v1beta/interactions")
                    .header("x-goog-api-key", key)
                    .json(&body),
            )
            .await?;
        audio_assessment(selection, artifact, &response, header_id)
    }
    pub async fn retrieve_audio_review(
        &self,
        selection: &Selection,
        artifact: &str,
        id: &str,
    ) -> Result<Option<crate::operations::Output>> {
        if selection.provider != Provider::Google {
            return Err(Error::input(
                "Audio review retrieval is supported only for Google interactions",
            ));
        }
        let (response, header) = self.google_interaction(&selection.model, id).await?;
        match response["status"].as_str() {
            Some("in_progress" | "requires_action") => Ok(None),
            Some("failed" | "cancelled") => Err(Error::new(
                "provider_failed",
                "Provider confirmed the review interaction failed or was cancelled",
                3,
            )),
            _ => Ok(Some(audio_assessment(
                selection, artifact, &response, header,
            )?)),
        }
    }
    async fn google_interaction(&self, model: &str, id: &str) -> Result<(Value, Option<String>)> {
        if id.is_empty()
            || id.len() > 512
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(Error::input(
                "Saved request ID is not a retrievable Google interaction ID",
            ));
        }
        let key = credentials::resolve(Provider::Google, &self.root)?;
        let result = self
            .json(
                self.client
                    .get(format!(
                        "https://generativelanguage.googleapis.com/v1beta/interactions/{id}"
                    ))
                    .header("x-goog-api-key", key),
            )
            .await?;
        if result.0["model"].as_str().is_some_and(|m| m != model)
            || result.0["id"].as_str().is_some_and(|r| r != id)
        {
            return Err(Error::input(
                "Provider retrieval returned a different interaction or model",
            ));
        }
        Ok(result)
    }
    pub fn new(root: &Path) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| Error::new("http_init", "Cannot initialize HTTPS client", 3))?;
        Ok(Self {
            root: root.into(),
            transport: Arc::new(HttpsTransport {
                client: client.clone(),
            }),
            client,
        })
    }
    pub fn with_transport(root: &Path, transport: Arc<dyn Transport>) -> Result<Self> {
        let mut adapter = Self::new(root)?;
        adapter.transport = transport;
        Ok(adapter)
    }
    pub async fn voices(&self, provider: Provider, public: bool) -> Result<Value> {
        self.voice_catalog(
            provider,
            &CatalogOptions {
                public,
                ..Default::default()
            },
        )
        .await
    }
    pub async fn voice_catalog(
        &self,
        provider: Provider,
        options: &CatalogOptions,
    ) -> Result<Value> {
        if options.page_size == 0
            || options.page_size > 100
            || options.max_pages == 0
            || options.max_pages > 20
            || options.page_number == 0
        {
            return Err(Error::input(
                "Catalog limits need page-size 1–100, max-pages 1–20, and a positive page-number",
            ));
        }
        if options.search.as_ref().is_some_and(|s| s.len() > 2048)
            || options.page_token.as_ref().is_some_and(|s| s.len() > 8192)
        {
            return Err(Error::input(
                "Catalog search or continuation token exceeds its size limit",
            ));
        }
        if options.public && provider != Provider::Fish {
            return Err(Error::input(
                "Public catalog selection is only implemented for Fish Audio",
            ));
        }
        if provider == Provider::Fish
            && (options.page_token.is_some() || options.category.is_some())
        {
            return Err(Error::input(
                "Fish uses --page-number and has no category filter; use --search/--language",
            ));
        }
        if provider != Provider::Fish && options.page_number != 1 {
            return Err(Error::input(
                "Google and ElevenLabs use --page-token, not --page-number",
            ));
        }
        if let Some(category) = &options.category {
            let allowed: &[&str] = match provider {
                Provider::Google => &["prebuilt", "replicated", "prompted"],
                Provider::Elevenlabs => &["premade", "cloned", "generated", "professional"],
                _ => &[],
            };
            if !allowed.contains(&category.as_str()) {
                return Err(Error::input("Unsupported voice category for this provider"));
            }
        }
        let url = match provider {
            Provider::Fish => "https://api.fish.audio/model",
            Provider::Google => "https://generativelanguage.googleapis.com/v1beta/voices",
            Provider::Elevenlabs => "https://api.elevenlabs.io/v2/voices",
            _ => {
                return Err(Error::input(
                    "Account voice discovery is implemented for Google, Fish, and ElevenLabs",
                ));
            }
        };
        let key = credentials::resolve(provider, &self.root)?;
        let mut token = options.page_token.clone();
        let mut page_number = options.page_number;
        let mut voices = vec![];
        let mut seen = std::collections::BTreeSet::new();
        let mut pages = 0;
        let mut more = false;
        let mut request_ids = vec![];
        for _ in 0..options.max_pages {
            let mut query = vec![("page_size", options.page_size.to_string())];
            if provider == Provider::Fish {
                if page_number != 1 {
                    query.push(("page_number", page_number.to_string()));
                }
                query.push(("self", (!options.public).to_string()));
                if options.public {
                    query.push(("licensed", "true".into()));
                }
            } else if let Some(t) = &token {
                query.push((
                    if provider == Provider::Elevenlabs {
                        "next_page_token"
                    } else {
                        "page_token"
                    },
                    t.clone(),
                ));
            }
            if let Some(s) = &options.search {
                query.push((
                    if provider == Provider::Fish {
                        "title"
                    } else {
                        "search"
                    },
                    s.clone(),
                ));
            }
            if let Some(s) = &options.language {
                query.push((
                    if provider == Provider::Google {
                        "language_code"
                    } else {
                        "language"
                    },
                    s.clone(),
                ));
            }
            if let Some(s) = &options.category {
                query.push((
                    if provider == Provider::Google {
                        "type"
                    } else {
                        "category"
                    },
                    s.clone(),
                ));
            } else if provider == Provider::Google {
                query.push(("type", "prebuilt".into()));
            }
            let request = self.client.get(url).query(&query);
            let request = match provider {
                Provider::Google => request.header("x-goog-api-key", &key),
                Provider::Elevenlabs => request.header("xi-api-key", &key),
                _ => request.bearer_auth(&key),
            };
            let (body, id) = self.json(request).await?;
            if let Some(id) = id {
                request_ids.push(id);
            }
            pages += 1;
            let items = body
                .get("voices")
                .or_else(|| body.get("items"))
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    Error::new("provider_response", "Provider returned no voice list", 3)
                })?;
            for v in items.iter().take(options.page_size) {
                let Some(id) = v
                    .get("voice_id")
                    .or_else(|| v.get("id"))
                    .or_else(|| v.get("_id"))
                    .and_then(Value::as_str)
                else {
                    continue;
                };
                if !seen.insert(id.to_owned()) {
                    continue;
                }
                let name = v
                    .get("name")
                    .or_else(|| v.get("display_name"))
                    .or_else(|| v.get("title"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                voices.push(json!({"id":id,"name":name,"language":v.get("language_code").or_else(||v.get("languages")),"category":v.get("category").or_else(||v.get("type"))}));
            }
            let next = body
                .get("next_page_token")
                .and_then(Value::as_str)
                .filter(|t| !t.is_empty())
                .map(str::to_owned);
            more = if provider == Provider::Fish {
                body.get("has_more")
                    .and_then(Value::as_bool)
                    .unwrap_or_else(|| {
                        body.get("total")
                            .and_then(Value::as_u64)
                            .is_some_and(|total| total > (page_number * options.page_size) as u64)
                    })
            } else {
                next.is_some()
            };
            if provider != Provider::Fish && more && next == token {
                return Err(Error::new(
                    "provider_response",
                    "Provider repeated the catalog continuation token",
                    3,
                ));
            }
            token = next;
            page_number += 1;
            if !more {
                break;
            }
        }
        Ok(
            json!({"provider":provider,"voices":voices,"request_id":request_ids.last(),"request_ids":request_ids,"pages_fetched":pages,"next_page_token":token,"next_page_number":if provider==Provider::Fish&&more {Some(page_number)}else{None},"has_more":more,"public_catalog":options.public,"bounded":true}),
        )
    }
    pub async fn review_image(
        &self,
        selection: &Selection,
        prompt: &str,
        bytes: &[u8],
        artifact: &str,
        references: &[Reference],
    ) -> Result<crate::operations::Output> {
        if !matches!(selection.provider, Provider::Openai | Provider::Openrouter) {
            return Err(Error::input(
                "Image AI review currently supports OpenAI and OpenRouter multimodal text models",
            ));
        }
        crate::catalog::require_current(selection)?;
        let key = credentials::resolve(selection.provider, &self.root)?;
        let url = if selection.provider == Provider::Openai {
            "https://api.openai.com/v1/chat/completions"
        } else {
            "https://openrouter.ai/api/v1/chat/completions"
        };
        let mut content = vec![
            json!({"type":"text","text":prompt}),
            json!({"type":"image_url","image_url":{"url":format!("data:image/png;base64,{}",STANDARD.encode(bytes))}}),
        ];
        for r in references {
            content.push(json!({"type":"image_url","image_url":{"url":format!("data:{};base64,{}",r.mime,r.base64)}}));
        }
        let body = json!({"model":selection.model,"messages":[{"role":"user","content":content}],"response_format":{"type":"json_object"}});
        let (v, request_id) = self
            .json(self.client.post(url).bearer_auth(key).json(&body))
            .await?;
        let content = v
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                receipt(
                    Error::unknown("Reviewer returned no assessment"),
                    &request_id,
                )
            })?;
        let mut review: Review = serde_json::from_str(content)
            .map_err(|_| receipt(Error::unknown("Reviewer returned invalid assessment JSON; inspect the saved receipt before retry"),&request_id))?;
        if review.schema_version != 1
            || review.artifact != artifact
            || review.quality == Quality::Unchecked
        {
            return Err(receipt(
                Error::unknown(
                    "Reviewer assessment failed contract validation; inspect the saved receipt before retry",
                ),
                &request_id,
            ));
        }
        review.provenance = "ai".into();
        review.reviewer = Some(selection.clone());
        Ok(crate::operations::Output {
            value: serde_json::to_value(review)?,
            request_id,
            usage: usage(&v),
        })
    }
    async fn response(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<(Vec<u8>, Option<String>)> {
        let request = request
            .build()
            .map_err(|_| Error::input("Cannot construct provider request"))?;
        let response = self.transport.send(request).await?;
        let request_id = remote_id(&response.headers);
        let status = reqwest::StatusCode::from_u16(response.status)
            .map_err(|_| Error::unknown("Invalid provider HTTP status"))?;
        if !status.is_success() {
            let mut e = if status.is_server_error() {
                Error::unknown(format!(
                    "Provider returned HTTP {status}; submission outcome cannot be established"
                ))
            } else {
                Error::new(
                    "provider_rejected",
                    format!(
                        "Provider rejected request (HTTP {status}); check model, credentials, account access, and billing in the provider console"
                    ),
                    3,
                )
            };
            e.request_id = request_id;
            // A 429 rejection is safe to retry; other errors are never blindly retried.
            if status.as_u16() == 429 {
                e.retryable = true;
                e.retry_after_seconds = response
                    .headers
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok()?.parse().ok())
                    .or(Some(1));
            }
            return Err(e);
        }
        let bytes = response.bytes;
        if bytes.len() > 128 * 1024 * 1024 {
            return Err(Error::unknown("Provider response exceeded 128 MiB"));
        }
        Ok((bytes.to_vec(), request_id))
    }
    async fn json(&self, request: reqwest::RequestBuilder) -> Result<(Value, Option<String>)> {
        let (bytes, id) = self.response(request).await?;
        let v = serde_json::from_slice(&bytes).map_err(|_| {
            let mut e =
                Error::unknown("Provider returned an invalid response after accepting the request");
            e.request_id = id.clone();
            e
        })?;
        Ok((v, id))
    }
}

fn localized_direction(e: &Effective) -> String {
    let mut prompt = e.prompt.clone();
    if let Some(locale) = &e.locale {
        prompt += &format!(
            "\nLocale direction: {locale}. Preserve exact supplied text, scripts, and lyrics without translation."
        );
    }
    prompt
}
fn receipt(mut error: Error, id: &Option<String>) -> Error {
    error.request_id = id.clone();
    error
}
fn decode(v: &str) -> Result<Vec<u8>> {
    STANDARD
        .decode(v)
        .map_err(|_| Error::unknown("Provider returned invalid base64 media"))
}
fn usage(v: &Value) -> Option<Value> {
    let source = v.get("usage").or_else(|| v.get("usageMetadata"))?;
    let mut out = serde_json::Map::new();
    for k in [
        "input_tokens",
        "output_tokens",
        "total_tokens",
        "prompt_tokens",
        "completion_tokens",
        "cost",
        "promptTokenCount",
        "candidatesTokenCount",
        "totalTokenCount",
    ] {
        if let Some(n) = source.get(k).filter(|n| n.is_number()) {
            out.insert(k.into(), n.clone());
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(Value::Object(out))
    }
}
pub fn audio_review_body(
    selection: &Selection,
    prompt: &str,
    bytes: &[u8],
    format: &str,
    references: &[Reference],
) -> Result<Value> {
    if selection.provider != Provider::Google
        || !["gemini-3.8-flash", "gemini-2.5-flash"].contains(&selection.model.as_str())
    {
        return Err(Error::input(
            "Audio review supports explicitly selected Google gemini-3.8-flash or gemini-2.5-flash",
        ));
    }
    let mime = match format {
        "wav" => "audio/wav",
        "mp3" => "audio/mp3",
        "flac" => "audio/flac",
        _ => return Err(Error::input("Unsupported audio review format")),
    };
    let mut content = vec![
        json!({"type":"text","text":prompt}),
        json!({"type":"audio","data":STANDARD.encode(bytes),"mime_type":mime}),
    ];
    for r in references.iter().filter(|r| r.mime.starts_with("audio/")) {
        content.push(json!({"type":"audio","data":r.base64,"mime_type":r.mime}));
    }
    let body = json!({"model":selection.model,"input":[{"type":"user_input","content":content}]});
    if serde_json::to_vec(&body)?.len() > 20 * 1024 * 1024 {
        return Err(Error::input(
            "Audio review exceeds Google's 20 MiB inline request limit; split the job or review individual clips",
        ));
    }
    Ok(body)
}
fn audio_assessment(
    selection: &Selection,
    artifact: &str,
    response: &Value,
    header: Option<String>,
) -> Result<crate::operations::Output> {
    let id = response["id"].as_str().map(str::to_owned).or(header);
    let result = (|| {
        if response["status"]
            .as_str()
            .is_some_and(|s| s != "completed")
        {
            return Err(Error::unknown(
                "Audio reviewer interaction is pending; retrieve its saved ID before retry",
            ));
        }

        let content = response
            .get("output_text")
            .and_then(Value::as_str)
            .or_else(|| {
                response
                    .get("steps")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter(|s| s["type"] == "model_output")
                    .flat_map(|s| {
                        s.get("content")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                    })
                    .rfind(|b| b["type"] == "text")
                    .and_then(|b| b["text"].as_str())
            })
            .ok_or_else(|| Error::unknown("Audio reviewer returned no assessment"))?;
        let content = content
            .trim()
            .strip_prefix("```json")
            .and_then(|c| c.trim().strip_suffix("```"))
            .unwrap_or(content);
        let mut review: Review = serde_json::from_str(content.trim())
            .map_err(|_| Error::unknown("Audio reviewer returned invalid assessment JSON; retrieve its saved ID before retry"))?;
        if review.schema_version != 1
            || review.artifact != artifact
            || review.quality == Quality::Unchecked
        {
            return Err(Error::unknown(
                "Audio assessment failed contract validation; retrieve its saved ID before retry",
            ));
        }
        review.provenance = "ai".into();
        review.reviewer = Some(selection.clone());
        Ok(crate::operations::Output {
            value: serde_json::to_value(review)?,
            request_id: id.clone(),
            usage: usage(response),
        })
    })();
    result.map_err(|mut error| {
        error.request_id = id;
        error
    })
}
fn google_result(e: &Effective, v: &Value, header: Option<String>) -> Result<Generated> {
    let remote = v["id"].as_str().map(str::to_owned).or(header);
    let id = remote.clone();
    let result = (|| {
        if v["status"].as_str().is_some_and(|s| s != "completed") {
            return Err(Error::unknown(
                "Provider interaction has not completed; reconcile its saved ID before retry",
            ));
        }
        let image = matches!(e.media, Media::Image | Media::Infographic);
        let kind = if image {
            "image"
        } else if e.media == Media::Text {
            "text"
        } else {
            "audio"
        };
        let block=v.get("steps").and_then(Value::as_array).into_iter().flatten().filter(|s|s["type"]=="model_output").flat_map(|s|s.get("content").and_then(Value::as_array).into_iter().flatten()).rfind(|b|b["type"]==kind).ok_or_else(||Error::unknown("No requested media block in Gemini response; inspect provider status before retry"))?;
        let bytes = if kind == "text" {
            block["text"]
                .as_str()
                .ok_or_else(|| Error::unknown("Missing text"))?
                .as_bytes()
                .to_vec()
        } else {
            decode(
                block["data"]
                    .as_str()
                    .ok_or_else(|| Error::unknown("Missing media bytes"))?,
            )?
        };
        let format = if image {
            "png"
        } else if e.media == Media::Text {
            "md"
        } else if e.media == Media::Speech {
            "wav"
        } else {
            "mp3"
        };
        Ok(Generated {
            bytes,
            format: format.into(),
            request_id: v["id"].as_str().map(str::to_owned).or(id),
            usage: usage(v),
        })
    })();
    result.map_err(|mut error| {
        error.request_id = remote;
        error
    })
}
#[async_trait]
impl Adapter for HttpAdapter {
    async fn reconcile(&self, input: &Effective, id: &str) -> Result<Reconciled> {
        if input.selection.provider != Provider::Google {
            return Err(Error::input(
                "Provider result retrieval is available for Google interactions; other uncertain calls require account reconciliation before explicit retry",
            ));
        }
        let (response, header) = self.google_interaction(&input.selection.model, id).await?;
        match response["status"].as_str() {
            Some("in_progress" | "requires_action") => Ok(Reconciled::Pending),
            Some("failed" | "cancelled") => Ok(Reconciled::Failed),
            _ => Ok(Reconciled::Complete(google_result(
                input, &response, header,
            )?)),
        }
    }
    async fn generate(&self, e: &Effective) -> Result<Generated> {
        validate(e)?;
        crate::catalog::require_input(e)?;
        let key = credentials::resolve(e.selection.provider, &self.root)?;
        let model = &e.selection.model;
        let prompt = localized_direction(e);
        let image = matches!(e.media, Media::Image | Media::Infographic);
        match e.selection.provider {
            Provider::Openai if image => {
                let request = if e.references.is_empty() {
                    let mut body =
                        json!({"model":model,"prompt":prompt,"n":1,"output_format":"png"});
                    if let Some(size) = &e.settings.size {
                        body["size"] = json!(size);
                    }
                    if let Some(quality) = &e.settings.image_quality {
                        body["quality"] = json!(quality);
                    }
                    self.client
                        .post("https://api.openai.com/v1/images/generations")
                        .bearer_auth(&key)
                        .json(&body)
                } else {
                    let mut form = reqwest::multipart::Form::new()
                        .text("model", model.clone())
                        .text("prompt", prompt.clone())
                        .text("n", "1")
                        .text("output_format", "png");
                    if let Some(size) = &e.settings.size {
                        form = form.text("size", size.clone());
                    }
                    if let Some(quality) = &e.settings.image_quality {
                        form = form.text("quality", quality.clone());
                    }
                    if let Some(fidelity) = &e.settings.input_fidelity {
                        form = form.text("input_fidelity", fidelity.clone());
                    }
                    for r in &e.references {
                        form = form.part(
                            "image[]",
                            reqwest::multipart::Part::bytes(decode(&r.base64)?)
                                .file_name(r.name.clone())
                                .mime_str(&r.mime)
                                .map_err(|_| Error::input("Invalid reference MIME type"))?,
                        );
                    }
                    self.client
                        .post("https://api.openai.com/v1/images/edits")
                        .bearer_auth(&key)
                        .multipart(form)
                };
                let (v, id) = self.json(request).await?;
                let data = v
                    .pointer("/data/0/b64_json")
                    .and_then(Value::as_str)
                    .ok_or_else(|| receipt(Error::unknown("No image in provider response"), &id))?;
                Ok(Generated {
                    bytes: decode(data)?,
                    format: "png".into(),
                    request_id: id,
                    usage: usage(&v),
                })
            }
            Provider::Openai if e.media == Media::Speech => {
                let voice = e.settings.voice.as_ref().unwrap();
                let mut body = json!({"model":model,"input":e.text,"voice":if voice.starts_with("voice_"){json!({"id":voice})}else{json!(voice)},"response_format":"wav"});
                if !prompt.trim().is_empty() {
                    body["instructions"] = json!(prompt);
                }
                let (bytes, id) = self
                    .response(
                        self.client
                            .post("https://api.openai.com/v1/audio/speech")
                            .bearer_auth(&key)
                            .json(&body),
                    )
                    .await?;
                Ok(Generated {
                    bytes,
                    format: "wav".into(),
                    request_id: id,
                    usage: None,
                })
            }
            Provider::Openai | Provider::Openrouter if e.media == Media::Text => {
                let url = if e.selection.provider == Provider::Openai {
                    "https://api.openai.com/v1/chat/completions"
                } else {
                    "https://openrouter.ai/api/v1/chat/completions"
                };
                let (v, id) = self
                    .json(self.client.post(url).bearer_auth(&key).json(
                        &json!({"model":model,"messages":[{"role":"user","content":prompt}]}),
                    ))
                    .await?;
                let text = v
                    .pointer("/choices/0/message/content")
                    .and_then(Value::as_str)
                    .ok_or_else(|| receipt(Error::unknown("No text in model response"), &id))?;
                Ok(Generated {
                    bytes: text.as_bytes().to_vec(),
                    format: "md".into(),
                    request_id: id,
                    usage: usage(&v),
                })
            }
            Provider::Openrouter if image => {
                let (v, id) = self
                    .json(
                        self.client
                            .post("https://openrouter.ai/api/v1/images")
                            .bearer_auth(&key)
                            .json(&json!({"model":model,"prompt":prompt,"output_format":"png"})),
                    )
                    .await?;
                let b = v
                    .pointer("/data/0/b64_json")
                    .and_then(Value::as_str)
                    .ok_or_else(|| receipt(Error::unknown("No image returned"), &id))?;
                Ok(Generated {
                    bytes: decode(b).map_err(|error| receipt(error, &id))?,
                    format: "png".into(),
                    request_id: id,
                    usage: usage(&v),
                })
            }
            Provider::Google => {
                let mut content = vec![];
                if e.media == Media::Speech {
                    content.push(json!({"type":"text","text":e.text,"annotations":[{"type":"speech_metadata","style":prompt}]}));
                } else {
                    let mut prompt = prompt.clone();
                    if e.media == Media::Music {
                        prompt += &format!(
                            "\nRequested duration: {} seconds. {}",
                            e.settings.duration_seconds.unwrap(),
                            if e.settings.instrumental == Some(true) {
                                "Instrumental only; no singing or speech."
                            } else {
                                "Follow the exact lyrics in the brief when supplied."
                            }
                        );
                    }
                    content.push(json!({"type":"text","text":prompt}));
                    for r in &e.references {
                        content.push(json!({"type":"image","data":r.base64,"mime_type":r.mime}));
                    }
                }
                let mut body =
                    json!({"model":model,"input":[{"type":"user_input","content":content}]});
                if e.media == Media::Speech {
                    body["response_format"] = json!({"type":"audio"});
                    body["generation_config"] =
                        json!({"speech_config":[{"voice":e.settings.voice}]});
                }
                let (v, id) = self
                    .json(
                        self.client
                            .post("https://generativelanguage.googleapis.com/v1beta/interactions")
                            .header("x-goog-api-key", key)
                            .json(&body),
                    )
                    .await?;
                google_result(e, &v, id)
            }
            Provider::Fish => {
                let request = if e.settings.voice.is_some() {
                    self.client.post("https://api.fish.audio/v1/tts").bearer_auth(&key).header("model",model).json(&json!({"text":e.text,"reference_id":e.settings.voice,"format":"wav","sample_rate":24000,"normalize":false}))
                } else {
                    #[derive(serde::Serialize)]
                    struct Sample {
                        #[serde(with = "serde_bytes")]
                        audio: Vec<u8>,
                        text: String,
                    }
                    #[derive(serde::Serialize)]
                    struct Body<'a> {
                        text: &'a str,
                        references: Vec<Sample>,
                        format: &'a str,
                        sample_rate: u32,
                        normalize: bool,
                    }
                    let references = e
                        .references
                        .iter()
                        .map(|r| {
                            Ok(Sample {
                                audio: decode(&r.base64)?,
                                text: r.transcript.clone().unwrap(),
                            })
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let bytes = rmp_serde::to_vec_named(&Body {
                        text: e.text.as_deref().unwrap(),
                        references,
                        format: "wav",
                        sample_rate: 24000,
                        normalize: false,
                    })
                    .map_err(|_| Error::input("Cannot encode Fish audio samples"))?;
                    self.client
                        .post("https://api.fish.audio/v1/tts")
                        .bearer_auth(&key)
                        .header("model", model)
                        .header("Content-Type", "application/msgpack")
                        .body(bytes)
                };
                let (bytes, id) = self.response(request).await?;
                Ok(Generated {
                    bytes,
                    format: "wav".into(),
                    request_id: id,
                    usage: None,
                })
            }
            Provider::Elevenlabs => {
                let (url, body) = match e.media {
                    Media::Speech => {
                        let voice = e.settings.voice.as_ref().unwrap();
                        if !voice
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                        {
                            return Err(Error::input("Invalid provider voice ID"));
                        }
                        (
                            format!(
                                "https://api.elevenlabs.io/v1/text-to-speech/{voice}?output_format=mp3_44100_128"
                            ),
                            json!({"text":e.text,"model_id":model}),
                        )
                    }
                    Media::Music => (
                        "https://api.elevenlabs.io/v1/music".into(),
                        json!({"prompt":prompt,"model_id":model,"music_length_ms":e.settings.duration_seconds.unwrap()*1000,"force_instrumental":e.settings.instrumental.unwrap_or(false)}),
                    ),
                    Media::Sfx => (
                        "https://api.elevenlabs.io/v1/sound-generation?output_format=mp3_44100_128"
                            .into(),
                        json!({"text":prompt,"model_id":model,"duration_seconds":e.settings.duration_seconds}),
                    ),
                    _ => return Err(Error::input("Unsupported ElevenLabs media type")),
                };
                let (bytes, id) = self
                    .response(self.client.post(url).header("xi-api-key", key).json(&body))
                    .await?;
                Ok(Generated {
                    bytes,
                    format: "mp3".into(),
                    request_id: id,
                    usage: None,
                })
            }
            _ => Err(Error::input("Unsupported adapter operation")),
        }
    }
}

#[async_trait]
impl crate::voices::Adapter for HttpAdapter {
    async fn clone_voice(
        &self,
        name: &str,
        description: &str,
        recordings: &[Reference],
    ) -> Result<crate::operations::Output> {
        if recordings.is_empty()
            || recordings.len() > 5
            || recordings
                .iter()
                .any(|r| !r.mime.starts_with("audio/") || r.consent_hash.is_none())
        {
            return Err(Error::input(
                "Voice cloning requires selected authorized audio recordings",
            ));
        }
        let key = credentials::resolve(Provider::Elevenlabs, &self.root)?;
        let mut form = reqwest::multipart::Form::new()
            .text("name", name.to_owned())
            .text("description", description.to_owned())
            .text("remove_background_noise", "false");
        for r in recordings {
            form = form.part(
                "files",
                reqwest::multipart::Part::bytes(decode(&r.base64)?)
                    .file_name(r.name.clone())
                    .mime_str(&r.mime)
                    .map_err(|_| Error::input("Invalid cloning sample MIME type"))?,
            );
        }
        let (response, request_id) = self
            .json(
                self.client
                    .post("https://api.elevenlabs.io/v1/voices/add")
                    .header("xi-api-key", key)
                    .multipart(form),
            )
            .await?;
        let id=response["voice_id"].as_str().filter(|id|!id.is_empty()).ok_or_else(||receipt(Error::unknown("Provider may have created a cloned voice but returned no binding; reconcile before retry"),&request_id))?;
        Ok(crate::operations::Output {
            value: json!({"voice_id":id,"requires_verification":response["requires_verification"].as_bool()}),
            request_id,
            usage: usage(&response),
        })
    }
    async fn verify(&self, voice: &str) -> Result<crate::operations::Output> {
        if voice.is_empty()
            || !voice
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(Error::input("Invalid provider voice ID"));
        }
        let key = credentials::resolve(Provider::Elevenlabs, &self.root)?;
        let (response,request_id)=self.json(self.client.get(format!("https://api.elevenlabs.io/v1/voices/{voice}")).header("xi-api-key",key)).await.map_err(|mut error| { if error.exit==7 {error.code="verification_unavailable";error.exit=3;error.message="Voice metadata lookup was interrupted; the saved creation result is intact".into();} error })?;
        let id = response["voice_id"].as_str().ok_or_else(|| {
            Error::new("provider_response", "Voice lookup returned no binding", 3)
        })?;
        Ok(crate::operations::Output {
            value: json!({"voice_id":id,"requires_verification":response.pointer("/voice_verification/requires_verification").and_then(Value::as_bool),"is_verified":response.pointer("/voice_verification/is_verified").and_then(Value::as_bool)}),
            request_id,
            usage: usage(&response),
        })
    }
    async fn design(
        &self,
        selection: &Selection,
        description: &str,
        text: &str,
    ) -> Result<crate::operations::Output> {
        crate::catalog::require_current(selection)?;
        crate::voices::validate_design(selection, description, text)?;
        let key = credentials::resolve(Provider::Elevenlabs, &self.root)?;
        let (response, request_id) = self.json(self.client.post("https://api.elevenlabs.io/v1/text-to-voice/design?output_format=mp3_44100_128").header("xi-api-key",key).json(&json!({"model_id":selection.model,"voice_description":description,"text":text,"auto_generate_text":false,"should_enhance":false,"stream_previews":false}))).await?;
        if response.get("text").and_then(Value::as_str) != Some(text) {
            let mut error = Error::unknown(
                "Provider voice preview changed the exact requested script; inspect provider response before retry",
            );
            error.request_id = request_id;
            return Err(error);
        }
        let previews = (|| {
            let previews = response
                .get("previews")
                .and_then(Value::as_array)
                .filter(|p| !p.is_empty() && p.len() <= 3)
                .ok_or_else(|| {
                    Error::unknown("Provider returned an unexpected voice preview count")
                })?;
            let previews = previews
                .iter()
                .map(|p| {
                    let id = p["generated_voice_id"]
                        .as_str()
                        .filter(|id| !id.is_empty())
                        .ok_or_else(|| Error::unknown("Voice preview has no generated ID"))?;
                    let encoded = p["audio_base_64"]
                        .as_str()
                        .ok_or_else(|| Error::unknown("Voice preview has no audio"))?;
                    let bytes = decode(encoded)?;
                    let mime = p["media_type"]
                        .as_str()
                        .ok_or_else(|| Error::unknown("Voice preview has no media type"))?;
                    Ok(crate::voices::Preview {
                        id: id.into(),
                        language: p["language"].as_str().map(str::to_owned),
                        audio: Reference {
                            blob: None,
                            origin: None,
                            transcript: None,
                            consent_hash: None,
                            role: ReferenceRole::Identity,
                            name: "voice-preview".into(),
                            mime: mime.into(),
                            sha256: crate::storage::hash(&bytes),
                            base64: encoded.into(),
                        },
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(previews)
        })()
        .map_err(|error| receipt(error, &request_id))?;
        Ok(crate::operations::Output {
            value: serde_json::to_value(previews)?,
            request_id,
            usage: usage(&response),
        })
    }
    async fn create(
        &self,
        name: &str,
        description: &str,
        candidate: &str,
    ) -> Result<crate::operations::Output> {
        if name.trim().is_empty()
            || candidate.trim().is_empty()
            || !(20..=1000).contains(&description.chars().count())
        {
            return Err(Error::input(
                "Voice creation needs a name, selected candidate, and 20–1000 character description",
            ));
        }
        let key = credentials::resolve(Provider::Elevenlabs, &self.root)?;
        let (response,request_id)=self.json(self.client.post("https://api.elevenlabs.io/v1/text-to-voice").header("xi-api-key",key).json(&json!({"voice_name":name,"voice_description":description,"generated_voice_id":candidate}))).await?;
        let id=response["voice_id"].as_str().filter(|id|!id.trim().is_empty()).ok_or_else(||receipt(Error::unknown("Provider may have created the voice but returned no binding; reconcile the account before retry"),&request_id))?;
        Ok(crate::operations::Output {
            value: json!({"voice_id":id,"requires_verification":response.pointer("/voice_verification/requires_verification").and_then(Value::as_bool)}),
            request_id,
            usage: usage(&response),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::Mutex};
    struct Fixture {
        status: u16,
        body: Value,
        captured: Mutex<Vec<(String, Value)>>,
        raw: Mutex<Vec<Vec<u8>>>,
    }
    #[async_trait]
    impl Transport for Fixture {
        async fn send(&self, mut request: reqwest::Request) -> Result<WireResponse> {
            let raw = match request.body_mut().take() {
                Some(body) => http_body_util::BodyExt::collect(body)
                    .await
                    .map_err(|_| Error::input("Cannot collect mock request body"))?
                    .to_bytes()
                    .to_vec(),
                None => b"{}".to_vec(),
            };
            self.raw.lock().unwrap().push(raw.clone());
            let body: Value = serde_json::from_slice(&raw).unwrap_or(Value::Null);
            self.captured
                .lock()
                .unwrap()
                .push((request.url().to_string(), body));
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "x-request-id",
                reqwest::header::HeaderValue::from_static("mock-request"),
            );
            headers.insert(
                "retry-after",
                reqwest::header::HeaderValue::from_static("2"),
            );
            Ok(WireResponse {
                status: self.status,
                headers,
                bytes: serde_json::to_vec(&self.body).unwrap(),
            })
        }
    }
    fn request(provider: Provider, model: &str, media: Media) -> Effective {
        Effective {
            assembly: None,
            id: "fixture".into(),
            media,
            selection: Selection {
                provider,
                model: model.into(),
            },
            prompt: if media == Media::Speech {
                String::new()
            } else {
                "An illustration".into()
            },
            text: if media == Media::Speech {
                Some("Exact words.".into())
            } else {
                None
            },
            locale: None,
            settings: Settings {
                voice: if media == Media::Speech {
                    Some("alloy".into())
                } else {
                    None
                },
                duration_seconds: if matches!(media, Media::Music | Media::Sfx) {
                    Some(3)
                } else {
                    None
                },
                ..Default::default()
            },
            character_hash: None,
            references: vec![],
            depends_on: vec![],
            output: None,
            turns: vec![],
            audiobook: None,
        }
    }
    fn fixture(status: u16, body: Value) -> Arc<Fixture> {
        Arc::new(Fixture {
            status,
            body,
            captured: Mutex::new(vec![]),
            raw: Mutex::new(vec![]),
        })
    }
    fn root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join(".env"),"OPENAI_API_KEY=fake\nGEMINI_API_KEY=fake\nFISH_API_KEY=fake\nOPENROUTER_API_KEY=fake\nELEVENLABS_API_KEY=fake\n").unwrap();
        root
    }
    #[tokio::test]
    async fn current_image_families_and_snapshots_keep_exact_models_and_supported_settings() {
        let root = root();
        for model in [
            "gpt-image-2.5-flare",
            "gpt-image-2.5-flare-2026-09-08",
            "gpt-image-2.5-sunburst",
            "gpt-image-2.5-sunburst-2026-09-08",
        ] {
            let transport = fixture(
                200,
                json!({"data":[{"b64_json":STANDARD.encode(b"fixture")}]}),
            );
            let adapter = HttpAdapter::with_transport(root.path(), transport.clone()).unwrap();
            let mut e = request(Provider::Openai, model, Media::Image);
            e.settings.image_quality = Some("max".into());
            e.settings.size = Some("2048x2048".into());
            adapter.generate(&e).await.unwrap();
            let calls = transport.captured.lock().unwrap();
            assert_eq!(calls[0].1["model"], model);
            assert_eq!(calls[0].1["quality"], "max");
            assert_eq!(calls[0].1["size"], "2048x2048");
            assert!(calls[0].1.get("input_fidelity").is_none());
        }
        for size in ["1x1", "2049x2048", "4096x4096", "2048x100"] {
            assert!(!valid_image_size("gpt-image-2.5-flare", size));
        }
    }
    #[tokio::test]
    async fn image_response_and_usage_are_translated_without_echoing_arbitrary_fields() {
        let root = root();
        let f = fixture(
            200,
            json!({"data":[{"b64_json":STANDARD.encode(b"image-bytes")}],"usage":{"total_tokens":3,"cost":0.02,"authorization":"secret"}}),
        );
        let result = HttpAdapter::with_transport(root.path(), f.clone())
            .unwrap()
            .generate(&request(
                Provider::Openai,
                "gpt-image-2.5-flare",
                Media::Image,
            ))
            .await
            .unwrap();
        assert_eq!(result.bytes, b"image-bytes");
        assert_eq!(result.usage, Some(json!({"total_tokens":3,"cost":0.02})));
        assert_eq!(result.request_id.as_deref(), Some("mock-request"));
        let calls = f.captured.lock().unwrap();
        assert!(calls[0].0.ends_with("/images/generations"));
        assert_eq!(calls[0].1["model"], "gpt-image-2.5-flare");
    }
    #[tokio::test]
    async fn google_steps_and_openrouter_image_response_are_supported() {
        let root = root();
        let f = fixture(
            200,
            json!({"id":"interaction-1","steps":[{"type":"model_output","content":[{"type":"image","data":STANDARD.encode(b"png")}]}]}),
        );
        let result = HttpAdapter::with_transport(root.path(), f.clone())
            .unwrap()
            .generate(&request(
                Provider::Google,
                "gemini-nano-banana-2.1",
                Media::Image,
            ))
            .await
            .unwrap();
        assert_eq!(result.bytes, b"png");
        assert_eq!(result.request_id.as_deref(), Some("interaction-1"));
        assert!(
            f.captured.lock().unwrap()[0]
                .0
                .ends_with("/v1beta/interactions")
        );
        let f = fixture(200, json!({"data":[{"b64_json":STANDARD.encode(b"png")}]}));
        HttpAdapter::with_transport(root.path(), f.clone())
            .unwrap()
            .generate(&request(
                Provider::Openrouter,
                "openai/gpt-image-2.5-flare",
                Media::Image,
            ))
            .await
            .unwrap();
        assert!(f.captured.lock().unwrap()[0].0.ends_with("/api/v1/images"));
    }
    #[tokio::test]
    async fn speech_music_and_effect_requests_keep_exact_model_and_inputs() {
        let root = root();
        for (provider, model, media, path) in [
            (
                Provider::Openai,
                "gpt-4o-mini-tts",
                Media::Speech,
                "/audio/speech",
            ),
            (Provider::Fish, "s2.1-pro-free", Media::Speech, "/v1/tts"),
            (
                Provider::Elevenlabs,
                "music_v2_5",
                Media::Music,
                "/v1/music",
            ),
            (
                Provider::Elevenlabs,
                "eleven_text_to_sound_v2",
                Media::Sfx,
                "/v1/sound-generation?output_format=mp3_44100_128",
            ),
        ] {
            let f = fixture(200, json!({"binary":"fixture"}));
            HttpAdapter::with_transport(root.path(), f.clone())
                .unwrap()
                .generate(&request(provider, model, media))
                .await
                .unwrap();
            let calls = f.captured.lock().unwrap();
            assert!(calls[0].0.ends_with(path));
            if media == Media::Speech {
                let field = if provider == Provider::Openai {
                    "input"
                } else {
                    "text"
                };
                assert_eq!(calls[0].1[field], "Exact words.");
            }
            if provider == Provider::Elevenlabs {
                assert_eq!(calls[0].1["model_id"], model);
            }
        }
    }
    #[tokio::test]
    async fn rejected_and_uncertain_requests_redact_response_bodies() {
        let root = root();
        for (status, exit, retry) in [(401, 3, false), (429, 3, true), (503, 7, false)] {
            let f = fixture(status, json!({"error":"Authorization Bearer secret-echo"}));
            let error = HttpAdapter::with_transport(root.path(), f)
                .unwrap()
                .generate(&request(
                    Provider::Openai,
                    "gpt-image-2.5-flare",
                    Media::Image,
                ))
                .await
                .err()
                .unwrap();
            assert_eq!(error.exit, exit);
            assert_eq!(error.retryable, retry);
            assert_eq!(error.request_id.as_deref(), Some("mock-request"));
            assert!(!error.to_string().contains("secret-echo"));
            if retry {
                assert_eq!(error.retry_after_seconds, Some(2));
            }
        }
    }
    #[tokio::test]
    async fn capability_validation_precedes_transport_submission() {
        let root = root();
        let f = fixture(200, json!({}));
        let mut input = request(Provider::Fish, "unrecognized-model", Media::Speech);
        input.settings.voice = Some("saved-voice".into());
        assert!(
            HttpAdapter::with_transport(root.path(), f.clone())
                .unwrap()
                .generate(&input)
                .await
                .is_err()
        );
        assert!(f.captured.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn voice_design_and_creation_use_exact_inputs_and_exclude_account_metadata() {
        let root = root();
        let text = "An exact fictional narrator preview script. ".repeat(3);
        let description =
            "A calm fictional narrator with clear diction and a relaxed conversational pace.";
        let bytes = crate::audio::wav(&[0.1; 2400]).unwrap();
        let f = fixture(
            200,
            json!({"text":text,"previews":[{"generated_voice_id":"candidate-1","audio_base_64":STANDARD.encode(bytes),"media_type":"audio/wav","language":"en"}],"billing":"private"}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let selection = Selection {
            provider: Provider::Elevenlabs,
            model: "eleven_ttv_v3".into(),
        };
        let output = crate::voices::Adapter::design(&adapter, &selection, description, &text)
            .await
            .unwrap();
        assert_eq!(output.value[0]["id"], "candidate-1");
        assert!(!output.value.to_string().contains("private"));
        {
            let calls = f.captured.lock().unwrap();
            assert!(
                calls[0]
                    .0
                    .contains("/v1/text-to-voice/design?output_format=mp3_44100_128")
            );
            assert_eq!(calls[0].1["text"], text);
            assert_eq!(calls[0].1["model_id"], "eleven_ttv_v3");
            assert_eq!(calls[0].1["auto_generate_text"], false);
        }
        let f = fixture(
            200,
            json!({"voice_id":"saved-voice-1","voice_verification":{"requires_verification":false},"billing":"private","preview_url":"https://example.invalid/credential-bearing"}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let output =
            crate::voices::Adapter::create(&adapter, "Narrator", description, "candidate-1")
                .await
                .unwrap();
        assert_eq!(
            output.value,
            json!({"voice_id":"saved-voice-1","requires_verification":false})
        );
        assert_eq!(
            f.captured.lock().unwrap()[0].1["generated_voice_id"],
            "candidate-1"
        );
    }
    #[tokio::test]
    async fn image_review_retains_provider_receipt_and_validates_artifact_identity() {
        let root = root();
        let review = json!({"schema_version":1,"artifact":"artifact-1","quality":"passed","provenance":"ai","findings":[],"reviewer":null});
        let f = fixture(
            200,
            json!({"choices":[{"message":{"content":review.to_string()}}],"usage":{"total_tokens":4,"authorization":"private"}}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f).unwrap();
        let output = adapter
            .review_image(
                &Selection {
                    provider: Provider::Openai,
                    model: "gpt-6-luna".into(),
                },
                "Review source",
                b"fixture-image",
                "artifact-1",
                &[],
            )
            .await
            .unwrap();
        assert_eq!(output.request_id.as_deref(), Some("mock-request"));
        assert_eq!(output.usage, Some(json!({"total_tokens":4})));
        assert_eq!(output.value["artifact"], "artifact-1");
    }
    #[tokio::test]
    async fn voice_verification_reads_only_one_binding_and_redacts_account_fields() {
        let root = root();
        let f = fixture(
            200,
            json!({"voice_id":"saved-voice-1","voice_verification":{"requires_verification":true,"is_verified":true},"billing":"private","preview_url":"https://example.invalid/private"}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let output = crate::voices::Adapter::verify(&adapter, "saved-voice-1")
            .await
            .unwrap();
        assert_eq!(
            output.value,
            json!({"voice_id":"saved-voice-1","requires_verification":true,"is_verified":true})
        );
        assert_eq!(
            f.captured.lock().unwrap()[0].0,
            "https://api.elevenlabs.io/v1/voices/saved-voice-1"
        );
        assert!(
            crate::voices::Adapter::verify(&adapter, "invalid/id")
                .await
                .is_err()
        );
        assert_eq!(f.captured.lock().unwrap().len(), 1);
        let adapter =
            HttpAdapter::with_transport(root.path(), fixture(503, json!({"error":"secret"})))
                .unwrap();
        let error = crate::voices::Adapter::verify(&adapter, "saved-voice-1")
            .await
            .err()
            .unwrap();
        assert_eq!(error.exit, 3);
        assert!(!error.to_string().contains("secret"));
    }

    #[tokio::test]
    async fn voice_catalog_is_bounded_and_returns_only_selected_metadata() {
        let root = root();
        let items = (0..25).map(|i| json!({"_id":format!("voice-{i}"),"title":"Narrator","billing":"private","authorization":"secret"})).collect::<Vec<_>>();
        let f = fixture(
            200,
            json!({"items":items,"has_more":true,"billing":"private"}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let result = adapter.voices(Provider::Fish, true).await.unwrap();
        assert_eq!(result["voices"].as_array().unwrap().len(), 20);
        assert_eq!(result["voices"][0]["id"], "voice-0");
        assert_eq!(result["public_catalog"], true);
        assert_eq!(result["has_more"], true);
        assert!(!result.to_string().contains("private"));
        assert!(!result.to_string().contains("secret"));
        assert!(
            f.captured.lock().unwrap()[0]
                .0
                .ends_with("self=false&licensed=true")
        );
        assert!(adapter.voices(Provider::Elevenlabs, true).await.is_err());
        assert_eq!(f.captured.lock().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn fish_reference_synthesis_uses_messagepack_binary_and_exact_transcript() {
        let root = root();
        let f = fixture(200, json!({}));
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let mut e = request(Provider::Fish, "s2.1-pro-free", Media::Speech);
        e.settings.voice = None;
        let bytes = crate::audio::wav(&[0.1; 2400]).unwrap();
        e.references.push(Reference {
            blob: None,
            origin: None,
            name: "sample.wav".into(),
            mime: "audio/wav".into(),
            role: ReferenceRole::Identity,
            sha256: crate::storage::hash(&bytes),
            base64: STANDARD.encode(&bytes),
            transcript: Some("Exact sample!\n".into()),
            consent_hash: Some("authorized-fixture".into()),
        });
        adapter.generate(&e).await.unwrap();
        #[derive(serde::Deserialize)]
        struct Sample {
            #[serde(with = "serde_bytes")]
            audio: Vec<u8>,
            text: String,
        }
        #[derive(serde::Deserialize)]
        struct Body {
            text: String,
            references: Vec<Sample>,
        }
        let body: Body = rmp_serde::from_slice(&f.raw.lock().unwrap()[0]).unwrap();
        assert_eq!(body.text, "Exact words.");
        assert_eq!(body.references[0].audio, bytes);
        assert_eq!(body.references[0].text, "Exact sample!\n");
        e.references[0].consent_hash = None;
        assert!(adapter.generate(&e).await.is_err());
        assert_eq!(f.captured.lock().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn audio_review_preserves_source_and_validates_assessment_before_recording() {
        let root = root();
        let review = json!({"schema_version":1,"artifact":"speech-1","quality":"passed","provenance":"ai","reviewer":null,"findings":[]});
        let f = fixture(
            200,
            json!({"id":"review_interaction","steps":[{"type":"model_output","content":[{"type":"text","text":review.to_string()}]}],"usage":{"total_tokens":9}}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let selection = Selection {
            provider: Provider::Google,
            model: "gemini-3.8-flash".into(),
        };
        let bytes = crate::audio::wav(&[0.1; 2400]).unwrap();
        let output = adapter
            .review_audio(
                &selection,
                "Exact source: Hello.",
                &bytes,
                "wav",
                "speech-1",
                &[],
            )
            .await
            .unwrap();
        assert_eq!(output.request_id.as_deref(), Some("review_interaction"));
        assert_eq!(output.value["quality"], "passed");
        assert_eq!(output.usage, Some(json!({"total_tokens":9})));
        {
            let captured = f.captured.lock().unwrap();
            assert_eq!(
                captured[0].1["input"][0]["content"][0]["text"],
                "Exact source: Hello."
            );
            assert_eq!(
                captured[0].1["input"][0]["content"][1]["data"],
                STANDARD.encode(bytes)
            );
        }
        assert!(
            adapter
                .review_audio(
                    &selection,
                    "Source",
                    &[0; 16 * 1024 * 1024],
                    "wav",
                    "speech-1",
                    &[]
                )
                .await
                .is_err()
        );
        assert_eq!(f.captured.lock().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn google_retrieval_never_submits_generation_and_checks_receipt_identity() {
        let root = root();
        let f = fixture(
            200,
            json!({"id":"interaction_1","status":"completed","model":"nano-banana-2.1","steps":[{"type":"model_output","content":[{"type":"image","data":STANDARD.encode(b"saved media")}]}]}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let e = request(Provider::Google, "nano-banana-2.1", Media::Image);
        let Reconciled::Complete(result) = adapter.reconcile(&e, "interaction_1").await.unwrap()
        else {
            panic!("Expected recovered bytes")
        };
        assert_eq!(result.bytes, b"saved media");
        assert!(
            f.captured.lock().unwrap()[0]
                .0
                .ends_with("/interactions/interaction_1")
        );
        assert!(adapter.reconcile(&e, "unsafe/id").await.is_err());
        assert_eq!(f.captured.lock().unwrap().len(), 1);
        assert!(adapter.reconcile(&e, "interaction_2").await.is_err());
    }
    #[tokio::test]
    async fn catalog_filters_are_encoded_and_pagination_is_bounded() {
        let root = root();
        let f = fixture(
            200,
            json!({"voices":[{"voice_id":"voice-1","name":"Narrator"}],"next_page_token":"continuation & token"}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let options = CatalogOptions {
            search: Some("Alice & Bob".into()),
            language: Some("en".into()),
            category: Some("premade".into()),
            page_size: 3,
            ..Default::default()
        };
        let output = adapter
            .voice_catalog(Provider::Elevenlabs, &options)
            .await
            .unwrap();
        assert_eq!(output["pages_fetched"], 1);
        assert_eq!(output["has_more"], true);
        let url = f.captured.lock().unwrap()[0].0.clone();
        assert!(url.contains("search=Alice+%26+Bob"));
        assert!(url.contains("language=en"));
        let invalid = CatalogOptions {
            max_pages: 21,
            ..Default::default()
        };
        assert!(
            adapter
                .voice_catalog(Provider::Elevenlabs, &invalid)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn recorded_clone_targets_explicit_account_creation_and_sanitizes_response() {
        let root = root();
        let f = fixture(
            200,
            json!({"voice_id":"saved-clone","requires_verification":true,"billing":"private","api_key":"secret"}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let audio = crate::audio::wav(&[0.1; 2400]).unwrap();
        let recording = Reference {
            blob: None,
            origin: None,
            name: "selected.wav".into(),
            mime: "audio/wav".into(),
            sha256: crate::storage::hash(&audio),
            base64: STANDARD.encode(audio),
            role: ReferenceRole::Identity,
            transcript: None,
            consent_hash: Some("offline-permission".into()),
        };
        let result = crate::voices::Adapter::clone_voice(
            &adapter,
            "Offline narrator",
            "A fictional narrator",
            &[recording],
        )
        .await
        .unwrap();
        assert_eq!(
            result.value,
            json!({"voice_id":"saved-clone","requires_verification":true})
        );
        assert_eq!(result.request_id.as_deref(), Some("mock-request"));
        assert_eq!(
            f.captured.lock().unwrap()[0].0,
            "https://api.elevenlabs.io/v1/voices/add"
        );
    }
    #[tokio::test]
    async fn malformed_paid_responses_keep_receipts_for_explicit_recovery() {
        let root = root();
        let f = fixture(200, json!({"data":[]}));
        let adapter = HttpAdapter::with_transport(root.path(), f).unwrap();
        let error = adapter
            .generate(&request(
                Provider::Openai,
                "gpt-image-2.5-flare",
                Media::Image,
            ))
            .await
            .err()
            .unwrap();
        assert_eq!(error.exit, 7);
        assert_eq!(error.request_id.as_deref(), Some("mock-request"));
        let f = fixture(200, json!({"voice_id":null}));
        let adapter = HttpAdapter::with_transport(root.path(), f).unwrap();
        let bytes = crate::audio::wav(&[0.1; 2400]).unwrap();
        let recording = Reference {
            blob: None,
            origin: None,
            name: "selected.wav".into(),
            mime: "audio/wav".into(),
            sha256: crate::storage::hash(&bytes),
            base64: STANDARD.encode(bytes),
            role: ReferenceRole::Identity,
            transcript: None,
            consent_hash: Some("fixture".into()),
        };
        let error = crate::voices::Adapter::clone_voice(
            &adapter,
            "Fixture",
            "Fixture description",
            &[recording],
        )
        .await
        .err()
        .unwrap();
        assert_eq!(error.exit, 7);
        assert_eq!(error.request_id.as_deref(), Some("mock-request"));
    }
    #[tokio::test]
    async fn identity_edit_sends_high_fidelity_and_both_exact_reference_images() {
        let root = root();
        let f = fixture(
            200,
            json!({"data":[{"b64_json":STANDARD.encode(b"image-result")}]}),
        );
        let adapter = HttpAdapter::with_transport(root.path(), f.clone()).unwrap();
        let mut e = request(Provider::Openai, "gpt-image-2.5-flare", Media::Image);
        e.settings.input_fidelity = None;
        for (name, bytes) in [
            ("mira.png", b"first-identity".as_slice()),
            ("kito.png", b"second-identity".as_slice()),
        ] {
            e.references.push(Reference {
                blob: None,
                origin: None,
                name: name.into(),
                mime: "image/png".into(),
                role: ReferenceRole::Identity,
                sha256: crate::storage::hash(bytes),
                base64: STANDARD.encode(bytes),
                transcript: None,
                consent_hash: None,
            });
        }
        adapter.generate(&e).await.unwrap();
        let raw = f.raw.lock().unwrap()[0].clone();
        let body = String::from_utf8_lossy(&raw);
        assert!(!body.contains("name=\"input_fidelity\""));
        assert!(body.contains("first-identity"));
        assert!(body.contains("second-identity"));
        assert!(body.contains("filename=\"mira.png\""));
        assert!(body.contains("filename=\"kito.png\""));
        e.settings.input_fidelity = Some("low".into());
        assert!(adapter.generate(&e).await.is_err());
        e.settings.input_fidelity = Some("high".into());
        e.selection.model = "gpt-image-1-mini".into();
        assert!(adapter.generate(&e).await.is_err());
        e.selection.model = "gpt-image-2".into();
        assert!(adapter.generate(&e).await.is_err());
        assert_eq!(f.captured.lock().unwrap().len(), 1);
        e.settings.input_fidelity = None;
        assert!(adapter.generate(&e).await.is_err());
        e.selection.model = "gpt-image-2.5-sunburst".into();
        adapter.generate(&e).await.unwrap();
        assert!(
            !String::from_utf8_lossy(&f.raw.lock().unwrap()[1]).contains("name=\"input_fidelity\"")
        );
    }
}
