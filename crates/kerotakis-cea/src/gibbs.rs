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

/// Select independent atom constraints, retaining the original symbols as
/// components. This removes a multiplier gauge, not a chemical species or
/// a conservation law: e.g. a CO2-only pool always has O=2C. Verify the
/// identical relation in the supplied budget before dropping its row.
/// See NASA RP-1311 I §3.6, "Singularities".
fn independent_components(
    pool: &[&Species],
    elements: &[String],
    budget: &BTreeMap<String, f64>,
) -> Option<Vec<usize>> {
    // Establish rank exactly for the integer atomic compositions used by
    // these NASA records. A nearly dependent custom fractional pool must
    // retain the legacy equations rather than lose a real constraint.
    let mut integer_rows: Vec<(usize, Vec<i128>)> = Vec::new();
    let mut exact_basis = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let mut row = Vec::new();
        for species in pool {
            let value = species.composition.get(element).copied().unwrap_or(0.0);
            if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > 1e6 {
                return Some((0..elements.len()).collect());
            }
            row.push(value as i128);
        }
        for (pivot, prior) in &integer_rows {
            let multiplier = row[*pivot];
            if multiplier == 0 {
                continue;
            }
            let divisor = prior[*pivot];
            for (v, previous) in row.iter_mut().zip(prior) {
                let Some(next) = v.checked_mul(divisor).and_then(|left| {
                    multiplier
                        .checked_mul(*previous)
                        .and_then(|right| left.checked_sub(right))
                }) else {
                    // Exact rank unavailable at this size; do not guess.
                    return Some((0..elements.len()).collect());
                };
                if next == i128::MIN {
                    return Some((0..elements.len()).collect());
                }
                *v = next;
            }
            let gcd = row.iter().fold(0i128, |mut a, v| {
                let mut b = v.abs();
                while b != 0 {
                    (a, b) = (b, a % b);
                }
                a
            });
            if gcd > 1 {
                for v in &mut row {
                    *v /= gcd;
                }
            }
        }
        if let Some(pivot) = row.iter().position(|v| *v != 0) {
            integer_rows.push((pivot, row));
            exact_basis.push(index);
        }
    }
    if exact_basis.len() == elements.len() {
        return Some(exact_basis);
    }
    let mut basis = Vec::new();
    let mut orthogonal: Vec<(Vec<f64>, f64)> = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let original: Vec<f64> = pool
            .iter()
            .map(|s| s.composition.get(element).copied().unwrap_or(0.0))
            .collect();
        let norm = original.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !norm.is_finite() || norm == 0.0 {
            return None;
        }
        let mut row: Vec<f64> = original.iter().map(|v| v / norm).collect();
        let target = budget[element] / norm;
        let mut remaining = target;
        let mut scale = target.abs();
        // Reorthogonalize once to distinguish true rank loss from roundoff.
        for _ in 0..2 {
            for (q, q_target) in &orthogonal {
                let projection: f64 = row.iter().zip(q).map(|(a, b)| a * b).sum();
                for (v, qv) in row.iter_mut().zip(q) {
                    *v -= projection * qv;
                }
                remaining -= projection * q_target;
                scale += (projection * q_target).abs();
            }
        }
        let residual_norm = row.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !exact_basis.contains(&index) {
            // No absolute mole floor: even tiny incompatible inventories
            // must refuse. The scale covers cancellation in the relation.
            if residual_norm > 1e-12 || !remaining.is_finite() || remaining.abs() > 1e-12 * scale {
                return None;
            }
        } else {
            if residual_norm <= 1e-12 {
                return None;
            }
            for v in &mut row {
                *v /= residual_norm;
            }
            let q_target = remaining / residual_norm;
            if !q_target.is_finite() {
                return None;
            }
            orthogonal.push((row, q_target));
            basis.push(index);
        }
    }
    Some(basis)
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
    // Missing an element carrier is a source-domain boundary, not Newton
    // stiffness. HP must lower its upper bracket here; component rank
    // preflight must preserve the existing NoSpecies classification.
    if elements.iter().any(|element| {
        !pool
            .iter()
            .any(|species| species.composition.get(element).copied().unwrap_or(0.0) > 0.0)
    }) {
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

    let original_elements = elements;
    let component_indices = independent_components(&pool, &original_elements, budget)
        .ok_or(CeaError::NotConverged(0))?;
    let reduced = component_indices.len() != original_elements.len();
    let elements: Vec<String> = component_indices
        .into_iter()
        .map(|i| original_elements[i].clone())
        .collect();

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
            certified_condensed_boundary(&pool, &mu0, &original_elements, budget, &n, ln_p)
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
            // With one represented gas, simultaneous condensed phases
            // can overdetermine its fixed chemical potential. At reaction
            // coexistence the same singularity is a free extent. Recover
            // only an independently certified feasible endpoint; no failed
            // Newton state or guessed phase mixture is accepted.
            if let Some(endpoint) = certified_single_gas_endpoint(
                &pool,
                &mu0,
                &original_elements,
                budget,
                t,
                pressure_bar,
            ) {
                return Ok(endpoint);
            }
            if let Some(equilibrium) =
                certified_mixed_gas_line(&pool, &mu0, &original_elements, budget, t, pressure_bar)
            {
                return Ok(equilibrium);
            }
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
        let mut residual = balance_residual(&pool, &n, &original_elements, budget);
        if reduced {
            // Readback must obey every original constraint at its own
            // inventory scale, including redundant tiny element budgets.
            for element in &original_elements {
                let target = budget[element];
                let have: f64 = pool
                    .iter()
                    .zip(&n)
                    .map(|(s, amount)| s.composition.get(element).copied().unwrap_or(0.0) * amount)
                    .sum();
                residual = residual.max((have - target).abs() / target);
            }
        }

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

/// Bounded positive-gas endpoint certificate. With exactly one ideal gas,
/// its chemical potential is independent of amount at fixed T/P. Each
/// candidate contains that gas and at most one condensed phase, giving
/// an O(species*elements²) enumeration, not a combinatorial phase search.
/// Occupied-phase equalities and every inactive condensed inequality
/// provide the same global Gibbs lower bound as the zero-gas certificate.
fn certified_single_gas_endpoint(
    pool: &[&Species],
    mu0: &[f64],
    elements: &[String],
    budget: &BTreeMap<String, f64>,
    t: f64,
    pressure_bar: f64,
) -> Option<Equilibrium> {
    let gases: Vec<_> = (0..pool.len()).filter(|i| pool[*i].is_gas()).collect();
    if gases.len() != 1
        || !pool.iter().all(|species| {
            species
                .intervals
                .iter()
                .any(|interval| (interval.t_min..=interval.t_max).contains(&t))
        })
    {
        return None;
    }
    let gas = gases[0];
    let a = |i: usize, j: usize| {
        pool[i]
            .composition
            .get(&elements[j])
            .copied()
            .unwrap_or(0.0)
    };
    let chemical = |i: usize| {
        mu0[i]
            + if i == gas {
                (pressure_bar / P_STANDARD_BAR).ln()
            } else {
                0.0
            }
    };
    let mut candidates = Vec::new();
    // The gas-only endpoint covers proportional gas/solid compositions.
    let amount = (0..elements.len())
        .find(|j| a(gas, *j) > 0.0)
        .map(|j| budget[&elements[j]] / a(gas, j))?;
    let mut gas_only = vec![0.0; pool.len()];
    gas_only[gas] = amount;
    candidates.push(gas_only);
    for condensed in (0..pool.len()).filter(|i| !pool[*i].is_gas()) {
        let mut endpoint = None;
        'pair: for j in 0..elements.len() {
            for k in (j + 1)..elements.len() {
                let determinant = a(gas, j) * a(condensed, k) - a(gas, k) * a(condensed, j);
                if determinant == 0.0 || !determinant.is_finite() {
                    continue;
                }
                let gas_amount = (budget[&elements[j]] * a(condensed, k)
                    - budget[&elements[k]] * a(condensed, j))
                    / determinant;
                let condensed_amount = (a(gas, j) * budget[&elements[k]]
                    - a(gas, k) * budget[&elements[j]])
                    / determinant;
                let mut n = vec![0.0; pool.len()];
                n[gas] = gas_amount;
                n[condensed] = condensed_amount;
                endpoint = Some(n);
                break 'pair;
            }
        }
        if let Some(n) = endpoint {
            candidates.push(n);
        }
    }
    let dot = |left: &[f64], right: &[f64]| left.iter().zip(right).map(|(a, b)| a * b).sum::<f64>();
    let tol = 1e-8; // Unchanged dimensionless Newton affinity tolerance.
    for n in candidates {
        if n[gas] <= 0.0
            || n.iter().any(|amount| !amount.is_finite() || *amount < 0.0)
            || !n.iter().sum::<f64>().is_finite()
            || !elements.iter().enumerate().all(|(j, element)| {
                let have: f64 = n
                    .iter()
                    .enumerate()
                    .map(|(i, amount)| a(i, j) * amount)
                    .sum();
                have.is_finite() && (have - budget[element]).abs() <= BALANCE_TOL * budget[element]
            })
        {
            continue;
        }
        // Construct one dual potential satisfying every occupied equality.
        // Any remaining multiplier gauge is set to zero; failing an
        // inactive inequality declines this candidate rather than guessing.
        let mut basis: Vec<(Vec<f64>, f64)> = Vec::new();
        let mut valid = true;
        for i in (0..pool.len()).filter(|i| n[*i] > 0.0) {
            let mut row: Vec<_> = (0..elements.len()).map(|j| a(i, j)).collect();
            let mut rhs = chemical(i);
            for (q, value) in &basis {
                let projection = dot(&row, q);
                for (v, qv) in row.iter_mut().zip(q) {
                    *v -= projection * qv;
                }
                rhs -= projection * value;
            }
            let norm = dot(&row, &row).sqrt();
            if norm < 1e-12 {
                if rhs.abs() > tol {
                    valid = false;
                    break;
                }
            } else {
                for v in &mut row {
                    *v /= norm;
                }
                basis.push((row, rhs / norm));
            }
        }
        if !valid {
            continue;
        }
        let mut pi = vec![0.0; elements.len()];
        for (q, value) in basis {
            for (p, qv) in pi.iter_mut().zip(q) {
                *p += qv * value;
            }
        }
        if !pi.iter().all(|p| p.is_finite())
            || !(0..pool.len()).all(|i| {
                let potential: f64 = (0..elements.len()).map(|j| a(i, j) * pi[j]).sum();
                let difference = potential - chemical(i);
                difference.is_finite()
                    && if n[i] > 0.0 {
                        difference.abs() <= tol
                    } else {
                        difference <= tol
                    }
            })
        {
            continue;
        }
        let eq = finish(pool, &n, t, pressure_bar);
        if eq.enthalpy.is_finite() && eq.gas_moles.is_finite() && eq.gas_moles > 0.0 {
            return Some(eq);
        }
    }
    None
}

