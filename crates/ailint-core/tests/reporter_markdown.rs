use ailint_core::reporter::markdown::MarkdownReporter;
use ailint_core::reporter::Reporter;
use ailint_core::rules::{RuleId, Severity, Violation};
use std::path::PathBuf;

fn fixture_violations() -> Vec<Violation> {
    let sec_file =
        PathBuf::from("crates/ailint-core/tests/fixtures/security/prompt_injection/bad/AGENTS.md");
    let sem_file =
        PathBuf::from("crates/ailint-core/tests/fixtures/semantic/duplicate_rules/bad/AGENTS.md");
    vec![
        Violation::new(
            RuleId::new(200, "no-prompt-injection-marker"),
            Severity::Error,
            sec_file.clone(),
            "possible prompt-injection marker: 'ignore previous instructions'",
        )
        .at(6, 1),
        Violation::new(
            RuleId::new(103, "no-duplicate-rules"),
            Severity::Info,
            sem_file,
            "duplicate of rule at line 3",
        )
        .at(5, 1),
        Violation::new(
            RuleId::new(100, "no-vague-instruction"),
            Severity::Warning,
            sec_file,
            "vague phrase 'as appropriate' | consider being specific",
        )
        .at(2, 1),
    ]
}

#[test]
fn markdown_report_snapshot() {
    let vs = fixture_violations();
    let mut buf: Vec<u8> = Vec::new();
    MarkdownReporter::with_version("X.Y.Z")
        .report(&vs, &mut buf)
        .expect("report");
    insta::assert_snapshot!(String::from_utf8(buf).expect("utf8"));
}

#[test]
fn markdown_report_empty() {
    let mut buf: Vec<u8> = Vec::new();
    MarkdownReporter::default()
        .report(&[], &mut buf)
        .expect("report");
    let s = String::from_utf8(buf).expect("utf8");
    assert!(s.contains("_No violations._"));
    assert!(!s.contains("| Rule "));
}
