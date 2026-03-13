# Marketplace manifest schema

Extracted from Claude Code 2.1.74 binary. The manifest uses strict validation — no extra keys allowed.

## Marketplace manifest

```jsonc
{
  "name": "string",                    // marketplace name
  "description": "string",            // what this marketplace offers
  "owner": { "name": "string" },      // marketplace owner
  "plugins": [ PluginEntry ]          // array of plugin entries
}
```

## Plugin entry

```jsonc
{
  "name": "string",                    // plugin identifier
  "description": "string",            // one-line summary
  "source": PluginSource              // discriminated union on "source" key
}
```

## Plugin source (discriminated union on `source`)

| Type | Fields | Description |
| --- | --- | --- |
| `github` | `repo`, `ref?`, `sha?` | GitHub repository (`owner/repo` format) |
| `url` | `url`, `ref?`, `sha?` | Full git URL (`https://` or `git@`) |
| `git-subdir` | `url`, `path`, `ref?`, `sha?` | Subdirectory within a git repo |
| `npm` | `package`, `version?`, `registry?` | npm package |
| `pip` | `package`, `version?`, `registry?` | Python package |

### Examples

```json
{ "source": "github", "repo": "owner/repo" }
{ "source": "github", "repo": "owner/repo", "ref": "v1.0.0", "sha": "abc1234" }
{ "source": "url", "url": "https://github.com/owner/repo.git", "ref": "main" }
{ "source": "git-subdir", "url": "owner/repo", "path": "tools/my-plugin" }
{ "source": "npm", "package": "@scope/plugin", "version": "^1.0.0" }
{ "source": "pip", "package": "my-plugin", "version": ">=1.0.0, <2.0.0" }
```

## Marketplace source (for `claude plugin marketplace add`)

Discriminated union on `source` key in the marketplace registry (not the same as plugin source above).

| Type | Fields | Description |
| --- | --- | --- |
| `github` | `repo`, `ref?`, `path?`, `sparsePaths?` | GitHub repo containing marketplace.json |
| `url` | `url`, `headers?` | Direct URL to marketplace.json |
