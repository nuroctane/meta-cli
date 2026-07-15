//! Live model discovery for the `/model` picker.
//!
//! Almost every provider in [`crate::providers`] is OpenAI-compatible and
//! answers `GET {base_url}/models` with `{ "data": [ { "id": … } ] }`. This
//! fetches that list (blocking — call from a background thread, same pattern as
//! the OAuth flow) so the picker can show what a provider actually offers
//! instead of making the user memorize model ids.

use serde::Deserialize;

#[derive(Deserialize)]
struct ModelList {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

/// Fetch the provider's model ids from `{base_url}/models`.
///
/// `base_url` is the catalog base (no trailing slash, no endpoint path).
/// Returns a sorted, de-duplicated list. Errors are surfaced as a short string
/// so the picker can show them inline (no key, network down, endpoint missing).
pub fn fetch_model_ids(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("nur-cli/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("client error: {e}"))?;

    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }
    let res = req.send().map_err(|e| format!("request failed: {e}"))?;

    let status = res.status();
    let body = res.text().unwrap_or_default();
    if !status.is_success() {
        // Trim noisy HTML/error bodies to a readable snippet.
        let snippet: String = body.trim().chars().take(160).collect();
        return Err(format!("HTTP {} · {}", status.as_u16(), snippet));
    }

    // Accept the standard `{ "data": [...] }` shape, or a bare array fallback.
    let mut ids: Vec<String> = match serde_json::from_str::<ModelList>(&body) {
        Ok(list) => list.data.into_iter().map(|m| m.id).collect(),
        Err(_) => match serde_json::from_str::<Vec<ModelEntry>>(&body) {
            Ok(arr) => arr.into_iter().map(|m| m.id).collect(),
            Err(_) => return Err("unexpected /models response shape".to_string()),
        },
    };

    ids.retain(|id| !id.trim().is_empty());
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err("provider returned no models".to_string());
    }
    Ok(ids)
}

/// Parse a `/models` response body into sorted, de-duplicated ids. Split out so
/// the response-shape handling is unit-testable without a live network call.
#[cfg(test)]
fn parse_model_ids(body: &str) -> std::result::Result<Vec<String>, String> {
    let mut ids: Vec<String> = match serde_json::from_str::<ModelList>(body) {
        Ok(list) => list.data.into_iter().map(|m| m.id).collect(),
        Err(_) => match serde_json::from_str::<Vec<ModelEntry>>(body) {
            Ok(arr) => arr.into_iter().map(|m| m.id).collect(),
            Err(_) => return Err("unexpected /models response shape".to_string()),
        },
    };
    ids.retain(|id| !id.trim().is_empty());
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::parse_model_ids;

    #[test]
    fn parses_openai_data_shape() {
        let body = r#"{"object":"list","data":[{"id":"gpt-5.5"},{"id":"gpt-4o"}]}"#;
        assert_eq!(parse_model_ids(body).unwrap(), vec!["gpt-4o", "gpt-5.5"]);
    }

    #[test]
    fn parses_bare_array_fallback() {
        let body = r#"[{"id":"claude-sonnet-5"},{"id":"claude-opus-4-8"}]"#;
        assert_eq!(
            parse_model_ids(body).unwrap(),
            vec!["claude-opus-4-8", "claude-sonnet-5"]
        );
    }

    #[test]
    fn dedupes_and_drops_blanks() {
        let body = r#"{"data":[{"id":"a"},{"id":"a"},{"id":" "}]}"#;
        assert_eq!(parse_model_ids(body).unwrap(), vec!["a"]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_model_ids("not json").is_err());
    }
}
