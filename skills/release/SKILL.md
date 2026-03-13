---
name: release
description: Build, test, commit, and release. Run after code changes are complete.
disable-model-invocation: true
---

# Release

Run the full release workflow:

1. Run `task test` and abort if any tests fail
2. Run `task build` to compile all platform binaries
3. Read `README.md` and `CLAUDE.md` — update any sections affected by recent changes
4. Run `git status` and stage ALL modified and untracked files (including `bin/release/*`)
5. Commit with a descriptive message following the project's `type: description` style
6. Run `git push origin main`
7. Run `task release` to tag and trigger the GitHub Actions pipeline
8. If `gh run watch` fails, check `gh run list --workflow=release.yml --limit=1` for status
9. Report the tag name and pipeline result
