# Independent review of fourteen proposed precipitation selections

This is a separate review of the sibling precipitation assessment. Its original
files remain unchanged. The [rerunnable reader](review_precip_selections.py)
checks the exact original JSON, artifact, scripts, stdout/stderr and execution
receipts, reproduces positive phase comparisons and final represented element/
charge accounting, and emits the [review receipt](precip-selection-independent-review.json).
No engine, API, build or new experiment is involved. The archive reserializes
the workflow design: original and archived byte hashes differ, but complete
parsed values match; both hashes are preserved in the receipt. Source is c446,
binary670, with producer-attested executable/lock identities rather than raw
binary or Cargo.lock bytes inside this case archive.

For root's final consideration, **thirteen bounded candidates are supported**:
PR-001–004, PR-006–007, PR-009–015. **Hold PR-016.** This does not add catalogue
entries or certify complete physical forecasts, phase assemblages, spectra,
rates, empirical constants or thermal behaviour.

- **PR-001:** The solid changes from0.999223962355 to0.974339134834mmol,
  a0.0248848275212mmol decrease, or2.4904154%. Total owned zinc remains1mmol;
  positive Zn(OH)3-/Zn(OH)4-2 distribution accompanies the dissolved increment.
  Original C2 says “amount decreases”, without a clearing or minimum-fraction
  requirement. This is substantive partial redissolution, not clearing. Inventing
  a complete-clearing requirement would alter the original criterion.
- **PR-016:** AgCl is positively represented in both vessels; its aqueous and
  ethanol conversions differ. Nevertheless the remaining AgNO3 and NaCl, each
  0.443902606mmol, are labelled liquid, with no separate undissolved-solid owner.
  Thus the explicit original C1 distinction is inconclusive. More decisively,
  [nonaqueous.rs](https://github.com/CrispStrobe/kerotakis/blob/c44692053a94c59dcf62e7fb67549f84c7d18f83/crates/kerotakis-core/src/nonaqueous.rs#L36)
  says the historical solubility citations were withdrawn; the NaCl/ethanol
  0.065g/100mL and AgNO3/ethanol2.1g/100mL rows expressly claim no source.
  [curated.rs](https://github.com/CrispStrobe/kerotakis/blob/c44692053a94c59dcf62e7fb67549f84c7d18f83/crates/kerotakis-core/src/curated.rs#L682)
  still uses those magnitudes to cap available dissolved reagent and reaction
  extent. No named limitation appears in this saved case. Conserved atoms and
  honest reporting do not make that unsupported quantitative solubility an
  earned chemistry selection. Preserve the positive product and first outputs;
  investigate provenance admission and residual phase ownership separately.

Four supported candidates require particularly explicit model scope:

| Case | Limits that remain attached to its bounded positive result |
| --- | --- |
| PR-003 | Goethite missing; Hematite/Maghemite excluded; Fe(OH)2.7Cl.3 saturation-index classification absent. The observed Fe(OH)3 dissolution is not a full iron-phase prediction. |
| PR-004 | Only partial copper-solid dissolution with positive ammine distribution; Tenorite withheld by curated temperature policy; USBM constants are reference-temperature approximations, complex heat/rates/spectra incomplete; UV/visible instrument refuses complete absorbance. No clearing or quantitative colour claim. |
| PR-012 | Initial Witherite exclusion remains. Positive CaCO3/BaSO4 phases, carbonate carbon export and final calcium/barium separation are observed; no complete competing-phase model claimed. |
| PR-015 | Tenorite formation withheld by curated metastability threshold. Positive CuO dissolution and Cu(OH)2 product are observed; no general oxide nucleation or rate claim. |

The thirteen candidates have positive phase/equilibrium evidence beyond literal
honesty guards. Final represented global H/O/C/N/P/S/Na/Cl/Ca/Mg/Fe/Zn/Cu/Ag/Ba/Si
and charge residuals have maximum1.248e-11mol, including trapped air and actual
exports. This is final whole-script accounting, not per-step or energy closure.
Brochantite/antlerite/atacamite copper coefficients are4/3/2; bound-ligand
solution distributions and historical lots are not counted as extra matter.
The receipt retains actual limitation messages and phase ownership for every
reviewed case. Root owns any final selection and its exact scope. Catalogue
additions and new engine executions: zero.
