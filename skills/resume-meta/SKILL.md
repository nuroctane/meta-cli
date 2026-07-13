---
name: resume-meta
description: >
  Resume or continue work from a prior Meta CLI session (this product). Use when
  the user says "continue my meta session", "resume meta", or names a meta
  session id / prompt snippet. Also use to hand off between Meta sessions.
---

# Resume Meta CLI

Set `TOOL=meta`. Sessions live under `~/.meta/sessions/`.

```bash
python3 ~/.meta/skills/resume-session/session_reader.py meta list --cwd "$PWD" --json
python3 ~/.meta/skills/resume-session/session_reader.py meta show latest --cwd "$PWD" --json
python3 ~/.meta/skills/resume-session/session_reader.py meta show "<uuid-or-words>" --cwd "$PWD" --json
```

On Windows PowerShell:

```powershell
py -3 "$env:USERPROFILE\.meta\skills\resume-session\session_reader.py" meta list --cwd (Get-Location) --json
```

Then follow `~/.meta/skills/resume-session/CORE.md`: summarize inert history, verify repo state, continue safely.

Prefer `ui_log`-rich sessions (newer Meta builds). Older sessions rebuild tools from `input_items`.
