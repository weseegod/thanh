//! Regression tests for the OpenCode Zen vendor side event appended to
//! Chat Completions streams.
//!
//! The gateway appends a non-chunk billing notice shaped like
//! `{"choices":[],"cost":"0"}`, sometimes without a preceding `[DONE]`.
//! The strict chunk parse used to fail the whole turn with a fatal
//! serialization error ("missing field `id` at line 1 column 25"); the
//! vendor-trailer adapter now skips it. Captured verbatim from
//! opencode.ai/zen/go/v1 (2026-08-22).

use std::sync::Arc;

use futures_util::StreamExt;

use xai_grok_sampler::{SamplerConfig, SamplingClient};
use xai_grok_sampling_types::{
    ContentPart, ConversationItem, ConversationRequest, UserItem,
};
use xai_grok_test_support::{MockInferenceServer, ScriptedResponse};

const CONTENT_CHUNK: &str = r#"{"id":"20260822ee21ffc9da5b44c0","object":"chat.completion.chunk","created":1787361795,"model":"ox-alpha-free","choices":[{"index":0,"delta":{"role":"assistant","content":"hi","reasoning_content":null}}]}"#;

const FINISH_CHUNK: &str = r#"{"id":"20260822ee21ffc9da5b44c0","object":"chat.completion.chunk","created":1787361795,"model":"ox-alpha-free","choices":[{"index":0,"finish_reason":"stop","delta":{"role":"assistant","content":"","reasoning_content":null}}]}"#;

/// The exact production payload that killed turns.
const VENDOR_TRAILER: &str = r#"{"choices":[],"cost":"0"}"#;

fn sse(events: &[&str]) -> String {
    let mut body = String::new();
    for event in events {
        body.push_str("data: ");
        body.push_str(event);
        body.push_str("\n\n");
    }
    body
}

fn user_request(text: &str) -> ConversationRequest {
    ConversationRequest {
        items: vec![ConversationItem::User(UserItem {
            content: vec![ContentPart::Text {
                text: Arc::<str>::from(text),
            }],
            ..Default::default()
        })],
        ..Default::default()
    }
}

async fn streamed_content(body: String) -> String {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.enqueue_response("/v1/chat/completions", ScriptedResponse::text(200, body));
    let cfg = SamplerConfig {
        api_key: Some("test-key".to_string()),
        base_url: server.url(),
        model: "ox-alpha-free".to_string(),
        ..SamplerConfig::default()
    };
    let client = SamplingClient::new(cfg).expect("client");
    let (mut stream, _metadata) = client
        .conversation_stream(user_request("hi"))
        .await
        .expect("stream starts");
    let mut content = String::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                for choice in chunk.choices {
                    if let Some(text) = choice.delta.content {
                        content.push_str(&text);
                    }
                }
            }
            Err(e) => panic!("vendor side event must not fail the stream: {e}"),
        }
    }
    content
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trailer_without_done_marker_is_skipped() {
    // The production failure shape: no `[DONE]`, stream ends on the trailer.
    let content = streamed_content(sse(&[CONTENT_CHUNK, FINISH_CHUNK, VENDOR_TRAILER])).await;
    assert_eq!(content, "hi");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn trailer_after_done_marker_changes_nothing() {
    // The common shape: the mapper stops at `[DONE]`; behavior must stay
    // identical to before the adapter existed.
    let content =
        streamed_content(sse(&[CONTENT_CHUNK, FINISH_CHUNK, "[DONE]", VENDOR_TRAILER])).await;
    assert_eq!(content, "hi");
}
