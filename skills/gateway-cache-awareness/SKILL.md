---
name: gateway-cache-awareness
description: Protect prompt-cache hit rate when routing a coding agent (Claude Code, Codex, nur) through your own gateway/proxy. Use when configuring a custom base_url/gateway, debugging exploding token costs or slow first-token latency behind a proxy, or evaluating an "OpenAI/Anthropic-compatible" gateway. Watch the cache hit rate — most proxies silently break it.
---

# Gateway cache awareness

Source insight (Samantics / @inferencepoint): *"Almost every single thing I see
that claims to run a robust proxy is not aware of the cache-busting tricks Claude
Code and Codex have implemented in the harness when you try to hook it up to your
own gateways."* Keep an eye on your **cache hit rate**.

## Why it matters

Provider prompt caching (Anthropic `cache_control`, OpenAI automatic prefix
caching) is what keeps a long agent session cheap and fast: the stable prefix
(system prompt, tool schemas, prior turns) is billed at a large discount and
skips re-processing. A gateway that mutates the request — even harmlessly — moves
the cache breakpoint and you silently pay full price on every turn, with worse
first-token latency.

## What breaks the cache (audit your proxy for each)

- **Reordering or re-serializing JSON** — key order, whitespace, or dropping
  unknown fields changes the cached prefix byte-for-byte.
- **Stripping/rewriting `cache_control` breakpoints** or the `anthropic-beta` /
  prompt-caching headers.
- **Injecting a per-request value into the prefix** — a timestamp, request id,
  trace header echoed into the system block, or a rotating system-prompt banner.
- **Load-balancing across backends/regions/accounts** — caches are per-deployment;
  round-robin means a cold cache most turns. Pin a session to one backend
  (sticky routing on session id).
- **Normalizing models or params** that the harness relies on to stay identical
  across turns.
- **Re-chunking or re-encoding streaming** in a way that changes what gets cached.

## How to verify

- Read `cache_read_input_tokens` / `cache_creation_input_tokens` (Anthropic) or
  the cached-token counts (OpenAI) from the **usage** of each response. On turn 2+
  of a session, cache-read should dominate. If it's ~0, the proxy is busting it.
- Compare direct-to-provider vs. through-proxy on the same session: token cost and
  time-to-first-token should be close. A large gap = cache loss.
- In nur, watch `/status` and `/usage` (and the receipt) across turns — a flat
  cache-read line after several turns is the tell.

## Rules when putting a gateway in front of an agent

1. Pass the request body through **byte-identical**; do not re-serialize.
2. Preserve caching headers and `cache_control` breakpoints untouched.
3. Never inject per-request data into the cached prefix (system/tools/history).
4. Route a session **stickily** to one backend so the cache stays warm.
5. Make cache hit rate a first-class metric on the proxy dashboard, not an
   afterthought — regressions here look like a pricing/latency problem, not a
   correctness one, so they hide for a long time.

Upstream context: https://x.com/inferencepoint/status/2078950297884373051
