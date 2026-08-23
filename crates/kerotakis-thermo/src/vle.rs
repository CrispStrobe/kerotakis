//! Vapour–liquid equilibrium: Antoine, Raoult, and the flash between them.
//!
//! The activity coefficients live in [`crate::unifac`]; this module is the
//! machinery that consumes them, and it is deliberately usable without
//! them. Pin γ at 1 and you have the ideal mixture school chemistry
//! teaches — which is worth having on its own, because the bench can then
//! show you the ideal prediction and the real one side by side and let the
//! gap do the teaching.

use serde::{Deserialize, Serialize};

/// Antoine constants for `log10(P/kPa) = a − b / (T/°C + c)`.
///
/// The form matters and is a common source of silent error: Antoine
/// constants are published against at least four combinations of pressure
/// unit and temperature unit, and a set fitted for mmHg and kelvin plugged
/// into this equation gives a plausible-looking number that is wrong by a
/// factor. The first draft of this file made exactly that mistake — the
/// source string said "converted to kPa" while `a` was still the mmHg
/// value — and boiled water at 51.9 °C. A wrong `a` does not look wrong;
/// it looks like a temperature. The unit convention is therefore part of the type, and
/// `valid_c` is carried because extrapolating Antoine outside its fitted
/// range is the second common source.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Antoine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    /// Temperature range the constants were fitted over, °C.
    pub valid_c: (f64, f64),
    pub source: &'static str,
}

impl Antoine {
    /// Saturation vapour pressure, kPa. `None` outside the fitted range,
    /// rather than an extrapolation dressed as a measurement.
    pub fn pressure_kpa(&self, t_celsius: f64) -> Option<f64> {
        if t_celsius < self.valid_c.0 || t_celsius > self.valid_c.1 {
            return None;
        }
        Some(10f64.powf(self.a - self.b / (t_celsius + self.c)))
    }

    /// The same, extrapolated, for the solver's inner loop.
    ///
    /// A flash iterating towards a bubble point will step outside the range
    /// on its way to a temperature that is inside it, and refusing mid-walk
    /// would make the solve fail for arithmetic reasons rather than
    /// physical ones. The *answer* is range-checked by the caller;
    /// intermediate steps are not.
    pub fn pressure_kpa_unchecked(&self, t_celsius: f64) -> f64 {
        10f64.powf(self.a - self.b / (t_celsius + self.c))
    }
}

/// Standard atmospheric pressure, kPa.
pub const ATMOSPHERE_KPA: f64 = 101.325;

/// Water, from the classic two-range Antoine fit. This is the 1–100 °C
/// range, which is the one a bench lives in.
pub const WATER: Antoine = Antoine {
    a: 7.19621,
    b: 1730.63,
    c: 233.426,
    valid_c: (1.0, 100.0),
    source: "Antoine constants for water (1-100 °C): Stull, D.R., \
             Ind. Eng. Chem. 39(4), 517-540 (1947), Table I. \
             Published as log10(P/mmHg) = 8.07131 - 1730.63/(T/°C + 233.426); \
             `a` carries the kPa conversion: 8.07131 - log10(760/101.325) = 7.19621. \
             Gives 101.34 kPa at 100 °C (lit. 100.0 °C at 1 atm)",
};

/// Ethanol, over the range that spans its boiling point.
pub const ETHANOL: Antoine = Antoine {
    a: 7.32907,
    b: 1642.89,
    c: 230.300,
    valid_c: (-57.0, 80.0),
    source: "Antoine constants for ethanol (-57 to 80 °C): Stull, D.R., \
             Ind. Eng. Chem. 39(4), 517-540 (1947), Table I. \
             Published as log10(P/mmHg) = 8.20417 - 1642.89/(T/°C + 230.300); \
             `a` carries the kPa conversion: 8.20417 - log10(760/101.325) = 7.32907. \
             Gives 101.65 kPa at 78.4 °C (lit. 78.37 °C at 1 atm)",
};

/// Methanol, over the range spanning its boiling point at 64.7 °C.
pub const METHANOL: Antoine = Antoine {
    a: 7.20607,
    b: 1582.271,
    c: 239.726,
    valid_c: (15.0, 84.0),
    source: "Antoine constants for methanol (15-84 °C): Stull, D.R., \
             Ind. Eng. Chem. 39(4), 517-540 (1947), Table I. \
             Published as log10(P/mmHg) = 8.08097 - 1582.271/(T/°C + 239.726); \
             `a` carries the kPa conversion: 8.08097 - log10(760/101.325) = 7.20607. \
             Gives 102.3 kPa at 64.7 °C (lit. 64.7 °C at 1 atm)",
};

