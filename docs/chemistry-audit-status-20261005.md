# Chemistry audit: current state, 2026-10-05

Start here, then choose a task from [the next-step lanes](chemistry-audit-next-lanes.md). This checkpoint covers the independent chemistry audits and their general repairs. Existing product tasks retain their IDs in [PLAN.md](../PLAN.md), [OPTIMIZATION.md](../OPTIMIZATION.md), [ROADMAP-GUI.md](../ROADMAP-GUI.md) and [ROADMAP-Webapp.md](../ROADMAP-Webapp.md).

## Which revision this describes

The accepted engine source is [`8cdab1f6ecd0b1132c4008014a5a0c5476429d17`](https://github.com/CrispStrobe/kerotakis/commit/8cdab1f6ecd0b1132c4008014a5a0c5476429d17). Reports and acceptance receipts were published through [`9331f9e9`](https://github.com/CrispStrobe/kerotakis/commit/9331f9e974ff460a91a1617883ae22e1d319e4d1) on [`audit/systematic-chemistry-20261002`](https://github.com/CrispStrobe/kerotakis/tree/audit/systematic-chemistry-20261002).

At implementation checkpoint `9331f9e9`, that branch was 168 commits ahead of `main` at [`f7d45f6f`](https://github.com/CrispStrobe/kerotakis/commit/f7d45f6fefacc0a53312356f8b801f7e371b8042), with no commits behind it. Subsequent documentation commits do not change that accepted engine source. **Branch-verified does not mean merged into main, released, or deployed.** A documentation-only cherry-pick also does not port the implementation. Recheck ancestry before starting integration lane AUD-00.

Documentation delivery [PR #755](https://github.com/CrispStrobe/kerotakis/pull/755) merged on 2026-10-07. [The integration manifest](chemistry-audit-integration-20261007.md) tracks implementation ports separately; AUD-00 remains open.

## Integration update, 2026-10-08

Amount foundation [PR #756](https://github.com/CrispStrobe/kerotakis/pull/756)
merged to main as `b31455cc`. Main-based stock ownership
[PR #757](https://github.com/CrispStrobe/kerotakis/pull/757) remains pending.
Thermodynamic kernel [PR #758](https://github.com/CrispStrobe/kerotakis/pull/758) at `45c44a4a` is also pending, with six accepted source files and 58 preserved integration controls. This is partial AUD-00 delivery, not integration of the entire accepted engine.

The native 33-control pre-repair
[baseline 37728961663](https://github.com/CrispStrobe/kerotakis/actions/runs/37728961663)
executed with **13 passing and 20 failing** controls. A nine-control refinement
[baseline 37729480945](https://github.com/CrispStrobe/kerotakis/actions/runs/37729480945)
and repair validation
[37729728076](https://github.com/CrispStrobe/kerotakis/actions/runs/37729728076)
at `e7eaf03b` remain pending. AUD-01/02 are open; new full engine acceptance is
held until the focused results arrive. The
[integration manifest](chemistry-audit-integration-20261007.md) identifies the
repair branch and explicit original raw-harness protocol adaptation.

## Accepted evidence

| Evidence | Exact result | Public record |
| --- | --- | --- |
| Full source audit | 1,865 Rust test executions; 17 successful native/WASM/CLI stages; 198 source bindings | [Run 37241135174](https://github.com/CrispStrobe/kerotakis/actions/runs/37241135174) |
| Artifact integrity | 432 checks binding source, logs, dependency lock, executable and control captures | [Integrity receipt](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/accepted/full-integrity.json) |
| Third fifty replay | 1,065 checks; nine scientific/protocol qualifications retained | [Run 37242349555](https://github.com/CrispStrobe/kerotakis/actions/runs/37242349555) |
| Fourth fifty replay | 1,148 checks; 124 text/JSON captures for 50 originals and 12 separate followups | [Run 37242514682](https://github.com/CrispStrobe/kerotakis/actions/runs/37242514682) |
| Latest repair baseline | 15 core failures and four native readback failures demonstrated before repair; 33 new frozen boundary tests pass afterward | [Crystal report](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/RESULTS.md) |

The accepted executable SHA-256 is `67797d56ced1531f44dce7252c272b34d19183a4d4571a076b9dcf67d3a634cb`. [Full validation](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/accepted/full-validation.json) and [the binding index](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/accepted/binding-check.json) preserve the exact identities. CI artifacts may expire; checked-in receipts remain public, but receipts alone cannot recreate absent raw output. Use a new revision-bound hosted run when raw evidence is unavailable.

These are selected audit gates, not a claim that every workspace test, platform, browser or release gate passed on this revision. Rust test executions include repeated feature-mode tests. New boundary tests are source-informed controls, not additional blind experiments.

## What the branch repairs

- Transfer preparation certifies donor and receiver proposals; relevant mechanical operations screen contextual prospective state and commit accepted proposals. Spill creation/recovery has explicit ownership and rollback contracts.
- Extraction uses sequential fresh-solvent contacts at actual contact temperatures, screens each stage and commits atomically. Bounded coefficient/saturation coverage remains explicit.
- Accepted final equilibrium states have an opt-in safety hook. Titration uses that hook inside virtual dosing, discards refused trials and preserves earlier accepted increments and stock accounting.
- `SolverStack::with_required_conservation` provides opt-in represented element/formal-charge closure. Compensated inventory differences precede element totals, so unchanged background cannot conceal a trace change. Unsupported owners and unsafe bounded arithmetic refuse.
- The strict certificate now admits the existing typed CaCO3/SrCO3 crystal owner. General transaction checks reject malformed/ambiguous crystals and overflowing aggregate quantities. Raw native crystal components refuse negative/nonfinite values before correction and identify their selected column.
- Zero-work operations skip solver/safety processing; finite zero/scientific-notation wait and charge inputs have explicit syntax contracts. Nuclide ledgers have canonical persisted isotope identities and state validation.
- `Amount` provides bounded two-component arithmetic. Opt-in stock owns it authoritatively and persists its mode; vessel/still quantities are not thereby migrated.
- Sparse iodine optics exposes one reviewed, condition-specific aqueous datum with explicit scope. Copper displacement narration describes the supported pathway and actual remaining copper.
- Distillation, native trace serialization, phase ownership, donor/receiver accounting and refused-operation rollback have expanded preserved controls. Explicit refusal remains distinct from completing a physical forecast.

The [preceding systematic report](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/post-equilibrium-contracts-20261004/RESULTS.md) and [audit index](../tools/chemistry-audit/README.md) explain the earlier stages.

## What remains open

| Gap | Next lanes | Boundary to preserve |
| --- | --- | --- |
| Unbounded native Ca/Sr/C correction, phase overdraw and unsupported reporting normalization | AUD-01, AUD-02 | Corrected conservation is not independent evidence of accurate raw output |
| Authoritative compensated vessel/still ownership, public API, saves and solver reconstruction | NUM-01–NUM-04 | Stock adoption and strict ledger arithmetic do not migrate physical inventories |
| Surface/exchanger and incomplete material owners in the strict certificate | OWN-01, OWN-02 | No generic formula fallback or blanket admission of partially represented matter |
| Native trace/oxidation-state thresholds | VAL-01 | Numerical noise and real trace matter need different evidence |
| Observable support, missing identity/data, density, spectra and reaction heat | OBS-01, DATA-01–DATA-03, ENG-01 | Computed zero, unavailable data and uncalibrated estimates must stay distinguishable |
| Imposed headspace work, kinetics and transient electrochemical migration | ENG-02, KIN-01, ELEC-01 | Equilibrium and steady transport do not establish these capabilities |
| Complete transaction/presentation coverage and independent holdouts | TXN-01, UX-01, EXP-01 | Passing isolated examples does not close every operator path |
| Release performance attribution | PERF-01 | No current speedup or release-equivalence claim |

Strict molecular closure does not certify aqueous speciation, reaction energy, kinetics or nuclear closure. Typed multi-crystal bookkeeping does not extend the native adapter beyond one mixed phase in its existing dilute inorganic wateq4f domain. Generic unregistered bulk SrCO3 remains refused.

Four exact legacy exclusions remain disclosed in the full validator: pure-water extraction without extractable solute; the zero-transfer screen-call observer; and two extreme-ratio Drain forecasts in the modeled single-phase domain. Original tests and supported replacement controls remain preserved. The third replay retains nine qualifications. Fourth-original classifications remain 21 bounded agreements, seven qualified agreements, 13 expected invalid-input refusals and nine author protocol errors; followups retain two author errors, three bounded agreements and seven qualified agreements.

The earlier corrected representative profile covered ten workloads, 20 warmups and 80 timed debug-process runs at an earlier source. Median wall changes ranged from about 0.6% faster to 2.5% slower. Four repetitions, debug startup and those source identities do not establish a release speedup. See [run 37219758069](https://github.com/CrispStrobe/kerotakis/actions/runs/37219758069).

## Pickup order

1. Read this checkpoint, the chosen lane and its linked contracts. Establish the actual branch and implementation revision.
2. Start AUD-00 for integration. AUD-01, NUM-01 and OBS-01 designs can proceed independently against the accepted source; implementation ports must respect their dependencies.
3. Freeze new expectations and positive/refusal controls before production changes. Preserve failed evidence and immutable original experiments.
4. Finish one bounded lane with source-bound validation and a reviewable PR. Record source, executable, checks, retained qualifications and remaining scope in its report.
5. Update this checkpoint when accepted behavior changes. Environment, artifact-storage locations and machine resource decisions belong in private operating notes, not public task descriptions.
