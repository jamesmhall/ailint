use ailint_llm::{ChatRequest, LlmProvider, OllamaProvider};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn ollama_chat_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "llama-test",
            "message": {"role": "assistant", "content": "hello from ollama"},
            "done": true,
            "done_reason": "stop",
            "prompt_eval_count": 11,
            "eval_count": 6
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = OllamaProvider::new().with_base_url(server.uri());
    let resp = provider
        .chat(&ChatRequest {
            model: "llama-test".into(),
            user: "hi".into(),
            ..Default::default()
        })
        .await
        .expect("chat should succeed");

    assert_eq!(resp.text, "hello from ollama");
    assert_eq!(resp.usage.prompt_tokens, 11);
    assert_eq!(resp.usage.completion_tokens, 6);
    assert_eq!(resp.finish_reason.as_deref(), Some("stop"));
    assert_eq!(resp.model, "llama-test");
}
