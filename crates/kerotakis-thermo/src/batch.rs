//! Multicomponent, ideal-liquid Rayleigh distillation. This is a bounded
//! teaching approximation, not an activity-coefficient or azeotrope model.
//! No substance identifiers or experiment recipes enter the numerical kernel.

use crate::vle::StillTake;

/// Constant-latent-heat Clausius–Clapeyron approximation anchored at 1 atm.
/// `valid_k` is an explicit approximation domain, not a fitted-data claim.
#[derive(Clone, Copy, Debug)]
pub struct ConstantLatent {
    pub boiling_k: f64,
    pub latent_kj_mol: f64,
    pub valid_k: (f64, f64),
}

impl ConstantLatent {
    fn valid(self) -> bool {
        self.valid_k.0.is_finite()
            && self.valid_k.1.is_finite()
            && self.valid_k.0 > 0.0
            && self.valid_k.0 <= self.valid_k.1
            && self.boiling_k.is_finite()
            && self.boiling_k > 0.0
            && self.latent_kj_mol.is_finite()
            && self.latent_kj_mol > 0.0
    }

    pub fn pressure_kpa(self, t: f64) -> Option<f64> {
        if !t.is_finite() || !self.valid() || t < self.valid_k.0 || t > self.valid_k.1 {
            return None;
        }
        let p = 101.325
            * (self.latent_kj_mol / 0.008_314_462_618 * (1.0 / self.boiling_k - 1.0 / t)).exp();
        (p.is_finite() && p > 0.0).then_some(p)
    }
}

#[derive(Debug)]
pub struct BatchCut {
    pub overhead: Vec<f64>,
    pub t_start_k: f64,
    pub t_end_k: f64,
    pub energy_kj: f64,
}

