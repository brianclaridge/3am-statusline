# /// script
# requires-python = ">=3.10"
# ///
"""Check Claude API status via Statuspage.io."""

import json
import sys
import urllib.request

URL = "https://status.claude.com/api/v2/status.json"
INDICATORS = {"none": "🟢 ok", "minor": "🟡 slow", "major": "🔴 down", "critical": "🔴 down"}

try:
    with urllib.request.urlopen(URL, timeout=5) as resp:
        data = json.loads(resp.read())
    print(INDICATORS.get(data["status"]["indicator"], "⚪ ???"))
except Exception:
    print("⚪ ???")
    sys.exit(1)
