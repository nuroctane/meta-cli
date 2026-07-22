
use super::{arg_str, resolve_path, Tool, ToolContext};
use crate::error::{MuseError, Result};
use serde_json::Value;
use std::fs;

pub struct EditFile;

impl Tool for EditFile {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Replace an exact string in a file. old_string must match uniquely unless replace_all is true. Always read the file first and copy old_string exactly including whitespace."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "old_string": {"type": "string"},
                "new_string": {"type": "string"},
                "replace_all": {"type": "boolean", "default": false}
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let path = arg_str(args, "path")?;
        let old = arg_str(args, "old_string")?;
        let new = arg_str(args, "new_string")?;
        let replace_all = args
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let full = resolve_path(&ctx.cwd, &path)?;
        let file_content = fs::read_to_string(&full)
            .map_err(|e| MuseError::Tool(format!("read {}: {e}", full.display())))?;

        // First try exact match (fast path)
        let count = file_content.matches(&old).count();
        if count > 0 {
            if count > 1 && !replace_all {
                return Err(MuseError::Tool(format!(
                    "old_string matched {count} times; set replace_all=true or make old_string unique"
                )));
            }
            let updated = if replace_all {
                file_content.replace(&old, &new)
            } else {
                file_content.replacen(&old, &new, 1)
            };
            fs::write(&full, updated)
                .map_err(|e| MuseError::Tool(format!("write {}: {e}", full.display())))?;
            return Ok(format!(
                "edited {} ({} replacement{})",
                full.display(),
                if replace_all { count } else { 1 },
                if replace_all && count != 1 { "s" } else { "" }
            ));
        }

        // Fallback 1: normalize line endings CRLF -> LF for both sides.
        // Models sometimes emit LF while file is CRLF or vice versa.
        let content_n = file_content.replace("\r\n", "\n");
        let old_n = old.replace("\r\n", "\n");
        let count_n = content_n.matches(&old_n).count();
        if count_n > 0 {
            if count_n > 1 && !replace_all {
                return Err(MuseError::Tool(format!(
                    "old_string matched {count_n} times after normalizing line endings; make it unique"
                )));
            }
            let new_n = new.replace("\r\n", "\n");
            let updated_n = if replace_all {
                content_n.replace(&old_n, &new_n)
            } else {
                content_n.replacen(&old_n, &new_n, 1)
            };
            // Preserve original line ending style if file was CRLF-heavy
            let updated = if file_content.contains("\r\n") && !updated_n.contains("\r\n") {
                updated_n.replace("\n", "\r\n")
            } else {
                updated_n
            };
            fs::write(&full, updated)
                .map_err(|e| MuseError::Tool(format!("write {}: {e}", full.display())))?;
            return Ok(format!(
                "edited {} ({} replacement{} via line-ending normalization)",
                full.display(),
                if replace_all { count_n } else { 1 },
                if replace_all && count_n != 1 { "s" } else { "" }
            ));
        }

        // Fallback 2: trimmed old_string search — model often copies with extra
        // leading/trailing newline or spaces. If trimmed version occurs uniquely,
        // replace that exact slice.
        let old_trim = old.trim();
        if !old_trim.is_empty() {
            let count_trim = file_content.matches(old_trim).count();
            if count_trim == 1 && !replace_all {
                let updated = file_content.replacen(old_trim, &new, 1);
                fs::write(&full, updated)
                    .map_err(|e| MuseError::Tool(format!("write {}: {e}", full.display())))?;
                return Ok(format!(
                    "edited {} (1 replacement via trimmed old_string fallback) — tip: copy exact whitespace next time",
                    full.display()
                ));
            }
            // Also try trimmed with normalized line endings
            let old_trim_n = old_trim.replace("\r\n", "\n");
            if content_n.matches(&old_trim_n).count() == 1 && !replace_all {
                let new_n = new.replace("\r\n", "\n");
                let updated_n = content_n.replacen(&old_trim_n, &new_n, 1);
                let updated = if file_content.contains("\r\n") {
                    updated_n.replace("\n", "\r\n")
                } else {
                    updated_n
                };
                fs::write(&full, updated)
                    .map_err(|e| MuseError::Tool(format!("write {}: {e}", full.display())))?;
                return Ok(format!(
                    "edited {} (1 replacement via trimmed+normalized fallback)",
                    full.display()
                ));
            }
        }

        // Still not found — build a helpful diagnostic. Previous message
        // "old_string not found in file" left the model blind and the transcript
        // showed a correct-looking diff with a failing footer on every expand.
        // Now we include file length and nearby context.
        let file_len = file_content.len();
        let line_count = file_content.lines().count();
        let mut hint = format!(
            "old_string not found in file (file {} chars, {} lines). ",
            file_len, line_count
        );

        if let Some(first_line) = old.lines().next().map(|l| l.trim()).filter(|l| !l.is_empty()) {
            if let Some(pos) = file_content.find(first_line) {
                let start = file_content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let end = file_content[pos..]
                    .find('\n')
                    .map(|i| pos + i)
                    .unwrap_or(file_content.len());
                let ctx_start = start.saturating_sub(200);
                let ctx_end = (end + 200).min(file_content.len());
                let snippet: String = file_content[ctx_start..ctx_end].chars().take(500).collect();
                hint.push_str(&format!(
                    "First line of old_string '{}' found near byte {}. Nearby snippet:\n---\n{}\n---\nTip: read the file with read_file and copy old_string exactly including whitespace.",
                    first_line, pos, snippet
                ));
            } else {
                hint.push_str(&format!(
                    "First line '{}' not found in file. Tip: read the file first with read_file, then copy the exact block you want to replace.",
                    first_line
                ));
            }
        } else {
            hint.push_str("old_string appears empty or whitespace-only after trimming — provide a non-empty unique block and read the file first.");
        }

        Err(MuseError::Tool(hint))
    }
}