/// Propanone (acetone), over the range spanning its boiling point at 56.05 °C.
pub const PROPANONE: Antoine = Antoine {
    a: 6.14957,
    b: 1161.0,
    c: 224.0,
    valid_c: (-20.0, 77.0),
    source: "Antoine constants for propanone (-20 to 77 °C): Stull, D.R., \
             Ind. Eng. Chem. 39(4), 517-540 (1947), Table I. \
             Published as log10(P/mmHg) = 7.02447 - 1161.0/(T/°C + 224.0); \
             `a` carries the kPa conversion: 7.02447 - log10(760/101.325) = 6.14957. \
             Gives 100.7 kPa at 56.05 °C (lit. 56.05 °C at 1 atm)",
};

/// Ethanoic acid (acetic acid), over the range spanning its boiling point at 117.9 °C.
pub const ETHANOIC_ACID: Antoine = Antoine {
    a: 6.51292,
    b: 1533.313,
    c: 222.309,
    valid_c: (17.0, 157.0),
    source: "Antoine constants for ethanoic acid (17-157 °C): Stull, D.R., \
             Ind. Eng. Chem. 39(4), 517-540 (1947), Table I. \
             Published as log10(P/mmHg) = 7.38782 - 1533.313/(T/°C + 222.309); \
             `a` carries the kPa conversion: 7.38782 - log10(760/101.325) = 6.51292. \
             Gives 101.4 kPa at 117.9 °C (lit. 117.9 °C at 1 atm)",
};

/// A pure component's contribution to a mixture.
pub struct Volatile {
    pub antoine: Antoine,
    /// Mole fraction in the liquid (or feed z, or vapour y, depending on context).
    pub x: f64,
    /// Activity coefficient. 1.0 is Raoult's law.
    pub gamma: f64,
}

/// Extended component data for energy-coupled flash calculations.
pub struct FlashComponent {
    pub volatile: Volatile,
    /// Molar heat of vaporization in kJ/mol at the normal boiling point.
    pub delta_hv_kj_per_mol: f64,
    /// Liquid heat capacity in J/(mol·K).
    pub cp_liquid_j_per_mol_k: f64,
}

/// What a boiling mixture is doing.
#[derive(Debug, Clone, PartialEq)]
pub struct BubblePoint {
    /// Temperature at which the mixture's partial pressures reach the
    /// ambient pressure, °C.
    pub t_celsius: f64,
    /// Vapour composition at that temperature — what comes over.
    pub y: Vec<f64>,
    /// True when the vapour has the same composition as the liquid, so
    /// distilling changes nothing. This is the azeotrope, and it is a
    /// computed observation rather than a table entry.
    pub azeotropic: bool,
}

/// How close two compositions must be before distillation has nothing left
/// to separate. A tenth of a mole per cent is well below what a column
/// could act on.
const AZEOTROPE_TOLERANCE: f64 = 1e-3;

/// Absolute zero, °C — the one conversion this module admits.
pub const KELVIN_OFFSET: f64 = 273.15;

