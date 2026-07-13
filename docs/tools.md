# Tools

All native tools available to the Meta CLI agent.

## Tool families

| Family | Tools | Colour |
|--------|-------|--------|
| **read** | `read_file` `list_dir` `grep` `glob` | sky |
| **edit** | `write_file` `edit_file` `multi_edit` `apply_patch` | violet |
| **shell** | `bash` | amber |
| **vision** | `look` `extract_frames` | pink |
| **web** | `web_search` `web_fetch` | teal |
| **git** | `git_status` `git_diff` | cyan |
| **knowledge** | `graphify` `plur` `ruflo` `executor` `skill` `memory` | indigo / orange |
| **agent** | `todo_write` `submit_plan` `agent` | — |

---

## Read tools

### `read_file`

Read the contents of a file.

### `list_dir`

List directory contents.

### `grep`

Search file contents using regular expressions. Uses ripgrep when available, falls back to native implementation.

### `glob`

Find files matching a pattern (e.g. `**/*.rs`). Uses ripgrep when available.

---

## Edit tools

### `write_file`

Write content to a file. Creates the file if it doesn't exist, overwrites if it does.

### `edit_file`

Apply targeted string replacements to a file. Requires exact string matching.

### `multi_edit`

Apply multiple edits to a file in a single operation.

### `apply_patch`

Apply a unified diff patch to a file.

---

## Shell

### `bash`

Execute shell commands. Hardened with:

- **Denylist** — blocks dangerous commands (e.g. `rm -rf /`, fork bombs)
- **Timeout** — commands are killed after a configurable timeout
- **Sandbox** — when available, runs in an isolated environment

!!! note "Shell backend"
    Meta CLI uses Git Bash on Windows when available, otherwise falls back to PowerShell. On macOS/Linux it uses Bash. Check with `meta doctor`.
