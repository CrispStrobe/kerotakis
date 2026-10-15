# Kerotakis REST API

FastAPI and the native engine run directly in the **existing Vercel project**.
No VPS backend or separate compute hostname is required. GitHub Pages and the
Vercel app can both call this API. Each request starts a fresh lab. Execution
success is not scientific validation: responses retain `physical_verdict:
not_assigned`.

## Interface

- `GET /api/health`: API revision, configuration state and native source/binary/
  lock identity. `configured` is not proof of runtime readiness.
- `POST /api/run`: authenticated JSON containing a `script` string.
- `/api/docs` and `/api/openapi.json`: interactive OpenAPI documentation/schema.
  Authorization is not persisted across page reloads.

Example `request.json`:

```json
{"script":"add v1 water 100mL @ 25C\ninspect v1"}
```

```sh
curl --fail-with-body "$KERO_API_URL/api/run" \
  -H "Authorization: Bearer $KERO_API_KEY" \
  -H 'Content-Type: application/json' \
  --data-binary @request.json
```

The result contains JSON observations, diagnostics, exit status, elapsed time,
script SHA256 and engine identity. Execution statuses are `completed`, `refused`,
`timeout` and `output_limit`. A refused CLI script may have partial observations.
Sandbox/bootstrap failures return HTTP 503, malformed engine output 502,
authentication failure 401, excessive body size 413, invalid schema/subset 422,
and admission limit 429. Resource/store outages refuse execution. Respect
`Retry-After`; do not blindly retry after network failure because execution
may already have occurred. Durable jobs and idempotency are separate extensions.

## Same-project Vercel deployment

The normal dispatch-only Vercel workflow builds the shared static web payload,
then `tools/build-rest-native.sh` builds an x86_64 native engine in an Amazon
Linux 2023 container. It records source/tree/lock/binary/library hashes, actual
container digest and Rust version. The source mount is read-only and the build
uses two CPU jobs, four GiB container memory and a separate target directory.
This is a hosted build; do not run it locally without the approved headroom.

`tools/package-rest-api.py` adds one Python function at `api/index.py` and
immutable native/runtime assets. Only `/api/*` is rewritten to the function;
existing static routes and caching headers remain. The packaged API revision
is stamped from Git, including a dirty marker when relevant source differs.
The function is configured for 2 GiB memory and a 30-second platform deadline;
its engine has a stricter independent deadline. Actual Vercel native loading,
function routing and kernel support must pass preview acceptance.

Configure these privately in the existing project's server environment:

| Variable | Purpose |
| --- | --- |
| `KERO_API_KEYS` | JSON object mapping principal names to SHA256 digests of personal bearer keys |
| `KERO_API_REDIS_URL` | HTTPS Redis REST endpoint for shared admission |
| `KERO_API_REDIS_TOKEN` | Its private REST credential |
| `KERO_API_ORIGINS` | JSON array of exact HTTPS frontend origins, or no cross-origin access by default |

Standard `UPSTASH_REDIS_REST_*` or `KV_REST_API_*` URL/token pairs are also
recognized. The packaged function selects direct `vercel` mode; missing or
invalid authentication, native assets or shared-admission configuration disables
execution. No upstream URL or VPS address is involved.

Use personal bearer keys with at least 256 random bits. Store their SHA256
digests in configuration and distribute raw keys privately. Removing a principal
and redeploying revokes its key; requests already running are not retroactively
cancelled. Never put raw keys in source, query strings, static assets or logs.
The engine subprocess receives neither API nor Redis credentials.

Redis holds **admission counters and expiring execution leases**, not scripts or
chemistry results. Atomic Lua uses Redis server time to enforce one global
execution slot, 30 requests/principal/minute and 1,000/day across replicas. A
store error refuses execution; it never falls back to per-instance SQLite or
memory. A lost release conservatively occupies its lease until expiry. Frontend
CORS is independent of authentication. Configure Vercel deployment protection,
WAF/request limits and project spending controls before public activation, so
unauthenticated traffic cannot produce unbounded invocation costs.

