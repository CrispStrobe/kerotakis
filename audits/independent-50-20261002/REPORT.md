# Kerotakis independent experiment audit

Follow-up: [implemented fixes and final replay](AFTER.md), with [remaining systemic gaps](SYSTEMATIC.md). The baseline observations below are retained.

Fifty independently authored experiments were run against commit `f7d45f6f` on 2 October 2026, with predictions frozen before execution. The strongest improvement opportunities are consistent carbon inventory and speciation after a finite gas dose, observable-specific disclosure of missing thermal data, and separation operators that recognise the same material layers the user sees.

After the documented input adaptations, **42 predictions were met, 3 exposed engine mismatches, 1 encountered an explicit coarse model limitation, 1 required correcting my interpretation of the API, 1 matched the arithmetic but contradicted it in prose, and 2 reached explicit model boundaries**. These are dispositions of these fifty cases, not a coverage estimate for arbitrary chemistry.

| Disposition | Cases |
| --- | ---: |
| Met | 42 |
| Mismatch | 3 |
| Model limitation | 1 |
| Expectation corrected | 1 |
| Partial match | 1 |
| Explicit boundary | 2 |

The initial CLI runs completed successfully for 45 scripts and rejected 5 ingredient inputs. `C12H22O11` was replaced with the registered key `sucrose` in two follow-ups. `olive_oil` is unavailable; three follow-ups use the registered vegetable oil surrogate. That substitution checks general immiscible kitchen-oil behaviour, not an olive-specific composition. The original scripts and predictions remain unchanged.

## Fix priorities

### Keep carbon inventory and solved speciation consistent

Case 27 adds 1 mmol CO2 to 100 mL water in an open beaker. Its committed contents retain 1 mmol carbon, but multiplying the reported CO2 and bicarbonate molalities by the reported solvent mass gives only 0.001568 mmol. The ratio is 637.88. The reported pH is 5.6136. This is a contradiction inside one state, independent of any external reference value.

The `co2-consistency.lab` follow-up reproduces it. Waiting one second then reduces the represented carbon inventory to about 0.001570 mmol, while the pH barely changes. `aqueous.rs` at `recharacterise_canonically` solves the settled composition again and replaces `SolutionInfo` without updating contents. The likely constraint change is from the initial finite dose to the subsequent open reservoir problem. This follow-up diagnoses that path; it is not a new independently frozen prediction.

**Recommended first fix:** build the final inventory, speciation, pH, gas transfers, and energy record from the same constrained solve. A characterisation-only pass must verify that it has not changed total matter or gas constraints before replacing the snapshot. Add an invariant for carbon inventory versus represented solved species in this small, fully represented case, and check the finite-dose path through addition, waiting, and small heat pulses.

