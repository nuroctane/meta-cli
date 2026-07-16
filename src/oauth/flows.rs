//! Per-provider browser / device-code / external-CLI login flows.

use super::{expires_in_to_at, open_browser, CancelFlag};
use crate::auth::{Auth, OauthMeta};
use crate::error::{MuseError, Result};
use serde::Deserialize;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::sync::mpsc::Sender;
use std::time::Duration;

/// Tokens returned by a successful browser login.
#[derive(Debug, Clone)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub meta: Option<OauthMeta>,
}

/// Progress events for the TUI browser stage (Hugging Face–style URL + code).
#[derive(Debug, Clone)]
pub enum BrowserLoginProgress {
    Status(String),
    /// Device-code style: open this URL and enter the short code.
    DeviceCode {
        verification_url: String,
        user_code: String,
    },
    /// Loopback / SSO: browser opened (or open this URL).
    OpenUrl(String),
    Done(OAuthTokens),
    Failed(String),
}

pub type ProgressTx = Sender<BrowserLoginProgress>;

fn send(tx: &ProgressTx, ev: BrowserLoginProgress) {
    let _ = tx.send(ev);
}

/// Run browser login for `provider_id` on a background-friendly thread path.
/// Blocks until success, failure, cancel, or timeout.
pub fn login_browser(provider_id: &str, tx: ProgressTx, cancel: CancelFlag) {
    let result = match provider_id {
        "antigravity" => antigravity::login(&tx, &cancel),
        "huggingface" => huggingface::login(&tx, &cancel),
        "azure" => azure::login(&tx, &cancel),
        "bedrock" => bedrock::login(&tx, &cancel),
        "github-models" => github::login(&tx, &cancel),
        other => Err(MuseError::Other(api_key_only_reason(other))),
    };
    // Do not persist here — the TUI decides active login vs failover-only
    // storage so a `/failover` browser capture never overwrites auth.json.
    match result {
        Ok(tokens) => send(&tx, BrowserLoginProgress::Done(tokens)),
        Err(e) => send(&tx, BrowserLoginProgress::Failed(e.to_string())),
    }
}

/// Providers whose vendor gates model access to their own first-party CLI.
///
/// These have no third-party OAuth client we can register, so `/login` routes
/// straight to an API key instead of opening a browser flow that the vendor
/// will reject at the authorize step.
pub fn api_key_only_reason(provider_id: &str) -> String {
    let vendor = match provider_id {
        "openai" => Some(("OpenAI", "https://platform.openai.com/api-keys")),
        "anthropic" => Some(("Anthropic", "https://console.anthropic.com/settings/keys")),
        "xai" => Some(("xAI", "https://console.x.ai")),
        "kimi" => Some(("Kimi", "https://platform.moonshot.cn/console/api-keys")),
        _ => None,
    };
    match vendor {
        Some((name, url)) => format!(
            "{name} does not offer OAuth sign-in to third-party CLIs — use an API key instead: {url}"
        ),
        None => format!("browser login not supported for '{provider_id}'"),
    }
}

