---
name: dialkit
description: Dial in interface parameters of any kind — add live, tweakable controls (dials/knobs/sliders/timelines) to a UI so values like animation timing, spacing, colors, and physics can be tuned in real time instead of edit-reload-guess. Use when building or refining an interface and the user wants to tune parameters live, expose a debug/tuning panel, or find the right values by feel. Multi-framework (React, Svelte, Vue, Solid).
---

# Dialkit — tune interface parameters live

[Dialkit](https://github.com/joshpuckett/dialkit) (`dialkit` on npm) is a library for
**dialing in interface parameters of any kind**: instead of editing a magic number,
reloading, and guessing, you bind the value to a live control and adjust it by feel in
the running UI. Great for animation timing/easing, spacing, sizes, colors, and
physics/spring params. Framework-agnostic core with React, Svelte, Vue, and Solid
bindings.

## When to reach for it

- The user is hunting for "the right feel" of a motion/layout value and keeps
  edit-reload-guessing → give them live dials instead.
- Building a debug/tuning panel for a prototype or design-engineering session.
- Exposing a small set of parameters a designer can tweak without touching code.

Pairs naturally with the `improve-animations`, `emil-design-eng`, and
`skeuomorphic-ui` skills — use those for *what* good values are, dialkit for *finding*
them fast.

## Install (per project)

```sh
npm install dialkit          # or: pnpm add dialkit / bun add dialkit
```

Import the binding for the project's framework (e.g. `dialkit/svelte`,
`dialkit/vue`; the core is framework-agnostic). Check the repo `example/` and README
for the current import paths and component/store API — read them before wiring, since
the surface evolves.

## How to apply

1. Identify the handful of parameters actually worth tuning (timing, easing, gap,
   radius, color, spring stiffness/damping) — don't expose everything.
2. Bind each to a dialkit control (dial/slider/timeline) with sensible min/max/step
   and a good default, so the live range is meaningful.
3. Let the user tune by feel; then **capture the dialed-in values back into code** as
   the new defaults/tokens and remove or gate the panel for production.
4. Keep the tuning panel out of production bundles (dev-only import / flag).

## Rules

- Tune, then commit the result — the panel is a means to find values, not ship them.
- Give every control a real range and unit; a dial from 0–1 with no context helps
  no one.
- Prefer a few high-leverage dials over a wall of sliders.

Upstream: https://github.com/joshpuckett/dialkit
