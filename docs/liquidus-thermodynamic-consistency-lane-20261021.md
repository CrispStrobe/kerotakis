# Shared liquidus and fusion-enthalpy authority

This lane follows accepted delivery of PR857. Preserve its source-bound frozen12/current38 controls and first baseline/fixed failures. Do not change its reviewed head while full/platform validation is active. This lane closes a separately demonstrated model inconsistency; it does not add physical calibration or automatically qualify experiments.

## Demonstrated problem

The [source audit and independent quadrature](https://github.com/CrispStrobe/kerotakis/blob/77262d5f/audits/target468-20261016/liquidus-cp-consistency-source-review-20261021/README.md) pin candidate `28277d41`. `crates/kerotakis-core/src/states.rs::freezing_point_from_activity` assumes constant fusion enthalpy of6010 J/mol; `crates/kerotakis-core/src/solve.rs::water_fusion_enthalpy_at` adjusts that enthalpy using phase heat capacities. At252 K the latter is5173.46196 J/mol,13.9191% lower. At the same imposed solvent activity, integrating the latter gives a hypothetical250.3997024 K coexistence root. This contradicts the source comment claiming4% and under0.1 K throughout the allowed interval.

The hypothetical root is below the existing252 K cap. Do not extend that cap to admit it. Liquid-water Cp is held at its273.15 K table endpoint below that temperature; this is an explicit model extension, not authority for real supercooled-liquid properties. The calculation establishes a source inconsistency, not a new measured prediction or a demonstrated failure of PR857's energy-conserving refusal.

## Implementation contract

1. Start a narrow branch from actual accepted main. Refresh `states.rs`, `solve.rs`, `heat_capacity.rs`, the registry and cap/phase tests. Correct the unsupported comment. State whether endpoint holding remains a bounded model extension; introducing supercooled-liquid data requires its own provenance and domain review.
2. Supply one shared thermodynamic authority for fusion enthalpy, the chemical-potential integral and its inverse. With reference `Tm`, require `Hfus(T)=Hfus(Tm)+integral(Tm,T)(Cp_liquid−Cp_ice)dT` and `ln(a_w)=integral(Tm,T)Hfus(u)/(R*u²)du`. Preserve reference273.15 K and6010 J/mol. Use bounded numerical inversion with explicit out-of-domain refusal; never silently clamp an unconverged root into a supported answer.
3. Route both freezing/melting energy transfers and `freezing_point_from_activity`, `activity_at_freezing_point`, particle-molality inversion, concentration caps and `transitions_with` through that same authority. Keep the252 K admissibility bound and route-specific concentration ceilings distinct. Update all cap inversions consistently rather than retaining old numeric concentration values derived from the previous law.
4. Update cap-roundoff sensitivity consistently: the prospective derivative is `dT/dln(a_w)=R*T²/Hfus(T)`. The current6010 denominator is correct for the current constant law; carrying it unchanged into a variable-enthalpy law could understate sensitivity. Preserve strict0.001 J/1e−10 mol energy/material contracts and refuse unresolved comparisons instead of creating an excess-energy sink.

## Independent controls and completion

Before implementation, freeze a new reference suite separately from historical fixtures. Include zero-delta-Cp recovery of the old law, a nonzero constant-delta-Cp analytic solution, independent shipped-curve quadrature under the stated endpoint policy, pure-water reference, dilute limiting slope, monotonicity, forward/inverse round trips, finite/domain validation, and exact/inside/outside cap cases. Exercise both freezing and melting with complete owned energy/material ledgers and retained externally delivered heat. Include examples near270,260 and252 K; do not present a below-cap mathematical root as an admitted state.

Preserve original forecasts and historical frozen12 bytes as evidence for their original source. If numerical expectations genuinely depend on the old law, explicitly adjudicate those contracts and introduce independently frozen successor expectations; do not widen tolerances or silently relabel old evidence. Recheck affected original protocols, including TC008, only on the changed source and only after normal focused/full/platform/product acceptance. Record model-domain limits in any earned selection. A unit reference or improved comment earns no corpus credit.

Completion requires reviewed shared-authority code, independent reference agreement, exact retained energy/material budgets, typed and localized domain refusals, full applicable own-fleet/native/browser checks, normal exact-head integration and changed-release acceptance. Keep boiling/vaporization consistency as a separately scoped follow-up rather than silently broadening this freezing change.
