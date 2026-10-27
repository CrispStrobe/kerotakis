//! The accepted domain, as pure Rust (no engine needed, wasm-safe).

use crate::{CoolPropError, Quantity, Result};

/// IAPWS-95 triple point temperature, K.
pub const TRIPLE_T_K: f64 = 273.16;
/// IAPWS-95 triple point pressure, Pa.
pub const TRIPLE_P_PA: f64 = 611.654_771;
/// IAPWS-95 critical temperature, K.
pub const TC_K: f64 = 647.096;
/// IAPWS-95 critical pressure, Pa.
pub const PC_PA: f64 = 22.064e6;
/// Upper temperature accepted for single-phase queries, K.
pub const SINGLE_PHASE_T_MAX_K: f64 = 1273.15;
/// Upper pressure accepted for single-phase queries, Pa.
pub const SINGLE_PHASE_P_MAX_PA: f64 = 100.0e6;
/// Half-widths of the near-critical exclusion box.
pub const NEAR_CRITICAL_DT_K: f64 = 1.0;
pub const NEAR_CRITICAL_DP_PA: f64 = 1.0e6;

fn range(quantity: Quantity, value: f64, min: f64, max: f64, max_inclusive: bool) -> Result<()> {
    if !value.is_finite() {
        return Err(CoolPropError::NonFinite { quantity });
    }
    let above = if max_inclusive {
        value > max
    } else {
        value >= max
    };
    if value < min || above {
        return Err(CoolPropError::OutOfDomain {
            quantity,
            value,
            min,
            max,
        });
    }
    Ok(())
}

/// Saturation by temperature: [Tt, Tc).
pub fn check_saturation_temperature(t_k: f64) -> Result<()> {
    range(Quantity::Temperature, t_k, TRIPLE_T_K, TC_K, false)
}

/// Saturation by pressure: [pt, pc).
pub fn check_saturation_pressure(p_pa: f64) -> Result<()> {
    range(Quantity::Pressure, p_pa, TRIPLE_P_PA, PC_PA, false)
}

/// Single phase: box, minus the near-critical exclusion. The saturation-line
/// exclusion needs the engine and is applied in `water_state`.
pub fn check_single_phase(t_k: f64, p_pa: f64) -> Result<()> {
    range(
        Quantity::Temperature,
        t_k,
        TRIPLE_T_K,
        SINGLE_PHASE_T_MAX_K,
        true,
    )?;
    range(
        Quantity::Pressure,
        p_pa,
        TRIPLE_P_PA,
        SINGLE_PHASE_P_MAX_PA,
        true,
    )?;
    if (t_k - TC_K).abs() < NEAR_CRITICAL_DT_K && (p_pa - PC_PA).abs() < NEAR_CRITICAL_DP_PA {
        return Err(CoolPropError::NearCritical {
            temperature_k: t_k,
            pressure_pa: p_pa,
        });
    }
    Ok(())
}
