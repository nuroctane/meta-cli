//! Load agent skills (SKILL.md) — Claude Code-compatible shape.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
    pub path: PathBuf,
}

/// Discover skills from (first match wins per name):
/// - `$NUR_HOME/skills` (or `~/.nur/skills`) — primary
/// - enabled marketplace plugins under `~/.nur/plugins/` (skills/ + pack roots)
/// - legacy `~/.muse/skills`
/// - `~/.agents/skills` (Agent Skills / graphify install --platform agents)
/// - `<cwd>/.meta/skills` · `<cwd>/.muse/skills` · `<cwd>/.agents/skills` · `<cwd>/.nur/skills`
pub fn load_skills(cwd: &Path) -> Vec<Skill> {
    let mut out = Vec::new();
    let mut dirs = Vec::new();
    // Honor NUR_HOME / META_HOME / MUSE_HOME via meta_home() — not a hard-coded path.
    dirs.push(crate::config::meta_home().join("skills"));
    // Marketplace plugins (enabled only) — after core home skills so user overrides win.
    dirs.extend(crate::plugins::enabled_skill_roots());
    dirs.push(crate::config::legacy_muse_home().join("skills"));
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".agents").join("skills"));
    }
    dirs.push(cwd.join(".nur").join("skills"));
    dirs.push(cwd.join(".meta").join("skills"));
    dirs.push(cwd.join(".muse").join("skills")); // legacy workspace
    dirs.push(cwd.join(".agents").join("skills"));

    for root in dirs {
        if !root.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let skill_md = if p.is_dir() {
                p.join("SKILL.md")
            } else if p
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.eq_ignore_ascii_case("SKILL.md"))
                .unwrap_or(false)
            {
                p.clone()
            } else {
                continue;
            };
            if !skill_md.is_file() {
                continue;
            }
            if let Some(skill) = parse_skill(&skill_md) {
                // dedupe by name
                if !out.iter().any(|s: &Skill| s.name == skill.name) {
                    out.push(skill);
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn parse_skill(path: &Path) -> Option<Skill> {
    let text = std::fs::read_to_string(path).ok()?;
    let name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("skill")
        .to_string();

    // Optional YAML frontmatter
    let (description, body) = if text.starts_with("---") {
        if let Some(end) = text[3..].find("---") {
            let fm = &text[3..end + 3];
            let body = text[end + 6..].trim().to_string();
            let desc = fm
                .lines()
                .find_map(|l| {
                    l.strip_prefix("description:")
                        .map(|s| s.trim().trim_matches('"').to_string())
                })
                .unwrap_or_else(|| first_line(&body));
            (desc, body)
        } else {
            (first_line(&text), text.clone())
        }
    } else {
        (first_line(&text), text.clone())
    };

    let body: String = body.chars().take(12_000).collect();
    Some(Skill {
        name,
        description,
        body,
        path: path.to_path_buf(),
    })
}

fn first_line(s: &str) -> String {
    s.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("skill")
        .trim()
        .trim_start_matches('#')
        .trim()
        .chars()
        .take(200)
        .collect()
}

/// Compact catalog for the system prompt + optional full body for named skills.
pub fn skills_prompt_section(skills: &[Skill]) -> String {
    if skills.is_empty() {
        return String::new();
    }
    let mut s = String::from("\n# Installed skills\n");
    s.push_str("Use these when the user task matches. Full text is available via read_file on the skill path.\n");
    for sk in skills {
        s.push_str(&format!(
            "- **{}**: {} (`{}`)\n",
            sk.name,
            sk.description,
            sk.path.display()
        ));
    }
    // Inline small skills fully
    for sk in skills.iter().filter(|s| s.body.len() < 2500).take(6) {
        s.push_str(&format!(
            "\n## Skill: {}\n{}\n",
            sk.name,
            sk.body.chars().take(2500).collect::<String>()
        ));
    }
    s
}
