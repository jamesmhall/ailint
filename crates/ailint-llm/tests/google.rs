use ailint_llm::{ChatRequest, GoogleProvider, LlmProvider};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn google_chat_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-test:generateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "modelVersion": "gemini-test",
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{"text": "hello from gemini"}]
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 9,
                "candidatesTokenCount": 4
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let provider = GoogleProvider::new("test-key").with_base_url(server.uri());
    let resp = provider
        .chat(&ChatRequest {
            model: "gemini-test".into(),
            user: "hi".into(),
            ..Default::default()
        })
        .await
        .expect("chat should succeed");

    assert_eq!(resp.text, "hello from gemini");
    assert_eq!(resp.usage.prompt_tokens, 9);
    assert_eq!(resp.usage.completion_tokens, 4);
    assert_eq!(resp.finish_reason.as_deref(), Some("STOP"));
    assert_eq!(resp.model, "gemini-test");
}
