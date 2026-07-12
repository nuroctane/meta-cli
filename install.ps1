#Requires -Version 5.1
<#
.SYNOPSIS
  One-shot install of Meta CLI (unofficial) — builds the `muse` binary.

.DESCRIPTION
  Works two ways:
    1) From a clone:  .\install.ps1
    2) Remote one-shot (no clone yet):
         irm https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.ps1 | iex

  Steps: ensure Rust → clone if needed → cargo build --release →
  install muse to %USERPROFILE%\.local\bin → PATH → optional Orca hook →
  optional auth if MODEL_API_KEY is set.

  Secrets are NEVER written into the repo. Keys live only in:
    %USERPROFILE%\.muse\auth.json   or   env MODEL_API_KEY / MUSE_API_KEY

.PARAMETER SkipHook
  Skip Orca agent-hook install.

.PARAMETER RepoDir
  Where to clone/build (default: %USERPROFILE%\laboratory\meta-cli).
#>
param(
    [switch]$SkipHook,
    [string]$RepoDir = ""
)

$ErrorActionPreference = "Stop"
$RepoUrl = "https://github.com/nuroctane/meta-cli.git"
$Branch = "main"

function Write-Step($msg) { Write-Host "  → $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "  ✓ $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "  ! $msg" -ForegroundColor Yellow }

Write-Host ""
Write-Host "  Meta CLI (unofficial) installer" -ForegroundColor Blue
Write-Host "  Muse Spark agent · not affiliated with Meta" -ForegroundColor DarkGray
Write-Host ""

# ── locate or clone source ────────────────────────────────────────────────
$scriptRoot = $PSScriptRoot
if (-not $scriptRoot -and $MyInvocation.MyCommand.Path) {
    $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
}

$inRepo = $false
if ($scriptRoot -and (Test-Path (Join-Path $scriptRoot "Cargo.toml"))) {
    $toml = Get-Content (Join-Path $scriptRoot "Cargo.toml") -Raw
    if ($toml -match 'name\s*=\s*"meta-cli"') {
        $RepoDir = $scriptRoot
        $inRepo = $true
    }
}

if (-not $RepoDir) {
    $RepoDir = Join-Path $env:USERPROFILE "laboratory\meta-cli"
}

if (-not $inRepo) {
    Write-Step "Source: $RepoDir"
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        throw "git is required. Install Git for Windows: https://git-scm.com/download/win"
    }
    $parent = Split-Path -Parent $RepoDir
    if (-not (Test-Path $parent)) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
    if (Test-Path (Join-Path $RepoDir "Cargo.toml")) {
        Write-Step "Updating existing clone (git pull)…"
        Push-Location $RepoDir
        try {
            git fetch origin $Branch 2>$null | Out-Null
            git checkout $Branch 2>$null | Out-Null
            git pull --ff-only origin $Branch 2>$null | Out-Null
        } finally { Pop-Location }
    } else {
        if (Test-Path $RepoDir) {
            throw "Directory exists but is not a meta-cli checkout: $RepoDir"
        }
        Write-Step "Cloning $RepoUrl …"
        git clone --branch $Branch --single-branch $RepoUrl $RepoDir
        if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
    }
}

Write-Ok "Repo: $RepoDir"

# ── Rust toolchain ────────────────────────────────────────────────────────
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$env:Path = "$cargoBin;$env:Path"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Step "Rust/cargo not found — installing rustup (stable)…"
    $rustup = Join-Path $env:TEMP "rustup-init-meta-cli.exe"
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $rustup
    & $rustup -y --default-toolchain stable
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "rustup-init failed; trying winget…"
        winget install --id Rustlang.Rustup -e --accept-package-agreements --accept-source-agreements
    }
    $env:Path = "$cargoBin;$env:Path"
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "cargo still not on PATH. Open a new terminal and re-run, or install from https://rustup.rs"
    }
}
Write-Ok "cargo $((cargo --version))"

# ── build ─────────────────────────────────────────────────────────────────
Push-Location $RepoDir
try {
    Write-Step "Building release (first time can take a few minutes)…"
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }
} finally { Pop-Location }

$built = Join-Path $RepoDir "target\release\muse.exe"
if (-not (Test-Path $built)) { throw "missing $built" }

# ── install binary ────────────────────────────────────────────────────────
$destDir = Join-Path $env:USERPROFILE ".local\bin"
New-Item -ItemType Directory -Force -Path $destDir | Out-Null
$dest = Join-Path $destDir "muse.exe"
Copy-Item -Force $built $dest
$env:Path = "$destDir;$env:Path"

# Persist User PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $userPath) { $userPath = "" }
if ($userPath -notlike "*$destDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$destDir;$userPath", "User")
    Write-Ok "Added $destDir to User PATH (new terminals pick it up automatically)"
} else {
    Write-Ok "PATH already includes $destDir"
}

$ver = & $dest --version
Write-Ok "Installed $dest ($ver)"

# ── Orca hook (best-effort) ───────────────────────────────────────────────
if (-not $SkipHook) {
    try {
        & $dest install-hook 2>$null | Out-Null
        Write-Ok "Orca ADE hook installed (if Orca is present)"
    } catch {
        Write-Warn "Orca hook skipped ($($_.Exception.Message))"
    }
}

# ── auth: never print the key ─────────────────────────────────────────────
$key = $env:MODEL_API_KEY
if (-not $key) { $key = $env:MUSE_API_KEY }
if (-not $key) {
    $key = [Environment]::GetEnvironmentVariable("MODEL_API_KEY", "User")
}
if (-not $key) {
    $key = [Environment]::GetEnvironmentVariable("MUSE_API_KEY", "User")
}

if ($key -and $key.Trim().Length -gt 0) {
    Write-Step "MODEL_API_KEY found in environment — saving to ~/.muse/auth.json (local only)…"
    # Pipe via env so the key is not put on the process command line
    $env:MODEL_API_KEY = $key.Trim()
    & $dest auth login --key $env:MODEL_API_KEY 2>$null | Out-Null
    Write-Ok "Auth stored under $env:USERPROFILE\.muse\ (never committed to git)"
} else {
    Write-Warn "No API key in env yet. After install:"
    Write-Host "      muse auth login" -ForegroundColor DarkGray
    Write-Host "    or set User env MODEL_API_KEY from https://dev.meta.ai/" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "  Done." -ForegroundColor Green
Write-Host "  Run:   muse" -ForegroundColor White
Write-Host "  Auth:  muse auth login     (key stays in ~/.muse only)" -ForegroundColor DarkGray
Write-Host "  Docs:  https://github.com/nuroctane/meta-cli" -ForegroundColor DarkGray
Write-Host ""