Source: [canonical recharacterisation](../../crates/kerotakis-phreeqc/src/aqueous.rs#L2922). Reproducer: [CO2 consistency](followups/co2-consistency.lab). Evidence: [follow-up JSON](results/co2-consistency-rerun.json.stdout).

### Make separation operators use the visible material layers

Case 46, after replacing unavailable olive oil with vegetable oil, shows two 50 mL layers. `drain` reports that this is a single-phase liquid and leaves the receiver empty. The scene obtains unresolved material layers from `material::immiscible_liquid_layers`; the operator consults `solve::layered_pair`, which requires a represented species pair. This is a capability seam between household material handling and species thermodynamics.

**Recommended second fix:** supply a common layer description to the scene and transfer operators. Drain the lower aqueous layer with its dissolved solutes while preserving the unresolved upper oil. If that operation is intentionally unsupported, say that the upper material layer cannot yet be separated; do not call a visibly layered vessel single-phase. Verify total resolved and unresolved mass across both vessels.

Sources: [drain](../../crates/kerotakis-core/src/bench.rs#L3263), [scene material layers](../../crates/kerotakis-core/src/scene.rs#L836). Reproducer: [oil drain](followups/46-oil-substitute.lab). Evidence: [JSON](results/46.corrected.json.stdout).

### Distinguish unpriced heat from zero heat

Case 26 dissolves 0.01 mol KNO3 in 100 g water and stays at exactly 25 C without a warning. The CaCl2 control warms to 26.98 C. The enthalpy routine rejects solids lacking dissolution data, and the aqueous fallback silently skips a dissolution without a usable enthalpy. The source comment explicitly explains that a broad not-modelled event would incorrectly classify otherwise computed aqueous results as missing.

That classification problem should be fixed at the observable level. **Recommended third fix:** retain computed pH/speciation while marking the temperature prediction as incomplete when a contributing heat is unpriced. Then add reviewed KNO3 dissolution data with its conditions and validity range. Treating missing heat as zero makes an ordinary cold-pack experiment appear thermally inert.

The endothermic sign was independently checked after execution against Parr's own [6755 solution calorimeter manual](https://www.parrinst.com/wp-content/uploads/downloads/2011/07/458M_Parr_6755-Solution-Calorimeter-Inst.pdf), printed section 6-6, PDF page 42. It describes a measured temperature drop for KNO3 dissolution in water. This supports the sign, not a precise temperature prediction for our adiabatic vessel. No value was inserted into the engine.

Sources: [missing enthalpy](../../crates/kerotakis-phreeqc/src/enthalpy.rs#L245), [silent fallback](../../crates/kerotakis-phreeqc/src/aqueous.rs#L3389). Reproducer: [hot and cold packs](scripts/26.lab).

## Other decisions

- **Electrolysis narration needs a Kero-side correction.** Case 38 computes the right Faraday amounts but applies the metal-deposition efficiency clause to hydrogen production. The text says hydrogen is not co-evolved and refers to an electrode weighing a ceiling. Branch the explanation by product and process; preserve the correct gas amounts. Source: [efficiency clause](../../crates/kerotakis-core/src/render.rs#L386).
- **Zinc in acid needs model validation.** Case 35 fails my physical expectation, but the engine explicitly explains a fixed overpotential gate. The source itself calls the values approximate, current-density dependent, curated, and uncited. A value tabulated at a particular current density does not establish a zero rate for an unspecified metal area and lesson duration. Avoid tuning a threshold to force this one case to pass. Validate a mixed-potential/rate model, or label the outcome as an unresolved kinetic prediction rather than a definitive absence of reaction. Source: [overpotential assumptions](../../crates/kerotakis-core/src/displacement.rs#L219).
- **The battery sign expectation was mine to correct.** Case 36 reports the spontaneous cell magnitude and selects anode/cathode independently of argument order. That is coherent. A signed voltmeter mode would be a separate feature; the chemistry does not require a fix.
- **Copper absorbance and ethanol combustion have explicit boundaries.** Cases 39 and 43 do not silently fabricate those answers. They are capability opportunities. Ethanol ignition does vaporise the whole dose before declining combustion, so the explanation should identify which physical action committed. Sodium flame testing also emits a temporary melting event despite restoring the original state; preview events should be distinguishable from committed transitions.
- **Phase measurement scope could be clearer.** Case 47 returns vessel-level pH and conductivity for a vessel with water and oil. The contents and layers remain distinct; label the readings as properties of the aqueous layer so they cannot be read as properties of the oil.
- **Do not broaden models just to improve a pass count.** Milk curdling is a conserved bounded aggregate model and states its absent casein chemistry. Peroxide catalysis computes a stoichiometrically coherent response, but this audit does not validate its rate constant experimentally.

## Method and reproducibility

The predictions were authored from ordinary physical and chemical expectations. No shipped lesson, coverage corpus, prior experiment audit, or golden result was opened to design these cases. CLI grammar and syntax documentation were consulted. The introductory README was read while finding the CLI and contains capability examples, so this is an independent audit, not a strictly blinded assessment. Implementation code was inspected only after the relevant experimental results were obtained.

The fifty designs are in [predictions.json](predictions.json), their initial files in [scripts](scripts), and verbatim JSON/prose outputs and errors in [results](results). [execution.json](execution.json) records the commit, executable hash, prediction hash, exit codes, times, and single-worker setting. Predictions hash: `0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.

The native CLI was built from the pulled revision with PHREEQC enabled. Its aqueous router legitimately uses the analytic water relation for solvent-only states and PHREEQC for represented aqueous chemistry. The audit does not force a particular solver route. The large fresh dependency build was slowed by shared system load and memory pressure; bypassing the shared Rust compiler cache completed the final steps. No production source or registry data was changed.

Run the frozen suite from the repository root:

```sh
RUSTC_WRAPPER= cargo build -p kerotakis-cli -j 1
KERO_WORKERS=1 python3 audits/independent-50-20261002/run.py
```

Run individual follow-ups with `target/debug/kero run <file.lab> --json`. Their current outputs are archived separately and are not overwritten by the baseline runner. The original zero-second wait probe was rejected and was rerun with one second; the successful reproducer is the one-second script.

```sh
python3 audits/independent-50-20261002/assess.py
```

[checks.json](checks.json) records 50 numerical or explicit-boundary checks: 46 pass and 4 fail. Those four are KNO3 cooling, carbon consistency, zinc dissolution, and oil drainage. The checks are deliberately narrower than the final dispositions: for example, electrolysis arithmetic passes while its prose fails, and battery identity checks pass after correcting the API interpretation. The harness exits successfully when this recorded pattern is reproduced; its known mismatches are not passing chemistry tests. Follow-up corrections are explicit, and the frozen expectations are retained in [verdicts.json](verdicts.json).

## All fifty experiments

Each entry links its original frozen script. Observations specify any subsequent adaptation.

### 01 A measured joule becomes warm water

**Prediction:** 10 kJ into 100 g water should raise temperature roughly 24 K, without mass loss below boiling.

**Observed:** 48.9117 C and 100.0000 g after the pulse; temperature rose 23.9117 K.

**Disposition:** Met. [Original script](scripts/01.lab).

### 02 Undo a thermal pulse

**Prediction:** Heating then removing the same 10 kJ should return 100 g water close to its initial temperature and mass.

**Observed:** Returned to 25.0000 C with the original water inventory.

**Disposition:** Met. [Original script](scripts/02.lab).

### 03 Mix unequal hot and cold portions

**Prediction:** 75 g at 20 C plus 25 g at 80 C should equilibrate near 35 C; mass should be 100 g.

**Observed:** 35.0000 C and 100 g, matching the weighted mean.

**Disposition:** Met. [Original script](scripts/03.lab).

### 04 A double dose is not a double temperature rise

**Prediction:** Equal 10 kJ pulses into 100 g and 200 g water should produce approximately 2:1 temperature rises.

**Observed:** Temperature rises were 23.9117 K and 11.9559 K, giving a 2.000 ratio.

**Disposition:** Met. [Original script](scripts/04.lab).

### 05 Latent heat intercepts the cooling pulse

**Prediction:** Removing 20 kJ from 100 g water initially at 5 C should give an ice/water mixture near 0 C, not liquid at -43 C.

**Observed:** Ice and liquid coexist at approximately 0 C. Aqueous pH is explicitly withdrawn.

**Disposition:** Met. [Original script](scripts/05.lab).

### 06 Boiling spends energy on disappearing liquid

**Prediction:** 40 kJ into 100 g water at 95 C should approach 100 C and evaporate roughly 17 g rather than heat all liquid to 190 C.

**Observed:** 100 C plateau with 83.2 g remaining, approximately 16.8 g evaporated. Settled pH is withdrawn.

**Disposition:** Met. [Original script](scripts/06.lab).

### 07 Salt versus sugar freezing race

**Prediction:** At equal 0.02 mol doses in 100 g water, NaCl should depress freezing roughly twice as much as sucrose. Both should form ice after 15 kJ cooling from 5 C.

**Observed:** After using the registered key sucrose, salt froze at -1.086 C and sugar at -0.589 C; both formed ice. Freeze concentration makes the final ratio differ from the initial dilute estimate.

**Disposition:** Met. [Original script](scripts/07.lab).

### 08 Heat dilution cannot invent energy

**Prediction:** Mixing two equal water portions at 60 C should keep them near 60 C, not reset the second dose to room temperature.

**Observed:** 60.0000 C after both additions, with 100 g water.

**Disposition:** Met. [Original script](scripts/08.lab).

### 09 A tenfold acid dilution

**Prediction:** Diluting 0.001 mol HCl in 100 mL water with another 900 mL should raise pH by about one unit (roughly 2 to 3).

**Observed:** pH changed from 2.0381 to 3.0135, a rise of 0.9754.

**Disposition:** Met. [Original script](scripts/09.lab).

### 10 Acid/base cancellation and overshoot

**Prediction:** 0.001 mol HCl followed by equal NaOH should reach near-neutral pH; another 0.001 mol NaOH should make it alkaline.

**Observed:** Equal acid/base doses gave pH 6.9927; excess base gave 11.9275.

**Disposition:** Met. [Original script](scripts/10.lab).

### 11 Weak acid versus strong acid

**Prediction:** Equal 0.001 mol doses in 100 mL should give acetic acid a higher pH than HCl, by at least one unit.

**Observed:** HCl pH 2.0381 versus acetic acid 3.3871, a difference of 1.3490.

**Disposition:** Met. [Original script](scripts/11.lab).

### 12 Half-neutralisation exposes a pKa

**Prediction:** 0.01 mol acetic acid and 0.005 mol NaOH in 100 mL should form a buffer around pH 4.5-5.0.

**Observed:** pH 4.6630, within the frozen expected interval.

**Disposition:** Met. [Original script](scripts/12.lab).

### 13 A buffer survives a small acid pulse

**Prediction:** A half-neutralised acetate buffer should change pH far less than plain water after the same 0.0001 mol HCl pulse.

**Observed:** Buffer pH changed by 0.0173; water changed by 3.9839 under the same acid pulse.

**Disposition:** Met. [Original script](scripts/13.lab).

### 14 Neutral salt is an ionic conductor

**Prediction:** Adding 0.001 mol NaCl to 100 mL water should increase conductivity substantially while leaving pH approximately neutral.

**Observed:** Conductivity rose from 0.0551 to 1204.2742 microS/cm; pH remained 6.9968.

**Disposition:** Met. [Original script](scripts/14.lab).

### 15 Sugar is not an electrolyte

**Prediction:** A 0.001 mol sucrose dose should not increase conductivity like an equal NaCl dose; any missing conductivity model should be explicit.

**Observed:** After using sucrose, conductivity was 0.0551 microS/cm versus 1204.2742 for NaCl.

**Disposition:** Met. [Original script](scripts/15.lab).

### 16 Common ions make a silver cloud

**Prediction:** 0.001 mol each of NaCl and AgNO3 in 100 mL should produce close to 0.001 mol solid AgCl, with conserved silver and chloride.

**Observed:** Approximately 0.998 mmol AgCl formed from 1 mmol feeds.

**Disposition:** Met. [Original script](scripts/16.lab).

### 17 Precipitation cannot depend on bottle order

**Prediction:** Reversing the equal silver/chloride additions should leave essentially equal AgCl solid amounts and pH in the two vessels.

**Observed:** Both orders gave the same solid yield, pH, temperature, and final inventories within the checked tolerances.

**Disposition:** Met. [Original script](scripts/17.lab).

### 18 Remove the cloud without removing the salt water

**Prediction:** Filtering an AgCl suspension should retain its solid in v1 and move aqueous material to v2; total inventory should be conserved.

**Observed:** Approximately 1 mmol AgCl remained in v1. Water and dissolved Na/Cl/nitrate transferred to v2.

**Disposition:** Met. [Original script](scripts/18.lab).

### 19 Barium and sulfate meet

**Prediction:** Equal 0.001 mol BaCl2 and Na2SO4 in 100 mL should form very insoluble BaSO4, or explicitly identify unavailable chemistry.

**Observed:** Approximately 0.998 mmol BaSO4 formed.

**Disposition:** Met. [Original script](scripts/19.lab).

### 20 Chalk dissolves in acid

**Prediction:** 0.001 mol CaCO3 should be consumed by 0.003 mol HCl in 100 mL; carbon dioxide should be accounted for and excess acid remain.

**Observed:** Chalk was consumed; about 0.999 mmol CO2 vented, with trace dissolved carbon retained. Excess acid gave pH 2.0629.

**Disposition:** Met. [Original script](scripts/20.lab).

### 21 Finite solvent leaves salt behind

**Prediction:** 100 g NaCl in 100 g water should leave substantial undissolved solid instead of an unlimited solution.

**Observed:** About 1.100 mol NaCl remained solid alongside 0.611 mol dissolved sodium and chloride. Total mass was 200 g.

**Disposition:** Met. [Original script](scripts/21.lab).

### 22 Extra solvent rescues leftover salt

**Prediction:** Adding another 500 mL water to 50 g NaCl in 100 g water should dissolve the residual solid.

**Observed:** Residual 0.244 mol solid disappeared after dilution. Final dissolved sodium and chloride were approximately 0.856 mol each.

**Disposition:** Met. [Original script](scripts/22.lab).

### 23 Drying a dilute salt solution

**Prediction:** Evaporating 90% of the solvent from 0.001 mol NaCl in 100 g water should conserve sodium/chloride while increasing concentration roughly tenfold.

**Observed:** Water fell to 10 g while 1 mmol sodium/chloride remained. Vaporisation enthalpy is explicitly outside this operator's balance.

**Disposition:** Met. [Original script](scripts/23.lab).

### 24 A quarter of a solution carries a quarter of its salt

**Prediction:** Decanting fraction 0.25 should send about 25% of dissolved NaCl and water to v2 and conserve combined mass.

**Observed:** v2 received 2.50 mmol sodium/chloride and one quarter of the water; combined inventory was conserved.

**Disposition:** Met. [Original script](scripts/24.lab).

### 25 Splitting and reuniting a solution

**Prediction:** Decant 0.4 into v2 then all of v2 back to v1 should restore initial inventory, temperature, and pH within numerical tolerance.

**Observed:** The reunited contents and temperature match the initial solution; v2 was empty.

**Disposition:** Met. [Original script](scripts/25.lab).

### 26 A hot pack versus a cold pack

**Prediction:** Dissolving 0.01 mol CaCl2 should warm 100 g water; dissolving 0.01 mol KNO3 should cool it, or name missing enthalpy data.

**Observed:** CaCl2 warmed water to 26.98 C. KNO3 dissolved completely but left it at exactly 25 C, with no thermal limitation event.

**Disposition:** Mismatch. [Original script](scripts/26.lab).

### 27 CO2 changes water chemistry

**Prediction:** Adding 0.001 mol CO2 to 100 mL water should acidify it; absorbed and released carbon should be distinguished.

**Observed:** pH fell to 5.6136, but retained carbon was 1 mmol while the speciation snapshot represented only 0.001568 mmol. The qualitative prediction hides a quantitative inconsistency.

**Disposition:** Mismatch. [Original script](scripts/27.lab).

### 28 A small sealed headspace makes fizz push back

**Prediction:** Acidifying 0.001 mol NaHCO3 with 0.002 mol HCl over a sealed 10 mL headspace should raise pressure above ambient and retain carbon.

**Observed:** Pressure reached 128.016 kPa in the sealed 10 mL headspace; carbon was partitioned between liquid and gas.

**Disposition:** Met. [Original script](scripts/28.lab).

### 29 Headspace size controls pressure

**Prediction:** The same acid/carbonate doses should produce higher final pressure over 10 mL than 100 mL sealed headspace.

**Observed:** Pressures were 128.016 kPa for 10 mL and 114.854 kPa for 100 mL.

**Disposition:** Met. [Original script](scripts/29.lab).

### 30 Uncorking releases the compressed gas

**Prediction:** Opening an acid/carbonate vessel should restore ambient gas pressure and account for vented gas without losing sodium/chloride.

**Observed:** Opening restored 101.325 kPa, vented gas explicitly, and retained 1 mmol sodium and 2 mmol chloride.

**Disposition:** Met. [Original script](scripts/30.lab).

### 31 Fixed pressure trades pressure rise for expansion

**Prediction:** Acid/carbonate under 1 bar regulation should stay near 1 bar and increase gas volume or explicitly state the unsupported constraint.

**Observed:** Pressure stayed at 100.000 kPa while gas volume expanded from 10 to 13.466 mL.

**Disposition:** Met. [Original script](scripts/31.lab).

### 32 Sweeping carbon dioxide out

**Prediction:** A nitrogen sweep after a finite CO2 dose should lower retained carbon and raise pH relative to the carbonated state, with a gas-transfer account.

**Observed:** pH rose from 4.3475 to 6.9974 and represented carbon was removed with explicit outward gas events.

**Disposition:** Met. [Original script](scripts/32.lab).

### 33 A sacrificial zinc strip plates copper

**Prediction:** Zinc in 0.001 mol CuSO4 solution should produce copper metal and consume zinc; mass/inventory should show the exchange.

**Observed:** Approximately 0.99646 mmol copper plated while the same amount of zinc was consumed. Both metal totals were conserved.

**Disposition:** Met. [Original script](scripts/33.lab).

### 34 Copper refuses the reverse displacement

**Prediction:** Copper in ZnSO4 should not plate zinc; an explicit no-reaction result should be chemically distinguished from missing chemistry.

**Observed:** Copper remained as 2 mmol metal. Zinc metal did not form, and the refusal explained the thermodynamic direction.

**Disposition:** Met. [Original script](scripts/34.lab).

### 35 Acid dissolves zinc but leaves copper

**Prediction:** Equal metal doses with dilute HCl should dissolve zinc and evolve H2, while copper remains under non-oxidising conditions.

**Observed:** Both metals remained unchanged. Zinc was declared kinetically blocked using a fixed 0.72 V hydrogen-overpotential gate. This is an explicit approximation, not a validated reaction-rate prediction.

**Disposition:** Model limitation. [Original script](scripts/35.lab).

### 36 A battery has a polarity

**Prediction:** Zn/ZnSO4 and Cu/CuSO4 half-cells at similar concentration should give about 1.1 V; reversing cell arguments should reverse the sign.

**Observed:** Both argument orders reported +1.1041 V with the same physical anode and cathode. The API chooses spontaneous polarity rather than treating arguments as voltmeter leads; my sign expectation was wrong for this API.

**Disposition:** Expectation corrected. [Original script](scripts/36.lab).

### 37 A concentration cell vanishes at equality

**Prediction:** Identical copper half-cells should give near-zero voltage; diluting one tenfold should produce a nonzero voltage of order tens of mV.

**Observed:** Equal half-cells explicitly gave zero potential; tenfold dilution produced a 22.0 mV concentration cell.

**Disposition:** Met. [Original script](scripts/37.lab).

### 38 A measured charge liberates a measured amount

**Prediction:** Electrolysing acidified water at 0.1 A for 100 s should account for about 5.18e-5 mol H2 and 2.59e-5 mol O2, or name electrode/product limitations.

**Observed:** 10 C produced 5.1821e-5 mol H2 and 2.5911e-5 mol O2. The prose nevertheless says no hydrogen is co-evolved and refers to an electrode mass ceiling while hydrogen is the stated product.

**Disposition:** Partial match. [Original script](scripts/38.lab).

### 39 Dilution lightens a coloured solution

**Prediction:** Tenfold dilution of CuSO4 should lower optical absorbance and lighten colour; any unavailable spectrum should be named.

**Observed:** The drawing lightened from #008CDB to #CFEBFB. Both complete absorbance measurements were explicitly withheld because several copper-complex spectra are missing.

**Disposition:** Explicit boundary. [Original script](scripts/39.lab).

### 40 Purple colour follows concentration

**Prediction:** Tenfold dilution of dilute KMnO4 should reduce absorbance and fade purple toward pink without losing permanganate inventory.

**Observed:** Absorbance changed from 2.4000 to 0.2400 at 525 nm; purple became pink and permanganate inventory was retained.

**Disposition:** Met. [Original script](scripts/40.lab).

### 41 A salt flame is not fuel

**Prediction:** Igniting NaCl should report a yellow sodium flame signature or a boundary, without consuming salt as combustible fuel.

**Observed:** Sodium yellow emission was reported and all NaCl remained. A temporary melting event is emitted even though the committed state returns to solid at 25 C.

**Disposition:** Met. [Original script](scripts/41.lab).

### 42 Magnesium gains oxygen mass

**Prediction:** Igniting 0.001 mol Mg should create MgO and gain about 0.016 g from atmospheric oxygen, with the external mass source explained.

**Observed:** 1 mmol Mg became 1 mmol MgO; the balance read 0.0403 g, consistent with approximately 0.016 g oxygen uptake.

**Disposition:** Met. [Original script](scripts/42.lab).

### 43 Ethanol burns instead of returning a false negative

**Prediction:** Igniting 1 g ethanol should produce combustion products/heat or explicitly identify missing combustion modelling.

**Observed:** Combustion is explicitly unavailable. However, ignition first vaporises the entire ethanol dose, leaving an empty vessel at 25 C; this state change needs a clearer scope explanation.

**Disposition:** Explicit boundary. [Original script](scripts/43.lab).

### 44 Distillation enriches a volatile component

**Prediction:** Distilling 10% of an ethanol/water mixture should enrich ethanol in the receiver and conserve both components across vessels plus declared losses.

**Observed:** The receiver was enriched to approximately 63.4% ethanol by mass versus 20% initially; combined water and ethanol were conserved. The 0.1 cut refers to moles, so receiver mass was 14.3 g.

**Disposition:** Met. [Original script](scripts/44.lab).

### 45 Oil and water keep separate layers

**Prediction:** Equal 50 mL doses of water and olive oil should form separate phases; unresolved oil should be explicitly conserved, not dissolved into imaginary aqueous species.

**Observed:** Olive oil is not a registered input. Substituting vegetable_oil produced separate 50 mL water and oil layers, conserving 46 g unresolved oil.

**Disposition:** Met. [Original script](scripts/45.lab).

### 46 Draining the bottom layer

**Prediction:** Draining a water/oil mixture should send its lower aqueous layer to v2 while retaining oil in v1 and conserving combined inventory.

**Observed:** With vegetable_oil substituted, drain moved nothing and called the vessel single-phase even though the scene showed two layers.

**Disposition:** Mismatch. [Original script](scripts/46.lab).

### 47 Salt cannot pass through an oil layer as water

**Prediction:** Salt added to water/oil should reside in the aqueous phase; the oil layer should not acquire aqueous conductivity or pH as if homogeneous.

**Observed:** With vegetable_oil substituted, sodium/chloride remained aqueous beside the unresolved oil. Vessel measurements refer to the aqueous solution, but the interface could label that scope more clearly.

**Disposition:** Met. [Original script](scripts/47.lab).

### 48 Acid curdles milk without inventing protein molecules

**Prediction:** Acidifying milk should show aggregation/curdling or a clear protein-model boundary, and retain unresolved milk mass.

**Observed:** Acid produced a bounded curd response and approximately 3.416 g aggregate. The 12.20149 g unresolved material was retained; missing casein buffering was explicitly disclosed.

**Disposition:** Met. [Original script](scripts/48.lab).

### 49 A peroxide catalyst has an explicit model boundary

**Prediction:** MnO2 added to dilute hydrogen peroxide should yield oxygen and catalytic turnover or explicitly say kinetics/catalysis is unavailable; silence would fail.

**Observed:** After 60 s, 0.77449 mmol O2 formed and 1.54898 mmol peroxide was consumed; MnO2 stayed at 1 mmol. The numerical kinetic rate was not independently validated in this audit.

**Disposition:** Met. [Original script](scripts/49.lab).

### 50 Waiting is not permission to teleport matter

**Prediction:** Waiting one hour with dilute NaCl at room temperature should conserve dissolved salt and avoid spontaneous precipitation or chemical conversion; any evaporation must be accounted.

**Observed:** One hour preserved the salt and water inventory exactly; no spontaneous conversion or unaccounted evaporation occurred.

**Disposition:** Met. [Original script](scripts/50.lab).

## Resource use and worktree consolidation
Validation and experiment execution used resource-aware scheduling. Machine-specific resource snapshots, storage locations and preservation inventories are retained privately; public scientific evidence remains indexed by the run links and receipts in this report.