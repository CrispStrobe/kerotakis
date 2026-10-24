# Current phase product acceptance — prospective lane

**Ownership addendum —2026-10-24:** The current source candidate is `a0177cff`; source/product acceptance is still pending. Add [the separately frozen fresh-feed/wait ownership control](../audits/aqueous-overlap-product-control-20261024/FORECAST.md) to both lanes, preserving the original three cold scripts and baseline11API/82browser checks. Combined candidate totals are now **15API/86browser**, with fourth-control warm predicates distinct from the three phase predicates below. The original three-script preparation and older candidate binding below remain historical; bind actual accepted/deployed source before requests. No product execution or pass is claimed by this addendum.

Prepared 2026-10-23 from bounded source and saved-receipt inspection. This is a scope proposal only. No release/API/browser run, deployment, build, provider call, or replay was performed for this addendum.

## Gate and binding

The current candidate is source `c8171c4ebcf8961840605f2e925b6b8265b80c8a`; it is not yet the accepted/deployed product source. Run this addendum only after source/hosted/platform/protected acceptance, normal integration and candidate artifact/security checks permit deployment. Execute the controls on that newly deployed candidate before declaring product acceptance. Bind every result to the exact new source commit/tree, native executable and build metadata, generated `Cargo.lock`, Wasm module, shipped engine assets/database, and actual loaded browser worker/asset identities. Record the actual API deployment/release binding and browser worker identity with each lane. Do not reuse the prior release's identities as if they described the candidate.

Prior accepted baseline is PR857 source `beffb001b1fa2b73128ab0bce7d8776dab9edeb4`, native binary SHA-256 `2c91d7923d80e7927778e22e67a07dd23d9bb5e322862375d477e7eabe6672d4`, and lock SHA-256 `41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb`. That release passed the preserved 11 API and 82 browser controls. It is historical binding context only; it is not evidence for the new candidate.

## Three added behavior controls

Replay each frozen script byte-for-byte, once through the existing REST script route and once through the actual loaded browser engine worker. Keep all original 11 API and 82 browser controls intact. These are three additional distinct scripts in each lane (14 API control executions and 85 browser control executions if lanes are totaled); they are not original-corpus members or calibration experiments.

| Control | Public frozen script | SHA-256 |
|---|---|---|
| CaCl2/AgNO3 cooling | `audits/phase-particle-precision-20261023/corrected-cli-protocol.kero` | `a95e59408c36731599dcdf2a95457be77ddbfd1c6282a720fc80e0c2a5ec1ec8` |
| Carbonate/acetic weak acid | `audits/phase-microtransfer-closure-20261023/carbonate-acetic.kero` | `7a4356e73368ad63c7a28baf927ca3b6ae8a2794b3d93f1d916f4fee9e91d16a` |
| Bicarbonate/phosphoric weak acid | `audits/phase-microtransfer-closure-20261023/bicarbonate-phosphoric.kero` | `c98d40cbca3f222a2b2bab7746b539ecde810061cf17f686c136e4825d4dc0c6` |

The two weak-acid controls are frozen diagnostics, not proven product passes. Preserve request/script bytes and complete raw response bytes; report each result independently. A run is not accepted on HTTP success or a rendered sentence alone.

## Required result predicates

For each script and both lanes, evaluate the returned current vessel state and event history:

- Completed execution, with no `SolverFailed`; a present current numerical solution and finite, nonnegative `solution.particle_molality`; and model support/domain metadata admitting the current state. Unsupported, missing, stale, estimated-only, or out-of-domain data is a failure/coverage limit, never a pass.
- Both liquid water and ice are present. The vessel's actual final temperature agrees with its own current liquidus/melting point and the last actual WATER phase-event boundary within `1e-6 K`.
- Close all owned material, including H/O and charge, at `1e-10` using the frozen initial/current ledgers plus actual gas vent/absorption and phase-transfer events. Do not assume water inventory is invariant in acid/carbonate reactions. If the product payload cannot close a reservoir, report that exact observability gap and leave the control unaccepted.
- The actual cooling event records the delivered `20,000 J` within `0.001 J`. This is phase closure evidence only; it does not establish full reaction-heat calibration or general chemistry validity.

Use the same predicate implementation against the API JSON and the browser worker's actual response, and retain enough raw fields/events to independently reproduce the result. Do not map absent data to zero or infer current numerical census from rounded rendered text.

## Existing payload review and concrete harness condition

The Rust/Wasm `run_script` serializer returns `steps`, final `scene`, and raw `bench.vessels`; each step carries `events`. The REST saved response example at `audits/target468-20261016/phase-owned-release-acceptance-20261020/api-controls/finite-phase-wait-cycle/response.body` demonstrates the corresponding `observations[].bench.vessels`, `events`, and `scene` envelope. This makes current solution and event checks possible when the newly released serializer actually includes the candidate fields.

The browser worker protocol returns the engine JSON in its `done.result_json` field. `EngineHost.ts`'s `ScriptResult` consumer type does not declare `bench`, so the browser acceptance collector must preserve/parse the raw worker result JSON (or otherwise show the same fields are retained) rather than relying on the narrowed TypeScript type. This is a collection/typing condition, not a proposed new API or field. Before accepting, verify the actual newly loaded worker result contains current `bench.vessels[].solution.particle_molality`, current support/domain evidence, the needed owned material reservoirs, and phase/energy events. If any are absent in the actual release payload, record the missing field as a product evidence gap and do not pass by supplementing it from native/CLI output.

Existing public harness references: `audits/target468-20261016/phase-cap-browser82-accepted-20261022/README.md`, `audits/target468-20261016/phase-cap-browser82-accepted-20261022/source-and-execution-summary.json`, `audits/target468-20261016/phase-cap-release-api-and-browser-issued-20261022/script-assessment.json`, and `audits/live-rest-browser-20261018/release-binding.json`. Retain the old baseline's 11/82 evidence and add a separate candidate-bound receipt for these three scripts; do not rewrite frozen baseline reports.

## Source/document bindings inspected

- Candidate commit: `c8171c4ebcf8961840605f2e925b6b8265b80c8a` (prospective only).
- Frozen contract SHA-256: `audits/phase-microtransfer-closure-20261023/FORECAST.md` — `5b097da7485250ed3a7f4a5d51efee04fe9102bb63e9d214dc1d959d2a77f8e7`.
- `web/app/src/lib/host/EngineHost.ts` and `web/app/src/lib/host/engineWorker.ts` were reviewed for consumer and worker response shape. Source types and worker protocol describe the payload path; the candidate's future built assets still need actual release binding and execution evidence.
- Existing accepted browser report SHA-256 `e9efacfd266677134607472af582041e1ef567ffab17f867332db00242dc2eaa`; release script assessment SHA-256 `ef51982bc167d96445d0a75e23272f5fadf3901ed18720adb24d7f6a202e2c1b`.

No pass/fail result is claimed here. The phase-coupling forecast remains prospective until the candidate's complete controls and new product receipt are independently reviewed.
