# Dissolution, hydration and reaction-heat ownership coverage (2026-10-27)

Generated from the shipped registry and the vendored USGS databases; `crates/kerotakis-phreeqc/tests/dissolution_enthalpy_coverage.rs` recomputes every row and fails if this file or `crates/kerotakis-phreeqc/tests/fixtures/dissolution-enthalpy-coverage-20261027.json` goes stale (`KERO_BLESS_COVERAGE=1` rewrites both).

## How a solid is owned

One chain, `kerotakis_phreeqc::enthalpy::solid_route`, in this order:

1. `curated` - the registry enthalpy of dissolution with the `DISSOCIATION` products.
2. `hydration:<source>:<hydroxide>` - an anhydrous oxide priced through its registry hydroxide; the two stablest phases of ONE database must balance and differ by exactly `n H2O` (`oxide + n H2O = k hydroxide`).
3. `routed` - the routed database's own neutral dissociation phase.
4. `reference:<source>:<phase>` - the stablest phase of the routed database, then wateq4f, minteq.v4, pitzer, phreeqc.dat, llnl.dat, that passes every check: stated finite non-zero 25 C `delta_h` (exact zero is a placeholder); no `e-` or elemental redox partner; atom and charge balance over every term including H+, H2O and hydrate water; every species defined on the routed database; and every species the routed database prices by a reaction (non-basis) written identically, in either direction, by the source with log K within 0.1 and delta_h within 2 kJ/mol.

Anything else is a whole-step refusal (`not-modeled.aqueous-heat-unpriced`) naming the solid. Domain: 25 C, 1 bar reference reaction enthalpies, as every other term of the balance; no heat-capacity extrapolation. llnl.dat (LLNL thermo.com.V8.R6.230 as converted for PHREEQC, public domain) is carried only as the extracted rows in `crates/kerotakis-phreeqc/data/llnl-dissolution-references-20261027.json`.

## Counts (registry solids, 93)

| routed database | priced before (ef9e9632) | priced after |
|---|---|---|
| wateq4f | 20 | 41 |
| minteq.v4 | 22 | 42 |
| pitzer | 13 | 22 |

Priced after this change: wateq4f 41/93, minteq.v4 42/93, pitzer 22/93.

The "before" column counts the routes that were already owned at ef9e9632 (curated, the CaO hydration row, the routed neutral mineral); those paths are unchanged, so it is read off this table rather than recomputed.

## Every registry solid

