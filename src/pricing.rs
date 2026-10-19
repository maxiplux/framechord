//! Dated, official list prices for labelled cost estimates. Estimates are a separate,
//! read-only view: provider-reported cost is never replaced, cached reuse is never
//! charged, and anything the table cannot price stays unknown rather than zero.
use crate::{
    catalog,
    contracts::*,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceUnit {
    Image,
    AudioSecond,
    Character,
    Song,
    TextTokenIn,
    TextTokenOut,
    AudioTokenOut,
    ImageTokenIn,
    ImageTokenOut,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub provider: Provider,
    pub model: String,
    pub media: Media,
    pub unit: PriceUnit,
    pub qualifier: Option<String>,
    pub usd: f64,
    pub per: u64,
    pub source: String,
    pub valid_until: Option<String>,
    pub note: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub schema_version: u32,
    pub currency: String,
    pub verified_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources_reviewed: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<String>,
    pub entries: Vec<Price>,
}
pub const BUNDLED_SOURCE: &str = "bundled assets/pricing.json";

pub fn bundled() -> Table {
    serde_json::from_str(include_str!("../assets/pricing.json"))
        .expect("Bundled price table is validated by offline tests")
}
/// Reads an explicit JSON or YAML table and validates it before use.
pub fn load(path: &Path) -> Result<Table> {
    let table: Table = if path
        .extension()
        .is_some_and(|extension| extension == "yaml" || extension == "yml")
    {
        crate::storage::yaml(path)?
    } else {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text)
            .map_err(|error| Error::input(format!("Invalid price table JSON: {error}")))?
    };
    validate(&table)?;
    Ok(table)
}
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    }
}
/// `YYYY-MM-DD` naming a day that exists in that month, leap years included.
fn iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    let shaped = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit());
    if !shaped {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        value[..4].parse::<u32>(),
        value[5..7].parse::<u32>(),
        value[8..10].parse::<u32>(),
    ) else {
        return false;
    };
    (1..=days_in_month(year, month)).contains(&day)
}
pub fn validate(table: &Table) -> Result<()> {
    if table.schema_version != 1 {
        return Err(Error::input("Price table schema_version must be 1"));
    }
    if table.currency != "USD" {
        return Err(Error::input("Price table currency must be USD"));
    }
    if !iso_date(&table.verified_at) {
        return Err(Error::input(format!(
            "Price table verified_at {:?} must be a real calendar date (YYYY-MM-DD)",
            table.verified_at
        )));
    }
    let mut seen = HashSet::new();
    for entry in &table.entries {
        if !(entry.usd.is_finite() && entry.usd > 0.0) || entry.per == 0 {
            return Err(Error::input(format!(
                "Price entry for {} must have a positive usd rate and per quantity",
                entry.model
            )));
        }
        if !entry.source.starts_with("https://") {
            return Err(Error::input(format!(
                "Price entry for {} needs an https source",
                entry.model
            )));
        }
        if let Some(until) = entry.valid_until.as_deref().filter(|d| !iso_date(d)) {
            return Err(Error::input(format!(
                "Price entry for {} has valid_until {until:?}, which is not a real calendar date (YYYY-MM-DD)",
                entry.model
            )));
        }
        if !seen.insert((
            format!("{:?}", entry.provider),
            entry.model.clone(),
            entry.unit,
            entry.qualifier.clone(),
        )) {
            return Err(Error::input(format!(
                "Duplicate price entry for {} {:?} {:?}",
                entry.model, entry.unit, entry.qualifier
            )));
        }
    }
    Ok(())
}
pub fn for_model<'a>(table: &'a Table, selection: &Selection) -> Vec<&'a Price> {
    table
        .entries
        .iter()
        .filter(|entry| entry.provider == selection.provider && entry.model == selection.model)
        .collect()
}
/// Resolution tier by pixel count, matching how image prices are published.
pub fn image_tier(width: u32, height: u32) -> &'static str {
    let pixels = u64::from(width) * u64::from(height);
    if pixels <= 600_000 {
        "0.5K"
    } else if pixels <= 2_000_000 {
        "1K"
    } else if pixels <= 8_000_000 {
        "2K"
    } else {
        "4K"
    }
}
pub fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
#[derive(Debug, Clone)]
pub struct Breakdown {
    pub usd: f64,
    pub quantity: f64,
    pub unit: PriceUnit,
    pub qualifier: Option<String>,
    pub rate_usd: Option<f64>,
    pub rate_per: Option<u64>,
    pub partial: Option<String>,
    pub basis: String,
    pub source: String,
}
#[derive(Debug, Clone)]
pub enum Estimate {
    Estimated(Breakdown),
    ProviderReported(f64),
    Cached,
    /// A podcast parent that only assembles clips locally; its turns are separate jobs.
    AssembledLocally,
    NotEstimable(String),
}
fn quantity_value(quantity: f64) -> Value {
    if quantity.fract() == 0.0 && quantity >= 0.0 {
        json!(quantity as u64)
    } else {
        json!(round6(quantity))
    }
}
fn usage_number(usage: Option<&Value>, keys: &[&str]) -> Option<f64> {
    let usage = usage?;
    keys.iter()
        .find_map(|key| usage.get(key).and_then(Value::as_f64))
}
fn missing(table: &Table, selection: &Selection) -> String {
    format!(
        "no price entry for {} {} in the table verified {}",
        serde_json::to_value(selection.provider)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default(),
        selection.model,
        table.verified_at
    )
}
fn estimate_image(table: &Table, job: &JobRecord) -> Estimate {
    let prices = for_model(table, &job.input.selection);
    if prices.is_empty() {
        return Estimate::NotEstimable(missing(table, &job.input.selection));
    }
    if prices.iter().any(|p| p.unit == PriceUnit::Image) {
        let mut usd = 0.0;
        let mut tiers = vec![];
        let mut source = String::new();
        for artifact in &job.artifacts {
            let (Some(width), Some(height)) = (artifact.width, artifact.height) else {
                return Estimate::NotEstimable(format!(
                    "artifact {} has no recorded dimensions",
                    artifact.id
                ));
            };
            let tier = image_tier(width, height);
            let Some(price) = prices
                .iter()
                .find(|p| p.unit == PriceUnit::Image && p.qualifier.as_deref() == Some(tier))
            else {
                return Estimate::NotEstimable(format!(
                    "no {tier} image price for {} (artifact {width}x{height})",
                    job.input.selection.model
                ));
            };
            usd += price.usd / price.per as f64;
            tiers.push(format!("{tier} from {width}x{height} at ${}", price.usd));
            source = price.source.clone();
        }
        let first_tier = tiers
            .first()
            .and_then(|t| t.split(' ').next())
            .map(str::to_owned);
        let uniform = tiers
            .iter()
            .all(|t| t.split(' ').next() == first_tier.as_deref());
        let count = job.artifacts.len();
        return Estimate::Estimated(Breakdown {
            usd: round6(usd),
            quantity: count as f64,
            unit: PriceUnit::Image,
            qualifier: if uniform { first_tier } else { None },
            rate_usd: if uniform {
                Some(round6(usd / count as f64))
            } else {
                None
            },
            rate_per: if uniform { Some(1) } else { None },
            partial: None,
            basis: format!("{count} image(s): {}", tiers.join("; ")),
            source,
        });
    }
    let Some(price) = prices.iter().find(|p| p.unit == PriceUnit::ImageTokenOut) else {
        return Estimate::NotEstimable(missing(table, &job.input.selection));
    };
    let Some(output_tokens) = usage_number(
        job.usage.as_ref(),
        &["output_tokens", "completion_tokens", "candidatesTokenCount"],
    ) else {
        return Estimate::NotEstimable(format!(
            "no provider token usage recorded; {} is priced per token",
            job.input.selection.model
        ));
    };
    let mut usd = output_tokens * price.usd / price.per as f64;
    let mut basis = format!(
        "{output_tokens} output image tokens × ${}/{}",
        price.usd, price.per
    );
    // OpenAI reports `input_tokens_details` split into text and image counts; both
    // rates must be in the table before input tokens can be priced, otherwise the
    // estimate stays output-only and says so.
    let details = job
        .usage
        .as_ref()
        .and_then(|usage| usage.get("input_tokens_details"));
    let text_tokens = usage_number(details, &["text_tokens"]);
    let image_tokens = usage_number(details, &["image_tokens"]);
    let text_price = prices.iter().find(|p| p.unit == PriceUnit::TextTokenIn);
    let image_price = prices.iter().find(|p| p.unit == PriceUnit::ImageTokenIn);
    let partial = match (text_tokens, image_tokens, text_price, image_price) {
        (Some(text_tokens), Some(image_tokens), Some(text_price), Some(image_price)) => {
            usd += text_tokens * text_price.usd / text_price.per as f64
                + image_tokens * image_price.usd / image_price.per as f64;
            basis += &format!(
                " + {text_tokens} text input tokens × ${}/{} + {image_tokens} image input tokens × ${}/{}",
                text_price.usd, text_price.per, image_price.usd, image_price.per
            );
            None
        }
        (Some(_), Some(_), text_price, image_price) => {
            let missing = [
                (text_price.is_none(), "text_token_in"),
                (image_price.is_none(), "image_token_in"),
            ]
            .into_iter()
            .filter(|(absent, _)| *absent)
            .map(|(_, unit)| unit)
            .collect::<Vec<_>>()
            .join(" and ");
            Some(format!(
                "input tokens excluded because the table has no {missing} price for {}",
                job.input.selection.model
            ))
        }
        _ => Some(
            "input tokens excluded because text and image input rates differ and the usage has no input_tokens_details"
                .into(),
        ),
    };
    Estimate::Estimated(Breakdown {
        usd: round6(usd),
        quantity: output_tokens,
        unit: PriceUnit::ImageTokenOut,
        qualifier: None,
        rate_usd: Some(price.usd),
        rate_per: Some(price.per),
        partial,
        basis,
        source: price.source.clone(),
    })
}
fn estimate_speech(table: &Table, input: &Effective, usage: Option<&Value>) -> Estimate {
    let prices = for_model(table, &input.selection);
    if prices.is_empty() {
        return Estimate::NotEstimable(missing(table, &input.selection));
    }
    if let Some(price) = prices.iter().find(|p| p.unit == PriceUnit::Character) {
        let Some(text) = &input.text else {
            return Estimate::NotEstimable(format!("no script recorded for {}", input.id));
        };
        let characters = text.chars().count();
        return Estimate::Estimated(Breakdown {
            usd: round6(characters as f64 * price.usd / price.per as f64),
            quantity: characters as f64,
            unit: PriceUnit::Character,
            qualifier: None,
            rate_usd: Some(price.usd),
            rate_per: Some(price.per),
            partial: None,
            basis: format!(
                "{}: {characters} characters × ${}/{}",
                input.id, price.usd, price.per
            ),
            source: price.source.clone(),
        });
    }
    let Some(audio) = prices.iter().find(|p| p.unit == PriceUnit::AudioTokenOut) else {
        return Estimate::NotEstimable(missing(table, &input.selection));
    };
    let Some(output_tokens) = usage_number(
        usage,
        &["output_tokens", "completion_tokens", "candidatesTokenCount"],
    ) else {
        return Estimate::NotEstimable(format!(
            "no provider token usage recorded; {} is priced per token",
            input.selection.model
        ));
    };
    let mut usd = output_tokens * audio.usd / audio.per as f64;
    let mut basis = format!(
        "{output_tokens} audio output tokens × ${}/{}",
        audio.usd, audio.per
    );
    let mut partial = None;
    match (
        prices.iter().find(|p| p.unit == PriceUnit::TextTokenIn),
        usage_number(
            usage,
            &["input_tokens", "prompt_tokens", "promptTokenCount"],
        ),
    ) {
        (Some(text), Some(input_tokens)) => {
            usd += input_tokens * text.usd / text.per as f64;
            basis += &format!(
                " + {input_tokens} text input tokens × ${}/{}",
                text.usd, text.per
            );
        }
        (Some(_), None) => partial = Some("input tokens were not reported".into()),
        (None, _) => {}
    }
    Estimate::Estimated(Breakdown {
        usd: round6(usd),
        quantity: output_tokens,
        unit: PriceUnit::AudioTokenOut,
        qualifier: None,
        rate_usd: Some(audio.usd),
        rate_per: Some(audio.per),
        partial,
        basis,
        source: audio.source.clone(),
    })
}
fn estimate_audio(table: &Table, job: &JobRecord) -> Estimate {
    let prices = for_model(table, &job.input.selection);
    if let Some(price) = prices.iter().find(|p| p.unit == PriceUnit::Song) {
        let count = job.artifacts.len();
        return Estimate::Estimated(Breakdown {
            usd: round6(count as f64 * price.usd / price.per as f64),
            quantity: count as f64,
            unit: PriceUnit::Song,
            qualifier: None,
            rate_usd: Some(price.usd),
            rate_per: Some(price.per),
            partial: None,
            basis: format!("{count} song(s) × ${}", price.usd),
            source: price.source.clone(),
        });
    }
    let Some(price) = prices.iter().find(|p| p.unit == PriceUnit::AudioSecond) else {
        return Estimate::NotEstimable(missing(table, &job.input.selection));
    };
    let mut seconds = 0.0;
    for artifact in &job.artifacts {
        let Some(duration) = artifact.duration_seconds else {
            return Estimate::NotEstimable(format!(
                "artifact {} has no recorded duration",
                artifact.id
            ));
        };
        if !duration.is_finite() || duration < 0.0 {
            return Estimate::NotEstimable(format!(
                "artifact {} has an invalid recorded duration ({duration}); correct duration_seconds in run.yaml before pricing it",
                artifact.id
            ));
        }
        seconds += duration;
    }
    Estimate::Estimated(Breakdown {
        usd: round6(seconds * price.usd / price.per as f64),
        quantity: seconds,
        unit: PriceUnit::AudioSecond,
        qualifier: None,
        rate_usd: Some(price.usd),
        rate_per: Some(price.per),
        partial: None,
        basis: format!("{seconds} decoded seconds × ${}/{}s", price.usd, price.per),
        source: price.source.clone(),
    })
}
fn estimate_text(table: &Table, job: &JobRecord) -> Estimate {
    let prices = for_model(table, &job.input.selection);
    let (Some(input_price), Some(output_price)) = (
        prices.iter().find(|p| p.unit == PriceUnit::TextTokenIn),
        prices.iter().find(|p| p.unit == PriceUnit::TextTokenOut),
    ) else {
        return Estimate::NotEstimable(missing(table, &job.input.selection));
    };
    let (Some(input_tokens), Some(output_tokens)) = (
        usage_number(
            job.usage.as_ref(),
            &["input_tokens", "prompt_tokens", "promptTokenCount"],
        ),
        usage_number(
            job.usage.as_ref(),
            &["output_tokens", "completion_tokens", "candidatesTokenCount"],
        ),
    ) else {
        return Estimate::NotEstimable(format!(
            "no provider token usage recorded; {} is priced per token",
            job.input.selection.model
        ));
    };
    Estimate::Estimated(Breakdown {
        usd: round6(
            input_tokens * input_price.usd / input_price.per as f64
                + output_tokens * output_price.usd / output_price.per as f64,
        ),
        quantity: input_tokens + output_tokens,
        unit: PriceUnit::TextTokenOut,
        qualifier: None,
        rate_usd: Some(output_price.usd),
        rate_per: Some(output_price.per),
        partial: None,
        basis: format!(
            "{input_tokens} input tokens × ${}/{} + {output_tokens} output tokens × ${}/{}",
            input_price.usd, input_price.per, output_price.usd, output_price.per
        ),
        source: output_price.source.clone(),
    })
}
/// Decision order: cached reuse, provider-reported cost, missing artifacts, then the
/// media-specific rule. Nothing here reads or writes run records.
pub fn estimate_job(table: &Table, job: &JobRecord) -> Estimate {
    if !job.input.turns.is_empty() {
        // The engine expands podcast turns, audiobook chapters and speech parts into
        // their own jobs (`parent--child`), each priced on its own row; parents make
        // no provider request.
        return Estimate::AssembledLocally;
    }
    if job.cached {
        return Estimate::Cached;
    }
    if let Some(cost) = job
        .usage
        .as_ref()
        .and_then(|usage| usage.get("cost"))
        .and_then(Value::as_f64)
    {
        return Estimate::ProviderReported(cost);
    }
    if job.artifacts.is_empty() {
        return Estimate::NotEstimable(
            "no completed artifact; a failed or unknown attempt may still have been charged".into(),
        );
    }
    let estimate = match catalog::target(job.input.media) {
        Media::Image => estimate_image(table, job),
        Media::Speech => estimate_speech(table, &job.input, job.usage.as_ref()),
        Media::Music | Media::Sfx => estimate_audio(table, job),
        _ => estimate_text(table, job),
    };
    match estimate {
        // An estimate must be a finite amount; `null` can never count as estimated.
        Estimate::Estimated(breakdown)
            if !breakdown.usd.is_finite() || !breakdown.quantity.is_finite() =>
        {
            Estimate::NotEstimable(format!(
                "recorded quantities for {} are not finite numbers; check the run.yaml artifact fields",
                job.input.id
            ))
        }
        other => other,
    }
}
/// `(cost_estimate, cost_status, cost_note)` for a usage row.
pub fn job_fields(table: &Table, job: &JobRecord) -> (Value, &'static str, String) {
    match estimate_job(table, job) {
        Estimate::Estimated(breakdown) => {
            let mut note = format!(
                "Estimated from the FrameChord price table verified {}; not provider-reported.",
                table.verified_at
            );
            if let Some(partial) = &breakdown.partial {
                note += &format!(" Partial: {partial}.");
            }
            (
                json!({"usd":breakdown.usd,"quantity":quantity_value(breakdown.quantity),"unit":breakdown.unit,"qualifier":breakdown.qualifier,"rate_usd":breakdown.rate_usd,"rate_per":breakdown.rate_per,"partial":breakdown.partial.is_some(),"basis":breakdown.basis,"source":breakdown.source,"table_verified_at":table.verified_at}),
                "estimated",
                note,
            )
        }
        Estimate::ProviderReported(_) => (
            Value::Null,
            "provider_reported",
            "Provider returned a cost in its usage; no estimate applied.".into(),
        ),
        Estimate::Cached => (
            Value::Null,
            "cached_no_charge",
            "Cached reuse made no provider request.".into(),
        ),
        Estimate::AssembledLocally => (
            Value::Null,
            "not_estimated",
            "Local assembly (podcast turns, audiobook chapters, long-speech parts); each provider request is priced as its own job.".into(),
        ),
        Estimate::NotEstimable(reason) => (
            Value::Null,
            "not_estimable",
            format!("{reason}. Unknown cost is not zero."),
        ),
    }
}
