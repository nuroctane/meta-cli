//! Workspace sandbox: keep tools inside the session cwd and refuse drive-root walks.

use crate::error::{MuseError, Result};
use std::path::{Component, Path, PathBuf};

/// Paths that must never be used as a workspace root (no sandbox).
pub fn is_dangerous_workspace(path: &Path) -> bool {
    let Ok(canon) = path.canonicalize() else {
        // If it doesn't exist yet, still check raw form.
        return is_filesystem_root(path);
    };
    is_filesystem_root(&canon)
}

fn is_filesystem_root(path: &Path) -> bool {
    let s = path.to_string_lossy();
    // Unix /
    if path.parent().is_none() {
        return true;
    }
    // Windows drive roots: C:\ C:/
    #[cfg(windows)]
    {
        let t = s.trim_end_matches(['\\', '/']);
        if t.len() == 2 && t.as_bytes()[1] == b':' {
            return true;
        }
        // \\?\C:\
        if t.len() >= 6 && t.starts_with(r"\\?") {
            let rest = &t[4..];
            if rest.len() == 2 && rest.as_bytes()[1] == b':' {
                return true;
            }
        }
    }
    false
}

/// Resolve `path` against `cwd` and ensure the result stays under `cwd`.
pub fn resolve_in_workspace(cwd: &Path, path: &str) -> Result<PathBuf> {
    let cwd = cwd
        .canonicalize()
        .unwrap_or_else(|_| cwd.to_path_buf());
    let joined = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };

    // Normalize .. without requiring the path to exist.
    let normalized = normalize_path(&joined);
    let cwd_norm = normalize_path(&cwd);

    if !path_is_within(&normalized, &cwd_norm) {
        return Err(MuseError::Tool(format!(
            "path escapes workspace sandbox\n  path: {}\n  workspace: {}\n\
             Refuse: tools only operate under the session cwd.",
            normalized.display(),
            cwd_norm.display()
        )));
    }
    Ok(normalized)
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    let p = path.components().collect::<Vec<_>>();
    let r = root.components().collect::<Vec<_>>();
    if p.len() < r.len() {
        return false;
    }
    p.iter().zip(r.iter()).all(|(a, b)| a == b)
}

/// Lexical normalization (does not touch the filesystem).
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => out.push(c.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(s) => out.push(s),
        }
    }
    out
}

/// Prefer a git work tree root when the user launched from a dangerous root.
pub fn prefer_git_root(cwd: &Path) -> PathBuf {
    if !is_dangerous_workspace(cwd) {
        return cwd.to_path_buf();
    }
    // Walk up from process cwd first (often more specific than passed cwd)
    let start = std::env::current_dir().unwrap_or_else(|_| cwd.to_path_buf());
    let mut dir = start.as_path();
    loop {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => break,
        }
    }
    cwd.to_path_buf()
}

pub fn sandbox_warning(cwd: &Path) -> Option<String> {
    if is_dangerous_workspace(cwd) {
        Some(format!(
            "workspace is filesystem root ({}) — refuse wide globs; \
             start muse from a project directory or set --cwd",
            cwd.display()
        ))
    } else {
        None
    }
}
