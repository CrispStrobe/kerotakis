# Literal strong reagents -> certified strong acid/base reserve (2026-10-27)

Generalises PR #836. One shared, typed carrier table
(`crates/kerotakis-core/src/strong_reserve.rs`, `CARRIERS`) now backs both the
named saponification reserve (`named_alkaline.rs`) and the acid side of the
curated `acid_protons` rows (`curated.rs::acid_reserve`). The reserve is
analytical and charge-backed: spectator-cation equivalents minus strong-acid
anion equivalents (minus the acetate reservoir) on the base side; literal
strong-acid protons minus literal strong base and positive solute charge on the
acid side. It is never measured free OH-/H+ (the rejected free-OH design), and
weak buffers are never counted: their presence refuses (base side, named typed
detail) or blocks the certificate (acid side, the aqueous solve owns them).

Machine-checked row list: `crates/kerotakis-core/tests/fixtures/strong-reserve-coverage-20261027.json`,
enumerated from `curated::REACTIONS` and `curated::ORG_REACTIONS` by
`tests/strong_reserve_coverage.rs::literal_reagent_coverage_table_is_current`.

## Rows that name a literal strong reagent

| Row | Literal | Route | Notes |
|---|---|---|---|
| `NaOCl + 2 HCl -> Cl2 + NaCl + H2O` | HCl | `acid_protons` siblings (`NaOCl/ClO-/HClO + Cl- + n H+`) | literal HCl kept on its own rows |
| `ClO- + 2 HCl -> Cl2 + Cl- + H2O` | HCl | same siblings | |
| `HClO + HCl -> Cl2 + H2O` | HCl | same siblings | |
| `CH3COOC2H5 + NaOH -> NaOAc + C2H5OH` (saponification) | NaOH | named strong-base reserve | |

The audit's `curated.rs:193` (NaOH) and `:322` (KOH) are PRODUCTS of the
chloramine and permanganate/ethanol rows, not required reagents; nothing to
generalise there.

## Reserve forms x consumers

| Reserve form | Saponification (base side) | Bleach `acid_protons` rows (acid side) |
|---|---|---|
| NaOH (any condensed phase) | yes, 1 eq (keeps NaOAc molecular template) | subtracted as strong base |
| KOH | yes, 1 eq, K+ released | subtracted |
| Ba(OH)2 | **new**: 2 eq, Ba2+ released | **new**: subtracted |
| OH- / base_equivalents (dissolved) | yes, via spectator-cation excess | via positive solute charge |
| Na+/K+ with base_equivalents (solved caustic) | yes | via positive solute charge |
| Ba2+ (dissolved) | **new** spectator, 2 eq | n/a |
| HCl (liquid/aqueous) | anion eq | its own literal rows (not double counted) |
| HBr, HI | anion eq (HI **new**) | **new**: certified literal protons -> Br-/I- |
| HNO3 | anion eq | **new**: certified (when a pack defines HNO3; shipped registry lacks it) |
| H2SO4 | **new**: 2 anion eq | **not** certified (HSO4- pKa ~1.99; no HSO4- registry ion) |
| NaHSO4 | **new**: 1 cation + 2 anion eq | not certified |
| Cl-, Br-, I-, NO3-, SO4-2 (dissolved) | spectator anions (I-, SO4-2 **new**) | already via solute charge |
| NaCl, KCl, NaBr, KBr, KI, NaNO3, KNO3, Na2SO4, BaCl2 | neutral salts (KI, Na2SO4, BaCl2 **new**) | neutral |
| free acid on solved charge (-solute_charge) | n/a | unchanged (`unspent_acidity`) |
| CH3COOH / acetate | acetate reservoir (row-specific) | ledger weak acid (unchanged) |

## Never counted (weak buffers, typed refusal / blocked certificate)

carbonate (CO2, CO2(aq), CO3-2, HCO3-, Na2CO3, NaHCO3), ammonia (NH3, NH4+,
NH4Cl, NH4NO3), phosphate, borate, sulfite (incl. SO2), hypochlorite (except as
the bleach rows' own reactant), sulfide, organic acids (oxalic, citric, lactic,
malic, ascorbic). Ca(OH)2/Mg(OH)2 and their cations are excluded as
solubility-limited (not stoichiometric).

## Deferred

* Pre-solve molecular chloride (`NaCl` solid with `NaOCl` and a non-HCl acid):
  the `Cl-` siblings fire only after the aqueous tail has dissolved the salt
  (one step later). Not converted here.
* Sulfuric acid as an acid-side carrier needs a HSO4- registry ion and a
  reviewed second-dissociation treatment.
* Other ester/saponification rows: only ethyl acetate exists in
  `ORG_REACTIONS`.
