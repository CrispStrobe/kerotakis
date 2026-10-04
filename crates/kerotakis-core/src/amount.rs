//! Standalone, finite nonnegative compensated amounts.
//!
//! `Amount` stores a normalized sum of two binary64 components. Arithmetic
//! preserves a small correction that ordinary scalar updates can discard.
//! It is bounded compensated arithmetic, not arbitrary precision: a third
//! component can round away, and underflow can erase a product correction.
//! No existing vessel, stock, or `Moles` inventory uses this type yet.

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// A normalized two-component nonnegative quantity.
///
/// The high component is the scalar projection; the signed low component
/// retains a correction. Neither field is mutable outside this module.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Amount {
    hi: f64,
    lo: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountError {
    NonFinite,
    Negative,
    Overflow,
    Underflow,
    QuantizedScale,
    NonCanonical,
}

impl fmt::Display for AmountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NonFinite => "amount inputs must be finite",
            Self::Negative => "amount cannot be negative or overdrawn",
            Self::Overflow => "amount arithmetic overflowed",
            Self::Underflow => "a positive scaled amount rounded to zero",
            Self::QuantizedScale => "scaled amount exceeds the supported relative rounding error",
            Self::NonCanonical => "persisted amount must contain a normalized high/low pair",
        })
    }
}

impl std::error::Error for AmountError {}

/// Knuth's error-free sum of two finite binary64 values when intermediate
/// arithmetic stays finite. A finite result is required by each caller.
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let sum = a + b;
    let virtual_b = sum - a;
    let virtual_a = sum - virtual_b;
    let error = (a - virtual_a) + (b - virtual_b);
    (sum, error)
}

impl Amount {
    pub fn new(value: f64) -> Result<Self, AmountError> {
        if !value.is_finite() {
            return Err(AmountError::NonFinite);
        }
        if value < 0.0 {
            return Err(AmountError::Negative);
        }
        Ok(Self {
            hi: if value == 0.0 { 0.0 } else { value },
            lo: 0.0,
        })
    }

    /// Normalize a finite pair. The inputs may have either sign provided
    /// their sum is nonnegative. This constructor deliberately differs from
    /// persistence, which requires an already canonical pair.
    pub fn from_parts(hi: f64, lo: f64) -> Result<Self, AmountError> {
        if !hi.is_finite() || !lo.is_finite() {
            return Err(AmountError::NonFinite);
        }
        let (hi, lo) = two_sum(hi, lo);
        if !hi.is_finite() || !lo.is_finite() {
            return Err(AmountError::Overflow);
        }
        if hi < 0.0 || (hi == 0.0 && lo < 0.0) {
            return Err(AmountError::Negative);
        }
        Ok(Self {
            hi: if hi == 0.0 { 0.0 } else { hi },
            lo: if lo == 0.0 { 0.0 } else { lo },
        })
    }

    pub fn parts(self) -> (f64, f64) {
        (self.hi, self.lo)
    }

    /// Rounded scalar projection. A nonzero low component can be invisible
    /// here; use the complete Amount for ownership and equality decisions.
    pub fn to_f64(self) -> f64 {
        self.hi + self.lo
    }

    pub fn checked_add(self, other: Self) -> Result<Self, AmountError> {
        self.combine(other.hi, other.lo)
    }

    /// Refuse overdraw rather than clamping a negative balance to zero.
    pub fn checked_sub(self, other: Self) -> Result<Self, AmountError> {
        self.combine(-other.hi, -other.lo)
    }

    /// Two-component expansion addition. Normalization after each high/low
    /// combination avoids relying on FastTwoSum's operand ordering premise.
    /// As with ordinary double-double arithmetic, a third correction can
    /// round away; this does not promise exact sums at arbitrary scales.
    fn combine(self, other_hi: f64, other_lo: f64) -> Result<Self, AmountError> {
        let (high_sum, high_error) = two_sum(self.hi, other_hi);
        if !high_sum.is_finite() || !high_error.is_finite() {
            return Err(AmountError::Overflow);
        }
        let (low_sum, low_error) = two_sum(self.lo, other_lo);
        let (high, low) = two_sum(high_sum, high_error + low_sum);
        Self::from_parts(high, low + low_error)
    }

    /// Scale by a finite nonnegative scalar, retaining the FMA product error.
    ///
    /// A nonzero result that rounds entirely to zero is refused. Coarsely
    /// quantized subnormal products are refused at relative error > 1e-8.
    /// A product correction below the smallest subnormal can still be lost;
    /// the accepted result is not a claim of exact real multiplication.
    pub fn checked_scale(self, factor: f64) -> Result<Self, AmountError> {
        if !factor.is_finite() {
            return Err(AmountError::NonFinite);
        }
        if factor < 0.0 {
            return Err(AmountError::Negative);
        }
        if factor == 0.0 || self.hi == 0.0 {
            return Self::new(0.0);
        }
        let product = self.hi * factor;
        if !product.is_finite() {
            return Err(AmountError::Overflow);
        }
        if product == 0.0 {
            return Err(AmountError::Underflow);
        }
        // Divide by the smaller operand first. Reversing that order can
        // round a subnormal intermediate and hide the product's error.
        // The normalized low part changes the input only at roundoff scale.
        let ratio = product / self.hi.min(factor) / self.hi.max(factor);
        if !ratio.is_finite() || (ratio - 1.0).abs() > 1e-8 {
            return Err(AmountError::QuantizedScale);
        }
        let error = self.hi.mul_add(factor, -product);
        // Corrections can be negative, so they are not standalone Amounts.
        // Combine them directly with the product before nonnegative checking.
        Self::from_parts(product, error + self.lo * factor)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Pair {
            hi: f64,
            lo: f64,
        }
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Scalar(f64),
            Pair(Pair),
        }
        let value = match Stored::deserialize(deserializer)? {
            Stored::Scalar(value) => Self::new(value),
            Stored::Pair(Pair { hi, lo }) => {
                let normalized = Self::from_parts(hi, lo).map_err(serde::de::Error::custom)?;
                if normalized.hi != hi || normalized.lo != lo {
                    Err(AmountError::NonCanonical)
                } else {
                    Ok(normalized)
                }
            }
        };
        value.map_err(serde::de::Error::custom)
    }
}
