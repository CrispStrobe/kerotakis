//! CAP-25: the senses of the bench. The GUI's effects layer renders
//! only what the engine emits — so if the engine has no nose and no
//! notion of a flask's limits, smell and explosions cannot be drawn
//! honestly. This module gives it both, curated and bounded.
//!
//! Smell is taught technique: waft, never huff. The verb reports what
//! a careful waft would notice — headspace gases and volatile liquids
//! with curated odour rows — and flags the ones a real bench would
//! never let near a nose.

use crate::species::{Phase, SpeciesId};
use crate::vessel::Vessel;

/// A curated odour: what a waft notices, and whether a real bench
/// would forbid the waft at all. Descriptions are the classic
/// qualitative-analysis vocabulary (editorial curation, the same
/// register the appearance table uses).
pub struct Odor {
    pub species: &'static str,
    pub description: &'static str,
    /// True when the vapour itself is a hazard: the smell line comes
    /// with a warning, because on a real bench you must not learn this
    /// one nose-first.
    pub hazardous: bool,
}

pub const ODORS: &[Odor] = &[
    Odor {
        species: "NH3",
        description: "sharp, pungent ammonia",
        hazardous: true,
    },
    Odor {
        species: "Cl2",
        description: "choking, bleach-like",
        hazardous: true,
    },
    Odor {
        species: "SO2",
        description: "sharp, like a struck match",
        hazardous: true,
    },
    Odor {
        species: "NO2",
        description: "acrid, brown-fume sharpness",
        hazardous: true,
    },
    Odor {
        species: "NaOCl",
        description: "swimming-pool chlorine",
        hazardous: false,
    },
    Odor {
        species: "CH3COOH",
        description: "vinegar",
        hazardous: false,
    },
    Odor {
        species: "ethanol",
        description: "spirituous, wine-like",
        hazardous: false,
    },
    Odor {
        species: "methanol",
        description: "faintly spirituous",
        hazardous: true,
    },
    Odor {
        species: "propanone",
        description: "sharp, sweetish solvent",
        hazardous: false,
    },
    Odor {
        species: "ethyl_acetate",
        description: "sweet, pear-drop",
        hazardous: false,
    },
    Odor {
        species: "hexane",
        description: "faint petrol",
        hazardous: false,
    },
    Odor {
        species: "H2O2",
        description: "faintly sharp, ozone-like",
        hazardous: false,
    },
    Odor {
        species: "NH2Cl",
        description: "chlorinous, indoor-pool",
        hazardous: true,
    },
];

pub fn odor_of(species: &SpeciesId) -> Option<&'static Odor> {
    ODORS.iter().find(|o| o.species == species.0)
}

/// What one careful waft over the vessel notices: every gas in the
/// headspace and every liquid or dissolved species with a curated
/// odour row. Order is the vessel's own; deduplicated by species.
pub fn waft(vessel: &Vessel) -> Vec<&'static Odor> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for p in &vessel.contents {
        let volatile_enough = match p.phase {
            Phase::Gas => true,
            // A liquid at the bench breathes; a dissolved odorous
            // species (ammonia, acetic acid) reaches the nose from
            // solution too.
            Phase::Liquid | Phase::Aqueous => true,
            Phase::Solid => false,
        };
        if !volatile_enough || p.moles.0 <= 0.0 {
            continue;
        }
        if let Some(o) = odor_of(&p.species) {
            if seen.insert(o.species) {
                out.push(o);
            }
        }
    }
    out
}

/// The pressure at which sealed school glassware lets go. A single
/// conservative teaching constant (≈4 atm absolute), editorial: real
/// vessels vary widely and the point of the model is that sealed
/// vessels HAVE a limit, not to certify any particular flask.
pub const GLASS_BURST_PA: f64 = 405_300.0;
