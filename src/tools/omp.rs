//! Oh My Pi backend delegation - https://omp.sh and https://github.com/can1357/oh-my-pi
//!
//! OMP is a coding agent with LSP-wired edits, debugger support, AST rewrites,
//! and a broad provider catalog. Nur uses its headless one-shot entry point,
//! captures exact delegated usage, and defaults focused work to OMP's cheap
//! `pi/smol` role.

use super::{arg_str, Tool, ToolContext};
use crate::ecosystem;
use crate::error::{MuseError, Result};
use crate::usage::TokenUsage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_TIMEOUT_SECS: u64 = 300;
const MIN_TIMEOUT_SECS: u64 = 30;
const MAX_TIMEOUT_SECS: u64 = 600;
const MAX_PROMPT_CHARS: usize = 20_000;
const FOCUSED_TOOLS: &str = "read,grep,glob,lsp,edit,write,bash";

pub struct OmpTool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OmpAction {
    Run,
    Status,
    Version,
}

impl OmpAction {
    fn from_value(args: &Value) -> Self {
        match args.get("action").and_then(Value::as_str) {
            Some("status") => Self::Status,
            Some("version") => Self::Version,
            _ => Self::Run,
        }
    }

    fn is_read_only(self) -> bool {
        matches!(self, Self::Status | Self::Version)
    }
}

/// Only `status` and `version` are read-only. A run gives OMP write access to
/// the workspace and is approval-gated by Nur.
pub fn is_read_only_value(args: &Value) -> bool {
    OmpAction::from_value(args).is_read_only()
}

#[derive(Debug, Serialize, Deserialize)]
struct OmpRunEnvelope {
    backend: String,
    cost_mode: String,
    provider: Option<String>,
    model: Option<String>,
    output: String,
    usage: TokenUsage,
}

/// Extract delegated usage from a successful OMP tool result so the agent loop
/// can fold it into Nur's session budget, status, and usage display.
pub fn delegated_usage(result: &str) -> Option<TokenUsage> {
    let envelope: OmpRunEnvelope = serde_json::from_str(result).ok()?;
    (envelope.backend == "omp").then_some(envelope.usage)
}

impl Tool for OmpTool {
    fn name(&self) -> &str {
        "omp"
    }

    fn description(&self) -> &str {
        "Delegate a focused coding task to the Oh My Pi backend. Runs are write-class, \
         approval-gated, cancellation-aware, and included in Nur token/cost budgets. \
         cost_mode=economy (default) uses OMP's pi/smol role, low thinking, and a focused \
         tool set; use balanced only when the task needs the configured default model. \
         Strongest at LSP refactors, debugger-driven diagnosis, and AST rewrites. \
         action=run|status|version."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["run", "status", "version"],
                    "default": "run"
                },
                "prompt": {
                    "type": "string",
                    "maxLength": MAX_PROMPT_CHARS,
                    "description": "For run: a bounded task with scope and acceptance criteria"
                },
                "cost_mode": {
                    "type": "string",
                    "enum": ["economy", "balanced"],
                    "default": "economy",
                    "description": "economy uses pi/smol, low thinking, and focused tools"
                },
                "model": {
                    "type": "string",
                    "description": "Optional exact OMP model selector; overrides cost_mode model routing"
                },
                "thinking": {
                    "type": "string",
                    "enum": ["off", "minimal", "low", "medium", "high", "xhigh", "auto"],
                    "description": "Optional OMP thinking level; economy defaults to low"
                },
                "tool_profile": {
                    "type": "string",
                    "enum": ["focused", "full"],
                    "description": "Optional tool surface; economy defaults to focused"
                },
                "timeout_seconds": {
                    "type": "integer",
                    "minimum": MIN_TIMEOUT_SECS,
                    "maximum": MAX_TIMEOUT_SECS,
                    "default": DEFAULT_TIMEOUT_SECS
                }
            }
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let bin = ecosystem::find_bin("omp").ok_or_else(|| {
            MuseError::Tool(
                "omp CLI not found. Install Bun (bun.sh) then `nur ecosystem ensure`, \
                 or install directly: bun install -g @oh-my-pi/pi-coding-agent \
                 (Windows: irm https://omp.sh/install.ps1 | iex)"
                    .into(),
            )
        })?;

        match OmpAction::from_value(args) {
            OmpAction::Status | OmpAction::Version => {
                ecosystem::run_capture(&bin, &["--version"], None, 30_000)
                    .map(|version| {
                        if version.starts_with("omp") {
                            version
                        } else {
                            format!("omp {version}")
                        }
                    })
                    .map_err(MuseError::Tool)
            }
            OmpAction::Run => run_omp(&bin, args, ctx),
        }
    }
}

