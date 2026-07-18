//! tldraw offline integration — the official desktop app (github.com/tldraw/
//! tldraw-offline) that runs interactive `.tldraw` files, including agent-shape
//! document scripts.
//!
//! This tool covers the parts nur can do correctly and offline:
//!   * `status`  — is the app installed? which build?
//!   * `install` — download + run the official platform installer.
//!   * `open`    — launch the app on a `.tldraw`/`.tldr` file (open output directly).
//!
//! Authoring of interactive agent-shape documents happens through the app's
//! local exec port (its documented agent mechanism); `create` records a spec
//! and points at that pipeline until the exec-port helper is provisioned.

use super::{arg_str, resolve_path, Tool, ToolContext};
use crate::error::{MuseError, Result};
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

const RELEASES_API: &str = "https://api.github.com/repos/tldraw/tldraw-offline/releases/latest";

pub struct Tldraw;

/// Actions that only inspect state (approval-free in manual mode).
pub fn is_read_only_action(args: &str) -> bool {
    let v: Value = serde_json::from_str(args).unwrap_or_else(|_| Value::Object(Default::default()));
    let action = v
        .get("action")
        .and_then(|a| a.as_str())
        .unwrap_or("status");
    matches!(action, "status" | "detect")
}

impl Tool for Tldraw {
    fn name(&self) -> &str {
        "tldraw"
    }

    fn description(&self) -> &str {
        "Manage the tldraw offline desktop app and open interactive .tldraw files in it. \
         action=status: is the app installed + latest release info. \
         action=install: download + run the official installer for this OS. \
         action=open (path=…): launch the app on a .tldraw/.tldr file so the user sees it. \
         Prefer open for user-facing boards. The app runs agent-shape document scripts, so \
         opened files can be live interactive apps. Authoring new interactive docs uses the \
         app's exec port (see create)."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["status", "detect", "install", "open", "create"],
                    "default": "status"
                },
                "path": {
                    "type": "string",
                    "description": "For open/create: workspace-relative .tldraw file path"
                },
                "spec": {
                    "description": "For create: high-level node/wire spec for the document"
                }
            }
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let action = arg_str(args, "action").unwrap_or_else(|_| "status".into());
        match action.as_str() {
            "status" | "detect" => Ok(status_report()),
            "install" => install(),
            "open" => open_action(args, &ctx.cwd),
            "create" => create_action(args, &ctx.cwd),
            other => Err(MuseError::Tool(format!(
                "unknown tldraw action '{other}' — use status|install|open|create"
            ))),
        }
    }
}

// ── app detection ──────────────────────────────────────────────────────────

/// Locate an installed tldraw offline executable / bundle for the current OS.
pub fn app_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match std::env::consts::OS {
        "windows" => {
            let mut roots = vec![
                home.join("AppData").join("Local").join("Programs"),
                PathBuf::from(r"C:\Program Files"),
                PathBuf::from(r"C:\Program Files (x86)"),
            ];
            if let Ok(lad) = std::env::var("LOCALAPPDATA") {
                roots.push(PathBuf::from(lad).join("Programs"));
            }
            for root in roots {
                if let Some(exe) = scan_for_tldraw_exe(&root) {
                    return Some(exe);
                }
            }
            None
        }
        "macos" => {
            let candidates = [
                PathBuf::from("/Applications/tldraw offline.app"),
                PathBuf::from("/Applications/tldraw-offline.app"),
                home.join("Applications").join("tldraw offline.app"),
            ];
            candidates.into_iter().find(|p| p.exists())
        }
        _ => {
            // Linux AppImage / installed binary.
            let candidates = [
                home.join(".local").join("bin").join("tldraw-offline"),
                PathBuf::from("/usr/bin/tldraw-offline"),
                PathBuf::from("/opt/tldraw-offline/tldraw-offline"),
            ];
            candidates.into_iter().find(|p| p.exists())
        }
    }
}

/// Look one level deep under `root` for a `*tldraw*offline*.exe`.
fn scan_for_tldraw_exe(root: &Path) -> Option<PathBuf> {
    let dir = std::fs::read_dir(root).ok()?;
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.contains("tldraw") {
            continue;
        }
        let sub = entry.path();
        if sub.is_dir() {
            if let Ok(inner) = std::fs::read_dir(&sub) {
                for f in inner.flatten() {
                    let fname = f.file_name().to_string_lossy().to_lowercase();
                    if fname.ends_with(".exe") && fname.contains("tldraw") {
                        return Some(f.path());
                    }
                }
            }
        }
    }
    None
}

// ── release resolution + install ───────────────────────────────────────────

/// Choose the release asset name pattern for this OS/arch.
fn asset_pattern() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "aarch64") => "tldraw-offline-win-arm64.exe",
        ("windows", _) => "tldraw-offline-win-x64.exe",
        ("macos", _) => "tldraw-offline-mac-universal.dmg",
        ("linux", "aarch64") => "tldraw-offline-linux-arm64.AppImage",
        ("linux", _) => "tldraw-offline-linux-x86_64.AppImage",
        _ => "tldraw-offline-win-x64.exe",
    }
}

