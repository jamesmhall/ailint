use std::path::PathBuf;

use ailint_core::{lint, Config};

fn fixture(sub: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/security");
    p.push(sub);
    p.push("AGENTS.md");
    p
}

#[test]
fn ail200_fires_on_prompt_injection() {
    let path = fixture("prompt_injection/bad");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 200)
        .collect();
    assert!(!hits.is_empty(), "expected AIL200, got {violations:?}");
}

#[test]
fn ail200_silent_on_clean_file() {
    let path = fixture("prompt_injection/good");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 200)
        .collect();
    assert!(hits.is_empty(), "expected no AIL200, got {hits:?}");
}

#[test]
fn ail201_fires_on_unrestricted_grant() {
    let path = fixture("unrestricted_tool/bad");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 201)
        .collect();
    assert!(!hits.is_empty(), "expected AIL201, got {violations:?}");
}

#[test]
fn ail201_silent_on_scoped_tools() {
    let path = fixture("unrestricted_tool/good");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 201)
        .collect();
    assert!(hits.is_empty(), "expected no AIL201, got {hits:?}");
}

#[test]
fn ail202_fires_on_embedded_secret() {
    let path = fixture("sensitive_data/bad");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 202)
        .collect();
    assert!(!hits.is_empty(), "expected AIL202, got {violations:?}");
    for v in &hits {
        assert!(
            !v.message.contains("sk-"),
            "matched secret leaked into message: {}",
            v.message
        );
        assert!(v.snippet.is_none(), "snippet must be None for AIL202");
    }
}

#[test]
fn ail202_silent_on_placeholder_tokens() {
    let path = fixture("sensitive_data/good");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 202)
        .collect();
    assert!(hits.is_empty(), "expected no AIL202, got {hits:?}");
}

#[test]
fn ail203_fires_on_destructive_without_conf() {
    let path = fixture("tool_confirmation/bad");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 203)
        .collect();
    assert!(!hits.is_empty(), "expected AIL203, got {violations:?}");
}

#[test]
fn ail203_silent_on_destructive_with_conf() {
    let path = fixture("tool_confirmation/good");
    let violations = lint(&path, &Config::default()).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 203)
        .collect();
    assert!(hits.is_empty(), "expected no AIL203, got {hits:?}");
}

#[test]
fn ail203_extra_destructive_phrases_fires_on_custom_keyword() {
    // Baseline defaults do not know about `gcloud projects delete`; a team
    // adds it via config and the rule fires.
    let path = fixture("tool_confirmation/custom_destructive");
    let baseline = lint(&path, &Config::default()).expect("lint");
    assert_eq!(
        baseline.iter().filter(|v| v.rule_id.code == 203).count(),
        0,
        "expected AIL203 silent on baseline, got {baseline:?}"
    );

    let mut cfg = Config::default();
    let opts: serde_yaml::Value =
        serde_yaml::from_str(r#"extra_destructive_phrases: ["gcloud projects delete"]"#).unwrap();
    cfg.rules
        .options
        .insert("tool-confirmation-required".to_string(), opts);
    let violations = lint(&path, &cfg).expect("lint");
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 203)
        .collect();
    assert!(
        !hits.is_empty(),
        "expected configured AIL203 to fire, got {violations:?}"
    );
    assert_eq!(hits[0].detail.as_deref(), Some("gcloud projects delete"));
}

#[test]
fn ail202_empty_allowlist_markers_removes_defaults() {
    // The "good" fixture is silent because its secrets are guarded by
    // default placeholder markers (EXAMPLE, placeholder). Overriding
    // allowlist_markers to an empty list should strip those defaults and
    // let the rule fire.
    let path = fixture("sensitive_data/good");
    let baseline = lint(&path, &Config::default()).expect("lint");
    assert_eq!(
        baseline.iter().filter(|v| v.rule_id.code == 202).count(),
        0,
        "sanity: baseline should be silent"
    );

    let mut cfg = Config::default();
    let opts: serde_yaml::Value = serde_yaml::from_str("allowlist_markers: []").unwrap();
    cfg.rules
        .options
        .insert("no-sensitive-data-in-instructions".to_string(), opts);
    let violations = lint(&path, &cfg).expect("lint");
    assert!(
        violations.iter().any(|v| v.rule_id.code == 202),
        "expected AIL202 to fire with empty allowlist_markers, got {violations:?}"
    );
}
