//! CAP-13: Cross-validate InChI/InChIKey between the pure-Rust chematic
//! implementation and the official IUPAC InChI C library.
//!
//! The chematic-inchi crate provides wasm-compatible InChI generation.
//! The inchi crate wraps the official IUPAC C library (MIT since v1.07.1).
//! This module compares both outputs and flags any disagreement.

/// Result of cross-validating one species' InChIKey.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InchiValidation {
    pub species_key: String,
    pub smiles: String,
    pub chematic_inchikey: Option<String>,
    #[cfg(feature = "native-inchi")]
    pub native_inchikey: Option<String>,
    pub match_status: MatchStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    /// Both implementations agree.
    Match,
    /// Only one implementation produced a result.
    Partial,
    /// Both produced results but they disagree.
    Mismatch,
    /// Neither produced a result.
    BothFailed,
}

/// Cross-validate a SMILES string against both InChI implementations.
pub fn validate_smiles(species_key: &str, smiles: &str) -> InchiValidation {
    // Pure-Rust path (chematic)
    let chematic_key = {
        let mol = chematic::smiles::parse(smiles).ok();
        mol.and_then(|m| {
            let inchi_str = chematic::inchi::inchi(&m);
            if inchi_str.is_empty() {
                None
            } else {
                Some(chematic::inchi::inchi_key(&inchi_str))
            }
        })
    };

    #[cfg(feature = "native-inchi")]
    let native_key = {
        inchi::inchi_from_smiles(smiles)
            .ok()
            .map(|result| result.inchikey)
    };

    #[cfg(not(feature = "native-inchi"))]
    let native_key: Option<String> = None;

    let status = match (&chematic_key, &native_key) {
        (Some(a), Some(b)) if a == b => MatchStatus::Match,
        (Some(_), Some(_)) => MatchStatus::Mismatch,
        (Some(_), None) | (None, Some(_)) => MatchStatus::Partial,
        (None, None) => MatchStatus::BothFailed,
    };

    InchiValidation {
        species_key: species_key.to_string(),
        smiles: smiles.to_string(),
        chematic_inchikey: chematic_key,
        #[cfg(feature = "native-inchi")]
        native_inchikey: native_key,
        match_status: status,
    }
}

/// Validate all registry species that have SMILES representations.
pub fn validate_registry() -> Vec<InchiValidation> {
    // Known SMILES for registry species (a subset for cross-validation)
    let test_cases = [
        ("water", "O"),
        ("ethanol", "CCO"),
        ("NaCl", "[Na+].[Cl-]"),
        ("CH3COOH", "CC(=O)O"),
        ("CO2", "O=C=O"),
        ("NH3", "N"),
        ("H2SO4", "OS(=O)(=O)O"),
        ("NaOH", "[Na+].[OH-]"),
        ("HCl", "Cl"),
        ("CaCO3", "[Ca+2].[O-]C([O-])=O"),
    ];

    test_cases
        .iter()
        .map(|(key, smiles)| validate_smiles(key, smiles))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_validates() {
        let result = validate_smiles("water", "O");
        assert!(
            result.chematic_inchikey.is_some(),
            "chematic should produce InChIKey for water"
        );
        // Without native-inchi feature, status is Partial (chematic only)
        // With native-inchi, status should be Match
        assert_ne!(result.match_status, MatchStatus::Mismatch);
    }

    #[test]
    fn registry_cross_validation_runs() {
        let results = validate_registry();
        assert!(results.len() >= 10, "should validate at least 10 species");

        let mismatches: Vec<_> = results
            .iter()
            .filter(|r| r.match_status == MatchStatus::Mismatch)
            .collect();
        assert!(
            mismatches.is_empty(),
            "InChI mismatch for: {:?}",
            mismatches.iter().map(|r| &r.species_key).collect::<Vec<_>>()
        );
    }
}
