# Kerotakis REST API — production state and remaining lanes

The API is live in the existing project:
[interactive documentation](https://kerotakis-web-payload.vercel.app/api/docs),
[health/version](https://kerotakis-web-payload.vercel.app/api/health) and
[browser client](https://kerotakis-web-payload.vercel.app/rest.html).
`POST /api/run` accepts an authenticated JSON `script`; each admitted request
starts a fresh lab. Both approved frontend CORS preflights passed. The API runs
on Vercel and requires no VPS service or additional hostname.

Production deployment `dpl_BMfdaZVCPoDSynF3Weu7A9F7Su18` serves API revision
`9bee95d9` with native engine `d791d306`. The engine's binary and lock identities
are returned by health and execution responses. This is a preserved verified
engine, not a claim that current main or all recent frontend fixes were rebuilt.
Three public static asset hashes match the prior production files.

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
[Deployment receipt](../audits/fresh-syntax50-20261013/rest-api-production-20261015.json).

The requested full API comparison is now complete: **200 HTTP200 responses,
92 completed scripts, 108 CLI refusals, no errors/retries**. Execution statuses
and full parsed observations agree with the prior CLI run for all 200 unchanged
scripts. Median request time was 3.27 seconds. See the
[verified comparison](../audits/reaction-dataset-200-20261015/api-comparison/completion.json).
Physical validation remains separate; do not duplicate this finished API batch.

## Doable next lanes

1. **Finish source integration and rebuild acceptance.** Continue
   [PR795](https://github.com/CrispStrobe/kerotakis/pull/795) at its actual current
   head. Preserve the first hosted build failure: Amazon Linux installed
   IPhreeqc into `lib64`, while Cargo searched `lib`; container cleanup also
   lacked ownership. Both source fixes are pushed. Follow
   [the new hosted preview](https://github.com/CrispStrobe/kerotakis/actions/runs/38783017261)
   through native build, snapshot preparation, isolation controls and deployment.
   Authenticate its new engine/source/lock binding and repeat the production
   controls before replacing the retained engine. Complete applicable CI and
   normal protected auto-merge; never merge the broad audit branch.
2. **Complete browser delivery.** Use `web/rest.html` and the existing personal
   bearer key to exercise execution from both actual browser origins. Confirm
   that keys/scripts are cleared and never persisted, output renders as text,
   and service workers bypass API/client routes. HTTP CORS controls and the Node
   client tests are already complete; real-browser acceptance is separate.
   Publish the client through the normal Pages payload if it is not there yet.
3. **Maintain the deployed security binding.** Changed supervisor/seccomp code
   requires a new snapshot using `tools/prepare-rest-snapshot.py`, followed by
   `tools/test-rest-microvm.py`. Package its receipt with the matching native
   bundle. Verify auth, shared saturation and bounded cleanup after deployment.
   Snapshots expire after thirty days without use. Keep credentials private and
   preserve the configured quotas; do not add an unsandboxed fallback.
4. **Assess existing chemistry evidence.** Reuse the completed 200-record CLI
   batch before executing more. Its 92 completed and 108 nonzero runs are process
   outcomes. The source supplies no procedures; generated contact probes are
   not 200 individually validated protocols. Freeze valid condition-specific
   follow-ups and compare native/API/WebAssembly observations with independent
   physical expectations, distinguishing rendering from chemistry defects.
5. **Add durable jobs only when needed.** If measured synchronous limits block
   real protocols, add bounded submission/status, principal ownership,
   cancellation, idempotency and retained provenance. The conventional REST
   interface is already deployed; this extension is not required to activate it.

The implementation/deployment contract is `services/rest_api/README.md` on PR795.
[Vercel Sandbox](https://vercel.com/docs/sandbox) documents the isolated compute
boundary; actual platform controls are preserved in the deployment receipt.
