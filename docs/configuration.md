# Configuration

Meta CLI is configured via a TOML file and environment variables.

## Config file

The config file lives at `~/.meta/config.toml` and is created on first run.

```toml
model = "muse-spark-1.1"
base_url = "https://api.meta.ai/v1"
reasoning_effort = "high"
max_turns = 40
stream = true
context_window = 1000000
```

### Settings reference

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `model` | string | `muse-spark-1.1` | Meta Model API model id |
| `base_url` | string | `https://api.meta.ai/v1` | API endpoint |
| `reasoning_effort` | string | `high` | Reasoning depth: `minimal`, `low`, `medium`, `high`, `xhigh` |
| `max_turns` | integer | `40` | Max agent turns per prompt (range: 1–200) |
| `stream` | bool | `true` | Stream API responses |
| `context_window` | integer | `1000000` | Model context window in tokens (range: 1000–2000000) |

### Reasoning effort levels

| Level | Behaviour |
|-------|-----------|
| `minimal` | Fastest, shallowest reasoning |
| `low` | Light reasoning |
| `medium` | Balanced |
| `high` | Deep reasoning (default) |
| `xhigh` | Maximum reasoning depth |

---

## Environment variables

### API and model

| Variable | Purpose |
|----------|---------|
| `META_API_KEY` | API key (preferred) |
| `MODEL_API_KEY` | API key (alternative) |
| `MUSE_API_KEY` | API key (legacy) |
| `META_MODEL` | Override model id |
| `MUSE_MODEL` | Override model id (legacy) |

### Paths

| Variable | Purpose |
|----------|---------|
| `META_HOME` | Override data home (default `~/.meta`) |
| `MUSE_HOME` | Override data home (legacy) |
| `META_CWD` | Default working directory |

### Status and usage

| Variable | Purpose |
|----------|---------|
| `META_STATUS_PATH` | Path to live status file |
| `META_USAGE_LOG_PATH` | Path to usage log |
| `META_SESSION_ID` | Current session id |
| `META_PROVIDER` | Provider identifier (set to `meta`) |

### Update control

| Variable | Purpose |
|----------|---------|
| `DISABLE_AUTOUPDATER` | Set to `1` to disable background auto-updates |
| `DISABLE_UPDATES` | Set to `1` to block all update paths |

### Ecosystem

| Variable | Purpose |
|----------|---------|
| `CLAUDE_FLOW_DB_PATH` | Ruflo database path |
| `CLAUDE_FLOW_MEMORY_PATH` | Ruflo home path |
| `USE_BUILTIN_RIPGREP` | Set to `0` to use system ripgrep |

---

## Data home

All Meta CLI state lives under `~/.meta/` by default:

```
~/.meta/
├── auth.json           # API key
├── config.toml         # Configuration
├── status.json         # Live token/cost status
├── usage.jsonl         # Per-request usage log
├── ade.json            # ADE discovery manifest
├── sessions/           # Session files (UUID.json)
├── skills/             # Installed skill packs
├── ruflo/              # Vector memory database
└── skill-packs/        # Skill pack metadata
```

Override with `META_HOME` (or legacy `MUSE_HOME`).

---

## Legacy migration

If you upgraded from a pre-0.5.14 build, Meta CLI automatically gap-fills missing files from `~/.muse/` into `~/.meta/`. Existing files are never overwritten.

`meta auth logout` clears auth from both `~/.meta/` and legacy `~/.muse/`.

---

## Project instructions

Meta CLI reads project-level instruction files from your working directory:

| File | Purpose |
|------|---------|
| `META.md` | Primary project instructions |
| `AGENTS.md` | Agent conventions (shared with other tools) |
| `CLAUDE.md` | Legacy instructions (still loaded) |
| `MUSE.md` | Legacy instructions (still loaded) |

These are loaded at session start and prepended to the system prompt.
