# Corrected finite-solid replay: bounded regression pass

The unchanged wait-cycle script ran once on PR822 head `e2a4ff70`, tested
merge `1a3bd1e2`. Its [saved raw output](stdout.jsonl) meets all eight frozen
[acceptance criteria](frozen-design.json). The [assessment](assessment.json)
records **PASS** for this bounded regression: retained Fe(OH)2 changes by
2.01e-15 mol or less, pH remains unchanged, represented iron closes within
8.5e-16 mol and no ferric inventory appears. Same-dataset comparison output
agrees within the original display-rounding tolerance.

The [assessment program](assess.py) reads the saved observations and frozen
criteria; it never launches an engine. Reproduce the assessment with
`python3 audits/target468-20261016/finite-phase-corrected-replay/assess.py`.
The original forecasts, original engine outcomes and
[first repaired replay failure](../finite-phase-first-repaired-replay/README.md)
remain unchanged. No thresholds were relaxed and no catalog entry is added.

The [producer receipt](producer-receipt.json) binds the reviewed head, tested
merge parents/tree, artifact archive, actual binary and producer lock hashes.
The [execution envelope](execution.json) binds the unchanged script and raw
streams. The raw manifest's build_image/rustc/target fields are inherited
snapshot metadata, not independently attested producer build metadata. The
binary came from the authenticated Ubuntu Actions artifact; replacement
runtime-library hashes are bound separately in the frozen design.

Execution used a bounded network-denied disposable VM with an empty account
environment. SDK destruction returned; independent deletion is not attested.
This is native CLI acceptance, not REST/deployment acceptance or complete
H/O/charge/energy closure, empirical calibration or full-fleet acceptance.