/// The temperature at which a liquid mixture starts to boil, and what
/// comes off it first, with activity coefficients that follow the
/// temperature.
///
/// Bisection on Σ xᵢ γᵢ(T) P°ᵢ(T) − P. Both factors move with T and that is
/// why `gammas` is called *inside* the loop rather than once: UNIFAC's
/// ψ_mn = exp(−a_mn/T) is temperature-dependent, and γ(water) at x = 0.95
/// is 2.40 at 298 K against 2.53 at 351 K. Evaluating γ once at a guessed
/// temperature and then solving for a different one would be a subtle way
/// to be wrong by a few per cent — which is the size of the azeotrope's
/// whole displacement from pure ethanol.
///
/// `gammas` takes **kelvin**, because every thermodynamic expression on the
/// other side of the seam is in kelvin and a +273.15 at a boundary is a
/// bug waiting for a tired reader. Antoine is in Celsius on this side; the
/// conversion happens here, once, where it can be seen.
pub fn bubble_point_with<F>(
    antoines: &[Antoine],
    x: &[f64],
    pressure_kpa: f64,
    mut gammas: F,
) -> Option<BubblePoint>
where
    F: FnMut(f64) -> Vec<f64>,
{
    if antoines.is_empty() || antoines.len() != x.len() || pressure_kpa <= 0.0 {
        return None;
    }
    let total_x: f64 = x.iter().sum();
    if total_x <= 0.0 {
        return None;
    }
    let partials = |t_c: f64, gammas: &mut F| -> Vec<f64> {
        let g = gammas(t_c + KELVIN_OFFSET);
        antoines
            .iter()
            .zip(x)
            .enumerate()
            .map(|(i, (a, xi))| {
                xi / total_x * g.get(i).copied().unwrap_or(1.0) * a.pressure_kpa_unchecked(t_c)
            })
            .collect()
    };
    let total = |t_c: f64, gammas: &mut F| -> f64 { partials(t_c, gammas).iter().sum() };

    let (mut lo, mut hi) = (-100.0f64, 400.0f64);
    if total(lo, &mut gammas) > pressure_kpa || total(hi, &mut gammas) < pressure_kpa {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if total(mid, &mut gammas) < pressure_kpa {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-9 {
            break;
        }
    }
    let t = 0.5 * (lo + hi);
    let p = partials(t, &mut gammas);
    let p_total: f64 = p.iter().sum();
    let y: Vec<f64> = p.iter().map(|pi| pi / p_total).collect();
    // An azeotrope is not a special case in the arithmetic — it is what the
    // arithmetic says when the vapour comes out the same as the liquid.
    let azeotropic = x
        .iter()
        .zip(&y)
        .all(|(xi, yi)| (xi / total_x - yi).abs() < AZEOTROPE_TOLERANCE);
    Some(BubblePoint {
        t_celsius: t,
        y,
        azeotropic,
    })
}

/// The same, for a mixture whose activity coefficients do not move with
/// temperature — an ideal one, or a curated γ held fixed.
pub fn bubble_point(mix: &[Volatile], pressure_kpa: f64) -> Option<BubblePoint> {
    let antoines: Vec<Antoine> = mix.iter().map(|c| c.antoine).collect();
    let x: Vec<f64> = mix.iter().map(|c| c.x).collect();
    let g: Vec<f64> = mix.iter().map(|c| c.gamma).collect();
    bubble_point_with(&antoines, &x, pressure_kpa, |_| g.clone())
}

/// Where, if anywhere, a binary mixture stops separating.
///
/// Walks the composition axis looking for the crossing of y₁ − x₁, which is
/// positive where the first component enriches in the vapour and negative
/// where it depletes. A sign change is an azeotrope; no sign change means
/// the mixture separates all the way and a tall enough column reaches a
/// pure component.
///
/// Ideal mixtures never produce one, which is exactly why running this with
/// γ = 1 and then with real activity coefficients is the demonstration.
pub fn azeotrope<F>(
    a: Antoine,
    b: Antoine,
    pressure_kpa: f64,
    mut gammas: F,
) -> Option<(f64, BubblePoint)>
where
    F: FnMut(f64, f64) -> (f64, f64),
{
    let point = |x1: f64, gammas: &mut F| -> Option<BubblePoint> {
        bubble_point_with(&[a, b], &[x1, 1.0 - x1], pressure_kpa, |t_k| {
            let (g1, g2) = gammas(x1, t_k);
            vec![g1, g2]
        })
    };
    // The ends are excluded: at x = 0 and x = 1 the vapour trivially
    // matches the liquid, and calling that an azeotrope would report every
    // mixture as having two.
    let (mut lo, mut hi) = (0.001f64, 0.999f64);
    let mut f_lo = point(lo, &mut gammas)?.y[0] - lo;
    let f_hi = point(hi, &mut gammas)?.y[0] - hi;
    if f_lo.signum() == f_hi.signum() {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let f_mid = point(mid, &mut gammas)?.y[0] - mid;
        if f_mid.signum() == f_lo.signum() {
            lo = mid;
            f_lo = f_mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-12 {
            break;
        }
    }
    let x = 0.5 * (lo + hi);
    point(x, &mut gammas).map(|bp| (x, bp))
}

// ── THERMO-005: Dew point and TP flash ─────────────────────────────

/// What a condensing mixture is doing.
#[derive(Debug, Clone, PartialEq)]
pub struct DewPoint {
    /// Temperature at which the first liquid drop forms, °C.
    pub t_celsius: f64,
    /// Liquid composition at that temperature.
    pub x: Vec<f64>,
}

/// The dew-point temperature of a vapour mixture.
///
/// Bisection on Σ yᵢ/(γᵢ·P°ᵢ(T)) − 1/P = 0.
/// `mix[i].x` is treated as the vapour mole fraction yᵢ.
pub fn dew_point(mix: &[Volatile], pressure_kpa: f64) -> Option<DewPoint> {
    if mix.is_empty() || pressure_kpa <= 0.0 {
        return None;
    }
    let total_y: f64 = mix.iter().map(|c| c.x).sum();
    if total_y <= 0.0 {
        return None;
    }
    let residual = |t: f64| -> f64 {
        let sum: f64 = mix
            .iter()
            .map(|c| {
                let p_sat = c.antoine.pressure_kpa_unchecked(t);
                (c.x / total_y) / (c.gamma * p_sat)
            })
            .sum();
        sum - 1.0 / pressure_kpa
    };
    let (mut lo, mut hi) = (-100.0f64, 400.0f64);
    if residual(lo).signum() == residual(hi).signum() {
        return None;
    }
    let r_lo = residual(lo);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if residual(mid).signum() == r_lo.signum() {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-9 {
            break;
        }
    }
    let t = 0.5 * (lo + hi);
    let x_raw: Vec<f64> = mix
        .iter()
        .map(|c| {
            let p_sat = c.antoine.pressure_kpa_unchecked(t);
            (c.x / total_y) / (c.gamma * p_sat) * pressure_kpa
        })
        .collect();
    let x_sum: f64 = x_raw.iter().sum();
    let x: Vec<f64> = x_raw.iter().map(|xi| xi / x_sum).collect();
    Some(DewPoint { t_celsius: t, x })
}

/// Result of an isothermal (TP) flash calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct FlashResult {
    /// Vapour fraction (0 = all liquid, 1 = all vapour).
    pub vapour_fraction: f64,
    /// Liquid-phase mole fractions.
    pub x: Vec<f64>,
    /// Vapour-phase mole fractions.
    pub y: Vec<f64>,
    /// K-values: Kᵢ = yᵢ/xᵢ = γᵢ·P°ᵢ(T)/P.
    pub k: Vec<f64>,
}

/// Isothermal TP flash via the Rachford-Rice equation.
///
/// `components[i].x` is the overall feed mole fraction zᵢ.
pub fn tp_flash(components: &[Volatile], pressure_kpa: f64, t_celsius: f64) -> Option<FlashResult> {
    if components.is_empty() || pressure_kpa <= 0.0 {
        return None;
    }
    let z_total: f64 = components.iter().map(|c| c.x).sum();
    if z_total <= 0.0 {
        return None;
    }
    let z: Vec<f64> = components.iter().map(|c| c.x / z_total).collect();

    let k: Vec<f64> = components
        .iter()
        .map(|c| c.gamma * c.antoine.pressure_kpa_unchecked(t_celsius) / pressure_kpa)
        .collect();

    // Check subcooled liquid: Σ zᵢ·Kᵢ ≤ 1
    let sum_zk: f64 = z.iter().zip(&k).map(|(zi, ki)| zi * ki).sum();
    if sum_zk <= 1.0 {
        return Some(FlashResult {
            vapour_fraction: 0.0,
            x: z.clone(),
            y: z.iter().zip(&k).map(|(zi, ki)| zi * ki / sum_zk).collect(),
            k,
        });
    }
    // Check superheated vapour: Σ zᵢ/Kᵢ ≤ 1
    let sum_z_over_k: f64 = z.iter().zip(&k).map(|(zi, ki)| zi / ki).sum();
    if sum_z_over_k <= 1.0 {
        return Some(FlashResult {
            vapour_fraction: 1.0,
            x: z.iter()
                .zip(&k)
                .map(|(zi, ki)| zi / ki / sum_z_over_k)
                .collect(),
            y: z.clone(),
            k,
        });
    }

    // Two-phase: solve Rachford-Rice
    let rr = |v: f64| -> f64 {
        z.iter()
            .zip(&k)
            .map(|(zi, ki)| zi * (ki - 1.0) / (1.0 + v * (ki - 1.0)))
            .sum()
    };

    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    let rr_lo = rr(lo);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if rr(mid).signum() == rr_lo.signum() {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-12 {
            break;
        }
    }
    let v = 0.5 * (lo + hi);

    let x: Vec<f64> = z
        .iter()
        .zip(&k)
        .map(|(zi, ki)| zi / (1.0 + v * (ki - 1.0)))
        .collect();
    let y: Vec<f64> = x.iter().zip(&k).map(|(xi, ki)| xi * ki).collect();

    Some(FlashResult {
        vapour_fraction: v,
        x,
        y,
        k,
    })
}

// ── THERMO-006: HP and UV flashes ──────────────────────────────────

/// Result of an adiabatic (HP) flash.
#[derive(Debug, Clone, PartialEq)]
pub struct HpFlashResult {
    /// Equilibrium temperature, °C.
    pub t_celsius: f64,
    /// Vapour fraction.
    pub vapour_fraction: f64,
    /// Liquid-phase mole fractions.
    pub x: Vec<f64>,
    /// Vapour-phase mole fractions.
    pub y: Vec<f64>,
}

/// Adiabatic (constant H, P) flash: given feed enthalpy and pressure,
/// find the equilibrium temperature and phase split.
///
/// Iterates: guess T → TP flash → check energy balance → adjust T.
pub fn hp_flash(
    components: &[FlashComponent],
    pressure_kpa: f64,
    feed_enthalpy_kj: f64,
    total_moles: f64,
) -> Option<HpFlashResult> {
    if components.is_empty() || pressure_kpa <= 0.0 || total_moles <= 0.0 {
        return None;
    }

    let volatiles: Vec<Volatile> = components
        .iter()
        .map(|c| Volatile {
            antoine: c.volatile.antoine,
            x: c.volatile.x,
            gamma: c.volatile.gamma,
        })
        .collect();

    // Energy balance residual: H_feed - H(T, V) = 0
    // H(T, V) = Σ nᵢ [cp_L,i (T - T_ref) + V·yᵢ·ΔHv,i]
    let t_ref = 25.0; // reference temperature °C

    let enthalpy_at = |t: f64| -> Option<f64> {
        let flash = tp_flash(&volatiles, pressure_kpa, t)?;
        let mut h = 0.0;
        // Sensible heat: all feed enters as liquid at T_ref, heated to T
        for c in components.iter() {
            let n_i = c.volatile.x * total_moles;
            h += n_i * c.cp_liquid_j_per_mol_k * (t - t_ref) / 1000.0;
        }
        // Latent heat: moles that vaporized × ΔHv
        for (i, c) in components.iter().enumerate() {
            let n_vap_i = total_moles * flash.vapour_fraction * flash.y[i];
            h += n_vap_i * c.delta_hv_kj_per_mol;
        }
        Some(h)
    };

    // Bisection on H(T) - H_feed = 0
    let (mut lo, mut hi) = (-50.0f64, 350.0f64);
    let h_lo = enthalpy_at(lo)?;
    let h_hi = enthalpy_at(hi)?;
    let residual_lo = h_lo - feed_enthalpy_kj;
    let residual_hi = h_hi - feed_enthalpy_kj;

    if residual_lo.signum() == residual_hi.signum() {
        // Enthalpy not bracketed — return the closest bound
        return None;
    }

    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let h_mid = enthalpy_at(mid)?;
        let residual_mid = h_mid - feed_enthalpy_kj;
        if residual_mid.signum() == residual_lo.signum() {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-6 {
            break;
        }
    }

    let t = 0.5 * (lo + hi);
    let flash = tp_flash(&volatiles, pressure_kpa, t)?;
    Some(HpFlashResult {
        t_celsius: t,
        vapour_fraction: flash.vapour_fraction,
        x: flash.x,
        y: flash.y,
    })
}

/// Mass fraction of the first component, from its mole fraction.
pub fn mass_fraction(x1: f64, m1: f64, m2: f64) -> f64 {
    let (a, b) = (x1 * m1, (1.0 - x1) * m2);
    a / (a + b)
}

/// Bubble point of the ethanol–water binary with full UNIFAC γ(T)
/// (Fredenslund 1975 parameters) — the mixture the school still is built
/// around, packaged so a bench does not need to know group
/// decompositions. `x_ethanol` is the ethanol mole fraction of the
/// volatile liquid.
pub fn ethanol_water_bubble_point(x_ethanol: f64, pressure_kpa: f64) -> Option<BubblePoint> {
    let table = crate::unifac::approved_table();
    let mut ethanol_groups = crate::unifac::GroupDecomposition::new();
    ethanol_groups.insert(1, 1); // CH3
    ethanol_groups.insert(2, 1); // CH2
    ethanol_groups.insert(14, 1); // OH
    let mut water_groups = crate::unifac::GroupDecomposition::new();
    water_groups.insert(16, 1); // H2O
    bubble_point_with(
        &[ETHANOL, WATER],
        &[x_ethanol, 1.0 - x_ethanol],
        pressure_kpa,
        |t_k| {
            crate::unifac::activity_coefficients(
                &table,
                &[
                    (ethanol_groups.clone(), x_ethanol),
                    (water_groups.clone(), 1.0 - x_ethanol),
                ],
                t_k,
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_water_bubble_point() {
        let mix = [Volatile {
            antoine: WATER,
            x: 1.0,
            gamma: 1.0,
        }];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            (bp.t_celsius - 100.0).abs() < 0.5,
            "water boils at {:.2} °C",
            bp.t_celsius
        );
    }

    #[test]
    fn pure_water_dew_point() {
        let mix = [Volatile {
            antoine: WATER,
            x: 1.0,
            gamma: 1.0,
        }];
        let dp = dew_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            (dp.t_celsius - 100.0).abs() < 0.5,
            "water dew point at {:.2} °C",
            dp.t_celsius
        );
    }

    #[test]
    fn pure_methanol_bubble_point() {
        let mix = [Volatile {
            antoine: METHANOL,
            x: 1.0,
            gamma: 1.0,
        }];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            (bp.t_celsius - 64.7).abs() < 0.5,
            "methanol boils at {:.2} °C (expected ~64.7)",
            bp.t_celsius
        );
    }

    #[test]
    fn pure_propanone_bubble_point() {
        let mix = [Volatile {
            antoine: PROPANONE,
            x: 1.0,
            gamma: 1.0,
        }];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            (bp.t_celsius - 56.1).abs() < 0.5,
            "propanone boils at {:.2} °C (expected ~56.1)",
            bp.t_celsius
        );
    }

    #[test]
    fn pure_ethanoic_acid_bubble_point() {
        let mix = [Volatile {
            antoine: ETHANOIC_ACID,
            x: 1.0,
            gamma: 1.0,
        }];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            (bp.t_celsius - 117.9).abs() < 0.5,
            "ethanoic acid boils at {:.2} °C (expected ~117.9)",
            bp.t_celsius
        );
    }

    #[test]
    fn bubble_below_dew_for_binary() {
        let mix = [
            Volatile {
                antoine: ETHANOL,
                x: 0.5,
                gamma: 1.0,
            },
            Volatile {
                antoine: WATER,
                x: 0.5,
                gamma: 1.0,
            },
        ];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        let dp = dew_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            bp.t_celsius < dp.t_celsius,
            "bubble {:.2} should be below dew {:.2}",
            bp.t_celsius,
            dp.t_celsius
        );
    }

    #[test]
    fn tp_flash_subcooled_liquid() {
        // At 20°C, 1 atm: water-ethanol is all liquid
        let mix = [
            Volatile {
                antoine: ETHANOL,
                x: 0.5,
                gamma: 1.0,
            },
            Volatile {
                antoine: WATER,
                x: 0.5,
                gamma: 1.0,
            },
        ];
        let result = tp_flash(&mix, ATMOSPHERE_KPA, 20.0).unwrap();
        assert!(
            result.vapour_fraction < 1e-10,
            "should be all liquid at 20°C, V = {}",
            result.vapour_fraction
        );
    }

    #[test]
    fn tp_flash_superheated_vapour() {
        // At 200°C, 1 atm: everything is vapour
        let mix = [
            Volatile {
                antoine: ETHANOL,
                x: 0.5,
                gamma: 1.0,
            },
            Volatile {
                antoine: WATER,
                x: 0.5,
                gamma: 1.0,
            },
        ];
        let result = tp_flash(&mix, ATMOSPHERE_KPA, 200.0).unwrap();
        assert!(
            (result.vapour_fraction - 1.0).abs() < 1e-10,
            "should be all vapour at 200°C, V = {}",
            result.vapour_fraction
        );
    }

    #[test]
    fn tp_flash_two_phase() {
        // At bubble point + a few degrees: partial vaporization
        let mix_bp = [
            Volatile {
                antoine: ETHANOL,
                x: 0.3,
                gamma: 1.0,
            },
            Volatile {
                antoine: WATER,
                x: 0.7,
                gamma: 1.0,
            },
        ];
        let bp = bubble_point(&mix_bp, ATMOSPHERE_KPA).unwrap();

        // Flash a few degrees above the bubble point
        let mix_flash = [
            Volatile {
                antoine: ETHANOL,
                x: 0.3,
                gamma: 1.0,
            },
            Volatile {
                antoine: WATER,
                x: 0.7,
                gamma: 1.0,
            },
        ];
        let result = tp_flash(&mix_flash, ATMOSPHERE_KPA, bp.t_celsius + 2.0).unwrap();
        assert!(
            result.vapour_fraction > 0.0 && result.vapour_fraction < 1.0,
            "should be two-phase at T_bubble + 2°C, V = {}",
            result.vapour_fraction
        );
        // Ethanol should be enriched in the vapour
        assert!(
            result.y[0] > result.x[0],
            "ethanol should enrich in vapour: y={:.4} vs x={:.4}",
            result.y[0],
            result.x[0]
        );
    }

    #[test]
    fn flash_compositions_sum_to_one() {
        let mix = [
            Volatile {
                antoine: ETHANOL,
                x: 0.4,
                gamma: 1.0,
            },
            Volatile {
                antoine: WATER,
                x: 0.6,
                gamma: 1.0,
            },
        ];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        let result = tp_flash(&mix, ATMOSPHERE_KPA, bp.t_celsius + 3.0).unwrap();

        let x_sum: f64 = result.x.iter().sum();
        let y_sum: f64 = result.y.iter().sum();
        assert!(
            (x_sum - 1.0).abs() < 1e-10,
            "liquid fractions sum to {}",
            x_sum
        );
        assert!(
            (y_sum - 1.0).abs() < 1e-10,
            "vapour fractions sum to {}",
            y_sum
        );
    }

    // ── THERMO-006 tests ──────────────────────────────────────────

    fn water_component(x: f64) -> FlashComponent {
        FlashComponent {
            volatile: Volatile {
                antoine: WATER,
                x,
                gamma: 1.0,
            },
            delta_hv_kj_per_mol: 40.7, // water at 100°C
            cp_liquid_j_per_mol_k: 75.3,
        }
    }

    fn ethanol_component(x: f64) -> FlashComponent {
        FlashComponent {
            volatile: Volatile {
                antoine: ETHANOL,
                x,
                gamma: 1.0,
            },
            delta_hv_kj_per_mol: 38.6, // ethanol at 78°C
            cp_liquid_j_per_mol_k: 112.0,
        }
    }

    #[test]
    fn hp_flash_finds_temperature() {
        // Feed 1 mol of 50/50 ethanol/water at a known enthalpy
        let components = [ethanol_component(0.5), water_component(0.5)];
        let total_moles = 1.0;
        // Enthalpy of liquid at 80°C: roughly Σ x_i cp_i (80 - 25) / 1000
        let h_liquid = 0.5 * 112.0 * 55.0 / 1000.0 + 0.5 * 75.3 * 55.0 / 1000.0;

        let result = hp_flash(&components, ATMOSPHERE_KPA, h_liquid, total_moles);
        assert!(result.is_some(), "HP flash should converge");
        let r = result.unwrap();
        // Temperature should be between bubble and dew points
        assert!(
            r.t_celsius > 50.0 && r.t_celsius < 120.0,
            "T = {:.1} °C should be reasonable",
            r.t_celsius
        );
    }

    #[test]
    fn hp_flash_latent_heat_plateau() {
        // Feed enough enthalpy to partially vaporize: the temperature should
        // plateau at the bubble point while latent heat is absorbed.
        let components = [water_component(1.0)];
        let total_moles = 1.0;
        // Enthalpy just above bubble point: liquid heating + small vaporization
        let h_near_boil = 75.3 * 75.0 / 1000.0 + 0.02 * 40.7; // 75°C heating + 2% vaporized
        let result = hp_flash(&components, ATMOSPHERE_KPA, h_near_boil, total_moles);
        assert!(result.is_some());
        let r = result.unwrap();
        // The HP flash should find a temperature near 100°C with some
        // vaporization. For pure water, the flash is either all liquid
        // (below 100°C) or all vapour (above). A true HP flash on a
        // pure component at the boiling point gives V = (H - H_liq) / ΔHv.
        // With the simplified enthalpy model, the temperature should be
        // near the boiling point.
        assert!(
            r.t_celsius > 90.0 && r.t_celsius < 110.0,
            "should be near 100°C, got {:.1}",
            r.t_celsius
        );
    }
}

