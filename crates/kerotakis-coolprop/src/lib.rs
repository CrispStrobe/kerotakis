//! Optional CoolProp adapter: water/steam properties from IAPWS-95.
//!
//! This crate is an *opt-in* property-query surface, not part of the default
//! physics path. Nothing in Kerotakis calls it unless a consumer depends on it
//! and enables the `engine` feature. Without that feature every query still
//! runs the domain gate (so refusals are testable everywhere, wasm included)
//! and then returns [`CoolPropError::EngineNotLinked`].
//!
//! # Domain
//!
//! One fluid, water, through CoolProp's `HEOS::Water` backend, which is the
//! IAPWS-95 formulation (Wagner & Pruß, J. Phys. Chem. Ref. Data 31 (2002)
//! 387, doi:10.1063/1.1461829). The accepted region is deliberately narrower
//! than the formulation's own validity (273.16–1273 K, up to 1000 MPa):
//!
//! * saturation: triple point 273.16 K / 611.655 Pa up to, but excluding,
//!   the critical point 647.096 K / 22.064 MPa;
//! * single phase: 273.16 K ≤ T ≤ 1273.15 K and 611.655 Pa ≤ p ≤ 100 MPa,
//!   excluding a near-critical box (|T − Tc| < 1 K and |p − pc| < 1 MPa) where
//!   cp diverges, and excluding points on the saturation line itself, where
//!   "the" density is not one number.
//!
//! Anything outside is a typed refusal. Nothing extrapolates.

use std::fmt;

mod domain;
#[cfg(feature = "engine")]
mod ffi;

pub use domain::*;

/// Provenance phrase carried by every value this crate returns.
pub const PROVENANCE: &str = "CoolProp v6.8.0 (MIT, github.com/CoolProp/CoolProp, \
     commit 442e8bd29867b11b9a5d8b6bbba7b7dcf35d2a03), HEOS::Water backend = IAPWS-95: \
     W. Wagner and A. Pruss, J. Phys. Chem. Ref. Data 31 (2002) 387-535, \
     doi:10.1063/1.1461829; IAPWS R6-95(2018)";

/// The only fluid in the domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fluid {
    Water,
}

impl Fluid {
    /// CoolProp backend::fluid string. `HEOS::` pins the Helmholtz EOS
    /// (IAPWS-95) instead of letting CoolProp pick a backend.
    pub const fn coolprop_name(self) -> &'static str {
        match self {
            Self::Water => "HEOS::Water",
        }
    }
}

/// Which single phase a (T, p) state lies in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Liquid,
    Vapour,
    /// T ≥ Tc or p ≥ pc outside the near-critical exclusion box.
    Supercritical,
}

/// Both coexisting phases at one saturation state. SI units throughout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SaturationState {
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub density_liquid_kg_m3: f64,
    pub density_vapour_kg_m3: f64,
    pub cp_liquid_j_kg_k: f64,
    pub cp_vapour_j_kg_k: f64,
}

/// A single-phase state point. SI units throughout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatePoint {
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub phase: Phase,
    pub density_kg_m3: f64,
    pub cp_j_kg_k: f64,
}

/// Every way a query can fail. All are refusals; none carries a fallback value.
#[derive(Clone, Debug, PartialEq)]
pub enum CoolPropError {
    /// Input not a finite number.
    NonFinite { quantity: Quantity },
    /// Input outside the accepted domain.
    OutOfDomain {
        quantity: Quantity,
        value: f64,
        min: f64,
        max: f64,
    },
    /// Single-phase request inside the near-critical exclusion box.
    NearCritical {
        temperature_k: f64,
        pressure_pa: f64,
    },
    /// Single-phase request on (within 1e-6 relative of) the saturation line.
    OnSaturationLine {
        temperature_k: f64,
        pressure_pa: f64,
    },
    /// Crate built without the `engine` feature.
    EngineNotLinked,
    /// CoolProp itself reported an error (its `errstring`).
    Engine(String),
}

/// The physical quantity a refusal is about.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Quantity {
    Temperature,
    Pressure,
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Temperature => "temperature (K)",
            Self::Pressure => "pressure (Pa)",
        })
    }
}

