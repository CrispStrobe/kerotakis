# SP-012: supported peaks and an explicit model gap

The unchanged original earns **2/2 criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`, tree `3e1471c481b9ef6f91a962039e3158ab0df2763d`. Coverage becomes **426/468 originals, 42 remaining**. All 429 predecessor catalog objects remain exact.

The actual chromatogram identifies ethanol at `68.25333021555525 s` and ethyl acetate at `902.5415014011661 s`. Glucose is present as `0.001 mol` aqueous stock, but its missing curated group decomposition is explicitly reported. It has no invented peak, retention value or measured absence. The original C1 specifically requires supported identified peaks and distinguishing missing parameters from missing compound; it does not require a parameterised glucose prediction.

C2 passes with exact equality of the entire bench before measurement, after measurement and after the following inspection. All seven independently reconstructed C/H/O/charge budgets close with zero residual under the unchanged `1e-10` bound. There are no solver failures, reactions or gas transfers. Physical contents are counted once; lot records and mirrored solution species are not additional stock.

The pinned column relations independently reproduce retention, width and Rf from the actual reported partition coefficients. The equal `0.001 mol` supported inputs each receive relative area `1`, because the implementation scales to the largest supported molar amount. Their areas sum to two and their registry-derived masses differ; neither value is a calibrated mass fraction. The glucose model gap remains visible and outside-method is empty: an unparameterised neutral is not classified as an ion outside this column method.

Ten saved-output mutations reject lost ethanol/glucose, a hidden gap, an invented glucose peak, a missing supported peak, measurement-induced clock changes, mass-fraction-style areas, wrong retention, wrong outside-method classification and a solver failure. No engine replay, build, chemistry source change or criterion relaxation was used.

Qualification covers the original supported-peak/explicit-gap comparison and non-consuming analysis. It supplies no glucose retention prediction, complete sample coverage, empirical partition/retention or detector-response calibration. The earlier broad scientific-model hold remains a limit on glucose prediction, not a failure of the original explicit-gap inspection criteria. No new chemistry is declared supported by counting this comparison.

Original script, forecast, criteria, selected producer hashes, genuine historical lock and pinned source excerpts are preserved. The historical producer did not record an executable checksum; it remains null with no later binary substituted. PR882/883 acceptance and selector/floor controls remain separate from this historical qualification. A zero-match selector filter earns no control credit.

```console
python audits/sp012-saved-method-coverage-20261114/review_original.py audits/sp012-saved-method-coverage-20261114
python audits/sp012-saved-method-coverage-20261114/check_reader_negatives.py audits/sp012-saved-method-coverage-20261114
```
