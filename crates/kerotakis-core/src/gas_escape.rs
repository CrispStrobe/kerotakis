//! Open-vessel gas escape: one two-film transfer law for every free
//! molecular dissolved gas (lane E, 2026-11-11).
//!
//! A dissolved gas leaves an open vessel across its free surface at
//!
//! ```text
//! dn/dt = K_L * A * (c_aq - c_aq,eq)
//! 1/K_L = 1/k_L + 1/(H * k_G)                 (Whitman two-film resistances)
//! H     = c_gas / c_aq = 1 / (K_H * R * T)     (dimensionless)
//! k_L,X = k_L,ref * (Sc_X / Sc_CO2)^(-2/3)
//!       = k_L,ref * (D_X / D_CO2)^(2/3)        (same solvent, Sc = nu/D)
//! ```
//!
//! * `k_L,ref` is the still-surface CO₂ velocity the gas-exchange clock
//!   already carries ([`crate::clock::STILL_SURFACE_K_L_M_PER_S`]: Cole &
//!   Caraco 1998, Limnol. Oceanogr. 43, 647–656, doi:10.4319/lo.1998.43.4.0647,
//!   zero-wind intercept of k600 on a small lake). It is anchored on a
//!   natural-water surface at no wind, NOT a lab-beaker measurement, and is
//!   an upper bound for a still beaker (walls suppress overturn). No beaker
//!   value was found (see `docs/open-vessel-gas-escape-coverage-20261111.md`).
//! * The Schmidt exponent −2/3 is the smooth-surface value: Jähne,
//!   Münnich, Bösinger, Dutzi, Huber & Libner (1987), J. Geophys. Res.
//!   92(C2), 1937–1950, doi:10.1029/JC092iC02p01937 (abstract: k ∝ Sc^-2/3
//!   changing to Sc^-1/2 with the onset of waves); restated for a
//!   "hydrodynamically smooth water surface" by Nagel, Krall & Jähne
//!   (2019), Ocean Sci. 15, 235–247, doi:10.5194/os-15-235-2019.
//! * Diffusivities in water at 25 °C: E. L. Cussler, *Diffusion: Mass
//!   Transfer in Fluid Systems*, 2nd ed., Cambridge University Press 1997,
//!   ISBN 0-521-45078-0; values as quoted by Wikipedia "Mass diffusivity"
//!   (liquid table, units 1e-5 cm²/s), read 2026-11-11.
//! * `K_H`: the Sander (2015) table the bench already carries
//!   ([`crate::properties::HENRY_COEFFICIENTS`], Atmos. Chem. Phys. 15,
//!   4399–4981, doi:10.5194/acp-15-4399-2015), temperature-corrected.
//!
//! **The gas-side velocity `k_G` is NOT sourced.** No still-air value could
//! be read (Hicks & Liss 1976, Tellus 28, 348–354, and Liss 1973, Deep-Sea
//! Res. 20, 221–238, doi:10.1016/0011-7471(73)90013-2, were not readable in
//! full). The law is therefore applied only in its liquid-film limit
//! `K_L = k_L`, and only to gases for which that limit is the cited regime:
//! Liss (1973, abstract) states that oxygen and carbon dioxide exchange is
//! controlled by the liquid-phase resistance. A gas is admitted when its
//! dimensionless `H` is at least CO₂'s at the same temperature — no more
//! soluble than the most soluble gas the source places in the liquid-film
//! class. Ammonia and chlorine are more soluble, their gas film matters,
//! and they are refused with `not-modeled.gas-escape-no-data` rather than
//! given a rate whose controlling resistance the bench cannot price. Gases
//! with no tabulated diffusivity or Henry constant are refused the same way.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase};
use crate::vessel::{Vessel, VesselId};

/// Gas constant in L·atm/(mol·K) (CODATA R = 8.314462618 J/(mol·K)
/// divided by 101.325 J/(L·atm)).
const R_L_ATM: f64 = 0.082_057_366;

/// One row: a free molecular dissolved gas and its diffusivity in water.
pub struct EscapeRow {
    /// Bench formula key (also the Henry table key).
    pub formula: &'static str,
    /// Diffusivity in water at 25 °C, m²/s (Cussler 1997, as quoted by
    /// Wikipedia "Mass diffusivity"). `None` = no sourced value.
    pub diffusivity_m2_s: Option<f64>,
}

/// CO₂'s diffusivity in water, 1.92e-5 cm²/s (Cussler 1997, as quoted).
pub const D_CO2_M2_S: f64 = 1.92e-9;

/// The family. Every free molecular gas the lane names, plus every other
/// Henry-tabulated gas that has a Cussler diffusivity.
pub const ESCAPE_TABLE: &[EscapeRow] = &[
    EscapeRow {
        formula: "CO2",
        diffusivity_m2_s: Some(1.92e-9),
    },
    EscapeRow {
        formula: "O2",
        diffusivity_m2_s: Some(2.10e-9),
    },
    EscapeRow {
        formula: "N2",
        diffusivity_m2_s: Some(1.88e-9),
    },
    EscapeRow {
        formula: "H2",
        diffusivity_m2_s: Some(4.50e-9),
    },
    EscapeRow {
        formula: "NH3",
        diffusivity_m2_s: Some(1.64e-9),
    },
    EscapeRow {
        formula: "Cl2",
        diffusivity_m2_s: Some(1.25e-9),
    },
    EscapeRow {
        formula: "Br2",
        diffusivity_m2_s: Some(1.18e-9),
    },
    EscapeRow {
        formula: "H2S",
        diffusivity_m2_s: Some(1.41e-9),
    },
    EscapeRow {
        formula: "SO2",
        diffusivity_m2_s: None,
    },
];

