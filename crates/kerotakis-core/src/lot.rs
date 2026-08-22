//! ARCH-004: MaterialLot — provenance-aware material tracking.
//!
//! A lot records how material entered a vessel (user addition, transfer,
//! or reaction product) independently of how the solver resolves it.
//! Two lots can merge physically without losing their provenance or
//! particle-size metadata.

use serde::{Deserialize, Serialize};

use crate::species::Phase;
use crate::units::{Kelvin, Moles};
use crate::vessel::VesselId;
use crate::SpeciesId;

/// Unique identifier for a material lot within a bench session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LotId(pub u64);

/// How this lot entered the vessel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LotSource {
    /// Added by the user (the `add` command).
    User,
    /// Transferred from another vessel (decant, filter, pipette).
    Transfer { from_vessel: VesselId },
    /// Produced by a solver (reaction product, precipitate, gas evolved).
    Reaction { model: String },
}

/// A material lot with full provenance.
///
/// Lots are the "who added this, when, at what temperature" layer that
/// the conserved ledger does not carry. They merge physically (same
/// species in the same phase) but retain separate provenance chains for
/// audit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialLot {
    pub id: LotId,
    pub species: SpeciesId,
    pub amount: Moles,
    pub phase: Phase,
    /// Temperature at which this lot was created or entered the vessel.
    pub entry_temperature: Kelvin,
    /// Bench time (seconds) when this lot was created.
    pub entry_timestamp: f64,
    /// How this lot came into existence.
    pub source: LotSource,
    /// Optional particle size in micrometres (for solids).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_size_um: Option<f64>,
}

impl MaterialLot {
    /// Create a new lot from a user addition.
    pub fn from_addition(
        id: LotId,
        species: SpeciesId,
        amount: Moles,
        phase: Phase,
        temperature: Kelvin,
        timestamp: f64,
    ) -> Self {
        Self {
            id,
            species,
            amount,
            phase,
            entry_temperature: temperature,
            entry_timestamp: timestamp,
            source: LotSource::User,
            particle_size_um: None,
        }
    }

    /// Whether two lots can be merged (same species and phase).
    pub fn can_merge(&self, other: &Self) -> bool {
        self.species == other.species && self.phase == other.phase
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lots_with_same_species_and_phase_can_merge() {
        let a = MaterialLot::from_addition(
            LotId(1),
            SpeciesId::new("NaCl"),
            Moles(0.1),
            Phase::Solid,
            Kelvin(298.15),
            0.0,
        );
        let b = MaterialLot::from_addition(
            LotId(2),
            SpeciesId::new("NaCl"),
            Moles(0.2),
            Phase::Solid,
            Kelvin(310.0),
            5.0,
        );
        assert!(a.can_merge(&b));
    }

    #[test]
    fn lots_with_different_phase_cannot_merge() {
        let solid = MaterialLot::from_addition(
            LotId(1),
            SpeciesId::new("NaCl"),
            Moles(0.1),
            Phase::Solid,
            Kelvin(298.15),
            0.0,
        );
        let aqueous = MaterialLot::from_addition(
            LotId(2),
            SpeciesId::new("NaCl"),
            Moles(0.1),
            Phase::Aqueous,
            Kelvin(298.15),
            0.0,
        );
        assert!(!solid.can_merge(&aqueous));
    }

    #[test]
    fn lot_provenance_survives_serialization() {
        let lot = MaterialLot {
            id: LotId(42),
            species: SpeciesId::new("AgCl"),
            amount: Moles(0.005),
            phase: Phase::Solid,
            entry_temperature: Kelvin(298.15),
            entry_timestamp: 12.5,
            source: LotSource::Reaction {
                model: "PHREEQC".into(),
            },
            particle_size_um: Some(50.0),
        };
        let json = serde_json::to_string(&lot).unwrap();
        let loaded: MaterialLot = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, LotId(42));
        assert_eq!(loaded.source, LotSource::Reaction { model: "PHREEQC".into() });
        assert_eq!(loaded.particle_size_um, Some(50.0));
    }
}
