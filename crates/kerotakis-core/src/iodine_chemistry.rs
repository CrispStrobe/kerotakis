//! Iodine family (2026-11-10): triiodide association, iodine optics and the
//! ferric/iodide equilibrium, as one generic module over the analytical
//! inventory.
//!
//! The ledger keeps dissolved iodine as one analytical pool, `I2(aq)`, and
//! iodide as `I-` (or an undissociated Na/K iodide salt). Every curated row,
//! rate law and halogen writer already reads and writes those keys, so the
//! triiodide ion is NOT a separate inventory entry: it is a computed split of
//! the two pools, used wherever the split changes an observable:
//!
//! * the dissolved-iodine capacity (iodine is far more soluble in iodide
//!   solution, which is what Lugol's solution is);
//! * the visible absorbance (free I2 and I3- absorb very differently);
//! * the ferric/iodide equilibrium (which consumes free iodide).
//!
//! Sources (constants at 25 °C, ideal activities = molarities):
//!
//! * `I2 + I- ⇌ I3-`, K = 698 (molal, used here as L/mol): Palmer, Ramette
//!   & Mesmer, *J. Solution Chem.* 13 (1984) 673, doi:10.1007/BF00650374. (Earlier
//!   spectrophotometric values span 698–770; Awtrey & Connick,
//!   *J. Am. Chem. Soc.* 73 (1951) 1842, doi:10.1021/ja01148a097.)
//! * ε(I2, 460 nm) = 746 and ε(I3-, 460 nm) = 975 L mol⁻¹ cm⁻¹, ε(I3-,
//!   353 nm) = 26 400 L mol⁻¹ cm⁻¹: Awtrey & Connick (1951), as above.
//!   The I3- 353 nm value is confirmed by later sources (24 800–26 400);
//!   the I2 460 nm value of 746 is NOT yet confirmed by a second source.
//! * E°(Fe³⁺/Fe²⁺) from the halogen redox half table (llnl.dat, 0.7696 V);
//!   E°(I2(aq)/I⁻) = 0.621 V: Bard, Parsons & Jordan, *Standard Potentials
//!   in Aqueous Solution* (IUPAC/Dekker, 1985).
//!
//! The starch–iodine complex keeps its editorial eye colour in
//! [`crate::starch_iodine`]; no reviewed molar absorptivity exists for it
//! here, so the spectrophotometer refuses a vessel holding the complex with
//! a typed reason instead of a number.

use crate::species::Phase;
use crate::vessel::Vessel;
use crate::{Moles, SpeciesId};

/// Triiodide association constant at 25 °C, L/mol (Palmer et al. 1984).
pub const TRIIODIDE_K: f64 = 698.0;

/// Free I2(aq) visible band: maximum 460 nm, ε 746 L mol⁻¹ cm⁻¹ (Awtrey &
/// Connick 1951); the width is the band's ≈100 nm FWHM as a Gaussian σ.
pub const I2_BAND: (f64, f64, f64) = (460.0, 746.0, 42.0);

/// I3⁻: the 353 nm band (ε 26 400) whose visible tail colours the solution.
/// σ is FITTED so that ε(460 nm) = 975, the second cited point:
/// σ = 107 / sqrt(2 ln(26400/975)) = 41.66 nm.
pub const I3_BAND: (f64, f64, f64) = (353.0, 26_400.0, 41.66);

/// E°(I2(aq)/I⁻), V (Bard, Parsons & Jordan 1985).
pub const E0_I2AQ_IODIDE_V: f64 = 0.621;

/// Nernst slope at 25 °C, V per decade (RT ln10 / F).
const NERNST_V: f64 = 0.059_16;

/// Registry keys whose visible absorbance this module supplies.
pub const OPTICAL_KEYS: &[&str] = &["I2"];

const IODINE: &str = "I2";
const TRACE: f64 = 1e-12;

