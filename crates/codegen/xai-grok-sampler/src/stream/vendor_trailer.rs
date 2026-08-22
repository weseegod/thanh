//! Adapter for non-chunk side events some OpenAI-compatible gateways append
//! to a Chat Completions stream.
//!
//! Deserialization-only: consulted when the strict
//! [`ChatCompletionChunk`](xai_grok_sampling_types::ChatCompletionChunk) parse
//! fails, so conforming providers take exactly the same code path as before.

/// True when `data` is a gateway side event rather than a malformed chunk.
///
/// Observed in the wild: OpenCode Zen (`opencode.ai/zen`) appends a
/// post-`[DONE]` billing notice shaped like `{"choices":[],"cost":"0"}`,
/// sometimes without a preceding `[DONE]`; the strict chunk parse then kills
/// the whole turn with a fatal serialization error.
///
/// Deliberately narrow: the event must carry none of the chunk identity
/// fields (`id`, `object`, `created`, `model`). A payload missing only some
/// chunk fields is a genuine protocol violation, not a side event, and keeps
/// failing the strict parse as `SamplingError::Serialization`.
pub fn is_vendor_trailer(data: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(data) {
        Ok(serde_json::Value::Object(map)) => {
            !map.contains_key("id")
                && !map.contains_key("object")
                && !map.contains_key("created")
                && !map.contains_key("model")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_vendor_trailer;

    #[test]
    fn opencode_billing_trailer_matches() {
        // Captured verbatim from opencode.ai/zen/go/v1 (2026-08-22).
        assert!(is_vendor_trailer(r#"{"choices":[],"cost":"0"}"#));
    }

    #[test]
    fn trailer_with_extra_slots_still_matches() {
        assert!(is_vendor_trailer(
            r#"{"choices":[],"cost":123,"gateway":"zen"}"#
        ));
    }

    #[test]
    fn real_chunk_does_not_match() {
        let chunk = r#"{"id":"abc","object":"chat.completion.chunk","created":0,"model":"test","choices":[]}"#;
        assert!(!is_vendor_trailer(chunk));
    }

    #[test]
    fn partially_broken_chunk_does_not_match() {
        // Missing only `id` — a protocol violation, must stay fatal.
        let chunk = r#"{"object":"chat.completion.chunk","created":0,"model":"test","choices":[]}"#;
        assert!(!is_vendor_trailer(chunk));
    }

    #[test]
    fn non_objects_do_not_match() {
        assert!(!is_vendor_trailer("not-json-at-all"));
        assert!(!is_vendor_trailer(""));
        assert!(!is_vendor_trailer("[1,2,3]"));
        assert!(!is_vendor_trailer(r#""cost""#));
    }
}
