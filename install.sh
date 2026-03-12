#!/usr/bin/env bash
set -euo pipefail

# 3am-statusline installer
# Usage: curl -fsSL https://raw.githubusercontent.com/brianclaridge/3am-statusline/main/install.sh | bash

REPO="brianclaridge/3am-statusline"
INSTALL_DIR="${1:-$HOME/.local/share/3am-statusline}"

detect_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64)  echo "3am-statusline-linux-x64" ;;
        aarch64) echo "3am-statusline-linux-arm64" ;;
        *)       echo ""; return 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64)  echo "3am-statusline-darwin-x64" ;;
        arm64)   echo "3am-statusline-darwin-arm64" ;;
        *)       echo ""; return 1 ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*)
      echo "3am-statusline-win-x64.exe"
      ;;
    *)
      echo ""
      return 1
      ;;
  esac
}

main() {
  local binary_name
  binary_name="$(detect_platform)" || {
    echo "Error: unsupported platform $(uname -s)/$(uname -m)" >&2
    exit 1
  }

  echo "Installing 3am-statusline to $INSTALL_DIR"

  mkdir -p "$INSTALL_DIR/bin/release"

  # Download binary from latest release
  local release_url="https://github.com/$REPO/releases/latest/download/$binary_name"
  echo "Downloading $binary_name..."
  curl -fsSL "$release_url" -o "$INSTALL_DIR/bin/release/$binary_name"
  chmod +x "$INSTALL_DIR/bin/release/$binary_name"

  # Download shim.js
  local shim_url="https://raw.githubusercontent.com/$REPO/main/shim.js"
  curl -fsSL "$shim_url" -o "$INSTALL_DIR/shim.js"

  echo ""
  echo "Installed successfully."
  echo ""
  echo "Add to your project's .claude/settings.json:"
  echo ""
  echo "  \"statusLine\": {"
  echo "    \"type\": \"command\","
  echo "    \"command\": \"node $INSTALL_DIR/shim.js\","
  echo "    \"padding\": 1"
  echo "  }"
  echo ""
  echo "Optional: create .claude/statusline.yml for custom layouts."
  echo "See https://github.com/$REPO for configuration docs."
}

main
