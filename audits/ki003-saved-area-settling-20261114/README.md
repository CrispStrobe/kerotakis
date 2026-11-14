# KI-003: matched nominal area, explicit particle-size activity

The unchanged original earns **2/2 inspection criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`, tree `3e1471c481b9ef6f91a962039e3158ab0df2763d`. C1 compares fractional peroxide consumption at the same elapsed times. C2 verifies that actual catalyst activity uses the stated diameters. Coverage becomes **425/468 originals, 43 remaining**; all 428 predecessor catalog objects remain exact.

The original forecast of similar responses at equal nominal area is **not confirmed**. The experiment is assessed as the requested comparison and diameter-coupling inspection, not as a validation of that forecast. Both initial areas are `0.0001192842942345924 m²`, but gravity settling occurs before kinetics. At 10 seconds, the 10 µm powder retains suspended fraction `0.9383004394764993`, while the 100 µm powder is fully settled. The installed contact model retains a 0.35 activity floor for settled powder. At 30 seconds the fine powder's suspended fraction is `0.8225096296673946`; the coarse powder remains settled.

| Elapsed time | 10 µm: fraction consumed | 100 µm: fraction consumed |
| --- | ---: | ---: |
| 10 s | 0.0086120392655887 | 0.0032211322092527 |
| 30 s | 0.0235343290073072 | 0.0096326833038335 |

The actual lots retain the requested 10/100 µm sizes, and both grind events report coupled activity. Independent spherical-area calculations reproduce both declared areas. Analytic integration of the pinned installed first-order peroxide law, using the recorded start temperature and post-settling activity for each interval, agrees with all four retained amounts within `3.03e-14 mol`. Substituting the default 100 µm size for the fine powder misses its first retained amount by `7.747479e-6 mol`, far beyond the unchanged `1e-10` check. This distinguishes actual use of the specified diameter from merely recording it.

All 22 H/O/Mn/charge checkpoints close within `1.6670e-15 mol/eq`, below the unchanged `1e-10` inventory bound. The catalyst is unchanged; peroxide loss, water gain and retained gas oxygen independently satisfy `2 H2O2 → 2 H2O + O2`. `gas_produced` reports internal production and is not subtracted as an external sink. Physical contents are counted once; lots and mirrored solution species are not additional inventory. There are no solver failures, model warnings or external gas transfers. The solvent-only pH channel is not used as a peroxide observable.

Ten adversarial reader mutations reject lost water/peroxide/catalyst/oxygen, default-sized lots, an uncoupled grind, missing active settling, a wrong clock, a solver failure and atom-balanced falsified kinetics. The first reader incorrectly demanded another settling event from already fully settled powder; the pinned clock deliberately skips such lots. Its correction permits omission only for an unchanged zero-suspension lot. The [first reader failure](first-reader-failure.json), original empty report identity and original reader identity are preserved. No engine rerun, source repair or bound change occurred.

This is an inspection of the installed teaching model. Its absolute rate scale and contact/settling treatment are not empirical calibration of this experiment. No BET/pore area, particle-size-distribution, real suspension/contact, catalyst-product performance or calorimetric validation is claimed. The original forecast, script, criteria and limitations remain intact.

Selected producer manifest hashes, source/tree, genuine producer lock, original script and pinned registry/kinetics/clock sources are independently authenticated in [binding.json](binding.json). The historical producer did not record an executable checksum; it remains null, with no later binary substituted. This historical qualification is independent of PR883's pending source/platform/fleet and properly matched selector/floor acceptance. The zero-match selector filter earns no control credit.

Reproduce the saved assessment and meaningful reader controls without an engine run:

```console
python audits/ki003-saved-area-settling-20261114/review_original.py audits/ki003-saved-area-settling-20261114
python audits/ki003-saved-area-settling-20261114/check_reader_negatives.py audits/ki003-saved-area-settling-20261114
```
