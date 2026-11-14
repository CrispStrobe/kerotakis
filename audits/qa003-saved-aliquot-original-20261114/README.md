# QA003: unchanged aliquot and stock titration

The saved original meets **2/2 original criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`. No build or experimental replay was needed. The original script, forecast, acceptance wording and observations are retained.

The runtime reports pre-titration liquid volumes of **0.15 L** for the retained stock and **0.05 L** for its aliquot. Endpoint NaOH demands are **0.0015000005626678432 mol** and **0.0005000001859664917 mol**. Their ratio is **3.000000009536733**, satisfying C1 within the original 0.1 mL burette increment. Dividing by each reported pre-titration liquid volume gives **0.0100000037511 M** and **0.0100000037193 M**: the difference, `3.1789121e-11 M`, is below the combined dosing resolution `2.6666667e-4 M`, satisfying C2. Final volumes, which include added burette water, are not used as the denominator.

**Volume observation boundary:** both original `measure volume` commands explicitly refuse to report an open vessel's headspace volume. Those refusals remain in the raw evidence and are required by the reader. The actual liquid volumes are explicitly reported on the saved vessel lines at those same observation steps, independently checked against retained water and the historical source's density/molar-mass convention. The comparison uses those reported liquid observations; it does not invent successful headspace-meter readings or claim a calibrated liquid-volume instrument.

An independent per-vessel H/O/Na/Cl/charge ledger counts the original water/HCl, the quarter transfer, all accepted NaOH doses and their carrier water. Chloride remains retained, sodium matches delivered base, and solvent plus neutralisation water closes with all element/charge residuals below **1.4e-13 mol/eq**, under the unchanged `1e-10` bound. Both endpoints are reached within `0.0001 pH`; no solver failure, unresolved material or gas crossing appears. Eight mutations reject lost chloride, lost carrier water, a changed aliquot, a final-volume denominator, altered endpoint dose, an unreached endpoint, a hidden volume refusal and an injected solver failure.

`binding.json` authenticates the exact original script/output, source/tree, genuine producer lock and successful locked-release build receipt. The historical producer captured **no executable checksum**; `native_binary_sha256` remains null and no later hash is substituted. The aggregate producer failure flag for other experiments is not treated as QA003 chemistry evidence. `source-properties.json` extracts only the required compositions and water properties from this historical source.

```sh
python audits/qa003-saved-aliquot-original-20261114/review_original.py audits/qa003-saved-aliquot-original-20261114
```

This qualification establishes the model's original aliquot/stock comparison at its stated dosing resolution. It does not establish experimental titration accuracy or liquid-volume calibration.
