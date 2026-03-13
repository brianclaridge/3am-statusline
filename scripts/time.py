# /// script
# requires-python = ">=3.10"
# ///
"""World clock: outputs current time in given timezones."""

import sys
from datetime import datetime
from zoneinfo import ZoneInfo

ZONES = {
    "PST": "America/Los_Angeles", "MST": "America/Denver",
    "CST": "America/Chicago",     "EST": "America/New_York",
    "UTC": "UTC",                 "GMT": "Europe/London",
    "CET": "Europe/Paris",        "JST": "Asia/Tokyo",
    "IST": "Asia/Kolkata",        "AEST": "Australia/Sydney",
}

def fmt(dt):
    h = dt.hour % 12 or 12
    m = dt.strftime("%M")
    ap = "a" if dt.hour < 12 else "p"
    return f"{h}:{m}{ap}"

args = sys.argv[1:]
if not args:
    print("Usage: time.py TZ [TZ ...]", file=sys.stderr)
    sys.exit(1)

results = []
for arg in args:
    iana = ZONES.get(arg.upper(), arg)
    try:
        now = datetime.now(ZoneInfo(iana))
    except Exception:
        results.append((arg, "???"))
        continue
    results.append((arg, fmt(now)))

if len(results) == 1:
    print(results[0][1])
else:
    print(" | ".join(f"{label} {t}" for label, t in results))
