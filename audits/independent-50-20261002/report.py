import json, statistics
from collections import Counter
from pathlib import Path
root=Path(__file__).resolve().parent
predictions=json.loads((root/'predictions.json').read_text())
execution=json.loads((root/'execution.json').read_text())
observations=[
'48.9117 C and 100.0000 g after the pulse; temperature rose 23.9117 K.',
'Returned to 25.0000 C with the original water inventory.',
'35.0000 C and 100 g, matching the weighted mean.',
'Temperature rises were 23.9117 K and 11.9559 K, giving a 2.000 ratio.',
'Ice and liquid coexist at approximately 0 C. Aqueous pH is explicitly withdrawn.',
'100 C plateau with 83.2 g remaining, approximately 16.8 g evaporated. Settled pH is withdrawn.',
'After using the registered key sucrose, salt froze at -1.086 C and sugar at -0.589 C; both formed ice. Freeze concentration makes the final ratio differ from the initial dilute estimate.',
'60.0000 C after both additions, with 100 g water.',
'pH changed from 2.0381 to 3.0135, a rise of 0.9754.',
'Equal acid/base doses gave pH 6.9927; excess base gave 11.9275.',
'HCl pH 2.0381 versus acetic acid 3.3871, a difference of 1.3490.',
'pH 4.6630, within the frozen expected interval.',
'Buffer pH changed by 0.0173; water changed by 3.9839 under the same acid pulse.',
'Conductivity rose from 0.0551 to 1204.2742 microS/cm; pH remained 6.9968.',
'After using sucrose, conductivity was 0.0551 microS/cm versus 1204.2742 for NaCl.',
'Approximately 0.998 mmol AgCl formed from 1 mmol feeds.',
'Both orders gave the same solid yield, pH, temperature, and final inventories within the checked tolerances.',
'Approximately 1 mmol AgCl remained in v1. Water and dissolved Na/Cl/nitrate transferred to v2.',
'Approximately 0.998 mmol BaSO4 formed.',
'Chalk was consumed; about 0.999 mmol CO2 vented, with trace dissolved carbon retained. Excess acid gave pH 2.0629.',
'About 1.100 mol NaCl remained solid alongside 0.611 mol dissolved sodium and chloride. Total mass was 200 g.',
'Residual 0.244 mol solid disappeared after dilution. Final dissolved sodium and chloride were approximately 0.856 mol each.',
'Water fell to 10 g while 1 mmol sodium/chloride remained. Vaporisation enthalpy is explicitly outside this operator\'s balance.',
'v2 received 2.50 mmol sodium/chloride and one quarter of the water; combined inventory was conserved.',
'The reunited contents and temperature match the initial solution; v2 was empty.',
'CaCl2 warmed water to 26.98 C. KNO3 dissolved completely but left it at exactly 25 C, with no thermal limitation event.',
'pH fell to 5.6136, but retained carbon was 1 mmol while the speciation snapshot represented only 0.001568 mmol. The qualitative prediction hides a quantitative inconsistency.',
'Pressure reached 128.016 kPa in the sealed 10 mL headspace; carbon was partitioned between liquid and gas.',
'Pressures were 128.016 kPa for 10 mL and 114.854 kPa for 100 mL.',
'Opening restored 101.325 kPa, vented gas explicitly, and retained 1 mmol sodium and 2 mmol chloride.',
'Pressure stayed at 100.000 kPa while gas volume expanded from 10 to 13.466 mL.',
'pH rose from 4.3475 to 6.9974 and represented carbon was removed with explicit outward gas events.',
'Approximately 0.99646 mmol copper plated while the same amount of zinc was consumed. Both metal totals were conserved.',
'Copper remained as 2 mmol metal. Zinc metal did not form, and the refusal explained the thermodynamic direction.',
'Both metals remained unchanged. Zinc was declared kinetically blocked using a fixed 0.72 V hydrogen-overpotential gate. This is an explicit approximation, not a validated reaction-rate prediction.',
'Both argument orders reported +1.1041 V with the same physical anode and cathode. The API chooses spontaneous polarity rather than treating arguments as voltmeter leads; my sign expectation was wrong for this API.',
'Equal half-cells explicitly gave zero potential; tenfold dilution produced a 22.0 mV concentration cell.',
'10 C produced 5.1821e-5 mol H2 and 2.5911e-5 mol O2. The prose nevertheless says no hydrogen is co-evolved and refers to an electrode mass ceiling while hydrogen is the stated product.',
'The drawing lightened from #008CDB to #CFEBFB. Both complete absorbance measurements were explicitly withheld because several copper-complex spectra are missing.',
'Absorbance changed from 2.4000 to 0.2400 at 525 nm; purple became pink and permanganate inventory was retained.',
'Sodium yellow emission was reported and all NaCl remained. A temporary melting event is emitted even though the committed state returns to solid at 25 C.',
'1 mmol Mg became 1 mmol MgO; the balance read 0.0403 g, consistent with approximately 0.016 g oxygen uptake.',
'Combustion is explicitly unavailable. However, ignition first vaporises the entire ethanol dose, leaving an empty vessel at 25 C; this state change needs a clearer scope explanation.',
'The receiver was enriched to approximately 63.4% ethanol by mass versus 20% initially; combined water and ethanol were conserved. The 0.1 cut refers to moles, so receiver mass was 14.3 g.',
'Olive oil is not a registered input. Substituting vegetable_oil produced separate 50 mL water and oil layers, conserving 46 g unresolved oil.',
'With vegetable_oil substituted, drain moved nothing and called the vessel single-phase even though the scene showed two layers.',
'With vegetable_oil substituted, sodium/chloride remained aqueous beside the unresolved oil. Vessel measurements refer to the aqueous solution, but the interface could label that scope more clearly.',
'Acid produced a bounded curd response and approximately 3.416 g aggregate. The 12.20149 g unresolved material was retained; missing casein buffering was explicitly disclosed.',
'After 60 s, 0.77449 mmol O2 formed and 1.54898 mmol peroxide was consumed; MnO2 stayed at 1 mmol. The numerical kinetic rate was not independently validated in this audit.',
'One hour preserved the salt and water inventory exactly; no spontaneous conversion or unaccounted evaporation occurred.'
]
assert len(observations)==50
special={26:'Mismatch',27:'Mismatch',35:'Model limitation',36:'Expectation corrected',38:'Partial match',39:'Explicit boundary',43:'Explicit boundary',46:'Mismatch'}
rows=[dict(**c,observed=observations[i-1],disposition=special.get(i,'Met'),adaptation=('registered sucrose key' if i in (7,15) else 'vegetable oil substituted for unavailable olive oil' if i in (45,46,47) else None)) for i,c in enumerate(predictions,1)]
(root/'verdicts.json').write_text(json.dumps(rows,indent=2)+'\n')
counts=Counter(r['disposition'] for r in rows)
run_seconds=sum(r['runs'][mode]['seconds'] for r in execution['cases'] for mode in ('json','text'))
text=f'''# Kerotakis independent experiment audit

Fifty independently authored experiments were run against commit `{execution['commit'][:8]}` on 2 October 2026, with predictions frozen before execution. The strongest improvement opportunities are consistent carbon inventory and speciation after a finite gas dose, observable-specific disclosure of missing thermal data, and separation operators that recognise the same material layers the user sees.

After the documented input adaptations, **42 predictions were met, 3 exposed engine mismatches, 1 encountered an explicit coarse model limitation, 1 required correcting my interpretation of the API, 1 matched the arithmetic but contradicted it in prose, and 2 reached explicit model boundaries**. These are dispositions of these fifty cases, not a coverage estimate for arbitrary chemistry.

| Disposition | Cases |
| --- | ---: |
'''
for label,n in counts.items(): text+=f'| {label} | {n} |\n'
text+='''
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

The fifty designs are in [predictions.json](predictions.json), their initial files in [scripts](scripts), and verbatim JSON/prose outputs and errors in [results](results). [execution.json](execution.json) records the commit, executable hash, prediction hash, exit codes, times, and single-worker setting. Predictions hash: `'''+execution['predictions_sha256']+'''`.

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

'''
for r in rows:
 text+=f"### {r['id']} {r['title']}\n\n**Prediction:** {r['prediction']}\n\n**Observed:** {r['observed']}\n\n**Disposition:** {r['disposition']}. [Original script](scripts/{r['id']}.lab).\n\n"
