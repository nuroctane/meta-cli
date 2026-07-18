//! tldraw offline integration — official desktop app
//! (github.com/tldraw/tldraw-offline) for interactive `.tldraw` files.
//!
//! Actions:
//!   * `status`  — is the app installed?
//!   * `install` — download + run the official platform installer
//!   * `open`    — launch the app on a `.tldraw`/`.tldr` (robust Windows launch)
//!   * `create`  — write a **valid** `.tldraw` document from a shape list, then open
//!   * `run`     — alias of `open`
//!
//! Models must use `create` (or open an existing valid file). Invented JSON via
//! `write_file` is not a tldraw document and will fail to open usefully.

use super::{arg_str, resolve_path, Tool, ToolContext};
use crate::error::{MuseError, Result};
use serde_json::{json, Value};
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
        "tldraw offline desktop app + valid .tldraw boards. \
         action=status: installed? \
         action=install: download official installer. \
         action=create: write a REAL .tldraw from shapes and open it. \
         Outputs ALWAYS save to the user's Desktop (filename only; path= optional). \
         action=open (or run): launch the app on path=.tldraw (Desktop or absolute). \
         NEVER invent .tldraw JSON with write_file — use create. \
         shape items: {id?, x, y, w, h, text, color?, geo?} color= black|grey|white|blue|green|red|orange|yellow|violet|light-blue|light-green|light-red|light-violet."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["status", "detect", "install", "open", "run", "create"],
                    "default": "status"
                },
                "path": {
                    "type": "string",
                    "description": "Filename or path. create: saved under Desktop (basename used). open: Desktop-relative, absolute, or workspace-relative."
                },
                "title": {
                    "type": "string",
                    "description": "Document / page title for create (also used for default filename if path omitted)"
                },
                "shapes": {
                    "type": "array",
                    "description": "For create: list of boxes {x,y,w,h,text,color?,geo?}",
                    "items": { "type": "object" }
                },
                "spec": {
                    "description": "Legacy: ignored if shapes provided; else stored as .spec.json"
                }
            }
        })
    }

    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String> {
        let action = arg_str(args, "action").unwrap_or_else(|_| "status".into());
        match action.as_str() {
            "status" | "detect" => Ok(status_report()),
            "install" => install(),
            "open" | "run" => open_action(args, &ctx.cwd),
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
            // Prefer main app over Uninstall / elevate stubs.
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
            let candidates = [
                home.join(".local").join("bin").join("tldraw-offline"),
                PathBuf::from("/usr/bin/tldraw-offline"),
                PathBuf::from("/opt/tldraw-offline/tldraw-offline"),
            ];
            candidates.into_iter().find(|p| p.exists())
        }
    }
}

/// Look one level deep under `root` for the main `*tldraw*.exe` (skip Uninstall).
fn scan_for_tldraw_exe(root: &Path) -> Option<PathBuf> {
    let dir = std::fs::read_dir(root).ok()?;
    let mut best: Option<PathBuf> = None;
    let mut best_size: u64 = 0;
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.contains("tldraw") {
            continue;
        }
        let sub = entry.path();
        if !sub.is_dir() {
            continue;
        }
        if let Ok(inner) = std::fs::read_dir(&sub) {
            for f in inner.flatten() {
                let fname = f.file_name().to_string_lossy().to_lowercase();
                if !fname.ends_with(".exe") || !fname.contains("tldraw") {
                    continue;
                }
                if fname.contains("uninstall") || fname.contains("elevate") {
                    continue;
                }
                let len = f.metadata().map(|m| m.len()).unwrap_or(0);
                // Main Electron app is huge (100MB+); installer stubs are smaller.
                if len > best_size {
                    best_size = len;
                    best = Some(f.path());
                }
            }
        }
    }
    best
}

