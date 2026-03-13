# Release workflow

## Commit completeness

**MANDATORY** Every commit must include ALL modified and untracked files. Run `git status` before committing and stage everything. This includes:

- `bin/release/*` binaries after `task build`
- Config file changes (`.claude/settings.json`, `Taskfile.yml`)
- Generated or updated docs

Never leave the working tree dirty after a commit.

## Release sequence

1. `task test` — abort on failure
2. `task build` — compile all platform binaries
3. Stage and commit all changes (code, docs, binaries)
4. `git push origin main`
5. `task release` — tags with `YYYY.MM.DD-gitsha` and triggers GitHub Actions

## Tag format

`YYYY.MM.DD-<short-sha>` — generated automatically by `task release`.

## Known issue

`task release` may fail on `gh run watch` in non-interactive shells. The tag is still pushed and the pipeline runs. Verify with `gh run list --workflow=release.yml --limit=1`.
