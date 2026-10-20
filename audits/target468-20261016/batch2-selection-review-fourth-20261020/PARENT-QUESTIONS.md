# Additional parent review answers —2026-10-20

This addendum leaves the first reader, original assessment and every original experiment unchanged. Reproduce its separate checks with `python3 audits/target468-20261016/batch2-selection-review-fourth-20261020/review_addendum.py`; [raw addendum](parent-questions-addendum.json) binds all twelve original stdout/script hashes, the prior assessment, source registry and exact serializedVessel source. No engine or HTTP is invoked.

## AB034: actual solved phosphate, not only canonical bookkeeping

The following amounts are computed independently as **reported solved molality × actual solventkg**, with reported activities retained. These are views of the same phosphate already represented by the canonical inventory; they must not be added as extra owned material. Printed native species have finite precision, so their reconstructed sum is not asserted bit-identical to the conserved canonical1mmolP.

| Arm and actualpH | Dominant solved species | Solved amountmol | Reported activity |
| --- | --- | --- | --- |
|1equivalent,pH4.705253 |H2PO4- |0.000992792557340 |0.009007 |
| |HPO4-2 |0.00000433573929220 |0.00002899 |
| |H3PO4 |0.00000249594191545 |0.00002503 |
| |NaHPO4- |0.000000341135329315 |0.000003095 |
|2equivalents,pH9.312823 |HPO4-2 |0.000886852325268 |0.004801 |
| |NaHPO4- |0.000107116441446 |0.0009228 |
| |H2PO4- |0.0000042686999012 |0.00003678 |
| |PO4-3 |0.00000174438227271 |0.000004202 |
|2.5equivalents,pH11.396488 |HPO4-2 |0.000712141914960 |0.003596 |
| |NaHPO4- |0.000101734559280 |0.0008617 |
| |PO4-3 |0.000186114399624 |0.0003822 |
| |H2PO4- |0.0000000268100485632 |0.0000002272 |

This establishes predominantly dihydrogen phosphate at the first equivalence, hydrogen-phosphate free/Na-associated species at the second and substantial increasedPO4-3 in the highpH arm. It does **not** claim2.5equivalents produces a50:50freeHPO4/PO4 mixture; free hydroxide, activities and complexes remain explicit.

## PQ006: actual phase identities and contamination

The first retained solid is exclusively `Mg(OH)2`,0.0004827159238265mol. The second retained fraction contains `CaCO3`,0.0004978161940234mol, **and** `Mg(OH)2`,0.000006593969418134mol. The final liquid vessel includes trace reprecipitated `CaCO3`2.883210602827e-11mol and `Mg(OH)2`1.12910004979e-10mol alongside actual aqueousCa/Mg. The reader checks every solid identity and every Ca/Mg-bearing non-solid portion; no alternate mineral, adsorbed pool or hidden phase is substituted. The original separation remains preferential and contaminated, not quantitatively pure.

## QA006: every boundary export isCO2

All100 export events are explicitly `gas_evolved` with species `CO2`; there is no other exported species and no `gas_absorbed` event. Their sum is0.001999999999999996625mol. The addendum retains every event and row coordinate. This separately establishes the two carbons per consumed oxalic acid rather than merely summing all gases and naming the resultCO2. Actual retained inventories, titrant/carrier and every material/charge coordinate remain in the original complete reader.

## Every non-contents material reservoir in all twelve cases

The serialized bench has exactly one primary field, `vessels`. The addendum checks **every vessel snapshot**, not only final vessels:

- Explicit surfaces are empty. Unresolved materials, material objects, electrodes, exchange sites, adsorbed species, solid solutions, surface-colour spots and charred objects are empty. Optional soap-scum, marked-paper, surface-particle and emulsion objects are absent.
- Exact historicalVessel serde definitions are authenticated: these omitted vectors are omitted only when empty and optional objects only when `None`. The reader verifies those source attributes for each named field, rather than inferring absence from missing display text.
- Nuclide inventories are empty; every foam-state value and pendingCO2 transfer coordinate is zero. Resolved-state mirrors are `valid:false,solution:null`.
- Gas primary material lives in `contents`, including sealed-headspace air; `headspace` supplies boundary geometry, not a second gas amount. Solution/species/freeH/OH fields are resolved diagnostics of canonical material and not a second inventory. MaterialLots describe addition provenance/metadata and are not separately owned moles to add again. Transient step-start/heat ownership bookkeeping is not another physical material reservoir.

Thus this twelve-case ledger does not silently omit an active electrode deposit, adsorbed/exchange pool, mixed crystal, unresolved object or radionuclide reservoir. The complete list of actual serialized keys and per-case snapshot counts is in the addendum.

## EC007: narrow positive coupling and the threshold constraint

During the actual Zn-add step, AgCl solid decreases by0.0000008516413108mol and dissolvedAg decreases by0.0000005166754639mol; together they account for all0.0000013683167747mol newAg metal. Zinc oxidation is0.0000006841583873mol, matching2Ag perZn. This proves a **partial** positive dissolution/redox response against the original introducedAg budget, with both phase contributions retained.

However,0.0003977800419145molAgCl and0.0002993158416127molZn remain while aqueousAg is0.0000008516413108mol, below the historical1e-6mol gate. **No coupled equilibrium endpoint or complete solid reduction is established.** Parent selection must explicitly retain that narrow original positive-response scope if accepted.

The Nernst instrument admission problem in EC004 and reaction-extent ownership in EC007 are related threshold questions but require separate contracts. `displacement.rs` explicitly uses the small-reaction cutoff to avoid silently assigning picomoleMg/free-acid conversion to the equilibrium route while the kinetic layer declaresMg/water reaction unmodelled. Do **not** globally lower `OBSERVABLE_MOLES`, relabel every caller a display bug or remove that ownership guard. First scope nonconsuming Nernst reading admission with positive solved activities and real metal; preserve zero-ion, unknown-activity and overpotential/refusal domains. A later coupled solid-reservoir continuation contract must preserve model authority andMg/water non-double-counting, account explicit phase budgets and demonstrate quantitative extent separately. Existing partial positive evidence is not a licence to claim completion.
