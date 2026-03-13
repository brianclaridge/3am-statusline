---
name: theme
description: Switch the active 3am-statusline color theme. Use when the user wants to change themes.
disable-model-invocation: true
allowed-tools: Read, Edit, Bash, Glob, Grep
argument-hint: "[theme-name]"
---

# Switch 3am-statusline theme

Change the `current_theme` value in the statusline config file.

## Steps

### 1. Find config file

Look for the statusline config in this order and use the first one found:

1. `config/statusline.yml`
2. `.claude/statusline.yml`

If no config file exists, tell the user to run `/3am-statusline:setup` first and stop.

### 2. Read available themes

Read the config file. Parse the `themes:` section to get the list of available theme names. If there is no `themes:` section (only a flat `theme:` key), tell the user their config uses the legacy format and needs `themes:` to switch themes.

### 3. Select theme

If `$ARGUMENTS` is provided and matches an available theme name, use it.

Otherwise, list the available themes and ask the user to pick one using AskUserQuestion.

### 4. Update config

Edit the `current_theme:` line in the config file to the selected theme name.

If there is no `current_theme:` line, add one directly above the `themes:` line.

### 5. Confirm

Tell the user which theme was activated. The change takes effect on the next statusline render.
