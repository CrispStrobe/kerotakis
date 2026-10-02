# Fixes following the independent 50-experiment audit

Implemented the recommendations in [the original audit](REPORT.md). The frozen
50 numerical/boundary checks improved from **46 to 49 passes** after the same
five documented ingredient substitutions used in the baseline. Zinc dissolution
remains an unmet physical expectation: the app now reports a missing rate model
instead of declaring computed inertness. This is an explicit limitation, not a
claim that zinc does not dissolve in acid.

All 50 original scripts were replayed through the final CLI in both JSON and
prose mode. Forty-five accepted their original ingredient keys; two use the
registered `sucrose` key in follow-ups and three use the registered vegetable-oil
surrogate because `olive_oil` is unavailable. All five adapted scripts succeeded.
The expectations and original scripts were retained unchanged. Existing tests
and implementation were consulted only after the independent predictions and
baseline observations were recorded.

## Implemented changes

- **Carbon accounting:** canonical recharacterisation is accepted only when it
  preserves conserved element totals, including interfaces. It may redistribute
  ions without changing those totals. A finite CO₂ dose therefore keeps its
  acidic pH and matching dissolved-carbon snapshot; later atmospheric release
  appears as a gas event. Existing addition-order invariance still passes.
- **Oil drainage:** the drain uses the same material layers as the display.
  Water and dissolved ions drain while unresolved oil stays behind, conserving
  total mass. Neutral-solute partitioning, emulsions and other unresolved
  mixtures receive an explicit refusal. Text inspection also counts the visible
  oil volume.
- **Heat:** crystalline KNO₃ dissolution now uses the reviewed +34.89456 kJ/mol
  infinite-dilution datum at 25 °C. Its stoichiometric products are included in
  the heat balance. Missing heat for other species raises a temperature-specific
  event and persistent limitation, without reclassifying all aqueous chemistry
  as missing. Inspection, thermometry, scenes, saved state, ordinary transfers,
  spill recovery and transactional commits carry that limitation. A thermostatted
  temperature remains known; the web summary marks incomplete estimates unknown.
- **Electrolysis:** gas-product explanations no longer append the metal-plating
  claim that hydrogen is absent or that an electrode must weigh less. Metal
  current-efficiency explanations remain available.
- **Kinetics:** fixed hydrogen overpotentials no longer establish a measured
  lesson-timescale absence of reaction. Zinc/lead refusals name the unresolved
  rate; near-barrier equilibrium amounts are described as ideal upper bounds.
  Related English, German and French codex content and paced steps were updated.
- **Ignition:** an unsuccessful spark trial restores the original physical
  state and settles ordinary chemistry at its actual temperature. Trial boiling,
  gas loss and reaction products are not committed. Unsupported ethanol
  combustion preserves its fuel and reports the limitation; salt flame tests
  and real combustion remain covered by regression tests.
- **Layer measurements:** pH and conductivity identify the aqueous layer and
  retain their other calibration/range notes.

