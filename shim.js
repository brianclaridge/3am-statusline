#!/usr/bin/env node
// Statusline OS shim — auto-detects platform and delegates to the right binary.
// Wired into .claude/settings.json as the statusLine command.
const { spawnSync } = require("child_process");
const { join } = require("path");
const { readFileSync } = require("fs");

const BINARIES = {
  "linux-x64": "3am-statusline-linux-x64",
  "linux-arm64": "3am-statusline-linux-arm64",
  "win32-x64": "3am-statusline-win-x64.exe",
  "darwin-x64": "3am-statusline-darwin-x64",
  "darwin-arm64": "3am-statusline-darwin-arm64",
};

const key = `${process.platform}-${process.arch}`;
const name = BINARIES[key];

if (!name) {
  process.stderr.write(`3am-statusline: unsupported platform ${key}\n`);
  process.exit(1);
}

const bin = join(__dirname, "bin", "release", name);
const stdin = readFileSync(0, "utf-8");
const result = spawnSync(bin, {
  input: stdin,
  stdio: ["pipe", "pipe", "pipe"],
  env: { ...process.env, FORCE_COLOR: "1" },
});

if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr && result.stderr.length > 0) process.stderr.write(result.stderr);
process.exit(result.status || 0);
