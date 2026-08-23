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
    source: "Antoine constants for water, the widely reproduced 1-100 °C \
             fit attributed to Stull 1947 via the NIST WebBook. Published \
             in mmHg; `a` carries the conversion, 8.07131 - log10(760 / \
             101.325) = 7.19621, and gives 101.34 kPa at 100 °C",
};

/// Ethanol, over the range that spans its boiling point.
pub const ETHANOL: Antoine = Antoine {
    a: 7.32907,
    b: 1642.89,
    c: 230.300,
    valid_c: (-57.0, 80.0),
    source: "Antoine constants for ethanol, the widely reproduced fit \
             attributed to Stull 1947 via the NIST WebBook. Published in \
             mmHg; `a` carries the conversion, 8.20417 - log10(760 / \
             101.325) = 7.32907, and gives 101.65 kPa at 78.4 °C",
};

/// A pure component's contribution to a mixture.
pub struct Volatile {
    pub antoine: Antoine,
    /// Mole fraction in the liquid.
    pub x: f64,
    /// Activity coefficient. 1.0 is Raoult's law — the ideal mixture, kept
    /// as a first-class option because being able to compute the ideal
    /// answer is what makes the real one legible.
    pub gamma: f64,
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

/// Mass fraction of the first component, from its mole fraction.
pub fn mass_fraction(x1: f64, m1: f64, m2: f64) -> f64 {
    let (a, b) = (x1 * m1, (1.0 - x1) * m2);
    a / (a + b)
}
