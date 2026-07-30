use ailint_llm::{ChatRequest, LlmProvider, OpenAiProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn openai_chat_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "gpt-test",
            "choices": [{
                "message": {"role": "assistant", "content": "hello world"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 5, "completion_tokens": 2}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new("test-key").with_base_url(server.uri());
    let resp = provider
        .chat(&ChatRequest {
            model: "gpt-test".into(),
            user: "hi".into(),
            ..Default::default()
        })
        .await
        .expect("chat should succeed");

    assert_eq!(resp.text, "hello world");
    assert_eq!(resp.usage.prompt_tokens, 5);
    assert_eq!(resp.usage.completion_tokens, 2);
    assert_eq!(resp.finish_reason.as_deref(), Some("stop"));
    assert_eq!(resp.model, "gpt-test");
}
