# Commands

Full CLI reference for NurCLI.

## Usage

```bash
meta [OPTIONS] [PROMPT]
meta <COMMAND> [ARGS]
```

## Global options

| Flag | Short | Description |
|------|-------|-------------|
| `--model <MODEL>` | `-m` | Meta Model API model id (default from config) |
| `--cwd <DIR>` | | Working directory |
| `--yes` | `-y` | Auto-approve tools (sets permission mode to auto) |
| `--mode <MODE>` | | Permission mode: `manual`, `plan`, or `auto` |
| `--effort <LEVEL>` | | Reasoning effort: `minimal`, `low`, `medium`, `high`, `xhigh` |
| `--max-turns <N>` | | Max agent turns per prompt |
| `--verbose` | `-v` | Verbose tool logging (headless mode) |
| `--continue` | `-c` | Continue the most recent session for this cwd |
| `--resume <ID>` | `-r` | Resume a specific session id (full UUID or unique prefix) |
| `--version` | | Print version |
| `--help` | `-h` | Print help |

## Examples

```bash
meta                                    # open interactive TUI
nur install                            # one-stop stack install (same as release EXE)
meta "fix the bug"                      # start with a prompt
meta "design from ref.mp4"             # vision: auto-attach media
meta -c                                 # continue last session
meta -r abc123                          # resume session abc123
nur --mode plan "explain this"         # plan: explore + shell, no edits/commits
nur --effort xhigh "deep analysis"     # maximum reasoning
nur --model muse-spark-1.1 "hello"     # explicit model
nur run "add tests" -y                 # headless + auto-approve
```

---

## Subcommands

### `nur run`

Run a single agent turn headlessly. Prints the final answer to stdout.

```bash
nur run <PROMPT...> [OPTIONS]
```

| Arg / Flag | Description |
|------------|-------------|
| `PROMPT` | Prompt text (required, multiple words joined) |
| `-y`, `--yes` | Auto-approve all tools |

**Example:**

```bash
nur run "write a hello world in Rust" -y
nur run "explain what this repo does" -v
```

---

### `nur auth`

Manage the stored API key (`~/.nur/auth.json`).

For **multi-provider** sign-in (pick OpenAI, OpenRouter, Ollama, … + endpoint
and default model), use the TUI slash command **`/login`** — see
[Authentication](authentication.md). CLI `nur auth login` is the classic
prompt for a Meta Model API key.

```bash
nur auth <SUBCOMMAND>
```

#### `nur auth login`

Save API key to `~/.nur/auth.json`.

```bash
nur auth login [--key <KEY>]
```

| Flag | Description |
|------|-------------|
| `--key <KEY>` | API key (optional; prompts if omitted) |

#### `nur auth status`

Show auth status. Never prints the full key.

```bash
nur auth status
```

#### `nur auth logout`

Remove saved key from `~/.nur/auth.json` and legacy `~/.muse/`.

```bash
nur auth logout
nur auth logout --revoke   # local delete + best-effort revoke notes for az/aws/gcloud
```

---

### `nur sessions`

List recent sessions (prompt-first summaries from `~/.nur/sessions` and legacy `~/.muse/sessions`).

```bash
nur sessions [--limit <N>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--limit` | `20` | Max rows to display (`0` = all) |

Columns: **ID · UPDATED · MSGS · TOKENS · COST · CWD**.

---

### `meta usage`

Show last known token usage and cost. Displays paths to status and usage log files.

```bash
meta usage
```

---

### `nur install`

One-stop install — **same job as the release EXE and the shell one-liners** (minus compiling from source): copy binary → PATH → prereqs (best-effort) → ecosystem ensure → browser stage → Orca hook → optional auth from env. **No TUI** until this finishes (or until you open `nur` afterward).

```bash
nur install
# alias:
meta self-install
```

Double-clicking `nur-windows-x86_64.exe` from [Releases](https://github.com/nuroctane/nur-cli/releases/latest) runs this path automatically, then opens Meta.

### `nur update`

**How you upgrade Meta.** Pull latest source, rebuild release, reinstall binary + full stack.

```bash
nur update
```

| Step | Action |
|------|--------|
| Source | Uses `~/laboratory/nur-cli` or `~/Laboratory/nur-cli` if present |
| Git | `git pull --ff-only origin main` |
| Build | `cargo build --release` |
| Binary | Installs to `~/.local/bin/meta` (+ `muse`) |
| Stack | `ecosystem ensure --force`, `browser setup`, Orca hook |
| No checkout | Falls back to `nur install` (repair from the running binary) |

Afterward: `nur --version` · `nur doctor`.

Full paths and alternatives (one-liner / EXE / `nur install`): **[Setup → Update](setup.md#update-keep-meta-current)**.

---

### `nur doctor`

Diagnose install, auth, config, and ecosystem readiness.

```bash
nur doctor
```

Checks:

- Binary path and version
- Config file (model, effort, max_turns, **budget caps**)
- Auth status (key present, last 4 chars)
- Data home, status, usage, sessions paths
- Ecosystem readiness (Graphify, PLUR, Ruflo, browser, omp when present)
- Shell backend (Bash / PowerShell)
- Optional tools on PATH (rg, git, node, npm, uv, ffmpeg)
- Vision support (look, extract_frames)
- Binary SHA-256 integrity

See [Troubleshooting](troubleshooting.md) for interpreting results.

---

### `nur ecosystem`

Manage the Graphify / PLUR / Ruflo / browser / omp ecosystem.

```bash
nur ecosystem <SUBCOMMAND>
```

#### `nur ecosystem ensure`

Install or repair Graphify, PLUR, Ruflo, skills, and related packs. The one-liner, release EXE, and `nur install` already run this **in the foreground**. On later TUI opens it also runs as **background TTL repair** when `ecosystem_auto_ensure = true` (default).

```bash
nur ecosystem ensure [--force]
```

| Flag | Description |
|------|-------------|
| `--force`, `-f` | Force re-install even if marker is fresh |

#### `nur ecosystem status`

Show ecosystem readiness.

```bash
nur ecosystem status
```

---

### `nur browser`

Set up the real-browser `browser` tool for your **default Chromium browser**
(Arc, Chrome, Edge, Brave, …). Stages the `tmwd_cdp_bridge` extension and
walks you through the one-time Load unpacked click.

```bash
nur browser <SUBCOMMAND>
```

#### `nur browser setup`

Stage the extension (no download), detect the default browser, copy the staged
path to the clipboard, and open `chrome://extensions`.

```bash
nur browser setup
```

#### `nur browser status`

Show detected default browser + extension staging state.

```bash
nur browser status
```

Also runs automatically from the installer after `ecosystem ensure`.

---

### `nur install-hook`

Install the Orca agent hook for usage/status reporting.

```bash
nur install-hook
```

---

## Project instruction files

NurCLI loads project-level instructions from your working directory at session start:

| File | Purpose |
|------|---------|
| `META.md` | Primary project instructions |
| `AGENTS.md` | Agent conventions |
| `CLAUDE.md` | Legacy (still loaded) |
| `MUSE.md` | Legacy (still loaded) |

---

## Safe workspace

When launched from a drive root (`C:\` or `/`), NurCLI auto-selects a safe workspace by checking (in order):

1. Git repository root
2. Last session's working directory
3. `~/Laboratory` (or fallback)
