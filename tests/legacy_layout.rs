use assert_cmd::cargo::cargo_bin_cmd;
use framechord::{contracts::*, engine, project, storage};
use serde_json::Value;
use std::{fs, path::Path};

fn legacy_project(root: &Path) {
    fs::create_dir_all(root.join("mediaagent")).unwrap();
    storage::save(
        &root.join("mediaagent/config.yaml"),
        &framechord::setup::recommended(),
    )
    .unwrap();
    project::add_persona(root, "original").unwrap();
}

#[test]
fn renamed_init_reuses_legacy_data_without_rewriting_authored_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    legacy_project(root);
    let config = fs::read(root.join("mediaagent/config.yaml")).unwrap();
    let persona = root.join("mediaagent/characters/original/persona.md");
    fs::write(&persona, "An exact, user-authored identity.\r\n").unwrap();
    let old_skill = root.join(".agents/skills/mediaagent/SKILL.md");
    fs::create_dir_all(old_skill.parent().unwrap()).unwrap();
    fs::write(&old_skill, "User-modified legacy skill").unwrap();
    fs::write(root.join(".env"), framechord::credentials::ENV_TEMPLATE).unwrap();
    for _ in 0..2 {
        let out = cargo_bin_cmd!("framechord")
            .current_dir(root)
            .args(["--json", "--non-interactive", "init"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let result: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(result["data"]["legacy_layout"], true);
        assert_eq!(
            result["data"]["data_directory"],
            root.join("mediaagent")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
    assert!(!root.join("framechord").exists());
    assert_eq!(
        fs::read(root.join("mediaagent/config.yaml")).unwrap(),
        config
    );
    assert_eq!(
        fs::read_to_string(persona).unwrap(),
        "An exact, user-authored identity.\r\n"
    );
    assert_eq!(
        fs::read_to_string(old_skill).unwrap(),
        "User-modified legacy skill"
    );
    for agent in [".agents", ".claude"] {
        assert!(
            root.join(agent)
                .join("skills/framechord/SKILL.md")
                .is_file()
        );
    }
}

#[test]
fn legacy_run_receipts_remain_readable_and_are_not_rewritten() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    legacy_project(root);
    let input = Effective {
        assembly: None,
        id: "historical".into(),
        media: Media::Image,
        selection: Selection {
            provider: Provider::Google,
            model: "gemini-nano-banana-2.1".into(),
        },
        prompt: "An unchanged historical request".into(),
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
    // Local durable submission only; no worker or provider calls.
    let run = engine::submit(
        root,
        vec![input],
        Limits::default(),
        engine::Submit::default(),
    )
    .unwrap();
    let receipt = root.join("mediaagent/runs").join(&run.id).join("run.yaml");
    let original = fs::read(&receipt).unwrap();
    let output = cargo_bin_cmd!("framechord")
        .current_dir(root)
        .args(["--json", "runs", "list"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(&run.id));
    assert_eq!(fs::read(receipt).unwrap(), original);
    assert!(!root.join("framechord").exists());
}

#[test]
fn ambiguous_layout_is_rejected_before_modifying_either_project() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    legacy_project(root);
    fs::create_dir(root.join("framechord")).unwrap();
    let old = fs::read(root.join("mediaagent/config.yaml")).unwrap();
    let out = cargo_bin_cmd!("framechord")
        .current_dir(root)
        .args(["--json", "--non-interactive", "init"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["errors"][0]["code"], "project_layout_conflict");
    assert_eq!(fs::read(root.join("mediaagent/config.yaml")).unwrap(), old);
    assert!(!root.join("framechord/config.yaml").exists());
    assert!(!root.join(".env").exists());
    assert!(!root.join(".agents").exists());
}

#[test]
fn explicit_init_destination_takes_precedence_over_ambiguous_cwd() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    legacy_project(root);
    fs::create_dir(root.join("framechord")).unwrap();
    let destination = root.join("separate-project");
    cargo_bin_cmd!("framechord")
        .current_dir(root)
        .args(["--json", "--non-interactive", "init", "--preset", "empty"])
        .arg(&destination)
        .assert()
        .success();
    assert!(destination.join("framechord/config.yaml").is_file());
    assert!(!root.join("framechord/config.yaml").exists());
}
