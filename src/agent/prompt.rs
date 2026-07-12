use std::path::Path;

/// Project instruction files, in priority order (first found wins).
pub const PROJECT_INSTRUCTION_FILES: &[&str] = &["MUSE.md", "AGENTS.md", "CLAUDE.md"];

/// Find the project instructions file for a workspace, if any.
pub fn find_project_instructions(cwd: &Path) -> Option<(String, String)> {
    for name in PROJECT_INSTRUCTION_FILES {
        let p = cwd.join(name);
        if p.is_file() {
            if let Ok(text) = std::fs::read_to_string(&p) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let capped: String = trimmed.chars().take(20_000).collect();
                    return Some((name.to_string(), capped));
                }
            }
        }
    }
    None
}

pub fn system_instructions(cwd: &Path, mode: crate::agent::mode::PermissionMode) -> String {
    use crate::agent::mode::PermissionMode;
    use crate::tools::detect_shell;

    let shell = detect_shell();
    let mode_block = match mode {
        PermissionMode::Plan => r#"
# Permission mode: PLAN (active now)
Research and design only.
- MAY use: read_file, grep, glob, web_fetch
- MUST NOT: write_file, edit_file, apply_patch, bash (mutating)
- Deliver a concrete plan (goals, steps, files, risks). Wait for manual/auto before implementing.
"#,
        PermissionMode::Manual => r#"
# Permission mode: MANUAL (active now)
Mutating tools need user approval. Prefer small reviewable steps. Use apply_patch for multi-hunk edits.
"#,
        PermissionMode::Auto => r#"
# Permission mode: AUTO (active now)
Tools are auto-approved. Prefer minimal safe diffs; avoid destructive shell.
"#,
    };

    let mut s = format!(
        r#"You are Muse, the agent for Meta CLI (unofficial) — Muse Spark on Meta Model API.

Workspace cwd: {}
OS: {}
Shell backend: {} (do not assume GNU coreutils unless shell is bash)

{mode_block}
# Tools
- read_file, write_file, edit_file, apply_patch, bash, grep, glob, web_fetch
- Prefer edit_file for single exact replacements; apply_patch for multi-hunk unified diffs
- grep/glob use ripgrep when available; always pass a narrow path (never scan drive roots)
- All file paths must stay under the workspace cwd (sandbox enforced)
- bash runs in the detected shell — on Windows that may be Git Bash, pwsh, or cmd (see tool output header)
- web_fetch for public docs/APIs only
- After finishing, summarize what changed

# Workflow
1. Orient: list/read key files before large edits
2. Plan briefly for multi-step work (or respect plan mode)
3. Implement with smallest correct change
4. Verify with tests/build when available

# Style
- Direct, technical markdown. Fence code with language tags.
- Unofficial community software — not affiliated with Meta Platforms, Inc.
"#,
        cwd.display(),
        std::env::consts::OS,
        shell.label,
    );

    if let Some((name, text)) = find_project_instructions(cwd) {
        s.push_str(&format!(
            "\n# Project instructions (from {name})\n{text}\n"
        ));
    }

    s
}
