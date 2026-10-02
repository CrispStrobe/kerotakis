//! Restricted CaCO3/CaO crucible, without an arbitrary entrained air mass.
//!
//! Ca lives in pure condensed phases. Released carbon is an ideal gas at
//! 1 bar, distributed over CO2/CO/O2. At fixed gas pressure the decomposition
//! affinity, G(CaO)+mu(CO2)-G(CaCO3), determines the stable endpoint. At
//! equality the HP common-potential/lever-rule certificate determines the
//! extent. This avoids the general Newton solver's all-condensed/incipient
//! gas degeneracy without pretending to fix that wider numerical problem.
use crate::{
    db,
    gibbs::{self, CeaError, Equilibrium},
    Species, R,
};
use std::collections::BTreeMap;

pub(crate) fn pool() -> Vec<&'static Species> {
    let mut pool: Vec<_> = db()
        .species
        .values()
        .filter(|s| {
            matches!(
                s.name.as_str(),
                "CaCO3(cr)" | "CaO(cr)" | "CaO(L)" | "CO2" | "CO" | "O2"
            )
        })
        .collect();
    pool.sort_by(|a, b| a.name.cmp(&b.name));
    pool
}

pub(crate) fn tp(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    t: f64,
) -> Result<Equilibrium, CeaError> {
    let fail = || CeaError::NotConverged(0);
    if !t.is_finite()
        || !(300.0..=6000.0).contains(&t)
        || budget
            .iter()
            .any(|(e, n)| !matches!(e.as_str(), "Ca" | "C" | "O") || !n.is_finite() || *n < 0.0)
    {
        return Err(CeaError::NoSpecies);
    }
    let calcium = budget.get("Ca").copied().unwrap_or(0.0);
    let carbon = budget.get("C").copied().unwrap_or(0.0);
    let oxygen = budget.get("O").copied().unwrap_or(0.0);
    if calcium <= 0.0
        || carbon < 0.0
        || (oxygen - calcium - 2.0 * carbon).abs() > oxygen.abs() * 1e-9 + 1e-15
    {
        return Err(fail());
    }
    let valid = |s: &&Species| s.t_range().is_some_and(|(lo, hi)| (lo..=hi).contains(&t));
    let oxide = candidates
        .iter()
        .copied()
        .filter(|s| matches!(s.name.as_str(), "CaO(cr)" | "CaO(L)"))
        .filter(valid)
        .min_by(|a, b| a.g(t).unwrap().total_cmp(&b.g(t).unwrap()))
        .ok_or(CeaError::NoSpecies)?;
    let carbonate = candidates
        .iter()
        .copied()
        .find(|s| s.name == "CaCO3(cr)" && valid(s));
    let gases: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|s| s.is_gas() && matches!(s.name.as_str(), "CO2" | "CO" | "O2"))
        .collect();
    let gas = carbon_gas(&gases, t)?;
    let co2 = gases
        .iter()
        .find(|s| s.name == "CO2")
        .ok_or(CeaError::NoSpecies)?;
    let mu_co2 = co2.g(t).ok_or_else(fail)? + R * t * (gas.moles_of("CO2") / gas.gas_moles).ln();
    let carbonate_amount = match carbonate {
        Some(s) if oxide.g(t).ok_or_else(fail)? + mu_co2 - s.g(t).ok_or_else(fail)? > 0.0 => {
            calcium.min(carbon)
        }
        _ => 0.0,
    };
    let released = carbon - carbonate_amount;
    let amounts: Vec<_> = candidates
        .iter()
        .map(|s| {
            if carbonate.is_some_and(|c| c.name == s.name) {
                carbonate_amount
            } else if s.name == oxide.name {
                calcium - carbonate_amount
            } else if s.is_gas() {
                released * gas.moles_of(&s.name)
            } else {
                0.0
            }
        })
        .collect();
    let eq = gibbs::finish(candidates, &amounts, t, 1.0);
    // Every construction preserves Ca and C analytically; gas minimisation
    // supplies O to its native tolerance. Verify all three explicitly.
    for (element, expected) in budget {
        let actual: f64 = candidates
            .iter()
            .zip(&amounts)
            .map(|(s, n)| s.composition.get(element).copied().unwrap_or(0.0) * n)
            .sum();
        if (actual - expected).abs() > expected.abs() * 1e-9 + 1e-15 {
            return Err(fail());
        }
    }
    Ok(eq)
}

/// Exact one-reaction ideal-gas mass action at 1 bar. For one carbon mol,
/// n(CO2)=1-x, n(CO)=x, n(O2)=x/2; Kp=(pCO/pCO2)*sqrt(pO2).
/// Log-x bracketing remains stable when the low-temperature dissociation
/// is far below the general Newton solver's trace floor.
fn carbon_gas(pool: &[&Species], t: f64) -> Result<Equilibrium, CeaError> {
    let find = |name: &str| {
        pool.iter()
            .copied()
            .find(|s| s.name == name && s.t_range().is_some_and(|(lo, hi)| (lo..=hi).contains(&t)))
            .ok_or(CeaError::NoSpecies)
    };
    let co2 = find("CO2")?;
    let co = find("CO")?;
    let oxygen = find("O2")?;
    let log_k = -(co.g(t).ok_or(CeaError::NoSpecies)?
        + 0.5 * oxygen.g(t).ok_or(CeaError::NoSpecies)?
        - co2.g(t).ok_or(CeaError::NoSpecies)?)
        / (R * t);
    let (mut lo, mut hi) = (-745.0, 0.0);
    for _ in 0..96 {
        let mid = 0.5 * (lo + hi);
        let x = f64::exp(mid);
        let log_q = 1.5 * mid - (-x).ln_1p() - 0.5 * (0.5 * x).ln_1p() + 0.5 * 0.5_f64.ln();
        if log_q < log_k {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let x = f64::exp(0.5 * (lo + hi));
    let solved = [co2, co, oxygen];
    Ok(gibbs::finish(&solved, &[1.0 - x, x, 0.5 * x], t, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analytic_dissociation_agrees_with_independent_gibbs_minimisation() {
        let pool = [
            db().get("CO2").unwrap(),
            db().get("CO").unwrap(),
            db().get("O2").unwrap(),
        ];
        let budget = BTreeMap::from([("C".into(), 1.0), ("O".into(), 2.0)]);
        for t in [1000.0, 1400.0, 1800.0, 2500.0] {
            let exact = carbon_gas(&pool, t).unwrap();
            let native = gibbs::equilibrate_tp(&budget, &pool, t, 1.0).unwrap();
            for name in ["CO2", "CO", "O2"] {
                assert!(
                    (exact.moles_of(name) - native.moles_of(name)).abs() < 1e-8,
                    "{t} K {name}: {exact:?} versus {native:?}"
                );
            }
            assert!((exact.enthalpy - native.enthalpy).abs() < 1e-3);
        }
        // Cold stability does not require a guessed zero dissociation.
        let cold = carbon_gas(&pool, 400.0).unwrap();
        assert!(cold.enthalpy.is_finite());
        assert!((cold.moles_of("CO2") - 1.0).abs() < 1e-12);
    }

    #[test]
    fn carbonate_tp_refuses_inventory_outside_its_reaction_subspace() {
        let pool = pool();
        for oxygen in [0.29, 0.31] {
            assert!(tp(
                &BTreeMap::from([("Ca".into(), 0.1), ("C".into(), 0.1), ("O".into(), oxygen)]),
                &pool,
                1400.0
            )
            .is_err());
        }
    }
}
