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

Fifteen synthetic protocol tests currently cover projection/benign/consumed distinctions, missing/cutoff/wrong-phase/backend/acidity witnesses, partial/refused doses, boolean numeric fields, wrong warning metadata, duplicates/setup leaks, text mismatch and invalid inventories. These tests validate the collector's rejection logic only; first actual CLI results are now preserved and independently reviewed below.

[Prepared hosted reuse job](prepared-reuse.json) pins the already dispatched replay run38147457930 and verifies the parent GitHub artifact digest before using the same executable and lock. Dispatch only after the parent replay/build bindings are independently accepted. The preparation is not a dispatch or runtime result.

A different screened partner key or an unreviewed partner phase in the final inventory is a separate domain, not proof that the original reactant was consumed. The collector retains a failed domain witness rather than normalizing that transition. Benign counterparts also check the full source-bound reactive partner family, preventing another carbonate or metal from silently qualifying as benign.

## Independent reuse review

[The offline binding reviewer](../../../tools/acid-route-cli-review.py) is now implemented. It checks the actual completed reuse job/harness, the parent run/artifact API records and complete original ZIP digest, all non-cache parent archive bytes, and every inherited build file against a freshly recomputed parent CLI binding review. It verifies the accepted parent warning pair and recomputes the parent's hundred-case raw replay. It then rechecks the twelve route processes with the exact Git-bound archived collector and compares complete observations. Generated Python bytecode caches are excluded from the extracted-file comparison; the API digest still binds the complete original ZIP, and every source, lock, executable and raw stream remains covered.

```sh
python3 tools/acid-route-cli-review.py \
  --artifact ROUTE_ARCHIVE --run-metadata ROUTE_FINAL_RUN_JSON \
  --parent-artifact ACCEPTED_REPLAY_ARCHIVE \
  --parent-run-metadata PARENT_FINAL_RUN_JSON \
  --parent-results-review PARENT_REEVALUATED_RESULTS_JSON \
  --parent-bindings-review PARENT_BINDING_REVIEW_JSON \
  --parent-pair-review PARENT_WARNING_PAIR_REVIEW_JSON \
  --report NEW_REPORT_JSON
```

Keep reports outside the immutable artifact directories and use new filenames. The reviewer writes separate parent-replay and route-observation proofs beside the requested report. `accepted_collection_bindings` establishes collection lineage; `accepted_declared_warning_slice` additionally requires all six declared cases to qualify. Missing witnesses, mismatched warnings and consumed-partner classifications remain explicit. Neither flag establishes empirical accuracy, a new final-only exposure or integration acceptance. The public receipt omits raw vessel and runner-command records.

Six additional synthetic rejection tests cover failed/unfinished/wrong/duplicate/skipped jobs, parent run/head/artifact expiry/digest substitution, altered/missing/extra inherited files, traversal and changed archive payloads. These checks, and the fifteen collector tests, are not actual Kero route results. Apply the reviewer to the first actual reuse archive after the accepted parent replay; [Run38149577918](dispatch.json) is now dispatched once on exact390 harness after independently accepted parent replay/build/warning proofs. Runtime collection and independent reuse lineage review are now complete; the full original six-case warning slice remains unaccepted, as explained below.

The independent reviewer also checks the rendered warning severity and the exact HCl concentration/accepted-volume narration. Those checks are separate from the archived collector’s wording counts; a mismatch remains visible without altering the dispatched collector or its first outcomes.

Archived Python re-evaluation uses isolated Python with a fresh bytecode-cache namespace and bytecode writes disabled. A matching cached-bytecode header cannot substitute for the verified source; the reviewer tests this explicitly. This protects source provenance while retaining the complete original ZIP and raw observations.

The reviewer also requires the exact run ID and URL in the published dispatch, matching parent/source/case-count bindings, and successful execution of both named download and collection steps. Missing, duplicated, skipped, failed or unfinished required steps cannot qualify. A seventh reviewer test covers these substitutions; all 22 collector/reviewer tests pass. The actual queued run is rejected as incomplete by this boundary check, as recorded in [the dispatch-boundary receipt](reviewer-dispatch-boundary-20261011.json). Earlier compatibility receipts retain their historical reviewer hashes and test counts.

[First actual results](review-38149577918.json) now accept collection lineage for twelve raw processes, while the complete six-case warning slice remains unaccepted. Bleach/carbonate positive and benign pairs qualify; both positives consumed their original partner. The metal pair exposes setup-capability and unregistered-species prerequisites. [The separate metal follow-up](metal-followup/README.md) preserves these first outcomes and defines a new scoped contract rather than changing the original freeze.
