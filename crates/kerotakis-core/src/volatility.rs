//! EXP-25 / EXP-31: a dissolved volatile reaches the headspace above it.
//!
//! # The gap this closes
//!
//! `add v1 NH3` puts *ammonia solution* in the vessel — a liquid portion,
//! because that is the registry's standard phase for household ammonia —
//! and the four gas tests read the headspace inventory. Nothing moved a
//! dissolved volatile into that inventory: only CO₂ had a gas/liquid
//! exchange, inside the aqueous tail, and only when water was present. So
//! a learner who sealed a jar, poured ammonia in and held damp litmus over
//! it was told the bench could not see (`gas_tests.rs` refused rather than
//! read a false negative), while `smell v1` on the same jar reported
//! "sharp, pungent ammonia". Two paths, one physical fact.
//!
//! # The model
//!
//! Henry's law, and nothing more. For a species with a reviewed
//! coefficient in [`crate::properties::HENRY_COEFFICIENTS`] (Sander 2015),
//! the amount in an owned headspace at equilibrium with the liquid is
//!
//! ```text
//! n_gas = n_total · (V_gas / RT) / (H · V_liq + V_gas / RT)
//! ```
//!
//! with `H` in mol/(L·atm) at the vessel's temperature, `V_liq` the
//! vessel's liquid volume and `V_gas` its headspace. The pass moves the
//! difference between that and what the headspace already holds — in
//! either direction, so a cooled jar takes its ammonia back — and refreshes
//! the pressure. It is a phase distribution, not a reaction: no bond is
//! made, `chemistry_applies` is false, and the aqueous tail still sees the
//! liquid.
//!
//! # What it deliberately does not do
//!
//! - **It does not touch species the registry carries as a gas.** O₂, N₂,
//!   H₂, Cl₂ and CO₂ arrive in the headspace when added, and their
//!   dissolution into water is PHREEQC's (CO₂ today, through the tail's
//!   own gas phase). A second opinion here would fight the first.
//! - **It books no heat.** Sander's temperature coefficient for ammonia is
//!   a desorption enthalpy of about 35 kJ/mol, and the registry represents
//!   ammonia solution as a portion of NH₃ whose mass and heat capacity are
//!   those of the ammonia alone — 0.17 g for 0.01 mol — not of the bottle
//!   of solution a learner actually pours. Priced against that, the jar
//!   would cool by hundreds of kelvin. The *amount* partitioned is robust
//!   to the representation (0.01 mol in a 500 mL jar puts 65% of the
//!   ammonia in the air as pure NH₃, 16% as a 10% w/w solution — both
//!   unmistakable to litmus); the heat is not, so it is not claimed.
//! - **It is silent over an open vessel.** An infinite headspace has an
//!   equilibrium partial pressure but no inventory to read, and the
//!   reservoir exchange is the aqueous tail's.

use crate::ops::Event;
use crate::properties::{henry_at_t, henry_lookup, HenryCoefficient};
use crate::solve::{Equilibrator, SolveError, SolverRouteKind};
use crate::species::{self, Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Gas constant in L·atm/(mol·K): CODATA 2018 R = 8.314 462 618 J/(mol·K)
/// over 101.325 J/(L·atm).
pub const R_LITRE_ATM: f64 = 0.082_057_366;

/// Below this the ledger is not moved at all; between this and
/// `OBSERVABLE_MOLES` it is moved without an event.
const TRACE: f64 = 1e-12;

/// One species' equilibrium split between liquid and headspace.
#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    pub species: SpeciesId,
    /// Phase the condensed share is held in (the existing portion's phase,
    /// or `Aqueous` beside water and `Liquid` otherwise).
    pub condensed_phase: Phase,
    pub total: f64,
    pub gas_now: f64,
    pub gas_at_equilibrium: f64,
    /// Henry's constant at the vessel temperature, mol/(L·atm).
    pub henry_mol_per_l_atm: f64,
    /// Equilibrium partial pressure, Pa.
    pub partial_pressure_pa: f64,
    pub source: &'static str,
}

impl Partition {
    /// Positive: moles that must leave the liquid; negative: moles that
    /// must return to it.
    pub fn shortfall(&self) -> f64 {
        self.gas_at_equilibrium - self.gas_now
    }
}

