# Documentation maintenance

## Pre-commit requirement

**MANDATORY** Before every commit, read and update these files if the changes affect them:

1. `README.md` — user-facing install, config, usage, and feature docs
2. `CLAUDE.md` — architecture, file paths, CLI reference, and build instructions

## What triggers updates

- New or renamed source files — update `CLAUDE.md` architecture tree
- New CLI subcommands or flags — update both `README.md` usage and `CLAUDE.md` quick reference
- New config fields or template tokens — update `README.md` tables
- New dependencies — update `CLAUDE.md` stack section
- Changed build steps or cross-compile targets — update both files
- New or renamed event subcommands — update `README.md` events section and `CLAUDE.md` architecture

## Process

1. Read `README.md` and `CLAUDE.md` in full before staging
2. Identify sections affected by the current changes
3. Edit affected sections to reflect the new state
4. Stage the doc updates in the same commit as the code changes
