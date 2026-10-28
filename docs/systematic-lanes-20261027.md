# Systematic lanes, October 2026

PR #873 folded every lane opened by the remaining-129 production triage into one
integration. Each lane follows the same contract: an enumerated coverage table with a
checked-in fixture and a stale check (regenerate with the lane's BLESS variable), one
generic mechanism instead of per-case patches, and typed refusals in EN/DE/FR where the
data does not support an answer.

| Lane | Coverage table | Mechanism |
|------|----------------|-----------|
| Gas phases across routes | [gas-phase-coverage](gas-phase-coverage-20261027.md) | gases reconciled onto the routed database; gas-model mixing guard; `GasPhaseUnavailable` |
| Dissolution and hydration enthalpy | [dissolution-enthalpy-coverage](dissolution-enthalpy-coverage-20261027.md) | one ownership chain per solid (curated → hydration → routed mineral → reviewed reference) |
| Whole-step heat ledger | (this document, below) | partial ledgers refused; unpriced movers bounded by element-family ceiling below thermometer precision, decided once per step |
| Crystallisation heat | [crystallisation-heat-coverage](crystallisation-heat-coverage-20261027.md) | routed phase solubility; Debye–Hückel routes refused above I = 1 mol/kgw |
| Halogen redox and iodometry | [halogen-redox-coverage](halogen-redox-coverage-20261027.md) | enumerated couple pairs; admitted, owned elsewhere or refused |
| Chromophores and instruments | [chromophore-coverage](chromophore-coverage-20261027.md) | spectrophotometer refuses unparameterised chromophores |
| Metal–ligand routing | [metal-ligand-routing-coverage](metal-ligand-routing-coverage-20261027.md) | families that decide the route; refused families stated |
| Input aliases | [input-alias-coverage](input-alias-coverage-20261027.md) | generated aliases for typed input only; REST multi-word list derived from the same table |
| Strong reserve and media | [reserve-coverage](reserve-coverage-20261027.md), [medium-admission-coverage](medium-admission-coverage-20261027.md) | one carrier table; phase-aware withdrawal with a lint |
| Model domains | [model-domain-refusal-coverage](model-domain-refusal-coverage-20261027.md) | refuse instead of clamping (CEA record ranges, luminol) |

## Whole-step heat ledger

A step's heat is charged only when every species that moved has a reviewed enthalpy on the
routed database, with one bounded exception. A mover with no reviewed enthalpy is bounded at
|Δn| × the largest |ΔH| the database states for any phase containing all its heavy elements
(falling back to the database-wide ceiling). If the total bound would move the vessel by less
than the bench thermometer's stated precision (0.1 K), the priced heat is charged; otherwise
the step states `not-modeled.aqueous-heat-unpriced`. The decision is made once per step and
held across the thermal fixed point, which otherwise oscillated.

Known consequences, each recorded rather than hidden:

- Thiosulfate (S2O3-2) and hypochlorous acid (HClO) have no stated enthalpy in any shipped
  database, so steps that move them in quantity state the refusal.
- Bleach and betanin reach the same composition in either order but differ by 1.8–2.6 mK:
  one order bounds the HClO trace and charges the dissolution, the other refuses
  (`corpus_perturbation` ORDER_DEPARTURES `bio-094`, `th-077`).
- An electrolysis step's heat is not in this ledger; the products are still a computed answer.

## Follow-up lanes

Integrated by PR #873 as `abf4e9a34eb16e51f605a2009e542b62002a30ea`. Each lane below must meet the same contract before it closes.

| # | Lane | Trigger case(s) | Current state | Done when |
|---|------|-----------------|---------------|-----------|
| 1 | Concentrated-electrolyte activity for crystallisation | TH-007 (KNO3) | DH routes refused above I>1; llnl Niter solubility -51..-73%; only pitzer halite validated (-0.49%) | Pitzer (or SIT) parameters for nitrate/sulfate/chloride families admitted per salt; crystallisation fixture shows each salt priced on a validated route within a stated tolerance, others refused |
| 2 | FeOs azeotrope citation | ethanol–water | Bound cites in-repo kerotakis-thermo control; Kamihama 2012 doi:10.1021/je2008704 unconfirmed | Primary source confirmed (or replaced) in parameter provenance; FeOs stays optional |
| 3 | Acid-gas headspace coupling | H2S/SO2/HCl/Cl2 | Gas reconciliation covers NH3/H2 on minteq; acid gases not coupled | Reviewed Henry/logK data per gas and route in gas-phase coverage table; unreviewed gases refuse |
| 4 | Ammine complexes from llnl | zinc ammine (and Cu/Ni/Co/Ag/Cd ammines) | ligand_routing: 474 families, 5 refused | Ammine family imported with convention checks; route table regenerated |
| 5 | Chromophore optics | uvvis on unparameterised species | Spectrophotometer refuses (instrument_coverage.rs) | Cited molar absorptivities per chromophore family; coverage table counts priced vs refused |
| 6 | Deferred from #872 | UNIFAC/DH domain refusals; remaining phase-less withdraw sites | Lint exists for Vessel::withdraw_phases | All withdraw sites phase-aware (lint zero exceptions); UNIFAC/DH out-of-domain refusals typed |
| 7 | API ↔ engine derived tables | REST multi-word aliases (fixed in be2b5f5) | Generator + test now | Audit other hand-kept API lists (verbs, units, loop words) for the same drift; derive each from engine metadata |
| 9 | Phase answer for refused mixtures | sweep: Ag/thiosulfate cooled to 250 K stays liquid | phase_without_aqueous forbids invented phase answers; sweep exempts SolverFailed steps | Bounded brine-freezing model with typed uncertainty, or a refusal of the cooling step itself |
| 10 | Metal phases by routed name | plating: Cu(s) unpriced though wateq4f has CuMetal | name mismatch only | Registry metals map to each route's metal phase (CuMetal, ZnMetal, ...) with convention checks; coverage table |
| 11 | HClO protonation enthalpy | corpus order test bio-094, th-077 recorded departures | no shipped database states it (llnl "not possible") | Reviewed source for H+ + ClO- = HClO dH; then delete both recorded departures |
| 12 | Ignition energy into vapourised fuel | th-038 hexane+O2 ignite reaches 14343 K (was hidden by 6000 K clamp) | CEA now refuses; prompt qualitative | Find where ignition energy is booked against the residual mass; physical flame temperature or typed refusal |
| 13 | Electrolysis heat ledger | mat-063 heat note | electrical work outside whole-step ledger | Own electrolysis energy (work + reaction enthalpy) in one ledger |
| 8 | Reaktoro (LGPL) | — | Deferred | Native-only optional backend only if a lane above needs it |
