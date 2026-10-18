# Live browser-engine delivery

The [first observation](../audits/deployment-assets-20261018/first-observation.json)
on2026-10-18 records that the production Vercel origin returned404 for `iphreeqc.mjs` and
`iphreeqc.wasm`; the Pages origin returned200 for both. The Vercel workflow did
not install Emscripten. `tools/build-web.sh` explicitly builds only a cache-backed
frontend when `emcc` is absent, so a successful deployment did not establish a
live browser aqueous engine.

The repair uses the same Emscripten setup as the required actual-browser CI gate,
then requires both generated files before deployment. The frontend payload is
archived before API packaging. The native bundle also preserves actual producer
`Cargo.lock` bytes and checks them against its manifest hash. Neither addition
changes engine source, model data or server isolation policy.

## Acceptance and remaining delivery

Require protected CI, REST packaging/isolation tests and the applicable full own
fleet for the reviewed source. After integration, deploy the accepted combined
source through the normal existing-project workflow. Authenticate the frontend
archive, native binary/libraries, actual lock bytes and security snapshot binding.
Check live asset200s and exact byte identity, browser engine initialization, REST
health/chemistry/grammar controls, and both clients' CORS/privacy/offline behavior.
Use separately frozen requests and preserve first failures. The absent browser
assets are a deployment defect, not evidence that all unsupported chemistry is
implemented or that a new scientific catalog entry has passed.

Keep snapshot identifiers, credentials and host operation details private. Never
merge isolated browser-validation history into this product branch.
