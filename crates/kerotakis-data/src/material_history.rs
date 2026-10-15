//! Archived recipe revisions are replay data, never extra default shelf entries.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{MaterialRecipe, RegistryDocument, ValidationError, ValidationIssue};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialRecipeHistory {
    pub schema: u32,
    pub source_commit: String,
    pub recipes: Vec<MaterialRecipe>,
    pub legacy_defaults: Vec<MaterialRecipeRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialRecipeRevision {
    pub id: String,
    pub version: u32,
}

impl MaterialRecipeHistory {
    /// Validate old records with the same provenance, fraction and reference
    /// rules as current recipes, while retaining one default per live identity.
    pub fn validate_against(&self, registry: &RegistryDocument) -> Result<(), ValidationError> {
        let mut issues = Vec::new();
        let mut issue = |path: &str, detail: &str| {
            issues.push(ValidationIssue {
                path: path.to_string(),
                detail: detail.to_string(),
            })
        };
        if self.schema != 1 {
            issue("schema", "unsupported material history schema");
        }
        if self.source_commit.len() != 40
            || !self
                .source_commit
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            issue("source_commit", "must identify the source Git commit");
        }
        let mut identities = HashSet::new();
        for recipe in &self.recipes {
            if !identities.insert((recipe.id.as_str(), recipe.version)) {
                issue("recipes", "duplicate historical recipe revision");
            }
            let present: Vec<_> = registry
                .material_recipes
                .iter()
                .filter(|present| present.id == recipe.id)
                .collect();
            if present.len() != 1 {
                issue(
                    "recipes",
                    "historical recipe must retain exactly one current identity",
                );
                continue;
            }
            let present = present[0];
            if present.canonical_key != recipe.canonical_key {
                issue("recipes", "historical canonical identity changed");
            }
            if recipe.version > present.version {
                issue("recipes", "history contains a future recipe revision");
            }
            if recipe.version == present.version && recipe != present {
                issue("recipes", "same revision has different recipe records");
            }
            for other in registry
                .material_recipes
                .iter()
                .filter(|other| other.id != recipe.id)
            {
                for name in std::iter::once(&recipe.canonical_key)
                    .chain(std::iter::once(&recipe.name))
                    .chain(recipe.aliases.values().flatten())
                {
                    if other.matches(name, None) {
                        issue("recipes", "historical alias claimed by another identity");
                    }
                }
            }
            let mut historical_registry = registry.clone();
            historical_registry.material_recipes = vec![recipe.clone()];
            if let Err(error) = historical_registry.validate() {
                for invalid in error.issues {
                    issue(&format!("recipes.{}", invalid.path), &invalid.detail);
                }
            }
        }
        let mut defaults = HashSet::new();
        for entry in &self.legacy_defaults {
            if !defaults.insert(entry.id.as_str()) {
                issue("legacy_defaults", "duplicate legacy default identity");
            }
            if !identities.contains(&(entry.id.as_str(), entry.version)) {
                issue(
                    "legacy_defaults",
                    "default must select an archived revision",
                );
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(ValidationError { issues })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> (RegistryDocument, MaterialRecipeHistory) {
        (
            serde_json::from_str(include_str!(
                "../../../data/registry/registry-source-v1.json"
            ))
            .unwrap(),
            serde_json::from_str(include_str!(
                "../../../data/registry/material-recipe-history-v1.json"
            ))
            .unwrap(),
        )
    }

    #[test]
    fn reviewed_history_validates_before_and_after_a_new_default() {
        let (mut registry, history) = fixtures();
        history.validate_against(&registry).unwrap();
        for recipe in &mut registry.material_recipes {
            if history.recipes.iter().any(|old| old.id == recipe.id) {
                recipe.version += 1;
            }
        }
        history.validate_against(&registry).unwrap();
    }

    #[test]
    fn history_rejects_rewritten_same_revision_future_and_duplicate_records() {
        for mutation in 0..3 {
            let (mut registry, mut history) = fixtures();
            match mutation {
                0 => {
                    *registry
                        .material_recipes
                        .iter_mut()
                        .find(|recipe| recipe.id == history.recipes[0].id)
                        .unwrap() = history.recipes[0].clone();
                    history.recipes[0].preparation = Some("rewritten history".to_string());
                }
                1 => {
                    history.recipes[0].version = registry
                        .material_recipes
                        .iter()
                        .find(|recipe| recipe.id == history.recipes[0].id)
                        .unwrap()
                        .version
                        + 1
                }
                _ => history.recipes.push(history.recipes[0].clone()),
            }
            assert!(history.validate_against(&registry).is_err());
        }
    }

    #[test]
    fn history_rejects_removed_identity_and_stolen_old_alias() {
        for remove in [false, true] {
            let (mut registry, history) = fixtures();
            if remove {
                registry
                    .material_recipes
                    .retain(|recipe| recipe.id != history.recipes[0].id);
            } else {
                let other = registry
                    .material_recipes
                    .iter_mut()
                    .find(|recipe| !history.recipes.iter().any(|old| old.id == recipe.id))
                    .unwrap();
                other.aliases.insert(
                    "en".to_string(),
                    vec![history.recipes[0].canonical_key.clone()],
                );
            }
            assert!(history.validate_against(&registry).is_err());
        }
    }

    #[test]
    fn history_keeps_amount_and_provenance_validation() {
        for invalid_amount in [false, true] {
            let (mut registry, mut history) = fixtures();
            for recipe in &mut registry.material_recipes {
                if recipe.id == history.recipes[0].id {
                    recipe.version += 1;
                }
            }
            if invalid_amount {
                history.recipes[0].components[0].fraction.upper = 2.0;
            } else {
                history.recipes[0].evidence.source_id = "missing-source".to_string();
            }
            assert!(history.validate_against(&registry).is_err());
        }
    }

    #[test]
    fn history_rejects_missing_or_duplicate_legacy_default_revisions() {
        for duplicate in [false, true] {
            let (registry, mut history) = fixtures();
            if duplicate {
                history
                    .legacy_defaults
                    .push(history.legacy_defaults[0].clone());
            } else {
                history.legacy_defaults[0].version = 99;
            }
            assert!(history.validate_against(&registry).is_err());
        }
    }

    #[test]
    fn history_metadata_is_versioned_and_strict() {
        let (registry, history) = fixtures();
        let mut invalid = history.clone();
        invalid.schema = 2;
        assert!(invalid.validate_against(&registry).is_err());
        invalid = history.clone();
        invalid.source_commit = "unknown".to_string();
        assert!(invalid.validate_against(&registry).is_err());
        let mut json = serde_json::to_value(&history).unwrap();
        json["unexpected"] = true.into();
        assert!(serde_json::from_value::<MaterialRecipeHistory>(json).is_err());
    }
}
