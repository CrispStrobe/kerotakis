# Saved first120 precipitation assessment

Proposed criterion review; original forecasts and execution evidence remain unchanged. No engines or API requests were run. All 18 original cases and 36 criteria are included.

Dataset SHA-256: `8e9fb7a92c20ad4de747d43540e417684ba398ff3896ab9d3de270e5a99b1fdd`. Archive SHA-256: `9c33ddf3893ab32e0309dc1ef5c65b3b2c4e2967c8a3468c2808b0de1074201f`. The offline extractor verifies all selected exact scripts, stdout/stderr streams and execution receipts against `verified-results.json`; duplicate ZIP member names, duplicate JSON keys and non-finite JSON constants are rejected. Source and binary are the c446/670 producer receipt binding, not later merged capabilities. Raw Cargo.lock and executable bytes are absent; the lock/binary identity remains producer-attested. Original design bytes hash to `e89c2f91b07ecb639436d75bae7417b5af9d3febfecb977519a2742fde692ea0` as bound; archived design bytes hash to `8809b1558840709aed6f046087727a3d8b0a37fe031be1cf3a1e472c532ed05c`. Full parsed JSON values are identical. The runner reserializes Unicode escapes (`ensure_ascii=True`), so archive-design raw-byte identity is not claimed.

See [assessment.json](assessment.json) for original scripts, forecasts, verbatim criteria, authenticated hashes, every saved inventory snapshot, and exact limitation/gas events. Reproduce extraction with `python3 audits/target468-20261016/saved-assessment-first120-precip/assess.py`; it reads only saved repository files.

`met` describes the literal criterion within the represented model. It is neither empirical validation nor a full chemistry/heat pass. Canonical contents are analytical ownership; rounded solution species and historical lots must not be added again. Gas export is counted once, and sealed atmospheric inventory is included. No new numerical acceptance tolerance is supplied.

## new120:pr-001 — met

Zinc hydroxide precipitates, then dissolves in excess base

Original forecast: A moderate hydroxide dose gives Zn(OH)2; sufficiently strong excess hydroxide can redissolve it into soluble hydroxo-zinc species.

- **PR-001-C1: met** — A zinc-bearing hydroxide solid appears at the intermediate snapshot.
- **PR-001-C2: met** — Its amount decreases after excess base while total zinc remains constant; unsupported hydroxo-complex coverage is not a chemical pass.

Rows 3/5: Zn(OH)2 solid falls from 0.0009992239624 to 0.0009743391348 mol. Aqueous Zn rises from 7.76037646e-7 to 2.566086517e-5 mol; total Zn is 0.001 mol to floating-point residual. Final Zn(OH)4-2 and Zn(OH)3- molalities are 0.0001647 and 0.00008884. This supports a small decrease (about 2.49% of intermediate solid), not complete clearing.

No unsupported-hydroxo escape is used. No full dissolution yield, calibrated spectrum or complete heat claim.

## new120:pr-002 — met

Dissolve magnesium hydroxide and reprecipitate it

Original forecast: Acid dissolves the initial hydroxide; restoring alkalinity produces a magnesium hydroxide solid again without inventing a magnesium-chloride bottle key.

- **PR-002-C1: met** — The solid inventory falls on acidification and rises again after base addition.
- **PR-002-C2: met** — Total magnesium stays at 0.001 mol across the cycle.

Rows 2/4/6: Mg(OH)2 solid 0.0009867943731 -> 0 -> 0.0009835919635 mol. Aqueous Mg 0.00001320562691 -> 0.001 -> 0.00001640803648 mol. The phase sums retain the original 0.001 mol magnesium within recorded floating-point residual.

Exact numerical residuals remain in the snapshots; no new acceptance tolerance is introduced.

## new120:pr-003 — met

Ferric hydroxide redissolves when its pH support is removed

Original forecast: Brown ferric hydroxide formed by base addition is substantially removed by a deliberate excess of acid.

- **PR-003-C1: met** — Ferric solid decreases after the acid challenge.
- **PR-003-C2: met** — Iron remains in the vessel in dissolved forms; disappearance of turbidity is not disappearance of iron.