// ── release resolution + install ───────────────────────────────────────────

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
            MuseError::Tool(format!("no release asset '{want}' found for this platform"))
        })?;
    Ok((tag, url))
}

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

    let mut s = format!("downloaded tldraw offline {tag} → {}\n", dl.display());
    match std::env::consts::OS {
        "windows" => {
            // Do NOT use /S first — silent install sometimes leaves a broken
            // session. Prefer interactive so the user completes setup; still
            // try silent as a second launch if needed.
            match std::process::Command::new(&dl).spawn() {
                Ok(_) => s.push_str(
                    "launched the installer window — finish setup, then re-run action=status.\n",
                ),
                Err(e) => s.push_str(&format!(
                    "could not launch installer ({e}) — run manually: {}\n",
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

// ── open ───────────────────────────────────────────────────────────────────

fn open_action(args: &Value, cwd: &Path) -> Result<String> {
    let path = arg_str(args, "path")
        .map_err(|_| MuseError::Tool("open requires path= to a .tldraw file".into()))?;
    let abs = resolve_open_path(cwd, &path)?;
    if !abs.is_file() {
        return Err(MuseError::Tool(format!(
            "file not found: {}\n  (create saves boards to Desktop — try opening from there)",
            abs.display()
        )));
    }
    launch_on_file(&abs)
}

/// User's Desktop directory (Windows/macOS/Linux via `dirs`).
pub fn desktop_dir() -> PathBuf {
    dirs::desktop_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Desktop")))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve a path for `open`: absolute as-is; else try Desktop first, then workspace.
fn resolve_open_path(cwd: &Path, path: &str) -> Result<PathBuf> {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        return Ok(p);
    }
    let desktop = desktop_dir().join(path);
    if desktop.is_file() {
        return Ok(desktop);
    }
    // Bare filename on Desktop
    if let Some(name) = p.file_name() {
        let desk_name = desktop_dir().join(name);
        if desk_name.is_file() {
            return Ok(desk_name);
        }
    }
    // Fall back to workspace (sandbox-checked)
    resolve_path(&cwd.to_path_buf(), path)
}

/// Resolve output path for `create` — **always under Desktop**.
///
/// Relative paths and bare names become `Desktop/<basename>.tldraw`.
/// Absolute paths outside Desktop still land on Desktop using the file name
/// so boards stay easy to find.
fn resolve_create_path(args: &Value) -> Result<PathBuf> {
    let desktop = desktop_dir();
    let _ = std::fs::create_dir_all(&desktop);

    let title = arg_str(args, "title").unwrap_or_else(|_| "Board".into());
    let raw = arg_str(args, "path").ok();

    let mut name = match raw.as_deref() {
        Some(p) if !p.trim().is_empty() => {
            let pb = PathBuf::from(p);
            pb.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| slug_filename(&title))
        }
        _ => slug_filename(&title),
    };
    if name.is_empty() {
        name = "board.tldraw".into();
    }
    let lower = name.to_ascii_lowercase();
    if !(lower.ends_with(".tldraw") || lower.ends_with(".tldr")) {
        name.push_str(".tldraw");
    }

    Ok(desktop.join(name))
}

fn slug_filename(title: &str) -> String {
    let mut s: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "board".into()
    } else {
        // Cap length for Windows path comfort
        if s.len() > 48 {
            s.truncate(48);
            s = s.trim_end_matches('-').to_string();
        }
        s
    }
}

/// Shared launcher used by the tool and `/draw <file>`.
///
/// Important Windows behavior (verified):
/// - Launching the official Electron app **with an invalid `.tldraw`** starts
///   processes but often produces **no MainWindowHandle** (invisible app).
/// - `.tldraw` is frequently **not registered** with a file association, so
///   `cmd start file.tldraw` alone is unreliable.
/// - Reliable path: `cmd /c start "" "app.exe" [valid-file]` (or bare app).
pub fn launch_on_file(abs: &Path) -> Result<String> {
    let abs = abs
        .canonicalize()
        .unwrap_or_else(|_| abs.to_path_buf());
    let notes = validate_or_hint(&abs);
    let file_is_valid = notes.is_empty();

    let Some(app) = app_path() else {
        // Last-ditch: OS association + web note.
        let _ = crate::open_uri::open_path(&abs);
        return Err(MuseError::Tool(
            "tldraw offline is not installed — run action=install first, then open again.\n\
             Or drag the file onto https://www.tldraw.com/ in a browser."
                .into(),
        ));
    };

    let mut methods: Vec<String> = Vec::new();
    let mut opened = false;

    // Only pass the path when the document is valid. Invalid invented JSON
    // makes Electron run headless / without a real window.
    let file_arg = if file_is_valid {
        Some(abs.as_path())
    } else {
        None
    };

    // 1) Primary: `cmd /c start "" app [file]` — start detaches into the
    //    interactive desktop session; do NOT use DETACHED_PROCESS on Electron.
    match spawn_via_shell_start(&app, file_arg) {
        Ok(()) => {
            methods.push(if file_is_valid {
                format!("cmd start \"{}\" + file", app.display())
            } else {
                format!("cmd start bare app \"{}\" (file invalid — not passed)", app.display())
            });
            opened = true;
        }
        Err(e) => methods.push(format!("cmd start failed: {e}")),
    }

    // 2) Fallback: CreateProcess on the exe (no creation-flag tricks).
    if !opened {
        match spawn_app_plain(&app, file_arg) {
            Ok(()) => {
                methods.push(format!("direct spawn {}", app.display()));
                opened = true;
            }
            Err(e) => methods.push(format!("direct spawn failed: {e}")),
        }
    }

    // 3) Reveal in Explorer (user can File→Open from the app, or drag onto web).
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer.exe")
            .arg(format!("/select,{}", abs.display()))
            .spawn();
        methods.push("explorer /select".into());
    }

    methods.push("web: https://www.tldraw.com/ (drag file onto page)".into());

    if !opened {
        return Err(MuseError::Tool(format!(
            "could not launch tldraw offline at {}\n  tried: {}\n\
             Open the app from the Start menu, then File → Open:\n  {}",
            app.display(),
            methods.join(" · "),
            abs.display()
        )));
    }

    let mut out = format!(
        "opened tldraw offline for {}\n  methods: {}\n",
        abs.display(),
        methods.join(" · ")
    );
    if !notes.is_empty() {
        out.push_str(&notes);
        out.push('\n');
        out.push_str(
            "App launched without that file (invalid docs crash the window). \
             Use tldraw(action=create, path=…, shapes=[…]) to rewrite a real board, \
             then open again. Or File → Open after fixing the JSON.\n",
        );
    } else {
        out.push_str(
            "Look for the \"tldraw offline\" window (Alt+Tab if needed).\n",
        );
    }
    Ok(out)
}

/// Windows: `cmd /c start "" app [file]` so the GUI is owned by Explorer's
/// desktop session. Empty title arg is required so paths are not mis-parsed.
fn spawn_via_shell_start(app: &Path, file: Option<&Path>) -> std::result::Result<(), String> {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new("cmd.exe");
        cmd.arg("/C").arg("start").arg("").arg(app.as_os_str());
        if let Some(f) = file {
            cmd.arg(f.as_os_str());
        }
        cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        spawn_app_plain(app, file)
    }
}

/// Plain CreateProcess / exec — no DETACHED_PROCESS (that flag can leave
/// Electron running with MainWindowHandle=0).
fn spawn_app_plain(app: &Path, file: Option<&Path>) -> std::result::Result<(), String> {
    let mut cmd = std::process::Command::new(app);
    if let Some(f) = file {
        cmd.arg(f);
    }
    // Detach stdio so nur's pipes do not pin Electron; still a normal GUI process.
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NEW_PROCESS_GROUP only — survive parent, keep a real window.
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn validate_or_hint(path: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(path) else {
        return String::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return "warning: file is not JSON — tldraw may refuse it.".into();
    };
    if v.get("tldrawFileFormatVersion").is_some() && v.get("records").is_some() {
        return String::new();
    }
    // Common failure mode: models invent a fake schema via write_file.
    "warning: this is NOT a valid .tldraw document (missing tldrawFileFormatVersion + records). \
     Use tldraw(action=create, path=…, shapes=[…]) to write a real file the app can open."
        .into()
}

// ── create valid .tldraw ───────────────────────────────────────────────────

fn create_action(args: &Value, _cwd: &Path) -> Result<String> {
    // Always Desktop — not workspace (sandbox would block Desktop otherwise).
    let abs = resolve_create_path(args)?;
    if let Some(parent) = abs.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let title = arg_str(args, "title").unwrap_or_else(|_| "Board".into());
    let shapes = args
        .get("shapes")
        .and_then(|s| s.as_array())
        .cloned()
        .unwrap_or_default();

    if shapes.is_empty() {
        // Persist legacy spec for debugging, but still write an empty valid board.
        if let Some(spec) = args.get("spec") {
            let spec_path = abs.with_extension("tldraw.spec.json");
            let _ = std::fs::write(
                &spec_path,
                serde_json::to_string_pretty(spec).unwrap_or_default(),
            );
        }
    }

    let doc = build_tldraw_document(&title, &shapes);
    let body = serde_json::to_string_pretty(&doc)
        .map_err(|e| MuseError::Tool(format!("serialize tldraw: {e}")))?;
    std::fs::write(&abs, body).map_err(|e| MuseError::Tool(format!("write tldraw: {e}")))?;

    let mut out = format!(
        "wrote valid .tldraw ({} shapes) → Desktop\n  {}\n",
        shapes.len(),
        abs.display()
    );
    match launch_on_file(&abs) {
        Ok(launch) => {
            out.push_str(&launch);
        }
        Err(e) => {
            out.push_str(&format!("open failed: {e}\n"));
            out.push_str(
                "File is on your Desktop — double-click it, or drag onto https://www.tldraw.com/\n",
            );
        }
    }
    Ok(out)
}

/// Build a valid tldraw file for the offline app (v1.11+ / geo schema 11).
///
/// Critical: modern geo shapes use **`richText`**, not plain `text`.
/// Files with `props.text` load as a blank canvas (validation strips shapes).
fn build_tldraw_document(title: &str, shapes: &[Value]) -> Value {
    let mut records: Vec<Value> = vec![
        json!({
            "id": "document:document",
            "typeName": "document",
            "gridSize": 10,
            "name": title,
            "meta": {}
        }),
        json!({
            "id": "page:page",
            "typeName": "page",
            "name": title,
            "index": "a1",
            "meta": {}
        }),
    ];

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for (i, s) in shapes.iter().enumerate() {
        let id = s
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| {
                if s.starts_with("shape:") {
                    s.to_string()
                } else {
                    format!("shape:{s}")
                }
            })
            .unwrap_or_else(|| format!("shape:box{i}"));
        let x = num(s, "x", 80.0 + (i as f64 % 4.0) * 240.0);
        let y = num(s, "y", 80.0 + (i as f64 / 4.0).floor() * 160.0);
        let w = num(s, "w", 200.0).max(40.0);
        let h = num(s, "h", 100.0).max(40.0);
        let text = s
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let color = s
            .get("color")
            .and_then(|v| v.as_str())
            .map(normalize_color)
            .unwrap_or_else(|| "black".into());
        // Dark fills need light labels; light fills need dark labels.
        let label_color = match color.as_str() {
            "white" | "yellow" | "light-blue" | "light-green" | "light-red" | "light-violet" => {
                "black"
            }
            _ => "white",
        };
        let geo = s
            .get("geo")
            .and_then(|v| v.as_str())
            .unwrap_or("rectangle");
        // Must be a valid tldraw IndexKey. "a10" is REJECTED and blanks the
        // whole canvas with ValidationError — use a1..a9, aA..aZ, b1…
        let index = fractional_index(i);

        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);

        records.push(json!({
            "id": id,
            "typeName": "shape",
            "type": "geo",
            "x": x,
            "y": y,
            "rotation": 0,
            "index": index,
            "parentId": "page:page",
            "isLocked": false,
            "opacity": 1,
            "props": {
                "geo": geo,
                "w": w,
                "h": h,
                "growY": 0,
                "richText": to_rich_text(&text),
                "labelColor": label_color,
                "color": color,
                "fill": "solid",
                "dash": "draw",
                "size": "m",
                "font": "draw",
                "align": "middle",
                "verticalAlign": "middle",
                "url": "",
                "scale": 1
            },
            "meta": {}
        }));
    }

    // Session records so the viewport shows the shapes (not a blank far-away camera).
    let (cam_x, cam_y) = if shapes.is_empty() {
        (0.0, 0.0)
    } else {
        (-min_x + 40.0, -min_y + 40.0)
    };
    records.push(json!({
        "id": "camera:page:page",
        "typeName": "camera",
        "x": cam_x,
        "y": cam_y,
        "z": 1,
        "meta": {}
    }));
    records.push(json!({
        "id": "pointer:pointer",
        "typeName": "pointer",
        "x": 0,
        "y": 0,
        "lastActivityTimestamp": 0,
        "meta": {}
    }));
    records.push(json!({
        "id": "instance:instance",
        "typeName": "instance",
        "currentPageId": "page:page",
        "exportBackground": true,
        "isFocusMode": false,
        "isDebugMode": false,
        "isToolLocked": false,
        "isGridMode": false,
        "canMoveCamera": true,
        "isPenMode": false,
        "isReadonly": false,
        "openMenus": [],
        "followingUserId": null,
        "highlightedUserIds": [],
        "brush": null,
        "cursor": { "type": "default", "rotation": 0 },
        "opacityForNextShape": 1,
        "stylesForNextShape": {},
        "meta": {},
        "duplicateProps": null,
        "screenBounds": { "x": 0, "y": 0, "w": 1400, "h": 900 },
        "insets": [false, false, false, false],
        "chatMessage": "",
        "isChatting": false,
        "isFocused": true,
        "devicePixelRatio": 1,
        "isCoarsePointer": false,
        "isHoveringCanvas": null,
        "openDialog": null,
        "isChangingStyle": false,
        "isSnapping": false
    }));
    records.push(json!({
        "id": "instance_page_state:page:page",
        "typeName": "instance_page_state",
        "pageId": "page:page",
        "selectedShapeIds": [],
        "hintingShapeIds": [],
        "erasingShapeIds": [],
        "hoveredShapeId": null,
        "editingShapeId": null,
        "croppingShapeId": null,
        "focusedGroupId": null,
        "meta": {}
    }));

    // Schema versions match tldraw offline 1.11 (geo=11 requires richText).
    json!({
        "tldrawFileFormatVersion": 1,
        "schema": {
            "schemaVersion": 2,
            "sequences": {
                "com.tldraw.store": 5,
                "com.tldraw.asset": 1,
                "com.tldraw.camera": 1,
                "com.tldraw.document": 2,
                "com.tldraw.instance": 26,
                "com.tldraw.instance_page_state": 5,
                "com.tldraw.page": 1,
                "com.tldraw.instance_presence": 6,
                "com.tldraw.pointer": 1,
                "com.tldraw.shape": 4,
                "com.tldraw.user": 1,
                "com.tldraw.asset.bookmark": 2,
                "com.tldraw.asset.image": 6,
                "com.tldraw.asset.video": 5,
                "com.tldraw.shape.arrow": 8,
                "com.tldraw.shape.bookmark": 2,
                "com.tldraw.shape.draw": 5,
                "com.tldraw.shape.embed": 4,
                "com.tldraw.shape.frame": 1,
                "com.tldraw.shape.geo": 11,
                "com.tldraw.shape.group": 0,
                "com.tldraw.shape.highlight": 4,
                "com.tldraw.shape.image": 5,
                "com.tldraw.shape.line": 5,
                "com.tldraw.shape.note": 13,
                "com.tldraw.shape.text": 4,
                "com.tldraw.shape.video": 4,
                "com.tldraw.binding.arrow": 1
            }
        },
        "records": records
    })
}

