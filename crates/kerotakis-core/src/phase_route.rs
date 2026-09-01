//! EXP-33: the two phase routes that are not melting.
//!
//! `states.rs` is the solvent's story — water freezing and boiling, with the
//! thresholds moved by whatever is dissolved in it. This module is the other
//! two ways a solid leaves the bottom of a crucible, and neither of them is
//! water's:
//!
//! * **Sublimation.** Ammonium chloride does not melt on a hot plate; it goes
//!   straight to vapour at 338 °C and comes back as a white crust on anything
//!   cool. That is a *separation*: heat a mixture of ammonium chloride and
//!   common salt, and one of them leaves.
//! * **Hydrate bookkeeping.** Blue copper sulfate is not copper sulfate; it is
//!   copper sulfate plus five waters, and the crucible proves it — heat it,
//!   weigh it, and the missing mass is exactly the water. Put a drop back and
//!   the blue returns.
//!
//! Both are curated thresholds rather than computed equilibria, and both say
//! so. What is *not* curated is the arithmetic: the water driven off a
//! hydrate is counted in moles and reappears as mass on the balance, so the
//! classic mass-before / mass-after lesson closes to the digit rather than to
//! a rounding.
//!
//! ## What this module does not model
//!
//! * **Intermediate hydrates.** Copper sulfate pentahydrate really loses its
//!   waters stepwise (TGA: two near 63 °C, two near 109 °C, the last near
//!   200 °C) through a trihydrate and a monohydrate. Neither intermediate is
//!   in the registry, so this bench does the transition in ONE step at the
//!   final-water temperature and says so. A partially dehydrated hydrate is a
//!   real substance and this bench does not have it.
//! * **Dissociative sublimation.** Ammonium chloride vapour is really ammonia
//!   and hydrogen chloride, which recombine on the cold surface. The bench
//!   moves the intact formula unit, which is what the recovered crust weighs
//!   and what the demonstration shows, but the vapour is not NH₄Cl molecules.
//! * **Rates.** Both routes complete within the step that crosses the
//!   threshold. A real sublimation takes time and a real crucible takes
//!   minutes at temperature; no kinetics is claimed.
//! * **Water activity.** Whether an anhydrous salt takes water back as a
//!   hydrate or simply dissolves is, in truth, a question about water
//!   activity. This bench uses the stoichiometric proxy in
//!   `REHYDRATION_WATER_HEADROOM` below and states it rather than pretending
//!   to a phase diagram it does not have.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Amounts below this are not chemistry, they are float dust.
const TRACE: f64 = 1e-12;

/// How much more water than the crystal formula asks for may be present
/// before the bench stops calling the result a hydrate.
///
/// A stated model choice with a real justification: the school demonstration
/// is a *drop* of water on a spatula of white powder, and the blue that
/// appears is the hydrate, not a solution. Once there is enough water to
/// dissolve the salt, dissolution is the honest answer and the aqueous
/// engine owns it — chalcanthite and epsomite are both phases in the shipped
/// USGS database, so crystallising them back out of solution is a computed
/// solve, not this module's business. The proxy is stoichiometric because
/// the real criterion is water activity and this bench does not compute it.
pub const REHYDRATION_WATER_HEADROOM: f64 = 1.0;

/// A hydrate the bench can take apart and put back together.
///
/// The stoichiometry is not stored: it is read off the registry formula, so
/// a hydrate whose formula says `·5H2O` cannot disagree with a table saying
/// four. Only the *temperature* is curated, because only the temperature is
/// a measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HydratePair {
    /// Registry key of the hydrate, e.g. `chalcanthite`.
    pub hydrate: &'static str,
    /// Registry key of the anhydrous salt, e.g. `CuSO4`.
    pub anhydrous: &'static str,
    /// Waters of crystallisation per formula unit.
    pub waters: f64,
    /// Where this bench drives them all off, K.
    pub dehydration_k: f64,
}

/// Split a hydrate formula into its anhydrous part and its water count.
///
/// `MgSO4·7H2O` → `("MgSO4", 7.0)`. Returns `None` for anything without a
/// hydrate dot, which is most of the registry.
pub fn split_hydrate(formula: &str) -> Option<(&str, f64)> {
    let (anhydrous, waters) = formula
        .split_once('·')
        .or_else(|| formula.split_once('*'))?;
    let waters = waters.trim();
    let rest = waters.strip_suffix("H2O")?;
    let n: f64 = if rest.is_empty() {
        1.0
    } else {
        rest.parse().ok()?
    };
    (n > 0.0).then_some((anhydrous.trim(), n))
}

