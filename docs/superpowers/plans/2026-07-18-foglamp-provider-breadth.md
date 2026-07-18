# Evidence-First Foglamp Provider Map Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand the existing NurCLI Foglamp scan to advertise all 68 provider routes with source-backed model/auth facts, preserve the existing architecture, and ship NurCLI v0.18.12.

**Architecture:** Keep the current 42-node architecture and add three six-node provider branches: one hub plus five exhaustive cohorts per branch. Temporary ignored Python scripts derive and validate claims against `src/providers.rs`, publish through the existing edit token, and never print the token. Source-controlled docs adopt the exact 68-provider count; runtime behavior remains unchanged except the patch version metadata.

**Tech Stack:** Python 3.11, Foglamp Scan JSON v1, Rust/Cargo, GitHub CLI, 7-Zip

## Global Constraints

- Preserve the public slug and URL `nurcli-oxpatc`.
- Preserve all existing architecture nodes and edges.
- Final graph: exactly 60 nodes and 89 edges.
- Evidence: exactly 68 provider entries, 45 distinct default model IDs, and 11 browser-auth IDs from source.
- Every provider ID must appear in exactly one cohort.
- Project tagline must be exactly `68-provider Rust coding agent with live models, OAuth, tools, vision, and skills` (80 characters).
- Keep `topModels <= 3`, `topTools <= 10`, `topIntegrations <= 10`, nodes `<= 60`, and edges `<= 120`.
- Never print, commit, release, or back up `.foglamp/scan.lock.json` or any payload containing its edit token.
- Do not change the nuroctane `/cli` iframe URL or loading behavior.
- Ship pipeline for this run: commit → release build → push `origin/main` → GitHub `v0.18.12` release → backup. System install is explicitly deferred because the user has active NurCLI processes.

---

### Task 1: Create an evidence validator and prove the current scan fails

**Files:**
- Create (ignored): `.foglamp/validate_provider_scan.py`
- Read: `src/providers.rs`
- Read: `.foglamp/scan.json`

**Interfaces:**
- Consumes: Rust `PROVIDERS`, `oauth_browser_provider_ids()`, Foglamp JSON.
- Produces: exit 0 only when counts, cohort membership, schema limits, lengths, and edge references satisfy the spec.

- [ ] Write a validator with these constants:

```python
EXPECTED_PROVIDER_IDS = {
    'meta', 'openai', 'openai-cc', 'anthropic', 'google', 'antigravity', 'xai',
    'deepseek', 'mistral', 'cohere', 'ai21', 'reka', 'inception', 'writer',
    'upstage', 'thinkingmachines', 'huggingface', 'azure', 'bedrock',
    'github-models', 'github-copilot', 'groq', 'cerebras', 'sambanova', 'lepton',
    'together', 'fireworks', 'deepinfra', 'novita', 'hyperbolic', 'nebius',
    'anyscale', 'octoai', 'nvidia', 'baseten', 'perplexity', 'friendli', 'chutes',
    'venice', 'kluster', 'featherless', 'targon', 'kimi', 'moonshot', 'zhipu',
    'qwen', 'minimax', 'stepfun', 'baichuan', 'yi', 'openrouter', 'omniroute',
    'requesty', 'glama', 'unify', 'portkey', 'litellm', 'vercel', 'cloudflare',
    'nano-gpt', 'opencode', 'helicone', 'aimlapi', 'ollama', 'lmstudio',
    'jan', 'llamacpp', 'vllm'
}
EXPECTED_NEW_NODE_IDS = {
    'direct_routes', 'frontier_core', 'independent_labs_a', 'independent_labs_b',
    'identity_cloud_catalogs', 'github_model_surfaces', 'inference_routes',
    'speed_leaders', 'open_clouds_a', 'open_clouds_b', 'specialized_inference',
    'asian_model_labs', 'router_local_routes', 'one_key_routers', 'ai_gateways',
    'coding_observability', 'local_desktop', 'local_servers'
}
```

The validator must parse one-line `Provider { ... }` entries, parse the OAuth ID array, require 68/45/11, define the approved cohort ID mapping in the validator, require exact one-time coverage, and confirm each cohort node detail contains every mapped provider's source display name, require 60/89, validate all Foglamp length caps, and validate every edge endpoint.

- [ ] Run:

```bash
cd C:/Users/david/Laboratory/nur-cli
python .foglamp/validate_provider_scan.py
```

Expected: FAIL because the current graph has 42 nodes and lacks the 18 provider branch nodes.

### Task 2: Generate the expanded 60-node scan and make validation pass

**Files:**
- Create (ignored): `.foglamp/expand_provider_scan.py`
- Modify (ignored): `.foglamp/scan.json`
- Create (ignored): `.foglamp/scan.before-provider-breadth.json`

