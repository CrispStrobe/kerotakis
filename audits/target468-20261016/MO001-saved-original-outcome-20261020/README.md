# Original MO001: requested esterification and residual-acid assay

Both unchanged original criteria are parent-accepted within the explicitly requested ideal-mixture teaching model. The [saved assessment](assessment.json) and [independent parent reproduction](parent-assessment.json) are identical; [parent adjudication](parent-adjudication.json) supplies acceptance without changing the original proposal record.

The original requested reaction forms 0.6666666667 mmol ester. Its inventory remains exact through dilution and titration. Sample and blank use 0.3336279297 and 1.0002294922 mmol hydroxide; their difference agrees with the original conversion within the 10 µmol dosing increment. Independent acid-protonation and retained-base corrections recover 0.666666666666636 mmol, within 3.01×10⁻¹⁷ mol of the original ester. All 13 captured commands and 19 vessel states close cumulative atoms, charge, phases, counterions and titrant/neutralisation water. Maximum atom residual is 9.25×10⁻¹⁴ mol; every clock remains zero.

This qualifies the original conditional, requested-only assay. It does not predict physical hydrolysis stability, reaction rates, catalyst sufficiency, empirical assay accuracy or calorimetry. The initial organic mixture honestly has uncharacterised ionic speciation; the actual endpoint assay follows water addition. Internal root-finding trial states are not archived: source cloning and committed dose/carrier budgets are checked without inventing intermediate observations.

[Raw records](raw/) preserve the complete original script, 13 JSONL observations, stderr and exact archived producer bindings. Source **c4469205**, binary **670bd3d8** and lock **82f3196f** are historical bindings. **Binary and lock are producer-identified only: the artifact contains neither executable nor Cargo.lock bytes, so this review cannot independently rehash either.** There was no newer-release replay. The authenticated original archive remains at [its existing repository location](../../new-experiments-201-20261016/native-screen-results/raw-artifact.zip).

The [portable reader](review_saved.py) reauthenticates that archive, originals and pinned source, and independently reconstructs the endpoint and complete owned-material budgets. [Pinned source excerpts](source-quotes.json) document the model and timing authority. The [first reader failure](first-reader-source-span-failure.json) and its exact source are preserved: an overlong source-method slice included a later clock helper; the corrected method boundary retains every no-clock assertion. No original criterion or tolerance was weakened.

Run the complete saved-only selection verifier from the repository root:

```sh
python3 audits/target468-20261016/selection-MO001-20261020/verify_catalog.py
```
