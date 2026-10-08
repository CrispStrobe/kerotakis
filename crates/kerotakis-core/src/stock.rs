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

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::amount::{Amount, AmountError};

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
    #[error("stock withdrawal must be finite and nonnegative, got {requested}")]
    InvalidRequest { requested: f64 },
    #[error("cannot represent withdrawal of {requested} {unit} from {remaining} {unit}")]
    Precision {
        requested: f64,
        remaining: f64,
        unit: StockUnit,
    },
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

/// The authoritative balance; the public StockAmount is only its scalar view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnedStockAmount {
    amount: Amount,
    unit: StockUnit,
}

impl OwnedStockAmount {
    fn projected(self) -> StockAmount {
        StockAmount {
            amount: self.amount.to_f64(),
            unit: self.unit,
        }
    }
}

/// Finite bottles, by shelf key. An absent key is an unlimited supply.
///
/// The default retains the conservative scalar debit acceptance policy.
/// Compensated accounting is deliberately opt-in; both modes own their
/// balances as Amount, never as a scalar plus an independent residual map.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StockLedger {
    bottles: BTreeMap<String, OwnedStockAmount>,
    compensated: bool,
}

impl StockLedger {
    /// Explicitly opt into compensated bottle balances and strict overdraw
    /// refusal. No ordinary script or legacy snapshot switches modes.
    pub fn compensated() -> Self {
        Self {
            bottles: BTreeMap::new(),
            compensated: true,
        }
    }

    pub fn is_compensated(&self) -> bool {
        self.compensated
    }

    pub fn is_empty(&self) -> bool {
        self.bottles.is_empty()
    }

    /// Empty opted-in ledgers retain their mode in a Bench snapshot.
    pub fn is_serialization_empty(&self) -> bool {
        self.bottles.is_empty() && !self.compensated
    }

    /// Replace a bottle. Invalid or negative input retains the established
    /// stock API policy of creating an empty bottle rather than a debt.
    pub fn stock(&mut self, key: &str, amount: f64, unit: StockUnit) {
        let amount = Amount::new(amount).unwrap_or_else(|_| Amount::new(0.0).unwrap());
        self.bottles
            .insert(key.to_string(), OwnedStockAmount { amount, unit });
    }

    pub fn unlimit(&mut self, key: &str) {
        self.bottles.remove(key);
    }

    /// Rounded display value. In compensated mode a debit can change the
    /// authoritative low component while leaving this projection unchanged.
    pub fn remaining(&self, key: &str) -> Option<StockAmount> {
        self.bottles
            .get(key)
            .copied()
            .map(OwnedStockAmount::projected)
    }

    /// Complete authoritative amount, including its signed low component.
    pub fn remaining_exact(&self, key: &str) -> Option<Amount> {
        self.bottles.get(key).map(|bottle| bottle.amount)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, StockAmount)> + '_ {
        self.bottles
            .iter()
            .map(|(key, amount)| (key.as_str(), amount.projected()))
    }

    /// Debit atomically. Default mode retains the existing scalar accuracy
    /// certificate and relative final-bit exhaustion allowance. Compensated
    /// mode instead certifies the change in both authoritative components and
    /// refuses genuine overdraw without that allowance.
    pub fn draw(&mut self, key: &str, amount: f64) -> Result<(), StockRefusal> {
        if !amount.is_finite() || amount < 0.0 {
            return Err(StockRefusal::InvalidRequest { requested: amount });
        }
        if amount == 0.0 {
            return Ok(());
        }
        let Some(bottle) = self.bottles.get_mut(key) else {
            return Ok(());
        };
        let before = bottle.amount;
        let projected = before.to_f64();
        let exhausted = || StockRefusal::Exhausted {
            requested: amount,
            remaining: projected,
            unit: bottle.unit,
        };
        let precision = || StockRefusal::Precision {
            requested: amount,
            remaining: projected,
            unit: bottle.unit,
        };
        if self.compensated {
            let requested = Amount::new(amount).map_err(|_| precision())?;
            let after = before.checked_sub(requested).map_err(|error| {
                if error == AmountError::Negative {
                    exhausted()
                } else {
                    precision()
                }
            })?;
            let debit = before.checked_sub(after).map_err(|_| precision())?;
            if after == before || (debit.to_f64() / amount - 1.0).abs() > 1e-8 {
                return Err(precision());
            }
            bottle.amount = after;
        } else {
            // Conservative entries are scalar by construction and legacy
            // loading. No serialization projection discards a low component.
            if projected == 0.0 || (amount > projected && (amount / projected - 1.0).abs() > 1e-8) {
                return Err(exhausted());
            }
            let after = (projected - amount).max(0.0);
            if !after.is_finite() || ((projected - after) / amount - 1.0).abs() > 1e-8 {
                return Err(precision());
            }
            bottle.amount = Amount::new(after).map_err(|_| precision())?;
        }
        Ok(())
    }
}

impl Serialize for StockLedger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.compensated {
            #[derive(Serialize)]
            struct Versioned<'a> {
                schema: &'static str,
                bottles: &'a BTreeMap<String, OwnedStockAmount>,
            }
            Versioned {
                schema: "kerotakis-stock/2",
                bottles: &self.bottles,
            }
            .serialize(serializer)
        } else {
            let legacy: BTreeMap<&str, StockAmount> = self.entries().collect();
            legacy.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for StockLedger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Versioned {
            schema: String,
            bottles: BTreeMap<String, OwnedStockAmount>,
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Versioned(Versioned),
            Legacy(BTreeMap<String, StockAmount>),
        }
        match Stored::deserialize(deserializer)? {
            Stored::Versioned(value) => {
                if value.schema != "kerotakis-stock/2" {
                    return Err(serde::de::Error::custom("unsupported stock ledger schema"));
                }
                Ok(Self {
                    bottles: value.bottles,
                    compensated: true,
                })
            }
            Stored::Legacy(value) => {
                let bottles = value
                    .into_iter()
                    .map(|(key, entry)| {
                        Amount::new(entry.amount).map(|amount| {
                            (
                                key,
                                OwnedStockAmount {
                                    amount,
                                    unit: entry.unit,
                                },
                            )
                        })
                    })
                    .collect::<Result<_, _>>()
                    .map_err(serde::de::Error::custom)?;
                Ok(Self {
                    bottles,
                    compensated: false,
                })
            }
        }
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
        } = refusal
        else {
            panic!("expected exhaustion, got {refusal:?}");
        };
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