**Interfaces:**
- Consumes: current scan and the exact 15 cohort assignments in the approved spec.
- Produces: deterministic 60-node/89-edge Foglamp JSON.

- [ ] Back up the current scan locally before editing.
- [ ] Write the generator so reruns first remove the 18 new node IDs and their edges, then:
  - update the 80-character tagline;
  - set `stats.models` to 45;
  - change `providers.sub` to `68 routes · live models`;
  - update `providers.detail` with 68 routes, 45 defaults, live discovery, 11 browser-auth IDs, and three protocols;
  - append three grouped hubs and 15 exhaustive cohort nodes;
  - import the validator's approved cohort mapping and render each cohort's source display names into its detail;
  - append three labeled root edges and 15 hub-to-cohort call edges.
- [ ] Run the generator, then the validator.

```bash
python .foglamp/expand_provider_scan.py
python .foglamp/validate_provider_scan.py
```

Expected: `PASS: 68 providers · 45 defaults · 11 browser auth · 60 nodes · 89 edges`.

### Task 3: Publish to the existing Foglamp URL and verify the public graph

**Files:**
- Create (ignored): `.foglamp/publish_provider_scan.py`
- Modify (ignored): `.foglamp/scan.lock.json`
- Never retain: `.foglamp/update-payload.json`

**Interfaces:**
- Consumes: validated scan and existing edit token.
- Produces: updated public scan at the same slug, with a redacted local status message.

- [ ] Write a publisher that reads the token in memory, POSTs `{data, editToken}` to `https://api.foglamp.dev/scan`, raises on non-2xx, asserts the returned slug is `nurcli-oxpatc`, atomically replaces the lock file, and prints only slug/URL/expiry.
- [ ] Publish once; do not echo request JSON or response tokens.
- [ ] Fetch the public scan HTML and require all three hub labels and all 15 cohort labels.
- [ ] Open the scan in Chromium/CDP and require the rendered body to contain the three hub labels and `68 routes`.
- [ ] Load `https://www.nuroctane.xyz/cli` at desktop and mobile widths; require the iframe target URL to remain `https://www.foglamp.dev/scan/nurcli-oxpatc` and the child document to contain the new provider labels.

### Task 4: Align source-controlled advertising and release metadata

**Files:**
- Modify: `README.md`
- Modify: `docs/authentication.md`
- Modify: `docs/index.md`
- Modify: `docs/quickstart.md`
- Modify: `docs/tui.md`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `docs/superpowers/plans/2026-07-18-foglamp-provider-breadth.md`

**Interfaces:**
- Consumes: verified source count of 68.
- Produces: exact public docs and v0.18.12 build metadata.

- [ ] Replace user-facing `60+ providers`/`60+` provider-catalog claims with `68 providers` where the statement refers to the current catalog.
- [ ] Change package version from `0.18.11` to `0.18.12` in `Cargo.toml` and the root `nur-cli` package entry in `Cargo.lock`.
- [ ] Run formatting and verification:

```bash
cargo fmt --check
cargo test
cargo build --release
```

Expected: all commands exit 0 and `target/release/nur.exe --version` prints `nur 0.18.12`.

- [ ] Commit only source-controlled files:

```bash
git add README.md docs/authentication.md docs/index.md docs/quickstart.md docs/tui.md \
  Cargo.toml Cargo.lock docs/superpowers/plans/2026-07-18-foglamp-provider-breadth.md
git commit -m "v0.18.12: advertise full provider breadth"
```

### Task 5: Push, release, and back up (install deferred)

**Files:**
- Install: deferred by explicit user instruction; do not replace the running system binary.
- Release asset: `nur-windows-x86_64.exe`
- Backup: `D:/BACKUP/CODE Backups/nur-cli/nur-cli_2026-07-18_<sha>_<slug>.7z`

**Interfaces:**
- Consumes: verified release binary and committed main branch.
- Produces: `origin/main`, GitHub v0.18.12 release, Pages-triggering docs push, and integrity-tested backup. Installation remains pending for a later process-safe run.

- [ ] Do not replace `%USERPROFILE%/.local/bin/nur.exe` during this run. Verify only `target/release/nur.exe --version` and require `nur 0.18.12`; report installation as explicitly deferred.
- [ ] Fetch tags, verify `v0.18.12` does not exist, and push `main`.
- [ ] Tag `v0.18.12` and create a GitHub release with notes covering the evidence-first 68-provider Foglamp map, exact provider docs, and no runtime behavior change; attach the binary as `nur-windows-x86_64.exe`.
- [ ] Confirm the release URL and that the docs workflow was triggered by the push.
- [ ] Create an integrity-tested 7z backup excluding `.git`, `.foglamp`, `.nur`, `target`, `node_modules`, `graphify-out`, `.next`, and `dist`.
- [ ] Confirm local HEAD equals `origin/main`, the release asset exists, the public Foglamp graph is updated, and the backup exists before reporting completion.