/// TipTap/ProseMirror doc used by modern tldraw (`toRichText` equivalent).
fn to_rich_text(text: &str) -> Value {
    if text.is_empty() {
        return json!({
            "type": "doc",
            "content": [{ "type": "paragraph" }]
        });
    }
    let content: Vec<Value> = text
        .split('\n')
        .map(|line| {
            if line.is_empty() {
                json!({ "type": "paragraph" })
            } else {
                json!({
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": line }]
                })
            }
        })
        .collect();
    json!({ "type": "doc", "content": content })
}

/// Generate a tldraw-valid fractional index key for shape ordering.
///
/// Plain `a{n}` breaks at 10: `a10` is not a legal IndexKey and the offline
/// app shows a full-screen error ("Something went wrong") with a blank canvas.
fn fractional_index(i: usize) -> String {
    // a1..a9, aA..aZ, b1..b9, bA..bZ, …
    let major = i / 35; // 9 digits + 26 letters
    let minor = i % 35;
    let head = (b'a' + major as u8) as char;
    let tail = if minor < 9 {
        (b'1' + minor as u8) as char
    } else {
        (b'A' + (minor - 9) as u8) as char
    };
    format!("{head}{tail}")
}

fn num(v: &Value, key: &str, default: f64) -> f64 {
    v.get(key)
        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)))
        .unwrap_or(default)
}

