# Troubleshooting

## `meta doctor`

The built-in health check for install, auth, config, and ecosystem:

```bash
meta doctor
```

### What it checks

| Check | Shows |
|-------|-------|
| `binary` | Path to the `meta` binary |
| `config` | Model, effort, max_turns, config path |
| `auth` | Whether a key is set (last 4 chars only) |
| `home` | Data home directory |
| `status` | Path to `status.json` |
| `usage` | Path to `usage.jsonl` |
| `sessions` | Path to sessions directory |
| `ecosystem` | Graphify, PLUR, Ruflo readiness |
| `shell` | Bash / PowerShell backend |
| `rg`, `git`, `node`, `npm`, `uv`, `ffmpeg` | Whether on PATH |
| `vision` | look + extract_frames support |
| `sha256` | Binary integrity check |

### Common doctor output

**All green:**

```text
meta doctor · v0.6.6

binary  C:\Users\you\.local\bin\meta.exe
config  model=muse-spark-1.1 effort=high max_turns=40  (C:\Users\you\.meta\config.toml)
auth    key set (…abcd)
home    C:\Users\you\.meta
status  C:\Users\you\.meta\status.json
usage   C:\Users\you\.meta\usage.jsonl
sessions C:\Users\you\.meta\sessions

ecosystem
  graphify  ✓
  plur      ✓
  ruflo     ✓

shell   Git Bash
rg      C:\Program Files\Git\usr\bin\rg.exe
git     C:\Program Files\Git\bin\git.exe
node    C:\Program Files\nodejs\node.exe
npm     C:\Program Files\nodejs\npm.cmd
uv      C:\Users\you\.local\bin\uv.exe
ffmpeg  C:\Program Files\ffmpeg\bin\ffmpeg.exe
vision  look · extract_frames (input_image / input_video)

sha256  abc123...  (matches install record)

doctor complete
```

---

## Common issues

### `command not found: meta`

The `meta` binary is not on your PATH.

**Fix:**

1. Check where it was installed: `ls ~/.local/bin/meta`
2. Add `~/.local/bin` to your PATH:
    ```bash
    # Bash / Zsh
    export PATH="$HOME/.local/bin:$PATH"

    # PowerShell
    $env:Path += ";$env:USERPROFILE\.local\bin"
    ```
3. Restart your terminal

### `auth    not set`

No API key found.

**Fix:**

```bash
meta auth login
# or
export META_API_KEY="your-key-here"
```

### Ecosystem components missing

```text
ecosystem
  graphify  ✗
  plur      ✗
  ruflo     ✗
```

**Fix:**

1. Install Node.js 20+ and uv:
    ```bash
    # Windows
    winget install OpenJS.NodeJS.LTS
    winget install astral-sh.uv

    # macOS
    brew install node uv

    # Linux
    sudo apt install nodejs npm
    pip install uv
    ```
2. Re-run:
    ```bash
    meta ecosystem ensure --force
    ```

### `ffmpeg not on PATH`

`extract_frames` requires ffmpeg.

**Fix:** Install ffmpeg (see [Vision](vision.md#requirements)).

### `sha256 mismatch`

Binary may be corrupted or from a different source.

**Fix:** Re-run the install script:

```bash
# Windows
irm https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.ps1 | iex

# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.sh | bash
```

### API errors / rate limits

If you see API errors:

1. Check your key: `meta auth status`
2. Check the model: `cat ~/.meta/config.toml`
3. Verify the API is up: [dev.meta.ai](https://dev.meta.ai/)

### Session not resuming

```bash
meta sessions              # list sessions
meta -r <session-id>       # resume by id
meta -c                    # continue most recent for this cwd
```

### `config` validation errors

```text
config  invalid reasoning_effort 'super' — use minimal|low|medium|high|xhigh
```

**Fix:** Edit `~/.meta/config.toml` and set a valid effort level.

---

## Legacy migration

If you upgraded from a pre-0.5.14 build (using `~/.muse/`), Meta CLI automatically gap-fills missing files into `~/.meta/`. Existing files are never overwritten.

To manually migrate:

```bash
# Files are copied automatically on first run.
# To force a clean start:
meta auth logout     # clears both ~/.meta and ~/.muse
meta auth login      # re-authenticate
```

---

## Getting more help

- Run `meta doctor` for a full diagnostic
- Check the [GitHub issues](https://github.com/nuroctane/meta-cli/issues)
- Open a new issue with your `meta doctor` output
