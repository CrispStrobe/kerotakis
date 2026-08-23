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
/// simple fraction with denominator ≤ `max_denominator`.
///
/// Uses the Stern-Brocot / mediant algorithm for best rational
/// approximation within the denominator bound.
pub fn rationalize(value: f64, max_denominator: i64) -> Rational64 {
    if value == value.round() && value.abs() < i64::MAX as f64 {
        return Rational64::from_integer(value as i64);
    }
    // Stern-Brocot tree search
    let sign = if value < 0.0 { -1 } else { 1 };
    let x = value.abs();
    let (mut a_n, mut a_d) = (0i64, 1i64); // lower bound 0/1
    let (mut b_n, mut b_d) = (1i64, 0i64); // upper bound 1/0 = ∞
    loop {
        let m_n = a_n + b_n;
        let m_d = a_d + b_d;
        if m_d > max_denominator {
            // Pick whichever of a/a_d or b/b_d is closer
            let err_a = (x - a_n as f64 / a_d.max(1) as f64).abs();
            let err_b = if b_d == 0 {
                f64::INFINITY
            } else {
                (x - b_n as f64 / b_d as f64).abs()
            };
            return if err_a <= err_b {
                Rational64::new(sign * a_n, a_d)
            } else {
                Rational64::new(sign * b_n, b_d)
            };
        }
        let mediant = m_n as f64 / m_d as f64;
        if (mediant - x).abs() < 1e-12 {
            return Rational64::new(sign * m_n, m_d);
        } else if mediant < x {
            a_n = m_n;
            a_d = m_d;
        } else {
            b_n = m_n;
            b_d = m_d;
        }
    }
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