| solid | wateq4f | minteq.v4 | pitzer |
|---|---|---|---|
| `Ag` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `AgCl` | curated | curated | refused: a dissociation product the routed database does not define |
| `AgNO3` | curated | curated | refused: a dissociation product the routed database does not define |
| `Al` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Ba(OH)2` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `BaCl2` | reference:llnl.dat:BaCl2 | reference:llnl.dat:BaCl2 | reference:llnl.dat:BaCl2 |
| `BaSO4` | routed | routed | routed |
| `C` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Ca(OH)2` | curated | curated | curated |
| `Ca3(PO4)2` | reference:minteq.v4:Ca3(PO4)2(beta) | routed | refused: a reaction species is not defined on the routed database |
| `CaCO3` | routed | routed | routed |
| `CaCl2` | curated | curated | curated |
| `CaO` | hydration:minteq.v4:Ca(OH)2 | hydration:minteq.v4:Ca(OH)2 | hydration:minteq.v4:Ca(OH)2 |
| `Cu` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Cu(OH)2` | reference:wateq4f:Cu(OH)2 | reference:minteq.v4:Cu(OH)2 | refused: a reaction species is not defined on the routed database |
| `CuO` | hydration:wateq4f:Cu(OH)2 | hydration:wateq4f:Cu(OH)2 | refused: a reaction species is not defined on the routed database |
| `CuSO4` | curated | curated | refused: a dissociation product the routed database does not define |
| `Fe` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Fe(OH)2` | reference:llnl.dat:Fe(OH)2 | refused: the reviewed phase states an exact-zero enthalpy placeholder | reference:llnl.dat:Fe(OH)2 |
| `Fe(OH)3` | refused: the reviewed phase states no 25 C dissolution enthalpy | reference:minteq.v4:Ferrihydrite | refused: the reviewed phase states no 25 C dissolution enthalpy |
| `Fe2O3` | reference:wateq4f:Hematite | hydration:minteq.v4:Fe(OH)3 | refused: a reaction species is not defined on the routed database |
| `FeCl3` | refused: a product's convention differs between the reviewed source and the routed database | reference:llnl.dat:Molysite | refused: a reaction species is not defined on the routed database |
| `FeSO4` | reference:llnl.dat:FeSO4 | refused: a product's convention differs between the reviewed source and the routed database | reference:llnl.dat:FeSO4 |
| `H2C2O4` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `I2` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `KCl` | curated | curated | curated |
| `KI` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `KIO3` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `KMnO4` | curated | curated | refused: a dissociation product the routed database does not define |
| `KNO3` | reference:llnl.dat:Niter | reference:llnl.dat:Niter | refused: a reaction species is not defined on the routed database |
| `KOH` | curated | curated | curated |
| `Mg` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Mg(OH)2` | reference:wateq4f:Brucite | reference:minteq.v4:Brucite | routed |
| `MgO` | hydration:minteq.v4:Mg(OH)2 | hydration:minteq.v4:Mg(OH)2 | hydration:minteq.v4:Mg(OH)2 |
| `MgSO4` | reference:llnl.dat:MgSO4 | reference:llnl.dat:MgSO4 | reference:llnl.dat:MgSO4 |
| `MnO2` | refused: the reviewed phase dissolves by redox, not by dissociation | refused: the reviewed phase states an exact-zero enthalpy placeholder | refused: the reviewed phase dissolves by redox, not by dissociation |
| `NH4Cl` | curated | curated | refused: a dissociation product the routed database does not define |
| `NH4NO3` | curated | curated | refused: a dissociation product the routed database does not define |
| `Na2B4O7` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `Na2CO3` | curated | curated | curated |
| `Na2S2O3` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `Na2SO3` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `Na2SO4` | curated | curated | curated |
| `NaBr` | reference:llnl.dat:NaBr | reference:llnl.dat:NaBr | reference:llnl.dat:NaBr |
| `NaCl` | curated | curated | curated |
| `NaHCO3` | curated | curated | curated |
| `NaHSO3` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `NaHSO4` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `NaNO3` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `NaOAc` | refused: a dissociation product the routed database does not define | curated | refused: a dissociation product the routed database does not define |
| `NaOH` | curated | curated | curated |
| `PE` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `PET` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `PP` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `PS` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `PVA` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `Pb` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Pb(NO3)2` | curated | curated | refused: a dissociation product the routed database does not define |
| `S` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `SiO2` | reference:wateq4f:Quartz | reference:minteq.v4:Quartz | reference:wateq4f:Quartz |
| `Zn` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `Zn(OH)2` | reference:minteq.v4:Zn(OH)2(epsilon) | reference:minteq.v4:Zn(OH)2(epsilon) | refused: the reviewed phase states no 25 C dissolution enthalpy |
| `ZnSO4` | curated | curated | refused: a dissociation product the routed database does not define |
| `activated_charcoal` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `antlerite` | refused: the reviewed phase states no 25 C dissolution enthalpy | refused: the reviewed phase states an exact-zero enthalpy placeholder | refused: the reviewed phase states no 25 C dissolution enthalpy |
| `ascorbic_acid` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `atacamite` | reference:wateq4f:Atacamite | reference:minteq.v4:Atacamite | refused: a reaction species is not defined on the routed database |
| `brochantite` | reference:minteq.v4:Brochantite | reference:minteq.v4:Brochantite | refused: the reviewed phase states no 25 C dissolution enthalpy |
| `cellulose` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `chalcanthite` | reference:wateq4f:Chalcanthite | reference:minteq.v4:Chalcanthite | refused: a reaction species is not defined on the routed database |
| `chlorophyll` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `citric_acid` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `dehydroascorbic_acid` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `diamond` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `dry_ice` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `epsomite` | reference:wateq4f:Epsomite | reference:minteq.v4:Epsomite | reference:wateq4f:Epsomite |
| `fructose` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `glucose` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `graphite` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |
| `gypsum` | reference:wateq4f:Gypsum | reference:minteq.v4:Gypsum | reference:wateq4f:Gypsum |
| `hair_pigment` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `hair_pigment_ox` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `hydroxylapatite` | reference:wateq4f:Hydroxyapatite | reference:wateq4f:Hydroxyapatite | refused: a reaction species is not defined on the routed database |
| `langite` | reference:wateq4f:Langite | reference:minteq.v4:Langite | refused: a reaction species is not defined on the routed database |
| `malic_acid` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `maltose` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `naphthalene` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `nylon` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `octacalcium_phosphate` | refused: the reviewed phase states an exact-zero enthalpy placeholder | refused: the reviewed phase states an exact-zero enthalpy placeholder | refused: the reviewed phase states an exact-zero enthalpy placeholder |
| `paraffin` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `starch` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `sucrose` | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid | refused: no vendored USGS database writes a dissolution reaction for this solid |
| `uranium` | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving | refused: an elemental or condensed-gas solid changes state by redox or phase routing, not by dissolving |

## Reaction heats per declared extent

`kerotakis_core::reaction_heat` owns every heat-carrying reaction row: a reference heat is stated with its basis (per extent, or per mole of a named participant) and the heat per extent is derived as reference x |coefficient| in the equation the integrator advances. `reaction_heat::audit()` enumerates the rows; its tests fail on an unresolved basis, a non-finite per-extent heat, or a curated kinetic row that releases heat outside the table.

| family | row | basis | coefficient | per extent |
|---|---|---|---|---|
| kinetics | `peroxide-decomposition` | 98.2 kJ per mol H2O2 (PR #850) | 2 | 196.4 kJ |
| combustion | `paraffin` | -dHc per mol fuel | 1 | 16224 kJ |
| combustion | `cellulose` | -dHc per mol fuel | 1 | 2810 kJ |
| combustion | `sucrose` | -dHc per mol fuel | 1 | 5640 kJ |
| combustion | `dodecane` | -dHc per mol fuel | 1 | 8087 kJ |
| combustion | `hexadecane` | -dHc per mol fuel | 1 | 10699 kJ |
| gas-pack:h2-o2-skeletal-v1 | every reaction with formation enthalpies | per extent (sum of coefficient x dfH) | as written | derived |
| gas-pack:co-h2-wet-v1 | every reaction with formation enthalpies | per extent (sum of coefficient x dfH) | as written | derived |
| gas-pack:hydrocarbon-global-v1 | every reaction with formation enthalpies | per extent (sum of coefficient x dfH) | as written | derived |

No other curated kinetic reaction carries a reaction heat; those rows release none.

