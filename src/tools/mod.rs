mod bash;
mod edit_file;
mod glob;
mod grep;
mod read_file;
mod write_file;

use crate::api::types::ToolDef;
use crate::error::{MuseError, Result};
use serde_json::Value;
use std::path::PathBuf;

pub struct ToolContext {
    pub cwd: PathBuf,
    pub auto_approve: bool,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String>;
}

pub fn all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(read_file::ReadFile),
        Box::new(write_file::WriteFile),
        Box::new(edit_file::EditFile),
        Box::new(bash::Bash),
        Box::new(grep::Grep),
        Box::new(glob::GlobTool),
    ]
}

pub fn tool_defs() -> Vec<ToolDef> {
    all_tools()
        .into_iter()
        .map(|t| ToolDef {
            type_: "function".into(),
            name: t.name().into(),
            description: Some(t.description().into()),
            parameters: Some(t.parameters_schema()),
        })
        .collect()
}

pub fn dispatch(name: &str, arguments: &str, ctx: &ToolContext) -> Result<String> {
    let args: Value = serde_json::from_str(arguments).unwrap_or_else(|_| serde_json::json!({}));
    for tool in all_tools() {
        if tool.name() == name {
            return tool.execute(&args, ctx);
        }
    }
    Err(MuseError::Tool(format!("unknown tool: {name}")))
}

pub(crate) fn resolve_path(cwd: &PathBuf, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        cwd.join(p)
    }
}

pub(crate) fn arg_str(args: &Value, key: &str) -> Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| MuseError::Tool(format!("missing string arg: {key}")))
}

pub(crate) fn arg_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(|v| {
        v.as_u64()
            .or_else(|| v.as_i64().map(|i| i as u64))
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    })
}
