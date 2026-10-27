//! Optional FeOs (PC-SAFT) adapter for Kerotakis.
//!
//! One property query: the bubble or dew point of a binary alcohol–water
//! mixture at a given pressure, from the PC-SAFT equation of state
//! (Esper et al. 2023 pure + Rehner et al. 2023 binary parameters) as solved by
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
pub const PROVENANCE: &str = "PC-SAFT (Esper et al. 2023 pure, Rehner et al. 2023 binary records) \
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

/// Methanol, ethanol and water pure-component PC-SAFT records of Esper et
/// al. (2023), Ind. Eng. Chem. Res., doi:10.1021/acs.iecr.3c02255, extracted
/// unchanged by `tools/extract_feos_params.py` from feos-org/feos
/// `parameters/pcsaft/esper2023.json` at commit
/// aabc02bbf6c469cf702206cfa68bbb7b2105a324 (full-file sha256
/// 87fdcd8733245d90cd6ad9e9d7d0583bd3f7ffb3c4d67a25197190f289e2a679).
/// Only compiled with the `feos` feature.
#[cfg(feature = "feos")]
const ESPER2023_JSON: &str = include_str!("../data/esper2023.subset.json");

/// The methanol-water and ethanol-water binary records fitted on that pure
/// set by Rehner et al. (2023), Int. J. Thermophys.,
/// doi:10.1007/s10765-023-03290-3, extracted unchanged by the same script
/// from `parameters/pcsaft/rehner2023_binary.json` at the same commit
/// (full-file sha256
/// 8b7d008b2bdf7ec49542eb443e36078fbe46e2737775126589c1718dfa13c335).
/// Ethanol-water carries cross-association (kappa_ab, epsilon_k_ab, no
/// k_ij); methanol-water carries k_ij.
#[cfg(feature = "feos")]
const REHNER2023_BINARY_JSON: &str = include_str!("../data/rehner2023_binary.subset.json");

/// Name of the parameter record for each carried component.
#[cfg_attr(not(feature = "feos"), allow(dead_code))]
fn record_name(alcohol: Option<Alcohol>) -> &'static str {
    match alcohol {
        Some(Alcohol::Methanol) => "methanol",
        Some(Alcohol::Ethanol) => "ethanol",
        None => "water",
    }
}

#[cfg(feature = "feos")]
mod engine {
    use super::{record_name, Alcohol, Refusal, ESPER2023_JSON, REHNER2023_BINARY_JSON};
    use feos::core::parameter::{BinaryRecord, Identifier, PureRecord};
    use feos::pcsaft::{
        PcSaft, PcSaftAssociationRecord, PcSaftBinaryRecord, PcSaftParameters, PcSaftRecord,
    };
    use std::sync::OnceLock;

    type Pure = PureRecord<PcSaftRecord, PcSaftAssociationRecord>;
    type Binary = BinaryRecord<Identifier, PcSaftBinaryRecord, PcSaftAssociationRecord>;

    /// `{"_source": {...}, "records": [...]}` as written by the extractor.
    #[derive(serde::Deserialize)]
    struct Subset<T> {
        records: Vec<T>,
    }

    fn bad(e: impl std::fmt::Display) -> Refusal {
        Refusal::NotConverged(format!("parameter file: {e}"))
    }

