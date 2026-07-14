# Authentication

Meta CLI can talk to **many providers** — not only the Meta Model API. Sign-in is
a two-step flow: pick a provider, then enter its API key (local servers can skip
the key). The active provider, endpoint, and default model are stored in
`~/.meta/config.toml`; the key lives only in `~/.meta/auth.json`.

## Get a key

| Provider | Where to get a key |
|----------|--------------------|
| **Meta Model API** (default) | [dev.meta.ai](https://dev.meta.ai/) → API keys |
| OpenAI, Anthropic, Gemini, xAI, … | Each vendor’s dashboard |
| OpenRouter, OmniRoute, Together, Groq, … | Aggregator / cloud dashboard |
| Ollama, LM Studio, llama.cpp, vLLM | Often **no key** (local) |

## Log in from the TUI (recommended)

```text
/login
```

What happens:

1. Prior key is cleared so you start from a clean slate.
2. A **scrollable, type-to-filter** picker lists **45+ providers** (frontier APIs,
   inference clouds, Chinese labs, OpenAI-compatible routers, local servers).
3. You enter the key for that provider (**masked** — never echoed to the
   transcript or shell history). Local providers may allow an empty key.
4. Config is updated: `provider`, `base_url`, and `model` (that provider’s
   default). The HTTP client is **hot-swapped** for the rest of the session.

`/logout` clears the stored key and blocks further turns until you `/login`
again (environment-variable keys still apply on the next launch).

No key on launch → the login modal opens automatically.

## Log in from the command line

CLI login still targets a **Meta Model API** key path (prompt / `--key`):

```bash
meta auth login
meta auth login --key YOUR_KEY   # avoid on shared machines
```

Key is written to `~/.meta/auth.json` and never printed.

To use a non-Meta provider end-to-end, prefer **`/login`** in the TUI so the
provider catalog, endpoint, model, and API style all switch together.

## Via environment variable

```bash
export META_API_KEY="your-key-here"
# or
export MODEL_API_KEY="your-key-here"
```

If a key is found in the environment, Meta CLI can save it to `~/.meta/auth.json`
automatically. Many catalog entries also document a vendor-specific env name
(e.g. `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`) — use those with your shell when
you prefer not to store a key via `/login`.

!!! note "Legacy variables"
    `MUSE_API_KEY` is also accepted for backwards compatibility.

## Check auth status

```bash
meta auth status
```

Shows whether a key is set (last 4 characters only — never the full key).

## Log out

```bash
meta auth logout
```

Removes the stored key from `~/.meta/auth.json` (and any migrated key under
legacy `~/.muse/`). Same effect as TUI `/logout` for the key file.

---

## Providers & API styles

The catalog lives in code (`src/providers.rs`). Categories include:

| Category | Examples |
|----------|----------|
| Default | **Meta Model API** (`muse-spark-1.1`) |
| Frontier | OpenAI, Anthropic, Google Gemini, xAI Grok, DeepSeek, Mistral, Cohere, … |
| Inference clouds | Groq, Cerebras, Together, Fireworks, DeepInfra, Perplexity, NVIDIA NIM, … |
| Chinese labs | Moonshot (Kimi), Zhipu GLM, Qwen (DashScope), MiniMax, … |
| Aggregators / routers | OpenRouter, OmniRoute, Requesty, Vercel / Cloudflare AI gateways, … |
| Local | Ollama, LM Studio, llama.cpp, vLLM (key often optional) |

Each entry declares:

- **base URL** and a sensible **default model**
- usual **env var** for the key
- **API style**: **Responses** (`POST /responses`) or **Chat Completions**
  (`POST /chat/completions`)

Meta CLI’s agent always speaks an internal Responses-shaped protocol. For Chat
Completions providers, a built-in adapter (`src/api/chat.rs`) translates
requests and replies (including streamed tool-call fragments) so tools and
streaming keep working.

---

## Auth precedence

API key resolution order:

1. `~/.meta/auth.json` (from `meta auth login` or successful `/login`)
2. `META_API_KEY`
3. `MODEL_API_KEY`
4. `MUSE_API_KEY` (legacy)
5. Interactive TUI prompt (opens `/login` when no key is found)

Active **provider id / base URL / model** come from `~/.meta/config.toml`
(written by `/login`).

---

## Where secrets live

| Location | Contents |
|----------|----------|
| `~/.meta/auth.json` | API key only |
| `~/.meta/config.toml` | `provider`, `base_url`, `model`, … (no secret) |
| Env `META_API_KEY` / `MODEL_API_KEY` | Optional override (never printed in logs) |
| `~/.meta/sessions/` | Session metadata (no key) |
| `~/.meta/status.json` | Live token usage (no key) |
| `~/.meta/usage.jsonl` | Per-request usage log (no key) |

!!! warning "Never commit"
    Never commit `~/.meta/`, `.env` files with keys, or session dumps containing base64 media.
