---
name: akarso
description: Post, schedule, and reply across 14 social platforms (X, LinkedIn, Instagram, Facebook, TikTok, YouTube, Threads, Reddit, Bluesky, Mastodon, Discord, Slack, Pinterest, Google Business) from nur. Use when the user wants to publish/schedule a social post, list or delete posts, connect a social account, or check posting status.
---

# Akarso — social posting from nur

nur bundles the **`akarso`** tool (native, wraps the `akarso` CLI) and auto-installs
it. Prefer the `akarso` tool or `/akarso` — do **not** shell out to bash for this.

## First-time setup (user runs once)

```sh
akarso auth login          # browser Google sign-in; saves an API key
akarso accounts connect x  # OAuth-connect a platform (per platform)
```

Publishing requires an Akarso subscription; connecting accounts does not.

## Tool actions (`akarso` tool)

| action | purpose | outward-facing? |
|--------|---------|-----------------|
| `auth_check` | verify the saved key works | no |
| `accounts_list` / `accounts_health` | list / health-check connected accounts | no |
| `accounts_get` (platform) | account details + selectable channels | no |
| `posts_list` (status, platforms) | list posts | no |
| `posts_get` (id) | one post's details | no |
| `posts_create` (text, platforms, media, scheduled_at, publish) | draft / schedule / publish | **YES** |
| `posts_delete` (id) | delete a post | **YES** |
| `accounts_connect` (platform) | browser OAuth connect | **YES** |

`platforms` is a comma list: `x,linkedin,instagram`. `scheduled_at` accepts
relative shortcuts (`2h`, `3d`, `1w`) or an ISO timestamp. Omit `scheduled_at`
and `publish` to save a **draft**.

## Rules

- **Confirm before publishing.** `posts_create` with `publish=true` (or a
  `scheduled_at`) and `posts_delete` are outward-facing and hard to reverse — they
  are approval-gated, and you should restate the exact text + platforms + timing to
  the user before firing.
- Default to a **draft** (no `publish`, no `scheduled_at`) unless the user clearly
  said publish/schedule now.
- Never invent post content or platforms — use exactly what the user asked for.
- If `auth_check` fails, tell the user to run `akarso auth login` (and
  `akarso accounts connect <platform>`); do not try to work around auth.
- Media is a local path or URL via `media`.

Docs: https://akarso.co/docs/getting-started/quickstart · Upstream: https://github.com/remorses/akarso
