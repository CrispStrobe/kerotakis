# Browser solver ownership repair: source and validation preparation

Status: **focused validation issued once; provider reports ERROR, raw failure collection pending; not accepted or delivered**. The existing phase-transfer full acceptance must finish and receive independent root acceptance before issuing this separate focused validation. No experiment is scientifically qualified by this preparation. Existing production browser checks and first failures remain immutable.

Baseline: `cc2fe02adac818b6af6c8834fa19ca36ab1540c4` (unchanged runtime plus frozen tests). Candidate: `d7f7b130d207f918033a2c0bf7a1aaeb5f78c5d6`, tree `2868a9fc70de4ca1ea62093192df37794adab2e7`. Both carry exactly the same twelve-test `crates/kerotakis-wasm/src/solver_contract_tests.rs`, SHA-256 `d0a4a52dcc59ccb4cfeb83373d512b85fe1b57c66c25d115729ed53d689ce4ad`. The fixed-only integration control `crates/kerotakis-core/tests/borrowed_solver_routes.rs` uses a newly introduced API; its absence on baseline is **not** an earned baseline failure.

The dependency lock SHA-256 is `668868136fa9ee1ebd6433d59eb68d5fa46224d874fd96c4036f7614b3c4cd85`. Candidate source blobs:

| File | SHA-256 |
| --- | --- |
| `crates/kerotakis-core/src/solve.rs` | `9c2a6414d95341b095e33cba6ce65f94a52d1b6bde057e79ae5af162d5d0e17b` |
| `crates/kerotakis-wasm/src/lib.rs` | `2c7b6a01ad8e18c8e2071a67fd5458bc247052bc99ee156c4ca328388abeb6fe` |
| `crates/kerotakis-core/tests/borrowed_solver_routes.rs` | `0c211e6b3789a997d71d66c783e931779c9ab811c7922a30836b2d54e6de3d3f` |

## Source contract and repair

The source audit in `audits/target468-20261016/localization-browser-preparation-20261020/README.md` found that the actual Wasm `CombinedSolver` inherited empty family-time boundaries, no-op direct-route recording and absent trusted aqueous readback. Its manually implemented child loop also bypassed native route validation, checkpoint rollback and outward-gas bookkeeping.

The candidate forwards those three hooks to their actual owners, refuses ambiguous aqueous authority and keeps solvent phase coupling unconditional. It extracts the existing native single-route transaction into a shared implementation and exposes range/borrowed-route entries on `SolverStack`. Both entries retain the stack's private required-conservation policy. The browser executes its installed prefix, concrete borrowed phase-coupled aqueous owner and installed tail through that same acceptance path. It does not assign a complete composite conservation claim to an arbitrary inner adapter.

Independent static comparison found the extracted native order preserved: applicability, checkpoint, evaluation, numeric/gas/identity and applicable conservation validation, accepted gas booking, failed rollback/owned diagnostic invalidation, then route/event recording. That is source review, not executed acceptance.

## Frozen test forecast

Forecast: **nine actual baseline failures, three positive controls; twelve repaired controls pass**. All unit names are fully qualified with `solver_contract_tests::` in the execution plan.

- Two actual `Lab` Fischer-family tests use the shipped structural oracle: distinct cold temperatures must report actual typed temperature gates; a warm uncatalyzed mixture must report its real catalyst gate. They preserve elapsed time, original doses, every phase's reagent inventory and absent product. They make no esterification-yield claim.
- Actual trapped-air sealing and a successful public pressure measurement must retain its owning direct route.
- The actual concrete analytic-water aqueous adapter must supply the same current-input answer/receipt through the composite as through the concrete browser adapter. No synthetic hook or manufactured receipt supplies this result.
- Clearly labeled adversarial transaction fixtures compare native and browser host behavior for mutation followed by error, invalid finite temperature, negative outward gas, accepted outward oxygen visible to the following observer, and finite unbalanced material creation under a required-conservation stack. These test acceptance machinery, not chemical yields.
- Positive controls retain unmatched-water/single-reagent clock and inventory without a Fischer match, prohibit an untrusted wrapper from obtaining a receipt from an existing solution, and refuse two concrete aqueous authorities.
- The additional fixed-only borrowed-route integration test rejects finite sodium-chloride creation under the installing stack's required conservation policy, with rollback and failed attribution.

