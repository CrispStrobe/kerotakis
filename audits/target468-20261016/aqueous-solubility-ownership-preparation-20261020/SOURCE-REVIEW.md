# SE009 source review: phase-specific drain ownership —2026-10-20

## Actual cause and disproved first hypothesis

The original lead was that aqueous solubility was being applied to dry hexane after draining water. **The saved response instead demonstrates a wrong-phase withdrawal in the drain operator.** Existing dissolved silica is copied to the receiver while the same number of moles is withdrawn from the source's solid silica. The source's aqueous silica remains, so it appears that more dissolved after the drain. No new dissolution event occurs.

The evidence response binds served source `dacf76513328c2226e2f5133a79fb9af84343431`, binary `f6aef37ddadfa0f1163e2aff9f436ec560eae767d6d86203bfa850c04f0c9955`, producer lock `41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb`, unchanged original SE009 script SHA256 `e1a9c24c63fc3c3eca32e6ebf76e57e5c7815e95f062df019ce0efb134a2c8cc`, and response SHA256 `b6f571b110c035c9a5dcf84ff5ad891d21d5c863f665b8823235ebbfb1ef2d13`. The source audit uses refreshed actual main `2e41d17addbabc1a64bd3a34a2a81cb4feb139ff`. Its entire `Operator::Drain` arm is byte-identical to the served source, SHA256 `30a19063586e5aba48c35a32446cb1bff9d737387e88f856c06b7f0ef2477018`.

The response's `observations` array uses these zero-based coordinates:

| Coordinate | Source solidSiO2 mol | Source aqueousSiO2 mol | Receiver aqueousSiO2 mol | Source water mol | Source hexane mol |
| --- | --- | --- | --- | --- | --- |
|3: after original silica addition |0.016623393915185407 |0.000019972039145196722 |receiver absent |5.534276991396059 |0.15331058970967068 |
|4: after receiver creation |same |same |0 |same |same |
|5: after `drain v1 v2` |0.01660342187604021 |0.000019972039145196722 |0.000019972039145196722 |0 |0.15331058970967068 |
|7: after `filter v1 v3` |0.01660342187604021 |0 |v2 still has original aqueous share |0 |0 |

At coordinate5, solid loss is exactly the pre-existing aqueous amount. Source aqueous amount does not increase. Total species amount remains conserved; phase identity and subsequent separation are wrong. Filtering then sends the stranded aqueous-tagged silica out with dry hexane into v3. The original three-fraction separation is therefore not qualified by total silicon conservation alone.

Coordinate5's events include `drained`, a source `inert` silica sentence and receiver aqueous-route/characterization events. **There is no new `dissolved` event.** Source v1 has `solution:null` after draining; the PHREEQC route announcement belongs to water-containing receiver v2. Do not attribute a receiver's route to the dry source.

## Responsible source contract

In actual main:

- `crates/kerotakis-core/src/bench.rs`, `Bench::apply`, `Operator::Drain` arm starts near3505. Planning preserves each moved portion as `(SpeciesId, Moles, Phase)`. Its comment explicitly promises that solids stay behind.
- Near3653, commit discards the planned phase with `for (spec, m, _) in &moved { src.withdraw(spec, *m); }`.
- `crates/kerotakis-core/src/vessel.rs:1768`, `Vessel::withdraw`, explicitly withdraws across **any** phase in storage order. In this response, solidSiO2 is stored before aqueousSiO2, so the wrong source portion pays the transfer.
- `Vessel::withdraw_phase` at1792 already restricts the actual contents debit to a named phase. Use the phase retained by the drain plan at this caller. Preserve the any-phase method for callers intentionally consuming a whole species pool.

Reviewed actual-main blob SHA256 values:

| Repository path | SHA256 |
| --- | --- |
|`crates/kerotakis-core/src/bench.rs` |`7e7aeb09486764401ce8e5432c1e88c1fcd28c85db3ad083bde5176a522808ec` |
|`crates/kerotakis-core/src/vessel.rs` |`2be6a351110a65eff68cc51bc320530d2bfdb07d0e88bddc43b998fc13cfeaba` |
|`crates/kerotakis-core/src/solve.rs` |`653d655cefd81e2100033d037779cf02def5fd566c44d8e866bec699505a8e20` |

No dissolution curve, solubility value, UNIFAC coefficient, original command or forecast needs changing to repair this demonstrated caller defect.

## Why a solvent-domain patch would miss this failure

