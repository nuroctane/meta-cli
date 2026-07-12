use super::search_util::{is_hard_excluded, rg_files, walk_builder, SEARCH_BUDGET};
use super::{arg_str, arg_u64, resolve_path, Tool, ToolContext};
use crate::error::Result;
use serde_json::Value;
use std::time::Instant;

pub struct GlobTool;

impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a pattern (uses ripgrep --files when available; skips node_modules/target/etc.)."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "e.g. *.rs, **/mod.rs, src/"},
                "path": {"type": "string", "description": "Root directory (default cwd)"},
                "max_results": {"type": "integer", "default": 200}
            },
            "required": ["pattern"]
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let pattern = arg_str(args, "pattern")?;
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let max = arg_u64(args, "max_results").unwrap_or(200) as usize;
        let max = max.clamp(1, 1000);
        let root = resolve_path(&ctx.cwd, path)?;

        if let Some(out) = rg_files(&ctx.cwd, &pattern, &root, max) {
            if out.is_empty() {
                return Ok("no files matched".into());
            }
            return Ok(out);
        }

        // Fallback walk
        let pattern_lower = pattern.to_lowercase();
        let start = Instant::now();
        let mut hits = Vec::new();
        let walker = walk_builder(&root).build();
        for entry in walker.flatten() {
            if start.elapsed() > SEARCH_BUDGET {
                hits.push(format!(
                    "[search stopped after {}ms budget — install ripgrep for speed]",
                    SEARCH_BUDGET.as_millis()
                ));
                break;
            }
            let p = entry.path();
            if !p.is_file() || is_hard_excluded(p) {
                continue;
            }
            let s = p.to_string_lossy().replace('\\', "/");
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            let matched = if pattern_lower.starts_with("*.") {
                name.ends_with(&pattern_lower[1..])
            } else if pattern.contains('*') {
                let parts: Vec<&str> = pattern_lower.split('*').filter(|p| !p.is_empty()).collect();
                let s_lower = s.to_lowercase();
                parts.iter().all(|part| s_lower.contains(part))
            } else {
                s.to_lowercase().contains(&pattern_lower) || name.contains(&pattern_lower)
            };

            if matched {
                hits.push(s);
            }
            if hits.len() >= max {
                break;
            }
        }

        if hits.is_empty() {
            return Ok("no files matched".into());
        }
        hits.sort();
        Ok(hits.join("\n"))
    }
}