/// A bounded two-gas/two-condensed pool with one affine reaction degree.
/// Stoichiometric elimination is independent of gas inventory: an inert
/// stock below TRACE remains a component rather than a vanishing weighted
/// Newton pivot. Ideal-mixture G is convex on the feasible inventory line.
/// Only an independent global dual certificate can authorize readback.
fn certified_mixed_gas_line(
    pool: &[&Species],
    mu0: &[f64],
    elements: &[String],
    budget: &BTreeMap<String, f64>,
    t: f64,
    pressure_bar: f64,
) -> Option<Equilibrium> {
    if pool.len() != 4
        || pool.iter().filter(|s| s.is_gas()).count() != 2
        || !t.is_finite()
        || !pressure_bar.is_finite()
        || pressure_bar <= 0.0
        || !pool.iter().all(|s| {
            s.intervals.iter().any(|i| (i.t_min..=i.t_max).contains(&t))
                && s.composition
                    .values()
                    .all(|a| a.is_finite() && *a >= 0.0 && a.fract() == 0.0 && *a <= 1e6)
        })
    {
        return None;
    }
    let components = independent_components(pool, elements, budget)?;
    if components.len() != 3 {
        return None;
    }
    let scale: f64 = budget.values().sum();
    if !scale.is_finite() || scale <= 0.0 || budget.values().any(|n| !n.is_finite() || *n < 0.0) {
        return None;
    }
    let a = |i: usize, j: usize| {
        pool[i]
            .composition
            .get(&elements[j])
            .copied()
            .unwrap_or(0.0)
    };
    // RREF of atom constraints in inventory-normalized coordinates. These
    // pivots are stoichiometric coefficients, never tiny gas amounts.
    let mut matrix = [[0.0; 5]; 3];
    for (row, &component) in components.iter().enumerate() {
        for i in 0..4 {
            matrix[row][i] = a(i, component);
        }
        matrix[row][4] = budget[&elements[component]] / scale;
    }
    let mut pivots = Vec::new();
    for column in 0..4 {
        let row = pivots.len();
        if row == 3 {
            break;
        }
        let pivot = (row..3).max_by(|left, right| {
            matrix[*left][column]
                .abs()
                .total_cmp(&matrix[*right][column].abs())
        })?;
        if matrix[pivot][column].abs() < 1e-14 {
            continue;
        }
        matrix.swap(row, pivot);
        let divisor = matrix[row][column];
        for c in column..5 {
            matrix[row][c] /= divisor;
        }
        for r in 0..3 {
            if r != row {
                let multiplier = matrix[r][column];
                for c in column..5 {
                    matrix[r][c] -= multiplier * matrix[row][c];
                }
            }
        }
        pivots.push(column);
    }
    if pivots.len() != 3 {
        return None;
    }
    let free = (0..4).find(|i| !pivots.contains(i))?;
    let mut offset = [0.0; 4];
    let mut direction = [0.0; 4];
    direction[free] = 1.0;
    for (row, &column) in pivots.iter().enumerate() {
        offset[column] = matrix[row][4];
        direction[column] = -matrix[row][free];
    }
    let (mut lower, mut upper) = (f64::NEG_INFINITY, f64::INFINITY);
    for i in 0..4 {
        if !offset[i].is_finite() || !direction[i].is_finite() {
            return None;
        }
        if direction[i] > 0.0 {
            lower = lower.max(-offset[i] / direction[i]);
        } else if direction[i] < 0.0 {
            upper = upper.min(-offset[i] / direction[i]);
        } else if offset[i] < 0.0 {
            return None;
        }
    }
    if !lower.is_finite() || !upper.is_finite() || lower >= upper {
        return None;
    }
    let endpoint = |x| {
        std::array::from_fn::<_, 4, _>(|i| {
            // A phase defining this exact algebraic bound is zero. This
            // avoids subtraction roundoff without clipping any inventory.
            if direction[i] != 0.0 && x == -offset[i] / direction[i] {
                0.0
            } else {
                offset[i] + x * direction[i]
            }
        })
    };
    let left = endpoint(lower);
    let right = endpoint(upper);
    if left
        .iter()
        .chain(&right)
        .any(|n| !n.is_finite() || *n < 0.0)
        || (0..4).any(|i| pool[i].is_gas() && (left[i] <= 0.0 || right[i] <= 0.0))
    {
        // Absent-gas tangent boundaries belong to the separate certificate;
        // this slice requires both gases throughout its closed interval.
        return None;
    }
    let amounts = |fraction: f64| {
        std::array::from_fn::<_, 4, _>(|i| (1.0 - fraction) * left[i] + fraction * right[i])
    };
    let chemical = |i: usize, n: &[f64; 4], gas_total: f64| {
        mu0[i]
            + if pool[i].is_gas() {
                n[i].ln() - gas_total.ln() + (pressure_bar / P_STANDARD_BAR).ln()
            } else {
                0.0
            }
    };
    let derivative = |fraction| {
        let n = amounts(fraction);
        let total: f64 = (0..4).filter(|i| pool[*i].is_gas()).map(|i| n[i]).sum();
        (0..4)
            .map(|i| (right[i] - left[i]) * chemical(i, &n, total))
            .sum::<f64>()
    };
    let fraction = if derivative(0.0) >= 0.0 {
        0.0
    } else if derivative(1.0) <= 0.0 {
        1.0
    } else {
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..80 {
            let mid = (lo + hi) / 2.0;
            let slope = derivative(mid);
            if !slope.is_finite() {
                return None;
            }
            if slope < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) / 2.0
    };
    let normalized = amounts(fraction);
    let normalized_gas: f64 = (0..4)
        .filter(|i| pool[*i].is_gas())
        .map(|i| normalized[i])
        .sum();
    let n: Vec<_> = normalized.iter().map(|n| n * scale).collect();
    if n.iter().any(|n| !n.is_finite() || *n < 0.0)
        || !n.iter().sum::<f64>().is_finite()
        || !(0..elements.len()).all(|j| {
            let have: f64 = (0..4).map(|i| a(i, j) * n[i]).sum();
            have.is_finite()
                && (have - budget[&elements[j]]).abs() <= BALANCE_TOL * budget[&elements[j]]
        })
    {
        return None;
    }
    // Global Gibbs lower-bound proof: every occupied phase has mu=a.pi,
    // inactive condensed phases have mu>=a.pi, and both ideal-gas
    // activities are the actual inventory fractions. This separately
    // validates the one-dimensional optimizer and every original atom row.
    let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    let tol = 1e-8;
    let mut basis: Vec<(Vec<f64>, f64)> = Vec::new();
    for i in (0..4).filter(|i| n[*i] > 0.0) {
        let mut row: Vec<_> = (0..elements.len()).map(|j| a(i, j)).collect();
        let mut rhs = chemical(i, &normalized, normalized_gas);
        for (q, target) in &basis {
            let projection = dot(&row, q);
            for (v, qv) in row.iter_mut().zip(q) {
                *v -= projection * qv;
            }
            rhs -= projection * target;
        }
        let norm = dot(&row, &row).sqrt();
        if norm < 1e-12 {
            if rhs.abs() > tol {
                return None;
            }
        } else {
            for v in &mut row {
                *v /= norm;
            }
            basis.push((row, rhs / norm));
        }
    }
    let mut pi = vec![0.0; elements.len()];
    for (row, target) in basis {
        for (p, a) in pi.iter_mut().zip(row) {
            *p += a * target;
        }
    }
    if !pi.iter().all(|p| p.is_finite())
        || !(0..4).all(|i| {
            let difference = (0..elements.len()).map(|j| a(i, j) * pi[j]).sum::<f64>()
                - chemical(i, &normalized, normalized_gas);
            difference.is_finite()
                && if n[i] > 0.0 {
                    difference.abs() <= tol
                } else {
                    difference <= tol
                }
        })
    {
        return None;
    }
    let eq = finish(pool, &n, t, pressure_bar);
    (eq.enthalpy.is_finite() && eq.gas_moles.is_finite() && eq.gas_moles > 0.0).then_some(eq)
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
    fn redundant_components_allow_unique_positive_gas_inventories() {
        let co2 = crate::db().get("CO2").unwrap();
        let ethanol = crate::db().get("C2H5OH").unwrap();
        for (species, amount) in [(co2, 0.07), (ethanol, 0.013), (co2, 1e-10)] {
            let b: BTreeMap<_, _> = species
                .composition
                .iter()
                .map(|(element, atoms)| (element.clone(), atoms * amount))
                .collect();
            for t in [400.0, 1200.0] {
                for pressure in [0.01, 1.0, 100.0] {
                    // The represented pool has exactly one gas: conservation
                    // fixes its amount independently of pressure or G.
                    let eq = equilibrate_tp(&b, &[species], t, pressure).unwrap();
                    assert!((eq.gas_moles / amount - 1.0).abs() < BALANCE_TOL);
                    assert!((eq.moles_of(&species.name) / amount - 1.0).abs() < BALANCE_TOL);
                    assert!(
                        (eq.enthalpy / (amount * species.h(t).unwrap()) - 1.0).abs() < BALANCE_TOL
                    );
                    assert_conserved(&eq, &b, "unique gas inventory");
                }
            }
        }
    }

    #[test]
    fn redundant_components_preserve_binary_gas_inventory_and_pool_order() {
        let co2 = crate::db().get("CO2").unwrap();
        let n2 = crate::db().get("N2").unwrap();
        for scale in [1e-6, 1.0, 1e6] {
            let b = budget(&[
                ("C", 0.07 * scale),
                ("N", 0.24 * scale),
                ("O", 0.14 * scale),
            ]);
            for pool in [[co2, n2], [n2, co2]] {
                let eq = equilibrate_tp(&b, &pool, 800.0, 3.0).unwrap();
                assert!((eq.moles_of("CO2") / (0.07 * scale) - 1.0).abs() < BALANCE_TOL);
                assert!((eq.moles_of("N2") / (0.12 * scale) - 1.0).abs() < BALANCE_TOL);
                assert!((eq.gas_moles / (0.19 * scale) - 1.0).abs() < BALANCE_TOL);
                let h = scale * (0.07 * co2.h(800.0).unwrap() + 0.12 * n2.h(800.0).unwrap());
                assert!((eq.enthalpy / h - 1.0).abs() < BALANCE_TOL);
                assert_conserved(&eq, &b, "binary gas inventory");
            }
        }
    }

    #[test]
    fn redundant_components_reject_incompatible_budgets_at_every_scale() {
        let pool = [crate::db().get("CO2").unwrap()];
        let elements = vec!["C".into(), "O".into()];
        for scale in [1e-30, 1e-10, 1.0, 1e20] {
            let compatible = budget(&[("C", scale), ("O", 2.0 * scale)]);
            assert_eq!(
                independent_components(&pool, &elements, &compatible),
                Some(vec![0])
            );
            let incompatible = budget(&[("C", scale), ("O", 2.001 * scale)]);
            assert!(independent_components(&pool, &elements, &incompatible).is_none());
            assert!(matches!(
                equilibrate_tp(&incompatible, &pool, 800.0, 1.0),
                Err(CeaError::NotConverged(0))
            ));
        }
        let mut nearly_same = pool[0].clone();
        nearly_same.composition.insert("O".into(), 2.0 + 1e-13);
        // A real, however small, extra composition direction is never
        // declared redundant by a floating-point rank threshold.
        assert_eq!(
            independent_components(
                &[pool[0], &nearly_same],
                &elements,
                &budget(&[("C", 1.0), ("O", 2.0)])
            ),
            Some(vec![0, 1])
        );
        assert_eq!(
            independent_components(
                &pool,
                &["O".into(), "C".into()],
                &budget(&[("C", 1.0), ("O", 2.0)])
            ),
            Some(vec![0])
        );
    }

    #[test]
    fn absent_element_carrier_remains_a_representability_ceiling() {
        let (b, pool) = chalk_in_air();
        // Above these represented condensed calcium fits the surviving
        // C/O/N gases cannot carry Ca. HP must search downward, not treat
        // this refusal as a low-temperature Newton singularity.
        assert!(matches!(
            equilibrate_tp(&b, &pool, 3200.0, 1.0),
            Err(CeaError::NoSpecies)
        ));
        assert!(matches!(
            equilibrate_tp(
                &budget(&[("C", 0.1), ("N", 0.1)]),
                &[
                    crate::db().get("CO2").unwrap(),
                    crate::db().get("N2").unwrap()
                ],
                800.0,
                1.0
            ),
            Err(CeaError::NoSpecies)
        ));
    }

    #[test]
    fn positive_gas_condensed_competition_selects_the_nasa_stable_endpoint() {
        let carbonate = crate::db().get("CaCO3(cr)").unwrap();
        let oxide = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        for scale in [1e-6, 1.0, 1e6] {
            let b = budget(&[("Ca", 0.1 * scale), ("C", 0.2 * scale), ("O", 0.5 * scale)]);
            for pool in [[carbonate, oxide, co2], [co2, oxide, carbonate]] {
                for t in [800.0, 1300.0, 1500.0] {
                    for pressure in [0.1_f64, 1.0, 10.0] {
                        let reaction_g =
                            oxide.g(t).unwrap() + co2.g(t).unwrap() + R * t * pressure.ln()
                                - carbonate.g(t).unwrap();
                        let converted = reaction_g < 0.0;
                        let eq = equilibrate_tp(&b, &pool, t, pressure).unwrap();
                        let calcium_phase = if converted { "CaO(cr)" } else { "CaCO3(cr)" };
                        let gas_amount = if converted { 0.2 } else { 0.1 } * scale;
                        // Excess carbon makes gas compulsory. At fixed P,
                        // pure CO2 has mu=G°+RTlnP; reaction affinity decides
                        // the only stable endpoint independently of Newton.
                        assert!(
                            (eq.moles_of(calcium_phase) / (0.1 * scale) - 1.0).abs() < BALANCE_TOL
                        );
                        assert!((eq.gas_moles / gas_amount - 1.0).abs() < BALANCE_TOL);
                        assert_conserved(&eq, &b, "positive-gas carbonate endpoint");
                        let phase = if converted { oxide } else { carbonate };
                        let h = 0.1 * scale * phase.h(t).unwrap() + gas_amount * co2.h(t).unwrap();
                        assert!((eq.enthalpy / h - 1.0).abs() < BALANCE_TOL);
                    }
                }
            }
        }
    }

    #[test]
    fn positive_gas_coexistence_certifies_a_feasible_equal_g_endpoint() {
        let carbonate = crate::db().get("CaCO3(cr)").unwrap();
        let oxide = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        let pool = [carbonate, oxide, co2];
        let b = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.5)]);
        let (mut lo, mut hi) = (800.0, 1500.0);
        for _ in 0..60 {
            let t = (lo + hi) / 2.0;
            if oxide.g(t).unwrap() + co2.g(t).unwrap() > carbonate.g(t).unwrap() {
                lo = t;
            } else {
                hi = t;
            }
        }
        let t = (lo + hi) / 2.0;
        let mu: Vec<_> = pool.iter().map(|s| s.g(t).unwrap() / (R * t)).collect();
        let elements: Vec<_> = b.keys().cloned().collect();
        let eq = certified_single_gas_endpoint(&pool, &mu, &elements, &b, t, 1.0).unwrap();
        let unconverted_g = 0.1 * carbonate.g(t).unwrap() + 0.1 * co2.g(t).unwrap();
        let converted_g = 0.1 * oxide.g(t).unwrap() + 0.2 * co2.g(t).unwrap();
        assert!((converted_g - unconverted_g).abs() < 1e-7);
        let result_g = eq
            .composition
            .iter()
            .map(|(name, n)| crate::db().get(name).unwrap().g(t).unwrap() * n)
            .sum::<f64>();
        assert!((result_g - unconverted_g).abs() < 1e-7);
        assert!(eq.gas_moles >= 0.1 && eq.gas_moles <= 0.2);
        assert_conserved(&eq, &b, "equal-G positive-gas endpoint");
    }

    #[test]
    fn single_gas_certificate_declines_incomplete_or_invalid_domains() {
        let carbonate = crate::db().get("CaCO3(cr)").unwrap();
        let oxide = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        let b = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.5)]);
        let elements: Vec<_> = b.keys().cloned().collect();
        let certify = |pool: &[&Species], budget: &BTreeMap<String, f64>, t| {
            let mu: Vec<_> = pool.iter().map(|s| s.g(t).unwrap() / (R * t)).collect();
            certified_single_gas_endpoint(pool, &mu, &elements, budget, t, 1.0)
        };
        let pool = [carbonate, oxide, co2];
        // All represented competitors must be source-covered. Do not omit
        // carbonate above its fit ceiling to certify an incomplete pool.
        assert!(certify(&pool, &b, 1700.0).is_none());
        assert!(certify(
            &[carbonate, oxide, co2, crate::db().get("CO").unwrap()],
            &b,
            1300.0
        )
        .is_none());
        let wrong = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.51)]);
        assert!(certify(&pool, &wrong, 1300.0).is_none());
        let huge = budget(&[("Ca", 1e305), ("C", 2e305), ("O", 5e305)]);
        assert!(certify(&pool, &huge, 1300.0).is_none());
        // A pool containing only the thermodynamically disfavored endpoint
        // may be valid in its restricted domain. With the competing phase
        // present, the certificate must select its lower-G endpoint.
        let cold = certify(&pool, &b, 800.0).unwrap();
        let hot = certify(&pool, &b, 1300.0).unwrap();
        assert_eq!(cold.moles_of("CaO(cr)"), 0.0);
        assert_eq!(hot.moles_of("CaCO3(cr)"), 0.0);
    }

    #[test]
    fn certified_positive_gas_readback_preserves_inventory_below_trace_floor() {
        let co2 = crate::db().get("CO2").unwrap();
        let b = budget(&[("C", 1e-30), ("O", 2e-30)]);
        let eq = equilibrate_tp(&b, &[co2], 800.0, 1.0).unwrap();
        assert_eq!(eq.gas_moles, 1e-30);
        assert_eq!(eq.moles_of("CO2"), 1e-30);
        assert_eq!(eq.enthalpy, 1e-30 * co2.h(800.0).unwrap());
    }

    #[test]
    fn mixed_gas_condensed_equilibrium_retains_tiny_independent_inert_stock() {
        let pool = pool_of(&["CaCO3(cr)", "CaO(cr)", "CO2", "N2"]);
        let b = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.5), ("N", 2e-20)]);
        let eq = equilibrate_tp(&b, &pool, 1300.0, 1.0).unwrap();
        // NASA reaction affinity is strongly negative here, so lime is
        // the unique condensed endpoint. Independent inert atoms still
        // require their actual amount, even below the legacy gas floor.
        assert!((eq.moles_of("CaO(cr)") / 0.1 - 1.0).abs() < BALANCE_TOL);
        assert!((eq.moles_of("CO2") / 0.2 - 1.0).abs() < BALANCE_TOL);
        assert!((eq.moles_of("N2") / 1e-20 - 1.0).abs() < BALANCE_TOL);
        assert_eq!(eq.moles_of("CaCO3(cr)"), 0.0);
        assert_conserved(&eq, &b, "tiny independent inert inventory");
    }

    #[test]
    fn mixed_gas_condensed_coexistence_matches_mass_action_and_hp() {
        let carbonate = crate::db().get("CaCO3(cr)").unwrap();
        let oxide = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        let n2 = crate::db().get("N2").unwrap();
        let pool = [carbonate, oxide, co2, n2];
        let b = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.5), ("N", 0.4)]);
        let t = 1100.0;
        let reaction_g = oxide.g(t).unwrap() + co2.g(t).unwrap() - carbonate.g(t).unwrap();
        let co2_fraction = (-reaction_g / (R * t)).exp();
        let co2_amount = 0.2 * co2_fraction / (1.0 - co2_fraction);
        let oxide_amount = co2_amount - 0.1;
        let carbonate_amount = 0.1 - oxide_amount;
        assert!(oxide_amount > 0.0 && carbonate_amount > 0.0);
        let h = carbonate_amount * carbonate.h(t).unwrap()
            + oxide_amount * oxide.h(t).unwrap()
            + co2_amount * co2.h(t).unwrap()
            + 0.2 * n2.h(t).unwrap();
        for eq in [
            equilibrate_tp(&b, &pool, t, 1.0).unwrap(),
            equilibrate_hp(&b, &pool, h, 1.0).unwrap(),
        ] {
            assert!((eq.temperature - t).abs() < 1e-4);
            assert!((eq.moles_of("CaCO3(cr)") - carbonate_amount).abs() < 1e-7);
            assert!((eq.moles_of("CaO(cr)") - oxide_amount).abs() < 1e-7);
            assert!((eq.moles_of("CO2") - co2_amount).abs() < 1e-7);
            assert!((eq.moles_of("N2") / 0.2 - 1.0).abs() < BALANCE_TOL);
            assert!((eq.enthalpy - h).abs() < 1e-6 + h.abs() * 1e-8);
            assert_conserved(&eq, &b, "mixed-gas carbonate coexistence");
        }
    }

    #[test]
    fn mixed_gas_line_certificate_matches_activity_across_pressure_and_inventory_scales() {
        let carbonate = crate::db().get("CaCO3(cr)").unwrap();
        let oxide = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        let n2 = crate::db().get("N2").unwrap();
        for scale in [1e-6, 1.0, 1e6] {
            for inert in [1e-20, 0.2] {
                let b = budget(&[
                    ("Ca", 0.1 * scale),
                    ("C", 0.2 * scale),
                    ("O", 0.5 * scale),
                    ("N", 2.0 * inert * scale),
                ]);
                let elements: Vec<_> = b.keys().cloned().collect();
                for pool in [[carbonate, oxide, co2, n2], [n2, oxide, carbonate, co2]] {
                    for t in [800.0, 1100.0, 1300.0] {
                        let equilibrium_pressure = (-(oxide.g(t).unwrap() + co2.g(t).unwrap()
                            - carbonate.g(t).unwrap())
                            / (R * t))
                            .exp();
                        for pressure in [0.1_f64, 1.0, 10.0] {
                            // Independent mass action pCO2=Kp. Its inventory
                            // must also fit the carbonate/oxide endpoints.
                            let desired = if equilibrium_pressure < pressure {
                                inert * scale * equilibrium_pressure
                                    / (pressure - equilibrium_pressure)
                            } else {
                                f64::INFINITY
                            };
                            let gas_carbon = desired.max(0.1 * scale).min(0.2 * scale);
                            let lime = gas_carbon - 0.1 * scale;
                            let chalk = 0.1 * scale - lime;
                            let mu: Vec<_> =
                                pool.iter().map(|s| s.g(t).unwrap() / (R * t)).collect();
                            let eq =
                                certified_mixed_gas_line(&pool, &mu, &elements, &b, t, pressure)
                                    .unwrap();
                            assert!((eq.moles_of("CO2") - gas_carbon).abs() <= 1e-8 * scale);
                            assert!((eq.moles_of("CaO(cr)") - lime).abs() <= 1e-8 * scale);
                            assert!((eq.moles_of("CaCO3(cr)") - chalk).abs() <= 1e-8 * scale);
                            assert!(
                                (eq.moles_of("N2") / (inert * scale) - 1.0).abs() < BALANCE_TOL
                            );
                            let h = chalk * carbonate.h(t).unwrap()
                                + lime * oxide.h(t).unwrap()
                                + gas_carbon * co2.h(t).unwrap()
                                + inert * scale * n2.h(t).unwrap();
                            assert!((eq.enthalpy - h).abs() <= 1e-3 * scale);
                            assert_conserved(&eq, &b, "certified mixed-gas activity");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn mixed_gas_line_certificate_declines_outside_its_proven_domain() {
        let carbonate = crate::db().get("CaCO3(cr)").unwrap();
        let oxide = crate::db().get("CaO(cr)").unwrap();
        let co2 = crate::db().get("CO2").unwrap();
        let n2 = crate::db().get("N2").unwrap();
        let pool = [carbonate, oxide, co2, n2];
        let b = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.5), ("N", 0.4)]);
        let elements: Vec<_> = b.keys().cloned().collect();
        let certify = |candidates: &[&Species], budget: &BTreeMap<String, f64>, t, pressure| {
            let mu: Vec<_> = candidates
                .iter()
                .map(|s| s.g(t).unwrap() / (R * t))
                .collect();
            certified_mixed_gas_line(candidates, &mu, &elements, budget, t, pressure)
        };
        assert!(certify(&pool, &b, 1700.0, 1.0).is_none());
        assert!(certify(
            &[carbonate, oxide, co2, n2, crate::db().get("O2").unwrap()],
            &b,
            1100.0,
            1.0
        )
        .is_none());
        for pressure in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
            assert!(certify(&pool, &b, 1100.0, pressure).is_none());
        }
        let incompatible = budget(&[("Ca", 0.1), ("C", 0.2), ("O", 0.51), ("N", 0.4)]);
        assert!(certify(&pool, &incompatible, 1100.0, 1.0).is_none());
        // At this inventory endpoint CO2 can vanish: that tangent boundary
        // is outside the strictly positive two-gas slice, not silently
        // assigned an artificial trace amount.
        let absent_gas = budget(&[("Ca", 0.1), ("C", 0.1), ("O", 0.3), ("N", 0.4)]);
        assert!(certify(&pool, &absent_gas, 1100.0, 1.0).is_none());
        let overflowing_h = budget(&[("Ca", 1e305), ("C", 2e305), ("O", 5e305), ("N", 4e305)]);
        assert!(certify(&pool, &overflowing_h, 1100.0, 1.0).is_none());
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