Rows 3/5: Fe(OH)3 solid 0.0009999958515 -> 0 mol; final aqueous Fe+3 analytical total 0.001 mol and pH 1.59949280306. Initial dissolved Fe 4.148537473e-9 mol. Dissolved ownership, rather than appearance, establishes iron retention.

No claim of a new pure iron salt bottle or free Fe3+ concentration from the analytical carrier.

## new120:pr-006 — met

An acid reserve can prevent calcium carbonate precipitation

Original forecast: Without prior acid, carbonate precipitates calcium. With excess proton capacity, carbonate is protonated and the calcium mostly remains dissolved.

- **PR-006-C1: met** — The unacidified vessel has more calcium-carbonate solid.
- **PR-006-C2: met** — Account for acid consumption and carbon leaving the open acidified vessel as CO2.

Rows 3/9: neutral CaCO3 solid 0.0009826363956 mol versus acidified solid 0. Final acidified dissolved Ca 0.001 mol. Added HCl 0.003 mol; final analytical H+ 0.0010000000846 mol, consistent with approximately 0.002 mol proton consumption. Row 8 CO2 gas_evolved exports 0.0009986985126323 mol; remaining aqueous carbon plus this export is 0.0010000000000000002 mol.

Gas_evolved in the open boundary is an export, not retained headspace inventory; count the event once, not at every later inspect. Carbon quantities are explicitly partitioned, not inferred from lost vessel mass.

## new120:pr-007 — met

Limewater clouds and then clears under excess carbon dioxide

Original forecast: Initial CO2 converts hydroxide into calcium carbonate; excess retained CO2 lowers pH and favours dissolved calcium/bicarbonate.

- **PR-007-C1: met** — A calcium-carbonate maximum occurs at the intermediate stage.
- **PR-007-C2: met** — The final solid fraction is lower, with carbon and calcium accounted for across all phases.

Rows 4/6: CaCO3 solid 0.0004854705420399 -> 0.00001193221964 mol; final pH 6.466422829211. Ca all-phase sum stays approximately 0.0005 mol. Sealing row 1 initially incorporates 0.00000817480890486671 mol atmospheric carbon; after initial CO2 dose total C is 0.0005081748088250311 and after final dose 0.005508174808824631 mol. Final retained gas CO2 0.003878660411 mol belongs inside the sealed boundary.

The initial atmospheric carbon must be included: comparing final C only to the 0.0055 mol explicit dose creates a false imbalance. Recorded C residual against row1 + doses is about -8.02e-14 mol; no empirical gas-equilibrium or kinetic validation.

## new120:pr-014 — met

Purify impure limestone by dissolution and reprecipitation

Original forecast: Acid dissolves the calcium carbonate, filtration rejects sand, and carbonate addition recovers a calcium-rich precipitate in the receiver.

- **PR-014-C1: met** — Most SiO2 remains in v1 during filtration.
- **PR-014-C2: met** — A calcium-carbonate solid reappears in v2; compare recovered calcium, not total crude solid mass.

Filter row 5: v1 solid SiO2 0.001644364588 mol, v2 dissolved SiO2 0.00001997200777 mol; about 98.8000% of represented Si remains in source. Receiver initially has approximately 0.001 mol dissolved Ca; final row 8 has CaCO3 solid 0.0009825806053 mol and dissolved Ca 0.00001741939335 mol, about 98.2581% calcium recovery into the precipitate.

The receiver has dissolved silica impurity: do not label the recovered material analytically pure or equate gross mass to calcium recovery. Both vessel boundaries are retained.

## new120:pr-015 — met

Convert copper oxide through a soluble salt to a hydroxide

Original forecast: Acid dissolves black copper oxide; subsequent base addition forms a copper hydroxide-rich solid while preserving copper inventory.

- **PR-015-C1: met** — The initial oxide decreases after acid addition.
- **PR-015-C2: met** — The final precipitate is identified by composition, not assumed to be unchanged CuO simply because copper stayed in the vessel.