fn normalize_color(c: &str) -> String {
    match c.to_ascii_lowercase().as_str() {
        "gray" | "grey" => "grey".into(),
        "gold" | "yellow" => "yellow".into(),
        "purple" | "violet" => "violet".into(),
        "brown" => "orange".into(), // closest built-in
        "teal" | "cyan" => "light-blue".into(),
        "pink" => "light-red".into(),
        other => other.to_string(),
    }
}

fn status_report() -> String {
    let mut s = String::new();
    match app_path() {
        Some(app) => {
            s.push_str(&format!("tldraw offline: INSTALLED\n  {}\n", app.display()));
            s.push_str(&format!("output dir (create): {}\n", desktop_dir().display()));
            s.push_str("open:    tldraw(action=open, path=board.tldraw)  # Desktop or absolute\n");
            s.push_str(
                "create:  tldraw(action=create, title=…, shapes=[{x,y,w,h,text,color}])\n\
                 \t→ always writes to Desktop (valid richText + index keys)\n",
            );
        }
        None => {
            s.push_str("tldraw offline: NOT INSTALLED\n");
            s.push_str(
                "install: tldraw(action=install) — github.com/tldraw/tldraw-offline\n",
            );
        }
    }
    match latest_asset_url() {
        Ok((tag, _)) => s.push_str(&format!("latest release: {tag}\n")),
        Err(_) => s.push_str("latest release: (offline / unknown)\n"),
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_document_has_format_version_and_shapes() {
        let shapes = vec![json!({"x": 10, "y": 20, "w": 100, "h": 50, "text": "A", "color": "blue"})];
        let doc = build_tldraw_document("Test", &shapes);
        assert_eq!(doc["tldrawFileFormatVersion"], 1);
        let records = doc["records"].as_array().unwrap();
        assert!(records.iter().any(|r| r["typeName"] == "document"));
        assert!(records.iter().any(|r| r["typeName"] == "page"));
        assert!(records.iter().any(|r| r["typeName"] == "camera"));
        let shape = records
            .iter()
            .find(|r| r["typeName"] == "shape")
            .expect("shape");
        // Modern geo schema: richText, not props.text
        assert!(shape["props"].get("text").is_none());
        assert_eq!(
            shape["props"]["richText"]["content"][0]["content"][0]["text"],
            "A"
        );
        assert_eq!(doc["schema"]["sequences"]["com.tldraw.shape.geo"], 11);
    }

    #[test]
    fn to_rich_text_splits_lines() {
        let rt = to_rich_text("Hello\nWorld");
        assert_eq!(rt["type"], "doc");
        assert_eq!(rt["content"].as_array().unwrap().len(), 2);
        assert_eq!(rt["content"][1]["content"][0]["text"], "World");
    }

    #[test]
    fn fractional_index_never_emits_a10() {
        let idxs: Vec<String> = (0..40).map(fractional_index).collect();
        assert_eq!(idxs[0], "a1");
        assert_eq!(idxs[8], "a9");
        assert_eq!(idxs[9], "aA");
        assert!(!idxs.iter().any(|s| s == "a10" || s == "a11"));
        // all unique
        let mut sorted = idxs.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), idxs.len());
    }

    #[test]
    fn validate_detects_fake_schema() {
        let dir = std::env::temp_dir().join(format!("nur-tldraw-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("fake.tldraw");
        std::fs::write(&p, r#"{"schemaVersion":30,"store":{}}"#).unwrap();
        let hint = validate_or_hint(&p);
        assert!(hint.contains("NOT a valid"), "{hint}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn create_path_lands_on_desktop() {
        let args = json!({"title": "Car Meet Parking", "path": "subdir/foo.tldraw"});
        let p = resolve_create_path(&args).unwrap();
        assert_eq!(p.parent().unwrap(), desktop_dir());
        assert_eq!(p.file_name().unwrap(), "foo.tldraw");
    }

    #[test]
    fn create_path_from_title_slug() {
        let args = json!({"title": "My Cool Board!"});
        let p = resolve_create_path(&args).unwrap();
        assert_eq!(p.parent().unwrap(), desktop_dir());
        let name = p.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with(".tldraw"), "{name}");
        assert!(name.contains("my-cool-board"), "{name}");
    }
}
