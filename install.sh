#!/usr/bin/env bash
# One-shot install of Meta CLI (unofficial) — builds the `meta` binary (muse alias).
#
# From a clone:
#   ./install.sh
#
# Remote one-shot:
#   curl -fsSL https://raw.githubusercontent.com/nuroctane/meta-cli/main/install.sh | bash
#
# Secrets are NEVER written into the repo. Keys live only in ~/.muse/auth.json
# or env MODEL_API_KEY / MUSE_API_KEY / META_API_KEY.

set -euo pipefail

REPO_URL="https://github.com/nuroctane/meta-cli.git"
BRANCH="main"
REPO_DIR="${META_CLI_DIR:-$HOME/laboratory/meta-cli}"
SKIP_HOOK="${SKIP_HOOK:-0}"

step() { printf '  → %s\n' "$*"; }
ok()   { printf '  ✓ %s\n' "$*"; }
warn() { printf '  ! %s\n' "$*"; }

echo ""
echo "  Meta CLI (unofficial) installer"
echo "  command: meta  ·  Meta Model API agent · not affiliated with Meta"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd || true)"
IN_REPO=0
if [[ -n "${SCRIPT_DIR}" && -f "${SCRIPT_DIR}/Cargo.toml" ]] && grep -q 'name = "meta-cli"' "${SCRIPT_DIR}/Cargo.toml"; then
  REPO_DIR="${SCRIPT_DIR}"
  IN_REPO=1
fi

if [[ "${IN_REPO}" -eq 0 ]]; then
  step "Source: ${REPO_DIR}"
  command -v git >/dev/null || { echo "git is required"; exit 1; }
  mkdir -p "$(dirname "${REPO_DIR}")"
  if [[ -f "${REPO_DIR}/Cargo.toml" ]]; then
    step "Updating existing clone…"
    git -C "${REPO_DIR}" fetch origin "${BRANCH}"
    git -C "${REPO_DIR}" checkout "${BRANCH}"
    git -C "${REPO_DIR}" pull --ff-only origin "${BRANCH}" || true
  else
    step "Cloning ${REPO_URL} …"
    git clone --branch "${BRANCH}" --single-branch "${REPO_URL}" "${REPO_DIR}"
  fi
fi
ok "Repo: ${REPO_DIR}"

export PATH="${HOME}/.cargo/bin:${PATH}"
if ! command -v cargo >/dev/null 2>&1; then
  step "Rust/cargo not found — installing rustup…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env" 2>/dev/null || export PATH="${HOME}/.cargo/bin:${PATH}"
fi
command -v cargo >/dev/null || { echo "cargo not found after rustup; open a new shell and re-run"; exit 1; }
ok "cargo $(cargo --version)"

step "Building release (first time can take a few minutes)…"
( cd "${REPO_DIR}" && cargo build --release )
BUILT="${REPO_DIR}/target/release/meta"
[[ -f "${BUILT}" ]] || BUILT="${REPO_DIR}/target/release/muse"
[[ -f "${BUILT}" ]] || { echo "missing release binary"; exit 1; }

DEST_DIR="${HOME}/.local/bin"
mkdir -p "${DEST_DIR}"
cp -f "${BUILT}" "${DEST_DIR}/meta"
cp -f "${BUILT}" "${DEST_DIR}/muse"
chmod +x "${DEST_DIR}/meta" "${DEST_DIR}/muse"
export PATH="${DEST_DIR}:${PATH}"

for rc in "${HOME}/.zprofile" "${HOME}/.zshrc" "${HOME}/.bash_profile" "${HOME}/.bashrc" "${HOME}/.profile"; do
  if [[ -f "${rc}" ]] && ! grep -q '\.local/bin' "${rc}" 2>/dev/null; then
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "${rc}"
    ok "Appended ~/.local/bin to ${rc}"
    break
  fi
done

ok "Installed ${DEST_DIR}/meta ($("${DEST_DIR}/meta" --version))"

# ── Ecosystem: Graphify · PLUR · Ruflo ────────────────────────────────────
step "Provisioning agent ecosystem (graphify · plur · ruflo)…"
if ! command -v node >/dev/null 2>&1; then
  warn "Node.js not on PATH — plur/ruflo need Node 20+. Install then: meta ecosystem ensure"
fi
if ! command -v uv >/dev/null 2>&1; then
  step "Installing uv (for graphify)…"
  curl -LsSf https://astral.sh/uv/install.sh | sh || warn "uv install skipped"
  export PATH="${HOME}/.local/bin:${PATH}"
fi
"${DEST_DIR}/meta" ecosystem ensure --force || warn "Ecosystem ensure deferred to first meta open"
ok "Ecosystem ready (or will finish on first open)"

if [[ "${SKIP_HOOK}" != "1" ]]; then
  "${DEST_DIR}/meta" install-hook >/dev/null 2>&1 && ok "Orca ADE hook installed (if applicable)" || true
fi

KEY="${MODEL_API_KEY:-${META_API_KEY:-${MUSE_API_KEY:-}}}"
if [[ -n "${KEY}" ]]; then
  step "API key found in environment — saving to ~/.muse/auth.json (local only)…"
  "${DEST_DIR}/meta" auth login --key "${KEY}" >/dev/null
  ok "Auth stored under ~/.muse/ (never committed to git)"
else
  warn "No API key in env yet. After install:  meta auth login"
  warn "Get a key: https://dev.meta.ai/"
fi

echo ""
echo "  Done."
echo "  Run:   meta"
echo "  Auth:  meta auth login     (key stays in ~/.muse only)"
echo "  Stack: graphify + plur + ruflo auto-ready on open"
echo "  Orca:  orca terminal create --command meta"
echo "  Docs:  https://github.com/nuroctane/meta-cli"
echo ""