## Native process restrictions

Every request uses a new CLI process with fixed arguments, private input,
empty credential environment and no shell. The native executable and bundled
libraries are verified against their build manifest. The child runs without
root privileges or Linux capabilities, with `no_new_privs` set.

Direct execution requires Landlock ABI3+ and seccomp in the **actual function
runtime**. Landlock permits reads only for the input, engine and runtime
libraries, and executable access only for the engine/ELF loader. File metadata
visibility is not equivalent to permission to read file contents. The seccomp
allowlist validates the architecture/x32 bit and denies networking, separate
process creation, cross-process memory/inspection, IPC and namespace changes.
Signals and resource-limit changes are restricted to the child itself; threads
are bounded by its per-user task ceiling. Unsupported restriction installation
returns 503; there is no unsandboxed fallback.

Limits: eight CPU seconds, ten wall seconds, one GiB child address space,
64 file descriptors/tasks, one MB combined process output and two MB serialized
response. Cancellation, timeout and output exhaustion kill/reap the process group
and drain residual pipe buffers. Vercel's function allocation governs the parent;
VPS host-memory readings are not used as serverless admission measurements.

Input limits: 20 KB JSON body, 16 KB UTF-8 script, 128 commands, 32 tokens/command,
16 explicit vessel creations and one hour aggregate explicit duration. Only the
English verbs in `app.py:VERBS` are accepted. Language/session directives,
explanations, quests, transport, extraction, titration and distillation are
excluded pending separate work budgets. The CLI still validates quantities,
identities and chemistry. No scientific support is invented by the wrapper.

## Browser clients

`web/rest.html` is included in the same static payload served by GitHub Pages
and Vercel. Enter the app's HTTPS API base ending in `/api` and a personal key.
The client saves neither key nor script, uses no cookies, refuses redirects,
and renders results as text. Both frontend origins must be allowed explicitly;
project paths are not part of a CORS origin. The existing local WebAssembly lab
continues to run independently in the browser.

## Optional Linux deployment

The same service can run on a VPS with `KERO_API_MODE=worker`, a reviewed native
`KERO_API_BINARY`, verified `KERO_API_ENGINE_MANIFEST`, local persistent
`KERO_API_STATE` SQLite file, keys and origins. Set approved memory/disk floors
using `KERO_API_MIN_MEMORY_MIB` and `KERO_API_MIN_DISK_MIB` rather than lowering
existing host policy to the portable defaults. `KERO_API_REVISION` identifies
the service commit. This optional mode uses Bubblewrap namespaces and atomic
single-host SQLite admission. `kerotakis-api.service` is an installation template.
It is not a dependency of Vercel execution. Standalone deployments serve `/run`,
`/health` and `/docs` at their origin root.

## Acceptance and current limitations

Requires Python 3.12+, pinned `requirements.txt`, a C compiler, Redis and, for
optional Linux-mode tests, Bubblewrap/user namespaces:

```sh
python -B -m unittest services.rest_api.test_api -v
node tools/test-rest-client.mjs
```

Acceptance covers auth, bounded bodies, CORS/OpenAPI, actual concurrent SQLite
and Redis Lua admission, store outage, redirects, packaged route mounting,
filesystem/network/process restrictions, timeout, output exhaustion, cancellation
and native identity. Sandbox failures must not be skipped to obtain a pass.

The direct sandbox and preserved native engine have executed locally. That
engine is bound to source `d791d306`, not a new AL2023 build. Hosted native build,
actual Vercel kernel/native-loading probes, live auth/quota controls, both real
browser origins and production delivery remain pending. No public compute
endpoint is enabled by merely pushing this code. The initial VPS-gateway design
was superseded by the user's direct-Vercel requirement.