text+='''## Resource use and worktree consolidation

The CLI experiments ran serially to limit additional load and memory pressure. The recorded baseline includes 100 processes, one JSON and one prose run per experiment; elapsed process times sum to '''+f'{run_seconds:.1f}'+''' seconds on the shared host. These are development-build measurements, not release performance benchmarks. Larger archives were placed on `/mnt/storage`; the current build stays on `/mnt/volume1`.

Seventy registered worktrees were inspected for ancestry, changes, ignored files, and active process working directories. Only two were both clean and confirmed merged: `kero-audit-table` and the nested `a4/mix-parity` worktree. Both were removed while preserving their branches. The latter's complete files and submodule Git metadata were archived to `/mnt/storage/kerotakis-maintenance-20261002` and compared against their originals before removal. No unmerged branch was deleted and the worktrees with local changes were preserved. There are 68 registered worktrees remaining.

The inactive `kero-canonical-pose` worktree had an ignored, reproducible Rust target cache of approximately 1.1 GiB. That cache was removed; its source and branch remain. The sibling worktree folders had occupied about 7.06 GiB before cleanup. Further consolidation needs checking squash-merged branch equivalence or archiving unique work before removing those checkouts; ancestry alone does not establish that they are merged.

The worktree inventory, archive hashes, cache cleanup receipt, sizes, and resource snapshots are in `/mnt/storage/kerotakis-maintenance-20261002`. At the post-cleanup check, `/mnt/volume1` had about 5.4 GiB free and memory availability was about 2.1 GiB; these values vary with other workloads. Approximately 1.8 TiB was available on the storage mount.
'''
if (root/'performance.json').exists():
 perf=json.loads((root/'performance.json').read_text())
 text+='\nA small follow-up measured an empty script and a script with one water addition plus 100 thermometer reads, three times each. '
 for name in ('empty','water-100-measurements'):
  vals=[r for r in perf if r['script']==name]
  text+=f"`{name}` had median wall time {statistics.median(r['wall_seconds'] for r in vals):.2f} s and median peak RSS {statistics.median(r['max_rss_kib'] for r in vals)/1024:.1f} MiB. "
 text+='These six runs help separate fixed session cost from repeated observation cost, but shared load and an unoptimised executable limit conclusions. Profile a release build before committing a cache or lazy-loading optimisation. See [performance.json](performance.json).\n'
(root/'REPORT.md').write_text(text)
print('Dispositions:',dict(counts),'Baseline process time:',round(run_seconds,1),'Report bytes:',len(text.encode()))
