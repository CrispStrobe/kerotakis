# HBr dissociative gas uptake: source and basis review

Reviewed 2026-09-06; model-assisted review and derived calculations disclosed.

## Rights and accepted fields

Rolf Sander (2023), *Compilation of Henry's law constants (version 5.0.0)
for water as solvent*, ACP 23, 10901–12440,
[doi:10.5194/acp-23-10901-2023](https://acp.copernicus.org/articles/23/10901/2023/).
The publisher page and PDF explicitly license this work **CC BY 4.0**.
Attribution, source link, licence link, and indication of changes are retained
in NOTICE and the data header. This is an attribution-only, commercial-use
permissive data slice, not GPL, NC, ND or ShareAlike content.

[PDF](https://acp.copernicus.org/articles/23/10901/2023/acp-23-10901-2023.pdf)
SHA-256: `8c059bc311cacb0b4626f1935a06f990303d9af0af50721dd5fb864ddbc88bad`.
HBr table: printed p.10924, PDF index 23. Note 146: printed p.12382,
PDF index 1481, visually checked. Equation 21, printed p.10906, defines
the dissociative quantity, distinct from molecular HBr solubility.

The accepted two fields are the reference prefactor **8.2e9
mol²/(m⁶ Pa)** at **298.15 K** and its local inverse-temperature slope
**10000 K**, from note 146 (underlying reference Carslaw, Clegg and
Brimblecombe, 1995, DOI 10.1021/j100029a039). Sander marks this row as a
thermodynamic calculation, not direct measurement. The source notes that a
more detailed temperature expression exists in the original paper; this
slice does not claim to reproduce that expression. No greater experimental
accuracy than the displayed source precision is implied.

The 6.8e-2 molecular-HBr machine-learning estimate is NOT used. Notes 147
and 148 give alternative dissociative prefactors; they are not averaged or
silently substituted. This review clears the named fields from Sander's
licensed compilation, not Carslaw's complete copyrighted paper.

## Explicit conversion and reference enthalpy

The phase reaction is `HBr(g) = H+ + Br-`. Both aqueous ions are native
masters in WATEQ4F, MINTEQ v4 and Pitzer, so no LLNL master-state or
activity-model parameters are imported. At infinite dilution `c = rho*b`,
where `rho` is pure-water density in kg/m³. Therefore:

```
K_molal,atm(T) = Hs_prime_SI(T) * 101325 / rho(T)^2
ln K(T) = ln Hs_prime(T) + ln(101325) - 2 ln rho(T)
delta_H(T0) = -R*10000 - 2*R*T0^2*rho_prime(T0)/rho(T0)
```

Pressure uses 1 atm = 101325 Pa exactly. This is a STANDARD-STATE density
conversion, not division by the salt solution's concentration-dependent
density. Finite-concentration activities subsequently use the routed native
database; this does not validate missing concentrated-mixture interactions.

For consistent density and its derivative we evaluate the existing USGS
`Phreeqc::calc_rho_0` formula in
`vendor/iphreeqc/src/phreeqcpp/utilities.cpp` at 25 °C, 1 atm and pure-water
activity 1. That source is already approved as `usgs-iphreeqc-code` under
the USGS user-rights notice. Evaluated file SHA-256:
`a93c82d158df1ef0bf8d3ee426614382a0dbfe5ab6431b6cb4bfefddfc7bfc28`.
The calculation follows its saturation-density expression and pressure
correction, including `pa -= (p_sat - 1e-6)`; it does not copy LLNL data.

Numerical results using R = 8.31446261815324 J/(mol K):

| Quantity | Computed value |
| --- | ---: |
| rho(25 °C, 1 atm), kg/m³ | 997.0430117423601 |
| rho derivative, kg/(m³ K), symmetric 0.001 K step | -0.25659174059455836 |
| log10 K at 298.15 K | 8.922102677041854 |
| Derived reaction enthalpy at 298.15 K, kJ/mol | -82.76420684218871 |

Derivative convergence check: symmetric 0.01 K and 0.0001 K steps give
enthalpies -82.76420684660474 and -82.76420684202017 kJ/mol. Additional
digits preserve deterministic conversion, not measurement precision.
Sander Table 2 independently uses rounded rho=997 kg/m³ for reference
conversions, consistent with the same dilute-water basis.

The enthalpy is a **derived local slope**, not a measured calorimetric
enthalpy or a standard formation enthalpy. Native PHREEQC's `delta_h`
continues it with a constant-enthalpy van't Hoff law. That continuation
is explicitly a local approximation at temperatures other than 298.15 K;
it is not validated 0–200 °C coverage. The adapter must expose that boundary.
No rate law, mass-transfer coefficient or concentrated-mixture parameter
is imported. Native mass action and finite inventories determine uptake.

## Rejected alternative and verification scope

The vendored `llnl.dat` HBr(g) row was considered but NOT imported. LLNL's
[official disclaimer](https://www.llnl.gov/disclaimer) explicitly reserves
possible copyright and says reproduction permission may be necessary.
USGS hosting and government sponsorship alone do not clear LLNL's source.
In particular, its 0–200 °C analytic fit is not covered by this review.

The new module test verifies the source-to-runtime standard-state conversion
and derived enthalpy in all three extended database indexes. Native dose,
temperature-boundary and closed-headspace tests belong to the parent
integration. At creation of this review these new tests have not yet run;
test results must not be inferred from the transcription check.
