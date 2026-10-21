# Liquidus and fusion-enthalpy consistency: source-only audit

Candidate `28277d41` uses constant 6010 J/mol in `states::freezing_point_from_activity`, while `solve::water_fusion_enthalpy_at` integrates the shipped phase heat capacities. Two independent calculations agree that the source comment claiming about 4% variation and less than 0.1 K boundary error is unsupported by those curves.

| Current constant-fusion liquidus | Source fusion enthalpy | Hypothetical consistent-liquidus shift |
| --- | --- | --- |
| 270 K | 5889.39734 J/mol | −0.0319411 K |
| 260 K | 5497.22901 J/mol | −0.5886776 K |
| 252 K | 5173.46196 J/mol | −1.6002976 K |

At 252 K the imposed activity is 0.800835131629332 and fusion enthalpy is 13.9191% below the reference. The corresponding 250.3997024 K root is **below the unchanged 252 K model cap**. It is a mathematical comparison, not an admissible equilibrium prediction or a measured correction. Liquid heat capacity below its table is endpoint-held, which is a model extension, not empirical authority for supercooled water.

`reference.py` integrates the coefficients analytically with 60/80 decimal precision. `independent_quadrature.py` evaluates heat capacities directly and integrates with 512/1024-panel Simpson quadrature; it agrees within 1.6e−12 K at these coordinates. Both read the pinned Git registry; neither runs the application or its phase solver. Their JSON outputs retain assumptions and limitations. Portable invocation:

```console
python3 -B audits/target468-20261016/liquidus-cp-consistency-source-review-20261021/reference.py --repo . --output reference.json
python3 -B audits/target468-20261016/liquidus-cp-consistency-source-review-20261021/independent_quadrature.py --repo . --reference reference.json --output independent.json
```

This is a separate prospective [source-consistency lane](../../../docs/liquidus-thermodynamic-consistency-lane-20261021.md). It does not change PR857, frozen cap controls, current energy-refusal acceptance, scientific selections or calibration. The first source-audit draft rejected a strict lower-table-bound assumption before producing results; checking the actual registry showed equality at 273.15 K, and the endpoint assertion was corrected before either published result.
