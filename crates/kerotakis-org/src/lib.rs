//! # kerotakis-org
//!
//! Organic chemistry structure toolkit for Kerotakis, built on the
//! pure-Rust `chematic` library (MIT/Apache-2.0, wasm-compatible).
//!
//! Provides:
//! - SMILES parsing and canonical identity (ORG-001, ORG-003)
//! - InChI/InChIKey generation for cross-checking (ORG-003)
//! - Molecular formula and weight (ORG-003)

use serde::{Deserialize, Serialize};

/// Error type for organic chemistry operations.
#[derive(Debug, thiserror::Error)]
pub enum OrgError {
    #[error("invalid SMILES: {0}")]
    InvalidSmiles(String),
    #[error("InChI generation failed: {0}")]
    InchiFailed(String),
}

/// A parsed molecule with its canonical identifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMolecule {
    pub input_smiles: String,
    pub canonical_smiles: String,
    pub formula: String,
    pub molecular_weight: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inchikey: Option<String>,
    pub atom_count: usize,
    pub bond_count: usize,
    pub formal_charge: i32,
}

/// Parse a SMILES string and compute canonical identifiers.
pub fn parse_smiles(smiles: &str) -> Result<ParsedMolecule, OrgError> {
    let mol = chematic::smiles::parse(smiles)
        .map_err(|e| OrgError::InvalidSmiles(format!("{e}")))?;

    let canonical = chematic::smiles::write(&mol);
    let mw = chematic::chem::molecular_weight(&mol);

    // Build formula with implicit H (mol.formula() only counts heavy atoms)
    let formula = {
        use std::collections::BTreeMap;
        let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
        for (idx, atom) in mol.atoms() {
            *counts.entry(atom.element.symbol()).or_insert(0) += 1;
            let ih = mol.implicit_hydrogen_count(idx);
            if ih > 0 {
                *counts.entry("H").or_insert(0) += ih as u32;
            }
        }
        let mut f = String::new();
        if let Some(&c) = counts.get("C") {
            f.push('C');
            if c > 1 { f.push_str(&c.to_string()); }
            counts.remove("C");
        }
        if let Some(&h) = counts.get("H") {
            f.push('H');
            if h > 1 { f.push_str(&h.to_string()); }
            counts.remove("H");
        }
        for (el, count) in &counts {
            f.push_str(el);
            if *count > 1 { f.push_str(&count.to_string()); }
        }
        f
    };

    let atom_count = mol.atom_count();
    let bond_count = mol.bond_count();
    let formal_charge: i32 = mol.atoms().map(|(_, a)| a.charge as i32).sum();

    let inchi_str = chematic::inchi::inchi(&mol);
    let inchikey = if inchi_str.is_empty() {
        None
    } else {
        Some(chematic::inchi::inchi_key(&inchi_str))
    };

    Ok(ParsedMolecule {
        input_smiles: smiles.to_string(),
        canonical_smiles: canonical,
        formula,
        molecular_weight: mw,
        inchikey,
        atom_count,
        bond_count,
        formal_charge,
    })
}

/// Cross-check: does a SMILES string produce the expected InChIKey?
pub fn verify_inchikey(smiles: &str, expected_inchikey: &str) -> Result<bool, OrgError> {
    let mol = parse_smiles(smiles)?;
    match &mol.inchikey {
        Some(key) => Ok(key == expected_inchikey),
        None => Err(OrgError::InchiFailed("InChIKey generation failed".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_water() {
        let mol = parse_smiles("O").unwrap();
        assert_eq!(mol.formula, "H2O");
        assert!((mol.molecular_weight - 18.015).abs() < 0.1);
        assert_eq!(mol.formal_charge, 0);
    }

    #[test]
    fn parse_ethanol() {
        let mol = parse_smiles("CCO").unwrap();
        assert_eq!(mol.formula, "C2H6O");
        assert!((mol.molecular_weight - 46.07).abs() < 0.1);
    }

    #[test]
    fn invalid_smiles_errors() {
        assert!(parse_smiles("not_a_molecule!!!").is_err());
    }

    #[test]
    fn same_input_produces_same_output() {
        let a = parse_smiles("CCO").unwrap();
        let b = parse_smiles("CCO").unwrap();
        assert_eq!(a.canonical_smiles, b.canonical_smiles);
        assert_eq!(a.formula, b.formula);
    }
}
