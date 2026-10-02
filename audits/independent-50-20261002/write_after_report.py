"""Publish reviewable fixes and evidence without changing the frozen audit."""
import hashlib, json, re
from pathlib import Path
root=Path(__file__).resolve().parent
archive=Path('/mnt/storage/kerotakis-maintenance-20261002')
checks=json.loads((root/'after/checks.json').read_text())
before={c['id']:c for c in json.loads((root/'checks.json').read_text())}
execution=json.loads((root/'after/execution.json').read_text())
followups=json.loads((root/'after/followups-execution.json').read_text())
assert execution['binary_sha256']==followups['binary_sha256']
assert execution['predictions_sha256']=='0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064'
assert sum(c['passed'] for c in checks)==49
assert next(c for c in checks if c['id']=='35')['passed'] is False
cases=json.loads((root/'predictions.json').read_text())
text=f'''# Fixes following the independent 50-experiment audit

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
'''
for c in checks:
    status='Pass' if c['passed'] else 'Unmet; explicit rate boundary'
    text+=f"| {c['id']} | {c['check']} | {'Pass' if before[c['id']]['passed'] else 'Fail'} | {status} |\n"
text+=f'''
## Reproduction and resources

Build with `RUSTC_WRAPPER= cargo build -p kerotakis-cli -j 1`, then run:

```sh
KERO_WORKERS=1 KERO_RESULTS_DIR="$PWD/audits/independent-50-20261002/after" python3 audits/independent-50-20261002/run.py
python3 audits/independent-50-20261002/rerun_followups.py
python3 audits/independent-50-20261002/verify_fixes.py
```

The original evidence is in `results/` and `execution.json`; final evidence is in
`after/`. Intermediate runs are kept separately. Raw stdout/stderr are retained.
Final binary SHA-256: `{execution['binary_sha256']}`.
Frozen prediction SHA-256: `{execution['predictions_sha256']}`.

Builds used one job and CLI replays one worker. Scientific PDFs, logs, receipts
and maintenance archives are under `/mnt/storage/kerotakis-maintenance-20261002/`;
active build libraries and the CLI remain on `/mnt/volume1`.
The earlier cleanup removed two verified merged, clean worktrees, retained their
branches and archived the submodule-bearing tree and metadata. It also removed
an inactive reproducible target cache. Dirty and unmerged worktrees were retained;
68 registered worktrees remain. See the maintenance receipts for paths and hashes.
'''
(root/'AFTER.md').write_text(text)
print('Written AFTER.md; 49 passing original checks and one explicit kinetic boundary')
