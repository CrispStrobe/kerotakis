//! BRD-002: the shelf holds finite bottles.
//!
//! A cabinet whose bottles never empty is a cabinet nobody has to think
//! about. Stocking one turns "add 100 mL of vinegar" into a withdrawal
//! against a real inventory, and the fourth withdrawal from a 250 mL
//! bottle is refused out loud rather than quietly succeeding.
//!
//! Two deliberate boundaries:
//!
//! * **A key that is not stocked is not limited.** An empty ledger is the
//!   sandbox every existing script and test already assumes, so the
//!   feature costs nothing until a story, lesson or teacher opts in.
//! * **Stock is counted in the unit the dispense already carries** — moles
//!   for a registry species, the recipe's own basis amount for a named
//!   material. Nothing here invents a mass from a volume; the conversion
//!   that does that lives in the parser, behind a reviewed bulk density,
//!   and by the time an operator reaches the ledger it has already run.
//!
//! The ledger lives on [`crate::Bench`], which is what the protocol's
//! opaque snapshot token serialises — so undo, redo and scrub restore the
//! bottle level with the same round-trip that restores the vessels.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::material;
use crate::species;

/// What a stocked bottle is measured in. Not a general unit system — these
/// are exactly the three quantities a dispensing operator already carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockUnit {
    /// A registry species: `Operator::Add` carries moles.
    Mole,
    /// A mass-basis material recipe: `total_amount` is grams.
    Gram,
    /// A volume-basis material recipe: `total_amount` is millilitres.
    Millilitre,
}

impl StockUnit {
    /// The symbol a reader sees, matching the one the `add` grammar takes.
    pub fn label(self) -> &'static str {
        match self {
            StockUnit::Mole => "mol",
            StockUnit::Gram => "g",
            StockUnit::Millilitre => "mL",
        }
    }
}

impl std::fmt::Display for StockUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// How much of one shelf entry is left, and in what.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StockAmount {
    pub amount: f64,
    pub unit: StockUnit,
}

/// Why a draw against the shelf failed. Typed, because "the bottle is
/// empty" is a fact a caller may want to act on, not a sentence.
#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum StockRefusal {
    #[error("the bottle holds {remaining} {unit}, and {requested} {unit} was asked for")]
    Exhausted {
        requested: f64,
        remaining: f64,
        unit: StockUnit,
    },
}

/// The unit a given shelf key is stocked in, or `None` when the key names
/// nothing the lab can dispense.
///
/// Registry species win, exactly as they do in the `add` grammar: a
/// built-in identity is never shadowed by a recipe.
pub fn stock_unit(key: &str) -> Option<StockUnit> {
    if species::lookup_key(key).is_some() {
        return Some(StockUnit::Mole);
    }
    let recipe = material::lookup(key, None)?;
    Some(match recipe.basis {
        material::MaterialBasis::MassFraction => StockUnit::Gram,
        material::MaterialBasis::MoleFraction => StockUnit::Mole,
        material::MaterialBasis::VolumeFraction => StockUnit::Millilitre,
    })
}

/// Finite bottles, by shelf key. An absent key is an unlimited supply —
/// the sandbox default — not a zero.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StockLedger {
    bottles: BTreeMap<String, StockAmount>,
}

impl StockLedger {
    pub fn is_empty(&self) -> bool {
        self.bottles.is_empty()
    }

    /// Put a finite amount of one key on the shelf, replacing whatever was
    /// there. A negative request is clamped to an empty bottle rather than
    /// creating a debt.
    pub fn stock(&mut self, key: &str, amount: f64, unit: StockUnit) {
        self.bottles.insert(
            key.to_string(),
            StockAmount {
                amount: if amount.is_finite() && amount > 0.0 {
                    amount
                } else {
                    0.0
                },
                unit,
            },
        );
    }

    /// Return this key to an unlimited supply.
    pub fn unlimit(&mut self, key: &str) {
        self.bottles.remove(key);
    }

    /// What is left, or `None` when this key is not tracked at all.
    pub fn remaining(&self, key: &str) -> Option<StockAmount> {
        self.bottles.get(key).copied()
    }

    /// Every tracked bottle, in stable key order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, StockAmount)> + '_ {
        self.bottles
            .iter()
            .map(|(key, amount)| (key.as_str(), *amount))
    }

    /// Take `amount` of `key` off the shelf.
    ///
    /// An untracked key succeeds and changes nothing — an unlimited supply
    /// has nothing to decrement. A tracked key with too little left is
    /// refused whole: no partial pour, because the operator that asked has
    /// one amount and half of it is a different experiment.
    pub fn draw(&mut self, key: &str, amount: f64) -> Result<(), StockRefusal> {
        let Some(bottle) = self.bottles.get_mut(key) else {
            return Ok(());
        };
        // A hair of float slack, so that drawing 100 mL from a bottle
        // stocked at 100 mL is the last dispense rather than a refusal
        // decided by the last bit of a f64.
        let tolerance = 1e-9 * bottle.amount.abs().max(1.0);
        if amount > bottle.amount + tolerance {
            return Err(StockRefusal::Exhausted {
                requested: amount,
                remaining: bottle.amount,
                unit: bottle.unit,
            });
        }
        bottle.amount = (bottle.amount - amount).max(0.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untracked_key_is_an_unlimited_supply() {
        let mut ledger = StockLedger::default();
        assert!(ledger.is_empty());
        assert_eq!(ledger.remaining("NaCl"), None);
        assert!(ledger.draw("NaCl", 1e9).is_ok());
    }

    #[test]
    fn drawing_decrements_and_running_out_is_refused_with_the_numbers() {
        let mut ledger = StockLedger::default();
        ledger.stock("NaCl", 0.5, StockUnit::Mole);
        ledger.draw("NaCl", 0.2).expect("first draw fits");
        assert!((ledger.remaining("NaCl").unwrap().amount - 0.3).abs() < 1e-12);

        let refusal = ledger.draw("NaCl", 0.4).expect_err("0.4 > 0.3 remaining");
        let StockRefusal::Exhausted {
            requested,
            remaining,
            unit,
        } = refusal;
        assert!((requested - 0.4).abs() < 1e-12);
        assert!((remaining - 0.3).abs() < 1e-12);
        assert_eq!(unit, StockUnit::Mole);
        // A refused draw takes nothing: the bottle is exactly as it was.
        assert!((ledger.remaining("NaCl").unwrap().amount - 0.3).abs() < 1e-12);
    }

    #[test]
    fn the_last_exact_dispense_is_not_lost_to_float_slack() {
        let mut ledger = StockLedger::default();
        ledger.stock("water", 100.0, StockUnit::Millilitre);
        ledger.draw("water", 100.0).expect("the whole bottle pours");
        assert_eq!(ledger.remaining("water").unwrap().amount, 0.0);
        assert!(ledger.draw("water", 0.1).is_err());
    }

    #[test]
    fn a_species_is_stocked_in_moles_and_an_unknown_key_has_no_unit() {
        assert_eq!(stock_unit("NaCl"), Some(StockUnit::Mole));
        assert_eq!(stock_unit("definitely-not-a-substance"), None);
    }
}
