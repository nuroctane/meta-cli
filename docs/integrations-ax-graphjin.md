# Evaluating Ax and GraphJin for NurCLI

An assessment of two candidate integrations — [Ax](https://axllm.dev/) and
[GraphJin](https://graphjin.com/) — against what NurCLI already ships, with a
recommendation for each.

Researched July 2026 against `ax-llm/ax` (crates.io `axllm` 23.0.3) and
`dosco/graphjin` (`main`).

**Bottom line**

| | Verdict | Why |
| :--- | :--- | :--- |
| **Ax** | **Don't link the crate. Steal two ideas.** | Its core layer is the one thing nur is already strongest at. Its *unique* value — typed signatures and the GEPA optimizer — is worth building natively on top of `nur bench`. |
| **GraphJin** | **Integrate as a first-class tool.** | Fills a capability nur simply does not have (governed access to live data), and its evidence ledger is a real answer to a real failure mode. |

!!! success "Shipped"
    Both verdicts have been acted on.

    - **`graphjin` tool** + `/graphjin` (`/gj`) — see [Tools](tools.md#graphjin).
      Detect-only in `ecosystem ensure`: it is not installed for you, because
      without a configured database it does nothing.
    - **`nur bench optimize`** (`/bench optimize`) — the GEPA-shaped optimiser
      over the existing bench harness, in `src/gepa.rs`.

    Still open: the native signature layer, and an Ax authoring skill.

---

## 1. The baseline: what nur already owns

Judging an integration means knowing what it would displace.

| Layer | Where it lives today | Maturity |
| :--- | :--- | :--- |
| Provider transport | `src/api/{client,chat,anthropic,sse}.rs` — Responses, Chat Completions, and Anthropic Messages wire formats, SSE streaming, retries | ~30 providers incl. local runtimes |
| Auth | `src/oauth/`, `src/auth.rs` — API keys **and** OAuth device/browser flows per provider | Beyond most frameworks |
| Routing | `src/api/failover.rs`, `src/api/fusion.rs` — failover chains, multi-model fusion | Shipping |
| Cost/usage | `src/pricing.rs`, `src/usage.rs` | Shipping |
| Agent loop | `src/agent/loop.rs` — tools, parallel-safe batching, permission modes, hooks, receipts | Shipping |
| Evaluation | `src/bench.rs` — replays recorded task trajectories across models in isolated git worktrees, scores pass/fail + wall time + tokens | **Shipping, and unusual** |
| Code knowledge | `graphify` tool — code AST → graph, with `query` / `path` / `explain` / `affected` | Shipping, offline, no API key |
| Shared memory | `plur`, `ruflo` tools | Shipping |
| MCP | **Proxied** through the Executor gateway (`executor` tool). No native MCP client. | Indirect |
| Databases | **Nothing.** | — |

Two rows matter most below: nur has **no data access at all**, and it has an
**eval harness but no optimizer**.

---

## 2. Ax

### What it actually is

Ax is a signature-driven LLM framework. You declare
`"review:string -> sentiment:class \"positive, negative, neutral\""` and it
generates the prompt, output parser, validator, and retry loop. On top sit
`AxAgent` (a distiller → executor → responder pipeline with a sandboxed JS
runtime), `AxFlow` (typed workflow graphs), and **GEPA**, an optimizer that
tunes prompts and agent configs against evals using a Pareto frontier.

TypeScript is the reference runtime. Everything else is generated from a
portable IR (AxIR) and checked in under `packages/<language>`.

### The Rust reality

There *is* a Rust crate, and it is current:

```
axllm = "23.0.3"          # updated 2026-07-21
deps: reqwest, serde, serde_json, regex
      rquickjs (optional, "runtime-quickjs")   # JS sandbox — needed for AxAgent
      tungstenite (optional, "realtime")
total downloads: 585
```

Read those two lines together: **585 lifetime downloads** for generated code
whose semantics are validated primarily through the TypeScript reference. For a
dependency that would sit on the request path of every turn, that is not a
maturity level worth betting the agent loop on.

### Overlap analysis

| Ax capability | nur equivalent | Who wins |
| :--- | :--- | :--- |
| Provider abstraction | `src/api/*` + OAuth + failover + fusion + pricing | **nur, decisively.** Ax has no OAuth flows, no failover chains, no local-runtime handling. |
| Streaming | `src/api/sse.rs` + `StreamAccumulator` | nur (already handles provider quirks like servers ignoring `stream:true`) |
| MCP → typed functions | Executor gateway | Tie — both are indirect |
| Telemetry (40+ OTel metrics) | `src/usage.rs`, receipts | Ax, marginally — but nur's is task-shaped, not request-shaped |
| Multi-modal I/O | `src/tools/media.rs` (+ the vision-capability fallback added alongside this doc) | nur |
| **Signatures** | *nothing* | **Ax** |
| **GEPA optimizer** | *nothing* (but `nur bench` is the missing half) | **Ax** |
| AxFlow workflow graphs | `todo_write` + `submit_plan` + subagents | Different shapes; nur's is agent-native |

Adopting `axllm` would mean importing a second, weaker HTTP/provider stack to
get at two features that sit above it. That is the wrong trade.

### What is genuinely worth taking

**a. Typed signatures.** nur does ad-hoc structured extraction in several
places (skill intent classification, plan parsing, fusion synthesis) with
hand-rolled prompts and hand-rolled parsing. A small native
`signature!("input -> label:class \"a,b,c\"")` layer that emits the prompt,
parses the response, validates, and retries once on a parse failure would
consolidate those and make new ones cheap. This is perhaps 300 lines against
the existing `ResponseRequest` — no new dependency.

**b. GEPA on top of `nur bench`.** This was the strongest finding in the whole
evaluation, and it is now `src/gepa.rs`. GEPA needs three things: a candidate
space (prompt variants), an eval set, and a scorer. **nur already had the
expensive two.** `src/bench.rs` replays recorded real tasks across models in
isolated git worktrees and scores pass/fail plus wall time plus tokens; the
optimiser reuses that path unchanged, passing a candidate instruction prefix.

```bash
nur bench optimize all --gens 3 --pop 4     # or /bench optimize all 3 4
```

Each generation scores every candidate on the task set, keeps the **Pareto
front** over (pass rate ↑, seconds ↓, tokens ↓), and asks the model to improve
a front member — showing it the measured scores *and the actual failures*, which
is what separates reflection from asking for synonyms. Parents are drawn
round-robin across the whole front rather than best-first, so the search does
not collapse onto one scalar. The winner is written to
`~/.nur/bench/optimized/<task>.md`; nur does not apply it automatically.

Two deliberate choices worth knowing:

- **Timing ties.** A sub-5% difference in seconds counts as equal, so timing
  jitter cannot evict a candidate that is genuinely just as good.
- **The baseline can win.** If no proposal beats the untuned prompt, the run
  says so and writes nothing. That is a result, not a failure.

It costs real tokens: `tasks × population × generations` full agent runs.

**c. An authoring skill.** Ax's real audience is TypeScript and Python. The
cheapest genuine "Ax integration" for a coding agent is *being good at writing
Ax programs* — a skill covering signatures, AxFlow, and GEPA wiring, in the
same shape as the existing skill packs. Hours, not weeks.

### Verdict

**Do not add `axllm` as a dependency.** Build (a) natively, treat GEPA as the
design reference for (b), and ship (c) as a skill. Revisit the crate only if
the Rust package gains real adoption *and* nur ever wants Ax's JS-sandboxed
agent runtime — which would duplicate nur's own agent loop anyway.

---

## 3. GraphJin

### What it actually is

A Go single binary that compiles GraphQL to optimised SQL across 12+ engines
(Postgres, MySQL, MongoDB, SQLite, Oracle, MSSQL, Snowflake, BigQuery,
Cassandra, …) with no N+1 queries. Around that core it has grown into an
agent-facing data platform:

- `graphjin mcp --path ./config` — **stdio MCP server** (there is already a
  Claude Code plugin in-tree at `claude-plugins/graphjin-mcp`)
- `graphjin serve` — HTTP/WS/SSE service, web console, and
  `POST /api/v1/agent`
- **`gj_*` system roots** — `gj_catalog` (discovery), `gj_security`,
  `gj_code`, `gj_config`, `gj_runtime`, served alongside application data
- **CodeSQL** — the source tree indexed as queryable tables behind `gj_code`
- Install: `npm install -g graphjin`, Homebrew, Scoop, .deb/.rpm, Docker

### The part that matters: catalog-first, with enforcement

GraphJin's contract is *discover before you act*: search `gj_catalog`, inspect
the evidence, check `gj_security` before anything risky, then execute. The
guards are Go-side, not prompt-side. An `answered` result is **downgraded to
`blocked`** when a required discovery step was skipped, and model-claimed
actions never count — only real tool results do.

That is a structurally different guarantee from "we told the model to be
careful", and it is aimed squarely at the failure mode nur cannot currently
defend against, because nur has no data surface to defend.

There is also an operator kill-switch, `agent.read_only: true`, which rejects
mutations at execution regardless of caller role — a clean mapping onto nur's
plan/manual/auto permission modes.

### Overlap analysis

| GraphJin capability | nur equivalent | Who wins |
| :--- | :--- | :--- |
| **Query live databases** | *nothing* | **GraphJin — pure gap fill** |
| RLS, allow-lists, saved queries, RBAC | `permissions.toml` (filesystem/command scope only) | GraphJin, for data |
| **Evidence ledger** | receipts (`src/agent/receipt.rs`) record *that* a tool ran | **GraphJin** — receipts prove execution, evidence proves *grounding* |
| `gj_code` / CodeSQL | `graphify` | **Split — see below** |
| Server-side agent (`ask_graphjin_agent`) | nur's own loop | nur, for coding; GraphJin, for data questions |
| MCP transport | Executor gateway | Direct CLI wrapper would be cleaner |

### `gj_code` vs `graphify` — the one real contest

They index the same thing and answer different questions.

- **`graphify`** is a *graph*: BFS, shortest path between two concepts,
  reverse-impact (`affected`). It runs fully offline via local AST extraction,
  needs no API key, and has no service to stand up. For "what breaks if I
  change this" and "how do these two subsystems connect", traversal is the
  right primitive.
- **`gj_code`** is a *relation*: SQL over code, which means it **joins** —
  code to live data, to config, to security posture, to runtime events. "Which
  tables does this handler touch, and what is their row count in prod" is a
  question `graphify` structurally cannot answer and `gj_code` answers in one
  query.

**Recommendation: keep both, and say so in the tool descriptions.** Graphify
stays the default for offline architecture questions — it is lighter and has no
prerequisites. GraphJin owns anything that joins code to data. Replacing
Graphify with `gj_code` would trade a zero-setup local capability for one
requiring a configured service, and lose path/affected traversal.

### Verdict

**Integrate.** It fills a genuine hole rather than competing with a strength,
it installs through the same npm path nur already uses for `plur`, `ruflo`,
`akarso`, and `executor`, and its safety model maps onto nur's permission
modes.

### The shape it took

`graphjin cli …` is not a local query runner — it is an **MCP/JSON-RPC client**
against a running GraphJin server, configured once with
`graphjin cli setup <url>` and storing server + token in
`~/.config/graphjin/client.json`. So the tool passes no `--path`, holds no
credentials, and drives the CLI the operator already pointed at their server.

Where GraphJin ships an ergonomic subcommand it is used directly (`explain`,
`query exec` / `query run`, `audit`, `health`); discovery and the server-side
agent go through the **tool-name parity surface**,
`graphjin cli <tool> --args '{…}'`, whose names are guaranteed stable by the
server's `MCPAllToolNames` (`query_catalog`, `graphql_help`,
`ask_graphjin_agent`).

| action | read-only | invocation |
| :--- | :---: | :--- |
| `status` | ✓ | `version` + client config + `cli health` |
| `catalog` | ✓ | `cli query_catalog --args '{"search":…}'` |
| `schema` | ✓ | `cli query_catalog --args '{"ids":[…]}'` |
| `help` | ✓ | `cli graphql_help` |
| `explain` | ✓ | `cli explain <graphql>` — compiled SQL, unexecuted |
| `query` | ✓ | `cli query exec <doc>` or `cli query run <name>` |
| `security` | ✓ | `cli audit [role]` — role permission matrix |
| `ask` | ✓ | `cli ask_graphjin_agent --args '{"instruction":…}'` |
| `mutate` | ✗ | `cli query exec` — gated, blocked in plan mode |

Two things the wrapper gets right on purpose:

1. **The evidence is surfaced, not flattened.** GraphJin returns
   `status`/`answer`/`data`/`evidence`/`actions`/`next`. Returning only
   `answer` would throw away the entire reason to prefer GraphJin, so the raw
   envelope passes through and a `blocked` or `refused` status is relabelled as
   the failure it is — a model reads a bare JSON blob as success otherwise.
2. **Honest about where enforcement lives.** nur's read-only classification is
   its permission layer, not a security boundary; the server's own roles, RLS,
   and `agent.read_only` kill-switch are what actually stop a write. The tool
   docs say so rather than implying nur is the guard.

`ecosystem ensure` **detects** GraphJin rather than installing it — unlike every
other component, it does nothing without a configured database, and it is a far
heavier install than a memory CLI.

---

## 4. Order of work

1. ~~**GraphJin tool**~~ — done. Biggest capability delta, lowest architectural
   risk, fits an existing pattern.
2. ~~**GEPA-style optimizer over `nur bench`**~~ — done. Highest leverage per
   line, because the expensive half already existed.
3. **Native signature layer** — consolidates the ad-hoc structured extraction
   in skill-intent classification, plan parsing, and fusion synthesis. Roughly
   300 lines against `ResponseRequest`; no new dependency.
4. **Ax authoring skill** — cheap, and the only form of "Ax integration" whose
   value does not depend on an unproven Rust port.

---

## 5. Postscript: what a third repo was worth

While evaluating these, [hermes-agent](https://github.com/NousResearch/hermes-agent)
was read for transferable ideas. Two paid off immediately:

- **`pane-shell/tree/grid-model.ts`** — its FancyZones-derived zone grid is the
  basis for `src/tui/grid.rs`, which lays out the `/swarm` subagent card.
- **`agent/lmstudio_reasoning.py`** — reading it surfaced a live bug rather than
  a feature: nur had no handling for reasoning from local runtimes. Models like
  Qwen3 and DeepSeek-R1 behind llama.cpp emit their chain of thought as
  `<think>…</think>` *inside* `content`, and LM Studio/vLLM/DeepSeek use a
  separate `reasoning_content` delta. nur handled neither, so local reasoning
  either landed in the answer or vanished — which is why the TUI reported
  "thought 0ms" on a local model. Both paths are now parsed in
  `src/api/chat.rs`, including markers split across SSE frames.
