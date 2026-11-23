//! Amphoteric metals in alkali: aluminium and zinc dissolving in sodium
//! hydroxide as their hydroxo complexes, with hydrogen.
//!
//! ```text
//! 2 Al + 2 OH⁻ + 6 H₂O → 2 Al(OH)₄⁻ + 3 H₂
//!   Zn + 2 OH⁻ + 2 H₂O →   Zn(OH)₄²⁻ +  H₂
//! ```
//!
//! The oxidant is water, not the hydroxide: both metals sit below the water
//! couple `2 H₂O + 2 e⁻ → H₂ + 2 OH⁻`. What the alkali does is take the
//! metal's ion away as a soluble hydroxo complex instead of letting a
//! hydroxide skin passivate the surface — which is why aluminium foil sits
//! in tap water for years and fizzes in caustic soda in a minute.
//!
//! The driving force is computed rather than asserted. The potential of the
//! hydroxo couple `M(OH)₄^(4−n)− + n e⁻ → M + 4 OH⁻` is DERIVED from two
//! independent sources: the metal-ion couple's E° (CRC, as in
//! [`crate::displacement::SERIES`]) and the complex's formation constant in
//! the vendored wateq4f.dat (`M⁺ⁿ + 4 H₂O = M(OH)₄ + 4 H⁺`, re-based onto
//! OH⁻ with pKw = 14). That is not the round trip that
//! `displacement::SERIES` rejects: the log_k used is a hydrolysis constant,
//! not a redox one. The derived values reproduce CRC's own tabulated
//! hydroxo potentials (−1.199 V for zincate, −2.33 V for aluminate) to
//! within a hundredth of a volt, and a test holds them there.
//!
//! The same hydrogen-overpotential rule the acid route uses decides whether
//! hydrogen can actually form on the metal on the timescale of a lesson.
//! Aluminium clears it by more than a volt. Zinc in cold dilute alkali does
//! not, which is what a bench shows — zinc granules in cold sodium
//! hydroxide barely bubble, and the textbook reaction is done hot or with
//! the zinc in contact with copper — and the lab says so as a computed,
//! kinetic result rather than leaving the zinc silently untouched.
//!
//! The extent is limited by the metal, the water, and the hydroxide the
//! complex binds (one per aluminium, two per zinc): an aluminium excess in a
//! little caustic soda stops when the free base is spent, as it does.

use crate::displacement::{self, Couple};
use crate::i18n::Locale;
use crate::ops::Event;
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Kelvin, Moles};
use crate::vessel::{ThermalMode, Vessel};

/// Aluminium's ion couple, `Al³⁺ + 3 e⁻ → Al`. Not in
/// [`displacement::SERIES`]: aluminium's oxide film makes its displacement
/// of other metals from sulfate solution, and its reaction with dilute acid,
/// a matter of induction times and chloride attack that the Nernst
/// bookkeeping there would overstate. Here it only supplies E° and ΔfH° of
/// the ion; the oxide film is exactly what the alkali removes.
pub const ALUMINIUM: Couple = Couple {
    oxidised: "Al+3",
    oxidised_per_reduced: 1.0,
    reduced: "Al",
    reduced_phase: Phase::Solid,
    electrons: 3.0,
    e0_volts: -1.662,
    dfh_oxidised_kj: -531.0,
    source: SOURCE,
};

const SOURCE: &str = "E°(Al³⁺/Al) = −1.662 V and E°(Zn²⁺/Zn) = −0.7618 V: CRC Handbook of Chemistry and Physics, 'Electrochemical Series' (Vanýsek), 25 °C vs SHE. ΔfH°(aq): NBS Tables of Chemical Thermodynamic Properties (Wagman et al., 1982) — Al³⁺ −531.0, Zn²⁺ −153.89, OH⁻ −229.994, H₂O(l) −285.830 kJ/mol. Hydroxo-complex formation: vendored wateq4f.dat, Al⁺³ + 4 H₂O = Al(OH)₄⁻ + 4 H⁺ (log_k −22.7, ΔH 42.3 kcal) and Zn⁺² + 4 H₂O = Zn(OH)₄⁻² + 4 H⁺ (log_k −41.2, no ΔH given)";

