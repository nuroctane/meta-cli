//! Local-model placeholder resolution — shared by sync and async paths.
//!
//! `local-model` provably 400s on real servers (Group C observed
//! `POST {"model":"local-model"}` → 400 on a live llama.cpp instance,
//! while a real id from `GET /v1/models` → 200). The fix is lazy `/models`
//! resolution, not a better placeholder. This module holds the tiny shared
//! surface so `models.rs` (blocking) and `client.rs` (async) don't duplicate
//! `/models` parsing logic.

use super::models::parse_model_ids;

/// Is this the placeholder that never resolves on a real local server?
pub fn is_placeholder(model: &str) -> bool {
    model.trim() == "local-model"
}

/// Pick the first usable id from a `/models` list, skipping empty and the
/// placeholder itself (defensive, in case a server echoes it).
pub fn pick_first_id(ids: Vec<String>) -> Option<String> {
    ids.into_iter()
        .find(|id| !id.trim().is_empty() && id.trim() != "local-model")
}

/// Parse a `/models` body and pick the first id, if any.
pub fn parse_first_id(body: &str) -> Option<String> {
    parse_model_ids(body).ok().and_then(pick_first_id)
}

/// Shared check: is this provider id a localhost inference server that
/// historically shipped the `local-model` placeholder? Delegates to the
/// catalog predicate so there is one source of truth.
pub fn is_local_provider_id(id: &str) -> bool {
    crate::providers::is_local_provider(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_detection() {
        assert!(is_placeholder("local-model"));
        assert!(is_placeholder("  local-model  "));
        assert!(!is_placeholder("llama3.3"));
    }

    #[test]
    fn picks_first_non_placeholder() {
        let ids = vec![
            "".to_string(),
            "local-model".to_string(),
            "llama3.3".to_string(),
            "mistral".to_string(),
        ];
        assert_eq!(pick_first_id(ids), Some("llama3.3".to_string()));
    }

    #[test]
    fn parses_openai_list() {
        let body = r#"{"data":[{"id":"llama3.3"},{"id":"mistral"}]}"#;
        assert_eq!(parse_first_id(body), Some("llama3.3".to_string()));
    }
}