`crates/kerotakis-core/src/solve.rs:1205`, `saturation_moves`, already sums only liquid water and returns no moves when water is absent. It computes capacity from that water's own volume, not vessel-wide liquid volume. This admits real minority water layers without making hexane count as water.

`crates/kerotakis-phreeqc/src/aqueous.rs:1827`, `partition`, similarly accumulates `kgw` only for the derived solvent role in `Phase::Liquid` near1954 and checks for positive water near2096. The dry source does not acquire an aqueous characterization in the saved response.

Keep pure-organic and minority-water controls as positive regression/domain controls. Do not invent organic silica/salt solubility, infer zero physical solubility from an unsupported question, or introduce a blanket aqueous-majority rule. An organic-rich vessel can still contain a genuine minority water phase. A separate any-liquid solution-catalyst admission in `MixingEquilibrator` exists but is not the writer of SE009's silica: silica is not a declared solution catalyst.

## Frozen proposed baseline and repaired controls —not executed here

These controls are source-grounded forecasts, not claimed test results. Freeze their actual tests-only baseline source separately before bounded validation. For the operator-isolation cases, use an empty solver stack and sealed headspace so equilibrium/evaporation cannot conceal or create the phase debit under test. Use real `Bench::step(Operator::Drain)` and owned contents, not self-assigned result fields.

1. **Solid-first/aqueous-second silica drain, direct reproduction.** Build the same water/hexane layers and source ordering shown above, with0.016623393915185407 mol solidSiO2 and0.000019972039145196722 mol aqueousSiO2. Drain once. Require source solid unchanged, source aqueous exhausted, source hexane unchanged, all liquid water and original aqueous silica in the receiver, and phase/species totals conserved. The storage-order-independent answer must survive reversing silica portion insertion. Existing any-phase withdrawal predicts a genuine failure in the solid-first variant. Do not require zero total silica in the aqueous receiver: its legitimate original dissolved trace belongs there.
2. **Gas-first/aqueous-second same neutral species.** Use a supported neutral solute such as ethanol with a gas portion inserted before a distinct aqueous portion, plus genuine water/hexane layers. Isolate the operator as above. Require gas inventory unchanged, phase-qualified aqueous transfer matching the plan and species totals conserved. Reverse gas/aqueous insertion to prove order independence. Independently retain the supported distribution coefficient's domain; this test concerns who pays the planned transfer, not a new measured coefficient. Existing any-phase withdrawal predicts a genuine gas-first failure.
3. **Ordinary supported layer transfer positive.** Water/hexane with a supported dissolved ethanol dose and complete dissolvedNa+/Cl−, no competing solid/gas portions of those species. Preserve the accepted partition fractions, water transfer, ionic transfer, receiver temperatures and all inventories. This should already pass and prevents a wrong fix that refuses every mixture or skips known solutes.
4. **Water-budget/domain controls, already-positive source expectations.** Call the actual saturation planner with dry hexane+solidSiO2, dry hexane+solidNaCl, frozen/gas water without liquid water, and plain water+supported soluble nonelectrolyte. Dry/no-liquid-water controls produce no **aqueous** saturation move; water positive remains supported. Add identical liquid-water inventories with and without a larger hexane layer and require identical aqueous solubility capacity, including a genuine minority-water configuration. These establish what was already correct; do not mislabel them as baseline failures.
5. **Real original beneficiary after changed delivery.** Execute unchanged original SE009 once on the delivered changed artifact and independently review all three physical fractions, per-vessel/per-step phase inventory and every original criterion. Retain its previous failed served response. No unchanged broad-corpus replay is needed.

## Adjacent extraction and direct/delta parity —separate scope if needed

`Operator::Extract` near3772 collects only `Phase::Aqueous` and `Phase::Solid` into its solute pool. A supported neutral solute stored as `Phase::Liquid`, such as freshly added ethanol, can be omitted. Freeze separate liquid-tagged and aqueous-tagged ethanol controls before extending this admission. A positive complete-ion control must remain outside neutral UNIFAC inference.

The extraction commit near4058 also calls `src.withdraw(&split.species, extracted+remaining)` across all phases before redepositing the remaining aqueous share. A gas-first same-species portion can therefore pay for an admitted aqueous/solid extraction pool. Preserve crystalline-reservoir policy: the helper already refuses a solid inventory exceeding water plus extraction-stage capacity. Do not “repair” this by removing legitimate solid participation or declaring that missing guard nonexistent.

## Same phase-debit defect in direct mixing writers

Three direct `MixingEquilibrator::equilibrate` writers use any-phase withdrawal for a move explicitly planned from solids:

