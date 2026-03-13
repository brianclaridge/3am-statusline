# Auto memory

Guidelines for managing persistent memory in `.data/claude-data/projects/.../memory/`.

## How it works

- `MEMORY.md` loads at session start — **only the first 200 lines**
- Topic files (e.g., `debugging.md`, `patterns.md`) load on demand
- Memory is per-project (scoped to git repo), machine-local
- Claude reads and writes memory files throughout the session

## What belongs in MEMORY.md

- Stable project patterns confirmed across multiple sessions
- Key file paths and architecture decisions
- Build commands and workflow preferences
- Solutions to recurring problems

## What does NOT belong in MEMORY.md

- Session-specific context (current task, in-progress work)
- Unverified conclusions from reading a single file
- Content that duplicates CLAUDE.md or rules
- Speculative or temporary notes

## Organization

- Keep MEMORY.md as a concise index — link to topic files for detail
- Organize semantically by topic, not chronologically
- Update or remove memories that turn out to be wrong
- Check for existing entries before writing duplicates

## When to update

- User explicitly asks to remember something — save immediately
- User corrects something from memory — fix at the source before continuing
- A pattern is confirmed across 2+ sessions — save it
- A saved fact becomes outdated — update or remove it

## Commands

- `/memory` — view loaded files, toggle auto memory, open memory folder
- `autoMemoryEnabled: false` in settings to disable
