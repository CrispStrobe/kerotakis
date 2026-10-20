# Real-browser localization acceptance preparation

Status: **NOT EXECUTED; blocked by a source-proven Wasm wrapper gap.** This preparation adds no scientific selection and certifies neither deployment nor complete application localization. Preserve the existing 37 browser controls and seven targeted production controls unchanged. Run this supplementary acceptance only against authenticated newly shipped artifacts after the separate wrapper repair and its source-bound acceptance; do not modify the active phase-transfer repair to make these checks green.

`locale-controls.mjs` exports `runLocaleDeliveryChecks(page, binding)`; importing it performs no requests or browser launch. Its caller supplies a fresh actual HTTPS browser page, a full expected engine commit and an authenticated map of served frontend filenames to SHA-256 digests. The module fetches and verifies those bytes, imports the actual public Wasm/PHREEQC modules, installs the real solver hook, checks `Lab.meta()` source identity, preserves results and stops at the first failed assertion. A failed call attaches its partial report to `error.localeDeliveryReport`; retain that first failure. The executable design has only received `node --check`, not browser or chemistry execution. The release binding must include the actual Wasm binary, Wasm loader, application module and PHREEQC module/binary; five arbitrary assets are insufficient release evidence. Preserve the originating build lock and asset manifest separately.

## Eighteen supplemental cases

Each of German and French gets these nine cases, using actual public `Lab.setLocale`, `runScript`, `snapshot`, `restore`, `meta`, `aqueousDatabase`, `setSolver` and `free` APIs:

1. Ordinary live water: real analytic aqueous characterization supplies the pH; no neutral-unspeciated fallback note is expected.
2. Restored uncharacterized water: LV2 and LV3 measurements carry the localized neutral-water Autoprotolyse/autoprotolyse caveat.
3. Restored uncharacterized water plus glucose: the same caveat, within the actual fallback domain.
4. Restored uncharacterized warm water: honest missing-solution refusal, without the blank caveat.
5. Restored uncharacterized dilute acid: the same refusal.
6. Restored uncharacterized dry vessel: the same refusal.
7. Actual shipped Fischer-family substrates at 15 °C: structured temperature gate and the actual numeric state render in the selected language.
8. The same real family at 25 °C: a second distinct actual temperature coordinate, without an English gate or raw `temperature_k` fallback.
9. The same real family at 65 °C without catalyst: localized sulfuric-acid/hydrogen-ion alternatives render from the nested structured reason.

The first two blank controls are **engineered uncharacterized-state presentation fixtures**. Their public snapshots lose only derived aqueous answers (`solution`, `resolved`, `free_proton`, `free_hydroxide`); physical inventory, identity, temperature, phase and clock are compared before/after restore. They do not establish that an ordinary water addition naturally enters this state. Measurement must leave physical/derived vessel state unchanged and make no additional external-engine call. If real glucose preparation falls outside the supported blank domain, retain that failure and inspect the genuine fixture; do not edit its temperature or composition to manufacture acceptance.

The real-family cases must find the actual shipped structural-oracle match and nested reason. No synthetic family, injected event or fabricated yield is allowed. They require retained acid/alcohol, no ethyl-acetate product and no Fischer reaction event; this tests a condition boundary, not an esterification yield. The cold gate checks the stated minimum 333.15 K against actual vessel temperature. The warm gate checks actual temperature, absent sulfuric acid and absent free-proton catalyst. Their first assertions remain intentionally strict despite the wrapper defect below.

These checks exercise engine-rendered product text in an actual browser, as consumed by the terminal/worker presentation paths. They do not click every UI control, inspect all Svelte DOM labels or prove that every string in the app is localized.

## Source-proven blocker

Reviewed engine source: `b4daecf742be080577e9fda57fc1ef55aefef3d3`. The entire `crates/kerotakis-wasm/src/lib.rs` file is byte-identical at `2e41d17a` and this reviewed head; the four phase-transfer source edits are unrelated. Its SHA-256 is `fae137348cda8c6d7778122ef2a09c40b4e50b1ead567f946bb2391af76642dd`.

`Lab::new` creates the real standard stack. `Lab::run` supplies `CombinedSolver` to `Bench::step_with`. On a positive wait, `Bench::step_with_inner` calls that supplied solver's `time_boundaries`. `SolverStack::time_boundaries` forwards to its installed solvers, including the actual `kerotakis_org::family_oracle::family_equilibrator()` installed by `kerotakis-stack`. `FamilyRouter::time_boundaries` constructs the typed `not-modeled.family-declined-at-gate` event with `family`, `version`, a `family-gate-label` term and a nested gate recipe. **CombinedSolver implements no such hook and inherits the empty-vector default**, so this route never reaches the family label, actual temperature or catalyst alternatives in the public Wasm path. Native family rendering tests cannot establish delivery through this wrapper.

Relevant repository coordinates at the reviewed source:

- `crates/kerotakis-wasm/src/lib.rs`: `Lab::run`, `CombinedSolver`, `BrowserAqueous`, `run_solvers`.
- `crates/kerotakis-core/src/bench.rs`: positive-wait `solver.time_boundaries` call and `record_direct_model_route`.
- `crates/kerotakis-core/src/solve.rs`: `Equilibrator` defaults, `SolverStack` forwarding, checkpoint and `validate_result` acceptance.
- `crates/kerotakis-core/src/family.rs`: `FamilyRouter::time_boundaries` and nested decline recipes.
- `crates/kerotakis-stack/src/lib.rs`: shipped family router installation.
- `crates/kerotakis-core/src/aqueous_continuation.rs`: `characterize_via`, `characterize_optional_via`, `continue_via`.

