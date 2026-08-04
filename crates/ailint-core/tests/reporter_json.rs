use std::path::PathBuf;

use ailint_core::reporter::{json::JsonReporter, Reporter};
use ailint_core::rules::{RuleId, Severity, Violation};

fn fixture_violations() -> Vec<Violation> {
    let ail200 = RuleId::new(200, "no-prompt-injection-marker");
    let ail100 = RuleId::new(100, "no-vague-instruction");
    let ail002 = RuleId::new(2, "instructions-file-empty");

    let file_a = PathBuf::from("crates/ailint-core/tests/fixtures/AGENTS.md");
    let file_b = PathBuf::from("crates/ailint-core/tests/fixtures/CLAUDE.md");

    vec![
        Violation::new(
            ail200,
            Severity::Error,
            file_a.clone(),
            "possible prompt-injection marker: '<system>'",
        )
        .at(6, 1),
        Violation::new(
            ail100,
            Severity::Warning,
            file_a,
            "vague instruction: 'be helpful'",
        )
        .at(12, 3),
        Violation::new(
            ail002,
            Severity::Info,
            file_b,
            "file has no non-whitespace content",
        ),
    ]
}

fn frozen_now() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

#[test]
fn json_report_snapshot() {
    let vs = fixture_violations();
    let mut buf: Vec<u8> = Vec::new();
    JsonReporter::with_now(frozen_now)
        .report(&vs, &mut buf)
        .unwrap();
    // Redact the tool version so the snapshot survives version bumps.
    let out = String::from_utf8(buf).unwrap();
    let redacted = out.replacen(
        &format!("\"version\": \"{}\"", env!("CARGO_PKG_VERSION")),
        "\"version\": \"[REDACTED]\"",
        1,
    );
    insta::assert_snapshot!(redacted);
}

#[test]
fn json_report_empty() {
    let mut buf: Vec<u8> = Vec::new();
    JsonReporter::with_now(frozen_now)
        .report(&[], &mut buf)
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(value["summary"]["total"], 0);
    assert_eq!(value["summary"]["file_count"], 0);
    assert_eq!(value["files"].as_array().unwrap().len(), 0);
    assert_eq!(value["violations"].as_array().unwrap().len(), 0);
    assert_eq!(value["schema_version"], "2");
    assert_eq!(value["tool"]["name"], "ailint");
}
