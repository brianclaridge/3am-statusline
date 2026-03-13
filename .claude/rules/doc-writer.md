---
paths:
  - "src/**/*.rs"
  - "config/**/*.yml"
  - "skills/**/SKILL.md"
  - ".claude/agents/*.md"
---

# Doc-writer after code changes

**MANDATORY** After completing any code modification, the task completion AskUserQuestion MUST include an option to run the doc-writer agent. This applies to:

- Rust source file changes (`src/**/*.rs`)
- Config file changes (`config/**/*.yml`)
- New or modified skills
- New or modified agents

Include this option in the completion prompt:

```json
{"label": "Update docs", "description": "Run doc-writer agent to sync documentation with code changes"}
```

When the user selects this option, spawn the doc-writer agent with a summary of what changed.