| Direct writer in `solve.rs` | Planned source | Existing delta counterpart |
| --- | --- | --- |
|Solution-catalyst promotion near740 |Collected `Phase::Solid` catalyst portions |Near861: negativeSolid and positiveAqueous deltas |
|Iodide-assisted iodine near767 |`starch_iodine::iodine_to_dissolve` bounds its amount by solidI2 |Near873: negativeSolid and positiveAqueous deltas |
|`SaturationMove::Dissolve` near782 |Planner bounds by solid inventory and remaining aqueous capacity |Near887: negativeSolid and positiveAqueous deltas |

The narrow repair may use the existing `withdraw_phase(...,Phase::Solid)` at these three callers alongside the drain caller. Preserve all existing admission conditions, quantities, curves, iodide assistance and catalyst classification. A more restrictive water-domain rule would answer a different question and must not be slipped into this patch.

Freeze these additional isolated source-grounded controls before baseline execution:

- **Catalyst writer:** real liquid water, aqueousKI inserted first0.001 mol, solidKI inserted second0.002 mol. After direct promotion require0.003 mol aqueousKI, no solidKI, unchanged totalKI. KI has no reviewed aqueous-solubility row, so the ordinary saturation branch cannot hide the catalyst writer. Compare a solid-first positive and the equivalent actual delta application; preserve source registry identity and current catalytic admission.
- **Saturation writer:** real water, aqueousSiO2 inserted at exactly half the source's reviewed finite capacity, solidSiO2 excess inserted second. Require aqueous amount reaches that unchanged capacity, solid falls by exactly the newly dissolved share, all species/phases close. Reverse portion insertion for the ordinary positive. Compare direct and delta contents for this one isolated move, not just emitted event amounts.
- **Iodide-assistance writer:** real water, aqueousI2 inserted at exactly the unchanged reviewed pure-water capacity, a smaller solidI2 dose inserted second, and excess aqueousKI. The direct iodide-assistance branch should consume the intended solidI2 and increase aqueousI2 by that amount; allKI stays. The initial ordinary saturation headroom is zero, isolating the actual writer. Reverse I2 insertion for a positive; compare isolated direct and delta inventories. The model may report the resulting supersaturation: do not assert that the iodide-supported solution equals a pure-water saturation prediction or suppress an existing boundary.

No baseline test has been executed by this review. The expected aqueous-first failures follow directly from any-phase storage-order debit, just as the real served drain failure does.

### Separate delta-planning overlap lead

The delta method computes catalyst promotion, iodide assistance and ordinary saturation against the same original vessel, whereas the direct method sees prior mutations. A naive iodine fixture *below* ordinary aqueous capacity can therefore let both iodine assistance and saturation independently spend the same solid inventory. This is a distinct source-inspection lead, not an executed failure here. Preserve the original per-phase ownership and finite-budget question; freeze a separate overlap/negative-delta control before addressing it. Do not claim universal direct/delta event or multi-route parity solely from the phase-specific debit repair. The isolated fixtures above each leave only one writer's amount active and legitimately test that writer without rewriting any chemistry coefficient.

## Lot metadata: do not silently change its meaning

`Vessel::withdraw_phase`'s lot loop does not debit or mutate lot amounts. It examines `remaining` after the contents loop and merely breaks; when the requested contents were found, `remaining` is already zero. `Vessel::withdraw` also leaves lot amounts unchanged. Therefore switching the drain caller to its phase-specific counterpart changes actual contents ownership without introducing a new lot-debit behavior.

This is not automatically an instruction to make all lots match remaining moles. `MaterialLot` documentation near1154 explicitly describes addition provenance independent of how solvers resolve species, `moles` as “How much was added,” and `phase` as the intended phase. Consumers also use lots for recipe identity, activation age, hydration and particle metadata: `material::named_objects` checks current species presence, while chemiluminescence and enzyme hydration read lot times/doses. Blindly decrementing all historical lots would change these meanings and can break deliberately retained provenance.

A separate owned-provenance transfer contract is warranted: distinguish immutable addition history from metadata attached to remaining/transferred portions; preserve source age/hydration/recipe identity and particle distributions in receivers; cap any physical effect by actual owned dose; cover full/partial transfer, clear/replay, multi-phase same-species and direct/delta behavior. Removing the inert lot loop is cleanup, not proof this larger contract works. The narrow drain fix should not claim that it closes this lane.

No runtime source, original evidence, catalog entry or provider execution was changed by this review.
