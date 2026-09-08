#!/usr/bin/env python3
"""Fourth discovery fleet, cases 123–158; authored before observing outputs.

Only virtual CLI inputs live here. The production chemistry has no case
dispatch and imports none of this audit. The shared recorder keeps evidence.
"""
import run as recorder

c = recorder.case
STILL = "add v1 water 3mol\nadd v1 ethanol 0.3mol\nadd v1 methanol 0.3mol"
ACID = "add v1 water 250mL\nadd v1 NaNO3 0.0003mol\nadd v1 HCl 0.001mol\nadd v1 KOH 0.0004mol"
MAGNESIUM = "add v1 water 400mL\nadd v1 MgSO4 0.001mol\nadd v1 NaOH 0.002mol"
BUFFER = "add v1 water 500mL\nadd v1 CH3COOH 0.004mol\nadd v1 NaOAc 0.002mol"
DRY_GAS = "seal v1 250mL\nadd v1 N2 0.002mol"

recorder.CASES = [
    c("123-three-alcohol-family-cut", "Does the computed water/ethanol/methanol cut conserve all three molecular inventories?", STILL + "\nnew\ndistil v1 v2 0.12"),
    c("124-three-solvent-small-scale", "Does reducing all inventories tenfold preserve the cut composition?", "add v1 water 0.3mol\nadd v1 ethanol 0.03mol\nadd v1 methanol 0.03mol\nnew\ndistil v1 v2 0.12"),
    c("125-three-solvent-reordered", "Does the same cut survive a different solvent addition order?", "add v1 methanol 0.3mol\nadd v1 water 3mol\nadd v1 ethanol 0.3mol\nnew\ndistil v1 v2 0.12"),
    c("126-ternary-one-kilojoule", "Does a small latent-only budget match the withdrawn components' enthalpies?", STILL + "\nnew\ndistil v1 v2 1kJ"),
    c("127-ternary-two-kilojoules", "Does doubling latent duty withdraw more of each component without exceeding the feed?", STILL + "\nnew\ndistil v1 v2 2kJ"),
    c("128-four-component-still", "Can the same generic still conserve four simultaneously volatile components?", "add v1 water 4mol\nadd v1 ethanol 0.2mol\nadd v1 methanol 0.15mol\nadd v1 isopropanol 0.1mol\nnew\ndistil v1 v2 0.1"),
    c("129-partial-neutralization-spectator", "Does residual strong acid follow equivalents in the presence of nitrate spectator?", ACID + "\nmeasure v1 ph"),
    c("130-partial-neutralization-reversed", "Does reversing acid/base addition preserve residual acidity and heat?", "add v1 water 250mL\nadd v1 NaNO3 0.0003mol\nadd v1 KOH 0.0004mol\nadd v1 HCl 0.001mol\nmeasure v1 ph"),
    c("131-partial-neutralization-scale", "Does a tenfold larger solution retain the same intensive acidity?", "add v1 water 2.5L\nadd v1 NaNO3 0.003mol\nadd v1 HCl 0.01mol\nadd v1 KOH 0.004mol\nmeasure v1 ph"),
    c("132-residual-acid-diluted-after", "Does fourfold solvent dilution shift residual-acid pH logarithmically?", ACID + "\nadd v1 water 750mL\nmeasure v1 ph"),
    c("133-residual-acid-diluted-before", "Does dilution before neutralization approach the same final chemical state?", "add v1 water 1L\nadd v1 NaNO3 0.0003mol\nadd v1 HCl 0.001mol\nadd v1 KOH 0.0004mol\nmeasure v1 ph"),
    c("134-complete-the-neutralization", "Can a second base dose spend precisely the remaining acid rather than overshoot from stale inventory?", ACID + "\nadd v1 KOH 0.0006mol\nmeasure v1 ph"),
    c("135-sulfate-limits-barium", "Does a sulfate-limited barium precipitation obey the smaller reagent inventory?", "add v1 water 500mL\nadd v1 BaCl2 0.001mol\nadd v1 Na2SO4 0.0004mol"),
    c("136-sulfate-first-barium", "Does reversing these unequal feeds preserve the precipitated barium amount?", "add v1 water 500mL\nadd v1 Na2SO4 0.0004mol\nadd v1 BaCl2 0.001mol"),
    c("137-sulfate-limited-scale", "Does scaling both unequal feeds and solvent scale the computed precipitate?", "add v1 water 5L\nadd v1 BaCl2 0.01mol\nadd v1 Na2SO4 0.004mol"),
    c("138-magnesium-hydroxide", "Does magnesium precipitation account for two hydroxide equivalents per supplied metal?", MAGNESIUM),
    c("139-magnesium-acid-reversal", "Does excess acid remove the magnesium solid rather than preserve a cached phase?", MAGNESIUM + "\nadd v1 HCl 0.003mol"),
    c("140-magnesium-half-base", "Does halving hydroxide reduce the solid magnesium amount?", "add v1 water 400mL\nadd v1 MgSO4 0.001mol\nadd v1 NaOH 0.001mol"),
    c("141-closed-co2-small-space", "Can finite CO2 uptake conserve carbon in a sealed water/headspace system?", "add v1 water 500mL\nseal v1 250mL\nadd v1 CO2 0.0002mol\nmeasure v1 pressure"),
    c("142-closed-co2-large-space", "Does larger headspace leave no more of the same CO2 dose in solution?", "add v1 water 500mL\nseal v1 1L\nadd v1 CO2 0.0002mol\nmeasure v1 pressure"),
    c("143-closed-co2-double-dose", "Does a larger finite dose increase condensed carbon without losing total carbon?", "add v1 water 500mL\nseal v1 250mL\nadd v1 CO2 0.0004mol\nmeasure v1 pressure"),
    c("144-closed-co2-more-water", "Does more water accommodate at least as much of the same sealed CO2 dose?", "add v1 water 1L\nseal v1 250mL\nadd v1 CO2 0.0002mol\nmeasure v1 pressure"),
    c("145-dry-gas-single-heat", "Does heating a dry sealed nitrogen/air inventory preserve atoms and ideal-gas pressure?", DRY_GAS + "\nheat v1 5J\nmeasure v1 pressure"),
    c("146-dry-gas-partitioned-heat", "Does splitting the same dry-gas energy delivery preserve final temperature and pressure?", DRY_GAS + "\nheat v1 2J\nheat v1 3J\nmeasure v1 pressure"),
    c("147-asymmetric-ester-forward", "Does an asymmetric four-component mixture solve the mass-action equation from its current state?", "add v1 CH3COOH 0.012mol\nadd v1 ethanol 0.03mol\nadd v1 ethyl_acetate 0.003mol\nadd v1 water 0.006mol\nreact v1 esterification"),
    c("148-asymmetric-ester-product-control", "Does adding more initial ester suppress net forward extent at the same other inventories?", "add v1 CH3COOH 0.012mol\nadd v1 ethanol 0.03mol\nadd v1 ethyl_acetate 0.012mol\nadd v1 water 0.006mol\nreact v1 esterification"),
    c("149-asymmetric-ester-reverse", "Can a product-rich four-component mixture compute a negative extent?", "add v1 CH3COOH 0.006mol\nadd v1 ethanol 0.01mol\nadd v1 ethyl_acetate 0.03mol\nadd v1 water 0.02mol\nreact v1 esterification"),
    c("150-asymmetric-reverse-scale", "Does a sevenfold scale change preserve the reverse equilibrium extent per unit amount?", "add v1 CH3COOH 0.042mol\nadd v1 ethanol 0.07mol\nadd v1 ethyl_acetate 0.21mol\nadd v1 water 0.14mol\nreact v1 esterification"),
    c("151-already-at-mass-action", "Does a mixture initially at the model's Q=K remain unchanged by a repeated equilibrium request?", "add v1 CH3COOH 0.01mol\nadd v1 ethanol 0.01mol\nadd v1 ethyl_acetate 0.04mol\nadd v1 water 0.01mol\nreact v1 esterification"),
    c("152-water-limited-reverse", "Does a product-only mixture with excess ester respect its smaller initial water supply?", "add v1 ethyl_acetate 0.048mol\nadd v1 water 0.012mol\nreact v1 esterification"),
    c("153-acetate-two-to-one-acid", "What pH does an acid-rich acetate buffer establish?", BUFFER + "\nmeasure v1 ph"),
    c("154-acetate-fourfold-dilution", "Does fourfold dilution preserve buffer-ratio pH approximately rather than shift it like strong acid?", "add v1 water 2L\nadd v1 CH3COOH 0.004mol\nadd v1 NaOAc 0.002mol\nmeasure v1 ph"),
    c("155-acetate-inverted-ratio", "Does inverting the acid/base ratio change pH by the logarithm of the ratio change?", "add v1 water 500mL\nadd v1 CH3COOH 0.002mol\nadd v1 NaOAc 0.004mol\nmeasure v1 ph"),
    c("156-acetate-small-acid-pulse", "Does a small strong-acid dose change the buffer ratio in the expected direction and magnitude?", BUFFER + "\nadd v1 HCl 0.0004mol\nmeasure v1 ph"),
    c("157-acetate-small-base-pulse", "Does an equal base dose perturb the same buffer in the opposite direction?", BUFFER + "\nadd v1 KOH 0.0004mol\nmeasure v1 ph"),
    c("158-acetate-extensive-scale", "Does tenfold scaling preserve the buffer's intensive pH?", "add v1 water 5L\nadd v1 CH3COOH 0.04mol\nadd v1 NaOAc 0.02mol\nmeasure v1 ph"),
]

if __name__ == "__main__":
    recorder.main()