    /// The three carried pure records, parsed once.
    fn pures() -> Result<&'static [Pure], Refusal> {
        static CELL: OnceLock<Result<Vec<Pure>, String>> = OnceLock::new();
        CELL.get_or_init(|| {
            let all: Vec<Pure> = serde_json::from_str::<Subset<Pure>>(ESPER2023_JSON)
                .map_err(|e| e.to_string())?
                .records;
            Ok(all
                .into_iter()
                .filter(|r| {
                    matches!(
                        r.identifier.name.as_deref(),
                        Some("methanol" | "ethanol" | "water")
                    )
                })
                .collect())
        })
        .as_deref()
        .map_err(bad)
    }

    /// The alcohol-water binary records, parsed once.
    fn binaries() -> Result<&'static [Binary], Refusal> {
        static CELL: OnceLock<Result<Vec<Binary>, String>> = OnceLock::new();
        CELL.get_or_init(|| {
            let all: Vec<Binary> = serde_json::from_str::<Subset<Binary>>(REHNER2023_BINARY_JSON)
                .map_err(|e| e.to_string())?
                .records;
            Ok(all
                .into_iter()
                .filter(|r| {
                    let n = [r.id1.name.as_deref(), r.id2.name.as_deref()];
                    n.contains(&Some("water"))
                        && (n.contains(&Some("ethanol")) || n.contains(&Some("methanol")))
                })
                .collect())
        })
        .as_deref()
        .map_err(bad)
    }

    fn record(component: Option<Alcohol>) -> Result<Pure, Refusal> {
        let name = record_name(component);
        pures()?
            .iter()
            .find(|r| r.identifier.name.as_deref() == Some(name))
            .cloned()
            .ok_or_else(|| Refusal::UnknownSpecies(name.into()))
    }

    /// Published binary record for alcohol + water. Missing is a refusal,
    /// never a silent k_ij = 0.
    fn binary(alcohol: Alcohol) -> Result<Binary, Refusal> {
        let a = record_name(Some(alcohol));
        binaries()?
            .iter()
            .find(|r| {
                let n = [r.id1.name.as_deref(), r.id2.name.as_deref()];
                n.contains(&Some(a)) && n.contains(&Some("water"))
            })
            .cloned()
            .ok_or_else(|| Refusal::UnknownSpecies(format!("{a}-water binary record")))
    }

    /// Single-component PC-SAFT equation of state.
    pub(crate) fn pure_eos(component: Option<Alcohol>) -> Result<PcSaft, Refusal> {
        let params = PcSaftParameters::new_pure(record(component)?)
            .map_err(|e| Refusal::NotConverged(e.to_string()))?;
        Ok(PcSaft::new(params))
    }

    /// Alcohol (component 0) + water (component 1) with the published binary
    /// record. Both carried pairs are symmetric (k_ij, and cross-association
    /// between single 2B site types with empty site ids), so the record's
    /// id1/id2 order does not matter.
    pub(crate) fn binary_eos(alcohol: Alcohol) -> Result<PcSaft, Refusal> {
        let b = binary(alcohol)?;
        let params = PcSaftParameters::new_binary(
            [record(Some(alcohol))?, record(None)?],
            b.model_record,
            b.association_sites,
        )
        .map_err(|e| Refusal::NotConverged(e.to_string()))?;
        Ok(PcSaft::new(params))
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
    #[error("mole fraction {0} outside the admitted [{MIN_FRACTION}, {max}]", max = 1.0 - MIN_FRACTION)]
    CompositionOutOfDomain(f64),
    #[error(
        "solved temperature {0} K outside the admitted {MIN_TEMPERATURE_K}-{MAX_TEMPERATURE_K} K"
    )]
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

/// Pure-component PC-SAFT saturation, for diagnostics: `None` = water.
#[derive(Debug, Clone, PartialEq)]
pub struct PureSaturation {
    pub t_kelvin: f64,
    pub pressure_kpa: f64,
}

/// Normal-pressure-style query: saturation temperature at `pressure_kpa`.
#[cfg(feature = "feos")]
pub fn pure_boiling_point(
    component: Option<Alcohol>,
    pressure_kpa: f64,
) -> Result<PureSaturation, Refusal> {
    use feos::core::PhaseEquilibrium;
    use quantity::{KELVIN, KILO, PASCAL};
    let eos = engine::pure_eos(component)?;
    let vle = PhaseEquilibrium::pure(
        &&eos,
        pressure_kpa * KILO * PASCAL,
        None,
        Default::default(),
    )
    .map_err(|e| Refusal::NotConverged(e.to_string()))?;
    Ok(PureSaturation {
        t_kelvin: vle.vapor().temperature.convert_into(KELVIN),
        pressure_kpa,
    })
}

/// Vapour pressure at `t_kelvin`.
#[cfg(feature = "feos")]
pub fn pure_vapour_pressure(
    component: Option<Alcohol>,
    t_kelvin: f64,
) -> Result<PureSaturation, Refusal> {
    use feos::core::{Contributions, PhaseEquilibrium};
    use quantity::{KELVIN, KILO, PASCAL};
    let eos = engine::pure_eos(component)?;
    let vle = PhaseEquilibrium::pure(&&eos, t_kelvin * KELVIN, None, Default::default())
        .map_err(|e| Refusal::NotConverged(e.to_string()))?;
    Ok(PureSaturation {
        t_kelvin,
        pressure_kpa: vle
            .vapor()
            .pressure(Contributions::Total)
            .convert_into(KILO * PASCAL),
    })
}

