# Meta CLI (unofficial)

**Unofficial** terminal coding agent for [Muse Spark](https://ai.meta.com/blog/introducing-muse-spark-meta-model-api/) via [Meta Model API](https://dev.meta.ai/).

> Not affiliated with Meta Platforms, Inc. · Community project · [nuroctane/meta-cli](https://github.com/nuroctane/meta-cli)

The command you run is **`muse`**.

---

## Install (one shot)

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.ps1 | iex
```

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.sh | bash
```

That single command will:

1. Install Rust if needed  
2. Clone this repo (or update it)  
3. `cargo build --release`  
4. Put `muse` on your PATH (`~/.local/bin`)  
5. Install the Orca ADE hook when possible  
6. If `MODEL_API_KEY` is already set, save auth under `~/.muse/` **on your machine only**

Then:

```powershell
muse auth login    # paste your Meta Model API key (stored only in ~/.muse)
muse               # open the TUI
```

Get a key: [dev.meta.ai](https://dev.meta.ai/) → API keys.

### Already cloned?

```powershell
cd meta-cli
.\install.ps1          # Windows
# ./install.sh         # macOS / Linux
```

### Windows: Laboratory clone script

```text
C:\Users\david\Scripts\clone meta-cli main to Laboratory local.cmd
```

Then `cd` into the folder and run `.\install.ps1`.

---

## Secrets (important)

| On GitHub | On your PC only |
|-----------|-----------------|
| Source code, README, install scripts | `~/.muse/auth.json` (API key) |
| No API keys, no `.env`, no sessions | `~/.muse/sessions/`, usage logs |

`.gitignore` excludes `.env*`, `auth.json`, `.muse/`, and common key material.  
See [SECURITY.md](./SECURITY.md).

**Never commit your Meta API key. Never paste it into issues or PRs.**

---

## Quick use

```
muse                      # interactive Meta-blue TUI
muse "fix the bug"       # start with a prompt
muse -c                   # continue last session in this directory
muse -r <session-id>      # resume a session
muse run "…" -y           # headless agent turn (streams; auto-approve tools)
muse sessions
muse usage                # token / cost for ADEs
muse auth status
```

### TUI highlights

- Live streaming · tool approvals (`y` / `a` / `n`) · slash commands (`/help`)  
- Markdown · multi-line input · usage statusline · Esc interrupts  
- Project instructions from `MUSE.md`, `AGENTS.md`, or `CLAUDE.md`  
- `/init` generates a `MUSE.md`

---

## ADE / Orca

Usage is written for host tools (never includes your API key):

| Path | Purpose |
|------|---------|
| `~/.muse/status.json` | Live tokens / est. USD / model / state |
| `~/.muse/usage.jsonl` | Per-request log |
| `~/.muse/ade.json` | Discovery manifest |

```powershell
muse install-hook
orca terminal create --worktree active --command "muse" --title "Meta CLI" --json
```

---

## Config

`~/.muse/config.toml` (created on first run):

```toml
model = "muse-spark-1.1"
base_url = "https://api.meta.ai/v1"
reasoning_effort = "high"
max_turns = 40
stream = true
context_window = 1000000
```

## Tools

`read_file` · `write_file` · `edit_file` · `bash` · `grep` · `glob`

## Model API

Responses API (`POST /v1/responses`), `muse-spark-1.1`, streaming + reasoning continuity.  
Docs: https://dev.meta.ai/docs/getting-started/overview

## License

MIT — unofficial community software; not a Meta product.
