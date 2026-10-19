use framechord::{catalog, contracts::*, engine, maintenance, pricing, project, storage};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

const NANO: &str = "gemini-nano-banana-2.1";

fn root() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    project::init(t.path(), &["none".into()]).unwrap();
    t
}
fn input(id: &str, media: Media, provider: Provider, model: &str, text: Option<&str>) -> Effective {
    Effective {
        assembly: None,
        id: id.into(),
        media,
        selection: Selection {
            provider,
            model: model.into(),
        },
        prompt: "fixture".into(),
        text: text.map(Into::into),
        locale: None,
        settings: Settings::default(),
        character_hash: None,
        references: vec![],
        depends_on: vec![],
        output: None,
        turns: vec![],
        audiobook: None,
    }
}
fn artifact(
    job: &str,
    format: &str,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<f64>,
) -> Artifact {
    Artifact {
        id: format!("{job}-artifact"),
        job_id: job.into(),
        path: PathBuf::from(format!("/fixture/{job}.{format}")),
        sha256: "0".repeat(64),
        bytes: 1,
        format: format.into(),
        quality: Quality::Unchecked,
        parent: None,
        duration_seconds: duration,
        width,
        height,
    }
}
fn record(
    input: Effective,
    state: State,
    artifacts: Vec<Artifact>,
    usage: Option<Value>,
    cached: bool,
) -> JobRecord {
    JobRecord {
        input,
        fingerprint: "fingerprint".into(),
        state,
        attempts: 1,
        artifacts,
        error: None,
        request_id: None,
        usage,
        cached,
        retry_not_before: None,
        duration_accounted: false,
    }
}
fn save_run(root: &Path, id: &str, jobs: Vec<JobRecord>) -> PathBuf {
    let run = Run {
        schema_version: 2,
        id: id.into(),
        budget_group: id.into(),
        request_id: None,
        state: State::Succeeded,
        jobs,
        limits: Limits::default(),
        fresh: false,
        overwrite: false,
        parent_artifact: None,
        submission_fingerprint: "submission".into(),
    };
    let path = engine::run_path(root, id).unwrap().join("run.yaml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    storage::save(&path, &run).unwrap();
    path
}
fn job_row<'a>(report: &'a Value, job: &str) -> &'a Value {
    report["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["job"] == job)
        .unwrap_or_else(|| panic!("no row for {job}: {report}"))
}
fn iso_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .chars()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7) || c.is_ascii_digit())
}

#[test]
fn bundled_price_table_is_valid_dated_and_matches_the_catalog() {
    let table = pricing::bundled();
    pricing::validate(&table).unwrap();
    assert_eq!(table.schema_version, 1);
    assert_eq!(table.currency, "USD");
    assert!(iso_date(&table.verified_at), "{}", table.verified_at);
    assert!(table.entries.len() >= 10);
    let models = catalog::models();
    for entry in &table.entries {
        assert!(entry.source.starts_with("https://"), "{}", entry.model);
        assert!(entry.usd > 0.0 && entry.per >= 1, "{}", entry.model);
        if let Some(until) = &entry.valid_until {
            assert!(iso_date(until), "{}", entry.model);
        }
        assert!(
            models.iter().any(|m| m.provider == entry.provider
                && m.model == entry.model
                && m.current
                && m.supported),
            "{:?} {} is not a current supported catalog model",
            entry.provider,
            entry.model
        );
    }
    assert!(
        table
            .entries
            .iter()
            .any(|e| e.provider == Provider::Google && e.model == NANO)
    );
    assert!(
        !table.entries.iter().any(|e| e.provider == Provider::Fish),
        "Fish publishes credits, not API list prices"
    );
}

#[test]
fn image_tier_uses_pixel_count_not_longest_side() {
    assert_eq!(pricing::image_tier(1024, 1024), "1K");
    assert_eq!(pricing::image_tier(1344, 768), "1K");
    assert_eq!(pricing::image_tier(2688, 1536), "2K");
    assert_eq!(pricing::image_tier(4096, 4096), "4K");
    assert_eq!(pricing::image_tier(512, 512), "0.5K");
}