Addition row 1 establishes CuO solid 0.00099990314562 mol before acid; row 2 and inspect row 3 show CuO absent, dissolved Cu 0.0009984574955 mol plus atacamite 7.712522298e-7 mol (two Cu per formula) retain approximately 0.001 mol Cu. Final row 5 identifies Cu(OH)2 0.0009987765155 mol plus atacamite 1.639896734e-7 mol and dissolved Cu 8.955020096e-7 mol.

The automatic add-row snapshot supplies the pre-acid baseline even though no explicit inspect was authored there. C2 composition is explicit and copper closure needs the two-Cu atacamite coefficient.

## new120:pr-004 — met

Excess ammonia can dissolve a copper hydroxide precipitate

Original forecast: Chemical expectation: excess ammonia stabilises dissolved copper ammine species and can remove the hydroxide precipitate, with a changed copper colour.

- **PR-004-C1: met** — Require a decrease of copper-bearing solid together with a copper-ammine increase; colour alone does not establish complex formation.
- **PR-004-C2: met** — Check copper and total nitrogen ownership, including headspace ammonia.

Rows 4/6: copper solid counted with stoichiometry is 0.0004991011238 -> 0.0004689092011 mol (initial brochantite contributes four Cu per formula). Final Cu(NH3)4+2 and Cu(NH3)3+2 molalities 0.0002845 and 0.00001816 are positive; no ammines in the initial reported distribution. All-phase Cu totals 0.0004999999999932997 -> 0.0004999999999932837 mol. Final aqueous analytical NH3 + NH4+ plus gas NH3 = 0.05000000000000062 mol; the gas share is 0.0001784815665 mol. Initial headspace N2 is retained separately and was not added as ammonia.

Equilibrium ligand carriers already include bound ammonia: do not add ammine ligand moles again. UV/visible measurement explicitly unsupported; reference-temperature complex constants, incomplete complex heat and spectra are disclosed. Only criterion-scoped equilibrium inventory behavior is met, not quantitative colour or full heat.

## new120:pr-005 — unsupported

Selective hydroxide precipitation separates ferric and cupric ions

Original forecast: A moderately acidic endpoint should preferentially remove ferric hydroxide while leaving much more copper in the filtrate.

- **PR-005-C1: met** — At the recorded endpoint, compare fractions of each metal in solid and dissolved pools.
- **PR-005-C2: unsupported** — The iron fraction retained in the source should exceed the copper fraction retained there; inspect endpoint overshoot and phases.

Rows 4/5 endpoint pH 3.999992675794: dissolved Fe+2 + Fe+3 = 0.0002 mol and Cu = 0.0002 mol; neither has represented solid. Filter row 7 leaves both source fractions zero and transfers both metal totals to v2. Thus Fe retained fraction 0 does not exceed Cu retained fraction 0.

The intended selective precipitation is not achieved. Named endpoint diagnostics exclude Fe(OH)3(a) despite SI +1.7 and other iron phases; missing phase selection/ownership is a model gap, not endpoint overshoot. C1 is the requested comparison, not a selective-separation pass.

## new120:pr-008 — unsupported

Carbonate and sulfate precipitates respond differently to acid

Original forecast: Barium carbonate is acid-labile and can release CO2; barium sulfate should largely persist under the same dilute-acid challenge.

- **PR-008-C1: unsupported** — First verify formation of each intended solid, including any witherite phase representation.
- **PR-008-C2: unsupported** — After acid addition, compare each solid with its own initial amount; carbonate loss should greatly exceed sulfate loss.

Rows 3/8: no BaCO3 solid in carbonate vessel; Ba remains 0.0005 mol aqueous. Sulfate vessel contains 0.0004984214374 mol BaSO4. After acid, BaSO4 is 0.0004971799529 mol, loss about 0.2491%; carbonate solid has no initial amount from which dissolution could be assessed.

Row 2 explicitly reports Witherite SI +3.0 with no registry solid. CO2 export 0.0004986843358451 mol proves acid action on dissolved carbonate only; it cannot demonstrate dissolution of absent BaCO3.

