---
name: setup
description: Configure 3am-statusline binary as the Claude Code status line. Run this after installing the plugin to wire the correct platform binary into settings.
disable-model-invocation: true
allowed-tools: Read, Write, Edit, Bash, Glob
---

# Setup 3am-statusline

Configure the 3am-statusline binary as the Claude Code status line for this system.

## Steps

### 1. Detect platform

Run `uname -s` and `uname -m` to determine the operating system and architecture.

Map the result to a binary name:

| OS (`uname -s`) | Arch (`uname -m`) | Binary |
| --- | --- | --- |
| Linux | x86_64 | `3am-statusline-linux-x64` |
| Linux | aarch64 | `3am-statusline-linux-arm64` |
| Darwin | x86_64 | `3am-statusline-darwin-x64` |
| Darwin | arm64 | `3am-statusline-darwin-arm64` |
| MINGW*, MSYS*, CYGWIN* | x86_64 | `3am-statusline-win-x64.exe` |

If the platform does not match any row, report the error and stop.

### 2. Resolve binary path

Construct the absolute path to the binary:

```
$CLAUDE_PLUGIN_ROOT/bin/release/<binary-name>
```

Where `$CLAUDE_PLUGIN_ROOT` is the root directory of this plugin (the directory containing `.claude-plugin/plugin.json`).

Verify the file exists and is executable. If not, report the error and stop.

### 3. Configure Claude Code settings

Read the user's `.claude/settings.json` (create the file and directory if they do not exist). Set the `"statusLine"` key to the absolute path of the binary resolved in step 2.

Preserve all other existing settings. Write the file back.

### 4. Copy default config (if needed)

Check whether either of these files exists in the current project:

- `config/statusline.yml`
- `.claude/statusline.yml`

If neither exists, copy `$CLAUDE_PLUGIN_ROOT/config/statusline.yml` to `config/statusline.yml` in the current project. Create the `config/` directory if needed.

### 5. Report result

Print a summary of what was configured:

- Platform detected
- Binary path set in settings
- Whether the default config was copied
