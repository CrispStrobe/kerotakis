//! Gibbs energy minimisation over the NASA-9 data: given an element
//! budget, a temperature and a pressure, find the composition that
//! minimises G. This is what makes `heat`, `decompose` and `ignite`
//! computed chemistry rather than curated lookups (PLAN.md, L2g).
//!
//! The formulation is the standard one (Gordon & McBride, NASA RP-1311):
//! minimise Σ nᵢ μᵢ subject to element conservation, with Lagrange
//! multipliers πⱼ per element. For gases
//!
//! ```text
//! μᵢ/RT = G°ᵢ(T)/RT + ln(nᵢ/n) + ln(P/P°)
//! ```
//!
//! and for a pure condensed phase μ_c/RT = G°_c(T)/RT (unit activity).
//! Newton iteration on ln nᵢ, damped, until the corrections vanish.
//!
//! Standard-state pressure is 1 bar, matching the NASA data.

use std::collections::BTreeMap;

use crate::nasa9::{Species, R};

/// Standard-state pressure, bar.
pub const P_STANDARD_BAR: f64 = 1.0;

#[derive(Debug, thiserror::Error)]
pub enum CeaError {
    #[error("no candidate species contain the requested elements")]
    NoSpecies,
    #[error("thermodynamic data unavailable for {0} at {1:.1} K")]
    OutOfRange(String, f64),
    #[error("equilibrium did not converge after {0} iterations")]
    NotConverged(usize),
    #[error("the element budget is empty")]
    EmptyBudget,
}

#[derive(Debug, Clone)]
pub struct Equilibrium {
    /// Species and their amounts, mol, descending; trace amounts dropped.
    pub composition: Vec<(String, f64)>,
    pub temperature: f64,
    pub pressure_bar: f64,
    /// Total enthalpy of the mixture, J.
    pub enthalpy: f64,
    /// Total moles of gas.
    pub gas_moles: f64,
    /// Literature citations of the species that carry the result.
    pub sources: Vec<String>,
}

impl Equilibrium {
    pub fn moles_of(&self, name: &str) -> f64 {
        self.composition
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, m)| *m)
            .unwrap_or(0.0)
    }

    /// Mole fraction in the gas phase.
    pub fn mole_fraction(&self, name: &str, db: &crate::ThermoDb) -> f64 {
        if self.gas_moles <= 0.0 {
            return 0.0;
        }
        match db.get(name) {
            Some(s) if s.is_gas() => self.moles_of(name) / self.gas_moles,
            _ => 0.0,
        }
    }
}

/// Amounts below this are numerically zero; the same floor is used when
/// evaluating chemical potentials, so a species' μ never disagrees with
/// its own amount.
const TRACE: f64 = 1e-18;

/// How far out of element balance a composition may be and still count as
/// solved, relative to each element's own budget. Atoms are conserved
/// exactly in nature, so this is a numerical tolerance and nothing more:
/// anything looser is not a rounding error but a wrong answer.
const BALANCE_TOL: f64 = 1e-9;

/// The worst relative element-balance violation in a composition.
///
/// `budget` says how many moles of each element went in; a valid answer
/// accounts for every one of them. Returning this rather than trusting the
/// formulation is the difference between claiming conservation and
/// checking it.
fn balance_residual(
    pool: &[&Species],
    n: &[f64],
    elements: &[String],
    budget: &BTreeMap<String, f64>,
) -> f64 {
    let mut worst: f64 = 0.0;
    for (j, el) in elements.iter().enumerate() {
        let target = budget.get(el).copied().unwrap_or(0.0);
        let have: f64 = pool
            .iter()
            .zip(n)
            .map(|(s, m)| s.composition.get(&elements[j]).copied().unwrap_or(0.0) * m)
            .sum();
        let _ = el;
        worst = worst.max((have - target).abs() / target.max(1e-12));
    }
    worst
}