#[cfg(feature = "feos")]
fn solve(
    alcohol: Alcohol,
    kind: Saturation,
    fraction: f64,
    pressure_kpa: f64,
) -> Result<SaturationPoint, Refusal> {
    use feos::core::PhaseEquilibrium;
    use quantity::{KELVIN, KILO, PASCAL};

    let eos = engine::binary_eos(alcohol)?;
    let p = pressure_kpa * KILO * PASCAL;
    // FeOs needs, at fixed pressure, an initial temperature and (to avoid
    // collapsing onto the trivial solution) an initial composition of the
    // incipient phase. Both come from an ideal Raoult estimate with Antoine
    // vapour pressures; FeOs then converges the PC-SAFT equilibrium.
    let (t_guess, other_guess) = raoult_initial_guess(alcohol, kind, fraction, pressure_kpa);
    let t0 = Some(t_guess * KELVIN);
    let init = nalgebra::DVector::from_vec(vec![other_guess, 1.0 - other_guess]);
    let vle = match kind {
        Saturation::Bubble => {
            PhaseEquilibrium::bubble_point(&&eos, p, fraction, t0, Some(&init), Default::default())
        }
        Saturation::Dew => {
            PhaseEquilibrium::dew_point(&&eos, p, fraction, t0, Some(&init), Default::default())
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

/// Antoine constants (log10 P/mmHg = A - B/(C + t/°C)), used ONLY to seed
/// the FeOs iteration, never as an answer. Source: Dortmund Data Bank
/// Antoine table as reprinted in common handbooks (methanol 15–84 °C,
/// ethanol 20–93 °C, water 1–100 °C); out-of-range seeding is harmless
/// because FeOs refines the result and the T window is checked after.
#[cfg_attr(not(feature = "feos"), allow(dead_code))]
fn antoine_kpa(alcohol: Option<Alcohol>, t_kelvin: f64) -> f64 {
    let (a, b, c) = match alcohol {
        Some(Alcohol::Methanol) => (8.08097, 1582.271, 239.726),
        Some(Alcohol::Ethanol) => (8.20417, 1642.89, 230.300),
        None => (8.07131, 1730.63, 233.426),
    };
    10f64.powf(a - b / (c + t_kelvin - 273.15)) * 0.133_322_368
}

/// Ideal (Raoult) bubble or dew temperature and the alcohol fraction of the
/// incipient phase, by bisection on 250–550 K.
#[cfg_attr(not(feature = "feos"), allow(dead_code))]
fn raoult_initial_guess(alcohol: Alcohol, kind: Saturation, z: f64, p_kpa: f64) -> (f64, f64) {
    let residual = |t: f64| {
        let pa = antoine_kpa(Some(alcohol), t);
        let pw = antoine_kpa(None, t);
        match kind {
            Saturation::Bubble => z * pa + (1.0 - z) * pw - p_kpa,
            Saturation::Dew => p_kpa * (z / pa + (1.0 - z) / pw) - 1.0,
        }
    };
    let rising = matches!(kind, Saturation::Bubble);
    let (mut lo, mut hi) = (250.0_f64, 550.0_f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if (residual(mid) > 0.0) == rising {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let t = 0.5 * (lo + hi);
    let pa = antoine_kpa(Some(alcohol), t);
    let other = match kind {
        Saturation::Bubble => z * pa / p_kpa,
        Saturation::Dew => z * p_kpa / pa,
    };
    (t, other.clamp(1e-6, 1.0 - 1e-6))
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
        assert!(matches!(
            q(0.5, 5000.0),
            Err(Refusal::PressureOutOfDomain(_))
        ));
        assert!(matches!(
            q(0.5, f64::NAN),
            Err(Refusal::PressureOutOfDomain(_))
        ));
        assert!(matches!(
            q(0.0, 101.325),
            Err(Refusal::CompositionOutOfDomain(_))
        ));
        assert!(matches!(
            q(1.0, 101.325),
            Err(Refusal::CompositionOutOfDomain(_))
        ));
        assert!(matches!(
            q(-0.2, 101.325),
            Err(Refusal::CompositionOutOfDomain(_))
        ));
    }

    #[test]
    fn raoult_seed_is_physical() {
        // Ideal ethanol-water at 1 atm, x = 0.5: between the pure boiling
        // points and vapour enriched in ethanol.
        let (t, y) = raoult_initial_guess(Alcohol::Ethanol, Saturation::Bubble, 0.5, 101.325);
        assert!(t > 351.0 && t < 373.2, "{t}");
        assert!(y > 0.5 && y < 1.0, "{y}");
        let (td, x) = raoult_initial_guess(Alcohol::Ethanol, Saturation::Dew, 0.5, 101.325);
        assert!(td > t && x < 0.5, "{td} {x}");
        // Water normal boiling point from its own Antoine seed.
        assert!((antoine_kpa(None, 373.15) - 101.325).abs() < 0.5);
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
