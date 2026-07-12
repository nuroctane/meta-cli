# Security

Meta CLI is **unofficial** community software. It is not affiliated with Meta Platforms, Inc.

## Where secrets live (local only)

| Location | Contents |
|----------|----------|
| `~/.muse/auth.json` | Meta Model API key after `muse auth login` |
| env `MODEL_API_KEY` / `MUSE_API_KEY` | Optional override (never print in logs) |
| `~/.muse/sessions/`, `status.json`, `usage.jsonl` | Session + usage metadata (no key in usage log) |

These paths are **gitignored**. They are never part of this repository.

## What is on GitHub

Only source, docs, and install scripts. Install scripts:

- Do **not** embed API keys
- May **read** a key already present in your environment and store it under `~/.muse/` on your machine
- Never echo the key value

## Reporting a leak

If you believe a secret was committed to this repo, rotate the key at [dev.meta.ai](https://dev.meta.ai/) immediately and open an issue (without pasting the secret).
