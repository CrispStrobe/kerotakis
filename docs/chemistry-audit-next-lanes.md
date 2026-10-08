# Chemistry audit: actionable next lanes

Read [the current checkpoint](chemistry-audit-status-20261005.md) first. These are proposed, uncompleted tasks, scoped for a fresh agent. They cover audit followup, not every pre-existing product roadmap item. Keep existing task numbers unchanged; the IDs here form a separate namespace.

## Shared execution contract

Starting points below are repository-relative paths at [accepted source 8cdab1f6](https://github.com/CrispStrobe/kerotakis/tree/8cdab1f6ecd0b1132c4008014a5a0c5476429d17). Some do not exist on main until AUD-00 ports them. Inspect current definitions instead of relying on historical line numbers. A design/research lane may start on the audit branch before integration; a main-based implementation must first carry its prerequisite changes.

Each lane must produce independently declared expectations, a preserved failing baseline when applicable, positive supported controls, explicit refusal/rollback controls, a small implementation diff and a source-bound result report. Classify execution success, bounded agreement, qualified agreement, expected refusal, author error and unmet expectations separately. Do not fit bounds to output, regenerate original forecasts, recognize experiment IDs in runtime code or treat an unsupported route as a scientific pass.

Use existing hosted validation in `.github/workflows/chemistry-audit.yml`; bind replay to a successful `validated_run` and its exact `validated_commit`. Run only the checks needed for the change, then required integration gates. Do not substitute the selected 17-stage audit for the repository's full required PR gates. One workflow concurrency group cancels older runs on the same ref: wait for completion before dispatching a dependent replay. Private machine policy controls local execution and agent count.

## Current priority order — 2026-10-08 14:15 UTC

Use [the integration checkpoint](../audits/integration-20261008/STATUS.md) for actual delivered heads and reviewed receipts; the earlier dated checkpoint is historical. The measured startup cache is merged with bounded single-host gains. Additional optimization is lower priority than the remaining ownership and transaction seams.

1. Preserve merged PR #767: the reviewed thermal diagnostic passes all six repair controls (baseline three failures), and the amount diagnostic establishes 24 baseline failures repaired under 37 controls. All five required checks passed on head `bce5eb22`; merge `158a29f5` is recorded in the integration receipt.
2. Preserve [the reviewed main-tree CLI replay](../audits/main-cli-replay-20261008/README.md): all 100 disclosed corrected cases and three strict distillation controls pass their checks across 164 processes. Fifth retains three refusals and one model qualification; sixth retains one qualification. Its source tree matches main `b4455b78`, not later PR #767. Keep original experiment scores and post-result adaptations distinct.
3. Complete TXN-01's first tranche, [PR #768](https://github.com/CrispStrobe/kerotakis/pull/768). [The snapshot diagnostic](../audits/solver-snapshot-contracts-20261008/README.md) verifies 14 baseline failures repaired under 16 frozen controls. A separately frozen emptiness control covers edited unchanged proposals. [The stack review](../audits/solver-stack-atomic-contracts-20261008/README.md) confirms baseline 2/13 and repair 13/13; [the boundary review](../audits/solver-boundary-contracts-20261008/README.md) confirms baseline 3/12 and repair 12/12. All 604 inherited core library tests passed on both sources in each of these two comparisons. The first stack run failed before compilation due to a checkout-ref typo and remains separately preserved. The PR now targets main and includes merged #767 at head `6ff42221`; require fresh matching-head gates and all 42 integration functions. CI also runs on stacked PR bases; the earlier contrary note was incorrect. Numeric validity and explicit unit-scale event balance remain separate from physical model support, the legacy absolute floor, complete owner/charge accounting and reaction energy. Historical `solver_transactions` retains additional requirements and is not yet fully ported. Do not transplant entire historical source files or bypass conservation for snapshots.
4. Continue AUD-00 from [the exact missing-file inventory](../audits/integration-20261008/full-audit-port-inventory.json). Port compatible test-only targets and their original freezes first, then implement dependencies per lane. Restore every historical binding and exclusion before claiming complete historical acceptance. Follow with NUM-01/NUM-02 authoritative ownership and NUM-03 chemical reconstruction; those are prerequisites for full still donor/receiver/energy closure.

## Dependency and ownership map

| Lane | First deliverable | Depends on | Main implementation seam |
| --- | --- | --- | --- |
| AUD-00 | Integration inventory and ordered PRs | None | Existing branch changes and CI |
| AUD-01 | Checked native phase budget | AUD-00 for main port | `kerotakis-phreeqc/src/aqueous.rs` |
| AUD-02 | Witnessed/bounded raw aqueous reconciliation | AUD-01 | Same native readback seam |
| NUM-01 | Amount ownership/API/save decision plus one owner | AUD-00 for main port | `amount.rs`, `vessel.rs`, persistence |
| NUM-02 | Complete mechanical inventory writer migration | NUM-01 | `bench.rs`, `delta.rs` |
| NUM-03 | Chemical reconstruction certificate | NUM-01; coordinate AUD-02 | `solve.rs`, native reconstruction |
| NUM-04 | Compensated still debit/receiver/energy closure | NUM-02, NUM-03 | Still and fractional transfer |
| OWN-01 | One supported interface owner certificate | AUD-00; coordinate NUM-03 | Strict ledger, surface/exchange owners |
| OWN-02 | One supported material owner certificate | AUD-00; chosen owner definition | Material/object/soap ownership |
| VAL-01 | Relative trace/valence error policy | AUD-01/AUD-02 where shared | Native trace reconstruction |
| OBS-01 | Shared observable coverage contract | Design can start now | Instruments and presentation DTOs |
| DATA-01–03 | One reviewed identity/property/optical slice each | OBS-01 for presentation integration | Registry/property/optics seams |
| ENG-01 | One reaction heat coverage/certificate slice | OBS-01; inventory policy | Thermal acceptance |
| ENG-02 | Explicit imposed-headspace work protocol | Boundary inventory definition | Headspace operations |
| KIN-01 | One evidence-supported rate domain | Reviewed data and inputs | Kinetics/electrochemistry |
| ELEC-01 | Transient migrating layer | Steady-layer contract | Interfacial transport |
| TXN-01 | Cross-operator failure-injection matrix | AUD-00 for main port | Bench/delta/solver acceptance |
| UX-01 | Chemical claim consistency matrix | OBS-01; coordinate TXN-01 | CLI/JSON/web/locales |
| EXP-01 | Next independently frozen fifty | Syntax only until freeze | CLI experiment harness |
| PERF-01 | Release baseline, then one measured optimization | Stable accepted implementation | Profiled hot path only |

AUD-01/AUD-02/VAL-01 must not edit native reconstruction concurrently. NUM-02/NUM-04/TXN-01 share `bench.rs`; agree on ownership before edits. NUM-03/OWN-01/OWN-02 share conservation seams. Data research, checker design and presentation work can proceed separately once their contracts are stable.

## AUD-00 — Integrate the accepted audit branch without losing evidence

**Problem:** accepted branch behavior is not yet main behavior. Start from the checkpoint's main/branch comparison, then refresh it.

**Do:** inventory `git log main..audit/systematic-chemistry-20261002`, patch dependencies and touched files. Group portable PRs by behavior: transfer/spill/extraction; final safety/titration; strict conservation/Amount/stock; numeric/nuclide/observable/narration changes; crystal ownership/readback; evidence/harness updates. Preserve prerequisite tests and freezes with each group. Re-author or cherry-pick coherent changes only after comparing current main; a single late repair commit cannot stand alone when its foundation is absent.

**Validate:** each PR runs its focused unchanged contracts and required PR gates. The integrated result gets a fresh full audit and third/fourth replay against its own executable. Preserve the four disclosed legacy exclusions and every qualified original forecast. Investigate conflicts by intended behavior, not by choosing an entire file from one branch.

**Done:** merged PR links, an integration manifest mapping original commits to accepted changes, a new exact-source acceptance receipt and an updated checkpoint. Do not claim deployment without deployment evidence. Documentation cherry-picks are a separate delivery, not completion of this lane.

## AUD-01 — Reject native phase overdraw before remainder clamping

**Start:** `crates/kerotakis-phreeqc/src/aqueous.rs`, `crates/kerotakis-phreeqc/tests/solid_solution.rs`, [raw-guard plan](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/solid-solution-raw-guards-20261004/NEXT.md).

**Do:** define one nonduplicated available Ca/Sr/C budget from initial aqueous, primary solid, typed crystal and closed gas owners. Check final solid/gas allocations before computing an aqueous remainder. Audit what `problem.totals` already contains. Reject missing, negative/nonfinite gas readback and nonfinite sums; do not let `filter_map` remove owned gas. External carbon exchange remains an explicitly open boundary; Ca/Sr still close.

**Freeze:** initial Ca 0.005, Sr 0.004, C 0.009 mol; final typed CaCO3 0.002 and SrCO3 0.003 mol leaves aqueous 0.003/0.001/0.004 mol. Accept that supported fixture. Refuse CaCO3 0.006, SrCO3 0.005 or excessive closed CO2. Include primary-solid co-owners, exact exhaustion, tiny scaled inventories and malformed gas columns. Declare rounding/arithmetic allowances independently before testing.

**Done:** direct budget controls and existing live precipitation/dissolution/repeat tests pass, overdraw refuses before projection, and native/stack callers preserve complete state on refusal. This establishes bounded element allocation, not speciation or heat accuracy.

## AUD-02 — Bound raw aqueous correction and prove reporting normalization

**Start:** native selected totals/readback in `aqueous.rs`; `vendor/iphreeqc/src/phreeqcpp/print.cpp` (`punch_totals`) and `model.cpp` (master-total accumulation).

**Do:** compare raw aqueous amounts with AUD-01's independent remainder before rescaling. Review selected-output semantics for each supported route. A discrepancy equal to crystal inventory is not proof that a total legitimately includes it. Where an inclusive representation is demonstrated, obtain a separate aqueous witness from the same solve and normalize only against identified owners. Isolate or refuse surface/exchange combinations until their accounting is witnessed.

**Freeze:** exact exclusive totals accept unchanged; 10% excess/deficit and zero aqueous amount with a positive remainder refuse. Apparent inclusive totals without a witness refuse. Only a proven inclusive total plus matching witness may normalize. Include inside/outside budget boundaries, both large and trace inventories, cache replay and open-carbon controls.

**Done:** no unexplained material synthesis or unbounded rescaling; output rounding, arithmetic error and permitted solver residual have separate documented budgets. Existing supported native cases still pass. Full H/O/charge/speciation certification is not part of this slice.

## NUM-01 — Decide authoritative amount ownership, APIs and save compatibility

**Start:** `crates/kerotakis-core/src/amount.rs`, `stock.rs`, `vessel.rs`; [migration contract](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/compensated-amount-contracts-20261004/MIGRATION.md).

**Do:** document the choice between changing public `Moles(f64)` and changing public inventory entry APIs, including struct literals, `.0` consumers, WASM DTOs and lots. Choose one narrow nonreacting owner and keep its compensated amount authoritative. Do not attach a residual sidecar to mutable/replaced portion vectors. Treat scalar model/display values as projections with explicit error contracts.

**Freeze:** old scalar saves load with zero low component; new saves retain normalized finite pairs; old readers reject nonzero residual-bearing schemas; malformed/negative/overflow states refuse. Test tiny changes against bulk background, duplicate portions, cancellation, exact exhaustion and rollback. Existing opt-in stock behavior and default conservative policy remain unchanged.

**Done:** a reviewed API/schema decision, complete writer/reader inventory and one fully migrated bounded owner. Primitive tests alone do not complete vessel migration.

## NUM-02 — Migrate mechanical inventory mutations completely

**Start:** `vessel.rs` deposit/withdraw APIs, `bench.rs` transfer preparation and operators, `delta.rs` scalar subtraction/replacement, spill and extraction paths.

**Do:** migrate every writer of NUM-01's chosen owner, including direct vector replacement, split/coalesce, lots, spill/recovery, discard, selected-phase transfer and persistence. Expand the mutation inventory with a workspace-wide search before marking coverage complete. Audit authoritative versus provenance-lot amounts explicitly.

**Freeze:** repeated tiny transfers, widely separated amounts, exact exhaustion, mixed duplicate portions, split/recombine, save/reload and failure after each preparation stage. Independently bind requested amount, actual donor debit and receiver credit; projections cannot certify the debit.

**Done:** all covered mechanical writers preserve the authoritative pair or refuse atomically. Existing positive transfer/safety contracts remain supported. Chemical solver reconstruction waits for NUM-03.

## NUM-03 — Reconcile compensated ownership through chemical reconstruction

**Start:** `solve.rs`, `required_conservation.rs`, native `aqueous.rs` content replacement, `ledger.rs`, adsorbed/electrode/gas quantities and custom solver proposals.

**Do:** specify elemental/formal-charge ownership when scalar solvers change species. Never retain an old per-species residual after its species reacts, and never discard it silently. Coordinate native scalar reconciliation with AUD-02. Permit only tested reconstruction domains; refuse unsupported allocations explicitly.

**Freeze:** neutral dissolution, precipitation/redissolution, redox speciation, closed gas exchange, custom routes replacing content, trace against bulk background and invalid proposals. Distinguish unchanged owner residuals from reaction-created identity changes.

**Done:** accepted reconstructed states have a bounded conserved ownership certificate, rejected states roll back, and the report states which reaction/owner domains are covered. Energy and nuclear certificates remain separate.

## NUM-04 — Close still donor/receiver precision without rejecting valid small cuts

**Start:** `bench.rs` distillation/fractional transfer, `crates/kerotakis-thermo/src`, preserved distillation/still/fractional control suites.

**Do:** carry authoritative amounts through component selection, donor removal, receiver creation, residue coalescing and thermal bookkeeping. Preserve supported `1e-14` cuts and their latent-energy contract; adding a strict scalar-relative debit guard alone would violate existing forecasts.

**Freeze:** pure and mixed small cuts, several stages, near exhaustion, bulk donor plus tiny requested cut, receiver background, latent/sensible energy, refusal and save/reload. Compare requested, debited and credited authoritative amounts separately from display precision.

**Done:** complete supported cuts have component and energy budgets; unsupported cuts refuse before mutation. All original still controls pass unchanged, and remaining low-component limits are documented rather than advertised as arbitrary precision.

## OWN-01 — Admit one fully typed surface or exchanger owner

**Start:** strict owner refusals in `required_conservation.rs`, `vessel.rs` surface/exchange structures, `delta.rs`, native surface/exchange transport suites.

**Do:** choose exactly one closed supported owner model. Specify support/site composition, bound species, charge/protons and released water before admitting it. Include the owner once in the molecular ledger; derived distributions are not a second inventory. Keep incomplete/unrelated interface models refused.

**Freeze:** unchanged typed owner, supported bind/release, proton/water exchange, trace loss/gain, malformed capacity/composition, duplicate owners and save/reload. Use independently specified atom/charge budgets and direct rollback assertions.

**Done:** this one owner can obtain a strict certificate with complete composition. Native equilibrium, kinetics and all surface models are not thereby certified.

## OWN-02 — Give one incomplete material class an honest ownership contract

**Start:** unresolved material/object/soap refusals in the strict certificate; `material.rs`, `vessel.rs`, `bench.rs`, preparation and separation paths.

**Do:** pick one model whose composition can actually be represented. Separate characterized molecular content from unresolved mass, support mass, aggregate membership and provenance lots. Preserve unknown composition as unknown; do not invent a formula to make a ledger pass. Define what a supported preparation changes and what it leaves unresolved.

**Freeze:** preparation/transfer/separation before and after characterization, mixed represented/unresolved content, exact mass ownership, malformed saves and refused reconstruction.

**Done:** one explicit supported owner has complete accounting and clear capability limits. Others still refuse. Independent optical, partition and biological properties need their own data.

## VAL-01 — Replace unexplained trace/valence floors with a justified policy

**Start:** native trace readback and oxidation-state fallback in `aqueous.rs`, `tests/trace_inventory.rs`, order-invariance/native-scale controls and strict ledger boundaries.

**Do:** inventory absolute noise thresholds, identify each quantity basis and distinguish numerical artifacts from genuinely owned trace valence fractions. Derive output-rounding and arithmetic bounds; use relative conserved budgets where appropriate. Coordinate with AUD-02 instead of layering a conflicting reconciliation rule.

**Freeze:** matched cases differing only in a real trace fraction, trace-only and trace-on-bulk inventories, scale sweeps, cache hits/misses, missing valence names and round-trip native serialization. Unsupported reconstruction must refuse, not erase trace ownership.

**Done:** every changed floor has a documented domain and positive/negative boundary controls. Do not promise arbitrary subnormal precision or relax solver convergence to make fixtures pass.

## OBS-01 — Define one observable coverage contract across outputs

**Start:** `instrument.rs`, `sparse_optics.rs`, `solution_optics.rs`, quantity/property coverage and web/WASM measurement DTOs.

**Do:** specify status, reason, phase scope, assumptions, provenance and validity for each observable dependency. Distinguish computed zero, missing data, outside calibration, qualitative surrogate and unsupported route. Start with heat/temperature and absorbance, then extend one observable at a time. A thermostat establishing temperature does not supply missing reaction enthalpy.

**Freeze:** missing-data injection, aqueous versus organic layers, supported and out-of-range conditions, genuine zeros, persistence and equivalent CLI JSON/prose/web descriptions.

**Done:** covered outputs never look fully confident when required inputs are absent, while unrelated supported readings remain usable. The coverage matrix names the next uncovered observable explicitly.

## DATA-01 — Add one missing registry identity with mandatory property evidence

**Start:** `species.rs`, registry source/validation, MgCl2 refusal in the preserved third fifty and existing Mg/Cl ionic masters.

**Do:** scope anhydrous MgCl2 first, or explicitly choose one hydrate. Review formula, identity/aliases, measured required solid properties and dissolution coverage; hydrate water and enthalpy cannot be copied from CaCl2. Publish only licensed, attributed scalar data with conditions and uncertainty. Unavailable mandatory properties remain a blocking coverage reason.

**Freeze:** canonical/alias lookup, phase-aware addition, supported dissolution, stoichiometric Mg/Cl/water budgets, missing-property/refusal cases and hydrate distinction if included.

**Done:** one reviewed identity is reachable through the normal registry pipeline with correct coverage reporting. No undocumented family-wide constants or empirical calorimetry claim.

## DATA-02 — Calibrate one dissolved-density/volume domain

**Start:** density/volume instruments, species property defaults and the E129 qualification.

**Do:** choose one dilute aqueous salt and a bounded temperature/concentration range. Obtain reviewed solution density or partial-volume data; solid density and water defaults are not substitutes. Specify mixture law, uncertainty and refusal/extrapolation behavior before fitting. Keep calibration points separate from holdout data.

**Freeze:** water blank, at least two concentrations, temperature/domain edges and independent holdouts; preserve extensive scaling and phase-scope behavior.

**Done:** numerical density/volume is defensible in the declared slice, with missing/outside-domain readings still qualified. This does not calibrate all dissolved ions or concentrated mixtures.

## DATA-03 — Extend optics by one independently supported solvent-specific slice

**Start:** `sparse_optics.rs`, `solution_optics.rs`, `surface_colour.rs`, sparse-iodine and optical quantization controls.

**Do:** choose one chromophore/species and solvent/medium. Review wavelength, extinction basis, concentration/temperature range and speciation conditions. Keep the current acidic aqueous iodine datum separate from plain water, polyiodide and organic extraction. Promote only the measured slice; do not invent a full spectrum from appearance.

**Freeze:** blank, supported concentration/path-length scaling, wrong-medium and out-of-range refusal, genuine zero versus missing spectrum, conflicting measurements and subnormal representability.

**Done:** one provenance-bearing numeric optical domain is usable with consistent coverage. Qualitative appearance remains labeled separately.

## ENG-01 — Certify one reaction-energy path and its uncertainty

**Start:** thermal/state acceptance in `bench.rs`, `delta.rs`, `solve.rs`, registry enthalpy coverage and preserved thermal/safety suites.

**Do:** choose one reaction with independently reviewed thermochemical inputs. Define system boundary, sensible/latent/reaction contributions, uncertainty propagation and imposed-temperature work. Do not equate an unavailable contribution with zero or use elemental closure as an energy certificate.

**Freeze:** closed adiabatic budget, thermostatted counterpart, transfer of uncertain heat, missing enthalpy, phase changes and veto after final equilibrium. Use independent references rather than app-generated temperature snapshots as the oracle.

**Done:** the selected path has a complete bounded energy/coverage contract and uncertainty survives transfer/save/reload. Other reactions remain explicitly uncalibrated.

## ENG-02 — Define work and exchange for imposed headspace changes

**Start:** seal/reseal/vent operators, `Headspace` in `vessel.rs`, gas/pressure event and thermal accounting.

**Do:** specify what the user-imposed geometric change means physically: displaced gas, pressure control, external work and heat boundary. If only a geometric control is supported, state that rather than claiming a reversible compression. Keep open, sealed and reservoir ledgers distinct.

**Freeze:** identical inventories with different imposed volumes, closed/open transitions, gas loss, pressure/temperature effects within the chosen model, unsupported work protocol and atomic refusal.

**Done:** observations and accounting consistently describe one explicit boundary protocol; no implicit thermodynamic reversibility claim.

## KIN-01 — Validate one promised rate domain with independent evidence

**Start:** `kinetics.rs`, `kinetics_integrator.rs`, electrochemical mixed-potential/overpotential routes, the original metal/acid and peroxide qualifications.

**Do:** pick one claimed route, such as a bounded metal/acid surface condition or one reviewed peroxide network. Identify necessary area, surface history/passivation, concentration, temperature and time inputs. Review independent rate data and declare applicability before implementing/fitting. Missing rate information must not appear as computed inertness.

**Freeze:** reacting and genuinely suppressed controls, amount versus time variation, surface area, temperature, time partition where justified and independent holdout trajectories. Equilibrium possibility is not a rate oracle.

**Done:** one bounded rate claim has provenance, input requirements and holdout evidence. Do not infer general catalyst, nucleation or lesson-timescale support.

## ELEC-01 — Extend steady migration to a bounded transient layer

**Start:** [transport continuation](../tools/chemistry-audit/CONTINUATION.md), `TransientDiffusionLayer`, steady reactive Nernst–Planck/interfacial current contracts.

**Do:** add migration to one transient layer using reviewed diffusivity/mobility, one shared physical layer thickness, explicit boundary flux and charge coupling. Preserve the diffusion-only zero-potential limit. Unsupported mobility/networks refuse; do not silently reuse the steady answer.

**Freeze:** zero-current relaxation, charged pulse, time-partition/convergence, steady limiting behavior, incompatible transport data, concentration depletion and atomic refusal.

**Done:** supported transient conservation/current/charge contracts pass without loosening established tolerances. Hysteresis, films and dynamic bubbles require separately identified state laws and are excluded from this first slice.

## TXN-01 — Expand failure injection across committing operator routes

**Start:** `StateDelta`, `SolverStack`, bench proposals, final-safety hooks, stock draws, spill/extraction/titration/still regression suites.

**Do:** enumerate committing routes and speculative stages. Build a bounded matrix of open/sealed/reservoir state, phase/interface owner, solver outcome and safety disposition. Inject failure after preparation, solve, final validation, safety and stock debit. Avoid duplicating tests that merely mirror implementation.

**Freeze:** complete before/after inventory, stock, pressure, energy/uncertainty, derived metadata, route history and accepted events. Earlier accepted titration increments may remain; the refused increment and its trial events must not leak. Deliberate disposal and accidental-spill policy stay distinct.

**Done:** the chosen operator tranche commits full state/events together or discards them together. CLI abort output alone is insufficient evidence of hidden rollback; use direct state controls and report uncovered routes.

## UX-01 — Make chemical claims agree across phase, locale and register

**Start:** operator capability descriptions, event narration, observation DTOs, CLI text/JSON, web summaries and localization lint.

**Do:** select transfer/separation and observable claims first. Assert product identity, amount, phase scope, operation disposition and uncertainty consistently across registers/locales. Structural placeholder lint does not detect chemical contradictions. Preserve the corrected Cu(II) narration and actual remaining-copper statement.

**Freeze:** valid separation, visibly layered but unsupported partition, unmodeled spectrum, incomplete heat and refused operation. Each prose claim must be backed by the committed event/state and coverage status; no locale may strengthen support.

**Done:** the selected claim matrix is consistent, with explicit unsupported mixtures/emulsions and useful supported outputs. Do not add implementation details to learner flows.

## EXP-01 — Freeze and execute the next independently authored fifty

**Start:** CLI syntax in `PROTOCOL.md` and the command examples in `README.md` only; the immutable independent audit harnesses after forecasts are frozen. Do not read existing corpus outcomes or production chemistry code before the freeze.

**Do:** independently invent 50 protocols/questions without reading existing corpus outcomes or implementation first. Declare expected observables, units, boundaries, applicability, positive/negative controls and independently justified bounds. Freeze scripts/predictions before execution. Only then inspect failed behavior/source and propose general repairs.

**Validate:** exact CLI text and JSON invocations, executable/source hashes, complete raw output, observation presence, finite values and all forecasts. Classify invalid author protocols separately; corrected followups get new IDs and freezes. Include sequence, scale, persistence, interface and boundary variants without assuming irreversible operations commute.

**Done:** all fifty originals are assessed, failures retained and minimized, gaps mapped to general lanes. Execution success or expected refusal is not completion of an originally successful physical forecast. Old fleets remain immutable holdouts.

## PERF-01 — Measure release behavior, then optimize one verified hot path

**Start:** `audits/independent-next-50-20261004/profile.py`, existing profile receipts, `OPTIMIZATION.md` and the accepted source.

**Do:** build exact release baseline/candidate artifacts with recorded dependencies, feature flags and runner conditions. Separate cold startup, warm in-process work, native solver calls and WASM/browser workload. Use blocking process waits and a process-group watchdog; the superseded polling profile contaminated parent wall time. Alternate runs and choose enough repetitions to characterize variance before selecting one optimization.

**Freeze:** correctness guard, workload/input hashes, timeout handling, wall/CPU/RSS/call-count measurements and intended statistical claim. Chemistry outputs remain unchanged unless the selected existing task explicitly permits a separately justified numerical difference.

**Done:** one hot-path change has reproducible before/after evidence and all relevant unchanged scientific contracts. Report regressions and variance; debug-process medians are not a release/browser speedup.

## How to report a completed lane

Record lane ID, exact source and prerequisite revision, frozen contracts, baseline failures, implemented behavior, acceptance commands/run links, executable identity, scientific qualifications, unsupported scope and follow-on lane IDs. Update the checkpoint and the existing roadmap/history entry without renumbering older tasks. Public reports use repository-relative paths and public evidence links only; research mappings, host configuration and artifact-storage details stay private.

## Latest bounded transaction and persistence evidence

Direct-stack and gas-boundary paired diagnostics are reviewed: [13/13 stack controls](../audits/solver-stack-atomic-contracts-20261008/README.md) and [12/12 boundary controls](../audits/solver-boundary-contracts-20261008/README.md), with 604 inherited core library tests passing on both sources in each comparison. PR #768 still needs final main-based integration acceptance for all 42 integration functions. Stacked PRs also trigger CI; any earlier contrary note is superseded.

The next independent port is [PR #769](https://github.com/CrispStrobe/kerotakis/pull/769): canonical JSON keys for owned nuclide inventories, preserving individual isotope objects and rejecting invalid or duplicate inventory entries. Eleven unchanged historical controls were ported before production edits. Consume paired run 37797436300, verify source/fixture/generated-lock bindings and inherited controls, then require exact-head PR gates. Do not count this narrow persistence port as runtime nuclide accounting or full-audit acceptance.

PR #767 has merged as `158a29f5` after all five required checks passed on tested head `bce5eb22`. PR #768 now targets main at `6ff42221`; require its fresh matching-head gates and all 42 integration functions. PR #769 includes merged main at `50088c4e`; its pinned paired diagnostic still compares original baseline `0028f3a7` with production repair `b42732c1`, separately from final-head integration.

The [nuclide persistence diagnostic](../audits/nuclide-inventory-serde-20261008/README.md) is reviewed: baseline 7/11, repair 11/11, inherited 604/604 on both. Required final-head checks for #769 remain pending. [PR #770](https://github.com/CrispStrobe/kerotakis/pull/770) begins the strict NUM-03 policy port on top of #768 with 30 unchanged historical controls and four new policy controls. Its APIs are introduced here; require compilation and all 34 controls rather than treating missing baseline APIs as behavioral failures. Retarget after #768 merges and require final-head gates. Then port the original supplementary owner/gas controls with their dependencies, preserving incomplete-owner refusals before admitting any new ownership model.

The strict-policy supplement now includes matched tolerance-precedence controls, preserving supported acceptance as well as refusal. The original historical creation test alone could not prove that a tighter route setting was retained. [The new manifest](../audits/required-conservation-port-20261008/policy-controls.json) binds the 12 direct/MIX boundary cases and the supplementary source hash. Require all 34 strict-policy functions on the final integrated head; no hosted acceptance is claimed yet.

A source review found shared solver acceptance bypassing parts of the canonical electrode schema. PR #770 now delegates to `ElectrodeState::validate` and rejects duplicate electrode labels; [the separate eight-function schema lane](../audits/solver-electrode-schema-20261008/README.md) has its own pre-production freeze, baseline and pending paired run. Consume run 37805353920 before accepting this repair, investigate inherited failures in their source slice, and require all 84 policy/schema/transaction functions plus final-head gates. No new model coverage is implied.
