# Kerotakis REST API

A FastAPI interface to the native CLI, following the
[`stc-compiler` service pattern](https://github.com/CrispStrobe/stc-compiler).
Every request gets a fresh lab and a bounded process. Exit success does not
establish chemical validity. The response reports `physical_verdict: not_assigned`.

## Endpoints

- `GET /health`: service configuration, API revision, native engine identity
  when running in worker mode. `configured` is not a live readiness probe.
  Gateway health describes the gateway; `/run` returns the actual worker engine.
- `POST /run`: authenticated JSON `{ "script": "add v1 water 100mL @ 25C\ninspect v1" }`.
  Returns JSON observations, diagnostics, exit status, elapsed time, script hash
  and engine source/binary/lock hashes.
- `/docs` and `/openapi.json`: interactive documentation and machine schema.
  Swagger authorization is not persisted across reloads.

```sh
curl --fail-with-body "$KERO_API_URL/run" \
  -H "Authorization: Bearer $KERO_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary @request.json
```

`execution_status` is `completed`, `refused`, `timeout` or `output_limit`.
Refusals may have partial observations: the CLI stops at its first refused
command. Sandbox/bootstrap failures use HTTP 503, malformed engine output 502,
authentication failure 401, body limit 413, schema/subset refusal 422, admission
limit 429, and resource/store outage 503. Respect `Retry-After` and do not blindly
retry requests after network failure: execution may already have happened.
This initial synchronous service does not provide idempotency or durable jobs.

## Linux execution worker

Requires Python 3.12+, an unprivileged Linux service account, Bubblewrap with
unprivileged user/network/PID namespaces, and a reviewed native `kero` executable.
Install the pinned dependencies from `services/rest_api/requirements.txt` into
an isolated virtual environment. The API service itself must not run as root.

Set these server-side environment variables:

| Variable | Value |
| --- | --- |
| `KERO_API_MODE` | `worker` |
| `KERO_API_BINARY` | Absolute path to the immutable reviewed native executable |
| `KERO_API_ENGINE_MANIFEST` | Absolute path to its verified JSON build manifest |
| `KERO_API_STATE` | Absolute path to SQLite admission state on local persistent disk |
| `KERO_API_KEYS` | JSON object mapping principal names to SHA256 digests of bearer keys |
| `KERO_API_ORIGINS` | JSON array of exact HTTPS browser origins; default none |
| `KERO_API_REVISION` | Full API service Git commit |
| `KERO_API_MIN_MEMORY_MIB` | Deployment's approved available-memory floor, at least 1024 MiB |
| `KERO_API_MIN_DISK_MIB` | Deployment's approved free-space floor for scratch and state, at least 1024 MiB |

The manifest must contain `source_head` (40 lowercase hex characters),
`binary_sha256` and `cargo_lock_sha256` (64 each). Obtain it from an authenticated
build receipt; declaring a source hash alone does not authenticate a build.
The service verifies executable bytes at configuration and before every run.
Keep the executable, manifest, Python code and runtime libraries non-writable
by the service account. Update them atomically when the service is stopped.

Generate separate keys with at least 256 random bits, distribute them privately,
and store only their SHA256 digests in configuration. Removing a principal and
restarting the worker and gateways revokes that key; already running requests
are not retroactively cancelled. Use one unique digest per principal. Never
commit bearer keys, put them in query strings or bake them into static assets.

The service defaults to disabled on missing/invalid configuration. The portable
memory default is not permission to lower an existing host's operating floor:
set the approved floor explicitly before starting this on an existing host.
One worker host is authoritative. SQLite atomically coordinates all its API
processes: one execution slot, 30 requests/principal/minute and 1,000/day.
Leases expire after 90 seconds, beyond the 10-second execution deadline. Keep
the state directory private and local, never on a shared/network filesystem or
ephemeral Vercel instance. A state-store outage refuses execution. Use a shared
transactional admission service before adding multiple execution hosts.

Start from the repository root:

```sh
uvicorn services.rest_api.app:app --host 127.0.0.1 --port 8081 --workers 1 --no-access-log
```

`kerotakis-api.service` is an installation template with generic deployment
paths. Install an HTTPS reverse proxy, disable body/credential logging, impose
request-size, connection and rate limits there, and restrict service ingress.
Do not expose the loopback HTTP listener directly. Public Vercel gateway access
also needs deployment protection/WAF limits so unauthenticated traffic cannot
consume unlimited gateway invocations. Application authentication remains
required even when deployment protection is enabled.

## Sandbox and admitted syntax

Each CLI child receives only a fixed `run /job/input.lab --json` invocation;
no shell is involved. Bubblewrap hides the host filesystem except the executable
and runtime libraries, disables external networking, provides a private PID
namespace, clears credentials/environment and mounts the input read-only.
The service kills/reaps the process group on cancellation, timeout and output
exhaustion. CPU is limited to 8 seconds, address space to 1 GiB, wall time to
10 seconds, combined output to 1 MB, and sandbox scratch space to 16 MiB.
Deploy with the supplied service's aggregate memory/task limits too.

Requests are limited to 20 KB JSON bodies, 16 KB UTF-8 scripts, 128 commands,
32 tokens per command, 16 explicit vessel creations, and one hour aggregate
explicit duration. Only the English verbs in `app.py:VERBS` are admitted.
Language/session directives, explanations, quests, transport, extraction,
titration and distillation are excluded pending separate work budgets.
Canonical Kero parsing still validates each allowed command; invalid quantities
or unknown substances are not automatically corrected by the service.

## Vercel and GitHub Pages

Use the **existing Vercel app project**. The deployment workflow calls
`tools/package-rest-api.py` after building its static payload. This adds one
Python function and rewrites only `/api/*` to it, preserving the app and caching
headers. Routes are `/api/run`, `/api/health`, `/api/docs` and `/api/openapi.json`.
The mounted gateway cannot enable worker mode. Configure the existing project:

- `KERO_API_MODE=gateway`
- `KERO_API_UPSTREAM` as the fixed HTTPS origin of the execution worker
- the same `KERO_API_KEYS` mapping, plus the required `KERO_API_ORIGINS`

The gateway authenticates locally, then forwards the caller's bearer key to
that fixed origin. The worker independently authenticates and enforces the
authoritative quotas. It accepts no caller-selected upstream, follows no
redirects, uses no proxy settings from the environment, and bounds both request
and response bytes. Do not point the upstream back at this gateway.

`web/rest.html` is a standalone client included by `tools/build-web.sh` in the
same payload both GitHub Pages and Vercel serve, at `rest.html`. Enter the API
HTTPS origin and a personal key at runtime. It saves neither key nor script,
uses no cookies, refuses redirects and renders responses as text. Existing
local WebAssembly lab execution remains unchanged. Allow both actual frontend
origins in API CORS configuration; project paths are not part of an origin.
For the same-project deployment, enter the app origin followed by `/api`.
The page itself supplies no production URL or credential automatically.
A separate standalone service deployment remains available through
`services/rest_api/vercel.json`, with routes at the origin root.

## Acceptance and deployment

```sh
python -B -m unittest services.rest_api.test_api -v
```

Tests use a small native fixture in the real sandbox. They exercise credential
refusal, body/schema/subset limits, CORS/OpenAPI, concurrent quota admission,
store outage, release after failure, fixed-origin gateway redirects, filesystem/
network isolation, output exhaustion, timeout, cancellation and engine identity.
Bubblewrap, a C compiler and namespace support are required; sandbox failures
must not be skipped to obtain a passing suite.

Before production, record the API source, native source/binary/lock binding and
the actual HTTPS `/run` smoke result. Check both frontend origins and invalid
credentials, validate proxy limits, and confirm key revocation on every replica.
The implementation's local native smoke used preserved engine source
`d791d306`; it does not claim that a new native build or remote deployment exists.
Deployment requires an operator-provisioned worker HTTPS origin and privately
configured credentials. No public production API is enabled by this commit.
