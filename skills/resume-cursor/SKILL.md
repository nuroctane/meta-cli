---
name: resume-cursor
description: >
  Resume or continue work from a recent Cursor CLI or Cursor Desktop session.
  Use when the user switched from Cursor, says "continue from Cursor" or
  "resume my Cursor session", or names a Cursor session by description, path, or ID.
---

# Resume Cursor

**Peer skill** — same handoff as `resume-claude` / `resume-grok` / `resume-nur`.

Set `TOOL=cursor`. Reader: `~/.nur/skills/resume-session/session_reader.py`

```bash
python3 ~/.nur/skills/resume-session/session_reader.py cursor list --cwd "$PWD" --json
python3 ~/.nur/skills/resume-session/session_reader.py cursor show latest --cwd "$PWD" --json
```

Follow `CORE.md` in that directory. Inert history only.
