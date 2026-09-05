//! Runtime registry for versioned named-material recipes (BRD-002).
//!
//! Built-ins are generated from the same reviewed registry document as pure
//! species. Optional packs may add recipes at runtime, but may never replace a
//! built-in key or make an existing name/alias ambiguous.

use std::sync::{OnceLock, RwLock};

pub use kerotakis_data::{
    CultureMetabolism, ExpandedMaterialComponent, MaterialBasis, MaterialConfidence,
    MaterialExpansion, MaterialGeometry, MaterialPhysicalForm, MaterialRecipe, MaterialRole,
};

include!(concat!(env!("OUT_DIR"), "/materials_generated.rs"));

fn builtins() -> &'static [MaterialRecipe] {
    static RECIPES: OnceLock<Vec<MaterialRecipe>> = OnceLock::new();
    RECIPES
        .get_or_init(|| {
            serde_json::from_str(BUILTIN_MATERIAL_RECIPES_JSON)
                .expect("build-generated material recipes must parse")
        })
        .as_slice()
}

fn loaded() -> &'static RwLock<Vec<MaterialRecipe>> {
    static RECIPES: OnceLock<RwLock<Vec<MaterialRecipe>>> = OnceLock::new();
    RECIPES.get_or_init(|| RwLock::new(Vec::new()))
}

pub fn lookup(query: &str, language: Option<&str>) -> Option<MaterialRecipe> {
    builtins()
        .iter()
        .find(|recipe| recipe.matches(query, language))
        .cloned()
        .or_else(|| {
            loaded()
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .iter()
                .find(|recipe| recipe.matches(query, language))
                .cloned()
        })
}

pub fn all() -> Vec<MaterialRecipe> {
    let mut recipes = builtins().to_vec();
    recipes.extend(
        loaded()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .cloned(),
    );
    recipes
}

/// Grains of a named powder sitting on the water rather than in it, with
/// how much of the visible surface they cover.
///
/// `SurfaceFloater` was declared, validated, exported and read by nothing.
/// Ground black pepper carries it, and the whole point of pepper in the
/// pepper-and-soap demonstration is that it floats until the detergent
/// arrives — while `look` said "the liquid is colourless and clear" over a
/// beaker with a skin of pepper on it. A role that describes an observable
/// and produces no observation is a fact the bench holds and does not say,
/// which is this project's commonest defect and was sitting inside the
/// mechanism named for it.
///
/// Coverage is the recipe's own declared dose response and saturates at 1:
/// it is a bounded classroom observable, not a packing fraction, and no
/// claim is made about grain size or how the raft breaks up.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceFloatObservation {
    pub material: String,
    pub coverage: f64,
}

pub fn surface_floaters(vessel: &crate::Vessel) -> Vec<SurfaceFloatObservation> {
    if vessel.liquid_volume().0 <= 0.0 {
        return Vec::new();
    }
    let mut seen: Vec<SurfaceFloatObservation> = Vec::new();
    for portion in &vessel.unresolved_materials {
        let Some(recipe) = lookup_versioned(&portion.recipe_id, portion.recipe_version) else {
            continue;
        };
        let Some(saturation) = recipe.roles.iter().find_map(|role| match role {
            MaterialRole::SurfaceFloater { saturation_amount } => Some(*saturation_amount),
            _ => None,
        }) else {
            continue;
        };
        if portion.amount <= 0.0 || saturation <= 0.0 {
            continue;
        }
        let coverage = (portion.amount / saturation).clamp(0.0, 1.0);
        if let Some(existing) = seen.iter_mut().find(|item| item.material == recipe.name) {
            existing.coverage = (existing.coverage + coverage).min(1.0);
        } else {
            seen.push(SurfaceFloatObservation {
                material: recipe.name,
                coverage,
            });
        }
    }
    seen
}

