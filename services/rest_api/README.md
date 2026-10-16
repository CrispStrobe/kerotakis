# Kerotakis REST API

FastAPI runs in the **existing Vercel project** and executes the native CLI in
a fresh Vercel Sandbox microVM for each admitted request.
No VPS backend or separate compute hostname is required. GitHub Pages and the
Vercel app can both call this API. Each request starts a fresh lab. Execution
success is not scientific validation: responses retain `physical_verdict:
not_assigned`.

## Interface

- `GET /api/health`: API revision, configuration state and native source/binary/
  lock identity. `configured` is not proof of runtime readiness.
- `GET /api/capabilities`: supported English commands, unavailable session commands
  and enforced request, loop and simulation-duration limits. This is admission
  policy, not a claim that every chemical model is available.
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

For a deployment without rebuilding, `tools/bundle-rest-native.py` packages an
already authenticated executable with its matching ELF loader and shared
libraries. The original source/binary/lock identity is retained; it is not
relabelled as the API service commit. The Python worker uses the bundled loader
only after installing restrictions, leaving its own runtime unchanged.

`tools/package-rest-api.py` adds one Python function at `api/index.py` and
immutable native/runtime assets. Only `/api/*` is rewritten to the function;
existing static routes and caching headers remain. The packaged API revision
is stamped from Git, including a dirty marker when relevant source differs.
The function has a 60-second platform deadline. The isolated VM has one vCPU,
two GiB memory, a 30-second lifetime, no exposed ports and denied outbound
networking. The complete VM lifecycle has a 35-second client deadline; its native
child has stricter independent limits. Vercel Fluid Compute controls the
function allocation and may ignore the configured memory field.

Configure these privately in the existing project's server environment:

| Variable | Purpose |
| --- | --- |
| `KERO_API_KEYS` | JSON object mapping principal names to SHA256 digests of personal bearer keys |
| `KERO_API_REDIS_URL` | HTTPS Redis REST endpoint for shared admission |
| `KERO_API_REDIS_TOKEN` | Its private REST credential |
| `KERO_API_EXECUTOR` | `microvm` for the actual Vercel runtime |
| `KERO_API_SNAPSHOT` | Exact immutable native/security snapshot ID |
| `KERO_API_ORIGINS` | JSON array of exact HTTPS frontend origins, or no cross-origin access by default |

The deployment workflow also accepts `KERO_API_KEYS` from an encrypted GitHub
Actions secret and passes only those key digests to the function environment.
Raw client keys stay outside the repository and deployment artifacts.
The repository variable `KERO_API_ORIGINS` supplies the two approved frontend
origins to the same deployment; add verified aliases there when needed.

Standard `UPSTASH_REDIS_REST_*` or `KV_REST_API_*` URL/token pairs are also
recognized. The packaged function selects `vercel` mode; missing or
invalid authentication, native assets or shared-admission configuration disables
execution. No upstream URL or VPS address is involved.

`tools/prepare-rest-snapshot.py NATIVE_BUNDLE NEW_RECEIPT` prepares a secret-free
image and executes a native smoke check before snapshotting. The deployment
workflow follows it with `tools/test-rest-microvm.py RECEIPT NEW_EVIDENCE`.
Pass the receipt as the third argument to `tools/package-rest-api.py`; packaging
binds its exact snapshot ID, full engine manifest, supervisor and seccomp hashes.
Changed policies require a new snapshot. Snapshots expire after 30 days without
use. SDK access uses project OIDC in Vercel; the account access token is never
uploaded into a function, VM or snapshot. A preview supplies its own snapshot ID
as a deployment override, preserving the production binding.

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

The deployed Vercel kernel returns `ENOSYS` for Landlock. The production
executor therefore uses a separate microVM rather than running the child in the
credential-bearing function. Its fixed supervisor enters a root containing only
verified engine libraries and the input, drops supplementary groups and uid/gid
to 65534, clears capabilities and installs seccomp. Read-only open filters,
mutation syscall denial and restricted `fcntl` prevent writes and asynchronous
signals. The VM is destroyed after the request; its server-enforced lifetime
bounds cleanup failures. Neither application keys nor shared-store credentials
enter it. The actual deployed native smoke and adversarial fixture passed.

The optional direct executor still requires Landlock ABI3+ and seccomp. Landlock permits reads only for the input, engine and runtime
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

Input limits: 20 KB JSON body, 16 KB UTF-8 script, 128 commands, 32 tokens/command
and 16 explicit vessel creations. All canonical English bench verbs and their
English synonyms/spellings are admitted; a test checks the engine inventory.
This includes finite `stock`, `remove`/`discard`, `smell`/`waft`,
`particles`/`zoom` and `cell`/`voltmeter`. The simulation verbs in `app.py:VERBS`
include `look`/`observe`, vessel explanations, evaporation, pressure regulation,
mixture transfer, extraction, distillation, titration and reactive transport.
`register lv1|lv2|lv3` changes presentation only. Filesystem commands, language
changes, quests and other session controls remain excluded. `help`, `species`,
`structure`, `identify` and `coverage` have no JSON batch observation handler;
`find` and `materials` are interactive shell facilities. These requests receive
an explicit batch-JSON refusal rather than silently doing nothing. Localization
is not enabled: admitting a language directive without canonicalizing its later
commands would let loop accounting disagree with execution.

Multiword ingredient spellings use the same reviewed identity aliases as the
engine before parameter-position accounting; submitted scripts remain unchanged.
Tests keep this small policy view aligned with the engine alias inventory.

Loop admission is explicit: titration defaults to the CLI's100 steps and accepts
at most2,000; extraction/distillation accept at most16 stages; transport accepts
at most256 steps and16 cells, charging steps × cells. The aggregate loop-work
budget is4,096 per request. Duplicate keywords, extra suffixes, noninteger/zero
counts and nonfinite Courant fractions refuse before execution.

Explicit simulated duration is bounded at30 days per command and90 days per
request. These are simulation horizons, independent of the unchanged eight CPU
and ten wall seconds allowed to the child. Long or costly chemistry may still
reach the execution deadline. The CLI validates quantities, identities and
chemistry; gateway admission does not claim chemical support or success.

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

Ubuntu 24.04 restricts unprivileged user namespaces through AppArmor. Its
`apparmor-profiles` package ships a purpose-built Bubblewrap profile at
`/usr/share/apparmor/extra-profiles/bwrap-userns-restrict`. An administrator can
install and load that vendor profile for this worker. It permits Bubblewrap's
namespace setup and strips capabilities from its children. The REST acceptance
workflow loads that scoped profile and runs a fixed startup probe before all
positive isolation controls. Keep system-wide user-namespace restrictions and
`--unshare-all` enabled; disabling AppArmor or sharing the host network is not
an acceptable repair. See Ubuntu's
[namespace restriction guidance](https://discourse.ubuntu.com/t/understanding-apparmor-user-namespace-restriction/58007).

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

Twenty-five focused tests pass, including malformed/rollback snapshot refusal
and isolated-job cleanup after identity mismatch and timeout. Actual Vercel
microVM controls verify filesystem isolation, immutable input, missing secrets,
network/fork/parent-signal denial, restricted file-control operations and blocked
privilege escalation. The authenticated FastAPI preview completed a native run.
The retained engine is bound to source `d791d306`; it is not a new AL2023 build
or a deployment of every current frontend change. The original VPS gateway and
in-function Landlock plans are superseded for this deployment.