/// Solve the (T, P) equilibrium problem.
///
/// `budget` maps element symbol → total moles of that element; the result
/// conserves it exactly. `candidates` is the species pool the mixture may
/// draw on — the caller decides what chemistry is in scope, which keeps the
/// honesty boundary explicit rather than silently searching all 2000
/// species.
pub fn equilibrate_tp(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    t: f64,
    pressure_bar: f64,
) -> Result<Equilibrium, CeaError> {
    if budget.is_empty() || budget.values().all(|v| *v <= 0.0) {
        return Err(CeaError::EmptyBudget);
    }
    let elements: Vec<String> = budget
        .iter()
        .filter(|(_, v)| **v > 0.0)
        .map(|(k, _)| k.clone())
        .collect();

    // Only species entirely composed of budgeted elements can appear.
    let pool: Vec<&Species> = candidates
        .iter()
        .copied()
        .filter(|s| {
            !s.composition.is_empty()
                && s.composition
                    .keys()
                    .all(|el| elements.iter().any(|e| e == el))
        })
        .collect();
    if pool.is_empty() {
        return Err(CeaError::NoSpecies);
    }

    // Standard-state chemical potentials, μ°/RT.
    let mut mu0 = Vec::with_capacity(pool.len());
    for s in &pool {
        let g = s
            .g(t)
            .ok_or_else(|| CeaError::OutOfRange(s.name.clone(), t))?;
        mu0.push(g / (R * t));
    }

    let gas: Vec<usize> = (0..pool.len()).filter(|i| pool[*i].is_gas()).collect();
    let cond: Vec<usize> = (0..pool.len()).filter(|i| !pool[*i].is_gas()).collect();

    let a = |i: usize, j: usize| -> f64 {
        pool[i]
            .composition
            .get(&elements[j])
            .copied()
            .unwrap_or(0.0)
    };
    let total_budget: f64 = budget.values().sum();

    // Initial guess: gases share the budget equally, condensed start empty.
    let mut n = vec![0.0f64; pool.len()];
    let start = (total_budget / gas.len().max(1) as f64).max(1e-6);
    for &i in &gas {
        n[i] = start * 0.1;
    }
    let mut n_total: f64 = gas.iter().map(|&i| n[i]).sum();
    let ln_p = (pressure_bar / P_STANDARD_BAR).ln();

    // Which condensed phases are currently in the mixture.
    //
    // An element that no gas species can carry (calcium in a limestone
    // kiln, say) must enter through a condensed phase from the start —
    // otherwise its element-balance row is all zeros and the very first
    // linear solve is singular.
    let mut active_cond: Vec<usize> = Vec::new();
    for (j, el) in elements.iter().enumerate() {
        let carried_by_gas = gas.iter().any(|&i| a(i, j) > 0.0);
        if carried_by_gas {
            continue;
        }
        // Pick the most stable condensed carrier (lowest μ° per atom).
        let carrier = cond
            .iter()
            .copied()
            .filter(|&c| a(c, j) > 0.0)
            .min_by(|&x, &y| (mu0[x] / a(x, j)).total_cmp(&(mu0[y] / a(y, j))));
        match carrier {
            Some(c) if !active_cond.contains(&c) => {
                active_cond.push(c);
                n[c] = budget.get(el).copied().unwrap_or(0.0) / a(c, j).max(1.0);
            }
            Some(_) => {}
            None => return Err(CeaError::NoSpecies),
        }
    }

    // How often each condensed phase has been admitted. A phase that is
    // admitted, driven out, and admitted again is oscillating rather than
    // converging; capping the retries keeps that from spinning the full 400
    // iterations.
    let mut admissions = vec![0u8; pool.len()];

    // Which elements a gas can carry at all. An element with no gaseous
    // form — calcium in a limestone kiln — lives entirely in the condensed
    // phases, so the last solid holding it may not leave: its balance row
    // would go all-zero and the next linear solve would be singular. This
    // is the same guard the initial guess above applies, enforced for the
    // rest of the iteration too.
    let gas_carries: Vec<bool> = (0..elements.len())
        .map(|j| gas.iter().any(|&i| a(i, j) > 0.0))
        .collect();
    let is_sole_carrier = |c: usize, active: &[usize]| -> bool {
        (0..elements.len()).any(|j| {
            a(c, j) > 0.0 && !gas_carries[j] && !active.iter().any(|&o| o != c && a(o, j) > 0.0)
        })
    };

    for iteration in 0..400 {
        let dim = elements.len() + active_cond.len() + 1;
        let mut m = vec![vec![0.0f64; dim + 1]; dim];

        // μᵢ/RT for the current composition.
        let mu = |i: usize, n: &[f64], n_total: f64| -> f64 {
            if pool[i].is_gas() {
                let ni = n[i].max(TRACE);
                mu0[i] + (ni / n_total.max(TRACE)).ln() + ln_p
            } else {
                mu0[i]
            }
        };

        // Element-balance rows.
        for (j, _) in elements.iter().enumerate() {
            for (k, _) in elements.iter().enumerate() {
                m[j][k] = gas.iter().map(|&i| a(i, j) * a(i, k) * n[i]).sum();
            }
            for (c_idx, &c) in active_cond.iter().enumerate() {
                m[j][elements.len() + c_idx] = a(c, j);
            }
            m[j][dim - 1] = gas.iter().map(|&i| a(i, j) * n[i]).sum();
            let b_current: f64 = (0..pool.len()).map(|i| a(i, j) * n[i]).sum();
            let target = budget.get(&elements[j]).copied().unwrap_or(0.0);
            m[j][dim] = target - b_current
                + gas
                    .iter()
                    .map(|&i| a(i, j) * n[i] * mu(i, &n, n_total))
                    .sum::<f64>();
        }

        // One row per active condensed phase: Σⱼ a_cj πⱼ = μ_c/RT.
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let row = elements.len() + c_idx;
            for (j, _) in elements.iter().enumerate() {
                m[row][j] = a(c, j);
            }
            m[row][dim] = mu(c, &n, n_total);
        }

        // Total-moles row.
        let last = dim - 1;
        for (j, _) in elements.iter().enumerate() {
            m[last][j] = gas.iter().map(|&i| a(i, j) * n[i]).sum();
        }
        let sum_gas: f64 = gas.iter().map(|&i| n[i]).sum();
        m[last][last] = sum_gas - n_total;
        m[last][dim] =
            n_total - sum_gas + gas.iter().map(|&i| n[i] * mu(i, &n, n_total)).sum::<f64>();

        let Some(sol) = solve(&mut m) else {
            // Singular. This happens when one condensed phase is the sole
            // repository of every element and the gas phase has collapsed:
            // the element rows then differ only by a stoichiometric factor
            // in that phase's single column, so the multipliers are
            // underdetermined. The composition is not in doubt there, but
            // this formulation cannot produce it, and saying so is better
            // than returning whichever answer the arithmetic fell into.
            return Err(CeaError::NotConverged(iteration));
        };
        let pi: Vec<f64> = sol[..elements.len()].to_vec();
        let d_ln_n = sol[dim - 1];

        // Corrections to each gas species.
        let mut d_ln: Vec<f64> = vec![0.0; pool.len()];
        for &i in &gas {
            let sum: f64 = (0..elements.len()).map(|j| a(i, j) * pi[j]).sum();
            d_ln[i] = sum + d_ln_n - mu(i, &n, n_total);
        }

        // Damping (RP-1311 §3.3): keep steps sane and amounts positive.
        let mut lambda: f64 = 1.0;
        for &i in &gas {
            if d_ln[i] > 0.0 {
                lambda = lambda.min(2.0 / d_ln[i].abs().max(2.0));
            }
        }
        // A condensed phase being driven out must not be allowed to freeze
        // the whole step. Limiting λ so the phase can only shrink by 90% is
        // right while it is genuinely present, but a phase the solution
        // wants *gone* demands λ→0, which stalls the iteration — and a
        // stalled iteration used to be misread as a converged one. Below a
        // tenth of a percent of a step, remove the phase instead.
        let mut forced_drop: Vec<usize> = Vec::new();
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let dn = sol[elements.len() + c_idx];
            if dn < 0.0 && n[c] > 0.0 {
                let limit = (0.9 * n[c] / -dn).min(1.0);
                if limit < 1e-3 && !is_sole_carrier(c, &active_cond) {
                    forced_drop.push(c);
                } else {
                    lambda = lambda.min(limit);
                }
            }
        }

        // Convergence is measured on each species' *contribution*, not on
        // its log step (RP-1311 eq. 3.14): a trace radical may still be
        // doubling every iteration while the mixture is settled, and it
        // must not hold the solution hostage.
        let mut max_change: f64 = 0.0;
        for &i in &gas {
            let step = lambda * d_ln[i];
            max_change = max_change.max(step.abs() * n[i] / n_total.max(TRACE));
            n[i] = (n[i].max(TRACE).ln() + step).exp().max(TRACE);
        }
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let dn = lambda * sol[elements.len() + c_idx];
            max_change = max_change.max((dn / n_total.max(TRACE)).abs());
            n[c] = (n[c] + dn).max(0.0);
        }
        let step_n = lambda * d_ln_n;
        n_total = (n_total.max(TRACE).ln() + step_n).exp();
        max_change = max_change.max(step_n.abs());

        for c in forced_drop {
            n[c] = 0.0;
        }
        // A sole carrier that the step drove to zero is floored back to a
        // trace so it stays in the basis. The amount is far below the
        // balance tolerance, and Newton solves condensed phases for Δn
        // directly, so it climbs back to its true value in one step.
        for &c in &active_cond {
            if n[c] <= TRACE && is_sole_carrier(c, &active_cond) {
                n[c] = (total_budget * 1e-14).max(1e-16);
            }
        }

        // How badly the element budget is still violated, relative to each
        // element's own total. This is the constraint the entire
        // formulation exists to satisfy, and it is *not* implied by a small
        // Newton step: damping can make steps vanish while the composition
        // sits arbitrarily far from balance. Testing convergence on the
        // step alone silently returned compositions that created matter —
        // heating chalk produced twice the carbon it started with.
        let residual = balance_residual(&pool, &n, &elements, budget);

        // Phase management: drop an exhausted condensed phase; admit one
        // whose chemical potential says it should exist.
        active_cond.retain(|&c| n[c] > TRACE);
        if max_change < 1e-8 && residual < BALANCE_TOL {
            let mut admitted = false;
            for &c in &cond {
                if active_cond.contains(&c) || admissions[c] >= 3 {
                    continue;
                }
                // π is only meaningful once the balance holds, which is why
                // this test lives behind the residual check: driving a phase
                // in on the strength of Lagrange multipliers from an
                // unconverged system is how solid carbon used to appear in
                // an oxidising atmosphere.
                let drive: f64 =
                    (0..elements.len()).map(|j| a(c, j) * pi[j]).sum::<f64>() - mu(c, &n, n_total);
                if drive > 1e-8 {
                    active_cond.push(c);
                    admissions[c] += 1;
                    // Seed a trace, not a lump. The old seed of 1% of the
                    // whole budget injected matter that the next steps then
                    // had to find a home for; Newton solves for Δn on
                    // condensed phases directly, so a trace grows to its
                    // true amount in a few iterations without ever putting
                    // the balance in debt.
                    n[c] = (total_budget * 1e-9).max(1e-14);
                    admitted = true;
                    break;
                }
            }
            if !admitted {
                return Ok(finish(&pool, &n, t, pressure_bar));
            }
        }
    }
    Err(CeaError::NotConverged(400))
}

