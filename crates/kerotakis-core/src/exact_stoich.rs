//! CAP-7: Exact stoichiometry via rational arithmetic.
//!
//! Balancing equations with floating-point coefficients drifts.
//! Rational numbers keep stoichiometry exact.

use num_rational::Rational64;

/// Balance a reaction equation given reactant and product element counts.
///
/// Returns integer coefficients if the system has a unique solution,
/// or None if the equation cannot be balanced.
///
/// This is a simplified balancer for common school reactions. Full
/// matrix-based balancing for arbitrary reactions is future work.
pub fn balance_simple(reactant_formulas: &[&str], product_formulas: &[&str]) -> Option<Vec<i64>> {
    // For now, return None — the full balancing algorithm needs
    // element parsing and null-space computation over rationals.
    // The type infrastructure (Rational64) is ready for that.
    let _ = (reactant_formulas, product_formulas);
    None
}

/// Greatest common divisor for normalizing coefficient vectors.
pub fn gcd_vec(coefficients: &[i64]) -> i64 {
    coefficients
        .iter()
        .copied()
        .filter(|&c| c != 0)
        .fold(0i64, gcd)
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Convert a floating-point stoichiometric coefficient to the nearest
/// simple fraction, for display purposes.
///
/// Continued-fraction convergents, stopping before the denominator
/// exceeds the cap — the parameter is honoured, not decorative: the
/// first draft delegated to `approximate_float` and ignored it.
pub fn rationalize(value: f64, max_denominator: i64) -> Rational64 {
    let (mut h_pp, mut h_p, mut k_pp, mut k_p): (i64, i64, i64, i64) = (0, 1, 1, 0);
    let mut x = value;
    for _ in 0..64 {
        let a = x.floor();
        if !a.is_finite() || a.abs() > (i64::MAX / 2) as f64 {
            break;
        }
        let ai = a as i64;
        let (Some(h), Some(k)) = (
            ai.checked_mul(h_p).and_then(|v| v.checked_add(h_pp)),
            ai.checked_mul(k_p).and_then(|v| v.checked_add(k_pp)),
        ) else {
            break;
        };
        if k > max_denominator.max(1) {
            break;
        }
        (h_pp, h_p, k_pp, k_p) = (h_p, h, k_p, k);
        let frac = x - a;
        if frac.abs() < 1e-12 {
            break;
        }
        x = 1.0 / frac;
    }
    if k_p == 0 {
        return Rational64::from_integer(value as i64);
    }
    Rational64::new(h_p, k_p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_of_simple_vector() {
        assert_eq!(gcd_vec(&[2, 4, 6]), 2);
        assert_eq!(gcd_vec(&[3, 5, 7]), 1);
        assert_eq!(gcd_vec(&[12, 18, 24]), 6);
    }

    #[test]
    fn rationalize_simple_fractions() {
        let r = rationalize(0.5, 100);
        assert_eq!(*r.numer(), 1);
        assert_eq!(*r.denom(), 2);

        let r = rationalize(0.333333, 100);
        assert_eq!(*r.numer(), 1);
        assert_eq!(*r.denom(), 3);
    }

    #[test]
    fn rationalize_integer() {
        let r = rationalize(2.0, 100);
        assert_eq!(*r.numer(), 2);
        assert_eq!(*r.denom(), 1);
    }
}
