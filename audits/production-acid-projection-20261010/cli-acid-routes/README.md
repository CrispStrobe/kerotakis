# Frozen actual acid-route probes

[The freeze](freeze.json) declares six source-informed virtual engineering probes before CLI parsing or execution: bleach, metal and carbonate positives with their benign counterparts. These are additional targeted probes, separate from the unchanged 100-case replay and original chloramine holdout. They are not independent chemistry forecasts.

Each script starts with 100 mL water and one partner, then accepts one 10 mL dose of 1 M HCl. Target pH0 is a stopping bound chosen to avoid endpoint refinement; it is not a predicted pH. Parsed inputs, the accepted 0.01 mol dose, inventory amounts/phases, charge ledger, solved pH and actual PHREEQC provenance must verify. Missing partner, insufficient final acidity, refusal, fallback or partial dose is a non-exercising case, never a passing warning proof.

A qualifying positive must publish exactly one primary warning with the reviewed metadata in JSON and text. Named HCl is present in the prospective dose; its solved H+ reporting must not duplicate that warning. A benign counterpart must have no primary partner and no primary warning. No setup primary warning is expected.

Metal and carbonate can be consumed by the dose. The collector explicitly separates a surviving partner from a consumed partner: a consumed partner can verify prospective warning narration, but cannot prove final surviving-exposure detection. A surviving partner with solved strong acidity can exercise the directional reporting policy. None of these protocols establish detection of intermediate products, new final exposure absent from prospective screening, or empirical chemical accuracy.

Use the independently accepted, source-bound candidate CLI archive after the prepared replay succeeds. Reuse its executable, generated lock and build binding; do not build another executable or substitute an unbound local binary. Require the source manifest to match that binding. Under the resource gate, the portable collector can run as follows:

```sh
python3 tools/acid-route-cli-probes.py \
  --binary CLI_ARCHIVE/combined-build/kero \
  --build-binding CLI_ARCHIVE/combined-build/binding.json \
  --source-manifest audits/production-acid-projection-20261010/cli-source-dbe86521.json \
  --out acid-route-observations --report acid-route-collection.json
```

The output directory must be a plain portable name. It archives twelve processes, exact copied scripts, complete stdout/stderr and hashes. Preserve the first outputs even if parsing or witnesses fail. Recheck the archive without executing Kero using `--review`, the same binding/manifest and a new report filename; the review output directory may be beneath an artifact directory. Collection always leaves `accepted_repair_proof` false.

Independent acceptance must bind the prior accepted executable bytes, generated lock, source/tree/submodules/toolchain, collector/freeze/scripts and all raw processes; rerun the archived evaluator and compare complete observations. Verify source-bound provenance and ordinary text separately. Report each protocol's category and failed witnesses, including consumed partners. Do not change the freeze, thresholds, doses or scripts in response to first outcomes. An author/schema prerequisite needs a separately named correction and retained original evidence.

Five synthetic protocol tests currently cover projection/benign/consumed distinctions, missing/cutoff/wrong-phase/backend/acidity witnesses, partial/refused doses, boolean numeric fields, wrong warning metadata, duplicates/setup leaks, text mismatch and invalid inventories. These tests validate the collector's rejection logic only; actual CLI parsing and route results remain pending.
