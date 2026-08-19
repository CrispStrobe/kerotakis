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
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let dn = sol[elements.len() + c_idx];
            if dn < 0.0 && n[c] > 0.0 {
                lambda = lambda.min((0.9 * n[c] / -dn).min(1.0));
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

        // Phase management: drop an exhausted condensed phase; admit one
        // whose chemical potential says it should exist.
        active_cond.retain(|&c| n[c] > TRACE);
        if max_change < 1e-8 {
            let mut admitted = false;
            for &c in &cond {
                if active_cond.contains(&c) {
                    continue;
                }
                let drive: f64 =
                    (0..elements.len()).map(|j| a(c, j) * pi[j]).sum::<f64>() - mu(c, &n, n_total);
                if drive > 1e-8 {
                    active_cond.push(c);
                    n[c] = (total_budget * 0.01).max(1e-8);
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
