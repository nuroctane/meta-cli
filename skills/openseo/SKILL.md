---
name: openseo
description: SEO research and audits via OpenSEO — an open-source Semrush/Ahrefs alternative exposed as an MCP server. Use when the user wants keyword research, backlink analysis, rank tracking, site audits, competitor analysis, or search-console insights. Covers connecting the OpenSEO MCP and running the workflows through it.
---

# OpenSEO — SEO data for the agent (via MCP)

OpenSEO ([openseo.so](https://openseo.so), open-source) exposes an **MCP server** so
the agent can pull real SEO data: keyword research, backlinks, rank tracking, site
audits, competitor analysis, and Search Console. There is **no CLI** — it is an
MCP + hosted/self-hosted web app. In nur, SEO tool calls go through the MCP layer
(the `executor` gateway), and results also show in the OpenSEO dashboard.

## Setup (once)

1. Sign up (free, no card): https://app.openseo.so/sign-up — or self-host
   (Docker or Cloudflare; needs a DataForSEO API key for data).
2. Connect the MCP to your agent following https://openseo.so/docs/mcp
   (register the OpenSEO MCP endpoint; in nur, add it through the `executor`
   tool / `/mcp` gateway, then its tools become callable).
3. Optional: install OpenSEO's own Agent Skills — https://openseo.so/docs/skills/setup

`/openseo` opens the dashboard and points at these docs.

## Workflows

- **Keyword research** — seed a topic → volume, difficulty, related/long-tail terms;
  cluster by intent before recommending targets.
- **Competitor analysis** — a competitor domain → their ranking keywords and gaps
  you can win.
- **Backlinks** — referring domains, anchor profile, toxic-link flags.
- **Rank tracking** — track target keywords over time; report movement, not vanity.
- **Site audit** — crawl for technical issues (indexability, meta, speed, broken
  links) and prioritize by impact.
- **Search Console** — real impressions/clicks/positions to ground the above.

## Rules

- Prefer **MCP tool calls** for live data over guessing metrics from memory — never
  fabricate volumes, difficulty, or backlink counts.
- Data costs money (DataForSEO when self-hosting) — batch queries and reuse results
  within a task rather than re-fetching.
- Turn findings into a prioritized, intent-clustered action list, not a raw dump.
- If the MCP isn't connected, walk the user through https://openseo.so/docs/mcp
  rather than inventing numbers.

Upstream: https://github.com/every-app/open-seo · Docs: https://openseo.so/docs/mcp
