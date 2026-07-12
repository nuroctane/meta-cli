use super::{arg_str, resolve_path, Tool, ToolContext};
use crate::error::Result;
use ignore::WalkBuilder;
use serde_json::Value;

pub struct GlobTool;

impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob-like pattern (substring / extension filters). Respects .gitignore."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string", "description": "e.g. *.rs, **/mod.rs, src/"},
                "path": {"type": "string", "description": "Root directory (default cwd)"}
            },
            "required": ["pattern"]
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let pattern = arg_str(args, "pattern")?;
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        let root = resolve_path(&ctx.cwd, path);

        let pattern_lower = pattern.to_lowercase();
        let mut hits = Vec::new();

        let walker = WalkBuilder::new(&root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker.flatten() {
            let p = entry.path();
            if !p.is_file() {
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
                // crude: strip * and check contains of remaining parts
                let parts: Vec<&str> = pattern_lower.split('*').filter(|p| !p.is_empty()).collect();
                let s_lower = s.to_lowercase();
                parts.iter().all(|part| s_lower.contains(part))
            } else {
                s.to_lowercase().contains(&pattern_lower) || name.contains(&pattern_lower)
            };

            if matched {
                hits.push(s);
            }
            if hits.len() >= 200 {
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
