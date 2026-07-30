#![cfg(feature = "llm")]

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("ailint").expect("ailint binary built")
}

#[test]
fn check_without_provider_configured_ignores_ail900() {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(
        dir.path().join("CLAUDE.md"),
        "# Project Guidance\n\nWrite tests for every new function.\n",
    )
    .expect("write");
    bin()
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("GOOGLE_API_KEY")
        .env_remove("GEMINI_API_KEY")
        .env_remove("AILINT_LLM_API_KEY")
        .env_remove("AILINT_CONFIG")
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn list_rules_prints_ail900_when_llm_feature_enabled() {
    bin()
        .arg("list-rules")
        .assert()
        .success()
        .stdout(contains("AIL900"));
}
