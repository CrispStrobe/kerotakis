# Saved kinetics: detailed review preparation

These14 records are a deeper extraction from the existing
[saved64 first screen](../saved64-first-screen/README.md), not14 new experiments
or selections. The catalog remains200 / fixed468 selected196. Original scripts,
forecasts, source/binary bindings and raw first outcomes are unchanged.

[Metrics](metrics.json) retain actual per-vessel times, temperatures, substrate,
catalyst and product amounts, acid/scavenger sampling and all explicit model
limits. The independent material projection remains linked by its original raw
archive hash. [Supplemental calculations](supplement-metrics-before-assessment.json)
include the actual additive liquid volumes for the substrate-saturation comparison.

A source calculation also establishes the stirring correlation's bound: for
finite nonnegative tip speed, `1 + gain * v / (half + v)` lies between1 and2
with the recorded gain1 and positive half-speed0.1m/s. This is a declared teaching
correlation, not empirical mixing calibration or a bound inferred from two plots.

Run the read-only reproduction against the original archive and repository inputs:

```console
python3 audits/target468-20261016/saved64-kinetics-preparation/verify_metrics.py
```

All14 numerical/source records reproduce exactly. This does not assign scientific
passes. Complete each original criterion and source-domain review in
[the selection lane](../../../docs/saved64-review-lanes-20261018.md#a-finalize-bounded-selections-from-existing-evidence),
retaining the catalyst-filtration oxygen failure, incomplete clock sampling,
withheld weak-acid kinetics and acid-reserve qualifications. An unsupported zero
rate is not a physical comparison; lower final iodine is not a measured clock delay.
