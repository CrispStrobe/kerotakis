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

#[cfg(test)]
mod tests {
    use super::*;

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
