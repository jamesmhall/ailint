use std::path::PathBuf;

use ailint_core::file_type::FileType;
use ailint_core::parser::{DocumentContent, ParsedDocument};
use ailint_core::rules::Severity;
use ailint_llm::{
    analyze, analyze_actionability, ChatRequest, ChatResponse, LlmProvider, ProviderKind, Usage,
};
use async_trait::async_trait;

struct CannedProvider {
    body: String,
}

#[async_trait]
impl LlmProvider for CannedProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Compatible
    }

    async fn chat(&self, _req: &ChatRequest) -> anyhow::Result<ChatResponse> {
        Ok(ChatResponse {
            text: self.body.clone(),
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
            },
            finish_reason: Some("stop".into()),
            model: "mock".into(),
        })
    }
}

fn doc(raw: &str) -> ParsedDocument {
    ParsedDocument {
        path: PathBuf::from("/tmp/AGENTS.md"),
        file_type: FileType::AgentsMd,
        raw: raw.to_string(),
        content: DocumentContent::Text,
    }
}

#[tokio::test]
async fn analyze_maps_issues_to_violations() {
    let provider = CannedProvider {
        body: r#"{"issues":[{"severity":"warning","line":3,"message":"vague","fix_hint":"be concrete"}]}"#.to_string(),
    };
    let vs = analyze(&provider, "mock", &doc("# Guide\n")).await.unwrap();
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].rule_id.code, 900);
    assert!(matches!(vs[0].severity, Severity::Warning));
    assert_eq!(vs[0].message, "vague");
    assert_eq!(vs[0].fix_hint.as_deref(), Some("be concrete"));
}

#[tokio::test]
async fn actionability_uses_warning_severity_and_ail901() {
    let provider = CannedProvider {
        // Model may hint any severity — the rule normalizes to Warning.
        body: r#"{"issues":[{"severity":"error","message":"undefined tool"}]}"#.to_string(),
    };
    let vs = analyze_actionability(&provider, "mock", &doc("# Guide\n"))
        .await
        .unwrap();
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].rule_id.code, 901);
    assert_eq!(vs[0].rule_id.slug, "llm-actionability-check");
    assert!(matches!(vs[0].severity, Severity::Warning));
    assert_eq!(vs[0].message, "undefined tool");
}

#[tokio::test]
async fn actionability_handles_empty_issue_list() {
    let provider = CannedProvider {
        body: r#"{"issues":[]}"#.to_string(),
    };
    let vs = analyze_actionability(&provider, "mock", &doc("# Guide\n"))
        .await
        .unwrap();
    assert!(vs.is_empty());
}
