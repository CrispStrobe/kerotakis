//! Optional FeOs (PC-SAFT) adapter for Kerotakis.
//!
//! One property query: the bubble or dew point of a binary alcohol–water
//! mixture at a given pressure, from the PC-SAFT equation of state
//! (Gross & Sadowski 2002 parameters, 2B association, k_ij = 0) as solved by
//! FeOs. It is a *query*, not a rewiring of the default physics path: the
//! UNIFAC/Antoine still in `kerotakis-thermo` stays authoritative; this crate
//! answers "what would PC-SAFT say here?" for comparison and opt-in use.
//!
//! The domain is explicit. Anything outside it — another species, a
//! pressure or composition outside the admitted box, or a solved
//! temperature outside the admitted window — is a typed [`Refusal`], never
//! an extrapolated number. Without the `feos` cargo feature every query
//! refuses with [`Refusal::EngineNotEnabled`].

use thiserror::Error;

/// Provenance phrase carried with every answer, in the style of the other
/// engine crates.
pub const PROVENANCE: &str = "PC-SAFT (Gross & Sadowski 2002, 2B association, k_ij = 0) \
     solved by FeOs 0.10.1 (MIT OR Apache-2.0)";

/// Lowest admitted pressure, kPa.
pub const MIN_PRESSURE_KPA: f64 = 10.0;
/// Highest admitted pressure, kPa.
pub const MAX_PRESSURE_KPA: f64 = 1000.0;
/// Smallest admitted mole fraction of either component. Infinite-dilution
/// endpoints are not part of this query.
pub const MIN_FRACTION: f64 = 1e-3;
/// Admitted temperature window for a solved bubble/dew point, K. Below it
/// the liquid may freeze; above it the parameters were not regressed.
pub const MIN_TEMPERATURE_K: f64 = 273.15;
/// Upper end of the admitted temperature window, K.
pub const MAX_TEMPERATURE_K: f64 = 523.15;

/// Alcohols this adapter carries PC-SAFT parameters for. Water is always
/// the second component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alcohol {
    Methanol,
    Ethanol,
}

impl Alcohol {
    /// Resolve a species name; anything not carried is `None`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "methanol" | "ch3oh" | "meoh" => Some(Self::Methanol),
            "ethanol" | "c2h5oh" | "etoh" => Some(Self::Ethanol),
            _ => None,
        }
    }
}

/// PC-SAFT pure-component record in FeOs `PcSaftBinary<f64, 8>` order:
/// m, sigma (Å), eps/k (K), mu (D), kappa_AB, eps_AB/k (K), n_A, n_B.
/// Source: Gross & Sadowski, Ind. Eng. Chem. Res. 41 (2002) 5510, Table 1;
/// verified identical to feos-org/feos `parameters/pcsaft/gross2002.json`
/// at commit aabc02bbf6c469cf702206cfa68bbb7b2105a324.
pub type PcSaftRecord = [f64; 8];

pub const METHANOL: PcSaftRecord = [1.5255, 3.2300, 188.90, 0.0, 0.035176, 2899.5, 1.0, 1.0];
pub const ETHANOL: PcSaftRecord = [2.3827, 3.1771, 198.24, 0.0, 0.032384, 2653.4, 1.0, 1.0];
pub const WATER: PcSaftRecord = [1.0656, 3.0007, 366.51, 0.0, 0.034868, 2500.7, 1.0, 1.0];
/// Unfitted: pure prediction (`gross2002_binary.json` has no alcohol-water entry).
pub const K_IJ: f64 = 0.0;

#[cfg(feature = "feos")]
fn record(alcohol: Alcohol) -> PcSaftRecord {
    match alcohol {
        Alcohol::Methanol => METHANOL,
        Alcohol::Ethanol => ETHANOL,
    }
}

/// Why a query was not answered.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum Refusal {
    #[error("FeOs engine not compiled in (enable the `feos` feature)")]
    EngineNotEnabled,
    #[error("species `{0}` has no PC-SAFT parameters in this adapter")]
    UnknownSpecies(String),
    #[error("pressure {0} kPa outside the admitted {MIN_PRESSURE_KPA}-{MAX_PRESSURE_KPA} kPa")]
    PressureOutOfDomain(f64),
    #[error("mole fraction {0} outside the admitted [{MIN_FRACTION}, {}]", 1.0 - MIN_FRACTION)]
    CompositionOutOfDomain(f64),
    #[error("solved temperature {0} K outside the admitted {MIN_TEMPERATURE_K}-{MAX_TEMPERATURE_K} K")]
    TemperatureOutOfDomain(f64),
    #[error("FeOs did not converge: {0}")]
    NotConverged(String),
}

/// Which saturation point is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saturation {
    /// Given liquid composition, find T and the first vapour.
    Bubble,
    /// Given vapour composition, find T and the first liquid drop.
    Dew,
}

/// A PC-SAFT saturation point of an alcohol–water binary.
#[derive(Debug, Clone, PartialEq)]
pub struct SaturationPoint {
    pub t_kelvin: f64,
    pub pressure_kpa: f64,
    /// Alcohol mole fraction in the liquid.
    pub x_alcohol: f64,
    /// Alcohol mole fraction in the vapour.
    pub y_alcohol: f64,
    pub provenance: &'static str,
}

impl SaturationPoint {
    pub fn t_celsius(&self) -> f64 {
        self.t_kelvin - 273.15
    }
}