The KNO₃ datum comes from V. B. Parker (1965), *Thermal Properties of Aqueous
Uni-univalent Electrolytes*, NSRDS-NBS 2, printed p.30 and table XX:
[original report](https://doi.org/10.6028/NBS.NSRDS.2).
Republished courtesy of the National Institute of Standards and Technology.
The conversion, uncertainty, conditions, rights review and pinned transcription
are recorded in [the source review](../../provenance/parker-1965-kno3-review.md).
Finite-concentration and temperature corrections are not calibrated by this datum.

## Validation

- 82 Rust tests passed across 17 relevant targets, covering chemistry, source
  promotion/re-export, addition order, persistence, ignition, drainage,
  electrolysis and codex export. Later inspection and localization changes also
  passed the targeted regression checks.
- 29 web result-summary tests passed; the Svelte/TypeScript check found zero
  errors (two existing warnings).
- Provenance lint, the per-field numerical promotion gate, dependency checks,
  engine locale coverage, placeholder checks and paced-step validation passed.
- 100 final CLI processes replayed the original 50 designs. Fourteen follow-up
  processes covered the five input adaptations plus carbon consistency and
  persistent heat disclosure. Supplementary assertions check zinc's explicit
  boundary, electrolysis prose, ignition rollback and aqueous-layer scope.

No validated zinc rate law, missing ethanol combustion route or missing copper
spectrum was invented to make the audit pass. No runtime speedup is claimed:
shared-host development-build timings are unsuitable for attributing performance
changes. A release profile remains the appropriate next step for speed work.

## Systematic follow-up

These regressions expose broader gaps in solver contracts, observable validity,
kinetics, transaction boundaries, material capabilities and presentation. The
local fixes do not close those classes throughout the engine.
[Systematic follow-up](SYSTEMATIC.md) records evidence, remaining work and closure
criteria for each area.

## Original checks after fixes

| Case | Check | Before | After |
|---|---|---|---|
| 01 | 10 kJ raises 100 g water about 24 K | Pass | Pass |
| 02 | Equal heat and cool restore initial temperature | Pass | Pass |
| 03 | Weighted hot/cold mixing gives 35 C | Pass | Pass |
| 04 | Temperature rises scale inversely with water mass | Pass | Pass |
| 05 | Cooling makes mixed ice and liquid near zero | Pass | Pass |
| 06 | Boiling plateau and approximately 17 g water loss | Pass | Pass |
| 07 | Salt freezes lower than equal molar sugar | Pass | Pass |
| 08 | Equal 60 C portions stay at 60 C | Pass | Pass |
| 09 | Tenfold dilution raises pH about one unit | Pass | Pass |
| 10 | Neutralisation followed by alkaline overshoot | Pass | Pass |
| 11 | Acetic acid pH is more than one unit above HCl | Pass | Pass |
| 12 | Half neutralised acetate pH near pKa | Pass | Pass |
| 13 | Buffer pH change is less than 1 percent of water change | Pass | Pass |
| 14 | NaCl greatly increases conductivity | Pass | Pass |
| 15 | Sugar conducts much less than NaCl | Pass | Pass |
| 16 | AgCl yield close to 1 mmol | Pass | Pass |
| 17 | Reversed reagent order gives equal solid and pH | Pass | Pass |
| 18 | Filter retains AgCl and transfers water and sodium | Pass | Pass |
| 19 | BaSO4 yield near 1 mmol | Pass | Pass |
| 20 | Acid dissolves chalk with accounted CO2 | Pass | Pass |
| 21 | Excess salt remains undissolved | Pass | Pass |
| 22 | Dilution dissolves residual NaCl | Pass | Pass |
| 23 | Evaporation retains salt with 10 g solvent | Pass | Pass |
| 24 | Quarter decant partitions water and salt | Pass | Pass |
| 25 | Split and reunite restores inventory and temperature | Pass | Pass |
| 26 | CaCl2 warms and KNO3 cools | Fail | Pass |
| 27 | Inventory carbon agrees with speciation snapshot | Fail | Pass |
| 28 | Fizz raises sealed pressure | Pass | Pass |
| 29 | Smaller headspace gives larger pressure | Pass | Pass |
| 30 | Opening restores ambient pressure and retains salt | Pass | Pass |
| 31 | Pressure regulator expands headspace at fixed pressure | Pass | Pass |
| 32 | Sweep removes carbon and raises pH | Pass | Pass |
| 33 | Zinc plates copper with conserved metal totals | Pass | Pass |
| 34 | Copper does not plate zinc | Pass | Pass |
| 35 | Zinc dissolves in acid while copper remains | Fail | Unmet; explicit rate boundary |
| 36 | Cell reports about 1.1 V with stable electrode identities | Pass | Pass |
| 37 | Equal cells have no potential then dilution produces tens of mV | Pass | Pass |
| 38 | Faraday product amounts and 2 to 1 gas stoichiometry | Pass | Pass |
| 39 | Missing complete Cu spectrum is explicitly bounded | Pass | Pass |
| 40 | Tenfold dilution reduces absorbance tenfold | Pass | Pass |
| 41 | Flame test retains NaCl | Pass | Pass |
| 42 | Burned magnesium gains expected oxygen mass | Pass | Pass |
| 43 | Ethanol combustion limitation is explicit | Pass | Pass |
| 44 | Distillation enriches ethanol and conserves both components | Pass | Pass |
| 45 | Oil and water are drawn as separate conserved layers | Pass | Pass |
| 46 | Drain transfers lower water layer | Fail | Pass |
| 47 | Salt remains aqueous alongside unresolved oil | Pass | Pass |
| 48 | Milk acidification conserves unresolved material and produces curd event | Pass | Pass |
| 49 | Peroxide oxygen stoichiometry and unchanged catalyst | Pass | Pass |
| 50 | One hour preserves dilute salt and water inventory | Pass | Pass |

## Reproduction and resources

Build with `RUSTC_WRAPPER= cargo build -p kerotakis-cli -j 1`, then run:

```sh
KERO_WORKERS=1 KERO_RESULTS_DIR="$PWD/audits/independent-50-20261002/after" python3 audits/independent-50-20261002/run.py
python3 audits/independent-50-20261002/rerun_followups.py
python3 audits/independent-50-20261002/verify_fixes.py
```

The original evidence is in `results/` and `execution.json`; final evidence is in
`after/`. Intermediate runs are kept separately. Raw stdout/stderr are retained.
Final binary SHA-256: `a4821caad931edafbe8b3af5176c06eadfe7b573fae495f4b4c832d12cc8dad4`.
Frozen prediction SHA-256: `0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.

Builds used one job and CLI replays one worker. Scientific PDFs, logs, receipts
and maintenance archives are under `/mnt/storage/kerotakis-maintenance-20261002/`;
active build libraries and the CLI remain on `/mnt/volume1`.
The earlier cleanup removed two verified merged, clean worktrees, retained their
branches and archived the submodule-bearing tree and metadata. It also removed
an inactive reproducible target cache. Dirty and unmerged worktrees were retained;
68 registered worktrees remain. See the maintenance receipts for paths and hashes.

After validation, 17 ignored, reproducible integration-test executables generated
by this audit were removed, recovering 911.8 MiB. Compiler libraries and the final
CLI were retained. `final-test-cache-cleanup.json` in the maintenance directory
records their paths, sizes and hashes; validation logs retain the passing results.
