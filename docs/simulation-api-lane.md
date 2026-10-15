# Direct Vercel REST API — current implementation and remaining lanes

[PR795](https://github.com/CrispStrobe/kerotakis/pull/795) runs FastAPI and the
native engine directly in the existing Vercel project. It does **not** require
a VPS backend, gateway-to-VPS connection or additional worker hostname. This
follows the service/client split in `stc-compiler`: Pages hosts the client,
Vercel hosts REST compute. Both Kerotakis web hosts receive `web/rest.html`.

The routes are `/api/run`, `/api/health`, `/api/docs` and `/api/openapi.json`.
Authentication, bounded English scripts, independent process limits, Landlock/
seccomp restrictions and atomic shared Redis admission are implemented. Raw
client keys are excluded from Git and deployment assets; only their digests
are passed privately. Redis stores admission counters/leases, not chemistry
scripts or results. An optional standalone Linux deployment remains available.

Twenty-two local tests passed. A preserved native engine ran in the direct
sandbox, and client/service-worker controls passed at root and project scope.
This does not establish actual Vercel kernel compatibility or an AL2023 build.
See the [exact-source preparation receipt](../audits/fresh-syntax50-20261013/rest-api-direct-vercel-20261015.json).
The earlier VPS-gateway receipt remains historical and is superseded.

## Doable next lanes

1. **Native build and actual preview.** Complete the already-dispatched current
   source Vercel preview. Authenticate the AL2023 native artifact, source/lock/
   binary/library binding and container/compiler identity. Verify `/api/health`
   reports the packaged revision, the native loader works, and the actual
   platform installs Landlock/seccomp. Refuse unsupported restrictions rather
   than silently dropping isolation. Verify `/api/docs` schema base and existing
   static app routes; local packaging tests do not prove Vercel routing.
2. **Activate and verify admission.** The deployment key-digest secret and two
   frontend origins are configured. Verify Redis REST credentials in the project
   environment, using the documented standard integration names or dedicated
   variables. Test valid/invalid keys, concurrent saturation, rate quotas, store
   outage and cross-origin controls on the actual preview. Keep invocation/WAF
   and spending limits configured independently of in-function authorization.
   Missing configuration remains disabled. No VPS network setup is needed.
3. **Integrate and deliver.** After the named non-protected acceptance and normal
   repository CI pass, mark PR795 ready and enable protected auto-merge at its
   exact reviewed head. Then deploy production deliberately to the existing
   project, record actual build/deployment identity and repeat bounded health,
   auth and representative native controls. Do not infer deployment from a merge.
4. **Dataset and runtime comparison.** Assess the already completed 200 supplied
   contact probes separately from process exits. Freeze valid condition-specific
   native/API/WebAssembly comparisons and distinguish chemistry from rendering.
   The source supplies no procedures; generated screens are not 200 validated
   laboratory protocols. Reuse saved results before requesting new executions.
5. **Optional asynchronous jobs.** Only if synchronous request limits prove
   restrictive, add bounded durable submission/status routes with principal
   ownership, cancellation, idempotency and retained provenance. This extension
   is not required for the initial conventional REST interface.

The detailed service/deployment contract is in `services/rest_api/README.md` on
PR795. [Vercel Python runtime](https://vercel.com/docs/functions/runtimes/python)
and [function limits](https://vercel.com/docs/functions/limitations) describe
platform packaging; actual preview tests remain authoritative for this build.