/// Why a gas has no escape rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeGap {
    /// Not in [`ESCAPE_TABLE`] or no sourced diffusivity.
    NoDiffusivity,
    /// No Henry constant in the bench's Sander (2015) table.
    NoHenryConstant,
    /// More soluble than CO₂: the gas film is not negligible and `k_G` is
    /// unsourced.
    GasFilmUnsourced,
}

impl EscapeGap {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoDiffusivity => "no sourced diffusivity in water",
            Self::NoHenryConstant => "no sourced Henry constant",
            Self::GasFilmUnsourced => "gas-side transfer velocity unsourced",
        }
    }
}

/// Dimensionless gas/liquid Henry constant `c_gas/c_aq` at `t_kelvin`.
pub fn dimensionless_henry(formula: &str, t_kelvin: f64) -> Option<f64> {
    let coeff = crate::properties::henry_lookup(formula)?;
    let k_h = crate::properties::henry_at_t(coeff, t_kelvin).value;
    (k_h > 0.0 && t_kelvin > 0.0).then(|| 1.0 / (k_h * R_L_ATM * t_kelvin))
}

/// Overall liquid-side transfer velocity `K_L` (m/s) for `formula` across a
/// still open surface at `t_kelvin`, or why there is none.
pub fn transfer_velocity_m_per_s(formula: &str, t_kelvin: f64) -> Result<f64, EscapeGap> {
    let d = ESCAPE_TABLE
        .iter()
        .find(|row| row.formula == formula)
        .and_then(|row| row.diffusivity_m2_s)
        .ok_or(EscapeGap::NoDiffusivity)?;
    let h = dimensionless_henry(formula, t_kelvin).ok_or(EscapeGap::NoHenryConstant)?;
    let h_co2 = dimensionless_henry("CO2", t_kelvin).ok_or(EscapeGap::NoHenryConstant)?;
    // Same comparison at the same temperature; the tiny tolerance keeps
    // CO2 itself admitted whatever the rounding.
    if h < h_co2 * (1.0 - 1e-12) {
        return Err(EscapeGap::GasFilmUnsourced);
    }
    Ok(crate::clock::STILL_SURFACE_K_L_M_PER_S * (d / D_CO2_M2_S).powf(2.0 / 3.0))
}

/// First-order escape rate constant `K_L*A/V` (1/s) for a vessel.
pub fn escape_rate_constant_per_s(
    formula: &str,
    t_kelvin: f64,
    area_m2: f64,
    volume_l: f64,
) -> Result<f64, EscapeGap> {
    let k = transfer_velocity_m_per_s(formula, t_kelvin)?;
    if volume_l <= 0.0 || area_m2 <= 0.0 {
        return Ok(0.0);
    }
    // m/s * m² = m³/s; a cubic metre is a thousand litres.
    Ok(k * area_m2 * 1000.0 / volume_l)
}

/// Moles leaving in `seconds` from an excess `excess_mol` above the room's
/// equilibrium, with no chemical source refilling it: the exponential
/// approach `excess * (1 - exp(-k t))`. Never exceeds the excess.
pub fn escaped_moles(excess_mol: f64, rate_per_s: f64, seconds: f64) -> f64 {
    if excess_mol <= 0.0 || rate_per_s <= 0.0 || seconds <= 0.0 {
        return 0.0;
    }
    excess_mol * (1.0 - (-rate_per_s * seconds).exp())
}

/// Typed refusals for every dissolved free molecular gas in an open vessel
/// whose escape rate the bench cannot price, one per species.
pub fn refusals(vessel_id: VesselId, vessel: &Vessel) -> Vec<Event> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for portion in &vessel.contents {
        if portion.phase != Phase::Aqueous || portion.moles.0 < crate::OBSERVABLE_MOLES {
            continue;
        }
        let key = portion.species.0.as_str();
        let Some(row) = ESCAPE_TABLE.iter().find(|row| row.formula == key) else {
            continue;
        };
        if seen.contains(&row.formula) {
            continue;
        }
        let Err(gap) = transfer_velocity_m_per_s(row.formula, vessel.temperature.0) else {
            continue;
        };
        seen.push(row.formula);
        out.push(refusal(vessel_id, row.formula, gap));
    }
    out
}

/// The refusal for one gas.
pub fn refusal(vessel_id: VesselId, formula: &str, gap: EscapeGap) -> Event {
    Event::not_modeled(
        vessel_id,
        NotModelledCause::NotParameterised,
        Phrase::new(
            "not-modeled.gas-escape-no-data",
            "how fast dissolved {name} leaves the open vessel: the lab has no sourced transfer data for it ({gap}), so it does not give an escape rate",
            vec![
                (
                    "name".to_string(),
                    Slot::term("species", species::english_name(formula)),
                ),
                ("gap".to_string(), Slot::text(gap.as_str().to_string())),
            ],
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carbon_dioxide_uses_the_reference_velocity() {
        let k = transfer_velocity_m_per_s("CO2", 298.15).expect("CO2 covered");
        assert!((k - crate::clock::STILL_SURFACE_K_L_M_PER_S).abs() < 1e-15);
    }

    #[test]
    fn soluble_gases_are_refused_for_their_gas_film() {
        assert_eq!(
            transfer_velocity_m_per_s("NH3", 298.15),
            Err(EscapeGap::GasFilmUnsourced)
        );
        assert_eq!(
            transfer_velocity_m_per_s("SO2", 298.15),
            Err(EscapeGap::NoDiffusivity)
        );
    }
}