impl fmt::Display for CoolPropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { quantity } => write!(f, "{quantity} is not a finite number"),
            Self::OutOfDomain {
                quantity,
                value,
                min,
                max,
            } => write!(
                f,
                "{quantity} {value} outside the CoolProp/IAPWS-95 domain [{min}, {max}]"
            ),
            Self::NearCritical {
                temperature_k,
                pressure_pa,
            } => write!(
                f,
                "({temperature_k} K, {pressure_pa} Pa) is inside the near-critical exclusion"
            ),
            Self::OnSaturationLine {
                temperature_k,
                pressure_pa,
            } => write!(
                f,
                "({temperature_k} K, {pressure_pa} Pa) lies on the saturation line; \
                 ask for a saturation state instead"
            ),
            Self::EngineNotLinked => {
                f.write_str("kerotakis-coolprop was built without the `engine` feature")
            }
            Self::Engine(msg) => write!(f, "CoolProp error: {msg}"),
        }
    }
}

impl std::error::Error for CoolPropError {}

pub type Result<T> = std::result::Result<T, CoolPropError>;

#[cfg(feature = "engine")]
fn prop(output: &str, n1: &str, v1: f64, n2: &str, v2: f64) -> Result<f64> {
    ffi::props_si(output, n1, v1, n2, v2, Fluid::Water.coolprop_name())
        .map_err(CoolPropError::Engine)
}

#[cfg(feature = "engine")]
fn saturation_from(name: &str, value: f64) -> Result<SaturationState> {
    let (t, p) = match name {
        "T" => (value, prop("P", "T", value, "Q", 0.0)?),
        _ => (prop("T", "P", value, "Q", 0.0)?, value),
    };
    Ok(SaturationState {
        temperature_k: t,
        pressure_pa: p,
        density_liquid_kg_m3: prop("D", name, value, "Q", 0.0)?,
        density_vapour_kg_m3: prop("D", name, value, "Q", 1.0)?,
        cp_liquid_j_kg_k: prop("C", name, value, "Q", 0.0)?,
        cp_vapour_j_kg_k: prop("C", name, value, "Q", 1.0)?,
    })
}

/// Saturation state of water at temperature `t_k` (K).
pub fn saturation_at_temperature(t_k: f64) -> Result<SaturationState> {
    domain::check_saturation_temperature(t_k)?;
    #[cfg(feature = "engine")]
    {
        saturation_from("T", t_k)
    }
    #[cfg(not(feature = "engine"))]
    {
        Err(CoolPropError::EngineNotLinked)
    }
}

/// Saturation state of water at pressure `p_pa` (Pa).
pub fn saturation_at_pressure(p_pa: f64) -> Result<SaturationState> {
    domain::check_saturation_pressure(p_pa)?;
    #[cfg(feature = "engine")]
    {
        saturation_from("P", p_pa)
    }
    #[cfg(not(feature = "engine"))]
    {
        Err(CoolPropError::EngineNotLinked)
    }
}

/// Single-phase water state at (T, p). Refuses saturation-line points.
pub fn water_state(t_k: f64, p_pa: f64) -> Result<StatePoint> {
    domain::check_single_phase(t_k, p_pa)?;
    #[cfg(feature = "engine")]
    {
        let phase = if t_k >= TC_K || p_pa >= PC_PA {
            Phase::Supercritical
        } else {
            let psat = prop("P", "T", t_k, "Q", 0.0)?;
            if ((p_pa - psat) / psat).abs() < 1e-6 {
                return Err(CoolPropError::OnSaturationLine {
                    temperature_k: t_k,
                    pressure_pa: p_pa,
                });
            }
            if p_pa > psat {
                Phase::Liquid
            } else {
                Phase::Vapour
            }
        };
        Ok(StatePoint {
            temperature_k: t_k,
            pressure_pa: p_pa,
            phase,
            density_kg_m3: prop("D", "T", t_k, "P", p_pa)?,
            cp_j_kg_k: prop("C", "T", t_k, "P", p_pa)?,
        })
    }
    #[cfg(not(feature = "engine"))]
    {
        Err(CoolPropError::EngineNotLinked)
    }
}

/// Opt-in hook in Kerotakis' bench units: saturation vapour pressure of water,
/// kPa, at `t_celsius`. Same signature shape as
/// `kerotakis_thermo::vle::VapourPressure::pressure_kpa`, but a typed refusal
/// instead of `None`.
pub fn water_saturation_pressure_kpa(t_celsius: f64) -> Result<f64> {
    saturation_at_temperature(t_celsius + 273.15).map(|s| s.pressure_pa / 1000.0)
}

/// Opt-in hook: boiling (saturation) temperature of water, °C, at `p_kpa`.
pub fn water_boiling_point_celsius(p_kpa: f64) -> Result<f64> {
    saturation_at_pressure(p_kpa * 1000.0).map(|s| s.temperature_k - 273.15)
}

/// Version and git revision reported by the linked CoolProp, for receipts.
#[cfg(feature = "engine")]
pub fn linked_version() -> (String, String) {
    ffi::version()
}
