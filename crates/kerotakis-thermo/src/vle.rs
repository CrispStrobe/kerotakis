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
        if !t_celsius.is_finite()
            || !self.valid_c.0.is_finite()
            || !self.valid_c.1.is_finite()
            || self.valid_c.0 > self.valid_c.1
            || t_celsius < self.valid_c.0
            || t_celsius > self.valid_c.1
        {
            return None;
        }
        let pressure = 10f64.powf(self.a - self.b / (t_celsius + self.c));
        (pressure.is_finite() && pressure > 0.0).then_some(pressure)
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

/// A saturation-pressure correlation composed of contiguous Antoine fits.
///
/// Antoine fits are local correlations. This small value type lets a fluid
/// retain a trusted low-temperature fit while extending its domain with a
/// separately sourced fit; gaps remain refusals rather than extrapolations.
#[derive(Debug, Clone, Copy)]
pub enum VapourPressure {
    Antoine(Antoine),
    Piecewise(&'static [Antoine]),
}

impl VapourPressure {
    const fn segments(&self) -> &[Antoine] {
        match self {
            Self::Antoine(segment) => std::slice::from_ref(segment),
            Self::Piecewise(segments) => segments,
        }
    }

    pub fn valid_range(&self) -> Option<(f64, f64)> {
        let segments = self.segments();
        let first = segments.first()?;
        let mut lo = first.valid_c.0;
        let mut hi = first.valid_c.1;
        if !lo.is_finite()
            || !hi.is_finite()
            || lo > hi
            || !first.a.is_finite()
            || !first.b.is_finite()
            || !first.c.is_finite()
        {
            return None;
        }
        for segment in &segments[1..] {
            let (next_lo, next_hi) = segment.valid_c;
            if !next_lo.is_finite()
                || !next_hi.is_finite()
                || next_lo > next_hi
                || next_lo > hi
                || !segment.a.is_finite()
                || !segment.b.is_finite()
                || !segment.c.is_finite()
            {
                return None;
            }
            hi = hi.max(next_hi);
            lo = lo.min(next_lo);
        }
        Some((lo, hi))
    }

    pub fn pressure_kpa(&self, t_celsius: f64) -> Option<f64> {
        self.segments()
            .iter()
            .find(|segment| t_celsius >= segment.valid_c.0 && t_celsius <= segment.valid_c.1)
            .and_then(|segment| segment.pressure_kpa(t_celsius))
    }

