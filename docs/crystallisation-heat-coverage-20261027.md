# Crystallisation heat on cooling: coverage (2026-10-27)

Lane: when an aqueous solution is cooled, own solid-phase saturation and solvent freezing with their heats in the same energy ledger, with an explicit domain and typed refusals outside it. Motivating case: TH-007 (50 g water + 30 g KNO3 at 90 C, `cool v1 12kJ`), which on the heat-ownership branch (draft PR #867) cools on dissolution to 321.76 K and then, without this lane, falls to 264.5 K with neither KNO3 crystals nor ice.

## Mechanism (`crates/kerotakis-core/src/crystallisation.rs`)

A salt is owned when three independent facts hold: a reviewed measured pure-water solubility curve (`CURVES`) with its temperature domain; the dissolved salt booked on its ions; and a priced enthalpy of dissolution from the routed engine's registered pricer (`register_dissolution_pricer`), falling back to the registry's curated `dissolution_enthalpy_kj`. A `cool` dose is then solved for the final temperature `T_f` from

    Q = H_sensible(T_f -> T_0, contents after crystallising n) + n(T_f) * dH_diss,   n(T) = max(0, dissolved - capacity(T))

which is monotone (every shipped curve is checked to rise with temperature), so bisection finds its single root. The `cool` arm in `bench.rs` applies the result (ions -> solid, `Precipitated`, `EnergyTransferred` with `sensible_j` reporting the thermometer part). The phreeqc partition/rebuild (`aqueous.rs`) holds such crystals out of the solver while the solution is saturated, so the next aqueous step does not dissolve them again.

Domain and refusals (`Refusal`, localized EN/DE/FR under `not-modeled.*`):

| refusal | when |
|---|---|
| `crystallisation-heat-unpriced` | saturation is reached but no dissolution enthalpy is priced on the route |
| `crystallisation-below-curve-domain` | the dose would carry a saturated solution below the lowest measured point (toward the salt-ice eutectic) |
| `crystallisation-not-binary` | other dissolved ions or an unpaired excess of the salt's own ion (common ion): the binary curve does not apply |

An unsaturated solution never enters the solver: its cooling is byte-identical to before.

## Freeze paths

Solvent freezing is not duplicated. Brine freezing with latent heat, the shared fusion authority and the 252 K admissibility cap stay owned by `solve::StateEquilibrator` (PRs #852/#857, `docs/liquidus-thermodynamic-consistency-lane-20261021.md`). The crystallisation ledger refuses before the eutectic, so for a curve salt the two never meet: KNO3 is refused below 273.55 K (its eutectic with ice, about -2.9 C, is not modelled). NaCl has no curve here; halite saturation stays with the routed database phase and its freezing plateau with #852/#857 (tested unchanged).

## Curves

| salt | source | points | domain |
|---|---|---|---|
| KNO3 | Earl of Berkeley, Phil. Trans. R. Soc. A 203 (1904) 189-215, doi:10.1098/rsta.1904.0020, potassium nitrate table column II (public domain scan, archive.org philosophicaltr203royauoft) | 7 | 0.40-91.65 C |

The boiling-point row (311.64) is not carried: no temperature is printed for it. The OCR reads the sixth temperature as `7G-00`; it is taken as 76.00 C (bracketed by its neighbours and the printed Comey comparison 159.0). The paper also tabulates other nitrates and chlorides; they are not shipped because their OCR is not clean enough to transcribe without a manual page check, and they are listed below as gaps.

## Counts (registry solids, 93)

| class | count | meaning |
|---|---|---|
| owned | 0 | curve + registry-curated dissolution enthalpy |
| curve-unpriced | 1 | curve shipped; heat priced only once the routed engine registers its pricer (PR #867 reference route, llnl.dat Niter 35.4794 kJ/mol); refused on main |
| two-point-cap | 5 | older `saturation_moves` 20/100 C line on molecular portions, no heat |
| single-point-cap | 26 | older temperature-independent cap, no heat |
| none | 61 | no reviewed temperature-dependent curve |

Before this change: 0 solids crystallised on cooling with priced heat. After it, on main: 0 owned (KNO3 refuses as unpriced, honestly); after rebasing onto #867 and registering its pricer: KNO3 owned.

Generated from `kerotakis_core::crystallisation::coverage()`; `crates/kerotakis-core/tests/crystallisation_coverage.rs` fails if this table or `crates/kerotakis-core/tests/fixtures/crystallisation-heat-coverage-20261027.json` goes stale.

## Every registry solid

| solid | class | detail |
|---|---|---|
| `Ag` | none | no reviewed temperature-dependent solubility curve |
| `AgCl` | two-point-cap | saturation_moves 20/100 C line; molecular portion only, no heat |
| `AgNO3` | none | no reviewed temperature-dependent solubility curve |
| `Al` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `Ba(OH)2` | none | no reviewed temperature-dependent solubility curve |
| `BaCl2` | none | no reviewed temperature-dependent solubility curve |
| `BaSO4` | two-point-cap | saturation_moves 20/100 C line; molecular portion only, no heat |
| `C` | none | no reviewed temperature-dependent solubility curve |
| `Ca(OH)2` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `Ca3(PO4)2` | none | no reviewed temperature-dependent solubility curve |
| `CaCO3` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `CaCl2` | none | no reviewed temperature-dependent solubility curve |
| `CaO` | none | no reviewed temperature-dependent solubility curve |
| `Cu` | none | no reviewed temperature-dependent solubility curve |
| `Cu(OH)2` | none | no reviewed temperature-dependent solubility curve |
| `CuO` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `CuSO4` | none | no reviewed temperature-dependent solubility curve |
| `Fe` | none | no reviewed temperature-dependent solubility curve |
| `Fe(OH)2` | none | no reviewed temperature-dependent solubility curve |
| `Fe(OH)3` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `Fe2O3` | none | no reviewed temperature-dependent solubility curve |
| `FeCl3` | none | no reviewed temperature-dependent solubility curve |
| `FeSO4` | none | no reviewed temperature-dependent solubility curve |
| `H2C2O4` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `I2` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `KCl` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `KI` | none | no reviewed temperature-dependent solubility curve |
| `KIO3` | none | no reviewed temperature-dependent solubility curve |
| `KMnO4` | none | no reviewed temperature-dependent solubility curve |
| `KNO3` | curve-unpriced | 7 measured points, 273.55-364.80 K, binary with K+/NO3-; priced only where the routed engine registers a dissolution pricer (PR #867 llnl reference), refused otherwise |
| `KOH` | none | no reviewed temperature-dependent solubility curve |
| `Mg` | none | no reviewed temperature-dependent solubility curve |
| `Mg(OH)2` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `MgO` | none | no reviewed temperature-dependent solubility curve |
| `MgSO4` | none | no reviewed temperature-dependent solubility curve |
| `MnO2` | none | no reviewed temperature-dependent solubility curve |
| `NH4Cl` | none | no reviewed temperature-dependent solubility curve |
| `NH4NO3` | none | no reviewed temperature-dependent solubility curve |
| `Na2B4O7` | two-point-cap | saturation_moves 20/100 C line; molecular portion only, no heat |
| `Na2CO3` | none | no reviewed temperature-dependent solubility curve |
| `Na2S2O3` | none | no reviewed temperature-dependent solubility curve |
| `Na2SO3` | none | no reviewed temperature-dependent solubility curve |
| `Na2SO4` | none | no reviewed temperature-dependent solubility curve |
| `NaBr` | none | no reviewed temperature-dependent solubility curve |
| `NaCl` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `NaHCO3` | none | no reviewed temperature-dependent solubility curve |
| `NaHSO3` | none | no reviewed temperature-dependent solubility curve |
| `NaHSO4` | none | no reviewed temperature-dependent solubility curve |
| `NaNO3` | none | no reviewed temperature-dependent solubility curve |
| `NaOAc` | none | no reviewed temperature-dependent solubility curve |
| `NaOH` | none | no reviewed temperature-dependent solubility curve |
| `PE` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `PET` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `PP` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `PS` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `PVA` | none | no reviewed temperature-dependent solubility curve |
| `Pb` | none | no reviewed temperature-dependent solubility curve |
| `Pb(NO3)2` | none | no reviewed temperature-dependent solubility curve |
| `S` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `SiO2` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `Zn` | none | no reviewed temperature-dependent solubility curve |
| `Zn(OH)2` | none | no reviewed temperature-dependent solubility curve |
| `ZnSO4` | none | no reviewed temperature-dependent solubility curve |
| `activated_charcoal` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `antlerite` | none | no reviewed temperature-dependent solubility curve |
| `ascorbic_acid` | none | no reviewed temperature-dependent solubility curve |
| `atacamite` | none | no reviewed temperature-dependent solubility curve |
| `brochantite` | none | no reviewed temperature-dependent solubility curve |
| `cellulose` | none | no reviewed temperature-dependent solubility curve |
| `chalcanthite` | none | no reviewed temperature-dependent solubility curve |
| `chlorophyll` | none | no reviewed temperature-dependent solubility curve |
| `citric_acid` | none | no reviewed temperature-dependent solubility curve |
| `dehydroascorbic_acid` | none | no reviewed temperature-dependent solubility curve |
| `diamond` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `dry_ice` | none | no reviewed temperature-dependent solubility curve |
| `epsomite` | none | no reviewed temperature-dependent solubility curve |
| `fructose` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `glucose` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `graphite` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `gypsum` | two-point-cap | saturation_moves 20/100 C line; molecular portion only, no heat |
| `hair_pigment` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `hair_pigment_ox` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `hydroxylapatite` | none | no reviewed temperature-dependent solubility curve |
| `langite` | none | no reviewed temperature-dependent solubility curve |
| `malic_acid` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `maltose` | none | no reviewed temperature-dependent solubility curve |
| `naphthalene` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `nylon` | none | no reviewed temperature-dependent solubility curve |
| `octacalcium_phosphate` | none | no reviewed temperature-dependent solubility curve |
| `paraffin` | none | no reviewed temperature-dependent solubility curve |
| `starch` | single-point-cap | saturation_moves temperature-independent cap; no heat |
| `sucrose` | two-point-cap | saturation_moves 20/100 C line; molecular portion only, no heat |
| `uranium` | none | no reviewed temperature-dependent solubility curve |

## Remaining gaps

- More reviewed curves (KCl, NaNO3, Pb(NO3)2, CuSO4 hydrates, alum, ...) need a manual page transcription; hydrate-forming salts also need the hydrate as a registry solid.
- Partial dissolution on ADDING a curve salt to an unsaturated solution beyond its capacity still dissolves completely through the routed engine's `Dissolves` role (KNO3 lane task 1).
- Evaporation-driven crystallisation is not routed through this ledger yet; the same `solve` applies with water removal in place of the dose.
- The salt-ice eutectic and concentrated-nitrate activities (no Pitzer nitrate data is vendored) are refused, not modelled.