/// Every hydrate/anhydrous pair the registry can actually take apart: the
/// hydrate carries a dehydration temperature AND its anhydrous partner is a
/// shipped species. A hydrate with no partner is not an error — it is a
/// hydrate this bench will not claim to dehydrate, and the melting-point
/// apparatus still reports its dehydration temperature as data.
pub fn hydrate_pairs() -> Vec<HydratePair> {
    let mut pairs = Vec::new();
    for species in crate::species::registry() {
        let Some(t) = species.transitions else {
            continue;
        };
        let Some(dehydration_k) = t.dehydration_k else {
            continue;
        };
        let Some((anhydrous_formula, waters)) = split_hydrate(species.formula) else {
            continue;
        };
        let Some(partner) = crate::species::registry()
            .iter()
            .find(|candidate| candidate.formula == anhydrous_formula)
        else {
            continue;
        };
        pairs.push(HydratePair {
            hydrate: species.key,
            anhydrous: partner.key,
            waters,
            dehydration_k,
        });
    }
    pairs
}

/// The solids that leave as vapour without melting first.
pub fn sublimes_at(species: &SpeciesId) -> Option<f64> {
    let data = crate::species::lookup(species)?;
    let t = data.transitions?;
    // A substance with a melting point melts; sublimation at 1 atm is for
    // the ones whose vapour pressure reaches an atmosphere while they are
    // still solid, and the registry records that by having no melting point
    // and a sublimation point instead.
    t.melting_k.is_none().then_some(t.sublimation_k)?
}

