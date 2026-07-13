# Security

Meta CLI is **unofficial** community software. It is not affiliated with Meta Platforms, Inc.

## Where secrets live

| Location | Contents |
|----------|----------|
| `~/.meta/auth.json` | Meta Model API key after `meta auth login` |
| Env `META_API_KEY` / `MODEL_API_KEY` | Optional override (never printed in logs) |
| `~/.meta/sessions/` | Session metadata (no key) |
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

---

## Install script safety

`install.ps1` / `install.sh`:

- May **read** a key already present in your environment and store it under `~/.meta/` on your machine
- Do **not** write keys into the git checkout or GitHub
- Verify **SHA-256** of the installed binary

---

## Binary integrity

Each release includes a SHA-256 hash written next to the binary by the install script. `meta doctor` verifies this:

```bash
meta doctor
# should show: sha256  <hash>  (matches install record)
```

---

## Reporting vulnerabilities

Open a private report or issue on [nuroctane/meta-cli](https://github.com/nuroctane/meta-cli) if you find a vulnerability in this client.