fn http() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent(format!("nur-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| MuseError::Other(e.to_string()))
}


// ── Google Antigravity (browser SSO via gcloud — no embedded OAuth secrets) ─

pub mod antigravity {
    use super::*;

    /// Browser sign-in through the official Google Cloud SDK (`gcloud auth login`),
    /// then mint an access token for API calls. No OAuth client secrets ship in-repo
    /// (GitHub push protection). Users without gcloud can still paste a Gemini API key.
    pub fn login(tx: &ProgressTx, cancel: &CancelFlag) -> Result<OAuthTokens> {
        // Already signed in?
        if let Ok(t) = fetch_access_token() {
            send(
                tx,
                BrowserLoginProgress::Status("using existing gcloud session".into()),
            );
            return Ok(t);
        }
        send(
            tx,
            BrowserLoginProgress::Status(
                "launching Google browser login (gcloud auth login)…".into(),
            ),
        );
        send(
            tx,
            BrowserLoginProgress::OpenUrl("https://accounts.google.com/".into()),
        );
        let mut child = Command::new("gcloud")
            .args([
                "auth",
                "login",
                "--brief",
                "--update-adc",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                MuseError::Other(format!(
                    "gcloud not found ({e}). Install Google Cloud SDK, or choose “Enter API key” with a Gemini key."
                ))
            })?;
        // Surface any https URL from gcloud stderr (device / browser flow).
        if let Some(mut err) = child.stderr.take() {
            let tx2 = tx.clone();
            thread::spawn(move || {
                let mut buf = String::new();
                let _ = err.read_to_string(&mut buf);
                for word in buf.split_whitespace() {
                    if word.starts_with("https://") {
                        send(&tx2, BrowserLoginProgress::OpenUrl(word.to_string()));
                        let _ = open_browser(word);
                        break;
                    }
                }
                // Device-code style lines from older gcloud
                if buf.contains("enter the code") || buf.contains("verification code") {
                    send(
                        &tx2,
                        BrowserLoginProgress::Status(buf.chars().take(240).collect()),
                    );
                }
            });
        }
        loop {
            if cancel.is_cancelled() {
                let _ = child.kill();
                return Err(MuseError::Other("login cancelled".into()));
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(status)) => {
                    return Err(MuseError::Other(format!(
                        "gcloud auth login failed (exit {status}). Paste a Gemini API key as fallback."
                    )))
                }
                Ok(None) => thread::sleep(Duration::from_millis(200)),
                Err(e) => return Err(MuseError::Other(e.to_string())),
            }
        }
        send(
            tx,
            BrowserLoginProgress::Status("fetching Google access token…".into()),
        );
        fetch_access_token()
    }

    fn fetch_access_token() -> Result<OAuthTokens> {
        let out = Command::new("gcloud")
            .args(["auth", "application-default", "print-access-token"])
            .output()
            .map_err(|e| MuseError::Other(format!("gcloud ADC print-access-token: {e}")))?;
        if !out.status.success() {
            return Err(MuseError::Other(format!(
                "gcloud application-default print-access-token failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        let access = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if access.is_empty() {
            return Err(MuseError::Other("empty token from gcloud".into()));
        }
        let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                let out = Command::new("gcloud")
                    .args(["config", "get-value", "project"])
                    .output()
                    .ok()?;
                if !out.status.success() {
                    return None;
                }
                let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
                (!value.is_empty() && value != "(unset)").then_some(value)
            })
            .ok_or_else(|| {
                MuseError::Other(
                    "Google OAuth needs a quota project. Run `gcloud config set project PROJECT_ID` or set GOOGLE_CLOUD_PROJECT, then retry /login."
                        .into(),
                )
            })?;
        Ok(OAuthTokens {
            access_token: access,
            // Marker so ensure_fresh_oauth can re-call gcloud.
            refresh_token: Some("gcloud".into()),
            expires_at: Some(super::super::now_unix() + 3300),
            meta: Some(OauthMeta {
                issuer: "https://accounts.google.com".into(),
                client_id: "gcloud".into(),
                extra: serde_json::json!({
                    "product": "antigravity",
                    "via": "gcloud application-default login",
                    "project_id": project_id,
                }),
            }),
        })
    }

    pub fn refresh(_auth: &Auth, _refresh: &str) -> Result<OAuthTokens> {
        fetch_access_token()
    }
}

// ── GitHub Models (browser SSO via the official `gh` CLI) ───────────────────

pub mod github {
    use super::*;

    /// Sign in through the official GitHub CLI (`gh auth login --web`), then mint
    /// a token for GitHub Models. No OAuth client secrets ship in-repo. If `gh`
    /// is already authenticated, the existing session is reused. Users without
    /// `gh` can still paste a GitHub PAT (with `models:read`) via "Enter API key".
    pub fn login(tx: &ProgressTx, cancel: &CancelFlag) -> Result<OAuthTokens> {
        // Already signed in? Reuse the existing gh token.
        if let Ok(t) = fetch_token() {
            send(
                tx,
                BrowserLoginProgress::Status("using existing GitHub CLI session".into()),
            );
            return Ok(t);
        }
        send(
            tx,
            BrowserLoginProgress::Status("launching GitHub browser login (gh auth login)…".into()),
        );
        send(
            tx,
            BrowserLoginProgress::OpenUrl("https://github.com/login/device".into()),
        );
        // `--web` opens the device flow; feed newlines so the "press Enter to
        // open the browser" prompt proceeds without a TTY.
        let mut child = Command::new("gh")
            .args([
                "auth",
                "login",
                "--web",
                "--hostname",
                "github.com",
                "--git-protocol",
                "https",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                MuseError::Other(format!(
                    "gh not found ({e}). Install GitHub CLI, or choose “Enter API key” with a GitHub PAT (models:read)."
                ))
            })?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(b"\n\n");
        }
        // Surface the one-time code + verification URL from gh's stderr.
        if let Some(mut err) = child.stderr.take() {
            let tx2 = tx.clone();
            thread::spawn(move || {
                let mut buf = String::new();
                let _ = err.read_to_string(&mut buf);
                for word in buf.split_whitespace() {
                    if word.starts_with("https://") {
                        send(&tx2, BrowserLoginProgress::OpenUrl(word.to_string()));
                        let _ = open_browser(word);
                        break;
                    }
                }
                if let Some(idx) = buf.find("one-time code:") {
                    let code: String = buf[idx..].chars().take(40).collect();
                    send(&tx2, BrowserLoginProgress::Status(code));
                }
            });
        }
        loop {
            if cancel.is_cancelled() {
                let _ = child.kill();
                return Err(MuseError::Other("login cancelled".into()));
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(status)) => {
                    return Err(MuseError::Other(format!(
                        "gh auth login failed (exit {status}). Paste a GitHub PAT (models:read) as fallback."
                    )))
                }
                Ok(None) => thread::sleep(Duration::from_millis(200)),
                Err(e) => return Err(MuseError::Other(e.to_string())),
            }
        }
        send(tx, BrowserLoginProgress::Status("fetching GitHub token…".into()));
        fetch_token()
    }

    fn fetch_token() -> Result<OAuthTokens> {
        let out = Command::new("gh")
            .args(["auth", "token", "--hostname", "github.com"])
            .output()
            .map_err(|e| MuseError::Other(format!("gh auth token: {e}")))?;
        if !out.status.success() {
            return Err(MuseError::Other(format!(
                "gh auth token failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        let access = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if access.is_empty() {
            return Err(MuseError::Other("empty token from gh".into()));
        }
        Ok(OAuthTokens {
            access_token: access,
            // Marker so ensure_fresh_oauth can re-call `gh auth token`.
            refresh_token: Some("gh".into()),
            // gh manages token lifetime; re-fetch opportunistically.
            expires_at: None,
            meta: Some(OauthMeta {
                issuer: "https://github.com".into(),
                client_id: "gh".into(),
                extra: serde_json::json!({"product": "github-models", "via": "gh auth login"}),
            }),
        })
    }

    pub fn refresh(_auth: &Auth, _refresh: &str) -> Result<OAuthTokens> {
        fetch_token()
    }
}

// ── Hugging Face (device code — same spirit as `hf auth login`) ────────────

pub mod huggingface {
    use super::*;

    #[derive(Deserialize)]
    struct DeviceCodeResp {
        #[serde(default)]
        device_code: String,
        #[serde(default)]
        user_code: String,
        #[serde(default)]
        verification_uri: String,
        #[serde(default)]
        verification_uri_complete: Option<String>,
        #[serde(default)]
        expires_in: u64,
        #[serde(default = "default_interval")]
        interval: u64,
        // Some HF endpoints nest under different shapes.
        #[serde(default)]
        #[allow(dead_code)]
        request_id: Option<String>,
    }
    fn default_interval() -> u64 {
        5
    }

    #[derive(Deserialize)]
    struct TokenResp {
        access_token: Option<String>,
        refresh_token: Option<String>,
        expires_in: Option<u64>,
        error: Option<String>,
        #[allow(dead_code)]
        error_description: Option<String>,
        // HF classic: {"token":"..."}
        token: Option<String>,
    }

    pub fn login(tx: &ProgressTx, cancel: &CancelFlag) -> Result<OAuthTokens> {
        send(
            tx,
            BrowserLoginProgress::Status("starting Hugging Face device login…".into()),
        );
        let client = http()?;

        // Try OAuth device flow; fall back to token page + poll is not available —
        // fall back to opening token settings and asking user to paste is Key path.
        let device_endpoints = [
            "https://huggingface.co/oauth/device/code",
            "https://huggingface.co/api/oauth/device/code",
        ];
        let mut device: Option<DeviceCodeResp> = None;
        let mut last = String::new();
        for url in device_endpoints {
            // Public HF OAuth app client used by huggingface_hub (community-known).
            let form = [
                ("client_id", "85c97818-78c2-455a-9472-9a0f2e8a1b0d"),
                ("scope", "openid profile email"),
            ];
            match client.post(url).form(&form).send() {
                Ok(res) => {
                    let status = res.status();
                    let body = res.text().unwrap_or_default();
                    if status.is_success() {
                        if let Ok(d) = serde_json::from_str::<DeviceCodeResp>(&body) {
                            if !d.user_code.is_empty() || !d.device_code.is_empty() {
                                device = Some(d);
                                break;
                            }
                        }
                        last = body;
                    } else {
                        last = format!("{status}: {body}");
                    }
                }
                Err(e) => last = e.to_string(),
            }
        }

        if let Some(device) = device {
            let verify = device
                .verification_uri_complete
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    if device.verification_uri.is_empty() {
                        format!(
                            "https://huggingface.co/login/device?user_code={}",
                            device.user_code
                        )
                    } else {
                        device.verification_uri.clone()
                    }
                });
            send(
                tx,
                BrowserLoginProgress::DeviceCode {
                    verification_url: verify.clone(),
                    user_code: device.user_code.clone(),
                },
            );
            let _ = open_browser(&verify);

            let deadline = std::time::Instant::now()
                + Duration::from_secs(if device.expires_in > 0 {
                    device.expires_in
                } else {
                    900
                });
            let base_interval = device.interval.max(3);
            let mut attempt = 0u32;
            let mut slow = false;
            while std::time::Instant::now() < deadline {
                if cancel.is_cancelled() {
                    return Err(MuseError::Other("login cancelled".into()));
                }
                thread::sleep(crate::oauth::device_poll_sleep(base_interval, slow, attempt));
                attempt = attempt.saturating_add(1);
                slow = false;
                let form = [
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", device.device_code.as_str()),
                    ("client_id", "85c97818-78c2-455a-9472-9a0f2e8a1b0d"),
                ];
                for turl in [
                    "https://huggingface.co/oauth/token",
                    "https://huggingface.co/api/oauth/token",
                ] {
                    let Ok(res) = client.post(turl).form(&form).send() else {
                        continue;
                    };
                    let body = res.text().unwrap_or_default();
                    let parsed: TokenResp = serde_json::from_str(&body).unwrap_or(TokenResp {
                        access_token: None,
                        refresh_token: None,
                        expires_in: None,
                        error: Some("pending".into()),
                        error_description: None,
                        token: None,
                    });
                    if let Some(err) = parsed.error.as_deref() {
                        if err == "authorization_pending" || err == "pending" {
                            continue;
                        }
                        if err == "slow_down" {
                            slow = true;
                            continue;
                        }
                    }
                    if let Some(access) = parsed.access_token.or(parsed.token) {
                        return Ok(OAuthTokens {
                            access_token: access,
                            refresh_token: parsed.refresh_token,
                            expires_at: expires_in_to_at(parsed.expires_in),
                            meta: Some(OauthMeta {
                                issuer: "https://huggingface.co".into(),
                                client_id: "huggingface".into(),
                                extra: serde_json::json!({}),
                            }),
                        });
                    }
                }
                send(
                    tx,
                    BrowserLoginProgress::Status("waiting for Hugging Face approval…".into()),
                );
            }
            return Err(MuseError::Other("Hugging Face login timed out".into()));
        }

        // Fallback: open token page and instruct user to use API key path.
        let url = "https://huggingface.co/settings/tokens";
        send(tx, BrowserLoginProgress::OpenUrl(url.into()));
        Err(MuseError::Other(format!(
            "HF device flow unavailable ({last}). Open {url}, create a token, and choose “Enter API key” in /login."
        )))
    }

    pub fn refresh(_refresh: &str) -> Result<OAuthTokens> {
        Err(MuseError::Other(
            "Hugging Face token refresh not available — re-run browser login or paste HF_TOKEN"
                .into(),
        ))
    }
}