    fn pressure_kpa_unchecked(&self, t_celsius: f64) -> f64 {
        let segments = self.segments();
        let segment = self
            .segments()
            .iter()
            .find(|segment| t_celsius >= segment.valid_c.0 && t_celsius <= segment.valid_c.1)
            .unwrap_or_else(|| {
                if t_celsius < segments[0].valid_c.0 {
                    &segments[0]
                } else {
                    segments.last().expect("vapour-pressure segments")
                }
            });
        10f64.powf(segment.a - segment.b / (t_celsius + segment.c))
    }
}

impl From<Antoine> for VapourPressure {
    fn from(value: Antoine) -> Self {
        Self::Antoine(value)
    }
}

/// The common fitted temperature interval shared by every component.
/// A mixture has no defensible Antoine answer when those intervals do not
/// overlap: choosing the union would necessarily extrapolate at least one
/// component.
fn common_valid_range(antoines: &[VapourPressure], fractions: &[f64]) -> Option<(f64, f64)> {
    if antoines.len() != fractions.len() {
        return None;
    }
    let mut lo = f64::NEG_INFINITY;
    let mut hi = f64::INFINITY;
    let mut active = false;
    for (antoine, fraction) in antoines.iter().zip(fractions) {
        if *fraction == 0.0 {
            continue;
        }
        active = true;
        let (component_lo, component_hi) = antoine.valid_range()?;
        lo = lo.max(component_lo);
        hi = hi.min(component_hi);
    }
    (active && lo <= hi).then_some((lo, hi))
}

fn valid_fractions(values: &[f64]) -> bool {
    values
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
}

/// Standard atmospheric pressure, kPa.
pub const ATMOSPHERE_KPA: f64 = 101.325;

/// Water, from the classic two-range Antoine fit. This is the 1–100 °C
/// range, which is the one a bench lives in.
const WATER_ANTOINE: Antoine = Antoine {
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
pub const WATER: VapourPressure = VapourPressure::Antoine(WATER_ANTOINE);

/// Ethanol, over the range that spans its boiling point.
pub const ETHANOL_LOW: Antoine = Antoine {
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

/// Experimentally fitted high-temperature ethanol saturation pressure.
///
/// Susial Badajoz, García Montesdeoca, and Santiago measured pure-ethanol
/// vapour pressures from 107 to 1015 kPa and fitted
/// `log10(P/kPa) = 6.99161 - 1460.701/(T/K - 58.477)`. The Celsius `c`
/// below is the exact unit transform `273.15 - 58.477`. Their article and
/// its data tables are licensed CC BY 4.0:
/// <https://doi.org/10.1021/acsomega.6c04827>.
pub const ETHANOL_HIGH: Antoine = Antoine {
    a: 6.99161,
    b: 1460.701,
    c: 214.673,
    valid_c: (79.65, 151.95),
    source: "Susial Badajoz, P., Garcia Montesdeoca, I., & Santiago, D.E. (2026), \
             ACS Omega 11, 48295-48312, DOI 10.1021/acsomega.6c04827, CC BY 4.0. \
             Experimental pure-ethanol fit over 107-1015 kPa (352.8-425.1 K): \
             log10(P/kPa) = 6.99161 - 1460.701/(T/K - 58.477); \
             converted exactly to the Celsius denominator T/°C + 214.673",
};

const ETHANOL_SEGMENTS: &[Antoine] = &[ETHANOL_LOW, ETHANOL_HIGH];
pub const ETHANOL: VapourPressure = VapourPressure::Piecewise(ETHANOL_SEGMENTS);

/// Isopropanol over the NIST fit range that spans its normal boiling point.
/// NIST publishes pressure in bar and temperature in kelvin; `a` includes
/// the bar-to-kPa factor and `c` includes the kelvin-to-Celsius offset.
const ISOPROPANOL_ANTOINE: Antoine = Antoine {
    a: 6.861,
    b: 1357.427,
    c: 197.336,
    valid_c: (56.77, 89.26),
    source: "NIST Chemistry WebBook, SRD 69, isopropyl alcohol Antoine equation: Stull, D.R., Ind. Eng. Chem. 39, 517-540 (1947), 329.92-362.41 K; converted from log10(P/bar) = 4.8610 - 1357.427/(T/K - 75.814) to kPa and Celsius",
};
pub const ISOPROPANOL: VapourPressure = VapourPressure::Antoine(ISOPROPANOL_ANTOINE);

/// Methanol, over the range spanning its boiling point at 64.7 °C.
const METHANOL_ANTOINE: Antoine = Antoine {
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
pub const METHANOL: VapourPressure = VapourPressure::Antoine(METHANOL_ANTOINE);

/// Propanone (acetone), over the range spanning its boiling point at 56.05 °C.
const PROPANONE_ANTOINE: Antoine = Antoine {
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
pub const PROPANONE: VapourPressure = VapourPressure::Antoine(PROPANONE_ANTOINE);

/// Ethanoic acid (acetic acid), over the range spanning its boiling point at 117.9 °C.
const ETHANOIC_ACID_ANTOINE: Antoine = Antoine {
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
pub const ETHANOIC_ACID: VapourPressure = VapourPressure::Antoine(ETHANOIC_ACID_ANTOINE);

/// A pure component's contribution to a mixture.
pub struct Volatile {
    pub antoine: VapourPressure,
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
    antoines: &[VapourPressure],
    x: &[f64],
    pressure_kpa: f64,
    mut gammas: F,
) -> Option<BubblePoint>
where
    F: FnMut(f64) -> Vec<f64>,
{
    if antoines.is_empty()
        || antoines.len() != x.len()
        || !pressure_kpa.is_finite()
        || pressure_kpa <= 0.0
        || !valid_fractions(x)
    {
        return None;
    }
    let (mut lo, mut hi) = common_valid_range(antoines, x)?;
    let total_x: f64 = x.iter().sum();
    if total_x <= 0.0 {
        return None;
    }
    let partials = |t_c: f64, gammas: &mut F| -> Vec<f64> {
        let g = gammas(t_c + KELVIN_OFFSET);
        if g.len() != antoines.len() || g.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return vec![f64::NAN];
        }
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

    let total_lo = total(lo, &mut gammas);
    let total_hi = total(hi, &mut gammas);
    if !total_lo.is_finite()
        || !total_hi.is_finite()
        || total_lo > pressure_kpa
        || total_hi < pressure_kpa
    {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let total_mid = total(mid, &mut gammas);
        if !total_mid.is_finite() {
            return None;
        }
        if total_mid < pressure_kpa {
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
    if !p_total.is_finite() || p_total <= 0.0 {
        return None;
    }
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
    let antoines: Vec<VapourPressure> = mix.iter().map(|c| c.antoine).collect();
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
    a: VapourPressure,
    b: VapourPressure,
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
    // Some correlations overlap only on part of the composition axis at the
    // requested pressure. Find a bracket among valid points instead of making
    // one out-of-domain endpoint suppress an in-domain azeotrope.
    let mut previous: Option<(f64, f64)> = None;
    let mut bracket = None;
    for step in 1..1000 {
        let x = step as f64 / 1000.0;
        let Some(bp) = point(x, &mut gammas) else {
            continue;
        };
        let f = bp.y[0] - x;
        if let Some((previous_x, previous_f)) = previous {
            if f.signum() != previous_f.signum() {
                bracket = Some((previous_x, x, previous_f));
                break;
            }
        }
        previous = Some((x, f));
    }
    let (mut lo, mut hi, mut f_lo) = bracket?;
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
/// The same, for fixed activity coefficients: γ pinned per component.
pub fn dew_point(mix: &[Volatile], pressure_kpa: f64) -> Option<DewPoint> {
    let antoines: Vec<VapourPressure> = mix.iter().map(|c| c.antoine).collect();
    let y: Vec<f64> = mix.iter().map(|c| c.x).collect();
    let g: Vec<f64> = mix.iter().map(|c| c.gamma).collect();
    dew_point_with(&antoines, &y, pressure_kpa, &mut |_, _| g.clone())
}

/// The dew point with γ following the *liquid* composition and the
/// temperature (CAP-16).
///
/// Dew is the calculation where a fixed γ is most quietly wrong: the
/// activity coefficients belong to the condensing liquid, and the liquid
/// composition is exactly what the calculation is solving for. So γ here
/// is a function of (x, kelvin), and the answer is found by successive
/// substitution — start from x = y, bisect the temperature with γ(x, T)
/// live inside the residual, back the liquid composition out, repeat
/// until x stops moving. Constant γ converges on the first pass to the
/// same arithmetic the fixed wrapper always did.
pub fn dew_point_with(
    antoines: &[VapourPressure],
    y: &[f64],
    pressure_kpa: f64,
    gammas: &mut dyn FnMut(&[f64], f64) -> Vec<f64>,
) -> Option<DewPoint> {
    if antoines.is_empty()
        || antoines.len() != y.len()
        || !pressure_kpa.is_finite()
        || pressure_kpa <= 0.0
        || !valid_fractions(y)
    {
        return None;
    }
    let (range_lo, range_hi) = common_valid_range(antoines, y)?;
    let total_y: f64 = y.iter().sum();
    if total_y <= 0.0 {
        return None;
    }
    let y: Vec<f64> = y.iter().map(|v| v / total_y).collect();
    let mut x = y.clone();
    for _ in 0..80 {
        let mut residual = |t: f64, x: &[f64]| -> f64 {
            let g = gammas(x, t + KELVIN_OFFSET);
            if g.len() != antoines.len() || g.iter().any(|v| !v.is_finite() || *v <= 0.0) {
                return f64::NAN;
            }
            let sum: f64 = antoines
                .iter()
                .zip(&y)
                .enumerate()
                .map(|(i, (a, yi))| {
                    yi / (g.get(i).copied().unwrap_or(1.0) * a.pressure_kpa_unchecked(t))
                })
                .sum();
            sum - 1.0 / pressure_kpa
        };
        let (mut lo, mut hi) = (range_lo, range_hi);
        let r_lo = residual(lo, &x);
        let r_hi = residual(hi, &x);
        if !r_lo.is_finite() || !r_hi.is_finite() || r_lo.signum() == r_hi.signum() {
            return None;
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let r_mid = residual(mid, &x);
            if !r_mid.is_finite() {
                return None;
            }
            if r_mid.signum() == r_lo.signum() {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo < 1e-9 {
                break;
            }
        }
        let t = 0.5 * (lo + hi);
        let g = gammas(&x, t + KELVIN_OFFSET);
        if g.len() != antoines.len() || g.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return None;
        }
        let x_raw: Vec<f64> = antoines
            .iter()
            .zip(&y)
            .enumerate()
            .map(|(i, (a, yi))| {
                yi / (g.get(i).copied().unwrap_or(1.0) * a.pressure_kpa_unchecked(t)) * pressure_kpa
            })
            .collect();
        let x_sum: f64 = x_raw.iter().sum();
        if !x_sum.is_finite() || x_sum <= 0.0 {
            return None;
        }
        let x_new: Vec<f64> = x_raw.iter().map(|xi| xi / x_sum).collect();
        let moved = x
            .iter()
            .zip(&x_new)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        // Plain substitution: measured on the worst mid-range
        // ethanol–water case this contracts monotonically at a ratio near
        // 0.6 per pass, so eighty passes clear 1e-9 with a wide margin —
        // damping was tried and only slowed the walk down.
        x = x_new;
        if moved < 1e-9 {
            return Some(DewPoint { t_celsius: t, x });
        }
    }
    // Eighty passes without settling: refuse rather than return a drifting
    // composition dressed as an answer.
    None
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

/// Isothermal TP flash via the Rachford-Rice equation, γ fixed per
/// component. `components[i].x` is the overall feed mole fraction zᵢ.
pub fn tp_flash(components: &[Volatile], pressure_kpa: f64, t_celsius: f64) -> Option<FlashResult> {
    let antoines: Vec<VapourPressure> = components.iter().map(|c| c.antoine).collect();
    let z: Vec<f64> = components.iter().map(|c| c.x).collect();
    let g: Vec<f64> = components.iter().map(|c| c.gamma).collect();
    tp_flash_with(&antoines, &z, pressure_kpa, t_celsius, &mut |_, _| {
        g.clone()
    })
}

/// TP flash with γ following the liquid composition and temperature
/// (CAP-16): the γ–φ successive-substitution loop. K-values are built
/// from γ(x, T); Rachford–Rice gives a new liquid; repeat until the
/// liquid stops moving. Constant γ converges on the first pass to the
/// same arithmetic the fixed wrapper always did.
pub fn tp_flash_with(
    antoines: &[VapourPressure],
    z: &[f64],
    pressure_kpa: f64,
    t_celsius: f64,
    gammas: &mut dyn FnMut(&[f64], f64) -> Vec<f64>,
) -> Option<FlashResult> {
    if antoines.is_empty()
        || antoines.len() != z.len()
        || !pressure_kpa.is_finite()
        || pressure_kpa <= 0.0
        || !t_celsius.is_finite()
        || !valid_fractions(z)
    {
        return None;
    }
    let valid_range = common_valid_range(antoines, z);
    if match valid_range {
        Some((lo, hi)) => t_celsius < lo || t_celsius > hi,
        None => true,
    } {
        return None;
    }
    let z_total: f64 = z.iter().sum();
    if z_total <= 0.0 {
        return None;
    }
    let z: Vec<f64> = z.iter().map(|v| v / z_total).collect();
    let mut x_guess = z.clone();
    for _ in 0..60 {
        let g = gammas(&x_guess, t_celsius + KELVIN_OFFSET);
        if g.len() != antoines.len() || g.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return None;
        }
        let k: Vec<f64> = antoines
            .iter()
            .enumerate()
            .map(|(i, a)| {
                g.get(i).copied().unwrap_or(1.0) * a.pressure_kpa_unchecked(t_celsius)
                    / pressure_kpa
            })
            .collect();

        // Subcooled liquid: Σ zᵢ·Kᵢ ≤ 1. The liquid is the feed itself, so
        // γ(z) is already self-consistent and the answer stands.
        let sum_zk: f64 = z.iter().zip(&k).map(|(zi, ki)| zi * ki).sum();
        if sum_zk <= 1.0 {
            return Some(FlashResult {
                vapour_fraction: 0.0,
                x: z.clone(),
                y: z.iter().zip(&k).map(|(zi, ki)| zi * ki / sum_zk).collect(),
                k,
            });
        }
        // Superheated vapour: Σ zᵢ/Kᵢ ≤ 1. The trace liquid is dew-implied;
        // iterate its composition like the two-phase branch.
        let sum_z_over_k: f64 = z.iter().zip(&k).map(|(zi, ki)| zi / ki).sum();
        if sum_z_over_k <= 1.0 {
            let x_new: Vec<f64> = z
                .iter()
                .zip(&k)
                .map(|(zi, ki)| zi / ki / sum_z_over_k)
                .collect();
            let moved = x_guess
                .iter()
                .zip(&x_new)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            x_guess = x_new.clone();
            if moved < 1e-10 {
                return Some(FlashResult {
                    vapour_fraction: 1.0,
                    x: x_new,
                    y: z.clone(),
                    k,
                });
            }
            continue;
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

        let moved = x_guess
            .iter()
            .zip(&x)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        x_guess = x.clone();
        if moved < 1e-10 {
            return Some(FlashResult {
                vapour_fraction: v,
                x,
                y,
                k,
            });
        }
    }
    // Sixty passes without settling: refuse rather than publish a drifting
    // split as an equilibrium.
    None
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

/// Adiabatic (constant H, P) flash with fixed per-component γ: given
/// feed enthalpy and pressure, find the temperature and phase split.
pub fn hp_flash(
    components: &[FlashComponent],
    pressure_kpa: f64,
    feed_enthalpy_kj: f64,
    total_moles: f64,
) -> Option<HpFlashResult> {
    let g: Vec<f64> = components.iter().map(|c| c.volatile.gamma).collect();
    hp_flash_with(
        components,
        pressure_kpa,
        feed_enthalpy_kj,
        total_moles,
        &mut |_, _| g.clone(),
    )
}

/// The adiabatic flash with γ following the liquid composition and
/// temperature (CAP-16): the enthalpy bisection unchanged, every inner
/// TP flash running the γ–φ loop.
pub fn hp_flash_with(
    components: &[FlashComponent],
    pressure_kpa: f64,
    feed_enthalpy_kj: f64,
    total_moles: f64,
    gammas: &mut dyn FnMut(&[f64], f64) -> Vec<f64>,
) -> Option<HpFlashResult> {
    if components.is_empty()
        || !pressure_kpa.is_finite()
        || pressure_kpa <= 0.0
        || !feed_enthalpy_kj.is_finite()
        || !total_moles.is_finite()
        || total_moles <= 0.0
        || components.iter().any(|component| {
            !component.volatile.x.is_finite()
                || component.volatile.x < 0.0
                || !component.delta_hv_kj_per_mol.is_finite()
                || !component.cp_liquid_j_per_mol_k.is_finite()
        })
    {
        return None;
    }
    let antoines: Vec<VapourPressure> = components.iter().map(|c| c.volatile.antoine).collect();
    let z: Vec<f64> = components.iter().map(|c| c.volatile.x).collect();

    // Energy balance residual: H_feed - H(T, V) = 0
    // H(T, V) = Σ nᵢ [cp_L,i (T - T_ref) + V·yᵢ·ΔHv,i]
    let t_ref = 25.0; // reference temperature °C

    let enthalpy_at =
        |t: f64, gammas: &mut dyn FnMut(&[f64], f64) -> Vec<f64>| -> Option<(f64, FlashResult)> {
            let flash = tp_flash_with(&antoines, &z, pressure_kpa, t, gammas)?;
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
            Some((h, flash))
        };

    // Bisection on H(T) - H_feed = 0
    let (mut lo, mut hi) = common_valid_range(&antoines, &z)?;
    let (h_lo, _) = enthalpy_at(lo, gammas)?;
    let (h_hi, _) = enthalpy_at(hi, gammas)?;
    let residual_lo = h_lo - feed_enthalpy_kj;
    let residual_hi = h_hi - feed_enthalpy_kj;

    if residual_lo.signum() == residual_hi.signum() {
        // Enthalpy not bracketed — return the closest bound
        return None;
    }

    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let (h_mid, _) = enthalpy_at(mid, gammas)?;
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
    let (_, flash) = enthalpy_at(t, gammas)?;
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

/// Full-UNIFAC activity coefficients for the ethanol–water binary at a
/// liquid composition and temperature — the one seam every ethanol–water
/// helper shares, so the formulas cannot fork (the CAP-5 rule).
pub fn ethanol_water_activity(x_ethanol: f64, t_kelvin: f64) -> (f64, f64) {
    let table = crate::unifac::approved_table();
    let mut ethanol_groups = crate::unifac::GroupDecomposition::new();
    ethanol_groups.insert(1, 1); // CH3
    ethanol_groups.insert(2, 1); // CH2
    ethanol_groups.insert(14, 1); // OH
    let mut water_groups = crate::unifac::GroupDecomposition::new();
    water_groups.insert(16, 1); // H2O
    let g = crate::unifac::activity_coefficients(
        &table,
        &[(ethanol_groups, x_ethanol), (water_groups, 1.0 - x_ethanol)],
        t_kelvin,
    );
    (g[0], g[1])
}

/// Bubble point of the ethanol–water binary with full UNIFAC γ(T)
/// (Fredenslund 1975 parameters) — the mixture the school still is built
/// around, packaged so a bench does not need to know group
/// decompositions. `x_ethanol` is the ethanol mole fraction of the
/// volatile liquid.
pub fn ethanol_water_bubble_point(x_ethanol: f64, pressure_kpa: f64) -> Option<BubblePoint> {
    bubble_point_with(
        &[ETHANOL, WATER],
        &[x_ethanol, 1.0 - x_ethanol],
        pressure_kpa,
        |t_k| {
            let (ge, gw) = ethanol_water_activity(x_ethanol, t_k);
            vec![ge, gw]
        },
    )
}

/// Dew point of ethanol–water vapour with full UNIFAC γ(x, T): the γ of
/// the condensing liquid follows the successive-substitution loop in
/// [`dew_point_with`].
pub fn ethanol_water_dew_point(y_ethanol: f64, pressure_kpa: f64) -> Option<DewPoint> {
    dew_point_with(
        &[ETHANOL, WATER],
        &[y_ethanol, 1.0 - y_ethanol],
        pressure_kpa,
        &mut |x, t_k| {
            let (ge, gw) = ethanol_water_activity(x[0], t_k);
            vec![ge, gw]
        },
    )
}

/// TP flash of an ethanol–water feed with full UNIFAC γ(x, T).
pub fn ethanol_water_tp_flash(
    z_ethanol: f64,
    pressure_kpa: f64,
    t_celsius: f64,
) -> Option<FlashResult> {
    tp_flash_with(
        &[ETHANOL, WATER],
        &[z_ethanol, 1.0 - z_ethanol],
        pressure_kpa,
        t_celsius,
        &mut |x, t_k| {
            let (ge, gw) = ethanol_water_activity(x[0], t_k);
            vec![ge, gw]
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
    fn pure_isopropanol_boils_at_the_reviewed_temperature() {
        let mix = [Volatile {
            antoine: ISOPROPANOL,
            x: 1.0,
            gamma: 1.0,
        }];
        let bp = bubble_point(&mix, ATMOSPHERE_KPA).unwrap();
        assert!(
            (bp.t_celsius - 82.35).abs() < 0.5,
            "isopropanol boils at {:.2} °C (expected ~82.35)",
            bp.t_celsius
        );
        assert!(ISOPROPANOL.pressure_kpa(40.0).is_none());
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
                antoine: METHANOL,
                x: 0.5,
                gamma: 1.0,
            },
            Volatile {
                antoine: PROPANONE,
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
        // At 50°C and 10 kPa, everything is vapour without leaving either
        // component's fitted interval.
        let mix = [
            Volatile {
                antoine: METHANOL,
                x: 0.5,
                gamma: 1.0,
            },
            Volatile {
                antoine: PROPANONE,
                x: 0.5,
                gamma: 1.0,
            },
        ];
        let result = tp_flash(&mix, 10.0, 50.0).unwrap();
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
                antoine: METHANOL,
                x: 0.3,
                gamma: 1.0,
            },
            Volatile {
                antoine: PROPANONE,
                x: 0.7,
                gamma: 1.0,
            },
        ];
        let bp = bubble_point(&mix_bp, ATMOSPHERE_KPA).unwrap();

        // Flash just above the bubble point, inside this close-boiling
        // binary's narrow two-phase interval.
        let mix_flash = [
            Volatile {
                antoine: METHANOL,
                x: 0.3,
                gamma: 1.0,
            },
            Volatile {
                antoine: PROPANONE,
                x: 0.7,
                gamma: 1.0,
            },
        ];
        let result = tp_flash(&mix_flash, ATMOSPHERE_KPA, bp.t_celsius + 0.2).unwrap();
        assert!(
            result.vapour_fraction > 0.0 && result.vapour_fraction < 1.0,
            "should be two-phase just above T_bubble, V = {}",
            result.vapour_fraction
        );
        // The lower-boiling propanone should be enriched in the vapour.
        assert!(
            result.y[1] > result.x[1],
            "propanone should enrich in vapour: y={:.4} vs x={:.4}",
            result.y[1],
            result.x[1]
        );
    }

    #[test]
    fn flash_compositions_sum_to_one() {
        let mix = [
            Volatile {
                antoine: METHANOL,
                x: 0.4,
                gamma: 1.0,
            },
            Volatile {
                antoine: PROPANONE,
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

    #[test]
    fn antoine_range_endpoints_are_inclusive_and_nonfinite_is_refused() {
        let (lo, hi) = WATER.valid_range().unwrap();
        assert!(WATER.pressure_kpa(lo).is_some());
        assert!(WATER.pressure_kpa(hi).is_some());
        let immediately_below = f64::from_bits(lo.to_bits() - 1);
        assert!(WATER.pressure_kpa(immediately_below).is_none());
        assert!(WATER.pressure_kpa(f64::NAN).is_none());
        assert!(WATER.pressure_kpa(f64::INFINITY).is_none());
    }

    #[test]
    fn ethanol_piecewise_range_is_continuous_inclusive_and_bounded() {
        assert!(ETHANOL.pressure_kpa(ETHANOL_LOW.valid_c.0).is_some());
        assert!(ETHANOL.pressure_kpa(ETHANOL_HIGH.valid_c.1).is_some());
        assert!(ETHANOL
            .pressure_kpa(ETHANOL_HIGH.valid_c.1 + 1e-9)
            .is_none());

        let low = ETHANOL_LOW.pressure_kpa(80.0).unwrap();
        let high = ETHANOL_HIGH.pressure_kpa(80.0).unwrap();
        let relative_jump = (high - low).abs() / low;
        assert!(
            relative_jump < 0.0005,
            "80 °C segment jump must remain below 0.05%, got {:.5}%",
            relative_jump * 100.0
        );
    }

    #[test]
    fn water_rich_atmospheric_distillation_uses_high_ethanol_segment() {
        let bubble = ethanol_water_bubble_point(0.05, ATMOSPHERE_KPA)
            .expect("five mole-percent ethanol must have an in-range atmospheric root");
        assert!(bubble.t_celsius > 80.0 && bubble.t_celsius < 100.0);

        let cut = ethanol_water_still(0.95, 0.05, StillTake::Fraction(0.01), 1, ATMOSPHERE_KPA)
            .expect("water-rich wine-strength feed must no longer be range-refused");
        assert!(cut.t_start_c > 80.0 && cut.t_start_c < 100.0);
        assert!(cut.ethanol_over > 0.0);
    }

    #[test]
    fn solvers_refuse_non_overlapping_antoine_ranges() {
        let low = Antoine {
            valid_c: (0.0, 10.0),
            ..WATER_ANTOINE
        };
        let high = Antoine {
            valid_c: (20.0, 30.0),
            ..ETHANOL_LOW
        };
        let mix = [
            Volatile {
                antoine: low.into(),
                x: 0.5,
                gamma: 1.0,
            },
            Volatile {
                antoine: high.into(),
                x: 0.5,
                gamma: 1.0,
            },
        ];
        assert!(bubble_point(&mix, ATMOSPHERE_KPA).is_none());
        assert!(dew_point(&mix, ATMOSPHERE_KPA).is_none());
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, 15.0).is_none());
    }

    #[test]
    fn tp_flash_refuses_temperature_outside_any_component_fit() {
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
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, 0.0).is_none());
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, 80.0).is_some());
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, 90.0).is_some());
        let immediately_above = f64::from_bits(100.0f64.to_bits() + 1);
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, immediately_above).is_none());
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, f64::NAN).is_none());
    }

    #[test]
    fn solvers_refuse_nonfinite_inputs_and_activity_coefficients() {
        let mix = [Volatile {
            antoine: WATER,
            x: 1.0,
            gamma: f64::NAN,
        }];
        assert!(bubble_point(&mix, ATMOSPHERE_KPA).is_none());
        assert!(dew_point(&mix, ATMOSPHERE_KPA).is_none());
        assert!(tp_flash(&mix, ATMOSPHERE_KPA, 50.0).is_none());
        assert!(bubble_point(&mix, f64::INFINITY).is_none());
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
