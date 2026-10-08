# Prepared refusal/preflight repair

The repair is stacked above safety-veto PR #762. It is prepared for review; no runtime acceptance is claimed. The original 28 source-informed contracts and their freeze receipt precede the production change. Static draft review identified two missing-vessel compatibility cases, frozen separately before the production commit. Both receipts and original test files remain preserved.

Only `crates/kerotakis-core/src/bench.rs` changes production. Explicit no-water evaporation, unsupported-nuclide, unknown-reaction and single-phase Drain refusals gain Unchanged disposition. Drain receiver creation follows its read-only layer/partition-domain checks. Decant/Filter creation follows validation/source lookup. Existing safety-veto rollback and accepted creation/warning/success order remain in place. Unknown reaction/nuclide branches explicitly retain the previous NoSuchVessel exception for an absent vessel.

No generic diagnostic policy, complete-Bench cloning, public schema/API, stock, solver, numerical or thermal implementation changes are included. Partial evaporation notices, supported reactions/spikes, valid transfers and intentional accidental spills retain settlement.

Next, preserve a hosted pre-repair baseline at the test-only commit `5312bc77`, execute the same suites at the repaired revision, then run required full CI. Command: `cargo test -p kerotakis-core --test refusal_preflight_atomicity --test refusal_missing_vessel --test safety_veto_atomicity -j1 -- --test-threads=1`. Expect 30 new and 20 inherited controls to compile; report actual observed counts, not this forecast. Preserve compile/harness failures separately, raw logs, revision/toolchain, production/test/manifest hashes and the generated lock hash after Cargo. A branch-only baseline workflow must never merge.

Static formatting, diff and immutable fixture-hash checks passed. Hosted execution and integrated behavior remain pending. See `FORECASTS.md`, `freeze.json`, `missing-vessel-freeze.json` and `prepared-repair.json` in this directory for the exact scope and bindings.
