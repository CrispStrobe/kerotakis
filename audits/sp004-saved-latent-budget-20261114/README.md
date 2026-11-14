# SP-004: two distillation budgets and the declared energy boundary

The unchanged original earns **2/2 inspection criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`, tree `3e1471c481b9ef6f91a962039e3158ab0df2763d`. Coverage becomes **427/468 originals, 41 remaining**. All 430 predecessor catalog objects remain exact.

C1 compares actual collected and remaining feed stocks from the two initially identical 100 g water/10 g ethanol mixtures:

| Command budget | Collected water, mol | Collected ethanol, mol | Collected mass, g | Remaining feed, g |
| --- | ---: | ---: | ---: | ---: |
| 10 kJ | 0.18011979558124017 | 0.06938490075307156 | 6.441351110189295 | 103.55864888981069 |
| 20 kJ | 0.374116138261396 | 0.12414619405667339 | 12.458993244775935 | 97.54100675522405 |

The larger budget collects more and leaves less feed. Its collected mass is not exactly twice the smaller cut. Each component's source plus receiver returns its original amount. All 27 C/H/O/charge checkpoints close within `3.2e-16 mol/eq`, below the unchanged `1e-10` bound. All available scene masses match independently summed stock masses. There are no solver failures, model warnings, reactions or gas transfers. The canonical event component list is counted once; legacy water/ethanol fields are mirrors, not extra matter.

C2 reads the declared runtime boundary for these actual events. Their empty model field selects the binary `event.distilled.lv3` renderer at the original LV3 register. The pinned template says that the reference latent heat is paid by the burner, dumped by the condenser and deliberately outside the vessel ledger. The pinned bench code separately carries condensate at the source's sensible temperature; heating, reflux/reboiler circulation and condenser duty are not a calculated total apparatus heat balance.

**The historical JSON contains no captured rendered prose.** The review traces the exact selected runtime source branch and interprets its template with the actual saved event slots. Any displayed explanation in the report is explicitly source-derived decoding, not a new observed CLI reply. English interpolation and the source template are pinned. No synthetic runtime observation or claim of a prose replay is made.

The 10/20 kJ command numbers agree with the installed reference latent sums `40.657 × n_water + 38.58 × n_ethanol` within `3.66e-14 kJ` of arithmetic residual. This verifies the declared command convention; it does **not** establish that all energy supplied by real apparatus equals those numbers. Both physical source and receiver remain at `343.15 K`, while the event's calculated boiling path starts near `364.18 K`. Those boiling-event fields are not vessel thermometer readings or proof of a physically heated boiling plateau.

Qualification is the original paired transfer/feed comparison and runtime energy-boundary interpretation. No total apparatus, sensible/reflux or calorimetric closure, empirical VLE, real-column efficiency or recovery calibration is qualified. Original forecasts, script and criteria remain intact.

Ten reader mutations reject lost receiver/source stocks, duplicated event components, an inconsistent legacy mirror, a wrong budget, an invented full-heat model, a receiver spuriously set to the boiling-event temperature, a missing state-changing scene, a solver failure and a changed energy disclosure. The first reader assumed the final inspection repeated a scene and raised `KeyError: scene`; that reader and empty report identities are frozen in [first-reader-failure.json](first-reader-failure.json). The corrected reader permits an omitted scene only for inspection; all physical bench budgets and state-changing scene checks remain. No engine replay or bound change occurred.

Selected raw/producer manifest hashes, exact source/tree, genuine lock, original script, compositions, reference constants and renderer/English-interpolation excerpts are authenticated in [binding.json](binding.json). The historical producer recorded no executable checksum; it remains null with no later identity substituted. PR882/883 corrected-head acceptance remains separate from this historical qualification.

```console
python audits/sp004-saved-latent-budget-20261114/review_original.py audits/sp004-saved-latent-budget-20261114
python audits/sp004-saved-latent-budget-20261114/check_reader_negatives.py audits/sp004-saved-latent-budget-20261114
```