#[test]
fn usage_estimates_google_images_from_artifact_resolution() {
    let r = root();
    let job = record(
        input("cover", Media::Image, Provider::Google, NANO, None),
        State::Succeeded,
        vec![artifact("cover", "png", Some(1024), Some(1024), None)],
        Some(json!({"promptTokenCount": 12, "candidatesTokenCount": 1290})),
        false,
    );
    let path = save_run(r.path(), "run-image", vec![job]);
    let before = fs::read(&path).unwrap();
    let report = maintenance::usage(r.path()).unwrap();
    let row = job_row(&report, "cover");
    assert_eq!(row["cost_status"], "estimated");
    assert_eq!(row["cost_estimate"]["usd"], 0.0336);
    assert_eq!(row["cost_estimate"]["qualifier"], "1K");
    assert_eq!(row["cost_estimate"]["quantity"], 1);
    assert_eq!(row["cost_estimate"]["unit"], "image");
    assert_eq!(row["cost_estimate"]["partial"], false);
    assert!(
        row["cost_estimate"]["source"]
            .as_str()
            .unwrap()
            .starts_with("https://")
    );
    assert!(
        row["cost_note"]
            .as_str()
            .unwrap()
            .contains("not provider-reported")
    );
    assert_eq!(row["provider_reported_usage"]["candidatesTokenCount"], 1290);
    assert_eq!(report["estimated_total_usd"], 0.0336);
    assert_eq!(report["estimate_summary"]["estimated_jobs"], 1);
    assert_eq!(
        report["estimate_summary"]["table_verified_at"],
        pricing::bundled().verified_at
    );
    assert!(
        report["estimate_summary"]["disclaimer"]
            .as_str()
            .unwrap()
            .contains("not provider invoices")
    );
    assert!(report["currency_total"].is_null());
    assert_eq!(fs::read(&path).unwrap(), before, "usage is read-only");
}

#[test]
fn usage_marks_cached_jobs_as_no_charge_and_excludes_them_from_totals() {
    let r = root();
    let cached = record(
        input("reuse", Media::Image, Provider::Google, NANO, None),
        State::Succeeded,
        vec![artifact("reuse", "png", Some(1024), Some(1024), None)],
        None,
        true,
    );
    let fresh = record(
        input("fresh", Media::Image, Provider::Google, NANO, None),
        State::Succeeded,
        vec![artifact("fresh", "png", Some(2048), Some(2048), None)],
        None,
        false,
    );
    save_run(r.path(), "run-cache", vec![cached, fresh]);
    let report = maintenance::usage(r.path()).unwrap();
    assert_eq!(job_row(&report, "reuse")["cost_status"], "cached_no_charge");
    assert!(job_row(&report, "reuse")["cost_estimate"].is_null());
    assert_eq!(
        job_row(&report, "fresh")["cost_estimate"]["qualifier"],
        "2K"
    );
    assert_eq!(report["estimated_total_usd"], 0.0504);
    assert_eq!(report["estimate_summary"]["cached_jobs"], 1);
    assert_eq!(report["estimate_summary"]["estimated_jobs"], 1);
}

#[test]
fn usage_reports_not_estimable_reasons() {
    let r = root();
    let fish = record(
        input(
            "fish-line",
            Media::Speech,
            Provider::Fish,
            "s2.1-pro",
            Some("Hello there"),
        ),
        State::Succeeded,
        vec![artifact("fish-line", "mp3", None, None, Some(1.5))],
        None,
        false,
    );
    let sfx = record(
        input(
            "bell",
            Media::Sfx,
            Provider::Elevenlabs,
            "eleven_text_to_sound_v2",
            None,
        ),
        State::Succeeded,
        vec![artifact("bell", "mp3", None, None, None)],
        None,
        false,
    );
    let failed = record(
        input("lost", Media::Image, Provider::Google, NANO, None),
        State::Failed,
        vec![],
        None,
        false,
    );
    save_run(r.path(), "run-unknown", vec![fish, sfx, failed]);
    let report = maintenance::usage(r.path()).unwrap();
    for (job, needle) in [
        ("fish-line", "s2.1-pro"),
        ("bell", "duration"),
        ("lost", "charged"),
    ] {
        let row = job_row(&report, job);
        assert_eq!(row["cost_status"], "not_estimable", "{job}");
        assert!(row["cost_estimate"].is_null(), "{job}");
        assert!(
            row["cost_note"].as_str().unwrap().contains(needle),
            "{job}: {}",
            row["cost_note"]
        );
    }
    assert!(report["estimated_total_usd"].is_null());
    assert_eq!(report["estimate_summary"]["not_estimable_jobs"], 3);
    assert_eq!(report["estimate_summary"]["estimated_jobs"], 0);
}