fn run_omp(bin: &str, args: &Value, ctx: &ToolContext) -> Result<String> {
    let prompt = arg_str(args, "prompt")?;
    let prompt_chars = prompt.chars().count();
    if prompt.trim().is_empty() {
        return Err(MuseError::Tool("omp prompt cannot be empty".into()));
    }
    if prompt_chars > MAX_PROMPT_CHARS {
        return Err(MuseError::Tool(format!(
            "omp prompt is {prompt_chars} characters; keep delegated context under {MAX_PROMPT_CHARS}"
        )));
    }

    let cost_mode = args
        .get("cost_mode")
        .and_then(Value::as_str)
        .unwrap_or("economy");
    if !matches!(cost_mode, "economy" | "balanced") {
        return Err(MuseError::Tool(
            "omp cost_mode must be economy or balanced".into(),
        ));
    }

    let timeout_secs = args
        .get("timeout_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    if !(MIN_TIMEOUT_SECS..=MAX_TIMEOUT_SECS).contains(&timeout_secs) {
        return Err(MuseError::Tool(format!(
            "omp timeout_seconds must be {MIN_TIMEOUT_SECS}..={MAX_TIMEOUT_SECS}"
        )));
    }

    let handoff = format!(
        "{prompt}\n\nNur handoff contract: stay within the requested scope, avoid unrelated \
         work, verify the result, and return only a compact outcome with files changed and checks run."
    );
    let argv = build_run_args(args, cost_mode, timeout_secs, handoff);
    let refs: Vec<&str> = argv.iter().map(String::as_str).collect();
    let wrapper_timeout_ms = timeout_secs.saturating_add(15).saturating_mul(1_000);
    let output = ecosystem::run_capture_cancelled(
        bin,
        &refs,
        Some(&ctx.cwd),
        wrapper_timeout_ms,
        &ctx.cancel,
    )
    .map_err(MuseError::Tool)?;
    let envelope = parse_json_run(&output, cost_mode)?;
    serde_json::to_string_pretty(&envelope).map_err(|error| MuseError::Tool(error.to_string()))
}

fn build_run_args(args: &Value, cost_mode: &str, timeout_secs: u64, prompt: String) -> Vec<String> {
    let mut argv = vec![
        "--mode".into(),
        "json".into(),
        "--no-session".into(),
        "--no-title".into(),
        "--max-time".into(),
        timeout_secs.to_string(),
        "--approval-mode".into(),
        "yolo".into(),
    ];

    let explicit_model = args
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty());
    if let Some(model) = explicit_model.or((cost_mode == "economy").then_some("pi/smol")) {
        argv.extend(["--model".into(), model.into()]);
    }

    let thinking = args
        .get("thinking")
        .and_then(Value::as_str)
        .or((cost_mode == "economy").then_some("low"));
    if let Some(thinking) = thinking {
        argv.extend(["--thinking".into(), thinking.into()]);
    }

    let tool_profile =
        args.get("tool_profile")
            .and_then(Value::as_str)
            .unwrap_or(if cost_mode == "economy" {
                "focused"
            } else {
                "full"
            });
    if tool_profile == "focused" {
        argv.extend(["--tools".into(), FOCUSED_TOOLS.into()]);
    }

    argv.extend(["-p".into(), prompt]);
    argv
}

fn parse_json_run(output: &str, cost_mode: &str) -> Result<OmpRunEnvelope> {
    let mut final_text = String::new();
    let mut provider = None;
    let mut model = None;
    let mut usage = TokenUsage::default();
    let mut saw_usage = false;

    for line in output.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if event.get("type").and_then(Value::as_str) != Some("message_end") {
            continue;
        }
        let Some(message) = event.get("message") else {
            continue;
        };
        if message.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }

        provider = message
            .get("provider")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or(provider);
        model = message
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or(model);

        if let Some(parsed) = message.get("usage").and_then(parse_usage) {
            usage.add(&parsed);
            saw_usage = true;
        }
        if let Some(text) = assistant_text(message) {
            final_text = text;
        }
    }

    if final_text.trim().is_empty() {
        let tail: String = output
            .chars()
            .rev()
            .take(1_500)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        return Err(MuseError::Tool(format!(
            "omp returned no final assistant message. Output tail:\n{tail}"
        )));
    }
    usage.cost_known = saw_usage;
    Ok(OmpRunEnvelope {
        backend: "omp".into(),
        cost_mode: cost_mode.into(),
        provider,
        model,
        output: final_text,
        usage,
    })
}