fn bubble(n: &[f64], models: &[ConstantLatent], pressure: f64) -> Option<(f64, Vec<f64>)> {
    let total: f64 = n.iter().sum();
    if !total.is_finite() || total <= 0.0 {
        return None;
    }
    let mut lo: f64 = 0.0;
    let mut hi = f64::INFINITY;
    for (amount, model) in n.iter().zip(models) {
        if *amount > 0.0 {
            lo = lo.max(model.valid_k.0);
            hi = hi.min(model.valid_k.1);
        }
    }
    if !hi.is_finite() || lo > hi {
        return None;
    }
    let partials = |t| -> Option<Vec<f64>> {
        n.iter()
            .zip(models)
            .map(|(amount, model)| {
                if *amount == 0.0 {
                    Some(0.0)
                } else {
                    let fraction = amount / total;
                    let partial = fraction * model.pressure_kpa(t)?;
                    // Normalization must not silently remove a positive
                    // volatile before the integration sees its budget.
                    (fraction > 0.0 && partial > 0.0 && partial.is_finite()).then_some(partial)
                }
            })
            .collect()
    };
    if partials(lo)?.iter().sum::<f64>() > pressure || partials(hi)?.iter().sum::<f64>() < pressure
    {
        return None;
    }
    for _ in 0..64 {
        let mid = 0.5 * (lo + hi);
        if partials(mid)?.iter().sum::<f64>() > pressure {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let t = 0.5 * (lo + hi);
    let mut y = partials(t)?;
    let sum: f64 = y.iter().sum();
    if !sum.is_finite() || sum <= 0.0 {
        return None;
    }
    for (amount, v) in n.iter().zip(&mut y) {
        *v /= sum;
        if !v.is_finite() || (*amount > 0.0 && *v <= 0.0) {
            return None;
        }
    }
    Some((t, y))
}

/// Integrate a molar or latent-energy cut through ideal stages at total reflux.
/// Every component limits every step; a domain failure rejects the whole cut.
/// Exactly pure stocks use one phase solve and a closed-form latent account.
/// Mixtures use a 1024-step amount mesh refined further near depletion.
pub fn ideal_still(
    inventory: &[f64],
    models: &[ConstantLatent],
    take: StillTake,
    stages: u32,
    pressure_kpa: f64,
) -> Option<BatchCut> {
    ideal_still_with_bubble(inventory, models, take, stages, pressure_kpa, bubble)
}

// Passing the evaluator explicitly gives unit tests a local cost counter;
// production uses the same bubble solver without shared instrumentation.
fn ideal_still_with_bubble(
    inventory: &[f64],
    models: &[ConstantLatent],
    take: StillTake,
    stages: u32,
    pressure_kpa: f64,
    mut evaluate_bubble: impl FnMut(&[f64], &[ConstantLatent], f64) -> Option<(f64, Vec<f64>)>,
) -> Option<BatchCut> {
    if inventory.is_empty()
        || inventory.len() != models.len()
        || inventory.iter().any(|n| !n.is_finite() || *n < 0.0)
        // Even inactive components must have valid properties: zero times
        // NaN/infinity would otherwise contaminate latent-energy accounting.
        || models.iter().any(|model| !model.valid())
        || !pressure_kpa.is_finite()
        || pressure_kpa <= 0.0
        || stages == 0
        || stages > 128
    {
        return None;
    }
    let total: f64 = inventory.iter().sum();
    if !total.is_finite() || total <= 0.0 {
        return None;
    }
    let budget = match take {
        StillTake::Fraction(f) if f.is_finite() && (0.0..=1.0).contains(&f) => {
            let amount = total * f;
            if f > 0.0 && amount == 0.0 {
                return None; // a positive requested cut underflowed
            }
            amount
        }
        StillTake::EnergyKj(e) if e.is_finite() && e >= 0.0 => total,
        _ => return None,
    };
    let (start, _) = evaluate_bubble(inventory, models, pressure_kpa)?;
    let mut cut = BatchCut {
        overhead: vec![0.0; inventory.len()],
        t_start_k: start,
        t_end_k: start,
        energy_kj: 0.0,
    };
    let mut active = inventory.iter().enumerate().filter(|(_, n)| **n > 0.0);
    let (active_index, amount) = active.next()?;
    if active.next().is_none() {
        // A pure liquid keeps its initial boiling point throughout the cut.
        // Still validate every model and solve that point before taking this
        // shortcut; inactive properties and pressure domains remain binding.
        let (overhead, energy) =
            crate::vle::pure_still_amount(*amount, models[active_index].latent_kj_mol, take)?;
        cut.overhead[active_index] = overhead;
        cut.energy_kj = energy;
        return Some(cut);
    }
    if budget == 0.0 || matches!(take, StillTake::EnergyKj(0.0)) {
        return Some(cut);
    }
    let inventory_heat: f64 = inventory
        .iter()
        .zip(models)
        .map(|(n, model)| n * model.latent_kj_mol)
        .sum();
    let full_take = matches!(take, StillTake::Fraction(1.0))
        || matches!(take, StillTake::EnergyKj(energy) if energy >= inventory_heat);
    let energy_target = match take {
        StillTake::EnergyKj(energy) => energy.min(inventory_heat),
        StillTake::Fraction(_) => 0.0,
    };
    let mut pot = inventory.to_vec();
    // Completion is relative to the request, never to the starting inventory
    // or an absolute one-kilojoule floor. Compare ratios to avoid an underflowed
    // tolerance for otherwise representable microscopic requests.
    for _ in 0..100_000 {
        let removed_total = cut.overhead.iter().sum::<f64>();
        let remaining = budget - removed_total;
        let complete = match take {
            StillTake::Fraction(_) => (remaining / budget).abs() <= 1e-12,
            StillTake::EnergyKj(_) => {
                ((energy_target - cut.energy_kj) / energy_target).abs() <= 1e-12
            }
        };
        if complete || (full_take && (remaining / budget).abs() <= 1e-12) {
            // A complete take has no residual bubble state. Publish its exact
            // inventory and latent heat, with the last approached boiling
            // temperature as the endpoint convention.
            if full_take {
                cut.overhead = inventory.to_vec();
                cut.energy_kj = inventory
                    .iter()
                    .zip(models)
                    .map(|(n, model)| n * model.latent_kj_mol)
                    .sum();
                return cut.energy_kj.is_finite().then_some(cut);
            }
            // Partial cuts cannot silently round a positive residual to zero.
            // A microscopic positive overhead may leave bulk stock unchanged
            // at its floating-point resolution; the overhead remains explicit.
            for (residual, removed) in pot.iter().zip(&cut.overhead) {
                if *removed > 0.0 && *residual <= 0.0 {
                    return None;
                }
            }
            if removed_total <= 0.0 || cut.energy_kj <= 0.0 {
                return None;
            }
            cut.t_end_k = evaluate_bubble(&pot, models, pressure_kpa)?.0;
            return Some(cut);
        }
        if remaining <= 0.0 || !remaining.is_finite() {
            return None;
        }
        let (temperature, mut y) = evaluate_bubble(&pot, models, pressure_kpa)?;
        cut.t_end_k = temperature;
        for _ in 1..stages {
            y = evaluate_bubble(&y, models, pressure_kpa)?.1;
        }
        let mut dn = remaining.min(budget / 1024.0);
        // Preserve microscopic budgets whose default mesh itself underflows.
        if dn == 0.0 {
            dn = remaining;
        }
        for (n, fraction) in pot.iter().zip(&y) {
            if *fraction > 0.0 {
                dn = dn.min(0.25 * n / fraction);
            }
        }
        let latent: f64 = y.iter().zip(models).map(|(v, m)| v * m.latent_kj_mol).sum();
        if !latent.is_finite() || latent <= 0.0 {
            return None;
        }
        if let StillTake::EnergyKj(energy) = take {
            let available = energy.min(energy_target) - cut.energy_kj;
            if available <= 0.0 || !available.is_finite() {
                return None;
            }
            dn = dn.min(available / latent);
        }
        if !dn.is_finite() || dn <= 0.0 {
            return None;
        }
        for i in 0..pot.len() {
            let mut removed = dn * y[i];
            if y[i] > 0.0 && removed == 0.0 {
                return None;
            }
            let mut overhead = cut.overhead[i] + removed;
            if removed > 0.0 && overhead == cut.overhead[i] && full_take {
                // A true full take owns the complete component, including a
                // final residual too small to increment its accumulated cut.
                // This terminal convention must never apply to partial cuts.
                removed = pot[i];
                overhead = inventory[i];
            }
            let heat = removed * models[i].latent_kj_mol;
            if !overhead.is_finite()
                || overhead > inventory[i]
                || (removed > 0.0 && (overhead == cut.overhead[i] || heat == 0.0))
                || !heat.is_finite()
            {
                return None;
            }
            cut.overhead[i] = overhead;
            // Accumulate the cut, then subtract once from the original pot.
            // Repeated subtraction loses small but collectively representable
            // cuts (for example a 1e-14 fraction split over 1024 steps).
            pot[i] = inventory[i] - overhead;
            cut.energy_kj += heat;
        }
        if !cut.energy_kj.is_finite() {
            return None;
        }
    }
    None // no partial success disguised as the requested cut
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pure_cut_uses_one_bubble_solve_and_mixtures_keep_integrating() {
        for take in [
            StillTake::Fraction(0.0),
            StillTake::Fraction(0.4),
            StillTake::Fraction(1.0),
            StillTake::EnergyKj(7.0),
            StillTake::EnergyKj(1e6),
        ] {
            for stages in [1, 4, 128] {
                let mut calls = 0;
                let cut = ideal_still_with_bubble(
                    &[0.0, 2.0, 0.0],
                    &[model(330.0), model(350.0), model(390.0)],
                    take,
                    stages,
                    101.325,
                    |n, models, pressure| {
                        calls += 1;
                        bubble(n, models, pressure)
                    },
                )
                .unwrap();
                assert_eq!(calls, 1, "take={take:?}, stages={stages}");
                assert_eq!(cut.t_start_k, cut.t_end_k);
            }
        }
        let mut calls = 0;
        ideal_still_with_bubble(
            &[1.0, 2.0],
            &[model(330.0), model(390.0)],
            StillTake::Fraction(0.2),
            2,
            101.325,
            |n, models, pressure| {
                calls += 1;
                bubble(n, models, pressure)
            },
        )
        .unwrap();
        assert!(calls > 2, "a mixture must retain stage and residual solves");
    }

    fn model(tb: f64) -> ConstantLatent {
        ConstantLatent {
            boiling_k: tb,
            latent_kj_mol: 40.0,
            valid_k: (280.0, 430.0),
        }
    }
    #[test]
    fn zero_inventory_cannot_hide_invalid_component_properties() {
        for latent in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
            let mut invalid = model(350.0);
            invalid.latent_kj_mol = latent;
            for take in [StillTake::Fraction(0.2), StillTake::EnergyKj(1.0)] {
                let result = ideal_still(&[1.0, 0.0], &[model(350.0), invalid], take, 1, 101.325);
                assert!(
                    result.is_none(),
                    "invalid latent={latent}, take={take:?}: {result:?}"
                );
            }
        }
    }

    #[test]
    fn finite_inventory_cannot_return_overflowed_heat() {
        let result = ideal_still(
            &[1e307],
            &[model(350.0)],
            StillTake::Fraction(0.5),
            1,
            101.325,
        );
        assert!(result.is_none(), "unrepresentable cut energy: {result:?}");
    }

    #[test]
    fn inactive_domains_and_large_representable_cuts_remain_supported() {
        let inactive = ConstantLatent {
            valid_k: (370.0, 390.0),
            ..model(380.0)
        };
        for amount in [1e-6, 1.0, 1e300] {
            let cut = ideal_still(
                &[amount, 0.0],
                &[model(350.0), inactive],
                StillTake::Fraction(0.5),
                1,
                101.325,
            )
            .expect("inactive valid model imposes no bubble-temperature domain");
            assert!((cut.overhead[0] / amount - 0.5).abs() < 1e-10);
            assert_eq!(cut.overhead[1], 0.0);
            assert!(cut.energy_kj.is_finite());
            assert!((cut.energy_kj / amount - 20.0).abs() < 1e-9);
        }
        for invalid in [
            ConstantLatent {
                boiling_k: f64::NAN,
                ..inactive
            },
            ConstantLatent {
                valid_k: (0.0, 390.0),
                ..inactive
            },
            ConstantLatent {
                valid_k: (390.0, 370.0),
                ..inactive
            },
            ConstantLatent {
                valid_k: (370.0, f64::INFINITY),
                ..inactive
            },
        ] {
            assert!(ideal_still(
                &[1.0, 0.0],
                &[model(350.0), invalid],
                StillTake::Fraction(0.5),
                1,
                101.325,
            )
            .is_none());
        }
    }

    #[test]
    fn constant_relative_volatility_obeys_independent_rayleigh_identity() {
        // Equal latent heats make relative volatility independent of T.
        // Integrating dn_light/dn_heavy = alpha*n_light/n_heavy gives
        // ln(n_light/n_light0) = alpha*ln(n_heavy/n_heavy0).
        let models = [model(340.0), model(370.0)];
        let alpha = (40.0_f64 / 0.008_314_462_618 * (1.0 / 340.0 - 1.0 / 370.0)).exp();
        for f in [0.01, 0.1, 0.4] {
            let cut =
                ideal_still(&[0.2, 4.0], &models, StillTake::Fraction(f), 1, 101.325).unwrap();
            let light = (0.2 - cut.overhead[0]) / 0.2;
            let heavy = (4.0 - cut.overhead[1]) / 4.0;
            assert!((light.ln() - alpha * heavy.ln()).abs() < 0.005);
        }
    }
    #[test]
    fn pure_limit_scale_permutation_energy_and_conservation() {
        let models = [model(370.0), model(340.0), model(355.0)];
        for scale in [1e-6, 0.01, 1.0, 1000.0] {
            for fraction in [0.0, 0.01, 0.3, 0.9, 1.0] {
                let input = [4.0 * scale, 0.2 * scale, 0.3 * scale];
                let c = ideal_still(&input, &models, StillTake::Fraction(fraction), 2, 101.325)
                    .unwrap();
                assert!((c.overhead.iter().sum::<f64>() / scale - 4.5 * fraction).abs() < 1e-9);
                for (i, n) in input.iter().enumerate() {
                    assert!(c.overhead[i] >= 0.0 && c.overhead[i] <= *n + scale * 1e-12);
                }
                assert!((c.energy_kj / scale - 40.0 * 4.5 * fraction).abs() < 1e-7);
                let p = ideal_still(
                    &[input[2], input[0], input[1]],
                    &[models[2], models[0], models[1]],
                    StillTake::Fraction(fraction),
                    2,
                    101.325,
                )
                .unwrap();
                assert!((p.overhead[2] - c.overhead[1]).abs() < scale * 1e-10);
            }
        }
        let c = ideal_still(
            &[2.0],
            &[model(350.0)],
            StillTake::EnergyKj(20.0),
            3,
            101.325,
        )
        .unwrap();
        assert!((c.overhead[0] - 0.5).abs() < 1e-10);
        assert!((c.t_start_k - 350.0).abs() < 1e-10);
        assert!((c.energy_kj - 20.0).abs() < 1e-10);
        assert!(ideal_still(&[2.0], &[model(350.0)], StillTake::Fraction(0.2), 1, 1e9).is_none());
        assert!(ideal_still(
            &[f64::NAN],
            &[model(350.0)],
            StillTake::Fraction(0.2),
            1,
            101.325
        )
        .is_none());
    }
}
