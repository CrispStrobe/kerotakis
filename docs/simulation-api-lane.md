# Simulation API implementation and remaining lanes — 2026-10-15

[PR795](https://github.com/CrispStrobe/kerotakis/pull/795) implements the initial
authenticated service and same-project Vercel gateway. Eighteen local acceptance
tests and an actual native HTTP smoke passed; CI and preview are queued.
No public execution service is activated. See the
[preparation receipt](../audits/fresh-syntax50-20261013/rest-api-preparation-20261015.json).

The interface is a conventional FastAPI REST service, following
[`stc-compiler`](https://github.com/CrispStrobe/stc-compiler/tree/a043daa58c2437f936f94971b3b0695c9c2510cb):
its `app.py` serves FastAPI on Vercel, and `docs/index.html` on GitHub Pages
calls that service. GitHub Pages hosts the client and documentation, not the
Python execution service. Both Kerotakis web hosts can use one REST backend.

Start with authenticated `POST /run` accepting `{ "script": "..." }` and
returning bounded JSON observations, diagnostics, exit status and engine identity.
Provide `GET /health` with deployed revision/engine identity, and generated
OpenAPI `/docs`. Run one isolated CLI process per request with strict admission
and execution limits. A synchronous single-script endpoint is sufficient to
start; the asynchronous job routes below are an extension for larger batches,
not a prerequisite. API clients should send credentials at runtime; shared
credentials must not be embedded in either static site's assets. Restrict
browser origins explicitly while enforcing authorization independently of CORS.
Do not copy the reference service's permissive CORS or per-instance rate guard
as a substitute for authenticated, fleet-wide admission control.

The current web deployment is a static payload. `web/app/src/lib/host/WorkerHost.ts`
starts a browser worker; `engineWorker.ts` loads the bench WebAssembly and
IPhreeQC there. No server simulation route exists in the inspected source.
Using the deployed UI runs chemistry on the browser host. The latest successful
Vercel workflow's deployment URL redirected to Vercel authentication when checked;
its deployed engine version and runtime behaviour were not verified.

An API would provide remote compute and repeatable automation. It is separate
from browser acceptance: a native API result does not validate browser rendering
or establish native/WebAssembly agreement.

## Implementable lanes

1. **Bounded execution worker.** Reuse the CLI in one fresh process per script,
   with no shell interpolation. Enforce wall time, memory, output bytes, script
   bytes, command count and simulated-work limits before execution where possible.
   Disable user-supplied file paths, imports and network access. Run with an
   unprivileged identity, read-only engine assets and disposable job storage.
   Preserve source/build/lock/script hashes, exit status, diagnostics and timings.
   Acceptance: timeout/output exhaustion terminate and reap the worker; two jobs
   cannot share bench state or access each other's files. A timeout alone must
   not leave a solver running.
2. **Authenticated job service.** Proposed routes: `POST /api/v1/runs` accepts one
   bounded script and returns `202` plus an opaque job ID; authenticated
   `GET /api/v1/runs/{id}` returns only that principal's status/result. Validate
   revocable, scoped credentials server-side before accepting work; never ship a
   shared secret in browser assets. Use bounded durable queues, per-principal
   quotas, global concurrency and cost limits, and shared rate-limit counters.
   Reject requests when authentication, quota storage or queue admission fails.
   Acceptance: missing/invalid/revoked credentials, oversized/chunked bodies,
   cross-principal reads, concurrent quota races and queue saturation all fail
   safely. Do not use permissive CORS or process-local counters as authorization
   or fleet-wide admission control. Keep credentials/scripts out of access logs.
3. **Vercel integration.** Package an actual server function alongside the static
   payload, or use an authenticated gateway to a separate execution service.
   First prove that the selected runtime can load the exact solver assets and
   enforce worker termination. Native and WebAssembly are distinct build targets;
   choose and report the one actually used. Keep the compute origin inaccessible
   except to the gateway, with authenticated transport. Verify deployment
   protection and application authorization separately. Require configured
   credentials and admission controls before enabling execution; absent settings
   leave the route disabled. Test in a protected preview before production.
4. **Cross-runtime comparison.** Freeze the same valid scripts and explicit
   expectations for native CLI and deployed WebAssembly, record both build
   identities and compare chemistry observations separately from presentation.
   The 200 generated probes have missing procedures and known condition issues;
   execution coverage must not become a claim of 200 validated reactions.

For the immediate dataset, sequential native CLI needs no public service or new
build. A first partial VPS run completed thirteen attempts in 3.61 seconds before
its memory gate closed; this suggests practical speed but is not a timing bound
for the remaining complex cases. Preserve that prefix when resuming.

Platform references: [Vercel Functions](https://vercel.com/docs/functions),
[WAF rate limiting](https://vercel.com/docs/vercel-firewall/vercel-waf/rate-limiting).
The initial API is implemented in PR795; worker HTTPS routing, private
configuration, hosted acceptance and deployment remain pending. The asynchronous
job and cross-runtime lanes are extensions beyond this initial service.