// ── Azure OpenAI (Entra via `az login`, like Azure CLI) ────────────────────

pub mod azure {
    use super::*;

    pub fn login(tx: &ProgressTx, cancel: &CancelFlag) -> Result<OAuthTokens> {
        // If already logged in, just mint a token.
        if let Ok(t) = fetch_token() {
            send(
                tx,
                BrowserLoginProgress::Status("using existing Azure CLI session".into()),
            );
            return Ok(t);
        }
        send(
            tx,
            BrowserLoginProgress::Status("launching Azure device login (az login)…".into()),
        );
        send(
            tx,
            BrowserLoginProgress::DeviceCode {
                verification_url: "https://microsoft.com/devicelogin".into(),
                user_code: "(see az output — opening browser)".into(),
            },
        );
        let _ = open_browser("https://microsoft.com/devicelogin");
        let mut child = Command::new("az")
            .args(["login", "--use-device-code"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                MuseError::Other(format!(
                    "Azure CLI not found ({e}). Install `az` or paste AZURE_OPENAI_API_KEY."
                ))
            })?;
        // Best-effort parse device code from az stderr/stdout while waiting.
        let stderr = child.stderr.take();
        if let Some(mut err) = stderr {
            let tx2 = tx.clone();
            thread::spawn(move || {
                let mut buf = String::new();
                let _ = err.read_to_string(&mut buf);
                // az prints: To sign in, use a web browser to open the page https://microsoft.com/devicelogin
                // and enter the code XXXXXXXXX
                let url = "https://microsoft.com/devicelogin";
                let code = buf
                    .split_whitespace()
                    .find(|w| {
                        w.len() >= 8
                            && w.len() <= 15
                            && w.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                            && w.contains(|c: char| c.is_ascii_uppercase())
                    })
                    .unwrap_or("")
                    .to_string();
                if !code.is_empty() {
                    send(
                        &tx2,
                        BrowserLoginProgress::DeviceCode {
                            verification_url: url.into(),
                            user_code: code,
                        },
                    );
                }
            });
        }
        loop {
            if cancel.is_cancelled() {
                let _ = child.kill();
                return Err(MuseError::Other("login cancelled".into()));
            }
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(status)) => {
                    return Err(MuseError::Other(format!(
                        "az login failed (exit {status}). Paste AZURE_OPENAI_API_KEY as fallback."
                    )))
                }
                Ok(None) => thread::sleep(Duration::from_millis(200)),
                Err(e) => return Err(MuseError::Other(e.to_string())),
            }
        }
        send(
            tx,
            BrowserLoginProgress::Status("fetching Cognitive Services token…".into()),
        );
        fetch_token()
    }

    fn fetch_token() -> Result<OAuthTokens> {
        let out = Command::new("az")
            .args([
                "account",
                "get-access-token",
                "--resource",
                "https://cognitiveservices.azure.com",
                "-o",
                "json",
            ])
            .output()
            .map_err(|e| {
                MuseError::Other(format!(
                    "Azure CLI not available ({e}). Install `az`, run `az login`, or paste AZURE_OPENAI_API_KEY."
                ))
            })?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(MuseError::Other(format!(
                "az get-access-token failed: {err}. Fix: `az login` then retry, or paste AZURE_OPENAI_API_KEY in /login."
            )));
        }
        // Prefer structured JSON (stable az contract).
        #[derive(Deserialize)]
        struct AzToken {
            #[serde(rename = "accessToken")]
            access_token: Option<String>,
            #[serde(rename = "expiresOn")]
            expires_on: Option<String>,
            #[serde(default)]
            expires_on_ts: Option<String>,
        }
        let parsed: AzToken = serde_json::from_slice(&out.stdout).map_err(|e| {
            MuseError::Other(format!(
                "could not parse az JSON token output ({e}). Update Azure CLI or use API key path."
            ))
        })?;
        let access = parsed
            .access_token
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                MuseError::Other(
                    "az returned empty accessToken. Run `az login` or paste AZURE_OPENAI_API_KEY."
                        .into(),
                )
            })?;
        let expires_at = parsed
            .expires_on
            .as_deref()
            .and_then(|s| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                    .ok()
            })
            .map(|ndt| ndt.and_utc().timestamp() as u64)
            .or_else(|| {
                parsed
                    .expires_on_ts
                    .as_deref()
                    .and_then(|s| s.parse().ok())
            });
        Ok(OAuthTokens {
            access_token: access,
            refresh_token: Some("az-cli".into()),
            expires_at,
            meta: Some(OauthMeta {
                issuer: "https://login.microsoftonline.com".into(),
                client_id: "azure-cli".into(),
                extra: serde_json::json!({"via": "az login"}),
            }),
        })
    }

    pub fn refresh() -> Result<OAuthTokens> {
        fetch_token()
    }
}