/// Recover the exact recipe pinned by an unresolved material portion.
pub fn lookup_versioned(id: &str, version: u32) -> Option<MaterialRecipe> {
    all()
        .into_iter()
        .find(|recipe| recipe.id == id && recipe.version == version)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImmiscibleLiquidLayer {
    pub material: String,
    pub key: String,
    pub recipe_id: String,
    pub volume_l: f64,
    pub srgb: [u8; 3],
    pub colour_word: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColloidObservation {
    pub srgb: [u8; 3],
    pub cloudiness: f64,
}

fn portion_volume_l(
    portion: &crate::vessel::UnresolvedMaterialPortion,
    recipe: &MaterialRecipe,
) -> f64 {
    match portion.basis {
        MaterialBasis::MassFraction => recipe
            .bulk_density
            .as_ref()
            .map(|density| portion.amount / density.value / 1000.0)
            .unwrap_or(0.0),
        MaterialBasis::VolumeFraction => portion.amount / 1000.0,
        MaterialBasis::MoleFraction => 0.0,
    }
}

/// Visible unresolved liquids, aggregated by pinned recipe. Chemical solvent
/// volume intentionally remains `Vessel::liquid_volume`; this volume is for
/// geometry and rendering and must not leak into aqueous concentrations.
pub fn immiscible_liquid_layers(vessel: &crate::Vessel) -> Vec<ImmiscibleLiquidLayer> {
    let mut layers: Vec<ImmiscibleLiquidLayer> = Vec::new();
    for portion in &vessel.unresolved_materials {
        let Some(recipe) = lookup_versioned(&portion.recipe_id, portion.recipe_version) else {
            continue;
        };
        let Some((srgb, colour_word)) = recipe.roles.iter().find_map(|role| match role {
            MaterialRole::AqueousImmiscibleLiquid { srgb, colour_word } => {
                Some((*srgb, colour_word.clone()))
            }
            _ => None,
        }) else {
            continue;
        };
        let volume_l = portion_volume_l(portion, &recipe);
        if volume_l <= 0.0 {
            continue;
        }
        if let Some(existing) = layers.iter_mut().find(|layer| layer.recipe_id == recipe.id) {
            existing.volume_l += volume_l;
        } else {
            layers.push(ImmiscibleLiquidLayer {
                material: recipe.name,
                key: recipe.canonical_key,
                recipe_id: recipe.id,
                volume_l,
                srgb,
                colour_word,
            });
        }
    }
    layers
}

/// Unresolved volume that belongs to the ordinary mixed liquid rather than a
/// separate material layer. It is render/geometry state only and deliberately
/// does not enter aqueous concentrations.
pub fn homogeneous_unresolved_liquid_volume_l(vessel: &crate::Vessel) -> f64 {
    vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            if !matches!(
                recipe.physical_form,
                kerotakis_data::MaterialPhysicalForm::HomogeneousLiquid
            ) || recipe
                .roles
                .iter()
                .any(|role| matches!(role, MaterialRole::AqueousImmiscibleLiquid { .. }))
            {
                return None;
            }
            Some(portion_volume_l(portion, &recipe))
        })
        .sum()
}

/// Visible opacity contributed by conserved named colloids. Multiple colloids
/// combine by taking the strongest bounded contribution; v1 deliberately does
/// not pretend to solve droplet-size distributions or multiple scattering.
pub fn colloid_observation(vessel: &crate::Vessel) -> Option<ColloidObservation> {
    let visible_l = vessel.liquid_volume().0 + homogeneous_unresolved_liquid_volume_l(vessel);
    let curdling = crate::curdling::observe(vessel);
    if visible_l <= 1e-12 {
        return None;
    }
    vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            recipe.roles.iter().find_map(|role| {
                let MaterialRole::OpaqueLiquidColloid {
                    srgb,
                    opacity_saturation_g_per_litre,
                } = role
                else {
                    return None;
                };
                let mass_g = match portion.basis {
                    MaterialBasis::MassFraction => portion.amount,
                    MaterialBasis::VolumeFraction => recipe
                        .bulk_density
                        .as_ref()
                        .map(|density| portion.amount * density.value)
                        .unwrap_or(0.0),
                    MaterialBasis::MoleFraction => 0.0,
                } * (1.0
                    - curdling
                        .as_ref()
                        .filter(|curds| curds.recipe_id == recipe.id)
                        .map(|curds| curds.opacity_reduction)
                        .unwrap_or(0.0));
                Some(ColloidObservation {
                    srgb: *srgb,
                    cloudiness: (mass_g / visible_l / opacity_saturation_g_per_litre)
                        .clamp(0.0, 1.0),
                })
            })
        })
        .max_by(|a, b| a.cloudiness.total_cmp(&b.cloudiness))
}

