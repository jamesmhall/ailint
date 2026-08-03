use std::path::PathBuf;

use ailint_core::{discovery, lint, Config, FileType};

fn fixture_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/source_comments");
    p
}

fn enabled_config() -> Config {
    let mut cfg = Config::default();
    cfg.sources.enabled = true;
    cfg
}

fn count(vs: &[ailint_core::Violation], code: u16, name: &str) -> usize {
    vs.iter()
        .filter(|v| {
            v.rule_id.code == code
                && v.file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| n == name)
                    .unwrap_or(false)
        })
        .count()
}

#[test]
fn sources_disabled_by_default_skips_rust_files() {
    let files = discovery::walk(&fixture_root(), &Config::default()).expect("walk");
    assert!(
        !files
            .iter()
            .any(|f| matches!(f.file_type, FileType::SourceCode(_))),
        "expected no SourceCode files when sources.enabled is false, got {files:?}"
    );
}

#[test]
fn sources_enabled_discovers_rust_files() {
    let files = discovery::walk(&fixture_root(), &enabled_config()).expect("walk");
    let names: Vec<_> = files
        .iter()
        .filter(|f| matches!(f.file_type, FileType::SourceCode(_)))
        .filter_map(|f| f.path.file_name().and_then(|s| s.to_str()))
        .collect();
    assert!(names.contains(&"bad.rs"), "bad.rs missing: {names:?}");
    assert!(names.contains(&"good.rs"), "good.rs missing: {names:?}");
}

#[test]
fn language_filter_excludes_non_matching() {
    let mut cfg = enabled_config();
    cfg.sources.languages = vec!["python".into()];
    let files = discovery::walk(&fixture_root(), &cfg).expect("walk");
    assert!(
        !files
            .iter()
            .any(|f| matches!(f.file_type, FileType::SourceCode(_))),
        "expected no SourceCode when only python is allowed, got {files:?}"
    );
}

#[test]
fn bloated_doc_comment_triggers_ail106() {
    let violations = lint(&fixture_root(), &enabled_config()).expect("lint");
    assert!(
        count(&violations, 106, "bad.rs") >= 1,
        "expected AIL106 on bad.rs, got {violations:?}"
    );
    assert_eq!(
        count(&violations, 106, "good.rs"),
        0,
        "expected no AIL106 on good.rs, got {violations:?}"
    );
}

#[test]
fn vague_phrase_in_comment_triggers_ail100() {
    let violations = lint(&fixture_root(), &enabled_config()).expect("lint");
    assert!(
        count(&violations, 100, "bad.rs") >= 1,
        "expected AIL100 on bad.rs, got {violations:?}"
    );
    assert_eq!(
        count(&violations, 100, "good.rs"),
        0,
        "expected no AIL100 on good.rs, got {violations:?}"
    );
}

#[test]
fn negative_dominated_comments_trigger_ail104() {
    let violations = lint(&fixture_root(), &enabled_config()).expect("lint");
    assert!(
        count(&violations, 104, "bad.rs") >= 1,
        "expected AIL104 on bad.rs, got {violations:?}"
    );
    assert_eq!(
        count(&violations, 104, "good.rs"),
        0,
        "expected no AIL104 on good.rs, got {violations:?}"
    );
}
