# Foglamp Provider Breadth — Design

**Date:** 2026-07-18
**Repository:** `nur-cli`
**Public scan:** `https://www.foglamp.dev/scan/nurcli-oxpatc`

## Goal

Expand the existing NurCLI Foglamp scan so it accurately advertises the full provider and model-routing capacity already implemented in `src/providers.rs`, without removing the architecture that makes the scan useful as a codebase map.

The revised graph must make these source-backed facts discoverable:

- 68 selectable provider routes.
- 45 distinct default model IDs across those routes.
- Live model discovery for the active API key or OAuth credential.
- 11 provider IDs with browser authentication.
- Three supported wire protocols: Responses, Chat Completions, and Anthropic Messages.
- Direct APIs, enterprise identity surfaces, inference clouds, regional model labs, routers/gateways, and local runtimes.
- OpenRouter's catalog claim is attributed specifically to OpenRouter and is not presented as universal model access.

## Evidence

The current local renderer input is `C:\Users\david\Laboratory\nur-cli\.foglamp\scan.json`. It contains 42 nodes, 71 edges, and only five model nodes. Foglamp permits at most 60 nodes and 120 edges, leaving capacity for 18 nodes and 49 edges.

`src/providers.rs` currently defines 68 `Provider` entries and 45 distinct `default_model` values. `oauth_browser_provider_ids()` contains 11 IDs. `src/api/models.rs` performs live model discovery and deliberately avoids inventing a static model catalog the active credential may not access.

The scan lock points to the existing `nurcli-oxpatc` slug. Updating with its edit token preserves the URL already embedded at `https://www.nuroctane.xyz/cli`.

## Approaches considered

### Flat 18-node fan-out

Connect every new provider cohort directly to the Provider Catalog node. This maximizes immediate exposure but creates a visually noisy fan that competes with the runtime architecture.

### Provider directory replacement

Remove lower-priority architecture nodes and use individual vendor nodes. This produces a strong advertisement but weakens the scan's primary value as an explanation of how NurCLI works.

### Hierarchical provider ecosystem — selected

Use three provider-family hubs, each with five evidence cohorts. This consumes all 18 available node slots, preserves all existing architecture nodes, and enumerates every catalog entry in readable groups.

## Graph architecture

The existing 42 nodes and 71 edges remain. Add exactly 18 nodes and 18 edges, resulting in 60 nodes and 89 edges.

The existing `providers` node becomes the root of the advertising branch. Its visible subtitle changes from `60+ routes` to `68 routes · live models`. Its detail states the exact route/default/auth/protocol facts and continues to reference `src/providers.rs`.

Each provider-family branch contains one hub and five cohort nodes. The hub and its cohorts share a Foglamp `group`, producing three six-node vertical stacks.

### Branch 1: Direct providers — 21 routes

Group name: `Direct providers`

Hub:

- `direct_routes` — `Direct APIs` — subtitle `21 catalog routes`

Cohorts:

1. `frontier_core` — Meta, OpenAI, OpenAI Chat Completions, Anthropic, Google Gemini, Google Antigravity, xAI.
2. `independent_labs_a` — DeepSeek, Mistral, Cohere, AI21, Reka.
3. `independent_labs_b` — Inception Mercury, Writer Palmyra, Upstage Solar, Thinking Machines.
4. `identity_cloud_catalogs` — Hugging Face, Azure OpenAI, AWS Bedrock.
5. `github_model_surfaces` — GitHub Models, GitHub Copilot.

### Branch 2: Inference ecosystem — 29 routes

Group name: `Inference ecosystem`

Hub:

- `inference_routes` — `Inference ecosystem` — subtitle `29 catalog routes`

Cohorts:

1. `speed_leaders` — Groq, Cerebras, SambaNova, Lepton AI.
2. `open_clouds_a` — Together AI, Fireworks, DeepInfra, Novita AI, Hyperbolic.
3. `open_clouds_b` — Nebius AI Studio, Anyscale, OctoAI, NVIDIA NIM, Baseten.
4. `specialized_inference` — Perplexity, Friendli, Chutes.ai, Venice AI, Kluster.ai, Featherless, Targon.
5. `asian_model_labs` — Kimi Code, Moonshot Open Platform, Z.AI/Zhipu GLM, Alibaba Qwen, MiniMax, StepFun, Baichuan, 01.AI Yi.