fn finish(pool: &[&Species], n: &[f64], t: f64, pressure_bar: f64) -> Equilibrium {
    let mut composition: Vec<(String, f64)> = pool
        .iter()
        .zip(n)
        .filter(|(_, m)| **m > 1e-10)
        .map(|(s, m)| (s.name.clone(), *m))
        .collect();
    composition.sort_by(|a, b| b.1.total_cmp(&a.1));
    let enthalpy: f64 = pool
        .iter()
        .zip(n)
        .filter_map(|(s, m)| s.h(t).map(|h| h * m))
        .sum();
    let gas_moles: f64 = pool
        .iter()
        .zip(n)
        .filter(|(s, _)| s.is_gas())
        .map(|(_, m)| *m)
        .sum();
    let mut sources: Vec<String> = pool
        .iter()
        .zip(n)
        .filter(|(_, m)| **m > 1e-6)
        .filter(|(s, _)| !s.reference.is_empty())
        .map(|(s, _)| format!("{}: {}", s.name, s.reference))
        .collect();
    sources.truncate(5);
    Equilibrium {
        composition,
        temperature: t,
        pressure_bar,
        enthalpy,
        gas_moles,
        sources,
    }
}

/// Find the adiabatic temperature: the temperature at which the products'
/// enthalpy equals the reactants' — the flame temperature of a burning
/// mixture, computed rather than tabulated.
pub fn equilibrate_hp(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    enthalpy: f64,
    pressure_bar: f64,
) -> Result<Equilibrium, CeaError> {
    // Bisection on T: H(T) rises monotonically, so this is robust where a
    // Newton step on a stiff flame problem is not.
    let (mut lo, mut hi) = (250.0f64, 6000.0f64);
    let mut last = equilibrate_tp(budget, candidates, lo, pressure_bar)?;
    if last.enthalpy > enthalpy {
        return Ok(last); // colder than the data supports; honest floor
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        last = equilibrate_tp(budget, candidates, mid, pressure_bar)?;
        if last.enthalpy < enthalpy {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 0.5 {
            break;
        }
    }
    Ok(last)
}

/// Gauss-Jordan with partial pivoting on a small dense augmented matrix.
fn solve(m: &mut [Vec<f64>]) -> Option<Vec<f64>> {
    let n = m.len();
    for col in 0..n {
        let (pivot_row, pivot) = (col..n)
            .map(|r| (r, m[r][col].abs()))
            .max_by(|a, b| a.1.total_cmp(&b.1))?;
        if pivot < 1e-14 {
            return None;
        }
        m.swap(col, pivot_row);
        let d = m[col][col];
        for v in m[col].iter_mut().skip(col) {
            *v /= d;
        }
        for r in 0..n {
            if r == col {
                continue;
            }
            let f = m[r][col];
            if f == 0.0 {
                continue;
            }
            let (pivot_row, target) = if r < col {
                let (a, b) = m.split_at_mut(col);
                (&b[0], &mut a[r])
            } else {
                let (a, b) = m.split_at_mut(r);
                (&a[col], &mut b[0])
            };
            for c in col..=n {
                target[c] -= f * pivot_row[c];
            }
        }
    }
    Some((0..n).map(|r| m[r][n]).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    /// Every element that goes in comes out. This is not a quality metric
    /// to be tuned — it is the one law the minimiser is not permitted to
    /// break, and it broke silently for as long as convergence was tested
    /// on the size of the Newton step instead of on the residual: heating
    /// chalk produced 0.20 mol of CO2 from 0.10 mol of carbonate.
    fn assert_conserved(eq: &Equilibrium, budget: &BTreeMap<String, f64>, what: &str) {
        let db = crate::db();
        for (el, target) in budget {
            let have: f64 = eq
                .composition
                .iter()
                .filter_map(|(name, m)| {
                    let s = db.get(name)?;
                    Some(s.composition.get(el).copied().unwrap_or(0.0) * m)
                })
                .sum();
            let drift = (have - target).abs() / target.max(1e-12);
            assert!(
                drift < 1e-6,
                "{what}: {el} went in at {target:.6} mol and came out at {have:.6} mol \
                 ({:.2}% drift)",
                drift * 100.0
            );
        }
    }

    /// Every name must resolve. Filtering silently is how a test pool
    /// loses the very phase it was written to exercise — `CaO(a)` is not a
    /// species in the NASA set, and a pool that quietly dropped it made
    /// chalk look thermally stable at 1500 K.
    fn pool_of(names: &[&str]) -> Vec<&'static crate::nasa9::Species> {
        names
            .iter()
            .map(|n| {
                crate::db()
                    .get(n)
                    .unwrap_or_else(|| panic!("{n} is not in the NASA data"))
            })
            .collect()
    }

    /// A vessel of chalk standing open, as `thermal.rs` actually charges
    /// the solver: the atmosphere is always part of the problem.
    fn chalk_in_air() -> (BTreeMap<String, f64>, Vec<&'static crate::nasa9::Species>) {
        let b = budget(&[
            ("Ca", 0.0999),
            ("C", 0.0999),
            ("O", 0.2997 + 0.336),
            ("N", 1.248),
        ]);
        let pool = pool_of(&[
            "CO2",
            "CO",
            "O2",
            "N2",
            "NO",
            "CaO(cr)",
            "CaCO3(cr)",
            "Ca(a)",
            "C(gr)",
        ]);
        (b, pool)
    }

    #[test]
    fn calcining_chalk_conserves_every_element() {
        // The case that exposed the bug: 0.1 mol of chalk heated in air
        // used to yield 0.20 mol of CO2 and 0.11 mol of quicklime.
        let (b, pool) = chalk_in_air();
        for t in [800.0, 1100.0, 1400.0, 2000.0] {
            let eq = equilibrate_tp(&b, &pool, t, 1.0).expect("a solution");
            assert_conserved(&eq, &b, &format!("calcite at {t} K"));
        }
    }

    #[test]
    fn chalk_decomposes_when_it_is_hot_enough_and_not_before() {
        // The decomposition temperature is a computed result, so the two
        // sides of it are worth pinning: at 800 K the carbonate stands,
        // near 1200 K it does not.
        let (b, pool) = chalk_in_air();
        let cold = equilibrate_tp(&b, &pool, 800.0, 1.0).expect("a solution");
        let hot = equilibrate_tp(&b, &pool, 1500.0, 1.0).expect("a solution");
        assert!(
            cold.moles_of("CaCO3(cr)") > 0.09,
            "chalk survives 800 K: {:?}",
            cold.composition
        );
        assert!(
            hot.moles_of("CaO(cr)") > 0.09,
            "chalk calcines by 1500 K: {:?}",
            hot.composition
        );
    }

    #[test]
    fn a_degenerate_problem_is_refused_rather_than_guessed() {
        // Chalk alone, no atmosphere: one condensed phase holds every
        // element and the gas phase collapses, so the element-balance rows
        // become linearly dependent and the multipliers are
        // underdetermined. The composition is obvious to a chemist and
        // unavailable to this formulation; the solver must say so rather
        // than return whichever answer the arithmetic fell into.
        let b = budget(&[("Ca", 0.0999), ("C", 0.0999), ("O", 0.2997)]);
        let pool = pool_of(&["CO2", "CO", "O2", "CaO(cr)", "CaCO3(cr)", "Ca(a)", "C(gr)"]);
        match equilibrate_tp(&b, &pool, 800.0, 1.0) {
            Err(CeaError::NotConverged(_)) => {}
            Err(other) => panic!("expected a non-convergence, got {other}"),
            Ok(eq) => {
                // If it ever does solve, it must at least conserve.
                assert_conserved(&eq, &b, "degenerate chalk");
            }
        }
    }

    #[test]
    fn burning_magnesium_conserves_every_element() {
        let b = budget(&[("Mg", 0.0494), ("O", 0.4), ("N", 1.5)]);
        let pool = pool_of(&["MgO(cr)", "Mg(cr)", "O2", "N2", "MgO", "Mg"]);
        assert!(!pool.is_empty());
        let eq = equilibrate_tp(&b, &pool, 2450.0, 1.0).expect("a solution");
        assert_conserved(&eq, &b, "magnesium in air");
    }

    #[test]
    fn oxygen_does_not_leave_solid_carbon_behind() {
        // Graphite condensing out of an oxidising atmosphere was the visible
        // symptom of admitting phases on Lagrange multipliers taken from an
        // unconverged system.
        let b = budget(&[("C", 0.1), ("O", 1.0)]);
        let pool = pool_of(&["CO2", "CO", "O2", "C(gr)"]);
        let eq = equilibrate_tp(&b, &pool, 1500.0, 1.0).expect("a solution");
        assert_conserved(&eq, &b, "carbon in excess oxygen");
        assert!(
            eq.moles_of("C(gr)") < 1e-9,
            "carbon cannot stay solid in excess oxygen: {:?}",
            eq.composition
        );
    }

    #[test]
    fn the_adiabatic_solve_conserves_too() {
        let (b, pool) = chalk_in_air();
        let warm = equilibrate_tp(&b, &pool, 1000.0, 1.0).expect("a reference");
        let eq = equilibrate_hp(&b, &pool, warm.enthalpy, 1.0).expect("a solution");
        assert_conserved(&eq, &b, "adiabatic calcite");
    }
}