// ── CAP-17: the batch still — Rayleigh drift, stages, latent energy ──

/// Molar enthalpy of vaporisation, kJ/mol, at each component's normal
/// boiling point. Water from the IAPWS-95 formulation (Wagner & Pruß,
/// J. Phys. Chem. Ref. Data 31, 2002): 40.657 kJ/mol at 100 °C. Ethanol
/// from Majer & Svoboda, "Enthalpies of Vaporization of Organic
/// Compounds" (IUPAC Chemical Data Series No. 32, 1985): 38.56 kJ/mol
/// at 78.3 °C. Held constant over the still's narrow temperature range —
/// a stated approximation worth ~1 % across 78–100 °C.
pub const WATER_HVAP_KJ_PER_MOL: f64 = 40.657;
pub const ETHANOL_HVAP_KJ_PER_MOL: f64 = 38.56;

/// How much a still is asked to take overhead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StillTake {
    /// This fraction (0..=1) of the volatile liquid, by moles.
    Fraction(f64),
    /// As much as this much latent heat can lift, kJ.
    EnergyKj(f64),
}

/// What one batch cut produced.
#[derive(Debug, Clone, PartialEq)]
pub struct StillCut {
    /// Moles of each component condensed into the receiver.
    pub water_over: f64,
    pub ethanol_over: f64,
    /// Pot boiling temperature at the start and end of the cut, °C —
    /// the drift between them is the Rayleigh story made visible.
    pub t_start_c: f64,
    pub t_end_c: f64,
    /// Latent heat the cut consumed, kJ — what the burner really paid
    /// and the condenser really dumped.
    pub energy_kj: f64,
    /// The column ran into the azeotrope: the top-stage vapour matches
    /// its liquid and more stages change nothing.
    pub azeotrope_limited: bool,
}

