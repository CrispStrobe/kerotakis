# Electrochemical model-extension review

Reviewed 2026-09-08. No source files or raw polarization tables are vendored.

## Q345R mixed-potential model

Han et al., “Corrosion Behaviors of Q345R Steel at the Initial Stage in an
Oxygen-Containing Aqueous Environment: Experiment and Modeling,” *Materials*
11 (2018) 1462, [doi:10.3390/ma11081462](https://doi.org/10.3390/ma11081462),
is CC BY 4.0. The reviewed local article copy had SHA-256
`85dacee1d2c75b748e3ae3a90344c8b65e97654dc4b9a2fe449b94227a7f6f9b`.

The source supplies Fe dissolution, oxygen reduction, proton reduction and
water reduction models, including Arrhenius terms, activity orders, Tafel
slopes, diffusion correlations and numerical corrosion tables. Kerotakis now
represents those concepts generically: composable temperature/activity
scaling, temperature-derived Tafel slopes, union-shaped validity domains and
typed model evidence. Four source parameter sets are retained as unreviewed
candidates so their interpretation and reproduction test remain inspectable.
They cannot be selected by the runtime.

At 303.15 K, pH 6 and 0.08 mg/L dissolved oxygen, equations 2–25 as printed
give approximately -0.807 V versus SCE and 0.0262 A/m2 in the shared solver.
Table 2 reports -0.810 V and 0.017 A/m2 for the model. The potential agrees but
the current does not. The test pins both the successful potential reproduction
and the material current discrepancy; no coefficient is tuned to erase it.
The source records remain `published_model_pending_reproduction` and
`reviewed = false` until an attributable correction or independent exact
reproduction resolves the mismatch.

The reported experimental support forms two slices—30 °C over pH 5–9, and
pH 6 over 30–80 °C—not the full Cartesian rectangle. The candidate domain
stores that union explicitly.

## CC0 pure-iron polarization curves

Wilson, Sunde and Erbe, “Polarization curves of iron under different
conditions,” [doi:10.18710/CHYUQX](https://doi.org/10.18710/CHYUQX), is released
under CC0. The reviewed README had SHA-256
`1f34430cec6eca390df4dbf3795a64004ff0941e7b8149e3668e7f29428bc0da`.
The dataset contains ASCII replicate curves for polished 99.9% ARMCO iron in
acid, pH-9 borate, 3.5 wt.% NaCl and NaOH, with electrolyte, flow,
illumination, oxide-thickness, scan-rate and reference-electrode metadata.

The raw files are total measured polarization curves crossing open circuit;
they are not separately measured anodic partial currents. They can validate
whole-cell curve shape, transition regions and future component-separated
models, but fitting their positive-current limb as an isolated Fe-dissolution
law would silently include cathodic current. Therefore no kinetic record is
derived from them. Kerotakis gains a generic active/passive/transpassive law,
but no CC0 coefficients are admitted without a defensible partial-current
decomposition and its uncertainty.

## Qualitative active/passive validation

Xie, Li and Li, “Polarization Behavior of Steel Embedded in Cement-Based
Materials with Different pH,” is CC BY 4.0 at
[doi:10.5281/zenodo.1122257](https://doi.org/10.5281/zenodo.1122257). The
reviewed PDF had SHA-256
`5eb8ee0645a1b855ea1cd439c026ad6874387f86cdc09b84bb6094fa0eaee1d1`.
It supports the existence and ordering of pH-dependent active/passive and
polarity-reversal behavior, but supplies plotted curves rather than a numeric
table adequate for an isolated kinetic fit. It remains validation context;
no points are digitized and no runtime record is created.
