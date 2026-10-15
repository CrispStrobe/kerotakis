# First physical assessment of the forty spelling controls

The preserved hosted execution has now been assessed against its unchanged
forecasts: **28 pass, 1 fail, 9 unsupported, 2 inconclusive**. This is the old
source-bound follow-up set, not forty new blind designs or a current-main replay.
The original fifty remain **7 pass, 2 fail, 40 unsupported, 1 inconclusive**.

[The case-by-case assessment](normalized40-first-assessment.json) records each
judgment, stream hashes and exact `output_sequence` references. Its full execution
binding and [raw-integrity receipt](normalized40-raw-integrity.json) preserve the
source/binary/lock association. No chemistry was rerun to obtain these judgments.
An independent read-only assessment of all forty cases agreed with these
classifications. Scientific passes compare the frozen observable expectations; they do not certify
unrelated mechanisms, complete spectra, all reaction pathways or empirical accuracy.

| Result | Cases / scope |
| --- | --- |
| Fail | 15: both KNO3 samples completely dissolve 20 g in 50 g water; cold sample remains 20.004 °C with no solid, meeting the frozen failure criterion. |
| Unsupported: naming | 07 sucrose formula; 13 acetate spelling; 35/44 ethanol spelling; 36/37 hexane formula. No intended comparison completed. |
| Unsupported: coverage | 05 explicitly lacks reviewed dissolved-ion density; 20 rejects AlCl3, which is absent from the current identity registry; 41 explicitly refuses single-phase drainage because that operator separates liquid layers. |
| Inconclusive | 21 explicitly blocks zinc/acid kinetically and produces no hydrogen after one second; 27 retains Fe(II)/peroxide without resolved oxidation after one second. Neither observation meets its frozen physical failure criterion. |
| Pass | Remaining 28, with scoped qualifications in the assessment. |

Gas judgments subtract initially trapped air/CO2 and use pre-assay inventories,
because gas tests consume material. Electrolysis produces H2/O2 increments in
2:1 ratio after subtracting atmospheric oxygen. Dilution adds the stated solvent
volume; it does not specify a target volume. Titration refines a root near pH 7;
its 6.999995856 endpoint does not establish strict crossing above seven.

## Next executable lanes

1. **Nitrate solubility: reproduce, then repair the shared route.** Freeze an
   actual-current-main source/binary/lock binding separately. Replay case 15
   without changing doses, temperatures or predictions. Inspect current
   `crates/kerotakis-phreeqc/src/aqueous.rs`, PHREEQC phase selection and KNO3 source
   data to determine why the cold solid disappears. Retain both initial failures
   before fixing. Validate cold residual solid, hot complete dissolution,
   temperature effects, K/N/O/water inventories, phase ownership and refusal paths.
   Extend controls to other salts only where reviewed solubility data support
   them. Do not patch this by clamping the display or hiding dissolved inventory.
2. **Six naming follow-ups, separate controls.** Freeze explicit identity inputs
   `sucrose`, `NaOAc`, `ethanol` and `hexane` for cases 07/13/35/36/37/44 while
   retaining original forecasts. Confirm actual-current catalog identity first.
   Do not rewrite the original scripts or broadly alias ambiguous molecular
   formulas to one isomer. Reassess actual conductivity, mixing, extraction and
   enrichment after execution; parsing success alone earns no scientific pass.
3. **Three coverage gaps, three bounded contracts.** For density, follow the
   existing reviewed solution-density lane with independent reference/holdout
   data; water defaults are not solution-density data. For AlCl3, establish
   identity, aqueous aluminum mapping, phase/reaction data and provenance before
   introducing a spelling shortcut. For drainage, explain liquid-layer semantics
   and freeze a separate supported transfer control; do not silently change the
   funnel operator to satisfy the old assumption.
4. **Two kinetics questions.** Inspect the zinc overpotential cutoff and ferrous
   peroxide mechanism admission/rate ownership. Freeze condition/time/surface
   controls with explicit expected support and appropriate references before
   extending rates. Keep the one-second outcomes inconclusive; no invented
   kinetics or thermodynamic pass follows from an inert message.

Product repair PR789–792 still require their own acceptance and normal protected
integration. Resource gates apply before builds, replays or extra agents. Preserve
this assessment as a first outcome; corrections belong in a separate follow-up.
