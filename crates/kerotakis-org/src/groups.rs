//! ORG-004: Functional group perception via SMARTS patterns.
//!
//! Identifies the functional groups the codex mentions: alcohol, aldehyde,
//! ketone, carboxylic acid, ester, ether, amine, amide.

use chematic::smarts;

/// A recognized functional group.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FunctionalGroup {
    pub name: &'static str,
    pub smarts: &'static str,
    pub count: usize,
}

/// SMARTS definitions for school-chemistry functional groups.
const PATTERNS: &[(&str, &str)] = &[
    ("alcohol", "[OX2H][CX4]"),
    ("aldehyde", "[CX3H1](=O)[#6]"),
    ("ketone", "[#6][CX3](=O)[#6]"),
    ("carboxylic acid", "[CX3](=O)[OX2H1]"),
    ("ester", "[#6][CX3](=O)[OX2][#6]"),
    ("ether", "[OD2]([#6])[#6]"),
    ("primary amine", "[NX3H2][CX4]"),
    ("secondary amine", "[NX3H1]([CX4])[CX4]"),
    ("amide", "[NX3][CX3](=[OX1])"),
];

/// Identify functional groups in a SMILES string.
pub fn perceive_groups(smiles_str: &str) -> Vec<FunctionalGroup> {
    let mol = match chematic::smiles::parse(smiles_str) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for &(name, pattern) in PATTERNS {
        let matches = smarts::find_matches(pattern, &mol);
        if !matches.is_empty() {
            results.push(FunctionalGroup {
                name,
                smarts: pattern,
                count: matches.len(),
            });
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ethanol_has_alcohol_group() {
        let groups = perceive_groups("CCO");
        assert!(
            groups.iter().any(|g| g.name == "alcohol"),
            "ethanol should have an alcohol group, got: {groups:?}"
        );
    }

    #[test]
    fn acetic_acid_has_carboxylic_acid() {
        let groups = perceive_groups("CC(=O)O");
        assert!(
            groups.iter().any(|g| g.name == "carboxylic acid"),
            "acetic acid should have carboxylic acid, got: {groups:?}"
        );
    }

    #[test]
    fn diethyl_ether_has_ether() {
        let groups = perceive_groups("CCOCC");
        assert!(
            groups.iter().any(|g| g.name == "ether"),
            "diethyl ether should have ether, got: {groups:?}"
        );
    }

    #[test]
    fn water_has_no_organic_groups() {
        let groups = perceive_groups("O");
        assert!(groups.is_empty(), "water should have no organic groups");
    }
}
