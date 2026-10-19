# Two unchanged hydrolysis beneficiaries: one pass, one coverage limit

Both frozen originals were issued once on accepted source `8c584e20`, binary `13a4d134`, actual producer lock `66886813`. Raw responses and [before-issue design](design.json) retain original commands, forecasts and criteria. No baseline/corpus/OR006 rerun occurred.

**MO006 back-titration passes both original criteria.** Actual sample/blank HCl doses are0.0004997076416/0.0009996099854mol, independently matched to chloride. Their difference is0.00049990234375mol versus0.0005mol ester loss, within the original1e-5mol increment. Actual protonated acetate8.1682094e-8mol plus residual-base-coordinate difference1.5974156e-8mol explains the0.01953125% bias. The corrected difference is0.000499999999999978mol. Every-step C/H/O/Na/Cl/charge closes within1.733e-14mol/eq, including all titrant carrier water. Actual endpoint temperatures298.315276K/298.368813K remain explicit; no empirical assay/rate/heat calibration is claimed.

**MO011 remains unsupported.** Both successive0.5mmol conversions conserve matter and produce matched ethanol/acetate. But the column has no ester retention/group parameterisation, so there is no initial ester peak to track. Later ethanol peaks and area1.0 do not meet the original ester-feature criterion; ionic acetate remains explicitly outside this neutral method. Do not select the experiment or treat relative area as molar yield.

Reproduce the independent45-digit saved review and catalog membership, without engine or HTTP calls:

```console
python3 audits/target468-20261016/hydrolysis-beneficiaries-outcome-20261019/review_saved.py --out hydrolysis-beneficiaries-review.json
python3 audits/target468-20261016/hydrolysis-beneficiaries-outcome-20261019/verify_catalog.py
```

[Selection](selection.json) preserves all247 previous entries and adds only MO006:248 total /244 unique fixed468, leaving157 to401. The source-level chromatography follow-up needs licensed/attributed neutral retention or group/interaction data, valid molecular decompositions and actual estimator controls; an alias or hand-assigned peak would hide the gap. Keep baseline and this failed/unsupported observation immutable while binding any model extension separately.