fn parse_usage(value: &Value) -> Option<TokenUsage> {
    let input = value.get("input").and_then(Value::as_u64).unwrap_or(0);
    let output = value.get("output").and_then(Value::as_u64).unwrap_or(0);
    let cache_read = value.get("cacheRead").and_then(Value::as_u64).unwrap_or(0);
    let cache_write = value.get("cacheWrite").and_then(Value::as_u64).unwrap_or(0);
    let total = value
        .get("totalTokens")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| input + output + cache_read + cache_write);
    let cost = value
        .pointer("/cost/total")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    Some(TokenUsage {
        input_tokens: input + cache_read + cache_write,
        output_tokens: output,
        total_tokens: total,
        reasoning_tokens: value
            .get("reasoningTokens")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        cached_tokens: cache_read,
        cost_usd: cost,
        cost_known: true,
    })
}

fn assistant_text(message: &Value) -> Option<String> {
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        return (!text.trim().is_empty()).then(|| text.to_string());
    }
    let text = message
        .get("content")?
        .as_array()?
        .iter()
        .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    (!text.trim().is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economy_args_use_the_smol_role_and_bounded_headless_mode() {
        let args = serde_json::json!({"action": "run", "prompt": "fix it"});
        let argv = build_run_args(&args, "economy", 300, "fix it".into());
        assert!(argv.windows(2).any(|v| v == ["--model", "pi/smol"]));
        assert!(argv.windows(2).any(|v| v == ["--thinking", "low"]));
        assert!(argv.windows(2).any(|v| v == ["--mode", "json"]));
        assert!(argv.windows(2).any(|v| v == ["--max-time", "300"]));
        assert!(argv.windows(2).any(|v| v == ["--tools", FOCUSED_TOOLS]));
        assert!(argv.contains(&"--no-session".to_string()));
        assert!(argv.windows(2).any(|v| v == ["--approval-mode", "yolo"]));
    }

    #[test]
    fn explicit_model_wins_and_balanced_keeps_the_full_surface() {
        let args = serde_json::json!({"model": "openai/gpt-test", "thinking": "medium"});
        let argv = build_run_args(&args, "balanced", 120, "task".into());
        assert!(argv.windows(2).any(|v| v == ["--model", "openai/gpt-test"]));
        assert!(argv.windows(2).any(|v| v == ["--thinking", "medium"]));
        assert!(!argv.contains(&"--tools".to_string()));
    }

    #[test]
    fn json_events_yield_compact_output_and_exact_usage() {
        let raw = r#"{"type":"message_end","message":{"role":"assistant","provider":"openai","model":"gpt-test","content":[{"type":"toolCall","name":"read"}],"usage":{"input":10,"output":2,"cacheRead":3,"cacheWrite":4,"totalTokens":19,"reasoningTokens":1,"cost":{"total":0.004}}}}
{"type":"message_end","message":{"role":"assistant","provider":"openai","model":"gpt-test","content":[{"type":"text","text":"done"}],"usage":{"input":5,"output":6,"cacheRead":1,"cacheWrite":0,"totalTokens":12,"cost":{"total":0.006}}}}"#;
        let parsed = parse_json_run(raw, "economy").unwrap();
        assert_eq!(parsed.output, "done");
        assert_eq!(parsed.provider.as_deref(), Some("openai"));
        assert_eq!(parsed.model.as_deref(), Some("gpt-test"));
        assert_eq!(parsed.usage.input_tokens, 23);
        assert_eq!(parsed.usage.output_tokens, 8);
        assert_eq!(parsed.usage.total_tokens, 31);
        assert_eq!(parsed.usage.cached_tokens, 4);
        assert_eq!(parsed.usage.reasoning_tokens, 1);
        assert!((parsed.usage.cost_usd - 0.01).abs() < f64::EPSILON);
        assert!(parsed.usage.cost_known);

        let encoded = serde_json::to_string(&parsed).unwrap();
        assert_eq!(delegated_usage(&encoded).unwrap().total_tokens, 31);
    }

    #[test]
    fn action_classification_is_typed_and_fail_closed() {
        assert!(is_read_only_value(&serde_json::json!({"action": "status"})));
        assert!(is_read_only_value(
            &serde_json::json!({"action": "version"})
        ));
        assert!(!is_read_only_value(&serde_json::json!({"action": "run"})));
    }
}
