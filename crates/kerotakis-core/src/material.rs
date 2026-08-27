//! Runtime registry for versioned named-material recipes (BRD-002).
//!
//! Built-ins are generated from the same reviewed registry document as pure
//! species. Optional packs may add recipes at runtime, but may never replace a
//! built-in key or make an existing name/alias ambiguous.

use std::sync::{OnceLock, RwLock};

pub use kerotakis_data::{
    ExpandedMaterialComponent, MaterialBasis, MaterialExpansion, MaterialRecipe, MaterialRole,
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