/// Property query by species name; see [`saturation_point`].
pub fn saturation_point_by_name(
    alcohol: &str,
    other: &str,
    kind: Saturation,
    alcohol_fraction: f64,
    pressure_kpa: f64,
) -> Result<SaturationPoint, Refusal> {
    let a = Alcohol::from_name(alcohol).ok_or_else(|| Refusal::UnknownSpecies(alcohol.into()))?;
    if !matches!(other.trim().to_ascii_lowercase().as_str(), "water" | "h2o") {
        return Err(Refusal::UnknownSpecies(other.into()));
    }
    saturation_point(a, kind, alcohol_fraction, pressure_kpa)
}

/// Bubble or dew point of an alcohol–water binary at `pressure_kpa`, with
/// `alcohol_fraction` the alcohol mole fraction of the given phase (liquid
/// for [`Saturation::Bubble`], vapour for [`Saturation::Dew`]).
pub fn saturation_point(
    alcohol: Alcohol,
    kind: Saturation,
    alcohol_fraction: f64,
    pressure_kpa: f64,
) -> Result<SaturationPoint, Refusal> {
    admit(alcohol_fraction, pressure_kpa)?;
    let point = solve(alcohol, kind, alcohol_fraction, pressure_kpa)?;
    let t = point.t_kelvin;
    if !t.is_finite() || !(MIN_TEMPERATURE_K..=MAX_TEMPERATURE_K).contains(&t) {
        return Err(Refusal::TemperatureOutOfDomain(t));
    }
    Ok(point)
}

fn admit(fraction: f64, pressure_kpa: f64) -> Result<(), Refusal> {
    if !pressure_kpa.is_finite() || !(MIN_PRESSURE_KPA..=MAX_PRESSURE_KPA).contains(&pressure_kpa) {
        return Err(Refusal::PressureOutOfDomain(pressure_kpa));
    }
    if !fraction.is_finite() || !(MIN_FRACTION..=1.0 - MIN_FRACTION).contains(&fraction) {
        return Err(Refusal::CompositionOutOfDomain(fraction));
    }
    Ok(())
}

#[cfg(not(feature = "feos"))]
fn solve(_: Alcohol, _: Saturation, _: f64, _: f64) -> Result<SaturationPoint, Refusal> {
    Err(Refusal::EngineNotEnabled)
}

#[cfg(feature = "feos")]
fn solve(
    alcohol: Alcohol,
    kind: Saturation,
    fraction: f64,
    pressure_kpa: f64,
) -> Result<SaturationPoint, Refusal> {
    use feos::core::PhaseEquilibrium;
    use feos::pcsaft::PcSaftBinary;
    use quantity::{KELVIN, KILO, PASCAL};

    let eos = PcSaftBinary::new([record(alcohol), WATER], K_IJ);
    let p = pressure_kpa * KILO * PASCAL;
    // Start the temperature iteration between the two pure boiling points
    // at 1 atm; FeOs refines it.
    let t0 = Some(360.0 * KELVIN);
    let vle = match kind {
        Saturation::Bubble => {
            PhaseEquilibrium::bubble_point(&eos, p, fraction, t0, None, Default::default())
        }
        Saturation::Dew => {
            PhaseEquilibrium::dew_point(&eos, p, fraction, t0, None, Default::default())
        }
    }
    .map_err(|e| Refusal::NotConverged(e.to_string()))?;
    let x = vle.liquid().molefracs[0];
    let y = vle.vapor().molefracs[0];
    let t_kelvin = vle.vapor().temperature.convert_into(KELVIN);
    if !(x.is_finite() && y.is_finite()) || ((x - y).abs() < 1e-9 && (fraction - 0.5).abs() > 0.4) {
        // A trivial solution (both phases identical) away from any
        // azeotrope is a solver artefact, not an answer.
        return Err(Refusal::NotConverged("trivial solution".into()));
    }
    Ok(SaturationPoint {
        t_kelvin,
        pressure_kpa,
        x_alcohol: x,
        y_alcohol: y,
        provenance: PROVENANCE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_resolve_and_others_refuse() {
        assert_eq!(Alcohol::from_name(" Ethanol "), Some(Alcohol::Ethanol));
        assert_eq!(Alcohol::from_name("MeOH"), Some(Alcohol::Methanol));
        assert!(matches!(
            saturation_point_by_name("propanol", "water", Saturation::Bubble, 0.5, 101.325),
            Err(Refusal::UnknownSpecies(_))
        ));
        assert!(matches!(
            saturation_point_by_name("ethanol", "benzene", Saturation::Bubble, 0.5, 101.325),
            Err(Refusal::UnknownSpecies(_))
        ));
    }

    #[test]
    fn out_of_domain_inputs_refuse_before_any_solve() {
        let q = |x, p| saturation_point(Alcohol::Ethanol, Saturation::Bubble, x, p);
        assert!(matches!(q(0.5, 5.0), Err(Refusal::PressureOutOfDomain(_))));
        assert!(matches!(q(0.5, 5000.0), Err(Refusal::PressureOutOfDomain(_))));
        assert!(matches!(q(0.5, f64::NAN), Err(Refusal::PressureOutOfDomain(_))));
        assert!(matches!(q(0.0, 101.325), Err(Refusal::CompositionOutOfDomain(_))));
        assert!(matches!(q(1.0, 101.325), Err(Refusal::CompositionOutOfDomain(_))));
        assert!(matches!(q(-0.2, 101.325), Err(Refusal::CompositionOutOfDomain(_))));
    }

    #[cfg(not(feature = "feos"))]
    #[test]
    fn without_feature_in_domain_queries_refuse_typed() {
        assert_eq!(
            saturation_point(Alcohol::Ethanol, Saturation::Bubble, 0.5, 101.325),
            Err(Refusal::EngineNotEnabled)
        );
    }
}
