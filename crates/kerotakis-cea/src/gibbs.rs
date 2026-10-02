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
    if !t.is_finite()
        || t <= 0.0
        || !pressure_bar.is_finite()
        || pressure_bar <= 0.0
        || budget.values().any(|n| !n.is_finite() || *n < 0.0)
        || !budget.values().sum::<f64>().is_finite()
    {
        return Err(CeaError::NotConverged(0));
    }
    if budget.is_empty() || budget.values().all(|v| *v <= 0.0) {
        return Err(CeaError::EmptyBudget);
    }
    let elements: Vec<String> = budget
        .iter()
        .filter(|(_, v)| **v > 0.0)
        .map(|(k, _)| k.clone())
        .collect();

    // Only species entirely composed of budgeted elements can appear.
    //
    // A condensed phase additionally exists in this solve only where its
    // data does. `interval_for` clamps to the nearest interval rather than
    // refusing — right for reading a feed enthalpy near its range edge,
    // catastrophically wrong for judging phase stability: the liquid-water
    // polynomial extrapolated to 3125 K says liquid is the stable phase of
    // steam, and a hydrogen flame then "converges" onto boiling-hot
    // H2O(L) or, worse, never converges at all (curiosity th-034). Gases
    // retain the historical nearest-interval polynomial treatment: some
    // records start above a cold HP bracket. Omitting those gases would
    // change the candidate pool or lose an element carrier. The new
    // certified boundary below requires actual interval coverage for every
    // record; it never certifies extrapolated gas thermochemistry.
    let pool: Vec<&Species> = candidates
        .iter()
        .copied()
        .filter(|s| {
            !s.composition.is_empty()
                && s.composition
                    .keys()
                    .all(|el| elements.iter().any(|e| e == el))
                && (s.is_gas() || s.t_range().is_some_and(|(lo, hi)| t >= lo && t <= hi))
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
        if !g.is_finite() {
            return Err(CeaError::OutOfRange(s.name.clone(), t));
        }
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
    //
    // The seed also has to be FEASIBLE, and one phase is not always enough
    // to make it so. The most stable carrier of calcium is the carbonate,
    // and a crucible half way through a calcination holds 0.1 mol of
    // calcium against 0.05 mol of carbon: putting every calcium atom into
    // carbonate asks for twice the carbon that exists, the gas phase
    // cannot lend carbon back (that would be a negative amount of CO2),
    // and the oxide that should take up the slack is admissible only once
    // the balance holds — which it now never can. Every temperature went
    // singular. So walk the carriers most-stable-first and give each one
    // as much as the REMAINING budget supports, until the element is
    // covered. Where the first carrier can hold the lot, which is every
    // case this solver met before a burner could leave a crucible half
    // calcined, the seed is exactly what it always was.
    let mut active_cond: Vec<usize> = Vec::new();
    let mut left: Vec<f64> = elements
        .iter()
        .map(|el| budget.get(el).copied().unwrap_or(0.0))
        .collect();
    for (j, _) in elements.iter().enumerate() {
        let carried_by_gas = gas.iter().any(|&i| a(i, j) > 0.0);
        if carried_by_gas {
            continue;
        }
        // The condensed carriers of this element, most stable per atom
        // first (lowest μ° per atom).
        let mut carriers: Vec<usize> = cond.iter().copied().filter(|&c| a(c, j) > 0.0).collect();
        if carriers.is_empty() {
            return Err(CeaError::NoSpecies);
        }
        carriers.sort_by(|&x, &y| (mu0[x] / a(x, j)).total_cmp(&(mu0[y] / a(y, j))));
        for c in carriers {
            if left[j] <= 0.0 {
                break;
            }
            if active_cond.contains(&c) {
                continue;
            }
            // The most of this phase the budget can still support.
            let feasible = (0..elements.len())
                .filter(|&k| a(c, k) > 0.0)
                .map(|k| left[k] / a(c, k))
                .fold(f64::INFINITY, f64::min);
            let take = (left[j] / a(c, j)).min(feasible);
            if !take.is_finite() || take <= 0.0 {
                continue;
            }
            active_cond.push(c);
            n[c] = take;
            for (k, slot) in left.iter_mut().enumerate() {
                *slot -= take * a(c, k);
            }
        }
    }

    // NASA RP-1311 I §3.6 describes the component-rank singularity when a
    // stoichiometric condensed phase exhausts the gas. A zero gas phase
    // has no log-mole Newton variable; certify that boundary directly.
    // Ordinary gas-containing iterations retain their existing path.
    if pool
        .iter()
        .all(|s| s.intervals.iter().any(|i| (i.t_min..=i.t_max).contains(&t)))
    {
        if let Some((amounts, _)) =
            certified_condensed_boundary(&pool, &mu0, &elements, budget, &n, ln_p)
        {
            let eq = finish(&pool, &amounts, t, pressure_bar);
            if !eq.enthalpy.is_finite()
                || !eq.gas_moles.is_finite()
                || eq
                    .composition
                    .iter()
                    .any(|(_, n)| !n.is_finite() || *n <= 0.0)
                || !eq
                    .composition
                    .iter()
                    .map(|(_, n)| n)
                    .sum::<f64>()
                    .is_finite()
            {
                return Err(CeaError::NotConverged(0));
            }
            return Ok(eq);
        }
    }

    // How often each condensed phase has been admitted. A phase that is
    // admitted, driven out, and admitted again is oscillating rather than
    // converging; capping the retries keeps that from spinning the full 400
    // iterations.
    let mut admissions = vec![0u8; pool.len()];

    // How often a singular linear solve has been repaired by re-seeding a
    // crushed gas carrier (see the rescue below). Capped so a genuinely
    // degenerate problem cannot cycle seed → crush → seed forever.
    let mut rescues = 0u8;
    // Once a rescue has fired, extinction is rate-limited too (see the λ
    // loop): the same violent transient that crushed the species once
    // will otherwise crush the re-seeded copy in a single step and the
    // solve cycles instead of converging. The guard is armed only after
    // a rescue so every problem that never goes singular keeps its exact
    // current iteration path.
    let mut decay_guard = false;

    // Which elements a gas can carry at all. An element with no gaseous
    // form — calcium in a limestone kiln — lives entirely in the condensed
    // phases, so the last solid holding it may not leave: its balance row
    // would go all-zero and the next linear solve would be singular. This
    // is the same guard the initial guess above applies, enforced for the
    // rest of the iteration too.
    let gas_carries: Vec<bool> = (0..elements.len())
        .map(|j| gas.iter().any(|&i| a(i, j) > 0.0))
        .collect();
    // The most of a condensed phase the budget can hold at all. A mole of
    // calcium carbonate needs a mole of calcium and a mole of carbon, and
    // there are only so many of each: no equilibrium, and no step on the
    // way to one, may contain more of the phase than that. Newton does not
    // know it — the damping above bounds a condensed phase that is
    // SHRINKING and leaves a growing one free — and a half-calcined
    // crucible is where that showed: one step put 1.6 mol of carbonate in a
    // vessel holding 0.1 mol of calcium, and nothing recovered from it.
    let phase_cap = |c: usize| -> f64 {
        (0..elements.len())
            .filter(|&k| a(c, k) > 0.0)
            .map(|k| budget.get(&elements[k]).copied().unwrap_or(0.0) / a(c, k))
            .fold(f64::INFINITY, f64::min)
    };
    // Whether dropping this phase would leave an element no gas can carry
    // with no way back to its budget.
    //
    // One carrier per element is the ordinary case and this is then just
    // "is it the last one". Two carriers sharing an element is the case
    // that needs the AMOUNTS as well as the names: chalk and lime both hold
    // calcium, so neither looked like the last one, and a single step
    // dropped both — leaving the calcium row all zeros and every subsequent
    // solve singular.
    // Whether another ACTIVE phase carries the same gas-less element. One
    // carrier per element is what every solve here had until an open
    // crucible could stop half way through a calcination, so this is false
    // everywhere the old code ran and the two branches it guards below
    // leave every existing answer exactly where it was.
    let partnered = |c: usize, active: &[usize]| -> bool {
        (0..elements.len()).any(|j| {
            !gas_carries[j] && a(c, j) > 0.0 && active.iter().any(|&o| o != c && a(o, j) > 0.0)
        })
    };
    let would_strand = |c: usize, survivors: &[usize]| -> bool {
        (0..elements.len()).any(|j| {
            if gas_carries[j] || a(c, j) <= 0.0 {
                return false;
            }
            let target = budget.get(&elements[j]).copied().unwrap_or(0.0);
            let reachable: f64 = survivors
                .iter()
                .copied()
                .filter(|&o| o != c)
                .map(|o| a(o, j) * phase_cap(o))
                .sum();
            reachable < target * (1.0 - 1e-12)
        })
    };

    // OPT-5: allocate the working buffers once, outside the Newton loop.
    let nel = elements.len();
    let max_dim = nel + cond.len() + 1;
    let max_stride = max_dim + 1;
    let mut m_flat = vec![0.0f64; max_dim * max_stride];
    let mut pi = vec![0.0f64; nel];
    let mut d_ln = vec![0.0f64; pool.len()];
    let mut gas_ni = vec![0.0f64; nel];

    // 400 iterations is generous for a healthy problem; a rescued one pays
    // for its guarded, slower steps with a larger budget. Only solves that
    // actually went singular — which today fail outright — ever see the
    // extra iterations, so no other problem's outcome can move.
    let mut iteration = 0usize;
    while iteration < if decay_guard { 1200 } else { 400 } {
        iteration += 1;
        let dim = nel + active_cond.len() + 1;
        let stride = dim + 1;
        m_flat[..dim * stride].fill(0.0);

        // μᵢ/RT for the current composition.
        let mu = |i: usize, n: &[f64], n_total: f64| -> f64 {
            if pool[i].is_gas() {
                let ni = n[i].max(TRACE);
                mu0[i] + (ni / n_total.max(TRACE)).ln() + ln_p
            } else {
                mu0[i]
            }
        };

        // Precompute per-element gas sums: Σ_i a(i,j) * n[i].
        for (j, slot) in gas_ni[..nel].iter_mut().enumerate() {
            *slot = gas.iter().map(|&i| a(i, j) * n[i]).sum();
        }

        // Element-balance rows.
        for (j, _) in elements.iter().enumerate() {
            for (k, _) in elements.iter().enumerate() {
                m_flat[j * stride + k] = gas.iter().map(|&i| a(i, j) * a(i, k) * n[i]).sum();
            }
            for (c_idx, &c) in active_cond.iter().enumerate() {
                m_flat[j * stride + nel + c_idx] = a(c, j);
            }
            m_flat[j * stride + dim - 1] = gas_ni[j];
            let b_current: f64 = (0..pool.len()).map(|i| a(i, j) * n[i]).sum();
            let target = budget.get(&elements[j]).copied().unwrap_or(0.0);
            m_flat[j * stride + dim] = target - b_current
                + gas
                    .iter()
                    .map(|&i| a(i, j) * n[i] * mu(i, &n, n_total))
                    .sum::<f64>();
        }

        // One row per active condensed phase: Σⱼ a_cj πⱼ = μ_c/RT.
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let row = nel + c_idx;
            for (j, _) in elements.iter().enumerate() {
                m_flat[row * stride + j] = a(c, j);
            }
            m_flat[row * stride + dim] = mu(c, &n, n_total);
        }

        // Total-moles row.
        let last = dim - 1;
        for j in 0..nel {
            m_flat[last * stride + j] = gas_ni[j];
        }
        let sum_gas: f64 = gas.iter().map(|&i| n[i]).sum();
        m_flat[last * stride + last] = sum_gas - n_total;
        m_flat[last * stride + dim] =
            n_total - sum_gas + gas.iter().map(|&i| n[i] * mu(i, &n, n_total)).sum::<f64>();

        if !solve_flat(&mut m_flat, dim, stride) {
            // The repairable one: a Newton transient crushed a gas species
            // the element balance still needs. A cold H2/O2/air charge
            // (curiosity th-034) drives O2 and H2 to the trace floor within
            // a few iterations, leaving only saturated carriers — CO2, H2O,
            // N2 — whose compositions are linearly dependent (every
            // survivor's O content is exactly 2·C + H/2), while the oxygen
            // budget cannot fit that subspace. The rows are then dependent
            // but the RHS is not: no step exists, though the equilibrium —
            // with its leftover O2 — certainly does. Re-seed the most
            // stable crushed carrier of each under-carried element with the
            // missing amount and iterate on; this touches only states the
            // solve had already failed on, so every previously converging
            // problem is bit-identical.
            let mut reseeded = false;
            if rescues < 8 {
                // Re-seed every gas species sitting at the trace floor with
                // the same kind of trace the condensed admission uses. The
                // seeds are far below the balance tolerance, but they put
                // every composition direction back into the row space, so
                // the multipliers become determined again; Newton then
                // grows the ones the equilibrium wants (that leftover O2)
                // and re-extinguishes the rest.
                let seed = (total_budget * 1e-9).max(1e-14);
                for &i in &gas {
                    if n[i] < seed {
                        n[i] = seed;
                        reseeded = true;
                    }
                }
            }
            if reseeded {
                rescues += 1;
                decay_guard = true;
                n_total = gas.iter().map(|&i| n[i]).sum::<f64>().max(TRACE);
                continue;
            }
            // The genuine one: one condensed phase is the sole repository
            // of every element and the gas phase has collapsed — the
            // element rows differ only by a stoichiometric factor in that
            // phase's single column, so the multipliers are
            // underdetermined. The composition is not in doubt there, but
            // this formulation cannot produce it, and saying so is better
            // than returning whichever answer the arithmetic fell into.
            return Err(CeaError::NotConverged(iteration));
        };
        for j in 0..nel {
            pi[j] = m_flat[j * stride + dim];
        }
        let d_ln_n = m_flat[(dim - 1) * stride + dim];

        // Corrections to each gas species.
        d_ln.iter_mut().for_each(|v| *v = 0.0);
        for &i in &gas {
            let sum: f64 = (0..nel).map(|j| a(i, j) * pi[j]).sum();
            d_ln[i] = sum + d_ln_n - mu(i, &n, n_total);
        }

        // Damping (RP-1311 §3.3): keep steps sane and amounts positive.
        let mut lambda: f64 = 1.0;
        for &i in &gas {
            if d_ln[i] > 0.0 {
                lambda = lambda.min(2.0 / d_ln[i].abs().max(2.0));
            } else if decay_guard && n[i] > TRACE * 10.0 {
                // After a rescue: a species may fall by at most three
                // decades per iteration, so a stiff transient can no
                // longer erase in one step the very carrier whose absence
                // made the matrix singular. Legitimate extinction still
                // completes in a handful of iterations.
                lambda = lambda.min(6.9 / d_ln[i].abs().max(6.9));
            }
        }
        // A condensed phase being driven out must not be allowed to freeze
        // the whole step. Limiting λ so the phase can only shrink by 90% is
        // right while it is genuinely present, but a phase the solution
        // wants *gone* demands λ→0, which stalls the iteration — and a
        // stalled iteration used to be misread as a converged one. Below a
        // tenth of a percent of a step, remove the phase instead.
        let mut forced_drop: Vec<usize> = Vec::new();
        let mut survivors: Vec<usize> = active_cond.clone();
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let dn = m_flat[(nel + c_idx) * stride + dim];
            if dn < 0.0 && n[c] > 0.0 {
                let limit = (0.9 * n[c] / -dn).min(1.0);
                if limit >= 1e-3 {
                    lambda = lambda.min(limit);
                } else if !would_strand(c, &survivors) {
                    forced_drop.push(c);
                    survivors.retain(|&o| o != c);
                } else if !partnered(c, &active_cond) {
                    lambda = lambda.min(limit);
                }
                // A phase that may not be dropped AND has a partner
                // carrying the same element throttles nothing: let it fall
                // to the floor below, which is the amount conservation
                // leaves it, rather than dragging λ to 1e-19 and freezing
                // the gases mid-transient. That stall is what a
                // half-calcined crucible produced once the calcium row was
                // no longer allowed to empty.
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
            let dn = lambda * m_flat[(nel + c_idx) * stride + dim];
            max_change = max_change.max((dn / n_total.max(TRACE)).abs());
            n[c] = (n[c] + dn).max(0.0).min(phase_cap(c));
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
            if n[c] > TRACE || !would_strand(c, &active_cond) {
                continue;
            }
            if partnered(c, &active_cond) {
                // The other carriers cannot hold this element between
                // them, so whatever they are short of is here. That amount
                // — not a trace — is what conservation says this phase
                // holds, and putting it back is what lets the gas half of
                // the problem get on with converging.
                let need = (0..elements.len())
                    .filter(|&j| !gas_carries[j] && a(c, j) > 0.0)
                    .map(|j| {
                        let target = budget.get(&elements[j]).copied().unwrap_or(0.0);
                        let held: f64 = active_cond
                            .iter()
                            .filter(|&&o| o != c)
                            .map(|&o| a(o, j) * n[o])
                            .sum();
                        ((target - held) / a(c, j)).max(0.0)
                    })
                    .fold(0.0f64, f64::max);
                n[c] = need
                    .max((total_budget * 1e-14).max(1e-16))
                    .min(phase_cap(c));
            } else {
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
                    (0..nel).map(|j| a(c, j) * pi[j]).sum::<f64>() - mu(c, &n, n_total);
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
    Err(CeaError::NotConverged(if decay_guard { 1200 } else { 400 }))
}

/// Global Gibbs certificate for a feasible all-condensed inventory.
/// Dimensionless element potentials pi satisfy a_c.pi=mu_c for occupied
/// phases, a_c.pi<=mu_c for every other condensed phase. For an absent ideal
/// gas, log(sum exp(a_g.pi-mu_g-ln(P/P0)))<=0 is the tangent-plane condition.
/// Those inequalities imply G>=pi.b for every feasible mixture; the occupied
/// condensed phases attain equality. Thus this is proof of stability, not
/// acceptance of a stalled Newton iteration (RP-1311 I eqs2.9–2.11, §3.4).
fn certified_condensed_boundary(
    pool: &[&Species],
    mu0: &[f64],
    elements: &[String],
    budget: &BTreeMap<String, f64>,
    seed: &[f64],
    ln_p: f64,
) -> Option<(Vec<f64>, Vec<f64>)> {
    let tol = 1e-8; // Existing Newton chemical-potential tolerance, in RT units.
    let composition = |i: usize| {
        elements
            .iter()
            .map(|e| pool[i].composition.get(e).copied().unwrap_or(0.0))
            .collect::<Vec<_>>()
    };
    let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    let cond: Vec<_> = (0..pool.len()).filter(|i| !pool[*i].is_gas()).collect();
    if cond.is_empty() {
        return None;
    }
    let feasible = |n: &[f64]| {
        elements.iter().all(|e| {
            let have: f64 = pool
                .iter()
                .zip(n)
                .map(|(s, n)| s.composition.get(e).copied().unwrap_or(0.0) * n)
                .sum();
            (have - budget[e]).abs() <= budget[e] * BALANCE_TOL
        })
    };
    let mut candidates = Vec::new();
    let condensed_seed: Vec<_> = seed
        .iter()
        .enumerate()
        .map(|(i, n)| if pool[i].is_gas() { 0.0 } else { *n })
        .collect();
    if feasible(&condensed_seed) {
        candidates.push(condensed_seed);
    }
    // Pure stoichiometric inventory is common even when its elements could
    // be carried by gas, so the ordinary gas-first seed contains no solid.
    for &i in &cond {
        let amount = elements
            .iter()
            .filter_map(|e| {
                pool[i]
                    .composition
                    .get(e)
                    .filter(|a| **a > 0.0)
                    .map(|a| budget[e] / a)
            })
            .reduce(f64::min)?;
        let mut n = vec![0.0; pool.len()];
        n[i] = amount;
        if feasible(&n) && !candidates.iter().any(|previous| previous == &n) {
            candidates.push(n);
        }
    }
    candidates.sort_by(|a, b| dot(a, mu0).total_cmp(&dot(b, mu0)));
    for n in candidates {
        let active: Vec<_> = cond.iter().copied().filter(|i| n[*i] > 0.0).collect();
        // Orthonormal rows represent the occupied-phase equalities; all
        // following projections move only in their affine nullspace.
        let mut basis = Vec::<(Vec<f64>, f64)>::new();
        let mut consistent = true;
        for &i in &active {
            let mut row = composition(i);
            let mut rhs = mu0[i];
            for (q, value) in &basis {
                let factor = dot(&row, q);
                for (r, q) in row.iter_mut().zip(q) {
                    *r -= factor * q;
                }
                rhs -= factor * value;
            }
            let length = dot(&row, &row).sqrt();
            if length < 1e-12 {
                if rhs.abs() > tol {
                    consistent = false;
                    break;
                }
            } else {
                for r in &mut row {
                    *r /= length;
                }
                basis.push((row, rhs / length));
            }
        }
        if !consistent {
            continue;
        }
        let null = |mut row: Vec<f64>| {
            for (q, _) in &basis {
                let factor = dot(&row, q);
                for (r, q) in row.iter_mut().zip(q) {
                    *r -= factor * q;
                }
            }
            row
        };
        let mut pi = vec![0.0; elements.len()];
        for (q, value) in &basis {
            for (p, q) in pi.iter_mut().zip(q) {
                *p += q * value;
            }
        }
        // Cyclic projections onto condensed halfspaces and the ideal-gas
        // convex tangent inequality. The finite iteration budget affects
        // completeness only; success is checked independently at the end.
        for _ in 0..256 {
            for &i in &cond {
                let row = composition(i);
                let excess = dot(&row, &pi) - mu0[i];
                if excess > tol {
                    let direction = null(row);
                    let norm = dot(&direction, &direction);
                    if norm > 1e-24 {
                        for (p, d) in pi.iter_mut().zip(direction) {
                            *p -= excess * d / norm;
                        }
                    }
                }
            }
            let gases: Vec<_> = (0..pool.len())
                .filter(|i| pool[*i].is_gas())
                .map(|i| (i, dot(&composition(i), &pi) - mu0[i] - ln_p))
                .collect();
            let gas_max = gases
                .iter()
                .map(|(_, log)| *log)
                .reduce(f64::max)
                .unwrap_or(f64::NEG_INFINITY);
            let gas_sum: f64 = gases.iter().map(|(_, log)| (log - gas_max).exp()).sum();
            let log_tangent = if gases.is_empty() {
                f64::NEG_INFINITY
            } else {
                gas_max + gas_sum.ln()
            };
            let phases_valid = cond.iter().all(|i| {
                let difference = dot(&composition(*i), &pi) - mu0[*i];
                difference <= tol && (n[*i] <= 0.0 || difference.abs() <= tol)
            });
            if phases_valid && log_tangent <= tol && pi.iter().all(|p| p.is_finite()) {
                return Some((n, pi));
            }
            if log_tangent > tol {
                let mut gradient = vec![0.0; elements.len()];
                for (i, log) in &gases {
                    let weight = (log - gas_max).exp() / gas_sum;
                    for (g, a) in gradient.iter_mut().zip(composition(*i)) {
                        *g += weight * a;
                    }
                }
                let direction = null(gradient);
                let norm = dot(&direction, &direction);
                if norm <= 1e-24 {
                    break;
                }
                for (p, d) in pi.iter_mut().zip(direction) {
                    *p -= log_tangent * d / norm;
                }
            } else if !phases_valid && active.len() == cond.len() {
                break;
            }
        }
    }
    None
}

pub(crate) fn finish(pool: &[&Species], n: &[f64], t: f64, pressure_bar: f64) -> Equilibrium {
    let mut composition: Vec<(String, f64)> = pool
        .iter()
        .zip(n)
        // A presentation cutoff is not an inventory cutoff. Closed pressure,
        // energy and boundary-flow readbacks need the same numerical moles
        // that contributed to the minimisation and its enthalpy below.
        .filter(|(_, m)| **m > 0.0)
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

/// The part of an adiabatic charge that is the room the vessel stands in
/// rather than anything the vessel holds.
///
/// A closed bomb's contents are all inventory: everything in the charge is
/// there because it was weighed in, and an adiabatic solve may move heat
/// freely between any two parts of it. An OPEN vessel is not that problem.
/// Its atmosphere has to be in the element budget — a crucible with no air
/// above it has no gas phase at all, and nothing could burn — but it is
/// **not a thermal store the vessel owns**. It is room air, and room air is
/// at 298 K.
///
/// So the atmosphere gets one asymmetric rule, and this type is what
/// carries it into the temperature search:
///
/// > The air a vessel stands in may carry heat AWAY from the charge. It may
/// > never pay FOR it.
///
/// The upward half is the flame: a burn really does entrain the room and
/// really does heat it, and the nitrogen it drags through the flame front
/// is the diluent that keeps an adiabatic flame temperature finite. The
/// downward half is what this exists to forbid. An endothermic
/// decomposition — calcining chalk in a crucible — would otherwise be part
/// paid for by the sensible heat of eight times the vessel's own moles of
/// air cooling from kiln temperature back down, heat that no burner ever
/// delivered and that `Vessel::heat_capacity()` has never contained.
#[derive(Debug, Clone)]
pub struct OpenAtmosphere {
    /// Species name → moles of it admitted to the charge.
    pub admitted: BTreeMap<String, f64>,
    /// The temperature the admitted gas was valued at when the reactants'
    /// enthalpy was totalled.
    pub inlet_k: f64,
}

/// Heat the atmosphere would be HANDING the charge at this temperature, J,
/// as a negative number; zero when it is taking heat instead.
///
/// Only the admitted gas that is still atmosphere counts — oxygen that a
/// burn has bound into a product is no longer the room's, and its enthalpy
/// of formation is the reaction's business. Everything else enters at
/// `inlet_k` and leaves at `t`, so its sensible change is what the charge
/// gained or lost by having it there.
fn atmosphere_credit(atmosphere: Option<&OpenAtmosphere>, eq: &Equilibrium) -> f64 {
    let Some(atmosphere) = atmosphere else {
        return 0.0;
    };
    let db = crate::nasa9::db();
    let mut sensible = 0.0;
    for (name, admitted) in &atmosphere.admitted {
        let Some(species) = db.get(name) else {
            continue;
        };
        let still_air = eq.moles_of(name).min(*admitted);
        if still_air <= 0.0 {
            continue;
        }
        let (Some(out), Some(in_)) = (species.h(eq.temperature), species.h(atmosphere.inlet_k))
        else {
            continue;
        };
        sensible += still_air * (out - in_);
    }
    // Positive: the atmosphere absorbed heat, which is allowed and already
    // in the products' enthalpy. Negative: it is trying to pay, and this is
    // the amount that must be taken back out of the balance.
    sensible.min(0.0)
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
    equilibrate_hp_open(budget, candidates, enthalpy, pressure_bar, None)
}

/// [`equilibrate_hp`] for a vessel that stands open in a room.
///
/// Identical to it wherever the charge ends up hotter than it started —
/// every flame temperature in this crate is the same number it always was —
/// and different only where the answer would have been bought with the
/// atmosphere's own sensible heat. See [`OpenAtmosphere`].
pub fn equilibrate_hp_open(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    enthalpy: f64,
    pressure_bar: f64,
    atmosphere: Option<&OpenAtmosphere>,
) -> Result<Equilibrium, CeaError> {
    equilibrate_hp_using(budget, candidates, enthalpy, atmosphere, 250.0, None, |t| {
        equilibrate_tp(budget, candidates, t, pressure_bar)
    })
}

/// The same strict energy root for a scientifically restricted TP model.
/// Endpoints and latent mixtures still undergo all HP conservation and
/// common-potential validations; a callback is not an energy fallback.
pub(crate) fn equilibrate_hp_using(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    enthalpy: f64,
    atmosphere: Option<&OpenAtmosphere>,
    lower_floor: f64,
    precise_energy_tolerance: Option<f64>,
    mut tp: impl FnMut(f64) -> Result<Equilibrium, CeaError>,
) -> Result<Equilibrium, CeaError> {
    // Bisection on T: H(T) rises monotonically, so this is robust where a
    // Newton step on a stiff flame problem is not.
    //
    // The bracket endpoints are not the flame problem. An ignited H2/O2
    // charge at 250 K is frozen chemistry evaluated only to anchor the
    // search, and it is exactly where the equilibrium constants are most
    // savage (e^Δμ/RT in the hundreds) and the minimiser most likely to
    // stall. A convergence failure at a cold bracket point therefore does
    // not doom the flame solve: raise the floor until a temperature
    // converges, and treat a failing midpoint as belonging to the cold,
    // stiff side. These bracket heuristics never authorize an unmatched
    // result: every accepted HP state must close the corrected enthalpy
    // residual below, or the charge remains unsolved.
    let dbg = std::env::var("KERO_CEA_DEBUG").is_ok();
    let (mut lo, mut hi) = (lower_floor, 6000.0f64);
    let mut last = loop {
        match tp(lo) {
            Ok(eq) => {
                if dbg {
                    eprintln!(
                        "HP floor {lo:.0} K ok, H={:.3e} vs target {enthalpy:.3e}",
                        eq.enthalpy
                    );
                }
                break eq;
            }
            // 250 → 400 → 640 → 1024 → 1638 K; a charge whose equilibrium
            // cannot be computed anywhere below the search midpoint is
            // genuinely unsolved and keeps its honest error.
            Err(_) if lo < 2000.0 => {
                if std::env::var("KERO_CEA_DEBUG").is_ok() {
                    eprintln!("HP floor {lo:.0} K failed, raising");
                }
                lo *= 1.6;
            }
            Err(e) => return Err(e),
        }
    };
    // What the charge's own matter has to account for. The atmosphere's
    // sensible heat is subtracted back out wherever it would be a credit,
    // so the balance the search closes is the vessel's, not the room's.
    if last.enthalpy - atmosphere_credit(atmosphere, &last) > enthalpy {
        // A data/convergence floor is not an adiabatic energy solution.
        // Leave the charge unchanged when its energy cannot be bracketed.
        return Err(CeaError::NotConverged(0));
    }
    let energy_tolerance = precise_energy_tolerance.unwrap_or(1e-6 + enthalpy.abs() * 1e-8);
    // Deterministic restricted models can resolve source headroom far more
    // tightly than a noisy general Newton composition. Their numerical
    // acceptance is conservative: never claim more product H than supplied.
    let accepted = |eq: &Equilibrium| {
        let residual = eq.enthalpy - atmosphere_credit(atmosphere, eq) - enthalpy;
        residual.abs() <= energy_tolerance
            && (precise_energy_tolerance.is_none() || residual <= 0.0)
    };
    let mut lower_state = last.clone();
    let mut upper_state = None;
    let mut failed_mids = 0u8;
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        match tp(mid) {
            Ok(eq) => {
                if dbg {
                    eprintln!("HP mid {mid:.0} K ok, H={:.3e}", eq.enthalpy);
                }
                last = eq;
                if accepted(&last) {
                    return Ok(last);
                }
                if last.enthalpy - atmosphere_credit(atmosphere, &last) < enthalpy {
                    lo = mid;
                    lower_state = last.clone();
                } else {
                    hi = mid;
                    upper_state = Some(last.clone());
                }
            }
            Err(e) => {
                failed_mids += 1;
                if dbg {
                    eprintln!("HP mid {mid:.0} K FAILED ({e})");
                }
                if failed_mids > 8 {
                    return Err(e);
                }
                match e {
                    // Representability has a ceiling, not a floor: every
                    // condensed record ends somewhere, and above the last
                    // one an element with no gaseous form has no carrier.
                    // The answer, if the pool holds one, lies below.
                    CeaError::NoSpecies | CeaError::OutOfRange(_, _) => hi = mid,
                    // Stiffness lives on the cold side, where the
                    // equilibrium constants are most savage.
                    _ => lo = mid,
                }
            }
        }
        if hi - lo
            < if precise_energy_tolerance.is_some() {
                1e-12
            } else {
                1e-8
            }
        {
            break;
        }
    }
    if accepted(&last) {
        Ok(last)
    } else if precise_energy_tolerance.is_some() && accepted(&lower_state) {
        Ok(lower_state)
    } else if let Some(upper) = upper_state {
        if precise_energy_tolerance.is_some() {
            return condensed_coexistence_with_precision(
                budget,
                candidates,
                enthalpy,
                atmosphere,
                &lower_state,
                &upper,
                energy_tolerance,
                true,
            );
        }
        condensed_coexistence(
            budget,
            candidates,
            enthalpy,
            atmosphere,
            &lower_state,
            &upper,
        )
    } else {
        Err(CeaError::NotConverged(60))
    }
}

/// At a first-order condensed transition H(T) has a latent-heat interval,
/// not an unattainable gap. Stable phases with equal extensive Gibbs energy
/// may coexist at one T/P; their lever-rule amounts close assigned enthalpy.
/// Chemical coexistence additionally requires compatible gas composition
/// and a common element-potential certificate for every condensed phase.
fn condensed_coexistence(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    target: f64,
    atmosphere: Option<&OpenAtmosphere>,
    lower: &Equilibrium,
    upper: &Equilibrium,
) -> Result<Equilibrium, CeaError> {
    condensed_coexistence_with_precision(
        budget,
        candidates,
        target,
        atmosphere,
        lower,
        upper,
        1e-6 + target.abs() * 1e-8,
        false,
    )
}

fn condensed_coexistence_with_precision(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    target: f64,
    atmosphere: Option<&OpenAtmosphere>,
    lower: &Equilibrium,
    upper: &Equilibrium,
    energy_tolerance: f64,
    conservative: bool,
) -> Result<Equilibrium, CeaError> {
    let fail = || CeaError::NotConverged(60);
    if upper.temperature - lower.temperature > 1e-6
        || upper.temperature < lower.temperature
        || (upper.pressure_bar - lower.pressure_bar).abs() > lower.pressure_bar * 1e-10
    {
        return Err(fail());
    }
    let lookup = |name: &str| candidates.iter().copied().find(|s| s.name == name);
    let changing: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|s| {
            !s.is_gas() && (lower.moles_of(&s.name) - upper.moles_of(&s.name)).abs() > TRACE
        })
        .collect();
    if changing.is_empty() {
        return Err(fail());
    }
    let mut temperature = 0.5 * (lower.temperature + upper.temperature);
    // Adjacent solid/liquid data ranges meet at an assigned transition T.
    // Use that exact common endpoint, so neither phase is extrapolated.
    for cold in &changing {
        for hot in &changing {
            if cold.composition == hot.composition && cold.name != hot.name {
                if let (Some((_, top)), Some((bottom, _))) = (cold.t_range(), hot.t_range()) {
                    if (top - bottom).abs() <= 1e-8 && (top - temperature).abs() <= 1e-6 {
                        temperature = top;
                    }
                }
            }
        }
    }
    let polymorphic = changing.iter().all(|phase| {
        changing
            .iter()
            .any(|other| other.name != phase.name && other.composition == phase.composition)
    });
    for phase in &changing {
        let gp = phase.g(temperature).ok_or_else(fail)?;
        if !phase
            .t_range()
            .is_some_and(|(lo, hi)| temperature >= lo && temperature <= hi)
        {
            return Err(fail());
        }
        let counterpart = changing.iter().any(|other| {
            other.name != phase.name
                && other.composition == phase.composition
                && (lower.moles_of(&phase.name) - upper.moles_of(&phase.name))
                    * (lower.moles_of(&other.name) - upper.moles_of(&other.name))
                    < 0.0
                && other.g(temperature).is_some_and(|g| {
                    (g - gp).abs()
                        <= gibbs_coefficient_rounding(phase, temperature)
                            + gibbs_coefficient_rounding(other, temperature)
                })
        });
        if polymorphic && !counterpart {
            return Err(fail());
        }
        // A lower-G third polymorph invalidates the proposed coexistence.
        if candidates.iter().any(|other| {
            !other.is_gas()
                && other.composition == phase.composition
                && other
                    .t_range()
                    .is_some_and(|(lo, hi)| temperature >= lo && temperature <= hi)
                && other.g(temperature).is_some_and(|g| {
                    g < gp
                        - gibbs_coefficient_rounding(phase, temperature)
                        - gibbs_coefficient_rounding(other, temperature)
                })
        }) {
            return Err(fail());
        }
    }
    if polymorphic {
        let gas_scale = lower.gas_moles.max(upper.gas_moles).max(TRACE);
        if candidates.iter().any(|s| {
            s.is_gas()
                && (lower.moles_of(&s.name) - upper.moles_of(&s.name)).abs() > gas_scale * 1e-8
        }) {
            return Err(fail());
        }
    } else {
        certify_chemical_coexistence(budget, candidates, temperature, lower, upper)?;
    }
    let pool: Vec<_> = lower
        .composition
        .iter()
        .chain(upper.composition.iter())
        .map(|(name, _)| name.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|name| lookup(name).ok_or_else(fail))
        .collect::<Result<_, _>>()?;
    let elements: Vec<_> = budget.keys().cloned().collect();
    let mixed = |fraction: f64| -> Result<Equilibrium, CeaError> {
        let amounts: Vec<_> = pool
            .iter()
            .map(|s| {
                lower.moles_of(&s.name) * (1.0 - fraction) + upper.moles_of(&s.name) * fraction
            })
            .collect();
        if balance_residual(&pool, &amounts, &elements, budget) > BALANCE_TOL {
            return Err(fail());
        }
        Ok(finish(&pool, &amounts, temperature, lower.pressure_bar))
    };
    let low = mixed(0.0)?;
    let high = mixed(1.0)?;
    let corrected = |eq: &Equilibrium| eq.enthalpy - atmosphere_credit(atmosphere, eq);
    if corrected(&low) > target || corrected(&high) < target {
        return Err(fail());
    }
    let mut lo = 0.0;
    let mut hi = 1.0;
    for _ in 0..64 {
        let eq = mixed(0.5 * (lo + hi))?;
        let residual = corrected(&eq) - target;
        if residual.abs() <= energy_tolerance && (!conservative || residual <= 0.0) {
            return Ok(eq);
        }
        if residual < 0.0 {
            lo = 0.5 * (lo + hi);
        } else {
            hi = 0.5 * (lo + hi);
        }
    }
    Err(fail())
}

/// A linear combination of TP equilibria is itself an equilibrium only
/// when the gas chemical potentials agree (same fractions/P) and the
/// condensed phases share element multipliers. Merely matching atoms/H
/// would interpolate across arbitrary failed roots and is insufficient.
fn certify_chemical_coexistence(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    t: f64,
    lower: &Equilibrium,
    upper: &Equilibrium,
) -> Result<(), CeaError> {
    let fail = || CeaError::NotConverged(60);
    let template = if lower.gas_moles > upper.gas_moles {
        lower
    } else {
        upper
    };
    if template.gas_moles <= TRACE {
        return Err(fail());
    }
    // A vanishing incipient gas is below the native elemental resolution;
    // its rounded mole fractions are not a second chemical potential.
    // The finite endpoint supplies them, and the common-potential and G
    // certificates below still verify the phase-birth reaction itself.
    let gas_resolution = template.gas_moles * BALANCE_TOL;
    if lower.gas_moles > gas_resolution
        && upper.gas_moles > gas_resolution
        && candidates.iter().any(|s| {
            s.is_gas()
                && (lower.moles_of(&s.name) / lower.gas_moles
                    - upper.moles_of(&s.name) / upper.gas_moles)
                    .abs()
                    > 1e-8
        })
    {
        return Err(fail());
    }
    let elements: Vec<_> = budget
        .iter()
        .filter(|(_, n)| **n > 0.0)
        .map(|(e, _)| e)
        .collect();
    let active: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|s| {
            if s.is_gas() {
                template.moles_of(&s.name) > TRACE
            } else {
                lower.moles_of(&s.name).max(upper.moles_of(&s.name)) > TRACE
            }
        })
        .collect();
    let mu = |s: &Species| -> Option<f64> {
        let g = s.g(t)?;
        if s.is_gas() {
            let y = template.moles_of(&s.name) / template.gas_moles;
            (y > 0.0).then(|| g + R * t * (y * template.pressure_bar).ln())
        } else {
            Some(g)
        }
    };
    let dim = elements.len();
    let stride = dim + 1;
    let mut matrix = vec![0.0; dim * stride];
    for s in &active {
        let chemical = mu(s).ok_or_else(fail)?;
        for (i, e) in elements.iter().enumerate() {
            let a = s.composition.get(*e).copied().unwrap_or(0.0);
            for (j, f) in elements.iter().enumerate() {
                matrix[i * stride + j] += a * s.composition.get(*f).copied().unwrap_or(0.0);
            }
            matrix[i * stride + dim] += a * chemical;
        }
    }
    // An underdetermined certificate is not proof that every possible
    // third phase is stable; refuse rather than choose arbitrary potentials.
    if !solve_flat(&mut matrix, dim, stride) {
        return Err(fail());
    }
    let tolerance = active
        .iter()
        .map(|s| gibbs_coefficient_rounding(s, t))
        .sum::<f64>()
        + active
            .iter()
            .filter_map(|s| s.h(t))
            .map(f64::abs)
            .sum::<f64>()
            * (upper.temperature - lower.temperature)
            / t
        + R * t * 1e-8;
    for s in candidates {
        if s.composition.is_empty()
            || !s
                .composition
                .keys()
                .all(|e| budget.get(e).is_some_and(|n| *n > 0.0))
            || !s.t_range().is_some_and(|(lo, hi)| (lo..=hi).contains(&t))
        {
            continue;
        }
        let potential: f64 = elements
            .iter()
            .enumerate()
            .map(|(i, e)| s.composition.get(*e).copied().unwrap_or(0.0) * matrix[i * stride + dim])
            .sum();
        if active.iter().any(|a| a.name == s.name) {
            if (mu(s).ok_or_else(fail)? - potential).abs() > tolerance {
                return Err(fail());
            }
        } else if !s.is_gas() && s.g(t).ok_or_else(fail)? < potential - tolerance {
            return Err(fail());
        }
    }
    let extensive_g = |eq: &Equilibrium| -> Option<f64> {
        candidates.iter().try_fold(0.0, |sum, s| {
            let n = eq.moles_of(&s.name);
            if n <= TRACE {
                return Some(sum);
            }
            let g = s.g(t)?
                + if s.is_gas() {
                    R * t * (n / eq.gas_moles * eq.pressure_bar).ln()
                } else {
                    0.0
                };
            Some(sum + n * g)
        })
    };
    let amount_scale = lower
        .composition
        .iter()
        .chain(&upper.composition)
        .map(|(_, n)| n)
        .sum::<f64>();
    if (extensive_g(lower).ok_or_else(fail)? - extensive_g(upper).ok_or_else(fail)?).abs()
        > tolerance * amount_scale
    {
        return Err(fail());
    }
    Ok(())
}

/// NASA input coefficients are printed with ten significant figures. G is
/// linear in them; summing half a decimal ULP times each exact sensitivity
/// bounds representational mismatch between two fits at a shared endpoint.
/// This is not a tunable phase-stability tolerance.
fn gibbs_coefficient_rounding(species: &Species, t: f64) -> f64 {
    let Some(interval) = species
        .intervals
        .iter()
        .find(|i| t >= i.t_min && t <= i.t_max)
    else {
        return 0.0;
    };
    let weights = [
        -1.0 / (2.0 * t),
        t.ln() + 1.0,
        t * (1.0 - t.ln()),
        -t * t / 2.0,
        -t.powi(3) / 6.0,
        -t.powi(4) / 12.0,
        -t.powi(5) / 20.0,
        1.0,
        -t,
    ];
    let decimal = interval
        .coeffs
        .iter()
        .zip(weights)
        .map(|(a, weight)| {
            if *a == 0.0 {
                0.0
            } else {
                0.5 * 10_f64.powf(a.abs().log10().floor() - 9.0) * weight.abs()
            }
        })
        .sum::<f64>()
        * R;
    let floating = interval
        .coeffs
        .iter()
        .zip(weights)
        .map(|(a, weight)| (a * weight).abs())
        .sum::<f64>()
        * R
        * 32.0
        * f64::EPSILON;
    decimal + floating
}

/// Gauss-Jordan with partial pivoting on a flat row-major augmented matrix.
/// Returns true on success; the solution is in the last column (index `n`
/// within each row of stride `s`). Returns false on singular.
fn solve_flat(m: &mut [f64], n: usize, s: usize) -> bool {
    for col in 0..n {
        let (pivot_row, pivot) = match (col..n)
            .map(|r| (r, m[r * s + col].abs()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
        {
            Some(p) => p,
            None => return false,
        };
        if pivot < 1e-14 {
            return false;
        }
        if col != pivot_row {
            for c in 0..=n {
                m.swap(col * s + c, pivot_row * s + c);
            }
        }
        let d = m[col * s + col];
        for c in col..=n {
            m[col * s + c] /= d;
        }
        for r in 0..n {
            if r == col {
                continue;
            }
            let f = m[r * s + col];
            if f == 0.0 {
                continue;
            }
            for c in col..=n {
                m[r * s + c] -= f * m[col * s + c];
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prove_condensed_endpoint(
        pool: &[&Species],
        b: &BTreeMap<String, f64>,
        t: f64,
        p: f64,
    ) -> Vec<f64> {
        let elements: Vec<_> = b.keys().cloned().collect();
        let mu: Vec<_> = pool.iter().map(|s| s.g(t).unwrap() / (R * t)).collect();
        let (n, pi) =
            certified_condensed_boundary(pool, &mu, &elements, b, &vec![0.0; pool.len()], p.ln())
                .expect("a stable all-condensed endpoint needs a complete dual certificate");
        let potential = |s: &Species| {
            elements
                .iter()
                .zip(&pi)
                .map(|(e, lambda)| s.composition.get(e).copied().unwrap_or(0.0) * lambda)
                .sum::<f64>()
                * R
                * t
        };
        let mut tangent = 0.0;
        for (s, n) in pool.iter().zip(&n) {
            let g = s.g(t).unwrap();
            if s.is_gas() {
                tangent += ((potential(s) - g) / (R * t)).exp() / p;
            } else {
                assert!(
                    g >= potential(s) - 2e-8 * R * t,
                    "inactive {} would lower G",
                    s.name
                );
                if *n > 0.0 {
                    assert!(
                        (g - potential(s)).abs() < 2e-8 * R * t,
                        "occupied {} is not stationary",
                        s.name
                    );
                }
            }
        }
        assert!(
            tangent <= 1.0 + 2e-8,
            "ideal gas would be stable: tangent sum {tangent}"
        );
        let eq = equilibrate_tp(b, pool, t, p).unwrap();
        assert_eq!(eq.gas_moles, 0.0);
        assert!(eq.composition.iter().all(|(name, _)| !pool
            .iter()
            .find(|s| s.name == *name)
            .unwrap()
            .is_gas()));
        assert_conserved(&eq, b, "certified condensed endpoint");
        n
    }

    #[test]
    fn graphite_vapour_pressure_selects_the_correct_phase_on_both_sides() {
        let solid = crate::db().get("C(gr)").unwrap();
        let gas = crate::db().get("C").unwrap();
        let b = budget(&[("C", 0.1)]);
        for t in [1000.0, 2500.0, 4000.0] {
            // Independent one-component saturation pressure from NASA G.
            let saturated = ((solid.g(t).unwrap() - gas.g(t).unwrap()) / (R * t)).exp();
            let pool = [solid, gas];
            let n = prove_condensed_endpoint(&pool, &b, t, 2.0 * saturated);
            assert!((n[0] - 0.1).abs() < 1e-12);
            let lower = 0.5 * saturated;
            let mu = pool
                .iter()
                .map(|s| s.g(t).unwrap() / (R * t))
                .collect::<Vec<_>>();
            assert!(
                certified_condensed_boundary(
                    &pool,
                    &mu,
                    &["C".into()],
                    &b,
                    &[0.0, 0.0],
                    lower.ln()
                )
                .is_none(),
                "supersaturated gas must never be suppressed"
            );
            let eq = equilibrate_tp(&b, &pool, t, lower).unwrap();
            assert!((eq.gas_moles - 0.1).abs() < 1e-9);
            assert_eq!(eq.moles_of("C(gr)"), 0.0);
            assert_conserved(&eq, &b, "carbon vapour");
        }
    }

    #[test]
    fn certified_condensed_readback_preserves_positive_stock_below_newton_floor() {
        let pool = [
            crate::db().get("C(gr)").unwrap(),
            crate::db().get("C").unwrap(),
        ];
        let b = budget(&[("C", 1e-20)]);
        let eq = equilibrate_tp(&b, &pool, 1000.0, 1.0).unwrap();
        assert_eq!(eq.moles_of("C(gr)"), 1e-20);
        assert_eq!(eq.gas_moles, 0.0);
        assert_eq!(eq.composition.len(), 1);
    }

    #[test]
    fn condensed_boundary_rejects_malformed_or_overflowing_numeric_states() {
        let pool = [
            crate::db().get("C(gr)").unwrap(),
            crate::db().get("C").unwrap(),
        ];
        for value in [f64::NAN, f64::INFINITY, -1.0, f64::MAX] {
            assert!(
                equilibrate_tp(&budget(&[("C", value)]), &pool, 1000.0, 1.0).is_err(),
                "invalid stock {value}"
            );
        }
        assert!(equilibrate_tp(
            &budget(&[("C", f64::MAX), ("O", f64::MAX)]),
            &pool,
            1000.0,
            1.0
        )
        .is_err());
        let b = budget(&[("C", 0.1)]);
        for invalid in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
            assert!(equilibrate_tp(&b, &pool, invalid, 1.0).is_err());
            assert!(equilibrate_tp(&b, &pool, 1000.0, invalid).is_err());
        }
        assert!(equilibrate_tp(&b, &pool, f64::MAX, 1.0).is_err());
    }

    #[test]
    fn condensed_boundary_never_certifies_an_extrapolated_gas_record() {
        let liquid = crate::db().get_reactant("C2H5OH(L)").unwrap();
        let gas = crate::db().get("C2H5OH").unwrap();
        let pool = [liquid, gas];
        let b = budget(&[("C", 0.02), ("H", 0.06), ("O", 0.01)]);
        assert!(liquid
            .intervals
            .iter()
            .any(|i| (i.t_min..=i.t_max).contains(&250.0)));
        assert!(!gas
            .intervals
            .iter()
            .any(|i| (i.t_min..=i.t_max).contains(&250.0)));
        // This one-component/multiple-element pool has a rank-deficient
        // legacy gas Newton matrix. It must retain its refusal at250K;
        // otherwise the new path would have certified extrapolated G.
        assert!(equilibrate_tp(&b, &pool, 250.0, 1.0).is_err());
        // Both actual Cp/G domains contain310K, below ethanol's1bar boil.
        let eq = equilibrate_tp(&b, &pool, 310.0, 1.0).unwrap();
        assert_eq!(eq.gas_moles, 0.0);
        assert!((eq.moles_of("C2H5OH(L)") - 0.01).abs() < 1e-12);
        assert_conserved(&eq, &b, "in-range liquid ethanol boundary");
    }

    #[test]
    fn multiple_carbon_gases_use_the_joint_tangent_stability_test() {
        let solid = crate::db().get("C(gr)").unwrap();
        let pool = [
            solid,
            crate::db().get("C").unwrap(),
            crate::db().get("C2").unwrap(),
            crate::db().get("C3").unwrap(),
        ];
        let b = budget(&[("C", 0.1)]);
        for t in [1000.0, 2500.0] {
            let saturation: f64 = pool[1..]
                .iter()
                .map(|gas| {
                    let atoms = gas.composition["C"];
                    ((atoms * solid.g(t).unwrap() - gas.g(t).unwrap()) / (R * t)).exp()
                })
                .sum();
            prove_condensed_endpoint(&pool, &b, t, 2.0 * saturation);
            let mu = pool
                .iter()
                .map(|s| s.g(t).unwrap() / (R * t))
                .collect::<Vec<_>>();
            assert!(certified_condensed_boundary(
                &pool,
                &mu,
                &["C".into()],
                &b,
                &[0.0; 4],
                (0.5 * saturation).ln()
            )
            .is_none());
        }
    }

    #[test]
    fn bare_carbonate_is_certified_against_gas_and_competing_solids() {
        let b = budget(&[("Ca", 0.1), ("C", 0.1), ("O", 0.3)]);
        let candidates = [
            "CaCO3(cr)",
            "CaO(cr)",
            "Ca(a)",
            "Ca(b)",
            "Ca(L)",
            "C(gr)",
            "CO2",
            "CO",
            "O2",
        ];
        for t in [500.0, 800.0, 1000.0] {
            let pool: Vec<_> = candidates
                .iter()
                .map(|name| crate::db().get(name).unwrap())
                .filter(|s| {
                    s.is_gas() || s.t_range().is_some_and(|(lo, hi)| (lo..=hi).contains(&t))
                })
                .collect();
            prove_condensed_endpoint(&pool, &b, t, 1.0);
            let eq = equilibrate_tp(&b, &pool, t, 1.0).unwrap();
            assert!((eq.moles_of("CaCO3(cr)") - 0.1).abs() < 1e-9);
            assert_eq!(eq.moles_of("CaO(cr)"), 0.0);
        }
    }

    #[test]
    fn salt_condensed_phase_selection_uses_nasa_solid_and_liquid_records() {
        let b = budget(&[("Na", 0.1), ("Cl", 0.1)]);
        for (t, expected) in [(800.0, "NaCL(cr)"), (1200.0, "NaCL(L)")] {
            let pool: Vec<_> = ["NaCL(cr)", "NaCL(L)", "NaCL", "Na", "CL"]
                .iter()
                .map(|name| crate::db().get(name).unwrap())
                .filter(|s| {
                    s.is_gas() || s.t_range().is_some_and(|(lo, hi)| (lo..=hi).contains(&t))
                })
                .collect();
            prove_condensed_endpoint(&pool, &b, t, 1.0);
            let eq = equilibrate_tp(&b, &pool, t, 1.0).unwrap();
            assert!((eq.moles_of(expected) - 0.1).abs() < 1e-9);
        }
    }

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
                .map(|(name, m)| {
                    let s = db
                        .get(name)
                        .or_else(|| db.get_reactant(name))
                        .unwrap_or_else(|| panic!("{what}: {name} has no NASA species record"));
                    s.composition.get(el).copied().unwrap_or(0.0) * m
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

    /// The atmosphere `chalk_in_air` puts in that budget, by name.
    fn admitted_air() -> BTreeMap<String, f64> {
        budget(&[("N2", 0.624), ("O2", 0.168)])
    }

    /// Enthalpy of a composition at `t`, less whatever of it is still the
    /// room's own air: what the VESSEL is holding, in J.
    fn own_enthalpy(eq: &Equilibrium, t: f64) -> f64 {
        let db = crate::db();
        let air = admitted_air();
        eq.composition
            .iter()
            .filter_map(|(name, moles)| {
                let s = db.get(name)?;
                let mine = moles - air.get(name).copied().unwrap_or(0.0).min(*moles);
                Some(s.h(t)? * mine)
            })
            .sum()
    }

    #[test]
    fn the_room_a_vessel_stands_in_may_be_warmed_and_may_never_pay() {
        // 0.1 mol of chalk at burner temperature, charged the way
        // `thermal.rs` charges it: with eight times its own moles of air in
        // the budget, because the gas phase needs to exist.
        //
        // That air is 0.792 mol of N2 and O2 at 1773 K. Let it into the
        // energy balance and it is a 26 J/K flywheel against the chalk's
        // own 10 — so a calcination that costs 17.9 kJ barely moves the
        // thermometer, and most of the bill is paid by room air cooling
        // down. It cannot be: room air is at 298 K, and a crucible standing
        // in it is not a bomb calorimeter.
        let (b, pool) = chalk_in_air();
        let t = 1773.15;
        let db = crate::db();
        let chalk = 0.0999 * db.get("CaCO3(cr)").unwrap().h(t).unwrap();
        let air: f64 = admitted_air()
            .iter()
            .map(|(name, moles)| db.get(name).unwrap().h(t).unwrap() * moles)
            .sum();

        let as_inventory = equilibrate_hp(&b, &pool, chalk + air, 1.0).expect("closed answer");
        let atmosphere = OpenAtmosphere {
            admitted: admitted_air(),
            inlet_k: t,
        };
        let as_a_room = equilibrate_hp_open(&b, &pool, chalk + air, 1.0, Some(&atmosphere))
            .expect("open answer");

        assert!(
            as_a_room.temperature < as_inventory.temperature - 100.0,
            "treating the room as a thermal store holds the crucible up at {:.0} K              where its own enthalpy only reaches {:.0} K",
            as_inventory.temperature,
            as_a_room.temperature
        );
        assert_conserved(&as_a_room, &b, "chalk in a room");

        // And the open answer BALANCES on the vessel alone: what the chalk
        // was worth at 1773 K is what the crucible's own matter is worth
        // where the solve lands, to a few joules out of 17 900.
        let before = chalk;
        let after = own_enthalpy(&as_a_room, as_a_room.temperature);
        assert!(
            (after - before).abs() < 100.0,
            "the vessel's own energy balance must close: {before:.1} J of chalk went in              and {after:.1} J came out, a gap of {:.1} J against a 17 880 J calcination",
            after - before
        );
        // The same balance on the old answer is the size of the defect.
        let inventory_after = own_enthalpy(&as_inventory, as_inventory.temperature);
        assert!(
            inventory_after - before > 5_000.0,
            "the closed reading should be the one that invents energy, but it is out              by only {:.1} J",
            inventory_after - before
        );
    }

    #[test]
    fn declaring_the_room_does_not_move_a_flame() {
        // The rule is one-sided on purpose. A flame really does entrain the
        // room and really does heat it, and the nitrogen it drags through
        // the front is the diluent that keeps the adiabatic temperature
        // finite — so every solve that ends HOTTER than it started is the
        // number it was before this existed.
        let species = pool_of(&[
            "CH4", "O2", "N2", "CO2", "H2O", "CO", "H2", "OH", "O", "H", "NO",
        ]);
        let reactants = [("CH4", 1.0), ("O2", 2.0), ("N2", 7.52)];
        let h_cold: f64 = reactants
            .iter()
            .map(|(n, m)| crate::db().get(n).unwrap().h(298.15).unwrap() * m)
            .sum();
        let b = budget(&[("C", 1.0), ("H", 4.0), ("O", 4.0), ("N", 15.04)]);

        let plain = equilibrate_hp(&b, &species, h_cold, 1.0).expect("flame");
        let atmosphere = OpenAtmosphere {
            admitted: budget(&[("N2", 7.52), ("O2", 2.0)]),
            inlet_k: 298.15,
        };
        let open = equilibrate_hp_open(&b, &species, h_cold, 1.0, Some(&atmosphere))
            .expect("flame in a room");
        assert!(
            (plain.temperature - open.temperature).abs() < 1e-9,
            "an exothermic solve may not move: {:.1} K became {:.1} K",
            plain.temperature,
            open.temperature
        );
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
    fn a_crucible_half_way_through_a_calcination_still_solves() {
        // What one pass of a burner leaves: 0.052 mol of chalk beside
        // 0.048 mol of lime, standing in the same air. Calcium has no
        // gaseous carrier at all, so it has to be shared between two
        // solids — and the initial basis seeds exactly one of them, with
        // the whole calcium budget, which no amount of carbonate can hold
        // when there is only half as much carbon. Every temperature failed
        // with a singular matrix until the phase rescue existed.
        let b = budget(&[("C", 0.052207), ("Ca", 0.1), ("N", 1.248), ("O", 0.540414)]);
        let pool = pool_of(&["C(gr)", "CO", "CO2", "CaCO3(cr)", "CaO(cr)", "N2", "O2"]);
        let mut refused = Vec::new();
        for t in [400.0, 700.0, 1000.0, 1500.0] {
            match equilibrate_tp(&b, &pool, t, 1.0) {
                Ok(eq) => assert_conserved(&eq, &b, &format!("half-calcined chalk at {t} K")),
                Err(e) => refused.push(format!("{t} K: {e}")),
            }
        }
        assert!(
            refused.is_empty(),
            "a crucible half way through a calcination must still solve: {refused:?}"
        );
        // And the answer is chemistry, not merely arithmetic that closed:
        // cold, the carbon stays locked up as carbonate and the spare
        // calcium is lime; hot, nothing is left but lime and gas.
        let cold = equilibrate_tp(&b, &pool, 400.0, 1.0).expect("cold");
        assert!(
            (cold.moles_of("CaCO3(cr)") - 0.052207).abs() < 1e-4
                && (cold.moles_of("CaO(cr)") - 0.047793).abs() < 1e-4,
            "at 400 K both solids stand: {:?}",
            cold.composition
        );
        let hot = equilibrate_tp(&b, &pool, 1500.0, 1.0).expect("hot");
        assert!(
            hot.moles_of("CaCO3(cr)") < 1e-6 && (hot.moles_of("CaO(cr)") - 0.1).abs() < 1e-4,
            "at 1500 K the rest gives way too: {:?}",
            hot.composition
        );
    }

    #[test]
    fn rank_deficient_bare_chalk_returns_a_certified_condensed_state() {
        // Chalk alone, no atmosphere: one condensed phase holds every
        // element and the gas phase collapses, so the element-balance rows
        // become linearly dependent and the multipliers are
        // underdetermined. The boundary now has an independent global
        // Gibbs certificate rather than relying on the singular gas-mole
        // Newton equations or accepting whichever iterate stalled.
        let b = budget(&[("Ca", 0.0999), ("C", 0.0999), ("O", 0.2997)]);
        let pool = pool_of(&["CO2", "CO", "O2", "CaO(cr)", "CaCO3(cr)", "Ca(a)", "C(gr)"]);
        let eq = equilibrate_tp(&b, &pool, 800.0, 1.0).unwrap();
        assert_eq!(eq.gas_moles, 0.0);
        assert_eq!(eq.composition.len(), 1);
        assert!((eq.moles_of("CaCO3(cr)") - 0.0999).abs() < 1e-12);
        assert_conserved(&eq, &b, "certified rank-deficient chalk");
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

    #[test]
    fn assigned_enthalpy_resolves_the_condensed_latent_heat_interval() {
        let b = budget(&[("Mg", 0.05), ("O", 0.168), ("N", 0.624)]);
        let pool = pool_of(&["MgO(cr)", "MgO(L)", "O2", "N2"]);
        let t = 3100.0;
        let solid = crate::db().get("MgO(cr)").unwrap();
        let liquid = crate::db().get("MgO(L)").unwrap();
        let mismatch = (solid.g(t).unwrap() - liquid.g(t).unwrap()).abs();
        assert!(
            mismatch
                <= gibbs_coefficient_rounding(solid, t) + gibbs_coefficient_rounding(liquid, t)
        );
        let gas_h = 0.059 * crate::db().get("O2").unwrap().h(t).unwrap()
            + 0.312 * crate::db().get("N2").unwrap().h(t).unwrap();
        for liquid_fraction in [0.1, 0.5, 0.9] {
            let target = gas_h
                + 0.05
                    * (solid.h(t).unwrap() * (1.0 - liquid_fraction)
                        + liquid.h(t).unwrap() * liquid_fraction);
            let eq = equilibrate_hp(&b, &pool, target, 1.0).expect("latent-heat coexistence");
            assert!((eq.temperature - t).abs() < 1e-6);
            assert!((eq.enthalpy - target).abs() < 1e-6 + target.abs() * 1e-8);
            assert!((eq.moles_of("MgO(L)") - 0.05 * liquid_fraction).abs() < 1e-8);
            assert!((eq.moles_of("MgO(cr)") - 0.05 * (1.0 - liquid_fraction)).abs() < 1e-8);
            assert_conserved(&eq, &b, "latent heat coexistence");
        }
        for temperature in [3099.0, 3101.0] {
            let reference = equilibrate_tp(&b, &pool, temperature, 1.0).unwrap();
            let eq = equilibrate_hp(&b, &pool, reference.enthalpy, 1.0).unwrap();
            assert!((eq.temperature - temperature).abs() < 1e-4);
            assert!(eq.moles_of("MgO(cr)").min(eq.moles_of("MgO(L)")) < 1e-12);
        }
    }

    #[test]
    fn carbonate_reaction_coexistence_closes_independent_latent_energy() {
        let calcite = crate::db().get("CaCO3(cr)").unwrap();
        let lime = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        let gas_pool = [
            co2,
            crate::db().get("CO").unwrap(),
            crate::db().get("O2").unwrap(),
        ];
        let gas_budget = budget(&[("C", 0.1), ("O", 0.2)]);
        let products = |t| equilibrate_tp(&gas_budget, &gas_pool, t, 1.0).unwrap();
        // Find chemical equality using NASA standard G plus ideal-gas
        // partial pressure, independently of the HP/coexistence root.
        let mut lo = 900.0;
        let mut hi = 1400.0;
        for _ in 0..48 {
            let t = 0.5 * (lo + hi);
            let gas = products(t);
            let affinity = lime.g(t).unwrap()
                + co2.g(t).unwrap()
                + R * t * (gas.moles_of("CO2") / gas.gas_moles).ln()
                - calcite.g(t).unwrap();
            if affinity > 0.0 {
                lo = t;
            } else {
                hi = t;
            }
        }
        let t = 0.5 * (lo + hi);
        assert!((1000.0..1300.0).contains(&t));
        let gas = products(t);
        let pool = [calcite, lime, gas_pool[0], gas_pool[1], gas_pool[2]];
        let low = finish(&pool, &[0.1, 0.0, 0.0, 0.0, 0.0], t, 1.0);
        let high = finish(
            &pool,
            &[
                0.0,
                0.1,
                gas.moles_of("CO2"),
                gas.moles_of("CO"),
                gas.moles_of("O2"),
            ],
            t,
            1.0,
        );
        let b = budget(&[("Ca", 0.1), ("C", 0.1), ("O", 0.3)]);
        for fraction in [0.1, 0.5, 0.9] {
            let target = low.enthalpy * (1.0 - fraction) + high.enthalpy * fraction;
            let eq = condensed_coexistence(&b, &pool, target, None, &low, &high).unwrap();
            assert!((eq.moles_of("CaO(cr)") - 0.1 * fraction).abs() < 1e-8);
            assert!((eq.enthalpy - target).abs() < 1e-3);
            assert_conserved(&eq, &b, "carbonate chemical coexistence");
        }
        // A supplied pool may be wider than the element budget, exactly as
        // the TP API permits. Impossible silica is not a competing phase;
        // an explicit zero silicon budget is equivalent to absent silicon.
        let silica = crate::db().get("SiO2(b-qz)").unwrap();
        assert!(silica
            .t_range()
            .is_some_and(|(lo, hi)| (lo..=hi).contains(&t)));
        let mut wide = pool.to_vec();
        wide.push(silica);
        let target = 0.5 * (low.enthalpy + high.enthalpy);
        for zero_entry in [false, true] {
            let mut broad_budget = b.clone();
            if zero_entry {
                broad_budget.insert("Si".into(), 0.0);
            }
            let eq =
                condensed_coexistence(&broad_budget, &wide, target, None, &low, &high).unwrap();
            assert!((eq.moles_of("CaO(cr)") - 0.05).abs() < 1e-8);
            assert_eq!(eq.moles_of(&silica.name), 0.0);
        }
        let mut wrong = high.clone();
        wrong.temperature += 1.0;
        assert!(condensed_coexistence(
            &b,
            &pool,
            0.5 * (low.enthalpy + high.enthalpy),
            None,
            &low,
            &wrong
        )
        .is_err());
    }

    #[test]
    fn a_phase_record_boundary_without_equal_gibbs_is_not_a_coexistence_root() {
        let b = budget(&[("Mg", 0.05), ("O", 0.168), ("N", 0.624)]);
        let solid = crate::db().get("MgO(cr)").unwrap();
        let mut liquid = crate::db().get("MgO(L)").unwrap().clone();
        // Same composition and adjacent validity ranges are insufficient.
        // Raise the liquid's G by 100 J/mol without changing those ranges.
        liquid.intervals[0].coeffs[7] += 100.0 / R;
        let pool = [
            solid,
            &liquid,
            crate::db().get("O2").unwrap(),
            crate::db().get("N2").unwrap(),
        ];
        let low = finish(&pool, &[0.05, 0.0, 0.059, 0.312], 3100.0 - 1e-8, 1.0);
        let high = finish(&pool, &[0.0, 0.05, 0.059, 0.312], 3100.0 + 1e-8, 1.0);
        let target = 0.5 * (low.enthalpy + high.enthalpy);
        assert!(condensed_coexistence(&b, &pool, target, None, &low, &high).is_err());
    }
}
