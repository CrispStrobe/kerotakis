//! ADV-005: Nuclear module design — nuclide ledger.
//!
//! A separate ledger for radioactive species tracks isotopic identity,
//! decay chains, and activity. This is deliberately separate from the
//! chemical element ledger: the nuclide ledger distinguishes ¹⁴C from
//! ¹²C even though the chemistry treats both as "C".

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A specific nuclide: element + mass number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nuclide {
    /// Element symbol (e.g. "C", "U", "Ra").
    pub element: String,
    /// Mass number (e.g. 14, 235, 226).
    pub mass_number: u32,
}

impl Nuclide {
    pub fn new(element: impl Into<String>, mass_number: u32) -> Self {
        Self {
            element: element.into(),
            mass_number,
        }
    }

    /// Standard notation (e.g. "C-14", "U-235").
    pub fn notation(&self) -> String {
        format!("{}-{}", self.element, self.mass_number)
    }
}

/// Decay mode for a radioactive nuclide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayMode {
    Alpha,
    BetaMinus,
    BetaPlus,
    ElectronCapture,
    Gamma,
    SpontaneousFission,
}

/// A decay chain entry: parent decays to daughter with a half-life.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayChainEntry {
    pub parent: Nuclide,
    pub daughter: Nuclide,
    pub mode: DecayMode,
    /// Half-life in seconds.
    pub half_life_s: f64,
    /// Branching ratio (0.0–1.0).
    pub branching_ratio: f64,
}

/// The nuclide ledger: tracks amounts of specific isotopes separately
/// from the bulk element inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NuclideLedger {
    /// Nuclide → moles.
    pub inventory: BTreeMap<Nuclide, f64>,
}

impl NuclideLedger {
    /// Activity of a nuclide in becquerels (disintegrations per second).
    pub fn activity_bq(&self, nuclide: &Nuclide, half_life_s: f64) -> f64 {
        let moles = self.inventory.get(nuclide).copied().unwrap_or(0.0);
        let n_atoms = moles * 6.022e23;
        let lambda = (2.0_f64).ln() / half_life_s;
        n_atoms * lambda
    }

    /// Deposit moles of a nuclide.
    pub fn deposit(&mut self, nuclide: Nuclide, moles: f64) {
        *self.inventory.entry(nuclide).or_insert(0.0) += moles;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c14_notation() {
        let c14 = Nuclide::new("C", 14);
        assert_eq!(c14.notation(), "C-14");
    }

    #[test]
    fn activity_scales_with_amount() {
        let mut ledger = NuclideLedger::default();
        let c14 = Nuclide::new("C", 14);
        let half_life = 5730.0 * 365.25 * 86400.0; // 5730 years in seconds

        ledger.deposit(c14.clone(), 1e-12); // 1 picomole
        let a1 = ledger.activity_bq(&c14, half_life);

        ledger.deposit(c14.clone(), 1e-12); // now 2 picomoles
        let a2 = ledger.activity_bq(&c14, half_life);

        assert!((a2 / a1 - 2.0).abs() < 0.01, "activity should double");
    }
}
