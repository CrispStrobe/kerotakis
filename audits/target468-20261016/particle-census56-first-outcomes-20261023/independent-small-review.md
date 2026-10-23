# Particle census focus: independent saved-result review

## Scope and provenance

Reviewed only small saved JSON/text records and their manifest bindings. No compressed archive body was read or rehashed; no native/provider call or test rerun occurred. Therefore this review does not independently verify the archived binary/executable bodies.

The first collection attempt is preserved as stopped before its first fetch: `actual/STOPPED.json` records an `AssertionError` at the memory/load/disk gate and `no_retry_or_reissue: true`. The follow-up `actual/resource-gate-followup/STARTED.json` binds to that original issue and stop, keeps version 1 and the same runtime source, and names collector SHA-256 `66c075cd6eb6bedef8873d71c3c8fc034278da84dca1e580a118dabb6d2928fa`. Its `COLLECTED.json` reports terminal `COMPLETE`, serial streaming NAS readback verified, original stop preserved, same producer, and no scientific verdict.

Source/runtime binding:

- Issued and actually executed source: `283c3bfcd01e5e6bf6527bc91c9d9de1f923062d`.
- Runtime tree: `d75a45857b7ea484073d39a295e985726abc6bd6`.
- Actual producer lock SHA-256: `41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb`.
- Worker SHA-256: `70626c047f86d5f19d6d15cb79c8f306c7f52e31f3a03aff88c05dcae0805718`.
- Plan SHA-256: `7af92bbf40ce65031ef4c4a12031c531b804df6d9bd67f223ddde552834c6afc`.
- `frozen-plan.json` hashes to that plan value and parses identically to the externally retained plan.
- Manifest SHA-256: `415853df22acfa8b0fe8466c0afe67534a75565a8a614b0433b3012b6a79a033`.
- Completion-record SHA-256: `490e2b61e599a18b1a09aecd4794c85906639d5b74dad271901a23018a2e7aa8`.
- `actual-cli-binary.json` metadata SHA-256: `863c19cf80d917578deb69fd72dbf34878913f46eff6d0177411e1e4ea0ea40c`. It reports a debug CLI executable of 104,117,920 bytes with SHA-256 `57a01553d720f667f3da306a933f2de560be11df1ea2dd5fb3d426abd2676467`, stored as a 24,567,579-byte gzip with SHA-256 `b81c1e9e9c4fdb90bf4445fa847cc01a1daea59d2df98dd1694c12bf0747eb16`. These are saved metadata claims; this review did not read or rehash the compressed executable body.

The manifest has 287 members (278 non-gzip metadata/log members and 9 gzip archives), total reported transfer 152,195,450 bytes. All 278 non-gzip members were locally checked against both their manifest SHA-256/length and collector file rows; the manifest itself also matched the collector row. The closed collector set contains those members plus `manifest.json`. This authenticates the saved small records, not the contents of the nine archives.

The later `TEST-ALLOCATION-SUCCESSOR.json` is a separate proposed unit-only allocation change from source `283c3bf` to `ed7d8c5`; it changes `aqueous.rs` from a `Vec` borrow to a fixed-array borrow and says production prefix bytes are identical. The actual focus remains bound to source `283c3bf`; none of the later source is represented as executed evidence.

## Exact named-control results

The frozen plan defines 56 unique `(target, test)` identities across eight targets. The completion record contains exactly those 56 identities once each, all exit status 0, complete, and with the exact named test row `ok`. Each selected test record uses `--exact <test> --nocapture --test-threads=1`; all plan-selected names occur in that target's test listing. The eight targets have 13, 8, 12, 5, 2, 6, 1, and 9 selected tests respectively. The PHREEQC target lists 117 available tests, of which only the nine named plan controls were run; this is not a claim that its full suite ran.

The two `particle_census` controls are:

- `original_nearby_and_no_silver_cooling_and_partial_rewarming_settle`
- `native_particle_census_retains_precision_and_matches_independent_mol_columns`

Both are recorded complete with exit status 0 and the exact named test row `ok`. These focused outcomes are test results, not scientific catalog qualifications. Completion metadata assigns zero scientific selections and explicitly says the forecast is not acceptance and this focus is not the full fleet.

## Exact original Ca/Ag CLI result

The small raw CLI record binds `original-cycle-cli.json` to the debug CLI command running `audits/phase-particle-precision-20261023/corrected-cli-protocol.kero` with JSON output. That protocol SHA-256 is `a95e59408c36731599dcdf2a95457be77ddbfd1c6282a720fc80e0c2a5ec1ec8`; its five commands are exactly: add 100 mL water, add 0.05 mol CaCl2, add 0.005 mol AgNO3, cool by 20 kJ, inspect v1. The saved CLI record has exit status 0, stdout SHA-256 `5c319cf4db38da0f8fb2db3fb1557cda3992237eeced3d482540e82938e45de9`, and empty stderr.

Small raw JSON lines show no `solver_failed` event. After the 20 kJ cooling step:

- Vessel temperature: `270.026519111472 K`.
- Current numerical solution particle molality: `1.707985377835 mol/kgw`.
- The latest water liquid-to-solid phase event is at `270.02651911147143 K`.
- The displayed vessel melting/liquidus value is `270.02651911147143 K`.
- Vessel/event/liquidus discrepancies are below `1e-12 K`, within the requested `1e-6 K` criterion.
- The vessel contains both `0.6583447277849188 mol` solid water and `4.875932263611131 mol` liquid water, with `0.004999505642044 mol` solid AgCl.
- Delivered cooling is `20000.00000000001 J`.
- The subsequent `inspect v1` record still carries numerical particle molality `1.707985377835`.

The raw record reports the WATEQ Debye-Hückel fallback at a potentially concentrated load beyond its documented approximate ionic-strength range because Pitzer cannot represent all of the requested chemistry. The convergence and phase agreement are an observed solver result for this case, not validation of concentrated-mixture activities, calibration, or broad product accuracy.

## Full sweep result

The separately recorded `full-1536-sweep` ran 1,536 states and exited 1. Its saved summary records 10 steps with a refusal and two `StatesAgreeWithTemperature` violations:

1. Water 100 mL + Na2CO3 0.02 mol + CH3COOH 0.05 mol + cooling 20 kJ: liquid temperature `251.04 K`, liquidus `271.47 K`.
2. Water 100 mL + NaHCO3 0.05 mol + H3PO4 0.02 mol + cooling 20 kJ: liquid temperature `247.92 K`, liquidus `271.71 K`.

Sweep stdout SHA-256 is `01529806cb9a89fde5d6a4dc50d7da18cf08b534414f81238e68a3b2b8523898`; stderr SHA-256 is `4e7077253ab42bbf469a2e1cbb06d468cce80c3f5d0a02a0ed759b6fd5401469`. The full sweep is a failure with these two new carbonate/weak-acid counterexamples, not a pass. The focused 56-control result does not cancel those failures.

## Review conclusion and remaining limits

The requested exact original Ca/Ag CLI shows an actual current numerical solution, no solver failure, and sub-microkelvin temperature agreement with both its liquidus and final water phase event. The 56 plan-named controls are fully accounted for and passed their exact invocations. The wider sweep fails on the two carbonate/weak-acid cases above. Do not promote the focused result to full source/platform/product acceptance, fleet coverage, or scientific experiment credit. The archived compressed binary and executable body hashes remain producer/collector assertions until an authorized separate byte-level verification reads them.