/// One ideal-stage cascade at total reflux: the vapour of stage n is the
/// liquid of stage n+1. Returns the top-stage vapour composition and
/// whether the walk hit the azeotrope on the way up. The total-reflux
/// idealisation is stated, not hidden: a real column at finite reflux
/// separates less, never more, so this is the honest *upper bound* a
/// learner's column cannot beat.
fn cascade(x_pot: f64, stages: u32, pressure_kpa: f64) -> Option<(f64, BubblePoint, bool)> {
    let pot_bp = ethanol_water_bubble_point(x_pot, pressure_kpa)?;
    let mut y = pot_bp.y[0];
    let mut hit = pot_bp.azeotropic;
    for _ in 1..stages {
        let bp = ethanol_water_bubble_point(y, pressure_kpa)?;
        if bp.azeotropic {
            hit = true;
            break;
        }
        y = bp.y[0];
    }
    Some((y, pot_bp, hit))
}

/// A batch distillation cut of the ethanol–water binary with full UNIFAC
/// γ(T): Rayleigh integration — the vapour composition follows the pot as
/// it drifts — through an `stages`-stage column at total reflux.
///
/// Integration is 256 fixed steps of the overhead amount; halving the
/// step count moves the answers in the fourth decimal, which is far
/// inside the model's own honesty budget.
pub fn ethanol_water_still(
    water_moles: f64,
    ethanol_moles: f64,
    take: StillTake,
    stages: u32,
    pressure_kpa: f64,
) -> Option<StillCut> {
    if water_moles < 0.0 || ethanol_moles < 0.0 {
        return None;
    }
    let total0 = water_moles + ethanol_moles;
    if total0 <= 0.0 {
        return None;
    }
    let stages = stages.max(1);
    let (mut w, mut e) = (water_moles, ethanol_moles);
    let (mut w_over, mut e_over) = (0.0f64, 0.0f64);
    let mut energy_kj = 0.0f64;
    let mut azeo = false;

    let budget = match take {
        StillTake::Fraction(f) => {
            if !(0.0..=1.0).contains(&f) {
                return None;
            }
            f * total0
        }
        // Provisional mole budget for step sizing; the loop stops on the
        // real energy meter below.
        StillTake::EnergyKj(kj) => {
            if kj < 0.0 {
                return None;
            }
            (kj / WATER_HVAP_KJ_PER_MOL.min(ETHANOL_HVAP_KJ_PER_MOL)).min(total0)
        }
    };

    let (y0, bp0, _) = cascade(e / (w + e), stages, pressure_kpa)?;
    let _ = y0;
    let t_start_c = bp0.t_celsius;
    let mut t_end_c = t_start_c;

    const STEPS: usize = 256;
    let dn = budget / STEPS as f64;
    if dn <= 0.0 {
        return Some(StillCut {
            water_over: 0.0,
            ethanol_over: 0.0,
            t_start_c,
            t_end_c,
            energy_kj: 0.0,
            azeotrope_limited: false,
        });
    }
    for _ in 0..STEPS {
        let pot = w + e;
        if pot <= 1e-12 {
            break;
        }
        let x = e / pot;
        let Some((y_top, pot_bp, hit)) = cascade(x, stages, pressure_kpa) else {
            break;
        };
        t_end_c = pot_bp.t_celsius;
        azeo |= hit;
        let dn = dn.min(pot);
        let de = (dn * y_top).min(e);
        let dw = (dn - de).min(w);
        let step_kj = de * ETHANOL_HVAP_KJ_PER_MOL + dw * WATER_HVAP_KJ_PER_MOL;
        if let StillTake::EnergyKj(kj) = take {
            if energy_kj + step_kj > kj {
                // The burner's budget ends mid-step: take the affordable
                // share of this step and stop.
                let share = ((kj - energy_kj) / step_kj).clamp(0.0, 1.0);
                e -= de * share;
                w -= dw * share;
                e_over += de * share;
                w_over += dw * share;
                energy_kj = kj;
                break;
            }
        }
        e -= de;
        w -= dw;
        e_over += de;
        w_over += dw;
        energy_kj += step_kj;
    }
    Some(StillCut {
        water_over: w_over,
        ethanol_over: e_over,
        t_start_c,
        t_end_c,
        energy_kj,
        azeotrope_limited: azeo,
    })
}
