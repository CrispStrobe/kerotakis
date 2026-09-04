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
    /// KID-10b: the concentration in solution, mol/L, at or above which a
    /// careful waft finds it. Gas in the headspace is not gated by this —
    /// a gas that is there has already reached the nose.
    ///
    /// These are curated teaching thresholds and they differ by more than
    /// two orders of magnitude between rows, which is itself a fact worth
    /// carrying: ammonia is smelled far below the concentration at which
    /// vinegar is. They are not measured odour-detection thresholds, and
    /// no claim is made about any individual nose.
    pub detect_molar: f64,
}

pub const ODORS: &[Odor] = &[
    Odor {
        species: "NH3",
        description: "sharp, pungent ammonia",
        hazardous: true,
        // household ammonia is about 1 mol/L and unmistakable; the trace of free base above a pH 5 ammonium chloride solution is not.
        detect_molar: 1e-5,
    },
    Odor {
        species: "Cl2",
        description: "choking, bleach-like",
        hazardous: true,
        // chlorine is pungent far below its harmful concentration, which is the only reason a person survives meeting it.
        detect_molar: 1e-4,
    },
    Odor {
        species: "SO2",
        description: "sharp, like a struck match",
        hazardous: true,
        // sharp at very low concentration.
        detect_molar: 1e-4,
    },
    Odor {
        species: "NO2",
        description: "acrid, brown-fume sharpness",
        hazardous: true,
        // acrid at very low concentration.
        detect_molar: 1e-4,
    },
    Odor {
        species: "NaOCl",
        description: "swimming-pool chlorine",
        hazardous: false,
        // a bleach bottle is around 0.7 mol/L; a rinsed sink is not.
        detect_molar: 1e-3,
    },
    Odor {
        species: "CH3COOH",
        description: "vinegar",
        hazardous: false,
        // household vinegar carries 0.88 mol/L of undissociated acid and reeks; a pH 8.75 sodium acetate solution carries 7.65e-6 and does not. Four orders of headroom on each side of this floor.
        detect_molar: 1e-4,
    },
    Odor {
        species: "ethanol",
        description: "spirituous, wine-like",
        hazardous: false,
        // a glass of wine is about 2 mol/L; a millimolar solution smells of nothing.
        detect_molar: 1e-2,
    },
    Odor {
        species: "methanol",
        description: "faintly spirituous",
        hazardous: true,
        // faint even neat, so the floor is no lower than ethanol's.
        detect_molar: 1e-2,
    },
    Odor {
        species: "propanone",
        description: "sharp, sweetish solvent",
        hazardous: false,
        // nail-varnish remover is nearly neat and carries across a room.
        detect_molar: 1e-3,
    },
    Odor {
        species: "ethyl_acetate",
        description: "sweet, pear-drop",
        hazardous: false,
        // esters are the reason this floor is not uniform: fruit smells of them at concentrations where a solvent would not register.
        detect_molar: 1e-4,
    },
    Odor {
        species: "hexane",
        description: "faint petrol",
        hazardous: false,
        // petrol-like and volatile, but not detectable at trace.
        detect_molar: 1e-3,
    },
    Odor {
        species: "H2O2",
        description: "faintly sharp, ozone-like",
        hazardous: false,
        // 3% peroxide is about 1 mol/L and barely smells at all, so the floor is the highest in this table.
        detect_molar: 1e-1,
    },
    Odor {
        species: "NH2Cl",
        description: "chlorinous, indoor-pool",
        hazardous: true,
        // the sharp note of an over-chlorinated pool, noticed low.
        detect_molar: 1e-4,
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
    let litres = vessel.liquid_volume().0;
    for p in &vessel.contents {
        if p.moles.0 <= 0.0 {
            continue;
        }
        let Some(odor) = odor_of(&p.species) else {
            continue;
        };
        // KID-10b: a gas in the headspace has already reached the nose.
        // Anything dissolved has to be there in enough quantity, and the
        // floor belongs to the substance rather than to this function —
        // 1e-5 mol/L of ammonia is unmistakable and 1e-5 of acetic acid is
        // nothing at all.
        let detected = match p.phase {
            Phase::Gas => true,
            Phase::Liquid | Phase::Aqueous => {
                litres > 0.0 && p.moles.0 / litres >= odor.detect_molar
            }
            Phase::Solid => false,
        };
        if detected && seen.insert(odor.species) {
            out.push(odor);
        }
    }
    out
}

/// The pressure at which sealed school glassware lets go. A single
/// conservative teaching constant (≈4 atm absolute), editorial: real
/// vessels vary widely and the point of the model is that sealed
/// vessels HAVE a limit, not to certify any particular flask.
pub const GLASS_BURST_PA: f64 = 405_300.0;
