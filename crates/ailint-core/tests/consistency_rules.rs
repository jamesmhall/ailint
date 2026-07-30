use std::path::PathBuf;

use ailint_core::{lint, Config, Violation};

fn fixture(rule: &str, kind: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/consistency");
    p.push(rule);
    p.push(kind);
    p
}

fn count(vs: &[Violation], code: u16) -> usize {
    vs.iter().filter(|v| v.rule_id.code == code).count()
}

#[test]
fn ail300_fires_on_conflicting_files() {
    let path = fixture("conflicting_rules", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    assert!(
        count(&violations, 300) >= 1,
        "expected AIL300 to fire, got {violations:?}"
    );
}

#[test]
fn ail300_silent_on_good_files() {
    let path = fixture("conflicting_rules", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 300),
        0,
        "expected AIL300 silent, got {violations:?}"
    );
}

#[test]
fn ail301_fires_on_duplicate_files() {
    let path = fixture("duplicate_files", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    assert!(
        count(&violations, 301) >= 1,
        "expected AIL301 to fire, got {violations:?}"
    );
}

#[test]
fn ail301_silent_on_good_files() {
    let path = fixture("duplicate_files", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 301),
        0,
        "expected AIL301 silent, got {violations:?}"
    );
}

#[test]
fn ail340_fires_on_orphaned_island() {
    let path = fixture("orphaned_document", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id.code == 340)
        .collect();
    assert_eq!(hits.len(), 1, "expected one AIL340, got {violations:?}");
    assert!(hits[0].file.ends_with("island.md"));
}

#[test]
fn ail340_silent_when_all_reachable() {
    let path = fixture("orphaned_document", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 340),
        0,
        "expected no AIL340, got {violations:?}"
    );
}
