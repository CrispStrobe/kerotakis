# 100 diverse additional Kero experiment candidates — batch 3

**100 new candidate designs, not 100 verified working experiments.** This batch follows the previously generated 120 and 101 candidates, bringing the generated total to **321**. The user's separate upstream 147-case catalogue is not included in that total.

The target is Kero at commit `159fa6e08fa4ebea3105ac7e17d5b8350bc91189`, matching the user-supplied catalogue. The JSON contains complete `.lab` scripts; the wrapper JSON itself is not a Kero script. All records were created on **2026-10-16**.

## Files

| File | Purpose |
|---|---|
| `new-experiments-diverse-100-20261016.json` | Complete standalone dataset, source registry, input catalogue and 100 rich experiment objects. |
| `scripts/*.lab` | The same 100 scripts, ready to submit individually to a compatible Kero installation. |
| `experiments.jsonl` | One experiment object per line; shared source/catalogue metadata remains in the main JSON. |
| `experiment_index.csv` | Compact ranked index with English/German titles, questions, priorities and risk labels. |
| `dataset.schema.json` | Local JSON Schema, Draft 2020-12; not an official Kero schema. |
| `validation_report.json` | Actual offline checks, including per-script results. |
| `novelty_comparison.json` | Reproducible comparisons against both complete previous generated batches. |
| `validate_dataset.py` | Independent offline grammar-subset, integrity and duplicate checker. |
| `run_experiments.py` | Fresh-process runner that preserves raw output without claiming chemical acceptance. |
| `build_dataset.py`, `finalize_package.py` | Authoring source and schema/validation finalisation. |
| `reference/previous-120.json`, `reference/previous-101.json` | Unmodified local baselines for reproducing the duplicate checks. |
| `SHA256SUMS.txt` | SHA-256 manifest for the delivered package. |

## Breadth and selection

The batch uses **100 distinct explicit `add` input keys**, including **71 not explicitly added in either previous generated batch**. **88 of the 100 designs** use at least one of those new keys. Keys include molecular species, minerals, polymers, enzyme-family proxies, prepared formulations and material recipes; these are **not 100 pure substances**.

| Code | Theme | Count |
|---|---|---:|
| EN | Enzymes, digestion and substrate recognition | 10 |
| FE | Fermentation and coupled bioprocesses | 8 |
| PO | Polymers and macromolecular materials | 10 |
| CO | Colloids, foams and interfaces | 8 |
| LI | Light, colour and chemical emission | 8 |
| AD | Adsorption and water treatment | 6 |
| TR | Transport, tracers and reaction fronts | 8 |
| MI | Minerals, hydrates and geological chemistry | 8 |
| SO | Solvents and organic reaction selectivity | 6 |
| AN | Analytical fingerprints and diagnostic controls | 8 |
| SU | Corrosion, surface protection and electronic solids | 6 |
| TH | Thermal and phase phenomena | 6 |
| FO | Everyday formulations and ingredient roles | 4 |
| BD | Explicit model-boundary investigations | 4 |
| | **Total** | **100** |

Examples include enzyme substrate specificity, alcoholic versus lactic fermentation, PVA/borate gel state, thermoplastic versus thermoset behavior, foam versus total gas generation, plant-pigment tests, prepared-luminol emission, charcoal adsorption and desorption, reaction fronts through mineral beds, hydrate versus anhydrous-salt comparisons, organic substitution/elimination selectivity, galvanic protection, graphite versus diamond, and ingredient-specific browning or leavening controls.

Newness includes substantive controls and interpretation questions that reuse known chemistry. It does not mean that every design is an unrelated reaction family. Some designs investigate physical transformations, materials, numerical behavior or measurement coverage rather than a net chemical reaction.

### Ranking

The `experiments` array is sorted by an **editorial educational priority within this batch**, then by theme and stable code. `rank` runs from 1 to 100. `priority_tier` is 1 for broadly useful introductory concepts, 2 for upper-secondary/enrichment topics, and 3 for advanced or model-literacy extensions. The stable `code` remains thematic, so it need not match rank order. There are no measured encounter probabilities or claims of compulsory coverage in all German Länder.

## Record structure and interpretation

Each experiment includes stable identifiers; English and German titles; an experimental question; a complete script; structured additions with input key, identity type, formula when appropriate, amount, unit, vessel and source line; operations and observation commands; expected products or states; expected processes/equations where useful; controls; expected result; individual limitations; educational priority; source references; four script fingerprints; and acceptance criteria left unevaluated.

`expected_result` is a design hypothesis or a documented model-boundary expectation, **never a substituted observation**. Every record retains:

```json
{
  "status": "designed_not_run",
  "observed_result": null,
  "validated_on": null
}
```

All acceptance `result` fields are `null`, as are evidence `assessment` and `tested_script`. `candidate_script` points to the supplied `.lab` file. In `engine_support`, both `runtime_verified` and `official_parser_verified` remain `false`.

Coverage risk is **45 medium / 55 high**. These are editorial integration-risk labels, not estimated success probabilities. There are 90 controlled comparisons, 2 characterisations, 3 representation audits, 1 numerical-method comparison, 3 known-model-boundary probes, and 1 unverified-model-coupling probe.

### Important representation distinctions

