# Prepared analytical-basis follow-ups

Two prospective controls were frozen in [design.json](design.json), then executed
once each through the authenticated API. Both returned200/exit0 in about4.5s.
The [raw responses and receipts](results/sha256.json) and
[saved assessment](results/assessment.json) bind the original scripts, enginec446,
binary670bd3 and gateway266121dd. No unchanged corpus batch was repeated.

All frozen checks and whole-vessel element/formal-charge balances pass. Rank084
passes1C, deposits5.182134828µmolCu and produces the same amount ofCl2. Retained
Cu/Cl include the small atacamite phase. Initial Cu-only addition emits charge
and withheld-Tenorite boundaries; sequential analytical ions are not a CuCl2
stock alias or dissolution-enthalpy model.

Rank118 forms10mmolCO2 and consumes permanganate, but retains uncompensated
analytical charge and excluded rhodochrosite phases. It is a bounded analytical
control, not a complete physical stock protocol. The separately frozen
[complete-stock follow-up](../oxalate-physical-stock-followup/README.md) addresses
preparation explicitly without changing this first result.

Reproduce the JSON assessment using [assess_saved.py](assess_saved.py) with
`--design`, `--results` and an unused `--out` filename. The reader uses exact-source
registry compositions, baseline observation2 and reaction observation5; it does
not count trapped-CO2 repartition as newly produced carbon. No engine or network
request is made. Catalog selection remains separate; original refusals, missing
identities, empirical rates and heat calibration remain unchanged.
