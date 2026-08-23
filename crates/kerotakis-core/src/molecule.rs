//! ORG-002: Molecule graph — bond orders, formal charge, stereochemistry.
//!
//! This is the data structure for representing molecular graphs. It does
//! not yet include perception algorithms (ORG-004) or reaction templates
//! (ORG-005). Those build on this representation.

use serde::{Deserialize, Serialize};

/// A molecular graph: atoms connected by bonds, with formal charges and
/// optional stereochemistry. No 3D coordinates — the graph is topological.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoleculeGraph {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
    /// Canonical identifier (InChIKey or SMILES).
    #[serde(default)]
    pub canonical_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atom {
    pub index: usize,
    pub element: String,
    pub formal_charge: i32,
    #[serde(default)]
    pub isotope: Option<u32>,
    #[serde(default)]
    pub implicit_hydrogens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bond {
    pub from: usize,
    pub to: usize,
    pub order: BondOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BondOrder {
    Single,
    Double,
    Triple,
    Aromatic,
}

impl MoleculeGraph {
    pub fn atom_count(&self) -> usize {
        self.atoms.len()
    }

    pub fn bond_count(&self) -> usize {
        self.bonds.len()
    }

    pub fn net_charge(&self) -> i32 {
        self.atoms.iter().map(|a| a.formal_charge).sum()
    }

    pub fn molecular_formula(&self) -> String {
        use std::collections::BTreeMap;
        let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
        for atom in &self.atoms {
            *counts.entry(&atom.element).or_insert(0) += 1;
            if let Some(h) = atom.implicit_hydrogens {
                *counts.entry("H").or_insert(0) += h;
            }
        }
        let mut formula = String::new();
        // Hill system: C first, H second, then alphabetical
        if let Some(&c) = counts.get("C") {
            formula.push('C');
            if c > 1 {
                formula.push_str(&c.to_string());
            }
            counts.remove("C");
            if let Some(&h) = counts.get("H") {
                formula.push('H');
                if h > 1 {
                    formula.push_str(&h.to_string());
                }
                counts.remove("H");
            }
        }
        for (el, count) in &counts {
            formula.push_str(el);
            if *count > 1 {
                formula.push_str(&count.to_string());
            }
        }
        formula
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> MoleculeGraph {
        MoleculeGraph {
            atoms: vec![Atom {
                index: 0,
                element: "O".into(),
                formal_charge: 0,
                isotope: None,
                implicit_hydrogens: Some(2),
            }],
            bonds: vec![],
            canonical_id: Some("XLYOFNOQVPJJNP-UHFFFAOYSA-N".into()),
        }
    }

    fn nacl() -> MoleculeGraph {
        MoleculeGraph {
            atoms: vec![
                Atom {
                    index: 0,
                    element: "Na".into(),
                    formal_charge: 1,
                    isotope: None,
                    implicit_hydrogens: None,
                },
                Atom {
                    index: 1,
                    element: "Cl".into(),
                    formal_charge: -1,
                    isotope: None,
                    implicit_hydrogens: None,
                },
            ],
            bonds: vec![Bond {
                from: 0,
                to: 1,
                order: BondOrder::Single,
            }],
            canonical_id: None,
        }
    }

    #[test]
    fn water_formula() {
        assert_eq!(water().molecular_formula(), "H2O");
    }

    #[test]
    fn nacl_net_charge() {
        assert_eq!(nacl().net_charge(), 0);
    }

    #[test]
    fn molecule_graph_round_trips() {
        let mol = nacl();
        let json = serde_json::to_string(&mol).unwrap();
        let loaded: MoleculeGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, mol);
    }
}
