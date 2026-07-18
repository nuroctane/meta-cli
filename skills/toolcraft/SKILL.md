---
name: toolcraft
description: >-
  Pointer skill for Toolcraft — design-app scaffolding and craft tooling docs.
  Use when the user mentions Toolcraft, design-app scaffold, craft tooling,
  or wants the upstream Toolcraft workflow. Does not embed the full guide;
  load the linked docs when activated.
---

# Toolcraft (pointer)

**Do not invent a full Toolcraft playbook from memory.** This skill is a thin
router so the catalog stays small and context stays lean.

## When this applies

- User says *toolcraft*, *Toolcraft*, *design app scaffold*, *craft tooling*
- User wants the Toolcraft design/scaffold workflow for an app

## What to do

1. Open / fetch the upstream docs and follow them:
   - **Primary:** https://github.com/toolcraft-ai/toolcraft (or the user's linked Toolcraft repo/docs if they provided one)
2. Prefer `skill(action=read, name=toolcraft)` only for this pointer; then
   **browse or fetch the live docs** for the actual steps.
3. If the user pasted a Toolcraft URL or local path, that source wins over any default.

## What not to do

- Do **not** dump long generic design-system essays unless the docs say so
- Do **not** load unrelated design skills unless the user also asked for them
- Do **not** treat OAuth / auth provider work as Toolcraft (different domain)

## Context discipline

Nur keeps skills as a **catalog of name + description** until a match fires.
Only then is a body injected. This pointer is intentionally short so activation
costs almost nothing; the heavy content lives in external docs on demand.
