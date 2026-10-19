//! Dated, reviewed model policy. Discovery never grants generation permission.
use crate::{
    contracts::*,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;

pub const VERIFIED_AT: &str = "2026-10-14";
/// Selection order only; this never authorizes a provider fallback during generation.
pub fn provider_preference(provider: Provider) -> usize {
    match provider {
        Provider::Google => 0,
        Provider::Openai => 1,
        Provider::Fish => 2,
        Provider::Openrouter => 3,
        Provider::Elevenlabs => 4,
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub provider: Provider,
    pub model: String,
    pub media: Media,
    pub name: String,
    pub description: String,
    pub current: bool,
    pub recommended: bool,
    pub supported: bool,
    pub lifecycle: String,
    pub source: String,
}
pub fn models() -> Vec<Model> {
    serde_json::from_str::<Value>(include_str!("../assets/models.json"))
        .and_then(|v| serde_json::from_value(v["models"].clone()))
        .expect("Bundled model catalog is validated by offline tests")
}
pub fn target(media: Media) -> Media {
    match media {
        Media::Infographic => Media::Image,
        Media::Podcast | Media::Audiobook => Media::Speech,
        other => other,
    }
}
/// Lifecycle guard, separate from adapter capability validation. Dedicated voice-design
/// models may be current while not selectable as ordinary speech-generation models.
pub fn require_current(selection: &Selection) -> Result<()> {
    if models().iter().any(|m| {
        m.provider == selection.provider
            && m.model == selection.model
            && m.current
            && m.lifecycle == "stable"
    }) {
        return Ok(());
    }
    let mut error = Error::new(
        "model_not_current",
        format!(
            "Model {} is not an approved current stable model (catalog verified {VERIFIED_AT}). Inspect framechord models, then explicitly update the job/configuration. For a pinned character, create a new character version and accept new references; never relabel the original model. Existing results remain readable.",
            selection.model
        ),
        2,
    );
    error.corrective_argv = vec!["framechord".into(), "models".into()];
    Err(error)
}
pub fn require_input(input: &Effective) -> Result<()> {
    if input.turns.is_empty() {
        require_current(&input.selection)?;
    }
    for turn in &input.turns {
        require_input(turn)?;
    }
    Ok(())
}
#[derive(Default)]
pub struct Filter<'a> {
    pub providers: &'a [Provider],
    pub media: Option<Media>,
    pub search: Option<&'a str>,
    pub recommended: bool,
    pub all: bool,
    pub sort_name: bool,
}
pub fn listing(root: &Path, filter: &Filter<'_>) -> Value {
    let mut rows = models()
        .into_iter()
        .filter(|m| {
            (filter.providers.is_empty() || filter.providers.contains(&m.provider))
                && filter.media.is_none_or(|media| target(media) == m.media)
                && (!filter.recommended || m.recommended)
                && (filter.all || (m.current && m.supported))
                && filter.search.is_none_or(|q| {
                    format!("{} {} {}", m.name, m.model, m.description)
                        .to_lowercase()
                        .contains(&q.to_lowercase())
                })
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|m| {
        (
            if filter.sort_name {
                false
            } else {
                !m.recommended
            },
            if filter.sort_name {
                m.name.to_lowercase()
            } else {
                String::new()
            },
            provider_preference(m.provider),
            m.model.clone(),
        )
    });
    let cached =
        crate::storage::yaml::<Value>(&crate::project::directory(root).join("cache/models.yaml"))
            .ok();
    let mut entries = rows
        .into_iter()
        .map(|row| {
            let mut value = serde_json::to_value(&row).expect("Catalog entry serializes");
            if let Some(cap) = crate::providers::capabilities().into_iter().find(|c| {
                c["provider"] == value["provider"]
                    && c["model"] == value["model"]
                    && c["media"] == value["media"]
            }) {
                for (k, v) in cap.as_object().unwrap() {
                    value.as_object_mut().unwrap().insert(k.clone(), v.clone());
                }
            }
            value
        })
        .collect::<Vec<_>>();
    let pricing = crate::pricing::bundled();
    for value in &mut entries {
        if let (Ok(provider), Some(model)) = (
            serde_json::from_value::<Provider>(value["provider"].clone()),
            value["model"].as_str(),
        ) {
            let selection = Selection {
                provider,
                model: model.to_owned(),
            };
            value["pricing"] = json!(crate::pricing::for_model(&pricing, &selection));
        }
    }
    if filter.all
        && !filter.recommended
        && let Some(cached) = &cached
        && let Some(providers) = cached["catalogs"].as_array()
    {
        for provider in providers {
            let Ok(p) = serde_json::from_value::<Provider>(provider["provider"].clone()) else {
                continue;
            };
            if !filter.providers.is_empty() && !filter.providers.contains(&p) {
                continue;
            }
            if let Some(models) = provider["models"].as_array() {
                for model in models {
                    let Some(id) = model["model"].as_str() else {
                        continue;
                    };
                    if entries
                        .iter()
                        .any(|m| m["provider"] == provider["provider"] && m["model"] == id)
                    {
                        continue;
                    }
                    if filter
                        .search
                        .is_some_and(|q| !id.to_lowercase().contains(&q.to_lowercase()))
                    {
                        continue;
                    }
                    if filter.media.is_some_and(|m| {
                        model["media"]
                            .as_str()
                            .is_some_and(|v| serde_json::to_value(target(m)).unwrap() != v)
                    }) {
                        continue;
                    }
                    entries.push(json!({"provider":p,"model":id,"name":id,"description":"Discovered provider model; purpose and execution support may be unverified","media":model["media"],"current":false,"supported":false,"recommended":false,"lifecycle":"unreviewed","source":"explicit cached provider discovery"}));
                }
            }
        }
    }
    json!({"models":entries,"verified_at":VERIFIED_AT,"pricing_verified_at":pricing.verified_at,"policy":"current_stable_only","account_access_verified":false,
        "cached_discovery":cached,"text_providers":["openai","google","openrouter"],
        "voice_design_models":[{"provider":"elevenlabs","model":"eleven_ttv_v3","preview_candidates":true,"explicit_voice_creation":true}],
        "image_review_providers":["openai","openrouter"],"audio_review_models":["gemini-3.8-flash"],"audio_review_provider":"google"})
}
