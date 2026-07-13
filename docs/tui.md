# TUI

The Meta-blue terminal UI for interactive sessions.

## Opening the TUI

```bash
meta                    # fresh session
meta "fix the bug"      # start with a prompt
meta -c                 # continue last session
meta -r <session-id>    # resume specific session
```

---

## Keyboard shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `↑` `↓` · mouse wheel · drag scrollbar | Scroll transcript |
| **Drag on chat text** | Select + auto-copy |
| **Click `↓ N · End`** | Jump to latest message |
| Click card / `▸` | Peek / expand a card |
| `p` (empty input) | Peek latest tool result |
| `e` (empty input) | Expand latest card |

### Input

| Key | Action |
|-----|--------|
| **Ctrl+A** | Select-all input (or whole transcript if input empty) |
| **Ctrl+C** | Copy selection (transcript or input); else interrupt / double-tap quit |
| **Ctrl+V** | Paste into input |
| **Ctrl+X** | Cut input selection (or whole input) |
| `Enter` | Send message |
| `Shift+Enter` | Newline in input |

### Control

| Key | Action |
|-----|--------|
| `Shift+Tab` | Cycle permission mode (manual → plan → auto → manual) |
| `Ctrl+R` | Open sessions browser |
| `Esc` | Close peek, then cancel current turn |

### Approval

When the agent requests permission to run a write/shell tool:

| Key | Action |
|-----|--------|
| `y` | Approve this one time |
| `a` | Always approve this tool (for this session) |
| `n` | Deny |

---

## Slash commands

Type `/` in the input to see available commands.

### Permission and mode

| Command | Purpose |
|---------|---------|
| `/mode` | Show current permission mode |
| `/plan` | Switch to plan mode (read-only) |
| `/manual` | Switch to manual mode (approval required for writes) |
| `/auto` | Switch to auto mode (auto-approve all) |

### Session and state

| Command | Purpose |
|---------|---------|
| `/sessions` | Open sessions browser (same as Ctrl+R) |
| `/resume` | Resume a session |
| `/todos` | Show current todos |
| `/clear` | Clear current screen |
| `/new` | Start a new session |

### Knowledge stack

| Command | Purpose |
|---------|---------|
| `/graphify` | Query the code knowledge graph |
| `/plur` | Search shared engram memory |
| `/ruflo` | Search vector memory |
| `/skills` | List available skills |
| `/ecosystem` | Show ecosystem status |
| `/memory` | Show session memory |

### Model and context

| Command | Purpose |
|---------|---------|
| `/model` | Change model (e.g. `/model muse-spark-1.1`) |
| `/effort` | Change reasoning effort |
| `/compact` | Manually compact context |
| `/usage` | Show token usage and cost |

### Project and shell

| Command | Purpose |
|---------|---------|
| `/init` | Initialise project instructions (`META.md`) |
| `/config` | Open config |
| `/help` | Show keys + commands reference |
| `/login` | Authenticate (masked key entry) |
| `/logout` | Clear stored API key |
| `/exit` | Quit Meta CLI |

---

## Visual design

### Colour system

Tool cards are colour-coded by family:

| Family | Hue | Tools |
|--------|-----|-------|
| read | sky | `read_file` `list_dir` `grep` `glob` |
| edit | violet | `write_file` `edit_file` `multi_edit` `apply_patch` |
| shell | amber | `bash` |
| vision | pink | `look` `extract_frames` |
| web | teal | `web_fetch` `web_search` |
| git | cyan | `git_status` `git_diff` |
| knowledge | indigo / orange | `graphify` `plur` `ruflo` `skill` `memory` |

### Thought cards

The model's reasoning is displayed in **violet thought cards** that are collapsed by default. Click to expand.

### Duration chips

Each tool call shows a duration chip (e.g. `1.2s`) so you can see where time is spent.

### Approval mini-diff

When a write tool requests approval, the TUI shows a compact diff preview of what will change.

### Sessions browser

Open with `Ctrl+R` or `/sessions`. Browse recent sessions with a prompt-first picker — see the first user message of each session to find the one you want.

### Sticky prompt

The PROMPT banner stays visible at the top as you scroll, so you always see the context of the conversation.

### Splash screen

The splash shows the active model title. The rest of the chrome is model-agnostic.

---

## ADE / Orca integration

Meta CLI writes live status files for host panels:

| Path | Contents |
|------|----------|
| `~/.meta/status.json` | Live tokens, cost, model, state |
| `~/.meta/usage.jsonl` | Per-request log |
| `~/.meta/ade.json` | Discovery manifest |

```bash
meta install-hook           # install Orca agent hook
orca terminal create --command meta   # launch in Orca
```