### Branch 3: Routers and local — 18 routes

Group name: `Routers + local`

Hub:

- `router_local_routes` — `Routers + local` — subtitle `18 catalog routes`

Cohorts:

1. `one_key_routers` — OpenRouter, OmniRoute, Requesty, Glama.
2. `ai_gateways` — Unify, Portkey, LiteLLM Proxy, Vercel AI Gateway, Cloudflare AI Gateway.
3. `coding_observability` — NanoGPT, OpenCode Zen, Helicone AI Gateway, AI/ML API.
4. `local_desktop` — Ollama, LM Studio, Jan.
5. `local_servers` — llama.cpp, vLLM.

The three root edges use labels `21 direct routes`, `29 inference routes`, and `18 router/local routes`. Each hub calls its five cohorts. Every new node references `src/providers.rs:104` and has a detail no longer than 200 characters.

## Advertising copy

Update the project tagline to:

> 68-provider Rust coding agent with live models, OAuth, tools, vision, and skills

Set `stats.models` to 45, the exact number of distinct catalog default model IDs. Keep the three representative `topModels` entries and existing top tools/integrations; they remain highlights rather than exhaustive lists.

The Provider Catalog detail will explain that model availability is discovered live for the active credential. The OpenRouter cohort alone may state `400+ models, one key`, matching its source catalog note. No graph text will imply that every credential can access every model.

## Data flow

1. Parse the `PROVIDERS` constant in `src/providers.rs`.
2. Derive the provider count, distinct default-model count, and browser-auth count from source.
3. Assign each provider ID to exactly one approved cohort.
4. Generate the new hub/cohort nodes and edges in `.foglamp/scan.json`.
5. Validate the Foglamp contract and evidence invariants locally.
6. POST `{ data, editToken }` to `https://api.foglamp.dev/scan`.
7. Save the response back to `.foglamp/scan.lock.json` without printing the edit token.
8. Verify the returned slug remains `nurcli-oxpatc` and inspect the public graph.
9. Verify the existing `/cli` embed still resolves and displays the updated scan.

## Validation

A local validation step must fail unless all of the following are true:

- The source catalog has exactly 68 provider entries at implementation time.
- It has exactly 45 distinct default model IDs at implementation time.
- `oauth_browser_provider_ids()` has exactly 11 IDs.
- Every provider ID appears in exactly one cohort.
- No unknown provider ID appears in a cohort.
- The cohort counts are exactly 21, 29, and 18.
- Graph node IDs are unique.
- Every edge endpoint exists.
- There are at most 60 nodes and 120 edges.
- `topModels <= 3`, `topTools <= 10`, and `topIntegrations <= 10`.
- Project name, slug, tagline, node labels, subtitles, details, source references, group names, and edge labels satisfy Foglamp length limits.

After upload, verify by observation:

- The API response returns the existing slug and URL.
- The public scan contains all three provider-family groups and their 15 cohort nodes.
- Clicking cohort nodes exposes the complete provider names.
- The page still exposes the existing architecture flows.
- `https://www.nuroctane.xyz/cli` returns HTTP 200 and its iframe target loads the same Foglamp URL.

## Failure handling

- If source counts differ from 68/45/11, stop and reconcile the design with the current catalog rather than publishing stale claims.
- If validation finds a missing or duplicate provider, fix the cohort assignment before upload.
- If Foglamp returns HTTP 422, preserve the existing lock, fix only contract violations, revalidate, and retry.
- If the update returns a different slug, do not change the `/cli` embed automatically; stop and investigate the update token or API payload.
- Never log, commit, or include the edit token in a report.
- Keep the previous local scan data available until the public update is verified so it can be restored with the same edit token.

## Scope

In scope:

- Updating the gitignored `.foglamp/scan.json` and `.foglamp/scan.lock.json`.
- Updating the existing public scan in place.
- Evidence validation and public/embed verification.

Out of scope:

- NurCLI runtime behavior, provider implementations, authentication flows, or model discovery logic.
- The `/cli` React embed, styling, URL, or loading behavior.
- Claims about models not returned by a user's active provider credential.
