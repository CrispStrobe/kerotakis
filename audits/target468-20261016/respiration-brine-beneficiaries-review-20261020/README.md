# Respiration and brine: actual production saved-response review

This review authenticates the two unchanged original scripts issued once against accepted source `dacf76513328c2226e2f5133a79fb9af84343431`, binary `f6aef37ddadfa0f1163e2aff9f436ec560eae767d6d86203bfa850c04f0c9955`, and actual producer lock `41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb`. It performs no engine execution, request, retry or automatic scientific selection. The original historical fifth-review failures remain unchanged.

The reader requires an evidence bundle supplied through arguments. It authenticates the frozen design, once-only issuance, runner bytes, completion and HTTP response receipts, requests, original intake forecasts/checks/scripts, accepted production/artifact receipts, exact source tree and thermodynamic tables. Actual API observations are the top-level `observations` list. The earlier parent reader's incorrect schema assumption and its recorded KeyError remain preserved separately; that call-site error is not a physical failure.

```sh
python3 audits/target468-20261016/respiration-brine-beneficiaries-review-20261020/read_responses.py \
  --repo "$REPOSITORY" --design "$EVIDENCE/design.json" \
  --results-dir "$EVIDENCE/results" \
  --artifact-receipt "$REPOSITORY/audits/target468-20261016/partition-release-acceptance-20261020/artifact-acceptance.json" \
  --production-receipt "$REPOSITORY/audits/target468-20261016/partition-release-acceptance-20261020/production-acceptance.json" \
  --out "$NEW_ASSESSMENT"
```

`--out` must be a new file. All quantities are computed from saved observations and the exact delivered source. Atom/charge ledgers cover every observed step and vessel, filter ownership, ideal trapped air, boundary exports, carrier water and otherwise omitted material reservoirs. Inspect/measure snapshots preserve the unchanged other vessel and clock. Diagnostic solution species do not duplicate canonical material inventory.

| Original | Independent proposal | Evidence and limits |
| --- | --- | --- |
| MO-005 | Pass, parent independently reproduced and accepted | Consumed glucose 0.0001 mol produces 0.0006 mol inorganic carbon and consumes 0.0006 mol O2. Actual initial oxygen includes 0.002140778 mol trapped-air oxygen beyond the explicit 0.001 mol. All donor N2/O2/CO2 remain through filtration. Retained carbonate is 0.00060292448 mol CaCO3; remaining calcium is aqueous, not hidden Ca(OH)2 solid. Total carbon includes 0.00000408740445 mol trapped-air carbon. No isotope assignment distinguishes individual air/glucose carbon atoms. This qualifies the requested stoichiometric template and carbon budget, not biological respiration, rate calibration or complete carbonate capture. |
| TC-008 | Hold | Ice decreases from 1.66389351 to 1.43374923 mol while temperature drops from 273.15 to 260.60581049 K. Actual declared dissolution consumes 331.94736752 J; no solid NaCl remains and recorded excess-enthalpy change is zero. Independent phase-specific water sensible-plus-fusion energy decreases by 568.33597629 J, leaving **-236.38860877 J unclosed**. Signs alone do not satisfy the original coupled dissolution/enthalpy criterion. |

MO-005 maximum material residual is 5.505e-13 mol-equivalent. TC-008 uses delivered phase-specific Cp tables, water fusion 6010 J/mol, incoming salt sensible energy and the declared salt dissolution enthalpy. Liquid-water Cp clamps below its table endpoint in this model; this calculation does not assert calibrated nonideal subzero brine calorimetry. Missing enthalpy/domain capability remains a hold, rather than an invented compensating heat term. Neither case is selected by this reader.

Parent independently reproduced the complete saved reader output and inspected the original MO-005 criteria, source domain, gas/air ownership and budgets. The separate `parent-adjudication.json` accepts MO-005 only. The safe bounded evidence bundle is in `evidence/`, authenticated by `evidence-manifest.json`; it includes both equal independent assessments and the preserved first parent schema error. The successor selection records MO-005 without altering the historical fifth review or TC-008 hold.
