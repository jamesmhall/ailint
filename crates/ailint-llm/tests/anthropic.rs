use ailint_llm::{AnthropicProvider, ChatRequest, LlmProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn anthropic_chat_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msg_1",
            "model": "claude-test",
            "role": "assistant",
            "type": "message",
            "content": [{"type": "text", "text": "hello from claude"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 7, "output_tokens": 3}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new("test-key").with_base_url(server.uri());
    let resp = provider
        .chat(&ChatRequest {
            model: "claude-test".into(),
            user: "hi".into(),
            ..Default::default()
        })
        .await
        .expect("chat should succeed");

    assert_eq!(resp.text, "hello from claude");
    assert_eq!(resp.usage.prompt_tokens, 7);
    assert_eq!(resp.usage.completion_tokens, 3);
    assert_eq!(resp.finish_reason.as_deref(), Some("end_turn"));
    assert_eq!(resp.model, "claude-test");
}
