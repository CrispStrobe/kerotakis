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

/// The latent heat one phase route has to pay for, in kJ/mol, positive
/// meaning "the vessel supplies it".
///
/// This is a curated table for the same reason `combustion::FUELS` is one:
/// the value is a measurement with a source, not something the registry
/// schema has a slot for. `PhaseTransitions` carries five temperatures and
/// no energies, and widening it would touch the build script, the runtime
/// loader, the export crate and their three fidelity tests for a claim
/// that two rows need.
///
/// **The table is deliberately not total, and that is load-bearing.** A
/// substance with no row here sublimes exactly as it did before this
/// tranche: all of it, in the step that crosses the threshold, at no
/// energy cost. Ammonium chloride is such a substance, and its behaviour
/// is unchanged — the crucible separation is a *separation*, and nobody
/// weighed the heat it took. Adding a row is therefore a deliberate act
/// that changes what a vessel does.
#[derive(Debug, Clone, Copy)]
pub struct LatentHeat {
    /// Registry species key of the CONDENSED phase.
    pub species: &'static str,
    /// kJ per mole of the formula unit.
    pub kj_per_mol: f64,
    pub provenance: &'static str,
}

/// Enthalpies of sublimation, keyed by the solid that leaves.
pub const SUBLIMATION_ENTHALPIES: &[LatentHeat] = &[LatentHeat {
    species: "dry_ice",
    // 25.2 kJ/mol at the 1 atm sublimation point.
    kj_per_mol: 25.2,
    provenance: "Enthalpy of sublimation of carbon dioxide at its 194.7 K normal sublimation point, 25.2 kJ/mol, as commonly tabulated from NIST/CODATA-class evaluated data. PENDING REVIEW: no positively identified page was opened for this row, so no edition-level provenance is claimed and the value stands as the standard tabulated one. Sanity check a reviewer can run without a book: it is the sum of the tabulated 8.65 kJ/mol enthalpy of fusion at the triple point and about 16.7 kJ/mol of vaporisation there, and it is the number that makes 5 g of dry ice cool 100 g of water by about 6.8 K, which is what a kitchen thermometer reads",
}];

/// The enthalpy of sublimation of a solid, J/mol, or `None` where the
/// bench does not claim one.
pub fn sublimation_enthalpy(species: &str) -> Option<f64> {
    SUBLIMATION_ENTHALPIES
        .iter()
        .find(|row| row.species == species)
        .map(|row| row.kj_per_mol * 1000.0)
}

/// The gas a subliming solid becomes.
///
/// Ammonium chloride's vapour is ammonium chloride: nothing else on the
/// shelf has its formula, so the route moves the same key between phases
/// and the crust that comes back is the salt that left. Dry ice's vapour
/// is *carbon dioxide*, which this registry carries as its own gas
/// species — and calling it "dry ice gas" in a vessel would be a
/// contradiction in terms.
///
/// The pair is derived from the formula rather than tabulated, exactly as
/// `hydrate_pairs` derives its stoichiometry from one: a species that
/// claims to be the solid form of a shipped gas cannot disagree with the
/// gas about what it is made of.
pub fn sublimation_product(solid_key: &str) -> &'static str {
    let Some(solid) = crate::species::lookup(&SpeciesId::new(solid_key)) else {
        return "";
    };
    crate::species::registry()
        .iter()
        .find(|candidate| {
            candidate.standard_phase == Phase::Gas
                && candidate.formula == solid.formula
                && candidate.key != solid.key
        })
        .map_or(solid.key, |gas| gas.key)
}

/// The solid a gas deposits as, with the temperature it happens at:
/// the inverse of [`sublimation_product`], resolved over the registry.
pub fn deposition_partner(gas_key: &str) -> Option<(&'static str, f64)> {
    crate::species::registry().iter().find_map(|candidate| {
        let k = sublimes_at(&SpeciesId::new(candidate.key))?;
        (candidate.standard_phase != Phase::Gas && sublimation_product(candidate.key) == gas_key)
            .then_some((candidate.key, k))
    })
}

