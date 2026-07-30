use std::path::PathBuf;

use ailint_core::file_type::FileType;
use ailint_core::parser::{DocumentContent, ParsedDocument};
use ailint_core::rules::{BatchRule, RuleContext, RuleId, Severity, Violation};
use ailint_core::Config;

struct FiresOnceRule;

const TEST_RULE: RuleId = RuleId::new(999, "test-batch-fires-once");

impl BatchRule for FiresOnceRule {
    fn id(&self) -> RuleId {
        TEST_RULE
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn run_batch(&self, docs: &[ParsedDocument], _ctx: &RuleContext<'_>) -> Vec<Violation> {
        let file = docs
            .first()
            .map(|d| d.path.clone())
            .unwrap_or_else(|| PathBuf::from("<corpus>"));
        vec![Violation::new(
            TEST_RULE,
            self.default_severity(),
            file,
            format!("batch rule saw {} docs", docs.len()),
        )]
    }
}

fn fake_doc(path: &str, raw: &str) -> ParsedDocument {
    ParsedDocument {
        path: PathBuf::from(path),
        file_type: FileType::AgentsMd,
        raw: raw.to_string(),
        content: DocumentContent::Text,
    }
}

#[test]
fn batch_rule_fires_once_across_corpus() {
    let docs = vec![
        fake_doc("a/AGENTS.md", "one"),
        fake_doc("b/AGENTS.md", "two"),
    ];
    let config = Config::default();
    let ctx = RuleContext {
        config: &config,
        options: None,
        severity: Severity::Info,
    };
    let rule = FiresOnceRule;
    let violations = rule.run_batch(&docs, &ctx);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule_id.code, 999);
    assert!(violations[0].message.contains("2 docs"));
}