/// The reviewed coefficient for a species this pass may partition: one
/// the registry does NOT carry as a gas, matched on key or on formula
/// with the `(aq)` suffix removed.
pub fn coefficient_for(key: &str) -> Option<&'static HenryCoefficient> {
    let data = species::lookup_key(key)?;
    if data.standard_phase == Phase::Gas {
        return None;
    }
    henry_lookup(key).or_else(|| henry_lookup(data.formula.trim_end_matches("(aq)")))
}

/// Every partition the vessel's owned headspace asks for, in content order.
pub fn partitions(vessel: &Vessel) -> Vec<Partition> {
    if !vessel.owns_headspace_gas() {
        return Vec::new();
    }
    let Some(v_gas) = vessel.headspace_volume().map(|v| v.0).filter(|v| *v > 0.0) else {
        return Vec::new();
    };
    let t = vessel.temperature.0;
    if t <= 0.0 {
        return Vec::new();
    }
    let gas_capacity = v_gas / (R_LITRE_ATM * t);
    let v_liq = vessel.liquid_volume().0;
    let water_present = vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > 0.0);

    let mut out: Vec<Partition> = Vec::new();
    for p in &vessel.contents {
        if p.phase == Phase::Solid || out.iter().any(|q| q.species == p.species) {
            continue;
        }
        let Some(coeff) = coefficient_for(&p.species.0) else {
            continue;
        };
        let mut condensed = 0.0;
        let mut gas = 0.0;
        let mut condensed_phase = None;
        for q in vessel.contents.iter().filter(|q| q.species == p.species) {
            match q.phase {
                Phase::Gas => gas += q.moles.0,
                Phase::Liquid | Phase::Aqueous => {
                    condensed += q.moles.0;
                    condensed_phase.get_or_insert(q.phase);
                }
                Phase::Solid => {}
            }
        }
        let total = condensed + gas;
        if total <= 0.0 {
            continue;
        }
        let h = henry_at_t(coeff, t).value;
        let gas_at_equilibrium = total * gas_capacity / (h * v_liq + gas_capacity);
        out.push(Partition {
            species: p.species.clone(),
            condensed_phase: condensed_phase.unwrap_or(if water_present {
                Phase::Aqueous
            } else {
                Phase::Liquid
            }),
            total,
            gas_now: gas,
            gas_at_equilibrium,
            henry_mol_per_l_atm: h,
            partial_pressure_pa: gas_at_equilibrium / gas_capacity * 101_325.0,
            source: coeff.provenance,
        });
    }
    out
}

/// Move each partition to its equilibrium and say so. Returns the events;
/// the vessel's pressure is refreshed when anything moved.
pub fn settle(vessel: &mut Vessel) -> Vec<Event> {
    let mut events = Vec::new();
    let mut moved = false;
    for part in partitions(vessel) {
        let delta = part.shortfall();
        if delta.abs() <= TRACE {
            continue;
        }
        if delta > 0.0 {
            // Liquid first, then aqueous: whichever the species is held as.
            let mut left = delta;
            for phase in [Phase::Liquid, Phase::Aqueous] {
                if left <= 0.0 {
                    break;
                }
                left -= vessel.withdraw_phase(&part.species, Moles(left), phase).0;
            }
            vessel.deposit(part.species.clone(), Moles(delta - left), Phase::Gas);
        } else {
            let back = vessel
                .withdraw_phase(&part.species, Moles(-delta), Phase::Gas)
                .0;
            vessel.deposit(part.species.clone(), Moles(back), part.condensed_phase);
        }
        moved = true;
        if delta.abs() >= crate::OBSERVABLE_MOLES {
            events.push(Event::HeadspacePartitioned {
                vessel: vessel.id,
                species: part.species.clone(),
                to_gas: delta > 0.0,
                moles: Moles(delta.abs()),
                gas_fraction: part.gas_at_equilibrium / part.total,
                partial_pressure_pa: part.partial_pressure_pa,
                henry_mol_per_l_atm: part.henry_mol_per_l_atm,
                source: part.source.to_string(),
            });
        }
    }
    if moved {
        vessel.refresh_pressure();
    }
    events
}

/// The solver-stack face of [`settle`].
#[derive(Debug, Default, Clone, Copy)]
pub struct HeadspacePartitionEquilibrator;