The first ten-test draft SHA-256 `d8901d0c6d4806f02c3f243d8f3b7d33dd5e5635e30a738c11c9958c1548e95d` is preserved separately. Before any execution, static review replaced its unsupported assumption that 15 °C acetic acid must remain liquid with dose and all-phase ledger checks; the ten-test revision SHA-256 was `eb9a69f0088fc40f06262a22fce1093a624b92b29a4876e4810130696600ed51`. The final twelve-test baseline adds ambiguity and required-conservation controls. None of this pre-execution hardening changes a measured failure or original experiment forecast.

## Validation and evidence limits

The focused plan binds exact source/tree/lock and unchanged fixtures, preserves first raw stdout/stderr and exit status, rejects compilation failure as regression evidence, and requires all expected failures and positives by exact name. The first prepared worker used unqualified Rust unit names; independent review caught the module-prefix mismatch **before execution**. Its worker, reader, plan and receipt remain separately preserved; the successor keeps all assertions and uses actual qualified names.

After the prerequisite acceptance, issue only one bounded focused job. Require both baseline/fixed test results, the fixed-only borrowed adapter and existing transaction suite, formatting, locale/composed-locale checks, Clippy and Wasm-target compilation. Native focused `Lab` execution remains narrower than real-browser delivery or original-corpus qualification. Producer-recorded executable hashes and sizes are producer-attested bindings unless actual executable bytes are also retained and independently rehashed; raw test output alone does not supply those bytes.

After focused acceptance, complete applicable full own-fleet acceptance and source-bound browser delivery. Retain the prior 37 browser checks and seven targeted checks unchanged, and execute the supplemental real-browser localization design against authenticated served artifacts. Do not mark every application string localized or increase scientific experiment counts from these host-contract controls. Capture any remaining failure as a separately scoped source/protocol lane rather than weakening the acceptance.

The finalized preparation additionally requires the actual prior phase-full receipt in both the launch binding and worker input. The worker refuses before Rust/network setup if that receipt is absent; the launch process waits for its authenticated acceptance and resource gates, embeds exact receipt bytes/hash and issues at most once. The published pre-prerequisite worker is intentionally unlaunchable. Its first-line plan must be finalized from the real prerequisite receipt, not populated with a fabricated success marker.

Both actual baseline and fixed test executables will be retained compressed and independently streamed/rehashed by the reader. The fixed transaction summaries must be exactly one new borrowed-route control plus all38 existing named controls. Prefix-qualified unit names, source/tree/lock/fixture bytes, actual exit status and raw first failures remain mandatory; no changed-source result has yet been observed. No source integration or production delivery is implied.

This candidate branches from actual main `ed012116` before PR846 integration. Once the phase repair integrates, refresh this narrow branch with actual main, independently review the combined source and run its applicable full acceptance before normal exact-head integration. Preserve the once-only older baseline and focused source receipts separately; the earlier focus cannot certify a later combined head. The low-memory host does not run these Rust/browser builds locally.

## Actual first issued job

After actual phase-full acceptance, the focused job was issued once at09:34UTC on2026-10-20, version1. [Issue binding](issue-binding.json), [actual finalized plan](issued-plan.json), [actual worker](issued-worker.py) and [embedded prerequisite receipt](actual-phase-full-prerequisite.json) preserve the source and launch contract separately from the earlier intentionally unlaunchable preparation. Worker SHA-256 is `70fff8b7b905117ae55168409771915ab997d16e7f87b2a0a8cf8ee94e2ec02c`; prior-full acceptance SHA-256 is `0da542894254065fe0672c7b35c7a56ff1d81ee9d6f177b94c7e5ed02b5940e6`.

At09:37UTC the provider reported **ERROR**, with no failure detail in its status response. [First terminal status](first-terminal-status.json) is preserved. Raw collection waits for resource gates; no compilation, expected-baseline failure or repaired pass is accepted from this status. Inspect actual saved logs before any causal correction or separately issued successor. No automatic rerun is authorized by this preparation. Native focus remains distinct from actual-browser validation and integration.