/// The water couple, `2 H₂O + 2 e⁻ → H₂ + 2 OH⁻`, V vs SHE at 25 °C (CRC).
pub const WATER_REDUCTION_E0_VOLTS: f64 = -0.8277;

const PKW: f64 = 14.0;
const DFH_WATER_LIQUID_KJ: f64 = -285.830;
const DFH_HYDROXIDE_AQ_KJ: f64 = -229.994;
const KCAL: f64 = 4.184;

/// One metal that dissolves in alkali as a hydroxo complex.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Amphoteric {
    pub couple: Couple,
    /// The complex as PHREEQC names it; the inventory books the metal ion
    /// and the hydroxide, and the aqueous solve speciates them into it.
    pub complex: &'static str,
    /// log_k of `M⁺ⁿ + 4 H₂O = M(OH)₄ + 4 H⁺` in wateq4f.dat.
    pub hydrolysis_log_k: f64,
    /// ΔH of that same reaction, kJ/mol, where wateq4f gives one.
    pub hydrolysis_dh_kj: Option<f64>,
}

impl Amphoteric {
    /// Hydroxides the complex binds per metal: 4 − n.
    pub fn hydroxide_per_metal(&self) -> f64 {
        4.0 - self.couple.electrons
    }

    /// E° of `M(OH)₄^(4−n)− + n e⁻ → M + 4 OH⁻`, V, derived from the ion
    /// couple and the complex's overall formation constant on OH⁻.
    pub fn hydroxo_e0_volts(&self, slope: f64) -> f64 {
        let log_beta4 = self.hydrolysis_log_k + 4.0 * PKW;
        self.couple.e0_volts - slope / self.couple.electrons * log_beta4
    }

    /// Reaction enthalpy per mole of metal, kJ/mol, for
    /// `M + (4−n) OH⁻ + n H₂O → M(OH)₄ + n/2 H₂`, when every term is known:
    /// the metal-with-water Hess sum plus the complex's formation from the
    /// ion and hydroxide (wateq4f's hydrolysis ΔH plus four neutralisations).
    pub fn enthalpy_kj_per_mol(&self) -> Option<f64> {
        let c = &self.couple;
        let with_water = c.dfh_oxidised_kj + c.electrons * DFH_HYDROXIDE_AQ_KJ
            - c.electrons * DFH_WATER_LIQUID_KJ;
        let neutralisation = DFH_WATER_LIQUID_KJ - DFH_HYDROXIDE_AQ_KJ;
        self.hydrolysis_dh_kj
            .map(|dh| with_water + dh + 4.0 * neutralisation)
    }

    /// The balanced equation in whole numbers.
    pub fn equation(&self) -> String {
        let n = self.couple.electrons as u64;
        let k = if n % 2 == 1 { 2 } else { 1 };
        let oh = (4 - n) * k;
        let water = n * k;
        let h2 = n * k / 2;
        let c = |x: u64| {
            if x == 1 {
                String::new()
            } else {
                format!("{x} ")
            }
        };
        format!(
            "{}{} + {}OH- + {}H2O → {}{} + {}H2",
            c(k),
            self.couple.reduced,
            c(oh),
            c(water),
            c(k),
            self.complex,
            c(h2),
        )
    }
}

/// The amphoteric metals this lab dissolves in alkali.
pub fn metals() -> [Amphoteric; 2] {
    let zinc = *displacement::couple_of_metal("Zn").expect("zinc is in the activity series");
    [
        Amphoteric {
            couple: ALUMINIUM,
            complex: "Al(OH)4-",
            hydrolysis_log_k: -22.7,
            hydrolysis_dh_kj: Some(42.3 * KCAL),
        },
        Amphoteric {
            couple: zinc,
            complex: "Zn(OH)4-2",
            hydrolysis_log_k: -41.2,
            hydrolysis_dh_kj: None,
        },
    ]
}

/// The amphoteric entry for a metal key.
pub fn of_metal(key: &str) -> Option<Amphoteric> {
    metals().into_iter().find(|a| a.couple.reduced == key)
}

