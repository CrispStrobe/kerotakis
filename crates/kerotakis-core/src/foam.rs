//! Bounded gas-to-foam observable (BRD-074).
//!
//! Kinetics owns gas production. This module does not create matter: it maps a
//! temporary fraction of that gas volume to bubbles when a named recipe
//! declares a reviewed foam-stabilizer role. Drainage/coalescence uses a simple
//! exponential half-life, deliberately short of CFD or a universal detergent
//! claim.

use crate::material::{self, MaterialRole};
use crate::vessel::Vessel;

const ROOM_MOLAR_GAS_VOLUME_L: f64 = 24.465;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoamObservation {
    pub trapped_gas_liters: f64,
    pub volume_liters: f64,
    pub height_cm: f64,
    pub overflow_liters: f64,
    pub half_life_seconds: f64,
}

/// Decay existing foam over `seconds`, then trap a bounded fraction of newly
/// produced oxygen. Returns `None` for the no-soap control.
pub fn advance(vessel: &mut Vessel, seconds: f64, oxygen_moles: f64) -> Option<FoamObservation> {
    let stabilizer = vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = material::lookup(&portion.material, None)?;
            recipe.roles.into_iter().find_map(|role| match role {
                MaterialRole::FoamStabilizer {
                    trapping_efficiency,
                    gas_volume_fraction,
                    half_life_seconds,
                    saturation_amount,
                } => Some((
                    trapping_efficiency,
                    gas_volume_fraction,
                    half_life_seconds,
                    saturation_amount,
                    portion.amount,
                )),
                MaterialRole::OpaquePigment { .. }
                | MaterialRole::SurfaceFloater { .. }
                | MaterialRole::SurfaceTensionReducer { .. }
                | MaterialRole::AqueousImmiscibleLiquid { .. }
                | MaterialRole::AqueousEmulsifier { .. } => None,
            })
        })
        .max_by(|a, b| a.0.total_cmp(&b.0));
    let Some((efficiency, gas_fraction, half_life, saturation, amount)) = stabilizer else {
        vessel.foam = Default::default();
        return None;
    };

    let decay = (-std::f64::consts::LN_2 * seconds.max(0.0) / half_life).exp();
    vessel.foam.trapped_gas_liters *= decay;
    let dose = (amount / saturation).clamp(0.0, 1.0);
    vessel.foam.trapped_gas_liters +=
        oxygen_moles.max(0.0) * ROOM_MOLAR_GAS_VOLUME_L * efficiency * dose;
    vessel.foam.volume_liters = vessel.foam.trapped_gas_liters / gas_fraction;
    vessel.foam.peak_volume_liters = vessel
        .foam
        .peak_volume_liters
        .max(vessel.foam.volume_liters);

    let (capacity_l, cross_section_cm2) = geometry(&vessel.label);
    let height_cm = vessel.foam.volume_liters * 1000.0 / cross_section_cm2;
    let overflow_liters =
        (vessel.liquid_volume().0 + vessel.foam.volume_liters - capacity_l).max(0.0);
    Some(FoamObservation {
        trapped_gas_liters: vessel.foam.trapped_gas_liters,
        volume_liters: vessel.foam.volume_liters,
        height_cm,
        overflow_liters,
        half_life_seconds: half_life,
    })
}

fn geometry(label: &str) -> (f64, f64) {
    match label {
        "tube" => (0.030, 3.0),
        "cylinder" => (0.100, 8.0),
        "flask" => (0.250, 20.0),
        "crucible" => (0.050, 18.0),
        _ => (0.250, 28.0),
    }
}
