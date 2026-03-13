# /// script
# requires-python = ">=3.10"
# ///
"""Compact git status: branch, ahead/behind, modified/staged/untracked counts."""

import subprocess
import sys


def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
    return r.stdout.strip() if r.returncode == 0 else ""


def main():
    branch = run(["git", "rev-parse", "--abbrev-ref", "HEAD"])
    if not branch:
        print("\u26aa no repo")
        return

    # Ahead/behind
    upstream = run(["git", "rev-parse", "--abbrev-ref", "@{u}"])
    ahead = behind = 0
    if upstream:
        lr = run(["git", "rev-list", "--left-right", "--count", f"HEAD...{upstream}"])
        if lr:
            parts = lr.split()
            ahead, behind = int(parts[0]), int(parts[1])

    # File counts from porcelain status
    status = run(["git", "status", "--porcelain"])
    modified = staged = untracked = 0
    for line in status.splitlines():
        if not line or len(line) < 2:
            continue
        x, y = line[0], line[1]
        if x == "?":
            untracked += 1
        else:
            if x in "MADRCT":
                staged += 1
            if y in "MADRT":
                modified += 1

    # Build output
    parts = [f"\U0001f33f {branch}"]

    if ahead or behind:
        ab = []
        if ahead:
            ab.append(f"+{ahead}")
        if behind:
            ab.append(f"-{behind}")
        parts.append("/".join(ab))

    counts = []
    if modified:
        counts.append(f"{modified}M")
    if staged:
        counts.append(f"{staged}S")
    if untracked:
        counts.append(f"{untracked}U")

    if counts:
        parts.append("| " + " ".join(counts))
    else:
        parts.append("| \u2728 clean")

    print(" ".join(parts))


try:
    main()
except Exception:
    print("\u26aa ???")
    sys.exit(1)
