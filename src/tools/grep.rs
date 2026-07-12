use super::{arg_str, arg_u64, resolve_path, Tool, ToolContext};
use crate::error::{MuseError, Result};
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct Grep;

impl Tool for Grep {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents with a regex. Respects .gitignore. Returns matching lines with paths."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {"type": "string"},
                "path": {"type": "string", "description": "Directory or file to search (default cwd)"},
                "case_insensitive": {"type": "boolean", "default": false},
                "max_matches": {"type": "integer", "default": 50}
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
        let case_insensitive = args
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_matches = arg_u64(args, "max_matches").unwrap_or(50) as usize;

        let re = RegexBuilder::new(&pattern)
            .case_insensitive(case_insensitive)
            .build()
            .map_err(|e| MuseError::Tool(format!("invalid regex: {e}")))?;

        let root = resolve_path(&ctx.cwd, path);
        let mut matches = Vec::new();

        if root.is_file() {
            search_file(&root, &re, &mut matches, max_matches)?;
        } else {
            let walker = WalkBuilder::new(&root)
                .hidden(false)
                .git_ignore(true)
                .build();
            for entry in walker.flatten() {
                if matches.len() >= max_matches {
                    break;
                }
                let p = entry.path();
                if p.is_file() {
                    let _ = search_file(p, &re, &mut matches, max_matches);
                }
            }
        }

        if matches.is_empty() {
            return Ok("no matches".into());
        }
        Ok(matches.join("\n"))
    }
}

fn search_file(
    path: &Path,
    re: &regex::Regex,
    matches: &mut Vec<String>,
    max: usize,
) -> Result<()> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // skip binary / unreadable
    };
    for (i, line) in content.lines().enumerate() {
        if matches.len() >= max {
            break;
        }
        if re.is_match(line) {
            matches.push(format!("{}:{}:{}", path.display(), i + 1, line));
        }
    }
    Ok(())
}
