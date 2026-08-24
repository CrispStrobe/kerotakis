//! CAP-13: Cross-validate InChI/InChIKey between the pure-Rust chematic
//! implementation and the official IUPAC InChI C library.
//!
//! The chematic-inchi crate provides wasm-compatible InChI generation.
//! The inchi crate wraps the official IUPAC C library (MIT since v1.07.1).
//! This module compares both outputs and flags any disagreement.

/// The curated structure claims (CAP-13): registry species with a
/// hand-pinned SMILES. Each one is recomputed by the official IUPAC
/// InChI library in the gate (`tests/native_identity.rs`) and must
/// reproduce the registry's `canonical_key` exactly. Species without a
/// molecular structure identity (minerals by formula unit, enzymes,
/// aromatic dyes awaiting kekulisation care) join as their SMILES are
/// curated.
pub const CURATED_STRUCTURES: &[(&str, &str)] = &[
    ("water", "O"),
    ("ethanol", "CCO"),
    ("methanol", "CO"),
    ("propanone", "CC(C)=O"),
    ("hexane", "CCCCCC"),
    ("ethyl_acetate", "CCOC(C)=O"),
    ("CH3COOH", "CC(=O)O"),
    ("CH3COO-", "CC(=O)[O-]"),
    ("NaOAc", "[Na+].CC(=O)[O-]"),
    ("CO2", "O=C=O"),
    ("NH3", "N"),
    ("H2O2", "OO"),
    ("HCl", "Cl"),
    ("H2SO4", "OS(=O)(=O)O"),
    ("H3PO4", "OP(=O)(O)O"),
    ("NaCl", "[Na+].[Cl-]"),
    ("NaOH", "[Na+].[OH-]"),
    ("KCl", "[K+].[Cl-]"),
    ("O2", "O=O"),
    ("H2", "[H][H]"),
    ("N2", "N#N"),
    ("Cl2", "ClCl"),
    ("SO2", "O=S=O"),
];

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

/// Standard InChIKey from the official IUPAC library: SMILES parsed by
/// chematic, written as a V2000 molfile, handed to the reference
/// implementation. This is the identity authority; chematic's own key
/// is a canonical key of a different algorithm and is expected to
/// differ (adopting the official key everywhere is the rest of
/// CAP-13).
#[cfg(feature = "native-inchi")]
pub fn native_inchikey_from_smiles(smiles: &str) -> Result<String, crate::OrgError> {
    let mol = chematic::smiles::parse(smiles)
        .map_err(|e| crate::OrgError::InchiFailed(format!("SMILES parse: {e}")))?;
    let molfile = chematic::mol::write_mol(&mol, &chematic::mol::MolMetadata::default());
    let out = inchi::from_molfile(&molfile, ())
        .map_err(|e| crate::OrgError::InchiFailed(format!("official InChI: {e}")))?;
    inchi::inchikey(out.inchi()).map_err(|e| crate::OrgError::InchiFailed(format!("InChIKey: {e}")))
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
    let native_key = native_inchikey_from_smiles(smiles).ok();

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
        // Without native-inchi the status is Partial (chematic only).
        // With native-inchi it is Mismatch, and that is *correct*:
        // chematic's canonical key is not the standard InChIKey. The
        // standard-identity contract lives in tests/native_identity.rs.
        assert_ne!(result.match_status, MatchStatus::BothFailed);
    }

    #[test]
    fn registry_cross_validation_runs() {
        let results = validate_registry();
        assert!(results.len() >= 10, "should validate at least 10 species");

        // chematic's canonical key is not the standard InChIKey, so with
        // the official library attached a Mismatch is the *expected*
        // status — the standard-identity contract lives in
        // tests/native_identity.rs, where the official key must equal
        // the registry's curated canonical_key. Here we only require
        // that every species yields a chematic key at all.
        assert!(
            results.iter().all(|r| r.chematic_inchikey.is_some()),
            "chematic failed to produce a key for: {:?}",
            results
                .iter()
                .filter(|r| r.chematic_inchikey.is_none())
                .map(|r| &r.species_key)
                .collect::<Vec<_>>()
        );
    }
}
