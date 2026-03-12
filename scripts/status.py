# /// script
# requires-python = ">=3.10"
# ///
"""Check Claude API status via Statuspage.io."""

import json
import sys
import urllib.request

URL = "https://status.claude.com/api/v2/status.json"
INDICATORS = {"none": "healthy", "minor": "degraded", "major": "outage", "critical": "outage"}

try:
    with urllib.request.urlopen(URL, timeout=5) as resp:
        data = json.loads(resp.read())
    print(INDICATORS.get(data["status"]["indicator"], "unknown"))
except Exception:
    print("unknown")
    sys.exit(1)