fn moles_in_phase(vessel: &Vessel, species: &SpeciesId, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == species && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn withdraw_phase(vessel: &mut Vessel, species: &SpeciesId, phase: Phase, moles: f64) {
    let mut remaining = moles;
    for p in vessel.contents.iter_mut() {
        if &p.species == species && p.phase == phase && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    vessel.contents.retain(|p| p.moles.0 > 1e-15);
}

/// Release `moles` of a gas: into the headspace if the vessel owns one,
/// otherwise across the boundary. Either way the balance notices.
fn release_gas(vessel: &mut Vessel, species: SpeciesId, moles: Moles, events: &mut Vec<Event>) {
    let id = vessel.id;
    if vessel.retain_gas(species.clone(), moles) {
        events.push(Event::GasContained {
            vessel: id,
            species,
            moles,
        });
    } else {
        events.push(Event::GasEvolved {
            vessel: id,
            species,
            moles,
        });
    }
}

/// Sublimation and hydrate bookkeeping, applied wherever the temperature
/// says they apply.
pub struct PhaseRouteEquilibrator;

impl PhaseRouteEquilibrator {
    fn sublimation(&self, vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
        let now = vessel.temperature.0;
        let mut moved = false;
        // Collect first: the loop mutates `contents`.
        let candidates: Vec<(SpeciesId, f64, Phase)> = vessel
            .contents
            .iter()
            .filter(|p| matches!(p.phase, Phase::Solid | Phase::Gas) && p.moles.0 > TRACE)
            .filter_map(|p| sublimes_at(&p.species).map(|k| (p.species.clone(), k, p.phase)))
            .collect();
        for (species, threshold, phase) in candidates {
            match phase {
                Phase::Solid if now >= threshold => {
                    let n = moles_in_phase(vessel, &species, Phase::Solid);
                    if n <= TRACE {
                        continue;
                    }
                    withdraw_phase(vessel, &species, Phase::Solid, n);
                    events.push(Event::StateChanged {
                        vessel: vessel.id,
                        species: species.clone(),
                        from: Phase::Solid,
                        to: Phase::Gas,
                        at: crate::units::Kelvin(threshold),
                        shifted_by: 0.0,
                    });
                    release_gas(vessel, species, Moles(n), events);
                    moved = true;
                }
                // Deposition: the cold-finger half of the separation. The
                // vapour only comes back where the vessel kept it.
                Phase::Gas if now < threshold => {
                    let n = moles_in_phase(vessel, &species, Phase::Gas);
                    if n <= TRACE {
                        continue;
                    }
                    withdraw_phase(vessel, &species, Phase::Gas, n);
                    vessel.deposit(species.clone(), Moles(n), Phase::Solid);
                    vessel.refresh_pressure();
                    events.push(Event::StateChanged {
                        vessel: vessel.id,
                        species,
                        from: Phase::Gas,
                        to: Phase::Solid,
                        at: crate::units::Kelvin(threshold),
                        shifted_by: 0.0,
                    });
                    moved = true;
                }
                _ => {}
            }
        }
        moved
    }

    fn hydrates(&self, vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
        let now = vessel.temperature.0;
        let water = SpeciesId::new("water");
        let mut moved = false;
        for pair in hydrate_pairs() {
            let hydrate = SpeciesId::new(pair.hydrate);
            let anhydrous = SpeciesId::new(pair.anhydrous);
            if now >= pair.dehydration_k {
                let n = moles_in_phase(vessel, &hydrate, Phase::Solid);
                if n <= TRACE {
                    continue;
                }
                withdraw_phase(vessel, &hydrate, Phase::Solid, n);
                vessel.deposit(anhydrous.clone(), Moles(n), Phase::Solid);
                events.push(Event::Dehydrated {
                    vessel: vessel.id,
                    hydrate: hydrate.clone(),
                    anhydrous: anhydrous.clone(),
                    formula_units: Moles(n),
                    water: Moles(n * pair.waters),
                    at: crate::units::Kelvin(pair.dehydration_k),
                });
                release_gas(vessel, water.clone(), Moles(n * pair.waters), events);
                moved = true;
                continue;
            }
            // Rehydration. Below the threshold, an anhydrous salt in contact
            // with a little water takes it back into the crystal — but only
            // a little: past the headroom, dissolving is what really happens
            // and the aqueous engine owns that.
            let salt = moles_in_phase(vessel, &anhydrous, Phase::Solid);
            if salt <= TRACE {
                continue;
            }
            let free_water = moles_in_phase(vessel, &water, Phase::Liquid);
            if free_water <= TRACE {
                continue;
            }
            let wanted = salt * pair.waters;
            if free_water > wanted * (1.0 + REHYDRATION_WATER_HEADROOM) {
                continue;
            }
            let formula_units = (free_water / pair.waters).min(salt);
            if formula_units <= TRACE {
                continue;
            }
            withdraw_phase(vessel, &anhydrous, Phase::Solid, formula_units);
            withdraw_phase(vessel, &water, Phase::Liquid, formula_units * pair.waters);
            vessel.deposit(hydrate.clone(), Moles(formula_units), Phase::Solid);
            events.push(Event::Hydrated {
                vessel: vessel.id,
                anhydrous,
                hydrate,
                formula_units: Moles(formula_units),
                water: Moles(formula_units * pair.waters),
            });
            moved = true;
        }
        moved
    }
}

impl Equilibrator for PhaseRouteEquilibrator {
    fn name(&self) -> &'static str {
        "phase-routes"
    }

    fn route_kind(&self) -> crate::solve::SolverRouteKind {
        crate::solve::SolverRouteKind::Curated
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        // Neither route is chemistry: no bond is made or broken by
        // sublimation, and a hydrate's water is held by the lattice. They
        // are phase changes, and claiming otherwise would route a beaker
        // away from the aqueous solver that should still see it.
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        // Two passes at most: dehydration can free water that a second
        // salt would take up, and deposition can never trigger sublimation
        // at the same temperature, so the sequence cannot cycle.
        for _ in 0..2 {
            let a = self.sublimation(vessel, &mut events);
            let b = self.hydrates(vessel, &mut events);
            if !a && !b {
                break;
            }
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrate_formulas_split_into_salt_and_water() {
        assert_eq!(split_hydrate("MgSO4·7H2O"), Some(("MgSO4", 7.0)));
        assert_eq!(split_hydrate("CaSO4·2H2O"), Some(("CaSO4", 2.0)));
        assert_eq!(split_hydrate("NaCl"), None);
        // A lone water without a count is one water, not zero.
        assert_eq!(split_hydrate("XY·H2O"), Some(("XY", 1.0)));
    }

    #[test]
    fn every_pair_conserves_mass_exactly() {
        // The whole hydrate lesson is a mass ledger, so the molar masses
        // have to be additive to the digit. If a hydrate's molar mass is
        // not its salt plus its water, the crucible cannot balance and no
        // amount of careful arithmetic downstream will fix it.
        for pair in hydrate_pairs() {
            let h = crate::species::lookup(&SpeciesId::new(pair.hydrate)).unwrap();
            let a = crate::species::lookup(&SpeciesId::new(pair.anhydrous)).unwrap();
            let w = crate::species::lookup(&SpeciesId::new("water")).unwrap();
            let sum = a.molar_mass + pair.waters * w.molar_mass;
            assert!(
                (h.molar_mass - sum).abs() < 1e-9,
                "{}: {} != {} + {}×{}",
                pair.hydrate,
                h.molar_mass,
                a.molar_mass,
                pair.waters,
                w.molar_mass
            );
        }
    }

    #[test]
    fn a_substance_that_melts_does_not_also_sublime() {
        // The registry records sublimation only where melting is not what
        // happens; iodine at one atmosphere melts, whatever the demo says.
        for species in crate::species::registry() {
            if let Some(t) = species.transitions {
                if t.sublimation_k.is_some() && t.melting_k.is_some() {
                    assert!(
                        sublimes_at(&SpeciesId::new(species.key)).is_none(),
                        "{} claims both a melting and a sublimation route",
                        species.key
                    );
                }
            }
        }
    }
}
