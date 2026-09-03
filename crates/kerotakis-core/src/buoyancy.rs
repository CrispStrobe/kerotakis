//! KID-13: the dancing raisin — an object that rides the gas.
//!
//! K11 was the last *unreachable* row in the children's first thirty: no
//! raisin on the shelf, and no model for a bubble that attaches to an
//! object and lifts it. It is one of the two experiments on that list
//! (with oobleck) where nothing reacts: the raisin is the same raisin
//! afterwards, the water is the same water, and what changes is only
//! where the raisin is.
//!
//! ## What this claims
//!
//! One number, and it is the number the experiment is about. A raisin
//! sinks because it is denser than water — about 1.35 g/mL against 1.00.
//! To lift it, the bubbles clinging to it must add enough VOLUME to bring
//! the average density of raisin-plus-bubbles below the liquid's:
//!
//! ```text
//!   m / (V + V_gas) < ρ_liquid   ⟹   V_gas / V > (ρ_object − ρ_liquid) / ρ_liquid
//! ```
//!
//! For a raisin in plain sparkling water that is 0.35 — the bubbles have
//! to be worth about a third of the raisin's own size before it goes up,
//! which is why you can watch them gather for a while before anything
//! happens. In a dense sugar syrup the same raisin floats with no bubbles
//! at all, and the fraction is the honest way to say why: it goes
//! negative, and there is nothing to lift.
//!
//! ## What this does not claim
//!
//! No period, no bubble count, no bubble size, no nucleation-site
//! density, no rise velocity, no number of trips, and no clock — this
//! bench degasses a glass in one step, so "how long the dancing lasts" is
//! not a question it can answer. Nothing here is a two-phase flow model.
//! The claim is: this object is denser than this liquid, gas is coming
//! out of the liquid, and THIS much attached gas would lift it.

use crate::vessel::Vessel;

/// A material the bench will let ride its own bubbles.
///
/// Membership is deliberately curated rather than derived from density
/// alone. A stone is also denser than water and gas does not lift it: the
/// effect needs a surface bubbles will actually stick to, and surface
/// texture is not something any recipe here describes. So the table
/// names the objects the demonstration is done with, and says so.
#[derive(Debug, Clone, Copy)]
pub struct BubbleRider {
    /// Material recipe canonical key.
    pub material: &'static str,
    pub provenance: &'static str,
}

pub const RIDERS: &[BubbleRider] = &[BubbleRider {
    material: "raisin",
    provenance: "A raisin's wrinkled skin is a field of nucleation sites, which is why this experiment is done with raisins and not with beads. Editorial judgement (Kerotakis): membership in this table is a statement that bubbles stick to the thing, and nothing in the registry describes surface texture — so the table is short and curated rather than inferred from density",
}];

/// What the bench can say about an object sitting in a fizzing liquid.
#[derive(Debug, Clone, PartialEq)]
pub struct BubbleRide {
    /// Display name of the material.
    pub material: String,
    pub object_density_g_per_ml: f64,
    pub liquid_density_g_per_ml: f64,
    /// Attached gas volume, as a fraction of the object's own volume,
    /// needed to lift it. Zero means it floats unaided.
    pub lift_gas_fraction: f64,
}

/// The density of the liquid the object is sitting in, g/mL: everything
/// dissolved in it counts, which is why sugar syrup lifts a raisin that
/// water does not.
///
/// `Vessel::liquid_volume` deliberately excludes solute volume — the
/// solution's volume is carried by its solvent, and a dissolved solid is
/// treated as taking none. That is a fine approximation for a pH and a
/// disastrous one here: 200 g of sugar in 100 mL of water came out at
/// **2.33 g/mL**, denser than any syrup that has ever existed, because
/// all of the sugar's mass and none of its volume was counted. So the
/// solute volume is added back from each species' own density, which is
/// the same additive-volume assumption the rest of the bench uses and
/// gives 1.33 g/mL for that syrup — the number a hydrometer would read.
pub fn liquid_density_g_per_ml(vessel: &Vessel) -> Option<f64> {
    let mut volume_ml = vessel.liquid_volume().0 * 1000.0;
    if volume_ml <= 0.0 {
        return None;
    }
    let mut mass_g = 0.0;
    for portion in &vessel.contents {
        let dissolved = portion.phase == crate::species::Phase::Aqueous;
        if !dissolved && portion.phase != crate::species::Phase::Liquid {
            continue;
        }
        let Some(data) = crate::species::lookup(&portion.species) else {
            continue;
        };
        let grams = portion.moles.0 * data.molar_mass;
        mass_g += grams;
        if dissolved && data.density > 0.0 {
            volume_ml += grams / data.density;
        }
    }
    (volume_ml > 0.0).then(|| mass_g / volume_ml)
}

/// The bubble-riding reading for this vessel, if it holds a rider in a
/// liquid.
pub fn observe(vessel: &Vessel) -> Option<BubbleRide> {
    let liquid_density = liquid_density_g_per_ml(vessel)?;
    if !liquid_density.is_finite() || liquid_density <= 0.0 {
        return None;
    }
    for portion in &vessel.unresolved_materials {
        if !RIDERS
            .iter()
            .any(|rider| portion.recipe_id.ends_with(rider.material))
        {
            continue;
        }
        let Some(recipe) =
            crate::material::lookup_versioned(&portion.recipe_id, portion.recipe_version)
        else {
            continue;
        };
        let Some(object_density) = recipe.bulk_density.as_ref().map(|density| density.value) else {
            continue;
        };
        return Some(BubbleRide {
            material: recipe.name.clone(),
            object_density_g_per_ml: object_density,
            liquid_density_g_per_ml: liquid_density,
            lift_gas_fraction: ((object_density - liquid_density) / liquid_density).max(0.0),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    /// The arithmetic the experiment is about, on its own: a raisin needs
    /// about a third of its own volume in attached gas, and in a liquid
    /// dense enough it needs none.
    #[test]
    fn the_lift_fraction_is_the_density_gap_over_the_liquid() {
        let ride = |object: f64, liquid: f64| ((object - liquid) / liquid).max(0.0);
        assert!((ride(1.35, 1.0) - 0.35).abs() < 1e-12);
        // A heavy sugar syrup: the raisin floats and there is nothing for
        // the bubbles to do.
        assert_eq!(ride(1.35, 1.4), 0.0);
    }
}
