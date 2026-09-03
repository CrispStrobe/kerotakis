//! KID-13: a suspension thick enough to argue back — the oobleck observable.
//!
//! Cornstarch in water is the one experiment on the children's list that is
//! not chemistry at all. Nothing reacts; what changes is how the mixture
//! *responds* to being pushed. Above a packing fraction the particles cannot
//! slide past one another fast enough, so the harder you stir the more it
//! resists — and the moment you stop, it flows again.
//!
//! So this reports a property of the mixture under a stated shear, not a
//! reaction and not a state change: nothing here moves a mole. It is the
//! same discipline the surface-spread and foam observables follow, and it is
//! deliberately a long way short of a rheological model — there is no
//! viscosity here, no yield stress and no critical shear rate, only the
//! bounded statement that this mixture is one of the ones that does this and
//! that this stir was fast enough to notice.

use crate::species::{self, Phase};
use crate::vessel::Vessel;

/// One curated shear-thickening suspension.
pub struct ShearThickening {
    /// The suspended solid.
    pub solid: &'static str,
    /// Mass fraction of solid, out of solid plus water, at which the effect
    /// starts to be noticeable by hand.
    pub onset_mass_fraction: f64,
    /// Where it is unmistakable — the classic "you can punch it" mixture.
    pub full_mass_fraction: f64,
    /// Stir-bar tip speed above which the mixture is being sheared hard
    /// enough to thicken rather than simply mixed, m/s.
    pub shear_threshold_m_s: f64,
    pub provenance: &'static str,
}

pub const SHEAR_THICKENING: &[ShearThickening] = &[ShearThickening {
    solid: "starch",
    // Cornstarch oobleck is made at roughly one-and-a-half to two parts
    // starch to one part water by mass, so the effect appears somewhere
    // above half and is unmistakable around two thirds.
    onset_mass_fraction: 0.50,
    full_mass_fraction: 0.65,
    // A 25 mm bar at about 300 rpm. Below this the mixture is being stirred;
    // above it, it is being sheared.
    shear_threshold_m_s: 0.39,
    provenance: "Dense cornstarch suspensions shear-thicken: above a packing fraction the particles cannot rearrange fast enough for the imposed shear rate, and the mixture resists. Editorial judgement (Kerotakis): the onset and saturation fractions bracket the familiar kitchen recipe rather than a measured jamming transition, and the shear threshold is one bar speed rather than a critical shear rate. No viscosity, yield stress, normal stress or time dependence is claimed, and nothing here is a chemical change — the ledger does not move",
}];

/// What the mixture does at this shear.
#[derive(Debug, Clone, PartialEq)]
pub struct ThickeningObservation {
    pub solid: &'static str,
    /// 0 at onset, 1 at the full mixture.
    pub strength: f64,
    pub solid_mass_fraction: f64,
    /// The tip speed that was applied.
    pub tip_speed_m_s: f64,
    /// True when the stir was fast enough to thicken rather than mix.
    pub sheared_hard: bool,
}

/// Look at the vessel as a suspension. `tip_speed_m_s` is the shear this
/// step applied; zero means the mixture is at rest.
pub fn observe(vessel: &Vessel, tip_speed_m_s: f64) -> Option<ThickeningObservation> {
    SHEAR_THICKENING.iter().find_map(|pair| {
        let grams = |key: &str, phase: Phase| -> f64 {
            let data = species::lookup_key(key);
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == key && p.phase == phase)
                .map(|p| p.moles.0 * data.map_or(0.0, |d| d.molar_mass))
                .sum()
        };
        let solid_g = grams(pair.solid, Phase::Solid);
        let water_g = grams("water", Phase::Liquid);
        if solid_g <= 1e-9 || water_g <= 1e-9 {
            return None;
        }
        let fraction = solid_g / (solid_g + water_g);
        let strength = ((fraction - pair.onset_mass_fraction)
            / (pair.full_mass_fraction - pair.onset_mass_fraction))
            .clamp(0.0, 1.0);
        (strength > 1e-9).then_some(ThickeningObservation {
            solid: pair.solid,
            strength,
            solid_mass_fraction: fraction,
            tip_speed_m_s,
            sheared_hard: tip_speed_m_s >= pair.shear_threshold_m_s,
        })
    })
}