fn moles_in(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

/// Free strong base in the vessel, mol: the excess of cation charge the
/// analytical inventory carries (see [`displacement::solute_charge`]).
pub fn free_base(vessel: &Vessel) -> f64 {
    vessel.solute_charge.max(0.0)
}

/// Whether this vessel is alkaline enough for the hydroxo route to own the
/// metal: there is free base beyond what the lab can see, and a solved pH
/// above neutral.
pub fn alkaline(vessel: &Vessel) -> bool {
    free_base(vessel) > crate::OBSERVABLE_MOLES
        && vessel.solution.as_ref().is_some_and(|s| s.ph > 7.0)
}

/// Whether this module owns the metal's fate in this vessel — the honesty
/// passes elsewhere must not also speak for it.
pub fn owns(vessel: &Vessel, metal: &str) -> bool {
    of_metal(metal).is_some() && alkaline(vessel)
}

/// Dissolve every amphoteric metal the alkali can take.
pub fn dissolve(vessel: &mut Vessel) -> Vec<Event> {
    let mut events = Vec::new();
    if !alkaline(vessel) {
        return events;
    }
    let slope = crate::relations::nernst_slope(vessel.temperature);
    for a in metals() {
        let c = a.couple;
        let metal = moles_in(vessel, c.reduced, Phase::Solid);
        if metal <= crate::OBSERVABLE_MOLES {
            continue;
        }
        let ph = vessel.solution.as_ref().map(|s| s.ph).unwrap_or(7.0);
        let log_a_oh = ph - PKW;
        // Fresh metal, no complex yet: the complex at unit activity, as
        // the acid route treats a metal's own ion.
        let e_water = WATER_REDUCTION_E0_VOLTS - slope * log_a_oh;
        let e_metal = a.hydroxo_e0_volts(slope) - 4.0 * slope / c.electrons * log_a_oh;
        let driving = e_water - e_metal;
        let eta = displacement::hydrogen_overpotential(c.reduced);
        let name = species::lookup_key(c.reduced)
            .map(|d| d.name)
            .unwrap_or(c.reduced);
        if driving <= eta {
            let reason = Phrase::new(
                "inert.amphoteric-overpotential",
                "{name} should dissolve in this alkali as its hydroxo complex (driving force {driving} V against water), but hydrogen has to form on {name}, and on that surface it costs an overpotential of about {eta} V. Kinetically blocked on the timescale of a lesson — warm, concentrated alkali, or contact with a less reactive metal, is how it is done on a bench",
                vec![
                    ("name".to_string(), Slot::term("species", name)),
                    ("driving".to_string(), Slot::number(format!("{driving:+.2}"))),
                    ("eta".to_string(), Slot::number(format!("{eta:.2}"))),
                ],
            );
            events.push(Event::Inert {
                vessel: vessel.id,
                species: SpeciesId::new(c.reduced),
                why: reason.render(Locale::EN),
                computed: true,
                spent: None,
                reason: Some(reason),
            });
            continue;
        }
        let water = moles_in(vessel, "water", Phase::Liquid);
        let reacted = metal
            .min(water / c.electrons)
            .min(free_base(vessel) / a.hydroxide_per_metal());
        if reacted <= crate::OBSERVABLE_MOLES {
            continue;
        }
        let t_ref = Kelvin::STANDARD.0;
        let held_before = vessel.energy_between(t_ref, vessel.temperature.0);
        let contents_before = vessel.contents.clone();
        let heat_joules = a.enthalpy_kj_per_mol().map(|dh| -dh * 1000.0 * reacted);
        let hydrogen = reacted * c.electrons / 2.0;

        // Booked as the ion and n hydroxides from water; the solve that
        // follows binds the remaining (4 − n) hydroxides from the free base
        // into the complex, which is where the base is spent.
        vessel.withdraw_phase(&SpeciesId::new(c.reduced), Moles(reacted), Phase::Solid);
        vessel.withdraw_phase(
            &SpeciesId::new("water"),
            Moles(reacted * c.electrons),
            Phase::Liquid,
        );
        vessel.deposit(SpeciesId::new(c.oxidised), Moles(reacted), Phase::Aqueous);
        vessel.deposit(
            SpeciesId::new("OH-"),
            Moles(reacted * c.electrons),
            Phase::Aqueous,
        );
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: a.equation(),
        });
        events.push(Event::Consumed {
            vessel: vessel.id,
            species: SpeciesId::new(c.reduced),
            moles: Moles(reacted),
            remaining: Some(Moles(moles_in(vessel, c.reduced, Phase::Solid))),
        });
        let h2 = SpeciesId::new("H2");
        let mut evolved = 0.0;
        if vessel.retain_gas(h2.clone(), Moles(hydrogen)) {
            events.push(Event::GasContained {
                vessel: vessel.id,
                species: h2,
                moles: Moles(hydrogen),
            });
        } else {
            evolved = hydrogen;
            events.push(Event::GasEvolved {
                vessel: vessel.id,
                species: h2,
                moles: Moles(hydrogen),
            });
        }
        vessel.solute_charge = displacement::solute_charge(vessel);
        match heat_joules {
            Some(q) => {
                if matches!(vessel.thermal_mode, ThermalMode::Adiabatic)
                    && vessel.heat_capacity() > 0.0
                {
                    let t_new = vessel.temperature_after_from(t_ref, held_before + q);
                    vessel.temperature = Kelvin(t_new.max(0.0));
                }
                if let Some(start) = vessel.step_start.as_mut() {
                    displacement::book_paid_change(start, &contents_before, &vessel.contents);
                    if evolved > 0.0 {
                        start.note_heat_paid(&SpeciesId::new("H2"), Phase::Gas, evolved);
                    }
                }
            }
            None => {
                // No sourced ΔH for the complex: charge nothing rather than
                // half a Hess sum, and leave the step's heat to the tail's
                // own (unpriced) verdict.
                let reason = Phrase::new(
                    "not-modeled.amphoteric-heat",
                    "the heat of {name} dissolving in alkali: the formation enthalpy of {complex} is not in this lab's sourced data, so no heat is charged for it and the temperature is left where the step began",
                    vec![
                        ("name".to_string(), Slot::term("species", name)),
                        ("complex".to_string(), Slot::text(a.complex)),
                    ],
                );
                events.push(Event::not_modeled(
                    vessel.id,
                    crate::ops::NotModelledCause::NoReviewedDatum,
                    reason,
                ));
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    const SLOPE_25C: f64 = 0.059_159;

    #[test]
    fn derived_hydroxo_potentials_match_the_tabulated_ones() {
        // CRC 'Electrochemical Series': Zn(OH)₄²⁻ + 2e⁻ → Zn + 4OH⁻ −1.199 V;
        // H₂AlO₃⁻ + H₂O + 3e⁻ → Al + 4OH⁻ −2.33 V.
        let [al, zn] = metals();
        assert!((zn.hydroxo_e0_volts(SLOPE_25C) - -1.199).abs() < 0.01);
        assert!((al.hydroxo_e0_volts(SLOPE_25C) - -2.33).abs() < 0.02);
    }

    #[test]
    fn both_metals_sit_below_the_water_couple_in_alkali() {
        for a in metals() {
            assert!(a.hydroxo_e0_volts(SLOPE_25C) < WATER_REDUCTION_E0_VOLTS);
        }
    }

    #[test]
    fn aluminate_enthalpy_is_exothermic_and_zincate_is_unpriced() {
        let [al, zn] = metals();
        let dh = al.enthalpy_kj_per_mol().unwrap();
        // Hess over Wagman: −1502.5 (Al(OH)₄⁻) − [−229.994 + 3(−285.83)] ≈ −415 kJ/mol;
        // the wateq4f route must land near it.
        assert!((dh - -415.0).abs() < 15.0, "{dh}");
        assert!(zn.enthalpy_kj_per_mol().is_none());
    }

    #[test]
    fn equations_balance() {
        let [al, zn] = metals();
        assert_eq!(al.equation(), "2 Al + 2 OH- + 6 H2O → 2 Al(OH)4- + 3 H2");
        assert_eq!(zn.equation(), "Zn + 2 OH- + 2 H2O → Zn(OH)4-2 + H2");
    }

    #[test]
    fn the_aluminium_ion_is_a_registry_species() {
        assert!(species::lookup_key("Al+3").is_some());
        assert!(species::lookup_key("Al").is_some());
    }
}