/// A condensed species that is a phase of a substance this registry also
/// ships as a gas: `dry_ice` for `CO2`.
///
/// Such a key exists so that a bench can HOLD the condensed phase — you
/// cannot put carbon dioxide gas in a beaker and call it dry ice. It is
/// emphatically not a mineral, and anything that pairs registry solids
/// with database phases by composition must skip it, or a carbonate
/// solution acquires a "mineral" with dry ice's formula and precipitates
/// it at 25 °C. `kerotakis_phreeqc::derived` is the caller that matters.
pub fn is_condensed_gas(key: &str) -> bool {
    let Some(data) = crate::species::lookup(&SpeciesId::new(key)) else {
        return false;
    };
    data.standard_phase != Phase::Gas && sublimation_product(key) != key
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

/// The molar heat capacity the registry gives a species, J/(mol·K).
fn molar_cp(key: &str) -> f64 {
    crate::species::lookup(&SpeciesId::new(key)).map_or(0.0, |d| d.heat_capacity)
}

/// What a latent-heat transition costs, and what the vessel can spend.
///
/// `budget` is the heat available to the transition, in joules, measured
/// from the transition temperature; `None` means no enthalpy is claimed
/// for this substance and the route stays athermal, exactly as it was
/// before this tranche.
struct Ledger {
    moles: f64,
    budget: Option<f64>,
    latent: f64,
    forward: bool,
}

/// ## The superheat a cryogen never had
///
/// `add` gives every portion the room temperature nobody asked it about
/// (`bench.rs` defaults `at` to `Kelvin::STANDARD`), and for a condensed
/// gas that is a state which does not exist: a block of dry ice is at
/// 194.7 K, not at 25 °C. Letting that fiction pay for the cooling would
/// be free energy — 5 g of "dry ice at 25 °C" carries 553 J of sensible
/// heat that no real block has.
///
/// So the forward route treats the condensed phase as having arrived AT
/// its transition temperature: the rest of the vessel supplies the latent
/// heat, and the sample's own superheat is discarded rather than spent.
/// The consequence is stated rather than hidden — **this bench cannot
/// show you a cryogen warming up**, only a cryogen at its transition
/// temperature and whatever it takes from the beaker around it. A lone
/// sample with nothing to draw on therefore sits at its own sublimation
/// point and does not leave, which is what an insulated flask of dry ice
/// really does and is why the temperature is settled even when no matter
/// moves.
///
/// Deposition needs no such correction: a vapour really is at the vessel
/// temperature, and the heat it gives back warms everything present.
fn ledger(
    vessel: &Vessel,
    condensed: &str,
    inventory: f64,
    now: f64,
    threshold: f64,
    forward: bool,
) -> Ledger {
    let Some(latent) = sublimation_enthalpy(condensed) else {
        return Ledger {
            moles: inventory,
            budget: None,
            latent: 0.0,
            forward,
        };
    };
    let budget = if forward {
        let own = inventory * molar_cp(condensed);
        (vessel.heat_capacity() - own).max(0.0) * (now - threshold)
    } else {
        vessel.heat_capacity() * (threshold - now)
    }
    .max(0.0);
    Ledger {
        moles: (budget / latent).min(inventory),
        budget: Some(budget),
        latent,
        forward,
    }
}

/// Spend the ledger and put the vessel at the temperature that leaves.
///
/// One expression covers all three cases the physics has. When the budget
/// is exactly consumed the vessel lands on the transition temperature and
/// stays there — the plateau, with condensed phase still in the flask.
/// When there is heat to spare the remainder warms (or, on deposition,
/// cools) whatever is left, over the heat capacity the vessel has AFTER
/// the move, so matter that left the ledger takes its own sensible heat
/// with it. And when nothing could be afforded at all, the same
/// expression puts the vessel on the transition temperature, which is the
/// correction described on [`ledger`].
fn settle(vessel: &mut Vessel, l: &Ledger, moved: f64, threshold: f64, events: &mut Vec<Event>) {
    let Some(budget) = l.budget else {
        return;
    };
    let cp = vessel.heat_capacity();
    if cp <= 0.0 {
        return;
    }
    let left = budget - moved * l.latent;
    let to = if l.forward {
        threshold + left / cp
    } else {
        threshold - left / cp
    };
    let to = crate::units::Kelvin(to.max(0.0));
    let from = vessel.temperature;
    if (to.0 - from.0).abs() <= 1e-9 {
        return;
    }
    vessel.temperature = to;
    vessel.refresh_pressure();
    events.push(Event::TemperatureChanged {
        vessel: vessel.id,
        from,
        to,
    });
}

/// Sublimation and hydrate bookkeeping, applied wherever the temperature
/// says they apply.
pub struct PhaseRouteEquilibrator;

impl PhaseRouteEquilibrator {
    fn sublimation(&self, vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
        let now = vessel.temperature.0;
        let mut moved = false;
        // Collect first: the loop mutates `contents`.
        //
        // A solid is a candidate if it has a sublimation point of its own;
        // a gas is a candidate if it has one (ammonium chloride vapour) or
        // if it is the vapour of a solid that does (carbon dioxide over
        // dry ice).
        let candidates: Vec<(SpeciesId, f64, Phase)> = vessel
            .contents
            .iter()
            .filter(|p| matches!(p.phase, Phase::Solid | Phase::Gas) && p.moles.0 > TRACE)
            .filter_map(|p| {
                let k = sublimes_at(&p.species).or_else(|| match p.phase {
                    Phase::Gas => deposition_partner(&p.species.0).map(|(_, k)| k),
                    _ => None,
                })?;
                Some((p.species.clone(), k, p.phase))
            })
            .collect();
        for (species, threshold, phase) in candidates {
            match phase {
                Phase::Solid if now >= threshold => {
                    let inventory = moles_in_phase(vessel, &species, Phase::Solid);
                    if inventory <= TRACE {
                        continue;
                    }
                    let l = ledger(vessel, &species.0, inventory, now, threshold, true);
                    if l.moles > TRACE {
                        let vapour = SpeciesId::new(sublimation_product(&species.0));
                        withdraw_phase(vessel, &species, Phase::Solid, l.moles);
                        events.push(Event::StateChanged {
                            vessel: vessel.id,
                            species: species.clone(),
                            from: Phase::Solid,
                            to: Phase::Gas,
                            at: crate::units::Kelvin(threshold),
                            shifted_by: 0.0,
                        });
                        release_gas(vessel, vapour, Moles(l.moles), events);
                        moved = true;
                    }
                    settle(vessel, &l, l.moles, threshold, events);
                }
                // Deposition: the cold-finger half of the separation. The
                // vapour only comes back where the vessel kept it.
                Phase::Gas if now < threshold => {
                    let inventory = moles_in_phase(vessel, &species, Phase::Gas);
                    if inventory <= TRACE {
                        continue;
                    }
                    let solid = SpeciesId::new(
                        deposition_partner(&species.0).map_or(species.0.as_str(), |(key, _)| key),
                    );
                    let l = ledger(vessel, &solid.0, inventory, now, threshold, false);
                    if l.moles > TRACE {
                        withdraw_phase(vessel, &species, Phase::Gas, l.moles);
                        vessel.deposit(solid.clone(), Moles(l.moles), Phase::Solid);
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
                    settle(vessel, &l, l.moles, threshold, events);
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
