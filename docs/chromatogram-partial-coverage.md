# Report missing neutral parameters beside supported chromatogram peaks

The [first smaller native control](../audits/target468-20261016/chromatogram-partial-first-control/)
uses20mL water,0.0003mol ethanol and0.0007mol ethyl acetate. Accepted source
`5194f961` reports an ethanol peak and an empty `outside_method`, but no warning
for the present ethyl acetate whose retention model is missing. The source of
the instrument branch, its parameters and registry matches main `e63a0824`.
Raw script, forecasts, output, exact binary/producer-lock binding and the first
failed assessment are retained; this is not a combined-main deployment claim.

The missing-model warning was conditional on there being no supported solutes.
The prepared repair reports it whenever neutral parameter sets are missing,
including when supported peaks are present. It retains the distinction between
ions outside the method and neutral solutes missing model parameters. A mixture
of ions and unsupported neutrals gets both reports; an entirely unsupported
neutral sample gets a warning, without an invented empty chromatogram or a
false claim that no solute was present. Supported peaks remain unchanged.

Core controls exercise partial support, wholly unsupported samples and mixed
ionic/model exclusions with unchanged owned sample contents. A native aqueous
control uses real dissolved glucose and ethyl acetate beside ethanol, verifies
both missing-parameter names and checks that the measurement consumes neither.
No original test, physical forecast, lesson, golden, coefficient or group
parameter changes. The existing localized warning is reused.

## Pending acceptance and model scope

Formatting and diff/source review pass. Actual repaired native/core tests on
both platforms, browser/serialization checks and the full original own fleet
remain required; keep the PR draft until accepted on its exact reviewed head.
Integrate after the prior delivery checkpoint through normal protected
exact-head auto-merge. Never merge broad audit history or waive a failed check.

This closes a reporting omission, not the missing retention data. SP012 remains
held for that coverage limit. Add differentiated peaks only after a reviewed,
compatible parameter/group model is available; do not invent coefficients or
label a warning as a scientific pass. The [analytical lane](https://github.com/CrispStrobe/kerotakis/blob/6200d78934d13fdd2e310216b7872c899fa1bc18/docs/saved64-review-lanes-20261018.md#e-repair-analytical-coverage-reporting-and-staged-separation)
retains the separately scoped higher-stage distillation gap.

## First native-control correction

The first macOS run at `7b5f62d1` reported the ethyl-acetate gap but not glucose.
The new control had used PHREEQC alone; glucose remained a solid outside the
injection. The standard stack's preceding `MixingEquilibrator` owns reviewed
neutral-solid dissolution. The corrected control includes that real solver
before the native aqueous engine and explicitly requires all glucose to be
dissolved before injection. Its original missing-parameter, supported-peak,
ionic-exclusion and inventory assertions remain unchanged. No solubility or
retention parameter was invented, and the runtime repair is unchanged.

The [original failed native job](https://github.com/CrispStrobe/kerotakis/actions/runs/39119814637/job/117652941834)
is preserved. Formatting passes; corrected-source execution remains required.
A separately frozen CPU-worker job still tests the original source and must
not be relabelled as acceptance of this correction.
