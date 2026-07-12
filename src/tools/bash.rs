use super::{arg_str, arg_u64, Tool, ToolContext};
use crate::error::{MuseError, Result};
use serde_json::Value;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Bash;

impl Tool for Bash {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Run a shell command in the workspace cwd. Prefer non-interactive commands. Captures stdout/stderr."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {"type": "string"},
                "timeout_ms": {"type": "integer", "description": "Timeout in ms (default 120000)"}
            },
            "required": ["command"]
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let command = arg_str(args, "command")?;
        let timeout_ms = arg_u64(args, "timeout_ms").unwrap_or(120_000);

        if !ctx.auto_approve {
            let lower = command.to_lowercase();
            let dangerous = ["rm -rf", "del /f", "format ", "shutdown", "mkfs"];
            if dangerous.iter().any(|d| lower.contains(d)) {
                return Err(MuseError::Tool(format!(
                    "refused potentially destructive command without --yes: {command}"
                )));
            }
        }

        run_command(&command, &ctx.cwd, timeout_ms)
    }
}

fn run_command(command: &str, cwd: &std::path::Path, timeout_ms: u64) -> Result<String> {
    let command = command.to_string();
    let cwd = cwd.to_path_buf();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        #[cfg(windows)]
        let result = Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&cwd)
            .output();

        #[cfg(not(windows))]
        let result = Command::new("sh")
            .args(["-c", &command])
            .current_dir(&cwd)
            .output();

        let _ = tx.send(result);
    });

    let result = rx
        .recv_timeout(Duration::from_millis(timeout_ms))
        .map_err(|_| MuseError::Tool(format!("command timed out after {timeout_ms}ms")))?
        .map_err(|e| MuseError::Tool(format!("command failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let code = result.status.code().unwrap_or(-1);

    let mut out = format!("exit_code: {code}\n");
    if !stdout.is_empty() {
        out.push_str("stdout:\n");
        out.push_str(&truncate(&stdout, 80_000));
        out.push('\n');
    }
    if !stderr.is_empty() {
        out.push_str("stderr:\n");
        out.push_str(&truncate(&stderr, 40_000));
        out.push('\n');
    }
    if stdout.is_empty() && stderr.is_empty() {
        out.push_str("(no output)\n");
    }
    Ok(out)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…\n[truncated {} chars]", &s[..max], s.len())
    }
}
