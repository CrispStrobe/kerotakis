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
    //
    // A condensed phase additionally exists in this solve only where its
    // data does. `interval_for` clamps to the nearest interval rather than
    // refusing — right for reading a feed enthalpy near its range edge,
    // catastrophically wrong for judging phase stability: the liquid-water
    // polynomial extrapolated to 3125 K says liquid is the stable phase of
    // steam, and a hydrogen flame then "converges" onto boiling-hot
    // H2O(L) or, worse, never converges at all (curiosity th-034). Gases
    // keep their historical clamped treatment: their records span the
    // whole working range, and a pool that loses its only carrier of an
    // element would turn a data gap into a silent element sink.
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
    let is_sole_carrier = |c: usize, active: &[usize]| -> bool {
        (0..elements.len()).any(|j| {
            a(c, j) > 0.0 && !gas_carries[j] && !active.iter().any(|&o| o != c && a(o, j) > 0.0)
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
            // Singular. Two distinct situations land here.
            //
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
        for (c_idx, &c) in active_cond.iter().enumerate() {
            let dn = m_flat[(nel + c_idx) * stride + dim];
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
            let dn = lambda * m_flat[(nel + c_idx) * stride + dim];
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
    // stiff side. This extends the existing convention — "colder than the
    // data supports; honest floor" — from data range to convergence range.
    let dbg = std::env::var("KERO_CEA_DEBUG").is_ok();
    let (mut lo, mut hi) = (250.0f64, 6000.0f64);
    let mut last = loop {
        match equilibrate_tp(budget, candidates, lo, pressure_bar) {
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
        return Ok(last); // colder than the data supports; honest floor
    }
    let mut failed_mids = 0u8;
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        match equilibrate_tp(budget, candidates, mid, pressure_bar) {
            Ok(eq) => {
                if dbg {
                    eprintln!("HP mid {mid:.0} K ok, H={:.3e}", eq.enthalpy);
                }
                last = eq;
                if last.enthalpy - atmosphere_credit(atmosphere, &last) < enthalpy {
                    lo = mid;
                } else {
                    hi = mid;
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
        if hi - lo < 0.5 {
            break;
        }
    }
    Ok(last)
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
