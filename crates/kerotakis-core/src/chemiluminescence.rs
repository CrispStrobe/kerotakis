//! Temperature-dependent luminol teaching observable.
//!
//! This is intentionally not a commercial glow-stick model. A declared
//! alkaline, catalyst-containing luminol solution begins emitting when its
//! hydrogen-peroxide activator is added. A bounded Arrhenius response exposes
//! the useful trade-off: warmer is brighter now and fades faster.

use crate::material::MaterialBasis;
use crate::vessel::Vessel;

pub const RECIPE_ID: &str = "school/luminol-glow-solution";

#[derive(Debug, Clone, PartialEq)]
pub struct ChemiluminescenceObservation {
    pub elapsed_s: f64,
    pub relative_intensity: f64,
    pub half_life_s: f64,
    pub temperature_k: f64,
    pub oxidant_moles: f64,
}

pub fn observe(vessel: &Vessel) -> Option<ChemiluminescenceObservation> {
    let solution_g: f64 = vessel
        .unresolved_materials
        .iter()
        .filter(|p| p.recipe_id == RECIPE_ID && p.basis == MaterialBasis::MassFraction)
        .map(|p| p.amount)
        .sum();
    if solution_g <= 1e-9 {
        return None;
    }
    let oxidant_moles: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "H2O2")
        .map(|p| p.moles.0)
        .sum();
    if oxidant_moles <= 1e-9 {
        return None;
    }
    let activated_at = vessel
        .lots
        .iter()
        .filter(|lot| lot.species.0 == "H2O2")
        .map(|lot| lot.added_at)
        .fold(f64::INFINITY, f64::min);
    let elapsed_s = (vessel.elapsed_seconds - activated_at).max(0.0);
    const REFERENCE_K: f64 = 293.15;
    const REFERENCE_HALF_LIFE_S: f64 = 60.0;
    const ACTIVATION_J_PER_MOL: f64 = 35_000.0;
    const GAS_CONSTANT: f64 = 8.314_462_618;
    let temperature_k = vessel.temperature.0.clamp(273.15, 333.15);
    let rate_ratio =
        (ACTIVATION_J_PER_MOL / GAS_CONSTANT * (1.0 / REFERENCE_K - 1.0 / temperature_k)).exp();
    let half_life_s = REFERENCE_HALF_LIFE_S / rate_ratio;
    let remaining = 2.0_f64.powf(-elapsed_s / half_life_s);
    let dose = (oxidant_moles / 0.001).clamp(0.0, 1.0);
    Some(ChemiluminescenceObservation {
        elapsed_s,
        relative_intensity: (rate_ratio * remaining * dose).min(4.0),
        half_life_s,
        temperature_k,
        oxidant_moles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::{MaterialLot, UnresolvedMaterialPortion, VesselId};
    use crate::{material::MaterialBasis, Phase, SpeciesId};

    fn activated_at(temperature_k: f64) -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.temperature = crate::Kelvin(temperature_k);
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "luminol glow solution".into(),
            recipe_id: RECIPE_ID.into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 20.0,
            enzyme_hydrolysis: None,
        });
        vessel.deposit(SpeciesId::new("H2O2"), crate::Moles(0.002), Phase::Liquid);
        vessel.lots.push(MaterialLot {
            species: SpeciesId::new("H2O2"),
            moles: crate::Moles(0.002),
            phase: Phase::Liquid,
            added_at: 0.0,
            hydrated_at: None,
            source: None,
            particle_size_um: None,
            suspended_fraction: None,
        });
        vessel
    }

    #[test]
    fn warmer_is_brighter_and_shorter_lived() {
        let cold = observe(&activated_at(283.15)).unwrap();
        let warm = observe(&activated_at(313.15)).unwrap();
        assert!(warm.relative_intensity > cold.relative_intensity);
        assert!(warm.half_life_s < cold.half_life_s);
    }

    #[test]
    fn light_fades_with_elapsed_time() {
        let mut vessel = activated_at(293.15);
        let initial = observe(&vessel).unwrap().relative_intensity;
        vessel.elapsed_seconds = 60.0;
        assert!((observe(&vessel).unwrap().relative_intensity - initial / 2.0).abs() < 1e-9);
    }
}
