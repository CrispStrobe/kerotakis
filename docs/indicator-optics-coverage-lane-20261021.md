# Indicator optics: missing pH must not become a zero absorbance

Status: demonstrated historical failure and current source gap; implementation and current-source runtime acceptance pending. Preserve the active PR852 precision head and its acceptance. This lane follows that delivery; it does not change the qualified count by itself.

## Evidence and shared cause

[Saved AD005 review](../audits/target468-20261016/AD005-saved-original-review-20261021/README.md) and [saved AD006 review](../audits/target468-20261016/AD006-saved-original-review-20261021/README.md) preserve disjoint original screens and limits. Historical AD001/AD002/AD003/AD004 have11 actual zero absorbance readings at405nm despite positive aqueous methyl-orange inventory. Every affected measurement has no solved aqueous state. Original protocols and raw results are in `audits/new-experiments-201-20261016/`; consult each reader's exact archive/script/output bindings rather than replaying the baseline.

In `crates/kerotakis-core/src/solution_optics.rs`, `absorbance` needs `solution.ph` for a pH-dependent indicator. When no solution exists, it skips that contribution and leaves zero bands. `spectral_gaps` returns an empty list without a solution. In `crates/kerotakis-core/src/instrument.rs`, `Spectrophotometer::measure` consequently reports the maximum of the zero bands as a complete reading. Its first maximum is405nm. Missing spectral prerequisites are being confused with measured transparency.

AD005/AD006 have a represented salt solution and actual positive465nm readings, but their dye ionic equilibrium is still unsupported. Their bounded material/qualitative adsorption qualifications explicitly retain this limit; adding salt is not a scientific repair for the held protocols. EN001's absent iodine/starch optical discrimination is a separate coupling/data gap; do not attribute it to methyl-orange pH alone.

## Lane A: truthful shared optical coverage

Start a narrow branch from refreshed actual main after the active candidate is accepted. Add a shared coverage query for the optical prerequisites of the actual aqueous/liquid components. A known pH-dependent indicator with positive visible inventory and missing/nonfinite pH cannot supply a complete absorbance reading. Route the instrument and appearance coverage messages through that shared query, preserving useful supported partial spectra while labeling their limits. Use existing typed refusal/Phrase patterns and localized catalogs; preserve machine identities and provide localized reasons. Inspect `Bench`'s UVvis measurement handling before choosing the event contract.

Required causal controls: no-solution methyl orange; nonfinite pH; supported solved-pH methyl orange; a non-pH-dependent supported absorber without an aqueous solution; mixed supported absorber plus missing-pH indicator; a genuine supported zero reading; retained solid/adsorbed-only indicator that cannot create an aqueous spectral contribution; and all existing free-ion/complex/partial-spectrum/native-gap tests. Check stock native and analytic routes, original instruments/appearance, all actual ownership before/after and refusal/no-mutation behavior. No fallback pH7, invented spectrum or blanket rejection of valid nonionic optics. Freeze first failures before changing behavior, retain original11 zero observations, and follow focused controls with applicable full/platform/product acceptance.

This repair improves honesty; an explicit refusal does not make the original colour-comparison experiments pass.

## Lane B: complete the original indicator chemistry where justified

Independently identify why each original water/dye protocol lacks a current represented pH, using source-bound solver admission/cache/analytic-route evidence. Determine the actual reagent identity and counterions from registry compositions. Acquire reviewed primary authority for the chosen indicator's acid/base forms, charge/proton accounting, pKa and supported domain; distinguish editorial colour parameters from validated aqueous equilibrium parameters. Scope a generic indicator-acid/base integration that retains both total dye and actual counterions, rather than adding salt or assigning a blank-solvent pH to an unrepresented mixture.

Freeze original forecast, literal script, first failure and material/charge/energy ownership. Test acidic/basic/intermediate conditions, concentration/dilution, adsorption and desorption, filtration, coexistence with other buffers, unsupported domain and repeated settlement. Demonstrate the independent aqueous model and supported optical state before replaying only affected originals. Retain empirical adsorption-provenance limits, calibrated absorbance range and actual controls. Qualification must still satisfy every original criterion; source/refusal fixes and zero exits alone do not earn selections.
