use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

fn fixtures_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("ailint-core");
    p.push("tests");
    p.push("fixtures");
    p
}

fn bin() -> Command {
    Command::cargo_bin("ailint").expect("ailint binary built")
}

#[test]
fn check_returns_zero_on_clean_fixture() {
    let mut path = fixtures_dir();
    path.push("structural/frontmatter_schema/good/AGENTS.md");
    bin()
        .args(["check", path.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn check_returns_one_on_error_fixture() {
    let mut path = fixtures_dir();
    path.push("security/prompt_injection/bad/AGENTS.md");
    bin()
        .args(["check", path.to_str().unwrap()])
        .assert()
        .code(1)
        .stdout(contains("AIL200"));
}

#[test]
fn list_rules_prints_ail002() {
    bin()
        .arg("list-rules")
        .assert()
        .success()
        .stdout(contains("AIL002"));
}

#[test]
fn init_refuses_overwrite_without_force() {
    let td = TempDir::new().expect("tempdir");
    let existing = td.path().join(".ailint.yaml");
    fs::write(&existing, "rules: {}\n").expect("write existing");
    bin()
        .current_dir(td.path())
        .arg("init")
        .assert()
        .code(1)
        .stderr(contains("already exists"));
    let after = fs::read_to_string(&existing).expect("read");
    assert_eq!(after, "rules: {}\n");
}

#[test]
fn init_overwrites_with_force() {
    let td = TempDir::new().expect("tempdir");
    let existing = td.path().join(".ailint.yaml");
    fs::write(&existing, "rules: {}\n").expect("write existing");
    bin()
        .current_dir(td.path())
        .args(["init", "--force"])
        .assert()
        .success();
    let after = fs::read_to_string(&existing).expect("read");
    assert_ne!(after, "rules: {}\n");
}

#[test]
fn check_config_discovery_walks_up() {
    let td = TempDir::new().expect("tempdir");
    let root = td.path();
    fs::write(
        root.join(".ailint.yaml"),
        "rules:\n  disabled:\n    - AIL200\n",
    )
    .expect("write config");

    let subdir = root.join("nested/fixtures");
    fs::create_dir_all(&subdir).expect("mkdir");
    let fixture = subdir.join("AGENTS.md");
    let mut source = fixtures_dir();
    source.push("security/prompt_injection/bad/AGENTS.md");
    let raw = fs::read_to_string(&source).expect("read fixture");
    fs::write(&fixture, raw).expect("write fixture");

    bin()
        .current_dir(&subdir)
        .args(["check", "AGENTS.md"])
        .assert()
        .success()
        .stdout(predicates::str::contains("AIL200").not());
}