/// The triiodide split of dissolved iodine and iodide, in moles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriiodideSplit {
    /// Free I2(aq).
    pub iodine: f64,
    /// I3⁻ (each holds one I2 and one I⁻ of the analytical pools).
    pub triiodide: f64,
    /// Free I⁻.
    pub iodide: f64,
}

/// Split `iodine_total` mol of dissolved I2 and `iodide_total` mol of
/// iodide in `litres` of solution by `I2 + I- ⇌ I3-`. Mass balance is exact
/// by construction: iodine + triiodide = iodine_total and iodide +
/// triiodide = iodide_total.
pub fn split(iodine_total: f64, iodide_total: f64, litres: f64) -> TriiodideSplit {
    let t = iodine_total.max(0.0);
    let b = iodide_total.max(0.0);
    if t <= 0.0 || b <= 0.0 || litres.is_nan() || litres <= 0.0 {
        return TriiodideSplit {
            iodine: t,
            triiodide: 0.0,
            iodide: b,
        };
    }
    // K z = (T - z)(B - z) / V in moles: K' = K / V.
    let k = TRIIODIDE_K / litres;
    let s = k * (t + b) + 1.0;
    let disc = (s * s - 4.0 * k * k * t * b).max(0.0);
    // Stable root of k z² - s z + k T B = 0.
    let z = (2.0 * k * t * b / (s + disc.sqrt())).clamp(0.0, t.min(b));
    TriiodideSplit {
        iodine: t - z,
        triiodide: z,
        iodide: b - z,
    }
}

/// Iodide in solution (moles of I⁻): the ion plus every dissolved Na/K
/// iodide salt the halogen table derives from the registry.
pub fn dissolved_iodide(vessel: &Vessel) -> f64 {
    crate::halogen_redox::registry_forms("I-")
        .iter()
        .map(|key| {
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == *key && p.phase == Phase::Aqueous)
                .map(|p| p.moles.0)
                .sum::<f64>()
        })
        .sum()
}

