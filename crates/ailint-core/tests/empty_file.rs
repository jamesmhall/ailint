use std::path::PathBuf;

use ailint_core::{lint, Config};

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/structural/empty_file");
    p.push(name);
    p
}

#[test]
fn ail002_fires_on_empty_file() {
    let path = fixture("AGENTS.md");
    let violations = lint(&path, &Config::default()).expect("lint");
    assert_eq!(
        violations.len(),
        1,
        "expected one violation, got {violations:?}"
    );
    assert_eq!(violations[0].rule_id.code, 2);
}

#[test]
fn ail002_silent_on_non_empty_file() {
    let path = fixture("CLAUDE.md");
    let violations = lint(&path, &Config::default()).expect("lint");
    assert_eq!(
        violations.len(),
        0,
        "expected no violations, got {violations:?}"
    );
}
