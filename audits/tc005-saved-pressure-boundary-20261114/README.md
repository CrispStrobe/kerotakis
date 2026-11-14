# TC005: unchanged pressure/inventory and property-domain inspection

The saved original meets **2/2 original inspection criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`. **The forecast's validated 2-bar pressure-aware boiling law is not established.** The original forecast, criteria and limitation remain unchanged; the qualification is their explicit actual-pressure/phase-inventory and reference-property review.

C1 asks to check actual pressure and phase inventories rather than assume the regulator has no mass exchange. The controller initially admits **0.004033954554584697 mol** and **0.008067909109169394 mol** room air. Those terms are included separately in the independent H/O/C/N/charge ledger. Thereafter the pressure-controlled boundary owns its gas and lets volume move; it is neither a fixed 100 mL sealed headspace nor an atmospheric reservoir. Actual pressures remain **100/200 kPa**, while final headspaces expand to **1.4501718855312429/0.7731766341929734 L**. All checkpoint gas inventories reproduce the reported volumes under the pinned source's ideal-gas constant with zero volume residual. Every element/charge checkpoint closes below **3.66e-16 mol/eq**, within `1e-10`; liquid and gaseous water are both retained, and no solver failure or external gas loss is reported.

C2 asks to compare reported boiling points with operative pressure rather than automatically use a 1-bar reference. The 100 kPa thermal route explicitly reports `cleared_correlation` and **99.63318778075063 °C**. The 200 kPa route explicitly reports `pressure_outside_cleared_window`, zero correlation shift and a **100.00000535673809 °C reference fallback**. That fallback is not interpreted as a validated 200 kPa boiling point. Both boiling-point apparatus events withhold a numeric result: `not_isolated`, `value_c: null`; their `lowest_component_c: 100` is not a successful measurement. These distinctions satisfy the original inspection criterion while keeping the pressure-domain limitation explicit. No extrapolation or guard relaxation occurs.

Both original 8 kJ deliveries are recorded with their reported sensible allocations and phase inventories. Those fields are retained as model output, without claiming high-pressure plateau accuracy, calorimetric calibration or a complete empirical sensible/latent law. The original pressure-aware forecast is preserved rather than rewritten to fit the fallback.

Nine controls reject lost nitrogen, lost gaseous water, a wrong admitted-air amount, reuse of the initial fixed volume, wrong operative pressure, a hidden pressure-domain limit, an invented numeric apparatus reading, a mismatched thermometer and solver failure. Original script, observations and all prior catalog objects remain unchanged.

`binding.json` authenticates original/output, source/tree, actual producer lock and successful locked-release build receipt. Boundary-source excerpts preserve the owned-gas/variable-volume definition and gas constant; registry compositions are extracted from the same source. The historical producer recorded **no executable checksum**; it remains null, with no newer artifact/hash substitution. No new build or experimental replay was performed.

```sh
python audits/tc005-saved-pressure-boundary-20261114/review_original.py audits/tc005-saved-pressure-boundary-20261114
```