* A named food, paint, culture or polymer formulation can carry aggregate or unresolved material state. Enzyme families and formulations do not receive invented exact molecular formulas. Polymers use a separate `repeat_unit_formula` when applicable. Material recipes are dispensed by mass in this batch.
* Formula/equation types are explicit. **No automated atom-and-charge-balance validation was performed for this batch.** Schematic families, hydrolysis of unresolved material pools and physical changes must not be read as exact molecular mechanisms.
* Temperature inputs after `@` describe an addition; they are not a thermostat. `heat` and `cool` add/remove energy rather than specifying a target temperature. Measurements must verify the state reached.
* `wait` advances the shared session clock and all live vessels. A sealed vessel is not automatically evacuated or oxygen-free.
* The transport inlet is a repeatedly sampled composition boundary, not a depleted finite reservoir. Transport designs require the engine's equal solvent-volume assumptions to remain valid after chemistry; that coupling is unverified.
* The charcoal/methyl-orange model, PVA/borate network, superabsorbent retained-water state, sunscreen bands, prepared-luminol emission and selected biological/material responses are bounded teaching models. Many numerical parameters are editorial or approximate, not independently calibrated measurements.
* The pinned nonaqueous-solubility module explicitly records withdrawn provenance for retained numeric data. SO-005 therefore treats that route as a model interrogation, not a source of validated quantitative solubility data.
* The four BD designs deliberately distinguish absent salinity dependence, unwired interfacial-rate coupling, unverified photochemical integration, and incomplete chromatographic coverage from evidence about real-world chemistry. A missing result is not a measured zero; a missing chromatographic peak is not proof of absence.

## Actual validation performed

All 100 records pass the delivered Draft 2020-12 JSON Schema and the custom offline checker. It checked **1,650 command lines across 19 verbs**, vessel references, the local input catalogue, structured-addition/script consistency, source IDs, script hashes, candidate-status consistency, standalone file equality and JSONL equality. The parser source was used to define a **bounded custom grammar subset**, not to execute Kero's official parser.

Duplicate comparisons cover **all 221 entries** from the two complete previous generated JSON files and the new batch itself. There were no matches under any of these fingerprints:

1. Full script text.
2. Comments removed and whitespace normalised.
3. Observation-only commands removed and vessel labels normalised in first-occurrence order.
4. The same action protocol with numeric-leading amounts, temperatures, times, fractions and step counts additionally masked, while preserving units and chemical/input keys.

The fourth test rejects simple quantity-only variants with otherwise identical protocols. These tests do **not** prove semantic independence or correct chemistry. The upstream 147-case catalogue was reviewed through selected excerpts, **not exhaustively downloaded and automatically compared**.

No Kero executable was found in the validation environment. **No official-parser execution, chemistry-engine execution, numerical convergence test or chemical acceptance assessment was performed.** The runner's listing mode was checked without executing Kero. A successful engine exit in a future run would still require substantive review of warnings, inventories, observables and the acceptance criteria.

## Source scope

The main JSON contains **33 source records** and pinned URLs. It points to the user-supplied working catalogue, script parser, registry, relevant implementation modules and both local previous datasets. These are implementation references, not fabricated evidence of successful runs.

Selected source excerpts were reviewed for the parser and registry identities/recipes, enzyme activity and hydrolysis representation, fermentation, gel and swelling, plastics, species/material representation, emulsion/foam/curdling, starch–iodine optics, chemiluminescence, sunscreen attenuation, opaque pigments, adsorption, transport, nonaqueous solubility, substitution/elimination selectivity, chromatography, corrosion, kitchen-biology models, kinetics and photochemical primitives. Instrument, conductivity and surface-spread interfaces were also inspected selectively. Some linked modules, including curated chemistry, serve as broader integration context; this is **not** an exhaustive audit of every line or every solver call.

Input-key evidence is recorded per catalogue entry with a registry identity/recipe locator or the dynamic enzyme-family source. Runtime input lookup remains unverified. The original upstream catalogue's observed results are not copied into any new experiment's observations.

## Run and review

From the extracted package directory, submit one script to a compatible installation:

```bash
kero run scripts/D3-EN-001.lab --json
```

The runner requires Python 3.10 or later and `jsonschema` for the schema preflight. Install the validation dependency in your own environment, then list or run selected designs:

```bash
python -m pip install -r requirements-validation.txt
python validate_dataset.py --schema dataset.schema.json --write-reports
python run_experiments.py --list
python run_experiments.py --id batch3-diverse:en-001 --kero /path/to/kero --out runs
python run_experiments.py --group polymers-and-macromolecules --kero /path/to/kero --out runs
```

Without a selection, the runner attempts all 100 in separate processes. It saves each exact submitted script, raw stdout/stderr, process exit status, executable hash, expected result and unevaluated acceptance checks in a new timestamped output directory. It never overwrites the candidate observations and never treats exit code zero as a chemistry pass. The optional `--reported-engine-commit` is explicitly user-reported, not independently verified.

To regenerate the authored files after editing designs:

```bash
python build_dataset.py
python finalize_package.py
```

Regeneration resets the top-level validation summary before rerunning offline checks. Recreate the checksum manifest after making local changes. The delivered schema describes this candidate package; adding evaluated results or changing the record count requires a deliberate schema/version change rather than silently altering the unrun status contract.

## Safety and use

These are **virtual chemistry designs**, not wet-lab operating procedures, risk assessments, food-safety instructions or permission for unsupervised handling. Simulator quantities and model temperatures should not be transcribed into classroom procedures without an independent professional experimental and safety assessment. Several inputs and processes require controlled laboratory conditions in the real world. No Kero executable, font files or upstream source-code snapshot is bundled.
