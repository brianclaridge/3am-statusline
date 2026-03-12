# Question Protocol

## Quick Reference

| Question | Answer |
| --------- | ------ |
| When to ask? | Ambiguity, missing context, multiple valid approaches |
| How to ask? | Always use `AskUserQuestion` tool |
| Format? | Structured options with descriptions |
| Limit? | 1-4 questions per invocation |

---

**MANDATORY** All clarifying questions MUST use the `AskUserQuestion` tool. Plain text questions are forbidden. No exceptions.

## Why

- Reduces back-and-forth
- Forces clear option framing
- User can respond with one click
- Keeps conversation focused

## When to Use

- Requirements are ambiguous
- Multiple valid implementation approaches exist
- User preferences are unknown
- Scope needs clarification
- Trade-offs require user decision

## When NOT to Use

- Task is unambiguous
- User gave explicit instructions
- Asking "is this okay?" (use `ExitPlanMode` instead)

## Task Completion Prompt

**MANDATORY** Every response that concludes work MUST end with an `AskUserQuestion` prompt. No exceptions. This applies to:

- Completing any task (trivial or significant)
- Reaching a natural stopping point
- Finishing requested changes
- Session boundaries

### Example

```json
{
  "question": "Task complete. What would you like to do next?",
  "header": "Next action",
  "options": [
    {"label": "Plan next task (Recommended)", "description": "Enter plan mode for next work item"},
    {"label": "Continue editing", "description": "Stay in edit mode for follow-up work"},
    {"label": "Done", "description": "End conversation gracefully"}
  ]
}
```

## Quality Standards

- **2-4 options per question** - not too few, not overwhelming
- **Clear descriptions** - explain implications of each choice
- **Recommend when confident** - put best option first with "(Recommended)"
- **Short headers** - 12 chars max for chip display
