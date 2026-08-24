//! THERMO-008: Liquid-liquid equilibrium (LLE).
//!
//! Determines whether a liquid mixture splits into two phases, and if so,
//! what the compositions are. Uses activity coefficients to detect
//! Gibbs energy minima.

/// Result of a liquid-liquid stability check.
#[derive(Debug, Clone, PartialEq)]
pub enum LleResult {
    /// Single liquid phase — no split.
    SinglePhase,
    /// Two liquid phases with the given compositions.
    TwoPhase {
        /// Mole fraction of component 1 in phase α.
        x1_alpha: f64,
        /// Mole fraction of component 1 in phase β.
        x1_beta: f64,
        /// Fraction of total moles in phase α.
        phase_fraction_alpha: f64,
    },
}

/// Check liquid-liquid miscibility for a binary system.
///
/// Stability first: scan the Gibbs energy of mixing
/// g = x₁ ln(x₁γ₁) + x₂ ln(x₂γ₂) for a concave (spinodal) interval —
/// no concavity, no split. Then the binodal properly: the tie line is
/// where *both* components have equal activity in the two phases,
///
/// ```text
/// x_α γ₁(x_α) = x_β γ₁(x_β)   and   (1−x_α) γ₂(x_α) = (1−x_β) γ₂(x_β)
/// ```
///
/// solved by nested bisection — for a trial x_α on the left stable
/// branch, find the x_β on the right branch matching component 1's
/// activity, then drive component 2's mismatch to zero. The first
/// version walked a fixed ±0.005 step on the two equations
/// alternately and stalled a quarter of the composition axis away
/// from the answer on the water–hexane pair; equal activities are a
/// pair of equations and get solved as one.
///
/// `gammas_fn` returns (γ₁, γ₂) at a given x₁.
pub fn lle_binary<F>(z1: f64, gammas_fn: &mut F) -> LleResult
where
    F: FnMut(f64) -> (f64, f64),
{
    if z1 <= 0.0 || z1 >= 1.0 {
        return LleResult::SinglePhase;
    }

    let g_mix = |x1: f64, gammas: &mut F| -> f64 {
        let x2 = 1.0 - x1;
        let (g1, g2) = gammas(x1);
        let t1 = if x1 > 1e-15 {
            x1 * (x1 * g1).max(1e-30).ln()
        } else {
            0.0
        };
        let t2 = if x2 > 1e-15 {
            x2 * (x2 * g2).max(1e-30).ln()
        } else {
            0.0
        };
        t1 + t2
    };

    // Spinodal interval from the concavity scan.
    let n = 400usize;
    let dx = 1.0 / n as f64;
    let (mut s_lo, mut s_hi) = (None, None);
    let mut g_prev = g_mix(dx, gammas_fn);
    let mut g_curr = g_mix(2.0 * dx, gammas_fn);
    for i in 3..n {
        let x = i as f64 * dx;
        let g_next = g_mix(x, gammas_fn);
        let d2g = (g_next - 2.0 * g_curr + g_prev) / (dx * dx);
        if d2g < -1e-10 {
            if s_lo.is_none() {
                s_lo = Some(x - 2.0 * dx);
            }
            s_hi = Some(x);
        }
        g_prev = g_curr;
        g_curr = g_next;
    }
    if std::env::var("KERO_LLE").is_ok() {
        eprintln!("  lle: spinodal {s_lo:?}..{s_hi:?}");
    }
    let (Some(s_lo), Some(s_hi)) = (s_lo, s_hi) else {
        return LleResult::SinglePhase;
    };

    // Log-activities, for conditioning where γ spans decades.
    let ln_a1 = |x: f64, g: &mut F| -> f64 {
        let (g1, _) = g(x);
        (x * g1).max(1e-300).ln()
    };
    let ln_a2 = |x: f64, g: &mut F| -> f64 {
        let (_, g2) = g(x);
        ((1.0 - x) * g2).max(1e-300).ln()
    };

    const EPS: f64 = 1e-9;
    // For a trial x_α on the left stable branch, the x_β on the right
    // branch with matching component-1 activity (ln a₁ rises with x on a
    // stable branch, so this bisection is well-posed); None when the
    // right branch never reaches that activity.
    let beta_for = |x_alpha: f64, g: &mut F| -> Option<f64> {
        let target = ln_a1(x_alpha, g);
        let (mut lo, mut hi) = (s_hi, 1.0 - EPS);
        let f_lo = ln_a1(lo, g) - target;
        let f_hi = ln_a1(hi, g) - target;
        // Endpoint tolerance: for a violently asymmetric pair the
        // feasible window is microns wide and its edges land on the
        // branch ends to within float noise — a target that matches an
        // endpoint to 1e-9 in log-activity *is* that endpoint.
        if f_lo.abs() <= 1e-9 {
            return Some(lo);
        }
        if f_hi.abs() <= 1e-9 {
            return Some(hi);
        }
        if f_lo.signum() == f_hi.signum() {
            return None;
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if (ln_a1(mid, g) - target).signum() == f_lo.signum() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some(0.5 * (lo + hi))
    };

    // Component 2's mismatch along the x_α axis; drive it to zero.
    let residual = |x_alpha: f64, g: &mut F| -> Option<f64> {
        let x_beta = beta_for(x_alpha, g)?;
        Some(ln_a2(x_alpha, g) - ln_a2(x_beta, g))
    };

    // The x_α bracket starts where the left branch's component-1
    // activity first reaches the right branch's minimum — below that
    // point no tie line exists to test, and the bisection would be
    // asked for a β that is not there.
    let a1_floor = ln_a1(s_hi, gammas_fn);
    let x_min = if ln_a1(EPS, gammas_fn) >= a1_floor {
        EPS
    } else {
        let (mut l, mut h) = (EPS, s_lo);
        for _ in 0..80 {
            let m = 0.5 * (l + h);
            if ln_a1(m, gammas_fn) < a1_floor {
                l = m;
            } else {
                h = m;
            }
        }
        h
    };
    // And it ends where the left branch's activity exceeds the right
    // branch's ceiling — near the spinodal the metastable branch
    // overshoots anything a β phase can match.
    let a1_ceil = ln_a1(1.0 - EPS, gammas_fn);
    let x_max = if ln_a1(s_lo, gammas_fn) <= a1_ceil {
        s_lo
    } else {
        let (mut l, mut h) = (x_min, s_lo);
        for _ in 0..80 {
            let m = 0.5 * (l + h);
            if ln_a1(m, gammas_fn) > a1_ceil {
                h = m;
            } else {
                l = m;
            }
        }
        l
    };
    let (mut lo, mut hi) = (x_min, x_max);
    let (r_lo_opt, r_hi_opt) = (residual(lo, gammas_fn), residual(hi, gammas_fn));
    if std::env::var("KERO_LLE").is_ok() {
        eprintln!("  lle: x_min={x_min:.6} x_max={x_max:.6} r_lo={r_lo_opt:?} r_hi={r_hi_opt:?}");
    }
    let (Some(r_lo), Some(r_hi)) = (r_lo_opt, r_hi_opt) else {
        return LleResult::SinglePhase;
    };
    if r_lo.signum() == r_hi.signum() {
        return LleResult::SinglePhase;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let Some(r_mid) = residual(mid, gammas_fn) else {
            return LleResult::SinglePhase;
        };
        if r_mid.signum() == r_lo.signum() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let x_alpha = 0.5 * (lo + hi);
    let Some(x_beta) = beta_for(x_alpha, gammas_fn) else {
        return LleResult::SinglePhase;
    };

    // A feed outside the tie line is a stable single phase.
    if z1 <= x_alpha || z1 >= x_beta {
        return LleResult::SinglePhase;
    }
    let denom = x_beta - x_alpha;
    let phase_fraction = if denom.abs() > 1e-10 {
        ((x_beta - z1) / denom).clamp(0.0, 1.0)
    } else {
        return LleResult::SinglePhase;
    };

    LleResult::TwoPhase {
        x1_alpha: x_alpha,
        x1_beta: x_beta,
        phase_fraction_alpha: phase_fraction,
    }
}

// ── CAP-20: LLE from UNIFAC, reaching for the bench ────────────────

/// Binary LLE with γ from full UNIFAC at a given temperature.
///
/// Component 1 is `groups_a`; `z1` is its overall mole fraction.
pub fn binary_lle_unifac(
    groups_a: &crate::unifac::GroupDecomposition,
    groups_b: &crate::unifac::GroupDecomposition,
    z1: f64,
    t_kelvin: f64,
) -> LleResult {
    let table = crate::unifac::approved_table();
    lle_binary(z1, &mut |x1| {
        let g = crate::unifac::activity_coefficients(
            &table,
            &[(groups_a.clone(), x1), (groups_b.clone(), 1.0 - x1)],
            t_kelvin,
        );
        (g[0], g[1])
    })
}

/// The water–hexane binary: the school's immiscible pair, computed.
///
/// Hexane is 2×CH3 + 4×CH2 — pure main-group 1, whose interaction with
/// H2O (a₁₂ = 1318 K, a₂₁ = 300 K, Fredenslund 1975) is what makes oil
/// and water demix. Stated honesty: UNIFAC-VLE parameters are known to
/// *underestimate* alkane–water γ∞ by orders of magnitude, so the
/// computed mutual solubilities here are upper bounds — the split
/// itself, and which layer is which, are robust; the trace
/// concentrations are not quantitative claims.
pub fn water_hexane_lle(z_hexane: f64, t_kelvin: f64) -> LleResult {
    let mut hexane = crate::unifac::GroupDecomposition::new();
    hexane.insert(1, 2); // CH3 × 2
    hexane.insert(2, 4); // CH2 × 4
    let mut water = crate::unifac::GroupDecomposition::new();
    water.insert(16, 1); // H2O
    binary_lle_unifac(&hexane, &water, z_hexane, t_kelvin)
}

/// Ethanol–water for the negative control: miscible in all proportions,
/// and the same machinery must say so.
pub fn water_ethanol_lle(z_ethanol: f64, t_kelvin: f64) -> LleResult {
    let mut ethanol = crate::unifac::GroupDecomposition::new();
    ethanol.insert(1, 1);
    ethanol.insert(2, 1);
    ethanol.insert(14, 1);
    let mut water = crate::unifac::GroupDecomposition::new();
    water.insert(16, 1);
    binary_lle_unifac(&ethanol, &water, z_ethanol, t_kelvin)
}

/// Infinite-dilution activity coefficient of a solute in a solvent, from
/// full UNIFAC at the given temperature — the γ∞ whose ratio between two
/// immiscible solvents is the partition coefficient a separating funnel
/// runs on.
pub fn infinite_dilution_gamma(
    solute: &crate::unifac::GroupDecomposition,
    solvent: &crate::unifac::GroupDecomposition,
    t_kelvin: f64,
) -> f64 {
    let table = crate::unifac::approved_table();
    let g = crate::unifac::activity_coefficients(
        &table,
        &[(solute.clone(), 1e-9), (solvent.clone(), 1.0 - 1e-9)],
        t_kelvin,
    );
    g[0]
}

/// How a neutral solute splits between two liquid layers at equilibrium:
/// the mole-fraction ratio is set by equal activity,
/// x_low γ∞_low = x_up γ∞_up, so with layer sizes N_low and N_up the
/// mole *amounts* split as
///
/// ```text
/// n_low / n_up = (γ∞_up / γ∞_low) · (N_low / N_up)
/// ```
///
/// Returns the fraction of the solute in the *lower* layer. Curated
/// solutes only — a solute enters when its UNIFAC decomposition exists.
pub fn partition_fraction_lower(
    solute: &crate::unifac::GroupDecomposition,
    lower_solvent: &crate::unifac::GroupDecomposition,
    upper_solvent: &crate::unifac::GroupDecomposition,
    lower_moles: f64,
    upper_moles: f64,
    t_kelvin: f64,
) -> f64 {
    let g_low = infinite_dilution_gamma(solute, lower_solvent, t_kelvin);
    let g_up = infinite_dilution_gamma(solute, upper_solvent, t_kelvin);
    let ratio = (g_up / g_low) * (lower_moles / upper_moles.max(1e-12));
    (ratio / (1.0 + ratio)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_mixture_no_split() {
        // Ideal solution (γ=1): always miscible
        let result = lle_binary(0.5, &mut |_x| (1.0, 1.0));
        assert_eq!(result, LleResult::SinglePhase);
    }

    #[test]
    fn highly_nonideal_mixture_splits() {
        // Margules one-parameter with A=3 (strongly non-ideal)
        // γ₁ = exp(A·x₂²), γ₂ = exp(A·x₁²)
        let a = 3.0;
        let result = lle_binary(0.5, &mut |x1| {
            let x2 = 1.0 - x1;
            ((a * x2 * x2).exp(), (a * x1 * x1).exp())
        });
        match result {
            LleResult::TwoPhase {
                x1_alpha, x1_beta, ..
            } => {
                assert!(
                    x1_alpha < 0.5 && x1_beta > 0.5,
                    "α={:.3}, β={:.3}",
                    x1_alpha,
                    x1_beta
                );
            }
            LleResult::SinglePhase => panic!("should split at A=3"),
        }
    }

    #[test]
    fn weakly_nonideal_no_split() {
        // Margules A=1 — not enough to cause splitting
        let a = 1.0;
        let result = lle_binary(0.5, &mut |x1| {
            let x2 = 1.0 - x1;
            ((a * x2 * x2).exp(), (a * x1 * x1).exp())
        });
        assert_eq!(result, LleResult::SinglePhase);
    }
}