// ── AWS Bedrock (IAM Identity Center via `aws sso login`) ──────────────────

pub mod bedrock {
    use super::*;

    pub fn login(tx: &ProgressTx, cancel: &CancelFlag) -> Result<OAuthTokens> {
        send(
            tx,
            BrowserLoginProgress::Status("launching AWS SSO login (aws sso login)…".into()),
        );
        send(
            tx,
            BrowserLoginProgress::Status(
                "complete browser SSO when prompted by the AWS CLI…".into(),
            ),
        );
        // Prefer sso login; fall back to `aws login` if present.
        let mut ok = false;
        let mut last = String::new();
        for args in [vec!["sso", "login"], vec!["login"]] {
            let mut child = match Command::new("aws")
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    last = format!("aws not found: {e}");
                    continue;
                }
            };
            // AWS SSO prints a URL — try to surface it.
            if let Some(mut err) = child.stderr.take() {
                let tx2 = tx.clone();
                thread::spawn(move || {
                    let mut buf = String::new();
                    let _ = err.read_to_string(&mut buf);
                    for word in buf.split_whitespace() {
                        if word.starts_with("https://") {
                            send(&tx2, BrowserLoginProgress::OpenUrl(word.to_string()));
                            let _ = open_browser(word);
                            break;
                        }
                    }
                    if buf.to_lowercase().contains("user code") || buf.contains("enter the code")
                    {
                        send(
                            &tx2,
                            BrowserLoginProgress::Status(buf.chars().take(200).collect()),
                        );
                    }
                });
            }
            loop {
                if cancel.is_cancelled() {
                    let _ = child.kill();
                    return Err(MuseError::Other("login cancelled".into()));
                }
                match child.try_wait() {
                    Ok(Some(s)) if s.success() => {
                        ok = true;
                        break;
                    }
                    Ok(Some(s)) => {
                        last = format!("aws {} exit {s}", args.join(" "));
                        break;
                    }
                    Ok(None) => thread::sleep(Duration::from_millis(200)),
                    Err(e) => {
                        last = e.to_string();
                        break;
                    }
                }
            }
            if ok {
                break;
            }
        }
        if !ok {
            return Err(MuseError::Other(format!(
                "AWS SSO login failed ({last}). Install AWS CLI v2, configure SSO, or paste a bearer/token if you use a Bedrock gateway."
            )));
        }

        // AWS SSO credentials are SigV4 material, not Bedrock bearer tokens. Nur's
        // OpenAI-compatible HTTP path can only use an actual Bedrock API key/token.
        // Never persist an access-key marker as a bearer: it makes login appear
        // successful and guarantees every subsequent request will be rejected.
        send(
            tx,
            BrowserLoginProgress::Status("checking for a Bedrock bearer token…".into()),
        );
        if let Ok(token) = std::env::var("AWS_BEARER_TOKEN_BEDROCK") {
            if !token.is_empty() {
                return Ok(OAuthTokens {
                    access_token: token,
                    refresh_token: Some("aws-sso".into()),
                    expires_at: Some(super::super::now_unix() + 3600),
                    meta: Some(OauthMeta {
                        issuer: "aws-sso".into(),
                        client_id: "aws-cli".into(),
                        extra: serde_json::json!({"via": "env AWS_BEARER_TOKEN_BEDROCK"}),
                    }),
                });
            }
        }

        Err(MuseError::Other(
            "AWS SSO completed, but SSO credentials require SigV4 and cannot be sent as a bearer token. Generate a short-term Bedrock API key, set AWS_BEARER_TOKEN_BEDROCK, then retry /login; or paste a Bedrock API key. The AWS CLI SSO session remains active."
                .into(),
        ))
    }

    pub fn refresh() -> Result<OAuthTokens> {
        if let Ok(token) = std::env::var("AWS_BEARER_TOKEN_BEDROCK") {
            if !token.is_empty() {
                return Ok(OAuthTokens {
                    access_token: token,
                    refresh_token: Some("aws-sso".into()),
                    expires_at: Some(super::super::now_unix() + 3600),
                    meta: None,
                });
            }
        }
        Err(MuseError::Other(
            "AWS Bedrock refresh: re-run /login browser (aws sso login)".into(),
        ))
    }
}