## Complete wrapper trait audit

All twelve `Equilibrator` methods were inspected at the reviewed head. This distinguishes proven omissions from defaults whose product impact has not been demonstrated.

| Method | CombinedSolver behavior | Assessment |
| --- | --- | --- |
| `name` | Explicit `wasm-stack` | Intentional aggregate name. |
| `trusted_aqueous_readback` | Default `None` | Concrete inner PHREEQC adapter and `BrowserAqueous` opt in; outer wrapper loses this capability. Continuation helpers explicitly refuse without it. Public-call impact needs actual regression evidence. |
| `element_conservation_tolerance` | Default `None` | Do not blindly forward an inner claim to a composite. Audit and enforce each owning route's policy. |
| `route_kind` | Default `Computed` | Aggregate default alone is not a demonstrated scientific defect. Preserve constituent route kinds. |
| `applies` | Default `true` | Phase-coupled browser pass deliberately runs unconditionally; blanket forwarding can break phase physics. |
| `chemistry_applies` | Explicit aqueous-or-stack | Forwarded aggregate claim. |
| `equilibrate` | Explicit manually ordered stack/aqueous/stack | Additional transaction discrepancy below. |
| `record_route` | Default no-op | Direct pressure, gas-test, starch/iodine appearance and explicit curated-reaction attribution is dropped after Bench calls this supplied wrapper. Actual direct measurements may still succeed. |
| `time_boundaries` | Default empty vector | Proven loss of real family wait-gate events described above. |
| `mix` | Default `None` | Trait explicitly permits fallback to equilibration. Native PHREEQC supports MIX; performance or result impact remains unexecuted, not an automatic failure. |
| `equilibrate_delta` | Default clone/equilibrate/diff | Documented general fallback; no direct loss established. |
| `capability` | Default aggregate boolean report, no validity | Not a substitute for constituent domain reports; no additional current defect proved solely by this default. |

`BrowserAqueous` explicitly forwards trusted readback, `applies` and `chemistry_applies`, supplies its own name, and wraps equilibration in displacement. Other trait defaults require the same ownership analysis rather than automatic forwarding.

There is a separate source-proven acceptance divergence: native `SolverStack::equilibrate` checkpoints each route, validates numeric state, gas events, target identity and applicable conservation policy, books gas boundaries, then restores a failed proposal and invalidates failed owned aqueous readback. Wasm `run_solvers` calls each solver directly and retains a mutation on `Err`, recording a failed event; it does not perform that native acceptance transaction. The external aqueous branch likewise returns on error without this per-route native checkpoint. This proves divergent control flow, **not** that any particular shipped experiment has corrupted inventory. A shared acceptance implementation is preferable to copying an expanding list of checks or merely patching the narration hook.

## Separate repair lane and acceptance

After the active phase-transfer delivery is accepted, create a narrow branch from actual main for the wrapper contract. Preserve the first failing source and all original assertions. Fix or share delegation/transaction ownership at its source; do not change the Fischer forecast, lower event expectations or forward aggregate conservation claims without justification.

Required source tests should instantiate the actual `Lab` and shipped family configuration. Public-call fixtures use the exact scripts in the supplementary module; two distinct cold temperatures and the warm catalyst refusal must fail on baseline because the real event is absent. Verify elapsed clock, phase-qualified retained reagent amounts, no invented product, typed reason slots and rendered DE/FR text. Include water-only and a single unmatched reagent wait as negative family controls: elapsed clock still advances and no unrelated Fischer match appears. These are gate-observation controls, not chemical-yield experiments.

For direct-route delivery, use a genuine successful public `measure v1 pressure` on a physically admitted gas/headspace state, capture the actual `measured` event and require the returned `routes` record `instrument:pressure-gauge` with matching vessel, computed kind, nonchemical ownership and successful one-event attribution. Pair it with an actual refused instrument operation and require no success route. Confirm the shipped grammar and fixture's physical admission before freezing executable assertions; do not count an empty/default pressure as a valid measurement.

For trusted readback, a focused Wasm-module test may access the real private CombinedSolver and call the public continuation bridge with its actual concrete aqueous adapter. It must characterize the current input, invalidate on a genuine unsupported/current-input failure, and never manufacture an opt-in receipt from `Some(solution)`. Include corresponding native adapter control and no-opt-in refusal. Do not claim a public `Lab` regression until a real public operator exercising this bridge has been identified.

For rollback, put failure-proposing solvers only inside a focused **transaction-contract** test (not the scientific/browser family fixture). Assert actual pre/post vessel equality, failed route/event attribution, later independent solver execution, gas-boundary ownership and stale aqueous diagnostic invalidation. A synthetic adversarial writer here tests acceptance machinery and must be labeled as such. A separate real shipped model/API control is needed to establish product benefit, without inventing unsupported chemistry to trigger it.

Run source-bound focused baseline/fixed tests first, preserve baseline failures and positive controls, then applicable full own-fleet acceptance. Only after that ship authenticated new artifacts and run the unchanged 37 plus seven prior browser controls and these 18 supplementary cases. Keep source, lock, bytes, first failures, stdout and browser receipts bound. This preparation launches none of those jobs.
