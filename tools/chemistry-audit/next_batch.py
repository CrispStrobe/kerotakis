#!/usr/bin/env python3
"""Second discovery batch: new virtual probes, not production reaction rules.

Uses the same process-isolated evidence recorder as the first audit. Choose a
fresh --out directory; successful execution alone is not a chemical verdict.
All authored inputs/checks are original; no external dataset is imported.
"""
import run as recorder

c = recorder.case
recorder.CASES = [
    c("51-ammonium-hydrolysis", "Does ammonium chloride acidify water without destroying nitrogen?", "add v1 water 500mL\nadd v1 NH4Cl 0.005mol\nmeasure v1 ph"),
    c("52-ammonium-buffer-acid", "Does an ammonia/ammonium buffer resist an acid perturbation?", "add v1 water 500mL\nadd v1 NH4Cl 0.01mol\nadd v1 NH3 0.01mol\nmeasure v1 ph\nadd v1 HCl 0.001mol\nmeasure v1 ph"),
    c("53-ammonium-buffer-base", "Does the matched buffer respond oppositely to base?", "add v1 water 500mL\nadd v1 NH4Cl 0.01mol\nadd v1 NH3 0.01mol\nmeasure v1 ph\nadd v1 NaOH 0.001mol\nmeasure v1 ph"),
    c("54-bisulfate-neutralisation", "Does one equivalent of KOH remove the bisulfate acidic proton?", "add v1 water 500mL\nadd v1 NaHSO4 0.005mol\nmeasure v1 ph\nadd v1 KOH 0.005mol\nmeasure v1 ph"),
    c("55-potassium-neutralisation", "Can potassium hydroxide neutralize HBr without a NaOH-specific implementation?", "add v1 water 500mL\nadd v1 HBr 0.003mol\nadd v1 KOH 0.003mol\nmeasure v1 ph"),
    c("56-spectator-ion-mixture", "Does soluble KCl plus sodium nitrate remain a neutral ionic mixture?", "add v1 water 500mL\nadd v1 KCl 0.005mol\nadd v1 NaNO3 0.005mol\nmeasure v1 ph"),
    c("57-gypsum-calcium-first", "Does concentrated calcium/sulfate precipitate hydrated gypsum?", "add v1 water 200mL\nadd v1 CaCl2 0.01mol\nadd v1 Na2SO4 0.01mol"),
    c("58-gypsum-sulfate-first", "Does reversing calcium/sulfate feed order preserve gypsum yield?", "add v1 water 200mL\nadd v1 Na2SO4 0.01mol\nadd v1 CaCl2 0.01mol"),
    c("59-magnesium-hydroxide", "Does magnesium sulfate plus hydroxide give a bounded solid yield?", "add v1 water 300mL\nadd v1 MgSO4 0.003mol\nadd v1 NaOH 0.006mol"),
    c("60-magnesium-redissolution", "Does acid reverse magnesium hydroxide precipitation?", "add v1 water 300mL\nadd v1 MgSO4 0.003mol\nadd v1 NaOH 0.006mol\nadd v1 HCl 0.006mol"),
    c("61-carbonate-acid-sealed", "Does acid consume chalk in a sealed vessel without losing calcium?", "add v1 water 300mL\nseal v1 400mL\nadd v1 CaCO3 0.002mol\nadd v1 HCl 0.004mol"),
    c("62-carbonate-no-acid", "Does the matched sealed chalk control retain much more solid?", "add v1 water 300mL\nseal v1 400mL\nadd v1 CaCO3 0.002mol"),
    c("63-filter-dissolved-sugar", "Does filtration pass dissolved glucose rather than misclassify it as solid?", "add v1 water 300mL\nadd v1 glucose 0.005mol\nnew\nfilter v1 v2"),
    c("64-filter-sand-and-salt", "Does filtration retain silica while passing dissolved KCl?", "add v1 water 300mL\nadd v1 SiO2 0.005mol\nadd v1 KCl 0.003mol\nnew\nfilter v1 v2"),
    c("65-mix-magnesium-solutions", "Does fractional solution MIX conserve all vessel inventories?", "add v1 water 200mL\nadd v1 MgSO4 0.002mol\nnew\nadd v2 water 400mL\nadd v2 NaCl 0.004mol\nnew\nmix v1 0.25 v2 0.75 into v3"),
    c("66-mix-reversed-solutions", "Does swapping MIX operands leave the receiver unchanged?", "add v1 water 200mL\nadd v1 MgSO4 0.002mol\nnew\nadd v2 water 400mL\nadd v2 NaCl 0.004mol\nnew\nmix v2 0.75 v1 0.25 into v3"),
    c("67-evaporate-potassium-salt", "Does solvent removal conserve nonvolatile potassium/chloride?", "add v1 water 300mL\nadd v1 KCl 0.005mol\nevaporate v1 0.5"),
    c("68-evaporate-glucose", "Does solvent removal concentrate a nonionic solute without ionizing it?", "add v1 water 300mL\nadd v1 glucose 0.005mol\nevaporate v1 0.5\nmeasure v1 ph"),
    c("69-reverse-strong-titration", "Can coarse HCl titration refine a downward pH crossing for KOH?", "add v1 water 300mL\nadd v1 KOH 0.0023mol\ntitrate v1 HCl 0.1M 7mL until ph 7\nmeasure v1 ph"),
    c("70-weak-base-titration", "Can ammonia be titrated down to an acidic ammonium endpoint?", "add v1 water 300mL\nadd v1 NH3 0.002mol\ntitrate v1 HCl 0.1M 3mL until ph 5\nmeasure v1 ph"),
    c("71-lactic-half-neutralisation", "Does partial neutralisation raise a second organic acid's pH?", "add v1 water 300mL\nadd v1 lactic_acid 0.006mol\nmeasure v1 ph\nadd v1 NaOH 0.003mol\nmeasure v1 ph"),
    c("72-citric-proton-ladder", "Does a tricarboxylic acid respond monotonically to successive equivalent doses?", "add v1 water 300mL\nadd v1 citric_acid 0.002mol\nmeasure v1 ph\nadd v1 KOH 0.002mol\nmeasure v1 ph\nadd v1 KOH 0.002mol\nmeasure v1 ph\nadd v1 KOH 0.002mol\nmeasure v1 ph"),
    c("73-phosphate-buffer-acid", "Does phosphate buffer resist a small acid challenge after preparation?", "add v1 water 500mL\nadd v1 H3PO4 0.004mol\nadd v1 NaOH 0.006mol\nmeasure v1 ph\nadd v1 HCl 0.0002mol\nmeasure v1 ph"),
    c("74-phosphate-buffer-base", "Does the matched phosphate buffer respond oppositely to base?", "add v1 water 500mL\nadd v1 H3PO4 0.004mol\nadd v1 NaOH 0.006mol\nmeasure v1 ph\nadd v1 NaOH 0.0002mol\nmeasure v1 ph"),
    c("75-hot-cold-mixing", "Does mixing warm and cool water give an intermediate temperature?", "add v1 water 200mL\nheat v1 8kJ\nnew\nadd v2 water 400mL\ncool v2 4kJ\nnew\nmix v1 1 v2 1 into v3\nmeasure v3 temp"),
    c("76-hot-cold-reversed", "Is thermal MIX invariant to operand ordering?", "add v1 water 200mL\nheat v1 8kJ\nnew\nadd v2 water 400mL\ncool v2 4kJ\nnew\nmix v2 1 v1 1 into v3\nmeasure v3 temp"),
    c("77-ester-product-loaded", "Does initially supplied ester reduce net forward conversion?", "add v1 CH3COOH 0.01mol\nadd v1 ethanol 0.01mol\nadd v1 ethyl_acetate 0.02mol\nreact v1 esterification"),
    c("78-ester-water-loaded", "Does initially supplied water alter the equilibrium extent?", "add v1 CH3COOH 0.01mol\nadd v1 ethanol 0.01mol\nadd v1 water 0.02mol\nreact v1 esterification"),
    c("79-ester-reverse-request", "Can an equilibrium request hydrolyse a product-only starting mixture?", "add v1 water 0.01mol\nadd v1 ethyl_acetate 0.01mol\nreact v1 esterification"),
    c("80-ester-reverse-passive", "Does waiting avoid silently assuming ester hydrolysis kinetics?", "add v1 water 0.01mol\nadd v1 ethyl_acetate 0.01mol\nwait 1h"),
    c("81-conductivity-potassium", "Does a dilute potassium salt conduct?", "add v1 water 300mL\nadd v1 KCl 0.003mol\nmeasure v1 conductivity"),
    c("82-conductivity-glucose", "Does matched molecular glucose conduct far less than KCl?", "add v1 water 300mL\nadd v1 glucose 0.003mol\nmeasure v1 conductivity"),
    c("83-distil-isopropanol", "Does a second alcohol's volatility enrich a dilute receiver?", "add v1 water 4mol\nadd v1 isopropanol 0.2mol\nnew\ndistil v1 v2 0.1"),
    c("84-distil-methanol", "Does a third alcohol's distillation conserve molecules across both vessels?", "add v1 water 4mol\nadd v1 methanol 0.2mol\nnew\ndistil v1 v2 0.1"),
    c("85-gypsum-dilution-control", "Does the same calcium/sulfate amount precipitate less in tenfold more water?", "add v1 water 2L\nadd v1 CaCl2 0.01mol\nadd v1 Na2SO4 0.01mol"),
    c("86-ammonium-nitrate-cooling", "Does a second cold-pack salt cool without an invented redox reaction?", "add v1 water 300mL\nadd v1 NH4NO3 0.015mol\nmeasure v1 temp"),
]

if __name__ == "__main__":
    recorder.main()
