//! Minimal web fetch for docs/APIs — addresses the "no web" assessment.

use super::{arg_str, arg_u64, Tool, ToolContext};
use crate::error::{MuseError, Result};
use serde_json::Value;

pub struct WebFetch;

impl Tool for WebFetch {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch a public HTTP(S) URL and return text content (HTML/JSON/plain). \
         Max 500KB. Use for docs and APIs — not for authenticated/private resources."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "max_bytes": {"type": "integer", "default": 200000}
            },
            "required": ["url"]
        })
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String> {
        let url = arg_str(args, "url")?;
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(MuseError::Tool("url must start with http:// or https://".into()));
        }
        // Block obvious local/metadata SSRF targets
        let lower = url.to_ascii_lowercase();
        for bad in ["127.0.0.1", "localhost", "0.0.0.0", "169.254.", "[::1]", "metadata.google"] {
            if lower.contains(bad) {
                return Err(MuseError::Tool(format!("refused local/metadata URL: {url}")));
            }
        }
        let max = arg_u64(args, "max_bytes").unwrap_or(200_000) as usize;
        let max = max.min(500_000);

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("meta-cli/{}", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| MuseError::Tool(e.to_string()))?;

        let resp = client
            .get(&url)
            .send()
            .map_err(|e| MuseError::Tool(format!("fetch failed: {e}")))?;

        let status = resp.status();
        let ctype = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = resp
            .bytes()
            .map_err(|e| MuseError::Tool(format!("read body: {e}")))?;
        let truncated = bytes.len() > max;
        let slice = if truncated { &bytes[..max] } else { &bytes };
        let text = String::from_utf8_lossy(slice);

        let mut out = format!("url: {url}\nstatus: {status}\ncontent-type: {ctype}\n\n");
        out.push_str(&text);
        if truncated {
            out.push_str(&format!("\n\n[truncated at {max} bytes]"));
        }
        Ok(out)
    }
}
