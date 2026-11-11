//! Melting RANGES for the materials that have no melting point, and the
//! one honesty rule every solid on this bench obeys when heated past where
//! it melts (reviewed-transition-points lane, 2026-11-11).
//!
//! # Why a range and not a point
//!
//! Paraffin wax is a blend of long-chain alkanes and a polyolefin is a
//! semicrystalline polymer with a distribution of lamella thicknesses.
//! Neither has a sharp melting point: each melts over a range, and the
//! range is a property of the grade. A single curated temperature would be
//! a sharper claim than the material supports, so these rows carry the
//! sourced lower and upper bound and nothing else. The melting-point
//! apparatus reports the range (`PurityVerdict::MeltingRange`), never a
//! point.
//!
//! # The rule ("never silently keep it solid")
//!
//! A solid stands above the upper bound of its sourced melting point or
//! range. If [`crate::phase_route::fusion_enthalpy`] carries a latent heat
//! for it, the phase route has already melted it and paid for it, and this
//! module says nothing. Otherwise the bench has no latent heat to pay with,
//! so it cannot model the melt — and it says so with a typed
//! `not-modeled.melting-not-modelled` refusal (EN/DE/FR) rather than
//! showing a white solid at 190 °C as though that were a prediction.
//!
//! No latent heat is shipped for these materials: none was found with a
//! primary source this lane could read, and an invented one would move the
//! thermometer by an amount nobody measured.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase, SpeciesId};
use crate::vessel::Vessel;

/// A sourced melting range at 101.325 kPa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeltingRange {
    pub species: &'static str,
    /// Lower bound, K, where one is sourced. `None` for the polyolefins:
    /// the source gives the equilibrium melting temperature of a perfect
    /// crystal, which bounds a real grade's melting from ABOVE, and nothing
    /// for where a commercial grade starts to melt.
    pub lower_k: Option<f64>,
    /// Upper bound, K: no crystal of this material survives above it.
    pub upper_k: f64,
    /// Primary citation with DOI, printed by the apparatus.
    pub source: &'static str,
    /// What the range does not claim.
    pub boundary: &'static str,
}

/// Paraffin wax and the two polyolefins of po-009.
///
/// * Paraffin wax: A. Sharma, V. V. Tyagi, C. R. Chen and D. Buddhi,
///   'Review on thermal energy storage with phase change materials and
///   applications', Renew. Sustain. Energy Rev. 13 (2009) 318-345,
///   doi:10.1016/j.rser.2007.10.005, Table 2a: technical-grade paraffins
///   melting between 42-44 and 66-68 degrees Celsius (315.15-341.15 K
///   across the grades), latent heat 189 kJ/kg for five grades.
/// * Polyethylene: U. Gaur and B. Wunderlich, J. Phys. Chem. Ref. Data 10
///   (1981) 119, doi:10.1063/1.555636: equilibrium melting temperature
///   414.6 +/- 0.5 K.
/// * Isotactic polypropylene: U. Gaur and B. Wunderlich, J. Phys. Chem.
///   Ref. Data 10 (1981) 1051, doi:10.1063/1.555650: best-estimate
///   equilibrium melting temperature 460.7 K.
pub const MELTING_RANGES: &[MeltingRange] = &[
    MeltingRange {
        species: "paraffin",
        lower_k: Some(315.15),
        upper_k: 341.15,
        source: "A. Sharma, V. V. Tyagi, C. R. Chen and D. Buddhi, 'Review on thermal energy storage with phase change materials and applications', Renew. Sustain. Energy Rev. 13 (2009) 318-345, doi:10.1016/j.rser.2007.10.005, Table 2a: technical-grade paraffin waxes melt between 42-44 and 66-68 degrees Celsius depending on grade; latent heat of fusion 189 kJ/kg for five grades",
        boundary: "commercial paraffin wax is a blend of long-chain alkanes and melts over a range set by its grade; this is the span of the technical grades in the source, not the range of any one candle. A latent heat is reported by the source but the bench has no single plateau temperature to pay it at, so melting is not modelled",
    },
    MeltingRange {
        species: "PE",
        lower_k: None,
        upper_k: 414.6,
        source: "U. Gaur and B. Wunderlich, 'Heat capacity and other thermodynamic properties of linear macromolecules. II. Polyethylene', J. Phys. Chem. Ref. Data 10 (1981) 119-152, doi:10.1063/1.555636: equilibrium melting temperature 414.6 +/- 0.5 K",
        boundary: "the equilibrium melting temperature of a perfect polyethylene crystal: a real semicrystalline grade melts over a range BELOW it, set by its crystallinity and lamella thickness, and that lower end is not sourced here",
    },
    MeltingRange {
        species: "PP",
        lower_k: None,
        upper_k: 460.7,
        source: "U. Gaur and B. Wunderlich, 'Heat capacity and other thermodynamic properties of linear macromolecules. IV. Polypropylene', J. Phys. Chem. Ref. Data 10 (1981) 1051-1064, doi:10.1063/1.555650: best-estimate equilibrium melting temperature of isotactic polypropylene 460.7 K",
        boundary: "the equilibrium melting temperature of a perfect isotactic polypropylene crystal: a real grade melts over a range BELOW it, set by tacticity and crystallinity, and that lower end is not sourced here",
    },
];

/// The sourced melting range of a material, where it has one.
pub fn range(species: &str) -> Option<&'static MeltingRange> {
    MELTING_RANGES.iter().find(|row| row.species == species)
}

/// The temperature above which a solid of this species must no longer be
/// shown as solid: its sourced melting point, or the top of its sourced
/// melting range.
pub fn melted_above_k(species: &str) -> Option<f64> {
    species::lookup(&SpeciesId::new(species))
        .and_then(|data| data.transitions)
        .and_then(|t| t.melting_k)
        .or_else(|| range(species).map(|row| row.upper_k))
}

/// One typed refusal per solid species standing above where it melts
/// without a latent heat to melt it with.
pub fn unmodelled_melts(vessel: &Vessel) -> Vec<Event> {
    let now = vessel.temperature.0;
    let mut seen: Vec<&str> = Vec::new();
    let mut events = Vec::new();
    for portion in &vessel.contents {
        if portion.phase != Phase::Solid || portion.moles.0 <= 1e-12 {
            continue;
        }
        let key = portion.species.0.as_str();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        // A latent heat on the bench means the phase route melts it; water's
        // ice belongs to the solvent model (`solve::StateEquilibrator`).
        if key == "water" || crate::phase_route::fusion_enthalpy(key).is_some() {
            continue;
        }
        let Some(melts) = melted_above_k(key) else {
            continue;
        };
        if now <= melts {
            continue;
        }
        events.push(Event::not_modeled(
            vessel.id,
            NotModelledCause::NoReviewedDatum,
            Phrase::new(
                "not-modeled.melting-not-modelled",
                "{name} heated above its melting point ({melting} °C): melting is not modelled here, so the solid shown is not a prediction that it stays solid",
                vec![
                    (
                        "name".to_string(),
                        Slot::term("species", species::english_name(key)),
                    ),
                    (
                        "melting".to_string(),
                        Slot::number(format!("{:.0}", melts - 273.15)),
                    ),
                ],
            ),
        ));
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_range_is_ordered_and_cited() {
        for row in MELTING_RANGES {
            assert!(
                row.lower_k.is_none_or(|low| low < row.upper_k),
                "{}",
                row.species
            );
            assert!(row.source.contains("doi"), "{} has no DOI", row.species);
            assert!(
                species::lookup(&SpeciesId::new(row.species)).is_some(),
                "{} is not in the registry",
                row.species
            );
        }
    }
}