fn aqueous(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

fn liquid_water(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

/// The triiodide split of this vessel's dissolved iodine, or `None` when it
/// holds no liquid water.
pub fn speciate(vessel: &Vessel) -> Option<TriiodideSplit> {
    if liquid_water(vessel) <= 0.0 {
        return None;
    }
    Some(split(
        aqueous(vessel, IODINE),
        dissolved_iodide(vessel),
        vessel.liquid_volume().0.max(1e-9),
    ))
}

/// Dissolved-iodine capacity (mol) in iodide solution, from the pure-water
/// capacity `pure` (mol) in `water_litres`: at saturation free I2 is at its
/// pure-water solubility S0, and I3⁻ adds K·S0·[I⁻]free with
/// [I⁻]free = I_total / (1 + K·S0). Without iodide this is `pure` exactly.
pub fn enhanced_capacity(vessel: &Vessel, pure: f64, water_litres: f64) -> f64 {
    if water_litres.is_nan() || water_litres <= 0.0 || pure.is_nan() || pure <= 0.0 {
        return pure;
    }
    let iodide = dissolved_iodide(vessel) / water_litres;
    if iodide <= 0.0 {
        return pure;
    }
    let s0 = pure / water_litres;
    pure * (1.0 + TRIIODIDE_K * iodide / (1.0 + TRIIODIDE_K * s0))
}

/// Pure-water I2 capacity (mol) and the water volume (L), read exactly as
/// `solve::saturation_moves` reads every registry solubility.
pub fn pure_water_capacity(vessel: &Vessel) -> Option<(f64, f64)> {
    let water = liquid_water(vessel);
    if water <= 0.0 {
        return None;
    }
    let water_ml = crate::species::lookup_key("water")?
        .liters_from_moles(Moles(water))
        .0
        * 1000.0;
    let data = crate::species::lookup(&SpeciesId::new(IODINE))?;
    let limit = data.aqueous_solubility_at(vessel.temperature.0)?;
    Some((
        limit * water_ml / 100.0 / data.molar_mass,
        water_ml / 1000.0,
    ))
}

/// Solid I2 that dissolves now: up to the triiodide-enhanced capacity.
pub(crate) fn iodine_to_dissolve(vessel: &Vessel) -> Option<Moles> {
    let (pure, litres) = pure_water_capacity(vessel)?;
    let capacity = enhanced_capacity(vessel, pure, litres);
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == IODINE && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    Some(Moles(
        solid.min((capacity - aqueous(vessel, IODINE)).max(0.0)),
    ))
}

/// Add free-I2 and I3⁻ absorbance. Iodine bound in the starch complex
/// (`complex` mol, one I2 and one I⁻ each) is excluded: the complex has its
/// own band.
pub fn add_absorbance(
    vessel: &Vessel,
    litres: f64,
    path_cm: f64,
    complex: f64,
    absorbance: &mut crate::spectrum::Spectrum,
) {
    if liquid_water(vessel) <= 0.0 {
        return;
    }
    let litres = litres.max(1e-12);
    let parts = split(
        (aqueous(vessel, IODINE) - complex).max(0.0),
        (dissolved_iodide(vessel) - complex).max(0.0),
        litres,
    );
    for (moles, band) in [(parts.iodine, I2_BAND), (parts.triiodide, I3_BAND)] {
        if moles <= 0.0 {
            continue;
        }
        let spectrum = crate::spectrum::bands(&[band]);
        for (total, epsilon) in absorbance.iter_mut().zip(spectrum) {
            *total += epsilon * moles / litres * path_cm;
        }
    }
}

// ---------------------------------------------------------------------------
// Fe³⁺/I⁻: 2 Fe³⁺ + 2 I⁻ ⇌ 2 Fe²⁺ + I2(aq)
// ---------------------------------------------------------------------------

const FERRIC: &str = "Fe+3";
const FERROUS: &str = "Fe+2";
const IODIDE: &str = "I-";

/// log10 K for 2 Fe³⁺ + 2 I⁻ ⇌ 2 Fe²⁺ + I2(aq), from the two E° values.
pub fn ferric_iodide_log_k() -> f64 {
    let fe = crate::halogen_redox::half_by_id("Fe+3/Fe+2")
        .map(|h| h.e0_v)
        .unwrap_or(0.7696);
    2.0 * (fe - E0_I2AQ_IODIDE_V) / NERNST_V
}

/// Signed extent (mol) that brings this vessel to the ferric/iodide
/// equilibrium: positive oxidises iodide, negative reduces iodine. Zero when
/// the medium is not aqueous, outside the 25 °C constants' domain, or
/// nothing can move.
pub fn ferric_iodide_extent(vessel: &Vessel) -> f64 {
    let (lo_k, hi_k) = crate::halogen_redox::DOMAIN_K;
    if liquid_water(vessel) <= 0.0 || !(lo_k..=hi_k).contains(&vessel.temperature.0) {
        return 0.0;
    }
    let a = aqueous(vessel, FERRIC);
    let b = aqueous(vessel, IODIDE);
    let c = aqueous(vessel, FERROUS);
    let d = aqueous(vessel, IODINE);
    // Forward needs Fe³⁺ and I⁻; reverse needs Fe²⁺ and I2.
    let hi = (a / 2.0).min(b / 2.0);
    let lo = -(c / 2.0).min(d);
    if hi <= TRACE && -lo <= TRACE {
        return 0.0;
    }
    let litres = vessel.liquid_volume().0.max(1e-9);
    let ln_k = ferric_iodide_log_k() * std::f64::consts::LN_10;
    // g(x) = ln Q(x) - ln K, increasing in x; Q uses FREE I2 and I⁻ from
    // the triiodide split (iodide bound as I3⁻ is not available).
    let g = |x: f64| {
        let parts = split(d + x, b - 2.0 * x, litres);
        let fe3 = (a - 2.0 * x) / litres;
        let fe2 = (c + 2.0 * x) / litres;
        2.0 * fe2.ln() + (parts.iodine / litres).ln()
            - 2.0 * fe3.ln()
            - 2.0 * (parts.iodide / litres).ln()
            - ln_k
    };
    let (mut low, mut high) = (lo, hi);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if mid == low || mid == high {
            break;
        }
        let value = g(mid);
        if value.is_nan() {
            break;
        }
        if value > 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }
    let x = 0.5 * (low + high);
    let scale = a.max(b).max(c).max(d);
    if x.abs() <= TRACE.max(1e-9 * scale) {
        0.0
    } else {
        x
    }
}

/// Whether the ferric/iodide equilibrium would move this vessel.
pub fn ferric_iodide_fires(vessel: &Vessel) -> bool {
    ferric_iodide_extent(vessel) != 0.0
}

/// Move the vessel to the ferric/iodide equilibrium. Element and charge
/// closure is exact: 2 Fe³⁺ + 2 I⁻ (charge +4) ↔ 2 Fe²⁺ + I2 (charge +4).
/// The caller refreshes `solute_charge`. No reaction heat is charged.
pub fn ferric_iodide_apply(vessel: &mut Vessel, events: &mut Vec<crate::ops::Event>) -> bool {
    let x = ferric_iodide_extent(vessel);
    if x == 0.0 {
        return false;
    }
    let (from, to, equation) = if x > 0.0 {
        (
            [(FERRIC, 2.0), (IODIDE, 2.0)],
            [(FERROUS, 2.0), (IODINE, 1.0)],
            "2 Fe³⁺ + 2 I⁻ → 2 Fe²⁺ + I₂",
        )
    } else {
        (
            [(FERROUS, 2.0), (IODINE, 1.0)],
            [(FERRIC, 2.0), (IODIDE, 2.0)],
            "2 Fe²⁺ + I₂ → 2 Fe³⁺ + 2 I⁻",
        )
    };
    let x = x.abs();
    for (key, n) in from {
        vessel.withdraw_phase(&SpeciesId::new(key), Moles(x * n), Phase::Aqueous);
    }
    for (key, n) in to {
        vessel.deposit(SpeciesId::new(key), Moles(x * n), Phase::Aqueous);
    }
    events.push(crate::ops::Event::ReactionOccurred {
        vessel: vessel.id,
        equation: equation.to_string(),
    });
    vessel.resolved.invalidate();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_closes_both_pools_and_meets_k() {
        for (t, b, v) in [(1e-4, 1e-3, 0.1), (3e-4, 3e-3, 0.03), (1e-3, 1e-5, 1.0)] {
            let s = split(t, b, v);
            assert!((s.iodine + s.triiodide - t).abs() <= 1e-18);
            assert!((s.iodide + s.triiodide - b).abs() <= 1e-18);
            let k = (s.triiodide / v) / ((s.iodine / v) * (s.iodide / v));
            assert!((k / TRIIODIDE_K - 1.0).abs() < 1e-8, "{k}");
        }
    }

    #[test]
    fn no_iodide_no_triiodide() {
        let s = split(1e-4, 0.0, 0.1);
        assert_eq!(s.triiodide, 0.0);
        assert_eq!(s.iodine, 1e-4);
    }

    #[test]
    fn i3_band_fit_reproduces_the_second_cited_point() {
        let (centre, peak, sigma) = I3_BAND;
        let t = (460.0 - centre) / sigma;
        let at_460 = peak * (-0.5 * t * t).exp();
        assert!((at_460 - 975.0).abs() < 5.0, "{at_460}");
    }

    #[test]
    fn ferric_iodide_constant_is_from_the_two_potentials() {
        let log_k = ferric_iodide_log_k();
        assert!((log_k - 2.0 * (0.7696 - 0.621) / 0.05916).abs() < 1e-9);
    }
}
