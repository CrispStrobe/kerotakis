# Kerotakis REST API — production state and remaining lanes

The API is live in the existing project:
[interactive documentation](https://kerotakis-web-payload.vercel.app/api/docs),
[health/version](https://kerotakis-web-payload.vercel.app/api/health) and
[browser client](https://kerotakis-web-payload.vercel.app/rest.html).
`POST /api/run` accepts an authenticated JSON `script`; each admitted request
starts a fresh lab. Both approved frontend CORS preflights passed. The API runs
on Vercel and requires no VPS service or additional hostname.

Production now serves integrated API/engine revision `c4469205`, native binary
`670bd3d8`, from [run38868092504](https://github.com/CrispStrobe/kerotakis/actions/runs/38868092504).
The [authenticated production receipt](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/reaction-dataset-200-20261015/production-c446-20261016/README.md)
verifies artifact/source/tree/native/libraries, fresh isolated snapshot and live
health. Raw build-lock bytes remain producer-qualified for this artifact. The
previous production `9bee95d9`/`d791d306` evidence below is retained historical
acceptance, rather than relabelled as new-engine execution.

Actual Vercel execution revealed unavailable Landlock support. The native child
therefore runs in a fresh Vercel Sandbox microVM with denied outbound networking,
a private filesystem root, dropped privileges/capabilities and restricted
syscalls. An immutable snapshot binds the engine, libraries and security code.
No API or shared-store credentials enter that VM. A dedicated free-plan Redis
store supplies atomic admission; its automatic paid upgrade is disabled.

Twenty-five focused tests, actual VM isolation controls and production HTTP
checks passed. Invalid/missing credentials return 401; oversized bodies 413;
unsupported API commands 422; an unapproved CORS preflight 400. Two simultaneous
production requests returned one completed execution and one 429. A native HCl
screen completed with three observations. The first unsupported `mmol` input was
preserved as a CLI refusal, then a separate `0.001mol` follow-up executed. Neither
HTTP200 nor process completion assigns a scientific pass.
[Deployment receipt](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/fresh-syntax50-20261013/rest-api-production-20261015.json).

The requested full API comparison is now complete: **200 HTTP200 responses,
92 completed scripts, 108 CLI refusals, no errors/retries**. Execution statuses
and full parsed observations agree with the prior CLI run for all 200 unchanged
scripts. Median request time was 3.27 seconds. See the
[verified comparison](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/reaction-dataset-200-20261015/api-comparison/completion.json).
Physical validation remains separate; do not duplicate this finished API batch.

## Current integration and delivery checkpoint — 2026-10-16

[PR795](https://github.com/CrispStrobe/kerotakis/pull/795) is integrated as
`c4469205`, with all59 successful checks and one skip. The
[integration receipt](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/reaction-dataset-200-20261015/pr795-integration.json)
records reviewed source, actual merge parents/tree and applicable acceptance.
The old source-build, native sandbox and UTF-8/lint failures remain preserved;
they are no longer pending integration work.

The source-pinned [production run38868092504](https://github.com/CrispStrobe/kerotakis/actions/runs/38868092504)
completed successfully. The independently rereviewed production receipt accepts
nine once-only requests: two finite chemistry controls, auth/subset/CORS checks,
fresh health and exact client bytes. The same-snapshot isolation probe passed.
No independent VM-deletion receipt or raw producer-lock rehash is claimed.

[Fifteen actual-browser controls](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/reaction-dataset-200-20261015/rest-browser-acceptance-20261016/README.md)
already accept the unchanged client UI/privacy and actual browser CORS behavior.
Its controlled Pages-origin check does not establish publication. Actual
`/kerotakis/rest.html` still returned404 at this checkpoint; the integrated-main
Pages pipeline must publish it before testing the served client.

## Doable next lanes

1. **Maintain verified production delivery.** The current c446 delivery is
   accepted for the recorded controls. Rebuild and repeat narrowly scoped
   acceptance when an accepted source change warrants a new engine. Preserve
   source/tree/native/lock/snapshot bindings and first failures; do not repeat
   unchanged200-case transport checks.
2. **Verify Pages publication.** Follow the existing integrated-main demo/publish
   pipeline, then confirm the actual served client bytes. Reuse accepted
   unchanged-client browser evidence where applicable; complete any outstanding
   actual-origin acceptance after publication. Do not repeat the entire200 batch.
3. **Repair protocols and underlying chemistry independently.** Use the
   [lane contract](reaction-repair-lane-contract.md) and
   [model intake](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/reaction-dataset-200-20261015/model-issues-20261016/index.json).
   The protocol lane owns procedures, syntax and proven aliases; Hermes owns
   model repairs and PR805 acceptance. All process outcomes remain separate
   from scientific verdicts. Freeze source-bound follow-ups before execution.
   The [201 new designs](https://github.com/CrispStrobe/kerotakis/blob/d847a0b112ffd0e85dd655ce98bf5448c9538008/audits/new-experiments-201-20261016/README.md) have
   separate originals, four proposed procedure corrections and acceptance lanes.
4. **Maintain the security binding.** Changed supervisor/seccomp code requires
   a rebuilt snapshot, actual isolation tests and the matching native bundle.
   Preserve authentication, shared rate limits, quotas, bounded cleanup and
   denial of untrusted network access. No unsandboxed fallback is permitted.
5. **Add durable jobs only when measured limits require them.** A bounded
   submission/status extension needs principal ownership, cancellation,
   idempotency and retained provenance. It is not an activation prerequisite.

The implementation/deployment contract is `services/rest_api/README.md` on
integrated main. Actual platform receipts, rather than configuration alone,
establish the accepted isolation boundary.
