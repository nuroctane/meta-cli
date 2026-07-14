# NurCLI

**Fully loaded terminal coding agent** for [Meta Model API](https://dev.meta.ai/) — not a thin wrapper.

Custom Rust harness, dense Nur-gold TUI, **native vision**, tools, knowledge stack, hardened sandbox. Any model id via `--model` / `/model` / config.

!!! info "Unofficial"
    Not affiliated with Meta Platforms, Inc. · Community · [nuroctane/nur-cli](https://github.com/nuroctane/nur-cli)

```text
meta          # primary — Nur-gold interactive TUI
muse          # legacy alias (same binary)
```

---

## Install — one line

=== "<span class='install-hot'>Windows (PowerShell)</span>"

    ```powershell
    irm https://raw.githubusercontent.com/nuroctane/nur-cli/main/install.ps1 | iex
    ```

=== "<span class='install-hot'>macOS / Linux</span>"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/nuroctane/nur-cli/main/install.sh | bash
    ```

Then: `nur auth login` → `nur`.

**<span class="install-hot">Windows EXE:</span>** download `nur-windows-x86_64.exe` from [Releases](https://github.com/nuroctane/nur-cli/releases/latest) → double‑click → **full install** (PATH + ecosystem + browser) then TUI. Same stack as the one-liner, no compile. Details: **[Setup](setup.md)**.

### Update

```bash
nur update
```

That’s the normal upgrade. Pulls / rebuilds when you have a Laboratory checkout, reinstalls `nur` + ecosystem. Or re-run the one-liner / re-download the EXE. Full detail: **[Setup → Update](setup.md#update-keep-meta-current)**.

---

## What you get

| Surface | Details |
|---------|---------|
| **TUI** | Streaming · duration chips · expandable thought/tool cards · click-to-peek · **green/red transcript diffs** · **prompt menu (fork · revert · copy)** · drag-select · always-on scrollbar · ↓ End · sticky prompt · sessions browser · approval mini-diff · lean banner · **`/login` (45+ providers)** · **`/goal` `/btw` `/codesearch` `/mc` `/feedback` `/tips`** · **`/budget` `/poor` `/permissions` `/hooks` `/cd` `/doctor`** |
| **Agent** | Manual / plan / auto · tool loop · subagents · todos · **smarter auto-compact** · **session $ / token budgets** · **tool-result spill** · Esc cancel · Shift+Tab mid-turn · prompt-cache keys · **Chat Completions adapter** for non-Responses providers |
| **Vision** | `look` (images / short video) · `extract_frames` (ffmpeg keyframes) · prompt auto-attach of media paths |
| **Tools** | read · edit · bash · web · **browser** · git · knowledge stack · agent — **all first-class** |
| **Ecosystem** | Graphify · PLUR · Ruflo · Executor · **omp** · **browser** · AKM · **800+ skills** — full install at setup; later open = TTL repair (`ecosystem_auto_ensure`) |
| **Hardening** | Sandbox · bash denylist · SSRF blocks · atomic `~/.nur` IO · session **`.json.bak`** · **permissions.toml** · optional **hooks.toml** · API retries · install SHA-256 · `nur doctor` |
| **Host panels** | Live `status.json` / `usage.jsonl` · Orca hook when present |

**Current version: v0.10.0**

---

## Quick links

- **[Setup](setup.md)** — System requirements, install, **how to update**, uninstall
- **[Quickstart](quickstart.md)** — Your first session in 60 seconds
- **[Commands](commands.md)** — Full CLI reference
- **[TUI](tui.md)** — Keyboard shortcuts, slash commands, colour system
- **[Tools](tools.md)** — All native tools (read, edit, shell, web, git, knowledge, agent)
- **[Vision](vision.md)** — Images, video, `look`, `extract_frames`
- **[Ecosystem](ecosystem.md)** — Graphify, PLUR, Ruflo, skills, AKM
- **[Configuration](configuration.md)** — `config.toml`, environment variables, settings
- **[Security](security.md)** — Where secrets live, sandbox, reporting
- **[Troubleshooting](troubleshooting.md)** — `nur doctor`, common issues

---

## Built with

The terminal UI is powered by **[Ratatui](https://ratatui.rs/)** ([github](https://github.com/ratatui/ratatui)) and **[crossterm](https://github.com/crossterm-rs/crossterm)** — huge thanks to the Ratatui project. Also built on [tokio](https://tokio.rs), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs), and [clap](https://github.com/clap-rs/clap).
