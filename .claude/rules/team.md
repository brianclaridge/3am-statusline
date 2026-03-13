# Team workflow

## Available agents

| Agent | Trigger | Purpose |
| --- | --- | --- |
| `doc-writer` | After code changes | Sync docs with source |
| `release-manager` | Build/commit/release | Full release pipeline |
| `rust-code-reviewer` | After Rust changes | Convention checks, safety review |

## When to delegate

**MANDATORY** Proactively use agents for their designated tasks. Do not do their work manually when an agent exists.

- **After any code change** — offer to spawn `doc-writer` to update docs
- **After Rust code changes** — spawn `rust-code-reviewer` to check conventions and safety
- **When building/releasing** — delegate to `release-manager`
- **For codebase exploration** — use the `Explore` subagent for broad searches

## Parallel execution

When multiple agents can run independently, spawn them in parallel. Example: after a code change, run `rust-code-reviewer` and `doc-writer` concurrently.

## Do not

- Do doc updates manually when `doc-writer` is available
- Skip code review when Rust files were modified
- Run build/release steps manually when `release-manager` can handle it