/// One named solid the registry deliberately does not resolve into installed
/// species, still present as conserved matter.
#[derive(Debug, Clone, PartialEq)]
pub struct ConservedSolidObservation {
    pub material: String,
    pub recipe_id: String,
    pub colour_word: String,
    pub srgb: [u8; 3],
    /// Conserved amount in the recipe's own basis.
    pub amount: f64,
    /// Reviewed bulk density of the whole named object, when supplied.
    /// This deliberately differs from the density of any resolved ingredient:
    /// pores and trapped air are part of why objects such as apples and pumice
    /// float.
    pub bulk_density_g_per_ml: Option<f64>,
}

/// A named, coherent solid object whose unresolved balance keeps its recipe
/// identity in vessel state. Unlike powders and suspensions, its reviewed
/// bulk density describes the object that floats or sinks.
#[derive(Debug, Clone, PartialEq)]
pub struct BulkSolidObservation {
    pub material: String,
    pub recipe_id: String,
    pub amount: f64,
    pub bulk_density_g_per_ml: f64,
}

pub fn bulk_solid_objects(vessel: &crate::Vessel) -> Vec<BulkSolidObservation> {
    let mut seen: Vec<BulkSolidObservation> = Vec::new();
    for portion in &vessel.unresolved_materials {
        let Some(recipe) = lookup_versioned(&portion.recipe_id, portion.recipe_version) else {
            continue;
        };
        if portion.amount <= 0.0
            || !matches!(
                recipe.physical_form,
                MaterialPhysicalForm::BulkSolid | MaterialPhysicalForm::CompositeObject { .. }
            )
            || recipe
                .roles
                .iter()
                .any(|role| matches!(role, MaterialRole::SurfaceFloater { .. }))
        {
            continue;
        }
        let Some(density) = recipe.bulk_density.map(|density| density.value) else {
            continue;
        };
        if let Some(existing) = seen.iter_mut().find(|item| item.recipe_id == recipe.id) {
            existing.amount += portion.amount;
        } else {
            seen.push(BulkSolidObservation {
                material: recipe.name,
                recipe_id: recipe.id,
                amount: portion.amount,
                bulk_density_g_per_ml: density,
            });
        }
    }
    seen
}

/// Named solids whose substance has no installed species, in the order they
/// were dispensed and aggregated by pinned recipe.
///
/// The observation carries the whole object's reviewed bulk density so the
/// buoyancy model compares like with like instead of treating a porous object
/// as a loose pile of its denser resolved ingredients.
pub fn conserved_unresolved_solids(vessel: &crate::Vessel) -> Vec<ConservedSolidObservation> {
    let mut seen: Vec<ConservedSolidObservation> = Vec::new();
    for portion in &vessel.unresolved_materials {
        let Some(recipe) = lookup_versioned(&portion.recipe_id, portion.recipe_version) else {
            continue;
        };
        let Some((srgb, colour_word)) = recipe.roles.iter().find_map(|role| match role {
            MaterialRole::ConservedUnresolvedSolid { srgb, colour_word } => {
                Some((*srgb, colour_word.clone()))
            }
            _ => None,
        }) else {
            continue;
        };
        if portion.amount <= 0.0 {
            continue;
        }
        if let Some(existing) = seen.iter_mut().find(|item| item.recipe_id == recipe.id) {
            existing.amount += portion.amount;
        } else {
            seen.push(ConservedSolidObservation {
                material: recipe.name,
                recipe_id: recipe.id,
                colour_word,
                srgb,
                amount: portion.amount,
                bulk_density_g_per_ml: recipe.bulk_density.map(|density| density.value),
            });
        }
    }
    seen
}