/// Resolve (tag, asset_url) for the latest release asset matching this OS.
fn latest_asset_url() -> Result<(String, String)> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("nur-cli")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| MuseError::Tool(format!("http client: {e}")))?;
    let body: Value = client
        .get(RELEASES_API)
        .send()
        .map_err(|e| MuseError::Tool(format!("fetch releases: {e}")))?
        .json()
        .map_err(|e| MuseError::Tool(format!("parse releases: {e}")))?;
    let tag = body
        .get("tag_name")
        .and_then(|t| t.as_str())
        .unwrap_or("latest")
        .to_string();
    let want = asset_pattern();
    let url = body
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|arr| {
            arr.iter().find_map(|asset| {
                let name = asset.get("name").and_then(|n| n.as_str())?;
                if name == want {
                    asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| {
            MuseError::Tool(format!(
                "no release asset '{want}' found for this platform"
            ))
        })?;
    Ok((tag, url))
}

/// Best-effort install used by the ecosystem provisioner (`nur install` /
/// `ensure`). No-ops when the app is already present.
pub fn ensure_installed() -> Result<String> {
    install()
}

pub fn install() -> Result<String> {
    if let Some(app) = app_path() {
        return Ok(format!(
            "tldraw offline already installed: {}\n(use action=open path=… to open a file)",
            app.display()
        ));
    }
    let (tag, url) = latest_asset_url()?;
    let client = reqwest::blocking::Client::builder()
        .user_agent("nur-cli")
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| MuseError::Tool(format!("http client: {e}")))?;
    let bytes = client
        .get(&url)
        .send()
        .map_err(|e| MuseError::Tool(format!("download installer: {e}")))?
        .bytes()
        .map_err(|e| MuseError::Tool(format!("read installer body: {e}")))?;

    let fname = url.rsplit('/').next().unwrap_or("tldraw-offline-installer");
    let dl = std::env::temp_dir().join(fname);
    {
        let mut f = std::fs::File::create(&dl)
            .map_err(|e| MuseError::Tool(format!("write installer: {e}")))?;
        f.write_all(&bytes)
            .map_err(|e| MuseError::Tool(format!("save installer: {e}")))?;
    }

    let mut s = format!(
        "downloaded tldraw offline {tag} → {}\n",
        dl.display()
    );
    match std::env::consts::OS {
        "windows" => {
            // electron-builder NSIS: /S = silent, per-user by default.
            let status = std::process::Command::new(&dl).arg("/S").spawn();
            match status {
                Ok(_) => s.push_str(
                    "running the installer (silent). It installs per-user; re-run \
                     action=status in a few seconds to confirm.\n",
                ),
                Err(e) => s.push_str(&format!(
                    "could not launch installer automatically ({e}) — run it manually: {}\n",
                    dl.display()
                )),
            }
        }
        _ => {
            let _ = crate::open_uri::open_path(&dl);
            s.push_str(
                "opened the installer/image — complete installation, then re-run action=status.\n",
            );
        }
    }
    Ok(s)
}

// ── open + create ──────────────────────────────────────────────────────────

fn open_action(args: &Value, cwd: &Path) -> Result<String> {
    let path = arg_str(args, "path")
        .map_err(|_| MuseError::Tool("open requires path= to a .tldraw file".into()))?;
    let abs = resolve_path(&cwd.to_path_buf(), &path)?;
    if !abs.is_file() {
        return Err(MuseError::Tool(format!("file not found: {}", abs.display())));
    }
    let Some(app) = app_path() else {
        return Err(MuseError::Tool(
            "tldraw offline is not installed — run action=install first, then open again".into(),
        ));
    };
    // Launch the app on the file. Electron accepts the file path as an argument;
    // fall back to the shell association if the direct spawn fails.
    match std::process::Command::new(&app).arg(&abs).spawn() {
        Ok(_) => Ok(format!(
            "opened {} in tldraw offline ({})",
            abs.display(),
            app.display()
        )),
        Err(e) => match crate::open_uri::open_path(&abs) {
            Ok(()) => Ok(format!(
                "opened {} via file association (direct launch failed: {e})",
                abs.display()
            )),
            Err(e2) => Err(MuseError::Tool(format!(
                "could not open {}: {e2}",
                abs.display()
            ))),
        },
    }
}

fn create_action(args: &Value, cwd: &Path) -> Result<String> {
    let path = arg_str(args, "path")
        .map_err(|_| MuseError::Tool("create requires output path= (e.g. board.tldraw)".into()))?;
    let abs = resolve_path(&cwd.to_path_buf(), &path)?;
    let spec = args.get("spec").cloned().unwrap_or(Value::Null);
    // Persist the spec next to the target so the authoring helper can consume it.
    let spec_path = abs.with_extension("tldraw.spec.json");
    if let Some(parent) = spec_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(
        &spec_path,
        serde_json::to_string_pretty(&spec).unwrap_or_default(),
    )
    .map_err(|e| MuseError::Tool(format!("write spec: {e}")))?;
    Ok(format!(
        "wrote document spec → {}\n\
         Interactive .tldraw authoring runs through the tldraw offline exec port \
         (the app's agent mechanism). Install the app (action=install), then the \
         exec-port authoring helper turns this spec into {}.",
        spec_path.display(),
        abs.display()
    ))
}

fn status_report() -> String {
    let mut s = String::new();
    match app_path() {
        Some(app) => {
            s.push_str(&format!("tldraw offline: INSTALLED\n  {}\n", app.display()));
            s.push_str("open a file:  tldraw(action=open, path=board.tldraw)\n");
        }
        None => {
            s.push_str("tldraw offline: NOT INSTALLED\n");
            s.push_str("install:  tldraw(action=install)  — official app from github.com/tldraw/tldraw-offline\n");
        }
    }
    match latest_asset_url() {
        Ok((tag, _)) => s.push_str(&format!("latest release: {tag}\n")),
        Err(_) => s.push_str("latest release: (offline / unknown)\n"),
    }
    s
}