#[test]
fn usage_keeps_provider_reported_cost_separate_from_estimates() {
    let r = root();
    let routed = record(
        input(
            "routed",
            Media::Image,
            Provider::Openrouter,
            "openai/gpt-image-2.5-flare",
            None,
        ),
        State::Succeeded,
        vec![artifact("routed", "png", Some(1024), Some(1024), None)],
        Some(json!({"total_tokens": 3, "cost": 0.02})),
        false,
    );
    save_run(r.path(), "run-routed", vec![routed]);
    let report = maintenance::usage(r.path()).unwrap();
    let row = job_row(&report, "routed");
    assert_eq!(row["cost_status"], "provider_reported");
    assert!(row["cost_estimate"].is_null());
    assert_eq!(row["provider_reported_usage"]["cost"], 0.02);
    assert!(report["estimated_total_usd"].is_null());
    assert_eq!(report["estimate_summary"]["provider_reported_jobs"], 1);
    assert!(report["currency_total"].is_null());
}

#[test]
fn podcast_turns_are_priced_once_and_the_assembly_job_is_not_estimated() {
    // The engine records a podcast as separate turn jobs plus a parent that only
    // assembles clips locally; the parent must never re-price its turns.
    let r = root();
    let host = "a".repeat(1000);
    let guest = "b".repeat(500);
    let host_turn = input(
        "episode--host",
        Media::Speech,
        Provider::Elevenlabs,
        "eleven_v3",
        Some(&host),
    );
    let guest_turn = input(
        "episode--guest",
        Media::Speech,
        Provider::Elevenlabs,
        "eleven_flash_v2_5",
        Some(&guest),
    );
    let mut episode = input(
        "episode",
        Media::Podcast,
        Provider::Elevenlabs,
        "eleven_v3",
        None,
    );
    episode.turns = vec![host_turn.clone(), guest_turn.clone()];
    let jobs = vec![
        record(
            host_turn,
            State::Succeeded,
            vec![artifact("episode--host", "mp3", None, None, Some(60.0))],
            None,
            false,
        ),
        record(
            guest_turn,
            State::Succeeded,
            vec![artifact("episode--guest", "mp3", None, None, Some(30.0))],
            None,
            false,
        ),
        record(
            episode,
            State::Succeeded,
            vec![artifact("episode", "wav", None, None, Some(90.0))],
            None,
            false,
        ),
    ];
    save_run(r.path(), "run-podcast", jobs);
    let report = maintenance::usage(r.path()).unwrap();
    assert_eq!(
        job_row(&report, "episode--host")["cost_estimate"]["usd"],
        0.08
    );
    assert_eq!(
        job_row(&report, "episode--guest")["cost_estimate"]["usd"],
        0.02
    );
    let parent = job_row(&report, "episode");
    assert_eq!(parent["cost_status"], "not_estimated");
    assert!(parent["cost_estimate"].is_null());
    assert!(
        parent["cost_note"].as_str().unwrap().contains("assembl"),
        "{}",
        parent["cost_note"]
    );
    assert_eq!(report["estimated_total_usd"], 0.1);
    assert_eq!(report["estimate_summary"]["estimated_jobs"], 2);
    assert_eq!(report["estimate_summary"]["not_estimable_jobs"], 0);
    assert_eq!(report["estimate_summary"]["not_estimated_jobs"], 1);
}

#[test]
fn openai_image_token_estimate_is_flagged_partial() {
    let r = root();
    let priced = record(
        input(
            "leaf",
            Media::Image,
            Provider::Openai,
            "gpt-image-2.5-flare",
            None,
        ),
        State::Succeeded,
        vec![artifact("leaf", "png", Some(1024), Some(1024), None)],
        Some(json!({"input_tokens": 50, "output_tokens": 1056, "total_tokens": 1106})),
        false,
    );
    let unknown = record(
        input(
            "leaf-2",
            Media::Image,
            Provider::Openai,
            "gpt-image-2.5-flare",
            None,
        ),
        State::Succeeded,
        vec![artifact("leaf-2", "png", Some(1024), Some(1024), None)],
        None,
        false,
    );
    save_run(r.path(), "run-openai", vec![priced, unknown]);
    let report = maintenance::usage(r.path()).unwrap();
    let row = job_row(&report, "leaf");
    assert_eq!(row["cost_status"], "estimated");
    assert_eq!(row["cost_estimate"]["partial"], true);
    assert_eq!(row["cost_estimate"]["usd"], 0.03168);
    assert_eq!(row["cost_estimate"]["unit"], "image_token_out");
    assert!(
        row["cost_note"].as_str().unwrap().contains("input tokens"),
        "{}",
        row["cost_note"]
    );
    let row = job_row(&report, "leaf-2");
    assert_eq!(row["cost_status"], "not_estimable");
    assert!(row["cost_note"].as_str().unwrap().contains("token"));
    assert_eq!(report["estimate_summary"]["partial_jobs"], 1);
    assert_eq!(report["estimated_total_usd"], 0.03168);
}

