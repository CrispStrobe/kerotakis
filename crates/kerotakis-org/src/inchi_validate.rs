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
    // --- small molecules & gases ---
    ("water", "O"),
    ("H2O2", "OO"),
    ("CO2", "O=C=O"),
    ("SO2", "O=S=O"),
    ("NH3", "N"),
    ("NH2Cl", "ClN"),
    ("O2", "O=O"),
    ("H2", "[H][H]"),
    ("N2", "N#N"),
    ("Cl2", "ClCl"),
    // --- acids ---
    ("HCl", "Cl"),
    ("H2SO4", "OS(=O)(=O)O"),
    ("H3PO4", "OP(=O)(O)O"),
    ("CH3COOH", "CC(=O)O"),
    // --- bases ---
    ("NaOH", "[Na+].[OH-]"),
    ("Ca(OH)2", "O[Ca]O"),
    ("Mg(OH)2", "O[Mg]O"),
    ("Zn(OH)2", "O[Zn]O"),
    ("Fe(OH)2", "O[Fe]O"),
    ("Fe(OH)3", "O[Fe](O)O"),
    // --- organic solvents ---
    ("ethanol", "CCO"),
    ("methanol", "CO"),
    ("propanone", "CC(C)=O"),
    ("hexane", "CCCCCC"),
    ("ethyl_acetate", "CCOC(C)=O"),
    // --- monatomic ions ---
    ("Na+", "[Na+]"),
    ("K+", "[K+]"),
    ("Br-", "[Br-]"),
    ("Cl-", "[Cl-]"),
    ("Ca+2", "[Ca+2]"),
    ("Mg+2", "[Mg+2]"),
    ("Sr+2", "[Sr+2]"),
    ("Ag+", "[Ag+]"),
    ("Cu+2", "[Cu+2]"),
    ("Cu+1", "[Cu+1]"),
    ("Fe+2", "[Fe+2]"),
    ("Fe+3", "[Fe+3]"),
    ("Zn+2", "[Zn+2]"),
    ("Mn+2", "[Mn+2]"),
    // --- polyatomic ions ---
    ("CH3COO-", "CC(=O)[O-]"),
    ("NO3-", "[O-][N+](=O)[O-]"),
    ("SO4-2", "[O-]S(=O)(=O)[O-]"),
    ("HCO3-", "OC([O-])=O"),
    ("H2PO4-", "OP(=O)(O)[O-]"),
    // --- metals ---
    ("Al", "[Al]"),
    ("Cu", "[Cu]"),
    ("Zn", "[Zn]"),
    ("Ag", "[Ag]"),
    ("Fe", "[Fe]"),
    // --- binary compounds & oxides ---
    ("NaCl", "[Na+].[Cl-]"),
    ("KCl", "[K+].[Cl-]"),
    ("AgCl", "[Ag+].[Cl-]"),
    ("CaO", "[Ca]=O"),
    ("MgO", "[Mg]=O"),
    ("CuO", "[Cu]=O"),
    ("MnO2", "O=[Mn]=O"),
    // --- salts ---
    ("NaOAc", "[Na+].CC(=O)[O-]"),
    ("NaOCl", "[Na+].[O-]Cl"),
    ("NaHCO3", "[Na+].OC([O-])=O"),
    ("Na2CO3", "[Na+].[Na+].[O-]C([O-])=O"),
    ("Na2SO3", "[Na+].[Na+].[O-]S([O-])=O"),
    ("Na2S2O3", "[Na+].[Na+].[O-]S(=O)(=O)[S-]"),
    ("AgNO3", "[Ag+].[O-][N+](=O)[O-]"),
    ("NaNO3", "[Na+].[O-][N+](=O)[O-]"),
    ("KNO3", "[K+].[O-][N+](=O)[O-]"),
    ("CaCl2", "[Ca+2].[Cl-].[Cl-]"),
    ("CaCO3", "[Ca+2].[O-]C([O-])=O"),
    ("MgSO4", "[Mg+2].[O-]S(=O)(=O)[O-]"),
    ("gypsum", "[Ca+2].[O-]S(=O)(=O)[O-].O.O"),
    ("CuSO4", "[Cu+2].[O-]S(=O)(=O)[O-]"),
    ("KMnO4", "[K+].[O-][Mn](=O)(=O)=O"),
    ("FeSO4", "[Fe+2].[O-]S(=O)(=O)[O-]"),
    ("ZnSO4", "[Zn+2].[O-]S(=O)(=O)[O-]"),
    // --- EXP-13: vitamin C iodine assay ---
    ("ascorbic_acid", "OCC(O)C1OC(=O)C(O)=C1O"),
    ("I2", "II"),
    ("dehydroascorbic_acid", "OCC(O)C1OC(=O)C(=O)C1=O"),
    ("HI", "I"),
    // --- EXP-14: amylase/starch hydrolysis ---
    ("maltose", "OCC1OC(OC2C(O)C(O)C(O)OC2CO)C(O)C(O)C1O"),
    // --- EXP-50: mechanistic selectivity (SN1/SN2/E1/E2) ---
    ("bromoethane", "CCBr"),
    ("tert_butyl_bromide", "CC(C)(C)Br"),
    ("NaBr", "[Na+].[Br-]"),
    ("ethene", "C=C"),
    ("tert_butanol", "CC(C)(C)O"),
    ("isobutylene", "CC(C)=C"),
    ("HBr", "Br"),
    // --- EXP-43: iodine-clock kinetics ---
    ("KI", "[K+].[I-]"),
    ("KIO3", "[K+].[O-][I](=O)=O"),
    ("NaHSO3", "[Na+].OS([O-])=O"),
    ("NaHSO4", "[Na+].OS(=O)(=O)[O-]"),
    // --- BRD-012.S02: school-essential salts and the barium pair ---
    ("NH4Cl", "[NH4+].[Cl-]"),
    ("NH4+", "[NH4+]"),
    ("FeCl3", "[Fe+3].[Cl-].[Cl-].[Cl-]"),
    ("Na2SO4", "[Na+].[Na+].[O-]S(=O)(=O)[O-]"),
    ("BaCl2", "[Ba+2].[Cl-].[Cl-]"),
    ("Ba(OH)2", "O[Ba]O"),
    ("Ba+2", "[Ba+2]"),
    ("BaSO4", "[Ba+2].[O-]S(=O)(=O)[O-]"),
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

/// Validate all curated registry species.
pub fn validate_registry() -> Vec<InchiValidation> {
    CURATED_STRUCTURES
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
        assert!(results.len() >= 65, "should validate all curated species");

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
