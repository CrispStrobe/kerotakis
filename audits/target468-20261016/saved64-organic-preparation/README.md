# Saved named-organic reaction selection preparation

The [reader](verify_metrics.py) independently reproduces [all12 original cases](metrics.json),
including16 before/after named-reaction snapshots, actual owned molecular
amounts, carbon and oxygen bounds, inspection controls and explicit model limits.
It authenticates the original source, archive, scripts and raw streams. No
simulation/API request, catalog selection or current-deployment claim is added.

```console
python3 audits/target468-20261016/saved64-organic-preparation/verify_metrics.py
```

The source's ester constant is the explicit ideal teaching model K≈4, without
activity, temperature, rate or reaction-heat calibration. OR001/002/004/005
return the expected ideal quotient to numerical resolution; OR003 suppresses
ester formation with added water but has an actual post-tail molecular quotient
of3.998559711649207. Its acetate/proton redistribution must not be hidden by
replacing that value with4. The whole owned carbon ledger still closes.

OR004 has a genuine negative ester extent (−0.016mol). OR005's second command
has zero additional ester extent. All four original saponification requests
OR006–009 have zero ester cleavage and remain held for the already scoped
aqueous-base prerequisite repair; conservation alone does not accept them.

Oxidation/respiration are explicitly requested stoichiometric models, not
spontaneous vinegar production or cellular mechanisms. The respiration oxygen
limit includes trapped room-air oxygen. Count CO2(aq), gas CO2, bicarbonate and
carbonate as owned inorganic carbon; returned solution distributions add no
matter. Both respiration carbon increments equal six times consumed glucose
within about1e-12mol. This is internal stoichiometry, not metabolic calibration.
A missing curated reaction heat does not imply an exactly unchanged temperature:
separate aqueous/physical tails can still contribute a thermal response.

[The first-reader correction](first-reader-failure.json) preserves the initial
assessment mistake: its inorganic-only subtotal omitted CO2(aq), while its
formula-based total carbon was already correct. Original reader/output hashes
and wrong subtotal values are retained. Only saved arithmetic was corrected;
no experiment was rerun. Formal criterion and source-domain review remains
separate; the catalog stays200 / fixed468 selected196.