## new120:pr-009 — met

Not every alkaline-earth sulfate precipitates at the same concentration

Original forecast: At these dilute analytical loadings, calcium sulfate is expected to remain mostly dissolved whereas barium sulfate precipitates strongly.

- **PR-009-C1: met** — The barium vessel has a much larger fraction of its alkaline-earth metal in a sulfate solid.
- **PR-009-C2: met** — A clear calcium solution is not evidence that calcium and sulfate were omitted from its dissolved inventory.

Rows 3/8: CaSO4 solid absent, dissolved Ca and sulfate each 0.001 mol. BaSO4 solid 0.0009873155095 mol, about 98.73155% of added barium, with 0.00001268449048 mol dissolved Ba and sulfate each. Dissolved calcium and sulfate ownership is explicit.

Comparison concerns these specified dilute loadings and represented phases, not all alkaline-earth salts or a kinetic crystallization prediction.

## new120:pr-011 — met

Cross the silver-chloride ion-product threshold

Original forecast: The first trace additions are below the precipitation threshold; increasing both ion inventories crosses the solubility-product boundary.

- **PR-011-C1: met** — There is no substantial AgCl solid at the trace-loading snapshot.
- **PR-011-C2: met** — The later snapshot contains AgCl solid while dissolved silver and chloride remain finite.

Rows 3/6: no trace-stage AgCl solid; dissolved Ag and Cl are each 1e-7 mol. Final AgCl solid 8.720695373e-6 mol and aqueous Ag/Cl each 1.379304627e-6 mol. Their phase sum is 1.01e-5 mol, consistent with the cumulative dose.

This brackets the boundary at two specified loadings; it does not measure a solubility constant independently.

## new120:pr-012 — met

Selectively dissolve carbonate from a mixed mineral precipitate

Original forecast: Acid removes calcium carbonate from a mixture containing barium sulfate, after which filtration retains the sulfate-rich residue.

- **PR-012-C1: met** — Calcium transfers mainly to the filtrate, while barium remains mainly in retained sulfate solid.
- **PR-012-C2: met** — Track gaseous carbon separately; the mass loss is not solely a filtration loss.

Rows 4/8: initial CaCO3 0.000984452378 mol and BaSO4 0.0004983887231 mol. Final v1 retains 0.0004968793854 mol BaSO4 (about 99.3759% of total Ba); v2 contains approximately 0.001 mol dissolved Ca and 3.120613858e-6 mol Ba. Row 5 exports CO2 0.0009986883928653 mol; final aqueous carbon 1.3116072119447213e-6 mol, total with export approximately 0.0010000000000772447 mol.

Retained solid and filtrate are separate owners; CO2 export precedes filtration and is not filtration loss. Incidental missing Witherite phase is disclosed but does not remove observed calcium-carbonate and sulfate evidence.

## new120:pr-013 — met

One mixture can contain two distinct precipitates

Original forecast: Barium chloride supplies barium for sulfate precipitation and chloride for silver precipitation, yielding a mixed solid fraction.

- **PR-013-C1: met** — Require phase-resolved evidence for both AgCl and BaSO4, not just an undifferentiated white solid.
- **PR-013-C2: met** — The filtrate should retain spectator ions and any stoichiometric chloride excess.

Row 4 identifies separate BaSO4 0.000998167183 mol and AgCl 0.0009999307984 mol. Rows 6/7 retain both in v1. v2 dissolved Na 0.002 mol, nitrate 0.001 mol, chloride 0.001000069202 mol, plus finite dissolved Ba and Ag. Excess chloride is approximately 0.001 mol, as required by the formula doses.

Spectator inventory is analytical ownership; aqueous AgCl complexes are not additional precipitate or additional Ag ownership.

## new120:pr-017 — unsupported

Iodide precipitation extends the silver-halide test

Original forecast: Chemical expectation: iodide and silver ions form poorly soluble silver iodide, commonly described as a yellow precipitate.

