//! ORG-012: Polymer population state.
//!
//! Track monomer conversion, molecular-weight distribution moments,
//! and chain-length statistics for polymerization kinetics.

use serde::{Deserialize, Serialize};

/// The state of a polymerization reaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolymerPopulation {
    /// Monomer conversion (0.0–1.0).
    pub conversion: f64,
    /// Number-average molecular weight, g/mol.
    pub mn: f64,
    /// Weight-average molecular weight, g/mol.
    pub mw: f64,
    /// Monomer molar mass, g/mol.
    pub monomer_mw: f64,
}

impl PolymerPopulation {
    /// Polydispersity index (Mw/Mn). A value of 1.0 means monodisperse.
    pub fn pdi(&self) -> f64 {
        if self.mn > 0.0 {
            self.mw / self.mn
        } else {
            f64::NAN
        }
    }

    /// Number-average degree of polymerization.
    pub fn dpn(&self) -> f64 {
        self.mn / self.monomer_mw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdi_of_monodisperse_is_one() {
        let pop = PolymerPopulation {
            conversion: 0.9,
            mn: 10000.0,
            mw: 10000.0,
            monomer_mw: 100.0,
        };
        assert!((pop.pdi() - 1.0).abs() < 1e-10);
        assert!((pop.dpn() - 100.0).abs() < 1e-10);
    }
}
