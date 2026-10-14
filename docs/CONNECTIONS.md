# API connection and credential checks

```sh
mediaagent doctor                         # Offline key-presence check
mediaagent doctor --live                  # Check all five providers
mediaagent doctor --live --provider openai,google --timeout-seconds 10
mediaagent doctor --live --json --non-interactive
```

`doctor` stays offline unless `--live` is explicitly supplied. No initialization is
required. The target project's root `.env` and process environment follow the same
credential rules as generation: process values win, including an explicitly empty
value; ancestor projects are never searched. Only selected provider keys are loaded.
Never put keys in flags, briefs, or config YAML.

Live diagnostics make at most one authenticated HTTPS GET per selected configured
provider, with up to five independent checks in parallel. `--provider` is repeatable
or comma-separated and duplicate selections are removed. The default selection is
all five, including missing keys. Each check has a 1–60 second timeout (default 10),
a 1 MiB response ceiling, no retries, no redirects, and no pagination. Missing keys
or invalid credential configuration send no request. Diagnostics create no runs,
media, provider voices, or project files and do not consume generation budgets.

| Provider | Read-only endpoint | Authentication |
| --- | --- | --- |
| OpenAI | `GET https://api.openai.com/v1/models` | Bearer token |
| Google | `GET https://generativelanguage.googleapis.com/v1beta/models?pageSize=1` | `x-goog-api-key` |
| Fish Audio | `GET https://api.fish.audio/wallet/self/api-credit` | Bearer token |
| OpenRouter | `GET https://openrouter.ai/api/v1/key` | Bearer token |
| ElevenLabs | `GET https://api.elevenlabs.io/v1/user` | `xi-api-key` |

These are authentication probes for the listed endpoints. A pass does not establish
that every image, music, speech, or other generation endpoint is working, or that a
specific model/voice, paid quota, balance, or cloning permission is available. A
restricted generation-only key may lack permission for the diagnostic endpoint;
that result is inconclusive, not proof the key is invalid. Explicit bounded media
smoke tests remain a separate operation.

JSON uses the normal command envelope. `data` follows `schema diagnostics`, with
`credentials` retained for existing offline doctor callers. Each row includes the
provider, credential variable name/presence, public endpoint, status, authentication
assessment, whether a network check was attempted, endpoint reachability when known,
HTTP status, elapsed milliseconds, numeric Retry-After when returned, safe message,
and corrective action. Bodies, raw transport errors, authorization headers, key
previews, account IDs, and billing information are discarded. `network_checked`
means a request was attempted, not that a successful connection was established.
`generation_performed` is always false. CLI human output uses readable provider rows.

| Status | Meaning / next step |
| --- | --- |
| `configured` | Offline presence only; use `--live` to authenticate |
| `missing_credentials` | Configure the named variable; no request sent |
| `invalid_configuration` | Correct `.env` syntax or invalid header characters; no request sent |
| `ok` | Successful expected response; authentication accepted on this endpoint |
| `authentication_rejected` | HTTP 401 without a known permission error, or recognized invalid/expired-key code; inspect the key/account restrictions |
| `access_denied` | Endpoint read permission or account/project/IP/region access denied; key validity remains unconfirmed |
| `rate_limited` | Honor returned Retry-After, then retry the read-only check |
| `service_unavailable` | Provider returned 5xx; retry after checking service status |
| `endpoint_error` | Unexpected status such as 404 or redirect; check endpoint documentation/proxy |
| `invalid_response` | Unexpected JSON/shape or excessive response; authentication unconfirmed |
| `timeout` | Deadline expired; authentication unconfirmed |
| `network_error` | DNS/TLS/proxy/connection failed or interrupted; authentication unconfirmed |

`authentication` is `not_checked`, `accepted`, `rejected`, or `inconclusive`.
Reachability stays unknown for timeouts and connection failures. Offline inspection
keeps its existing exit status 0 while reporting unavailable keys. Live checking exits
0 only if every selected provider passes, otherwise 3, retaining all provider rows
and corrective actions. Invalid command arguments exit 2. Agent callers should read
the per-provider results rather than the generic envelope error alone.

Official endpoint references: [OpenAI models](https://developers.openai.com/api/reference/resources/models/methods/list),
[Google models](https://ai.google.dev/api/models),
[Google key headers](https://ai.google.dev/gemini-api/docs/api-key),
[Fish OpenAPI](https://docs.fish.audio/api-reference/openapi.json),
[OpenRouter current key](https://openrouter.ai/docs/api/api-reference/api-keys/get-current-api-key),
and [ElevenLabs user lookup](https://elevenlabs.io/docs/api-reference/user/get).