- **PR-017-C1: unsupported** — Require an identified silver-iodide solid or an explicit unsupported-phase outcome.
- **PR-017-C2: met** — Do not count an AgCl-like generic precipitate or a successful command exit as confirmation.

Rows 3/5: no AgI solid; KI remains 0.0002 mol as aqueous unmapped salt, and Ag analytical total is 0.0002 mol. Row 1/2 explicitly says KI has no aqueous component mapping, excluding its ion distribution and reactions. Eyes report clear colourless liquid, not generic silver-halide precipitate.

C1 chemistry is unsupported at the earlier iodide-input mapping boundary; the warning is not specifically an AgI phase diagnostic. C2 guard is honored by this assessment: neither zero exit nor clear appearance is treated as confirmation.

## new120:pr-018 — unsupported

Bromide precipitation without inventing an unregistered salt bottle

Original forecast: Chemical expectation: bromide from hydrobromic acid forms a pale silver-bromide precipitate with silver ions.

- **PR-018-C1: unsupported** — Verify bromide and silver conservation and a specifically identified AgBr solid.
- **PR-018-C2: met** — Missing AgBr phase support is recorded as unsupported, not as evidence that no real chemical reaction exists.

Rows 3/5: total aqueous Ag and Br carriers each 0.0002 mol; specifically identified AgBr solid absent. Soluble AgBr, AgBr2- and AgBr3-2 occur in solution species but are aqueous complexes, not precipitate. Row 2 explicitly reports Bromyrite SI +5.1 with no matching registry solid.

Represented Ag/Br conservation is supported, precipitation is unsupported. C2 is the explicit coverage-limit guard, not a scientific no-reaction result. HBr uptake also discloses an instantaneous equilibrium/time-model boundary.

## new120:pr-010 — met

Distinguish a common-ion effect from a background-electrolyte effect

Original forecast: Chloride changes silver-chloride equilibrium through common-ion and complexation effects; nitrate mainly changes the ionic medium. Equal salt amounts do not make the effects equivalent.

- **PR-010-C1: met** — Compare total dissolved silver and free silver activity separately in all three vessels.
- **PR-010-C2: met** — In this modest-chloride range, common-ion suppression should exceed the nitrate-only change; note any competing soluble complexes.

Row 10, water/nitrate/chloride: analytical dissolved Ag 1.368316775e-6 / 1.511772570e-6 / 6.856108194e-8 mol; free Ag activity 1.334e-5 / 1.332e-5 / 1.968e-8 respectively. Chloride suppresses total dissolved Ag about 95.0% relative to water, whereas nitrate increases it about 10.48%. Chloride solution includes AgCl2-, AgCl and AgCl3-2 complexes at 3.313e-7, 3.301e-7 and 4.271e-9 mol/kgw.

Do not confuse aqueous AgCl complex with solid AgCl; solution activities are dimensionless reported values and species molalities have report rounding. No new equality tolerance or calibration.

## new120:pr-016 — met

Solvent identity changes the route to a silver-chloride precipitate

Original forecast: AgCl formation in water uses aqueous speciation, whereas the documented pure-ethanol route is constrained by dissolved reagent fractions and a curated metathesis rule.

- **PR-016-C1: met** — Check for AgCl in both systems and explicitly distinguish undissolved starting salt from precipitated product.
- **PR-016-C2: met** — Report actual conversion instead of assuming equal yields merely because equal formula amounts were added.

Rows 3/8: aqueous-route AgCl solid 0.0009992088475 mol (99.92088475% of Ag dose); ethanol-route AgCl solid 0.000556097394 mol (55.6097394%). Ethanol contains residual AgNO3 and NaCl, each 0.000443902606 mol labelled liquid, and NaNO3 solid 0.000556097394 mol. Neither starting salt is recorded as an undissolved solid in the final ethanol snapshot.

Report actual phase labels: the residual reagent is liquid-labelled, not invented undissolved solid. Solvent route is a curated metathesis model, not a validated equal-yield or generic nonaqueous equilibrium claim.