impl Equilibrator for HeadspacePartitionEquilibrator {
    fn name(&self) -> &'static str {
        "headspace-partition"
    }

    fn route_kind(&self) -> SolverRouteKind {
        // A reviewed coefficient applied by a closed-form law: the same
        // rung as the phase routes, not a computed equilibrium.
        SolverRouteKind::Curated
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        partitions(vessel)
            .iter()
            .any(|p| p.shortfall().abs() > TRACE)
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        // A phase distribution makes no bond. Claiming chemistry here would
        // route a beaker away from the aqueous solver that should see it.
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(settle(vessel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::{Kelvin, Liters};
    use crate::vessel::{Headspace, VesselId};

    fn sealed_jar(headspace_l: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "jar");
        v.headspace = Headspace::Sealed {
            volume: Liters(headspace_l),
        };
        v
    }

    #[test]
    fn only_species_the_registry_does_not_carry_as_gas_qualify() {
        assert!(
            coefficient_for("NH3").is_some(),
            "ammonia solution partitions"
        );
        for gas in ["CO2", "O2", "N2", "H2"] {
            assert!(
                coefficient_for(gas).is_none(),
                "{gas} arrives as a gas and is PHREEQC's to dissolve"
            );
        }
        assert!(coefficient_for("water").is_none());
        assert!(coefficient_for("NaCl").is_none());
    }

    #[test]
    fn ammonia_in_a_sealed_jar_reaches_the_headspace() {
        let mut v = sealed_jar(0.5);
        v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Liquid);
        let parts = partitions(&v);
        assert_eq!(parts.len(), 1);
        let p = &parts[0];
        // 0.5 L / (0.082057 × 298.15) = 0.02044 mol/atm of headspace;
        // 0.17 g of solution at 0.91 g/mL is 0.187 mL, × 57 mol/(L·atm)
        // = 0.01066 mol/atm of liquid. 0.02044 / 0.0311 = 65.7%.
        let frac = p.gas_at_equilibrium / p.total;
        assert!((frac - 0.657).abs() < 0.01, "gas share {frac}");
        let events = settle(&mut v);
        assert_eq!(events.len(), 1);
        let gas: f64 = v
            .contents
            .iter()
            .filter(|q| q.phase == Phase::Gas && q.species.0 == "NH3")
            .map(|q| q.moles.0)
            .sum();
        assert!((gas - p.gas_at_equilibrium).abs() < 1e-12);
        let total: f64 = v
            .contents
            .iter()
            .filter(|q| q.species.0 == "NH3")
            .map(|q| q.moles.0)
            .sum();
        assert!((total - 0.01).abs() < 1e-12, "nothing created or lost");
        // Second pass: already at equilibrium, nothing to do.
        assert!(settle(&mut v).is_empty());
        assert!(!HeadspacePartitionEquilibrator.applies(&v));
    }

    #[test]
    fn water_holds_ammonia_back() {
        let mut v = sealed_jar(0.5);
        v.deposit(SpeciesId::new("water"), Moles(27.75), Phase::Liquid); // 0.5 L
        v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Aqueous);
        let p = &partitions(&v)[0];
        // 57 × 0.5 = 28.5 mol/atm of water against 0.0204 of air.
        let frac = p.gas_at_equilibrium / p.total;
        assert!(frac < 1e-3, "dilute ammonia mostly stays dissolved: {frac}");
        assert!(frac > 1e-4, "but not entirely: {frac}");
        assert_eq!(p.condensed_phase, Phase::Aqueous);
    }

    #[test]
    fn a_cooled_jar_takes_its_ammonia_back() {
        let mut v = sealed_jar(0.5);
        v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Liquid);
        settle(&mut v);
        let warm_gas = partitions(&v)[0].gas_now;
        v.temperature = Kelvin(278.15);
        let events = settle(&mut v);
        assert!(matches!(
            events.as_slice(),
            [Event::HeadspacePartitioned { to_gas: false, .. }]
        ));
        let cold_gas = partitions(&v)[0].gas_now;
        assert!(cold_gas < warm_gas, "{cold_gas} < {warm_gas}");
        let liquid: f64 = v
            .contents
            .iter()
            .filter(|q| q.phase == Phase::Liquid && q.species.0 == "NH3")
            .map(|q| q.moles.0)
            .sum();
        assert!(liquid > 0.0, "it went back into the liquid it came from");
    }

    #[test]
    fn an_open_vessel_is_left_alone() {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Liquid);
        assert!(partitions(&v).is_empty());
        assert!(settle(&mut v).is_empty());
    }
}