#[test]
fn explicit_price_table_overrides_bundled_rates() {
    let r = root();
    let job = record(
        input("cover", Media::Image, Provider::Google, NANO, None),
        State::Succeeded,
        vec![artifact("cover", "png", Some(1024), Some(1024), None)],
        None,
        false,
    );
    save_run(r.path(), "run-override", vec![job]);
    let custom = r.path().join("custom-prices.json");
    fs::write(
        &custom,
        json!({"schema_version":1,"currency":"USD","verified_at":"2026-10-15","entries":[{"provider":"google","model":NANO,"media":"image","unit":"image","qualifier":"1K","usd":0.05,"per":1,"source":"https://example.test/pricing","valid_until":null,"note":null}]}).to_string(),
    )
    .unwrap();
    let table = pricing::load(&custom).unwrap();
    let report = maintenance::usage_with(r.path(), &table, Some(&custom)).unwrap();
    assert_eq!(job_row(&report, "cover")["cost_estimate"]["usd"], 0.05);
    assert_eq!(
        report["estimate_summary"]["table_source"],
        json!(custom.display().to_string())
    );
    let bad = r.path().join("bad-prices.json");
    fs::write(
        &bad,
        json!({"schema_version":1,"currency":"EUR","verified_at":"2026-10-15","entries":[]})
            .to_string(),
    )
    .unwrap();
    let error = pricing::load(&bad).unwrap_err();
    assert_eq!(error.code, "invalid_input");
    let duplicate = r.path().join("duplicate-prices.json");
    fs::write(
        &duplicate,
        json!({"schema_version":1,"currency":"USD","verified_at":"2026-10-15","entries":[
            {"provider":"google","model":NANO,"media":"image","unit":"image","qualifier":"1K","usd":0.05,"per":1,"source":"https://example.test/pricing","valid_until":null,"note":null},
            {"provider":"google","model":NANO,"media":"image","unit":"image","qualifier":"1K","usd":0.06,"per":1,"source":"https://example.test/pricing","valid_until":null,"note":null}
        ]}).to_string(),
    )
    .unwrap();
    assert!(pricing::load(&duplicate).is_err());
}

#[test]
fn price_table_dates_must_be_real_calendar_days() {
    let mut table = pricing::bundled();
    for bad in [
        "2026-02-30",
        "2023-02-29",
        "2100-02-29",
        "2026-04-31",
        "2026-13-01",
        "2026-00-10",
        "2026-1-01",
    ] {
        table.verified_at = bad.into();
        let error = pricing::validate(&table).unwrap_err();
        assert_eq!(error.code, "invalid_input", "{bad}");
        assert!(
            error.message.contains("verified_at") && error.message.contains("YYYY-MM-DD"),
            "{bad}: {}",
            error.message
        );
    }
    for good in ["2024-02-29", "2000-02-29", "2026-02-28", "2026-12-31"] {
        table.verified_at = good.into();
        pricing::validate(&table).unwrap_or_else(|e| panic!("{good}: {e}"));
    }
    table.verified_at = "2026-10-15".into();
    table.entries[0].valid_until = Some("2027-02-29".into());
    let error = pricing::validate(&table).unwrap_err();
    assert!(
        error.message.contains("valid_until") && error.message.contains("YYYY-MM-DD"),
        "{}",
        error.message
    );
}

#[test]
fn non_finite_durations_are_not_estimable_instead_of_null_estimates() {
    let r = root();
    let table = pricing::bundled();
    for (id, duration) in [
        ("nan", f64::NAN),
        ("inf", f64::INFINITY),
        ("neg-inf", f64::NEG_INFINITY),
        ("negative", -3.0),
    ] {
        let job = record(
            input(
                id,
                Media::Sfx,
                Provider::Elevenlabs,
                "eleven_text_to_sound_v2",
                None,
            ),
            State::Succeeded,
            vec![artifact(id, "mp3", None, None, Some(duration))],
            None,
            false,
        );
        assert!(
            matches!(
                pricing::estimate_job(&table, &job),
                pricing::Estimate::NotEstimable(_)
            ),
            "{id}"
        );
        let (estimate, status, note) = pricing::job_fields(&table, &job);
        assert_eq!(status, "not_estimable", "{id}");
        assert!(estimate.is_null(), "{id}: {estimate}");
        assert!(
            note.contains("duration") && note.contains("run.yaml"),
            "{id}: {note}"
        );
    }
    // A hand-edited run.yaml carrying `.inf` is rejected as invalid input when runs are
    // loaded, naming the file, so it can never reach the estimator as a number.
    let job = record(
        input(
            "edited",
            Media::Sfx,
            Provider::Elevenlabs,
            "eleven_text_to_sound_v2",
            None,
        ),
        State::Succeeded,
        vec![artifact("edited", "mp3", None, None, Some(7.25))],
        None,
        false,
    );
    let path = save_run(r.path(), "run-edited", vec![job]);
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("7.25"), "{text}");
    fs::write(&path, text.replace("7.25", ".inf")).unwrap();
    let error = maintenance::usage(r.path())
        .expect_err("a non-finite duration in run.yaml is rejected, not estimated");
    assert_eq!(error.code, "invalid_input");
    assert!(error.message.contains("run-edited"), "{}", error.message);
}