/// Whether this pinned unresolved portion follows a liquid pour/mix.
pub fn unresolved_portion_is_liquid(portion: &crate::vessel::UnresolvedMaterialPortion) -> bool {
    lookup_versioned(&portion.recipe_id, portion.recipe_version).is_some_and(|recipe| {
        matches!(
            recipe.physical_form,
            kerotakis_data::MaterialPhysicalForm::HomogeneousLiquid
        )
    })
}

/// Mass represented by conserved unresolved homogeneous-liquid portions
/// whenever their recipe basis defines an honest conversion. Other physical
/// forms retain their existing accounting boundary; a mole-fraction aggregate
/// still has no molecular mass.
pub fn unresolved_material_mass_g(vessel: &crate::Vessel) -> f64 {
    vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            if !matches!(
                recipe.physical_form,
                kerotakis_data::MaterialPhysicalForm::HomogeneousLiquid
            ) {
                return None;
            }
            Some(match portion.basis {
                MaterialBasis::MassFraction => portion.amount,
                MaterialBasis::VolumeFraction => recipe
                    .bulk_density
                    .as_ref()
                    .map(|density| portion.amount * density.value)
                    .unwrap_or(0.0),
                MaterialBasis::MoleFraction => 0.0,
            })
        })
        .sum()
}

/// Translate a reviewed recipe-level opaque-pigment role into the core's
/// fixed visible-band representation. Invalid optional-pack data yields no
/// optics; source documents are rejected earlier by the data validator.
pub fn pigment_optics(recipe: &MaterialRecipe) -> Option<crate::pigment::PigmentOptics> {
    recipe.roles.iter().find_map(|role| {
        let MaterialRole::OpaquePigment {
            absorption,
            scattering,
        } = role
        else {
            return None;
        };
        let absorption: crate::Spectrum = absorption.as_slice().try_into().ok()?;
        let scattering: crate::Spectrum = scattering.as_slice().try_into().ok()?;
        Some(crate::pigment::PigmentOptics {
            key: recipe.canonical_key.clone(),
            absorption,
            scattering,
        })
    })
}

/// Computed shelf swatch for one opaque paint recipe.
pub fn pigment_swatch(recipe: &MaterialRecipe) -> Option<crate::Rgb> {
    let optics = pigment_optics(recipe)?;
    crate::pigment::opaque_mixture_colour(&[crate::pigment::PigmentAmount {
        key: &optics.key,
        amount: 1.0,
        optics: Some(&optics),
    }])
    .ok()
}

/// Register already-validated recipes from an optional pack. Conflicts are
/// skipped as a whole recipe; built-in identity always wins.
pub fn register_loaded(recipes: Vec<MaterialRecipe>) -> (usize, usize) {
    let mut target = loaded()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut added = 0;
    let mut skipped = 0;
    for recipe in recipes {
        let conflicts = recipe_names(&recipe).into_iter().any(|name| {
            builtins()
                .iter()
                .chain(target.iter())
                .any(|present| present.matches(&name, None))
        });
        if conflicts {
            skipped += 1;
        } else {
            target.push(recipe);
            added += 1;
        }
    }
    (added, skipped)
}

fn recipe_names(recipe: &MaterialRecipe) -> Vec<String> {
    let mut names = vec![recipe.canonical_key.clone(), recipe.name.clone()];
    names.extend(recipe.aliases.values().flatten().cloned());
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_resolve_localized_names_and_expand() {
        let recipe = lookup("Wasserstoffperoxid 3%", Some("de")).expect("German alias");
        let expansion = recipe.expand(100.0, 0).expect("fixed expansion");
        assert_eq!(expansion.recipe_version, 1);
        assert_eq!(expansion.components[0].species_id, "H2O2");
        assert!((expansion.components[0].amount - 3.0).abs() < 1e-12);
    }
}
