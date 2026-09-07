#!/usr/bin/env python3
"""Third discovery fleet (87–122), authored before observing any output.

Virtual bench inputs only. No production code imports these cases. Use the
shared recorder with a fresh --out directory after the preceding gate passes.
"""
import run as recorder

c = recorder.case

TERNARY = "add v1 water 3mol\nadd v1 methanol 0.3mol\nadd v1 isopropanol 0.2mol"
THIO = "add v1 water 100mL\nadd v1 HCl 0.001mol\nadd v1 Na2S2O3 0.001mol"
THERMAL = "add v1 water 1mol\nheat v1 300J\nnew\nadd v2 water 2mol\ncool v2 200J\nnew\nadd v3 water 3mol\nheat v3 100J\nnew"
CELL = "add v1 water 300mL\nadd v1 ZnSO4 0.003mol\nadd v1 Zn 0.001mol\nnew\nadd v2 water 300mL\nadd v2 CuSO4 0.003mol\nadd v2 Cu 0.001mol"
ELECTROLYTE = "add v1 water 250mL\nadd v1 KNO3 0.001mol"

recorder.CASES = [
    c("87-ternary-still", "Can one computed cut transport and conserve three volatile components?", TERNARY + "\nnew\ndistil v1 v2 0.2"),
    c("88-ternary-still-feed-order", "Does changing addition order leave the isobaric three-solvent cut unchanged?", "add v1 isopropanol 0.2mol\nadd v1 methanol 0.3mol\nadd v1 water 3mol\nnew\ndistil v1 v2 0.2"),
    c("89-ternary-still-scale", "Does scaling all three inventories tenfold scale the cut without changing its composition?", "add v1 water 30mol\nadd v1 methanol 3mol\nadd v1 isopropanol 2mol\nnew\ndistil v1 v2 0.2"),
    c("90-ternary-latent-budget", "Does an energy-limited ternary cut obey its explicitly latent-only 3 kJ budget?", TERNARY + "\nnew\ndistil v1 v2 3kJ stages 2"),
    c("91-ammonia-still-boundary", "Does a known aqueous volatile with no reviewed still model cause atomic refusal rather than false pure-water distillation?", "add v1 water 3mol\nadd v1 NH3 0.003mol\nnew\ndistil v1 v2 0.2"),
    c("92-pure-methanol-still", "Does the multicomponent solver reduce to the pure-component normal-boiling and molar-cut limit?", "add v1 methanol 0.4mol\nnew\ndistil v1 v2 0.25"),
    c("93-dilute-hbr", "Does a dilute finite HBr dose obey strong-acid concentration scaling?", "add v1 water 500mL\nadd v1 HBr 0.0001mol\nmeasure v1 ph"),
    c("94-matched-hcl", "Does replacing HBr with equimolar HCl preserve dilute strong-acid pH?", "add v1 water 500mL\nadd v1 HCl 0.0001mol\nmeasure v1 ph"),
    c("95-hbr-small-headspace", "Does finite HBr uptake conserve bromine in a small sealed headspace?", "add v1 water 250mL\nseal v1 50mL\nadd v1 HBr 0.001mol\nmeasure v1 ph\nmeasure v1 pressure"),
    c("96-hbr-large-headspace", "Does increasing only headspace retain total bromine while not increasing its dissolved share?", "add v1 water 250mL\nseal v1 500mL\nadd v1 HBr 0.001mol\nmeasure v1 ph\nmeasure v1 pressure"),
    c("97-hbr-base-first", "Can gaseous acid neutralize pre-existing sodium hydroxide without losing neutralization heat or bromine?", "add v1 water 500mL\nadd v1 NaOH 0.002mol\nadd v1 HBr 0.002mol\nmeasure v1 ph\nmeasure v1 temp"),
    c("98-hbr-acid-first", "Does the reversed finite-gas neutralization path reach the same material and thermal state?", "add v1 water 500mL\nadd v1 HBr 0.002mol\nadd v1 NaOH 0.002mol\nmeasure v1 ph\nmeasure v1 temp"),
    c("99-zinc-hydroxide", "Does a different divalent metal precipitate without exceeding its supplied inventory?", "add v1 water 300mL\nadd v1 ZnSO4 0.002mol\nadd v1 NaOH 0.004mol"),
    c("100-zinc-acid-reversal", "Does excess acid reverse the zinc hydroxide precipitation?", "add v1 water 300mL\nadd v1 ZnSO4 0.002mol\nadd v1 NaOH 0.004mol\nadd v1 HCl 0.005mol"),
    c("101-ferric-hydroxide", "Does trivalent iron respect three hydroxide equivalents and its total metal inventory?", "add v1 water 300mL\nadd v1 FeCl3 0.001mol\nadd v1 NaOH 0.003mol"),
    c("102-ferric-acid-reversal", "Does sufficient acid consume the ferric hydroxide solid rather than leave a cached precipitate?", "add v1 water 300mL\nadd v1 FeCl3 0.001mol\nadd v1 NaOH 0.003mol\nadd v1 HCl 0.02mol"),
    c("103-carbonate-liquid-feeds", "Can calcium carbonate precipitate from two dissolved feeds, not just a CO2 or chalk recipe?", "add v1 water 300mL\nadd v1 CaCl2 0.001mol\nadd v1 Na2CO3 0.001mol"),
    c("104-carbonate-dilution-control", "Does tenfold more solvent reduce the calcium carbonate solid amount from the same feeds?", "add v1 water 3L\nadd v1 CaCl2 0.001mol\nadd v1 Na2CO3 0.001mol"),
    c("105-nitrate-electrolysis", "Does an inert potassium nitrate electrolyte support Faraday-limited water electrolysis?", ELECTROLYTE + "\nelectrolyse v1 0.2A 60s"),
    c("106-electrolysis-equal-charge", "Does half the current for twice the time deliver the same products?", ELECTROLYTE + "\nelectrolyse v1 0.1A 120s"),
    c("107-electrolysis-double-charge", "Does twice the charge double each gas amount while water remains in excess?", ELECTROLYTE + "\nelectrolyse v1 0.2A 120s"),
    c("108-electrolysis-split-charge", "Does dividing a charge delivery into two commands preserve cumulative Faraday products?", ELECTROLYTE + "\nelectrolyse v1 0.2A 30s\nelectrolyse v1 0.2A 30s"),
    c("109-cell-operand-reversal", "Does the automatically oriented cell keep the same physical anode, cathode and emf when its operands are swapped?", CELL + "\ncell v1 v2\ncell v2 v1"),
    c("110-identical-half-cells", "Do identical zinc half-cells have zero emf, or an explicit zero-driving-force result?", "add v1 water 300mL\nadd v1 ZnSO4 0.003mol\nadd v1 Zn 0.001mol\nnew\nadd v2 water 300mL\nadd v2 ZnSO4 0.003mol\nadd v2 Zn 0.001mol\ncell v1 v2"),
    c("111-finite-acid-reference", "Does a partially acid-limited thiosulfate trajectory react within both material capacities?", THIO + "\nwait 10s"),
    c("112-finite-acid-scaled", "Does scaling vessel and inventories preserve a kinetic trajectory per unit amount?", "add v1 water 1L\nadd v1 HCl 0.01mol\nadd v1 Na2S2O3 0.01mol\nwait 10s"),
    c("113-finite-acid-partitioned-time", "Is the full-stack finite-acid trajectory stable to dividing elapsed time into five steps?", THIO + "\nwait 2s\nwait 2s\nwait 2s\nwait 2s\nwait 2s"),
    c("114-near-exhausted-acid", "Can a long wait avoid manufacturing acid or sulfur beyond a tiny supplied acid capacity?", "add v1 water 100mL\nadd v1 HCl 0.0001mol\nadd v1 Na2S2O3 0.001mol\nwait 1000s"),
    c("115-buffered-kinetics-boundary", "Does uncoupled proton-consuming kinetics explicitly withhold a weak-acid buffered trajectory?", "add v1 water 100mL\nadd v1 CH3COOH 0.001mol\nadd v1 NaOAc 0.001mol\nadd v1 Na2S2O3 0.001mol\nwait 10s"),
    c("116-nonacid-donor-control", "Does glucose avoid the conservative proton-donor guard while strong acid can still be consumed?", THIO + "\nadd v1 glucose 0.002mol\nwait 10s"),
    c("117-thermal-three-way-left", "Does serial mixing conserve sensible heat across a three-reservoir grouping?", THERMAL + "\nmix v1 1 v2 1 into v4\nnew\nmix v4 1 v3 1 into v5\nmeasure v5 temp"),
    c("118-thermal-three-way-right", "Does changing the parenthesization of three-water mixing preserve final temperature?", THERMAL + "\nmix v2 1 v3 1 into v4\nnew\nmix v1 1 v4 1 into v5\nmeasure v5 temp"),
    c("119-three-feed-acid-ledger", "Does a three-vessel acid/base/spectator MIX conserve equivalents and predict residual acidity?", "add v1 water 100mL\nadd v1 HCl 0.002mol\nnew\nadd v2 water 200mL\nadd v2 KOH 0.001mol\nnew\nadd v3 water 300mL\nadd v3 NaNO3 0.001mol\nnew\nmix v1 1 v2 1 into v4\nnew\nmix v4 1 v3 1 into v5\nmeasure v5 ph"),
    c("120-serial-aliquot-ledger", "Do two successive quarter-decants leave 9/16 of a dissolved spectator inventory?", "add v1 water 400mL\nadd v1 KCl 0.008mol\nnew\nnew\ndecant v1 v2 0.25\ndecant v1 v3 0.25"),
    c("121-four-component-equilibrium", "Does an initially mixed four-component ester system solve mass action from its actual state?", "add v1 CH3COOH 0.01mol\nadd v1 ethanol 0.02mol\nadd v1 ethyl_acetate 0.005mol\nadd v1 water 0.015mol\nreact v1 esterification"),
    c("122-four-component-equilibrium-scale", "Does scaling all four initial amounts preserve the dimensionless equilibrium quotient and scaled extent?", "add v1 CH3COOH 0.001mol\nadd v1 ethanol 0.002mol\nadd v1 ethyl_acetate 0.0005mol\nadd v1 water 0.0015mol\nreact v1 esterification"),
]

if __name__ == "__main__":
    recorder.main()
