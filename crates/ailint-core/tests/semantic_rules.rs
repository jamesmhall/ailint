use std::path::PathBuf;

use ailint_core::{lint, Config};

fn fixture(rule: &str, kind: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic");
    p.push(rule);
    p.push(kind);
    p.push("AGENTS.md");
    p
}

fn count(vs: &[ailint_core::Violation], code: u16) -> usize {
    vs.iter().filter(|v| v.rule_id.code == code).count()
}

#[test]
fn ail100_fires_on_vague_phrase() {
    let path = fixture("vague_instruction", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    assert!(
        count(&violations, 100) >= 1,
        "expected AIL100 to fire, got {violations:?}"
    );
}

#[test]
fn ail100_silent_on_concrete_list() {
    let path = fixture("vague_instruction", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 100),
        0,
        "expected AIL100 silent, got {violations:?}"
    );
}

#[test]
fn ail101_fires_when_examples_section_lacks_code_or_eg() {
    let path = fixture("missing_examples", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    assert!(
        count(&violations, 101) >= 1,
        "expected AIL101 to fire, got {violations:?}"
    );
}

#[test]
fn ail101_silent_when_section_has_code_block() {
    let path = fixture("missing_examples", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 101),
        0,
        "expected AIL101 silent, got {violations:?}"
    );
}

#[test]
fn ail102_fires_on_long_list_item() {
    let path = fixture("excessive_length", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    assert!(
        count(&violations, 102) >= 1,
        "expected AIL102 to fire, got {violations:?}"
    );
}

#[test]
fn ail102_silent_on_short_list_items() {
    let path = fixture("excessive_length", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 102),
        0,
        "expected AIL102 silent, got {violations:?}"
    );
}

#[test]
fn ail103_fires_on_duplicate_list_items() {
    let path = fixture("duplicate_rules", "bad");
    let violations = lint(&path, &Config::default()).unwrap();
    assert!(
        count(&violations, 103) >= 1,
        "expected AIL103 to fire, got {violations:?}"
    );
}

#[test]
fn ail103_silent_on_distinct_list_items() {
    let path = fixture("duplicate_rules", "good");
    let violations = lint(&path, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 103),
        0,
        "expected AIL103 silent, got {violations:?}"
    );
}

#[test]
fn ail104_fires_on_negative_overload() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic/negative_constraint_overload/bad/AGENTS.md");
    let violations = lint(&p, &Config::default()).unwrap();
    assert!(
        count(&violations, 104) >= 1,
        "expected AIL104 to fire, got {violations:?}"
    );
}

#[test]
fn ail104_silent_on_positive_guidance() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic/negative_constraint_overload/good/AGENTS.md");
    let violations = lint(&p, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 104),
        0,
        "expected AIL104 silent, got {violations:?}"
    );
}

#[test]
fn ail105_fires_on_no_tags() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic/vendor_optimization_syntax/bad/CLAUDE.md");
    let violations = lint(&p, &Config::default()).unwrap();
    assert!(
        count(&violations, 105) >= 1,
        "expected AIL105 to fire, got {violations:?}"
    );
}

#[test]
fn ail105_silent_with_tags() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic/vendor_optimization_syntax/good/CLAUDE.md");
    let violations = lint(&p, &Config::default()).unwrap();
    assert_eq!(
        count(&violations, 105),
        0,
        "expected AIL105 silent, got {violations:?}"
    );
}

#[test]
fn ail104_extra_prefixes_fires_on_positive_fixture() {
    // The "good" fixture is dominated by "do"/"build"/"test"/"complete"/
    // "validate"/"you must" — configuring these as negative prefixes should
    // flip it to firing.
    let mut cfg = Config::default();
    let opts: serde_yaml::Value = serde_yaml::from_str(
        r#"
extra_prefixes: ["you must", "do ", "build", "test", "complete", "validate"]
"#,
    )
    .unwrap();
    cfg.rules
        .options
        .insert("negative-constraint-overload".to_string(), opts);
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic/negative_constraint_overload/good/AGENTS.md");
    let violations = lint(&p, &cfg).unwrap();
    assert!(
        count(&violations, 104) >= 1,
        "expected configured AIL104 to fire, got {violations:?}"
    );
}

#[test]
fn ail104_min_list_items_suppresses_firing() {
    let mut cfg = Config::default();
    let opts: serde_yaml::Value = serde_yaml::from_str("min_list_items: 100").unwrap();
    cfg.rules
        .options
        .insert("negative-constraint-overload".to_string(), opts);
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/semantic/negative_constraint_overload/bad/AGENTS.md");
    let violations = lint(&p, &cfg).unwrap();
    assert_eq!(
        count(&violations, 104),
        0,
        "expected AIL104 suppressed by min_list_items, got {violations:?}"
    );
}
