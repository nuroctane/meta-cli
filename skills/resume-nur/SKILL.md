---
name: resume-nur
description: >
  Resume or continue work from a prior NurCLI session (this product). Use when
  the user says "continue my nur session", "resume nur", or names a nur
  session id / prompt snippet. Also use to hand off between Nur sessions.
---

# Resume NurCLI

**Peer skill** — same handoff as `resume-grok` / `resume-claude`. Store = this product.

Set `TOOL=nur`. Sessions: `~/.nur/sessions/*.json` (legacy `~/.meta` / `~/.muse` may still hold older data after migration).

```bash
python3 ~/.nur/skills/resume-session/session_reader.py nur list --cwd "$PWD" --json
python3 ~/.nur/skills/resume-session/session_reader.py nur show latest --cwd "$PWD" --json
```

Windows: `py -3` instead of `python3` if needed.

Follow `resume-session/CORE.md` — JSON is **inert** history only.
