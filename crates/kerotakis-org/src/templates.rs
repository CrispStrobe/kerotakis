//! ORG-005: Atom-mapped transformation templates.
//!
//! A reaction template is a SMIRKS pattern that maps reactant atoms to
//! product atoms. The engine applies the template to actual molecules
//! to predict products.

use serde::{Deserialize, Serialize};

/// A curated reaction template with atom mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionTemplate {
    /// Human-readable name (e.g. "esterification").
    pub name: String,
    /// The reaction family this belongs to.
    pub family: String,
    /// SMIRKS string defining the atom-mapped transformation.
    pub smirks: String,
    /// Provenance: where this template came from.
    pub source: String,
    /// Whether this template has been validated against an oracle.
    pub validated: bool,
}

/// Apply a SMIRKS template to reactant SMILES and return product SMILES.
pub fn apply_template(
    template: &ReactionTemplate,
    reactant_smiles: &[&str],
) -> Result<Vec<String>, String> {
    let reactants: Vec<_> = reactant_smiles
        .iter()
        .map(|s| chematic::smiles::parse(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("invalid reactant SMILES: {e}"))?;

    let reactant_refs: Vec<_> = reactants.iter().collect();
    let products = chematic::rxn::run_reactants(&template.smirks, &reactant_refs)
        .map_err(|e| format!("template application failed: {e}"))?;

    Ok(products
        .iter()
        .map(|p| chematic::smiles::write(p))
        .collect())
}

// ── ORG-008: Conditions and incompatibility filters ────────────────

/// Conditions under which a reaction template applies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateConditions {
    /// Minimum temperature required (°C).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_temp_c: Option<f64>,
    /// Maximum temperature (°C) — above this, decomposition dominates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_temp_c: Option<f64>,
    /// Whether a catalyst is required.
    #[serde(default)]
    pub requires_catalyst: bool,
    /// Whether an acid/base medium is required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_medium: Option<String>,
    /// Functional groups that must NOT be present (incompatibility).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incompatible_groups: Vec<String>,
}

impl TemplateConditions {
    /// Check whether conditions are met for the given state.
    pub fn check(
        &self,
        temp_c: f64,
        has_catalyst: bool,
        present_groups: &[&str],
    ) -> Result<(), String> {
        if let Some(min) = self.min_temp_c {
            if temp_c < min {
                return Err(format!("temperature {temp_c}°C below minimum {min}°C"));
            }
        }
        if let Some(max) = self.max_temp_c {
            if temp_c > max {
                return Err(format!("temperature {temp_c}°C above maximum {max}°C"));
            }
        }
        if self.requires_catalyst && !has_catalyst {
            return Err("catalyst required but not present".into());
        }
        for incompat in &self.incompatible_groups {
            if present_groups.iter().any(|g| g == incompat) {
                return Err(format!("incompatible functional group: {incompat}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_reject_low_temperature() {
        let cond = TemplateConditions {
            min_temp_c: Some(100.0),
            ..Default::default()
        };
        assert!(cond.check(25.0, false, &[]).is_err());
        assert!(cond.check(150.0, false, &[]).is_ok());
    }

    #[test]
    fn conditions_reject_incompatible_group() {
        let cond = TemplateConditions {
            incompatible_groups: vec!["aldehyde".into()],
            ..Default::default()
        };
        assert!(cond.check(25.0, false, &["alcohol"]).is_ok());
        assert!(cond.check(25.0, false, &["aldehyde"]).is_err());
    }

    #[test]
    fn template_serializes() {
        let t = ReactionTemplate {
            name: "esterification".into(),
            family: "condensation".into(),
            smirks: "[C:1](=[O:2])[OH:3].[OH:4][C:5]>>[C:1](=[O:2])[O:4][C:5].[OH2:3]".into(),
            source: "curated".into(),
            validated: false,
        };
        let json = serde_json::to_string(&t).unwrap();
        let loaded: ReactionTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "esterification");
    }
}
