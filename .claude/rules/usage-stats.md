# Usage stats API

## Endpoint

```
GET https://api.anthropic.com/api/oauth/usage
```

## Authentication

```
Authorization: Bearer <oauth-access-token>
anthropic-beta: oauth-2025-04-20
Content-Type: application/json
```

Token source: `$CLAUDE_CONFIG_DIR/.credentials.json` → `claudeAiOauth.accessToken`

## Response format

```json
{
  "five_hour": {
    "utilization": 1.0,
    "resets_at": "2026-03-13T07:00:01.188734+00:00"
  },
  "seven_day": {
    "utilization": 23.0,
    "resets_at": "2026-03-13T04:00:00.188752+00:00"
  },
  "seven_day_oauth_apps": null,
  "seven_day_opus": null,
  "seven_day_sonnet": {
    "utilization": 2.0,
    "resets_at": "2026-03-15T19:00:00.188760+00:00"
  },
  "seven_day_cowork": null,
  "iguana_necktie": null,
  "extra_usage": {
    "is_enabled": true,
    "monthly_limit": 5000,
    "used_credits": 0.0,
    "utilization": null
  }
}
```

- `utilization` values are **percentages** (1.0 = 1%, 23.0 = 23%), NOT fractions
- `resets_at` is ISO 8601 with timezone offset
- Nullable claims return `null` when not applicable to the subscription tier

## Known issues

- Endpoint can return persistent HTTP 429 (`rate_limit_error`) — especially after many rapid calls
- Workaround: increase poll interval, handle 429 gracefully
- OAuth tokens expire; `expiresAt` field in credentials is millisecond timestamp

## Template fields (in statusline)

| Field | Source | Example |
| --- | --- | --- |
| `ratelimit.5h` | `five_hour.utilization` | `1` (percent) |
| `ratelimit.7d` | `seven_day.utilization` | `23` (percent) |
| `ratelimit.5h_eta` | `five_hour.resets_at` | `4h 32m` |
| `ratelimit.7d_eta` | `seven_day.resets_at` | `1d 22h` |
| `ratelimit.status` | HTTP status | `ok` or `rate_limited` |
