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
