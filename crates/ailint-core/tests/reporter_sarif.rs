use std::path::PathBuf;

use ailint_core::reporter::sarif::SarifReporter;
use ailint_core::reporter::Reporter;
use ailint_core::rules::{RuleId, Severity, Violation};

fn fixture_violations() -> Vec<Violation> {
    vec![
        Violation::new(
            RuleId::new(200, "no-prompt-injection-marker"),
            Severity::Error,
            PathBuf::from("docs/AGENTS.md"),
            "possible prompt injection marker",
        )
        .at(12, 3),
        Violation {
            fix_hint: Some("remove the vague phrase".to_string()),
            ..Violation::new(
                RuleId::new(100, "no-vague-instruction"),
                Severity::Warning,
                PathBuf::from("CLAUDE.md"),
                "vague phrase found: \"be careful\"",
            )
            .at(4, 1)
        },
        Violation::new(
            RuleId::new(2, "instructions-file-empty"),
            Severity::Info,
            PathBuf::from(".cursor/rules/empty.md"),
            "file is empty",
        ),
    ]
}

#[test]
fn sarif_report_valid_json_and_stable_shape() {
    let vs = fixture_violations();
    let mut buf: Vec<u8> = Vec::new();
    SarifReporter.report(&vs, &mut buf).unwrap();
    let out = String::from_utf8(buf).unwrap();
    let val: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(val["version"], "2.1.0");
    assert_eq!(
        val["$schema"],
        "https://json.schemastore.org/sarif-2.1.0.json"
    );
    assert!(val["runs"].is_array());

    let driver = &val["runs"][0]["tool"]["driver"];
    assert_eq!(driver["name"], "ailint");
    assert_eq!(
        driver["informationUri"],
        "https://github.com/jamesmhall/ailint"
    );
    assert!(driver["version"].is_string());

    let rules = driver["rules"].as_array().unwrap();
    assert!(
        rules.len() >= 10,
        "expected at least 10 registered rules, got {}",
        rules.len()
    );
    for r in rules {
        assert!(r["id"].as_str().unwrap().starts_with("AIL"));
        assert!(r["name"].is_string());
        assert!(r["helpUri"]
            .as_str()
            .unwrap()
            .starts_with("https://github.com/jamesmhall/ailint#"));
        let level = r["defaultConfiguration"]["level"].as_str().unwrap();
        assert!(matches!(level, "error" | "warning" | "note"));
    }

    let results = val["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);

    let levels: Vec<&str> = results
        .iter()
        .map(|r| r["level"].as_str().unwrap())
        .collect();
    assert_eq!(levels, vec!["error", "warning", "note"]);

    for r in results {
        assert!(r["ruleId"].as_str().unwrap().starts_with("AIL"));
        assert!(r["message"]["text"].is_string());
        let loc = &r["locations"][0]["physicalLocation"];
        assert!(loc["artifactLocation"]["uri"].is_string());
        assert!(loc["region"]["startLine"].is_i64());
        assert!(loc["region"]["startColumn"].is_i64());
    }

    let warning_result = &results[1];
    let fixes = warning_result["fixes"].as_array().unwrap();
    assert_eq!(fixes.len(), 1);
    assert_eq!(fixes[0]["description"]["text"], "remove the vague phrase");

    assert!(results[0]["fixes"].is_null());
}

#[test]
fn sarif_report_empty_still_valid() {
    let mut buf: Vec<u8> = Vec::new();
    SarifReporter.report(&[], &mut buf).unwrap();
    let val: serde_json::Value = serde_json::from_str(&String::from_utf8(buf).unwrap()).unwrap();
    assert_eq!(val["version"], "2.1.0");
    assert_eq!(val["runs"][0]["results"].as_array().unwrap().len(), 0);
    assert_eq!(val["runs"][0]["tool"]["driver"]["name"], "ailint");
    assert!(!val["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn sarif_results_snapshot() {
    let vs = fixture_violations();
    let mut buf: Vec<u8> = Vec::new();
    SarifReporter.report(&vs, &mut buf).unwrap();
    let val: serde_json::Value = serde_json::from_str(&String::from_utf8(buf).unwrap()).unwrap();

    let mut normalized = serde_json::Map::new();
    normalized.insert("version".to_string(), val["version"].clone());
    normalized.insert("$schema".to_string(), val["$schema"].clone());
    normalized.insert(
        "toolName".to_string(),
        val["runs"][0]["tool"]["driver"]["name"].clone(),
    );
    normalized.insert(
        "informationUri".to_string(),
        val["runs"][0]["tool"]["driver"]["informationUri"].clone(),
    );
    normalized.insert("results".to_string(), val["runs"][0]["results"].clone());

    let snapshot_value = serde_json::Value::Object(normalized);
    insta::assert_snapshot!(serde_json::to_string_pretty(&snapshot_value).unwrap());
}
