use std::path::PathBuf;

use ailint_core::{lint, Config};

fn fixture(rule_dir: &str, kind: &str, name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/structural");
    p.push(rule_dir);
    p.push(kind);
    p.push(name);
    p
}

#[test]
fn ail001_fires_on_invalid_frontmatter() {
    let path = fixture("frontmatter_schema", "bad", "AGENTS.md");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 1).collect();
    assert_eq!(hits.len(), 1, "expected one AIL001, got {violations:?}");
    assert!(hits[0].message.starts_with("invalid YAML frontmatter"));
}

#[test]
fn ail001_silent_on_valid_frontmatter() {
    let path = fixture("frontmatter_schema", "good", "AGENTS.md");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 1).collect();
    assert!(hits.is_empty(), "expected no AIL001, got {violations:?}");
}

fn config_with_required_setup() -> Config {
    let mut cfg = Config::default();
    let opts: serde_yaml::Value = serde_yaml::from_str("required: [\"Setup\"]").unwrap();
    cfg.rules
        .options
        .insert("missing-required-section".to_string(), opts);
    cfg
}

#[test]
fn ail003_fires_when_required_section_missing() {
    let path = fixture("required_section", "bad", "AGENTS.md");
    let cfg = config_with_required_setup();
    let violations = lint(&path, &cfg).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 3).collect();
    assert_eq!(hits.len(), 1, "expected one AIL003, got {violations:?}");
    assert_eq!(hits[0].detail.as_deref(), Some("Setup"));
}

#[test]
fn ail003_silent_when_required_section_present() {
    let path = fixture("required_section", "good", "AGENTS.md");
    let cfg = config_with_required_setup();
    let violations = lint(&path, &cfg).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 3).collect();
    assert!(hits.is_empty(), "expected no AIL003, got {violations:?}");
}

#[test]
fn ail040_fires_on_broken_link() {
    let path = fixture("broken_local_link", "bad", "doc.md");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 40).collect();
    assert_eq!(hits.len(), 1, "expected one AIL040, got {violations:?}");
    assert!(hits[0]
        .detail
        .as_deref()
        .unwrap_or("")
        .contains("does-not-exist.md"));
}

#[test]
fn ail040_silent_on_valid_links() {
    let dir = {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("tests/fixtures/structural/broken_local_link/good");
        p
    };
    let violations = lint(&dir, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 40).collect();
    assert!(hits.is_empty(), "expected no AIL040, got {violations:?}");
}

#[test]
fn ail041_fires_on_malformed_yaml() {
    let path = fixture("malformed_yaml", "bad", "config.yaml");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 41).collect();
    assert_eq!(hits.len(), 1, "expected one AIL041, got {violations:?}");
    assert!(hits[0].message.contains("parse error"));
    assert!(hits[0].detail.is_some());
}

#[test]
fn ail041_silent_on_valid_yaml() {
    let path = fixture("malformed_yaml", "good", "config.yaml");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 41).collect();
    assert!(hits.is_empty(), "expected no AIL041, got {violations:?}");
}

#[test]
fn ail004_silent_on_valid_mcp_config() {
    let path = fixture("mcp_schema", "good", "mcp.json");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 4).collect();
    assert!(hits.is_empty(), "expected no AIL004, got {violations:?}");
}

#[test]
fn ail004_fires_on_bad_server_entries() {
    let path = fixture("mcp_schema", "bad", "mcp.json");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 4).collect();
    // Three entries — each has one problem.
    assert!(
        hits.len() >= 3,
        "expected at least three AIL004 findings, got {violations:?}"
    );
}

#[test]
fn ail004_fires_when_mcp_servers_key_missing() {
    let path = fixture("mcp_schema", "missing_key", "mcp.json");
    let violations = lint(&path, &Config::default()).unwrap();
    let hits: Vec<_> = violations.iter().filter(|v| v.rule_id.code == 4).collect();
    assert_eq!(hits.len(), 1, "expected one AIL004, got {violations:?}");
    assert!(
        hits[0]
            .detail
            .as_deref()
            .is_some_and(|d| d.contains("mcpServers")),
        "expected detail to mention mcpServers, got {:?}",
        hits[0].detail
    );
}