#[test]
fn openai_input_token_details_are_priced_when_the_table_has_both_input_rates() {
    let r = root();
    let model = "gpt-image-2.5-flare";
    let usage = json!({
        "input_tokens": 50,
        "input_tokens_details": {"text_tokens": 20, "image_tokens": 30},
        "output_tokens": 1056,
        "total_tokens": 1106
    });
    let detailed = record(
        input("leaf", Media::Image, Provider::Openai, model, None),
        State::Succeeded,
        vec![artifact("leaf", "png", Some(1024), Some(1024), None)],
        Some(usage.clone()),
        false,
    );
    let undetailed = record(
        input("leaf-plain", Media::Image, Provider::Openai, model, None),
        State::Succeeded,
        vec![artifact("leaf-plain", "png", Some(1024), Some(1024), None)],
        Some(json!({"input_tokens": 50, "output_tokens": 1056, "total_tokens": 1106})),
        false,
    );
    save_run(r.path(), "run-details", vec![detailed, undetailed]);

    // The bundled table has no text_token_in entry for OpenAI images, so the split
    // input counts alone cannot clear the partial flag; the note says which rate is missing.
    let report = maintenance::usage(r.path()).unwrap();
    let row = job_row(&report, "leaf");
    assert_eq!(row["cost_status"], "estimated");
    assert_eq!(row["cost_estimate"]["partial"], true);
    assert_eq!(row["cost_estimate"]["usd"], 0.03168);
    assert!(
        row["cost_note"].as_str().unwrap().contains("text_token_in"),
        "{}",
        row["cost_note"]
    );

    let price = |unit: &str, usd: f64| json!({"provider":"openai","model":model,"media":"image","unit":unit,"qualifier":null,"usd":usd,"per":1_000_000,"source":"https://example.test/pricing","valid_until":null,"note":null});
    let custom = r.path().join("openai-prices.json");
    fs::write(
        &custom,
        json!({"schema_version":1,"currency":"USD","verified_at":"2026-10-15","entries":[
            price("text_token_in", 5.0),
            price("image_token_in", 8.0),
            price("image_token_out", 30.0),
        ]})
        .to_string(),
    )
    .unwrap();
    let table = pricing::load(&custom).unwrap();
    let report = maintenance::usage_with(r.path(), &table, Some(&custom)).unwrap();
    let row = job_row(&report, "leaf");
    assert_eq!(row["cost_status"], "estimated");
    assert_eq!(row["cost_estimate"]["partial"], false, "{row}");
    // 1056 × 30/1M + 20 × 5/1M + 30 × 8/1M
    assert_eq!(row["cost_estimate"]["usd"], 0.03202);
    assert_eq!(row["cost_estimate"]["unit"], "image_token_out");
    let basis = row["cost_estimate"]["basis"].as_str().unwrap();
    assert!(
        basis.contains("20 text input tokens") && basis.contains("30 image input tokens"),
        "{basis}"
    );
    assert!(
        !row["cost_note"].as_str().unwrap().contains("Partial"),
        "{}",
        row["cost_note"]
    );
    let row = job_row(&report, "leaf-plain");
    assert_eq!(row["cost_estimate"]["partial"], true);
    assert_eq!(row["cost_estimate"]["usd"], 0.03168);
    assert!(
        row["cost_note"]
            .as_str()
            .unwrap()
            .contains("input_tokens_details"),
        "{}",
        row["cost_note"]
    );
    assert_eq!(report["estimate_summary"]["partial_jobs"], 1);
    assert_eq!(report["estimated_total_usd"], 0.0637);
}
