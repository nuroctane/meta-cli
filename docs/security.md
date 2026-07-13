# Security

Meta CLI is **unofficial** community software. It is not affiliated with Meta Platforms, Inc.

## Where secrets live

| Location | Contents |
|----------|----------|
| `~/.meta/auth.json` | Meta Model API key after `meta auth login` |
| Env `META_API_KEY` / `MODEL_API_KEY` | Optional override (never printed in logs) |
| `~/.meta/sessions/` | Session files + `.json.bak` / `.precompact.bak` (no key) |
| `~/.meta/tool-results/` | Spilled large tool outputs (may include workspace text) |
| `~/.meta/meta.log` | Tracing log (not the terminal; may include paths) |
| `~/.meta/status.json` | Live token usage (no key) |
| `~/.meta/usage.jsonl` | Per-request usage log (no key) |
| Workspace `.meta/frames/` | Extracted video keyframes (local; may be large) |

---

## What is never committed

- `~/.meta/` directory
- `.env` files with keys
- Session dumps
- Workspace `.meta/frames/` dumps of sensitive UI

!!! warning "Session sensitivity"
    Session `input_items` may include base64 media when vision (`look` / auto-attach) is used — treat session files as potentially sensitive.

---

## Sandbox

Meta CLI hardens shell execution by default:

- **Bash denylist** — blocks dangerous commands
- **Timeout** — long-running commands are killed
- **SSRF blocks** — web tools reject private-IP targets
- **Atomic IO** — all writes to `~/.meta/` use atomic file operations (write-to-temp, rename)
- **Session bak** — each session save copies the previous file to `*.json.bak` first
- **Optional rules** — `permissions.toml` deny/ask/allow; plan mode still blocks code authoring / VCS
- **Optional hooks** — `hooks.toml` pre/post tool shell (local only; you control the script)

---

## Cost controls

- `/budget` and `max_session_cost_usd` / `max_session_tokens` hard-stop new API turns
- Oversized tool results spill to disk instead of re-entering context forever
- `/poor` reduces prompt bulk without removing tools

---

## Install safety

`install.ps1` / `install.sh` / release **EXE** (`meta install`):

- May **read** a key already present in your environment and store it under `~/.meta/` on your machine
- Do **not** write keys into the git checkout or GitHub
- Write the binary to `~/.local/bin` and verify **SHA-256** of the installed binary
- Best-effort prereq installs (Node, uv, …) are local to your machine

---

## Binary integrity

Each release includes a SHA-256 hash written next to the binary by the installer (one-liner or EXE). `meta doctor` verifies this:

```bash
meta doctor
# should show: sha256  <hash>  (matches install record)
```

---

## Reporting vulnerabilities

Open a private report or issue on [nuroctane/meta-cli](https://github.com/nuroctane/meta-cli) if you find a vulnerability in this client.
