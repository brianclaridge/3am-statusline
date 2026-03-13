# Documentation maintenance

## Pre-commit requirement

**MANDATORY** Before every commit, read and update these files if the changes affect them:

1. `README.md` — terse user-facing install and quick start
2. `CLAUDE.md` — architecture, file paths, build commands, stack
3. `docs/configuration.md` — config sections, template tokens, fields, color fields
4. `docs/events.md` — built-in events, custom events, wiring

## What triggers updates

- New or renamed source files — update `CLAUDE.md` architecture tree
- New CLI subcommands or flags — update `CLAUDE.md` quick reference and `docs/events.md`
- New config fields or template tokens — update `docs/configuration.md`
- New dependencies — update `CLAUDE.md` stack section
- Changed build steps or cross-compile targets — update `CLAUDE.md`
- New or renamed event subcommands — update `docs/events.md` and `CLAUDE.md` architecture
- New available fields — update `docs/configuration.md` available fields section

## Process

1. Read affected docs before staging
2. Edit affected sections to reflect the new state
3. Run `git status` and stage **all** modified and untracked files — never leave working tree dirty after a commit
4. This includes `bin/release/*` binaries after builds, config changes, and any other generated files
