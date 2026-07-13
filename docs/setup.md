# Setup

System requirements, platform-specific installation, updates, and uninstallation for Meta CLI.

## System requirements

Meta CLI runs on the following platforms:

| Requirement | Details |
|-------------|---------|
| **Operating system** | Windows 10+ · macOS 13+ · Ubuntu 20.04+ · Debian 10+ · Alpine 3.19+ |
| **Hardware** | 4 GB+ RAM, x64 or ARM64 processor |
| **Network** | Internet connection required (Meta Model API) |
| **Shell** | PowerShell, CMD, Bash, or Zsh |

### Additional dependencies

| Dependency | Required | Purpose |
|------------|----------|---------|
| **Node.js 20+** | No (recommended) | PLUR, Ruflo, Executor, skills CLI, AKM |
| **uv** or Python 3.10+ | No (recommended) | Graphify |
| **ripgrep** | No | Fast `grep` / `glob` (falls back if missing) |
| **ffmpeg** | No | `extract_frames` / design-from-video |

---

## Install Meta CLI

=== "Windows (PowerShell)"

    ```powershell
    irm https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.ps1 | iex
    ```

=== "macOS / Linux"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.sh | bash
    ```

The install script will:

1. Install Rust if needed
2. Clone or update the repo
3. `cargo build --release`
4. Install **`meta`** (+ `muse` alias) to `~/.local/bin` and verify SHA-256
5. Run `meta ecosystem ensure` when Node/uv are available
6. Set up Orca hook when possible
7. Save auth if `META_API_KEY` / `MODEL_API_KEY` is set (machine-local only)

!!! tip "Already cloned?"
    ```bash
    cd meta-cli
    .\install.ps1          # Windows
    # ./install.sh         # macOS / Linux
    ```

### Verify your installation

After installing, confirm Meta CLI is working:

```bash
meta --version
```

Run a full health check:

```bash
meta doctor
```

---

## Authenticate

Meta CLI requires a [Meta Model API key](https://dev.meta.ai/). Get one from [dev.meta.ai](https://dev.meta.ai/) → API keys.

Log in from the command line:

```bash
meta auth login
```

Or sign in from inside the TUI — run `meta` and use `/login` (secure masked key entry — never echoed to transcript or history) and `/logout` (clears the stored key). Launching with no key opens the login prompt automatically.

See [Authentication](authentication.md) for all options.

---

## Update Meta CLI

### Auto-updates (native install)

Native installations update automatically in the background. Updates take effect the next time you start Meta CLI.

### Manual update

```bash
meta update
```

Or re-run the install script:

=== "Windows"

    ```powershell
    irm https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.ps1 | iex
    ```

=== "macOS / Linux"

    ```bash
    curl -fsSL https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.sh | bash
    ```

### Disable auto-updates

Add to `~/.meta/config.toml`:

```toml
auto_updates = false
```

Or set the environment variable:

```bash
export DISABLE_AUTOUPDATER=1
```

---

## Uninstall Meta CLI

### Binary and data

=== "Windows"

    ```powershell
    Remove-Item -Path "$env:USERPROFILE\.local\bin\meta.exe" -Force
    Remove-Item -Path "$env:USERPROFILE\.local\bin\muse.exe" -Force
    Remove-Item -Path "$env:USERPROFILE\.local\share\meta" -Recurse -Force
    ```

=== "macOS / Linux"

    ```bash
    rm -f ~/.local/bin/meta
    rm -f ~/.local/bin/muse
    rm -rf ~/.local/share/meta
    ```

### Configuration files

!!! warning "This will delete all your settings, sessions, and usage history."

=== "Windows"

    ```powershell
    Remove-Item -Path "$env:USERPROFILE\.meta" -Recurse -Force
    ```

=== "macOS / Linux"

    ```bash
    rm -rf ~/.meta
    ```

### Legacy files

Older installs used `~/.muse/`. To remove:

=== "Windows"

    ```powershell
    Remove-Item -Path "$env:USERPROFILE\.muse" -Recurse -Force
    ```

=== "macOS / Linux"

    ```bash
    rm -rf ~/.muse
    ```
