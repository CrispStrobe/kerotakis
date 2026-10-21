# Remaining phase-energy lane: conservative refusal at the brine cap

**Validation update —2026-10-21 21:08UTC:** PR857 remains at reviewed28277d41 with actual38/38 focused controls accepted. Existing CI39671380308 and chemistry39671380305 are progressing; preserve completed groups instead of issuing the prepared full/component duplicates. The actual chemistry checkout is merge575e9c3c with exactly the candidate tree31a22af9. Its retained native binary isffa41cc8 and actual generated lock41bbdc74, distinct from the earlier focused lock66886813. Full raw fleet/platform acceptance is still pending; no merge or new delivery is claimed. Only the separate two no-engine peroxide-hook controls are currently identified as missing from ordinary CI. A newly independently checked [liquidus/fusion consistency lane](liquidus-thermodynamic-consistency-lane-20261021.md) follows accepted cap delivery: the source's4%/0.1K comment is unsupported by its own phase curves. Its below-cap hypothetical root is not a physical prediction. Qualified coverage stays335/468.


**Current work —2026-10-21 20:46UTC:** [PR857](https://github.com/CrispStrobe/kerotakis/pull/857) candidate28277d41 now has [actual38/38 focused acceptance](../audits/target468-20261016/phase-cap12-focused-accepted-20261021/README.md), authenticated by two parent saved reviews and an independent third review. Frozen12, uncapped8 and the full570-point historical sweep pass; first baseline8/4 and first fixed35/3 remain preserved. The generic cap refusal retains already delivered cooling before mutation and reuses localized route reasons. Current11 classifier tests,139-file encoding lint and localization surface lint pass. Full source/platform/product acceptance remains pending, so PR857 is draft. The current282 full recipe is prepared; the supersededa64 full package is ineligible. Coverage remains335/468, with no synthetic-control catalog promotion. Do not repeat the completed baseline or focused run.

**Current execution lane20:38UTC:** optical57edd/native74ba/lock41bb API11/browser82 acceptance is complete. The [actual unchanged-runtime twelve-control baseline](../audits/target468-20261016/phase-cap12-baseline-outcomes-20261021/README.md) completed with eight passes/four failures, matching the frozen forecast. All four outside-cap cases first fail complete entering-state equality before later energy/repetition assertions. Exact/interior positives pass. Two actual parent saved reviews and an independent saved review authenticate the full manifest, actual generated lock/executable and exact version1 issue. The first collector SDK environment failure and successful same-version correction are preserved; no producer reissue occurred.

A narrow candidate now checks the capped residual before mutation, with absolute polynomial-term conditioning rather than net-energy rounding alone. Frozen12 inputs remain byte-identical. Historical three temperature-only cap contracts are being adjudicated to preserve entering state on refusal; the full570-point sweep must distinguish direct refusal from valid exact-cap retained-brine settlement. Source review identified that distinction and stale comments for correction. Focused validation, full own-fleet/platform and changed-release acceptance remain pending. Keep strict0.001J/1e−10mol contracts and original eight uncapped/TC008 controls. No extra mixing-energy sink or beyond-cap equilibrium claim is authorized.


The first fixed38 focus on a64 actually passed35/failed3. Frozen12 and uncapped8 all pass; three historical tests still incorrectly required positive ice although their entering fixture has none. Their first raw failures remain frozen. The source-only correction on28277d41 requires exact entering liquid/ice for refusal, retaining positive ice for supported freezing, all physical/event checks and the full570-point sweep. The [necessary new38-control run](../audits/target468-20261016/phase-cap12-ice-inventory-followup-issued-20261021/README.md) is issued; outcomes remain pending. Do not activate the ineligible a64 full preparation or treat a saved-reader exit0 as all38 passing.

## Demonstrated source problem

In `crates/kerotakis-core/src/solve.rs`, the `StateEquilibrator` freezing branch caps pure-ice transfer at `maximum_freezing`. Its `reached_boundary` branch then forces the residual brine to the cap liquidus. If the conserved-energy root lies beyond the cap, this creates energy. Let `xc` be the capped transfer and `Tc` its liquidus. The positive discrepancy is:

`Rcap = initial_vessel.energy_between(now, Tc) - xc * water_fusion_enthalpy_at(Tc)`.

The earlier `Bench::apply` cooling operator already removed the reported `EnergyTransferred.delivered_j`; there is no outstanding coolant reservation to discard. Its energy is owned by the entering vessel at `now`. `excess_enthalpy_j` is a mixing-energy anchor and must not become an unrelated thermal sink.

## Recommended implementation

After the uncapped repair passes, create a narrow branch from refreshed main. Freeze independent cap-energy forecasts and real first-failure evidence before changing behavior. When the energy root would require transfer strictly beyond the model cap, emit the existing route-specific model-boundary refusal **before changing phase amounts or temperature**. Preserve the entering physical state and its already delivered cooling. Do not claim a computed cap equilibrium. A genuine root exactly at the boundary may still commit a conserved boundary state; numerical distinction needs a justified floating-point/integration bound and must retain the independent0.001 J acceptance contract.

Do not infer unknown salt precipitation, hydrates, eutectic phases or an unmodeled colder metastable phase split. Limiting the cooling operator's delivered energy instead would require consistent requested/delivered-event and temperature accounting across `Bench`; that is a broader alternative, not this narrow State refusal.

## Required acceptance

- Preserve the eight frozen uncapped controls, original TC008 strict ledger, real current chemistry ownership and existing absence-of-aqueous behavior.
- For a root just inside each eutectic/activity cap: actual transfer, correct own liquidus, closed energy/material ledger, no false refusal.
- For a genuine root exactly at a cap: closed energy before declaring a boundary state; no discarded remainder.
- For a required root beyond each cap: unchanged entering physical inventory, temperature and energy; explicit correct boundary refusal; no `StateChanged` event or claimed final cap equilibrium.
- Preserve already-outside-range cold refusal and quiet room-temperature behavior.
- Repeat refused settlement and verify no hidden energy/material drift.
- Execute a real `Bench` cooling command past each cap. Owned enthalpy change must equal actual delivered cooling; the refusal describes missing equilibrium coverage, not cooling that was supposedly never delivered.

Existing `crates/kerotakis-core/tests/states.rs` cases needing deliberate contract adjudication are `partial_freezing_stops_with_liquid_at_the_declared_model_boundary`, `the_ideal_route_stops_at_its_own_range_and_names_that_instead`, and `the_boundary_is_announced_only_when_the_brine_actually_reached_it`. Their beyond-cap arms currently require a boundary temperature without checking enthalpy. Preserve their historical source and first energy failures, then document the corrected refusal contract; do not keep energy creation to satisfy those geometric expectations or merely remove assertions. Preserve `a_solution_past_the_stated_range_is_refused_a_transition_rather_than_given_one`, the boundary-relation inversion checks, and the native saturated-brine domain behavior in `colligative_numbers.rs`.

Retain actual source, fixture, executable and dependency-lock bindings, frozen forecasts and raw outcomes. Follow focused causal validation with applicable full own-fleet, protected platform and changed-release acceptance. Unsupported beyond-cap equilibrium remains a coverage limit even after conservation/refusal is correct.
