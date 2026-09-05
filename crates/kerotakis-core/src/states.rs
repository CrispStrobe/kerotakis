//! When the solvent stops being a liquid.
//!
//! The bench used to report liquid water at −7.95 °C, with a pH. `Phase` was
//! assigned when matter was added and never reconsidered, so cooling a
//! beaker past its freezing point changed nothing but the number on the
//! thermometer. That is the same defect as the others found on 2026-08-19:
//! a state the engine does not model, returned as though it were the state.
//!
//! What makes this worth more than a bounds check is that the thresholds
//! *move*, and why they move is core curriculum. Dissolved particles lower
//! the freezing point and raise the boiling point, in proportion to how
//! many particles there are — which is why salt clears icy roads and why
//! seawater freezes below zero.
//!
//! **The van 't Hoff factor is not a fudge here, it is counted.** School
//! books introduce *i* as a correction you look up: 1 for sugar, 2 for
//! NaCl, 3 for CaCl₂. We never assume it. PHREEQC hands us the actual
//! species in solution — Na⁺ and Cl⁻ as separate entries, plus the neutral
//! ion pairs it finds — so summing solute molality *counts the particles*.
//! A solution where ion pairing is significant gets an effective *i* below
//! the textbook integer automatically, because the pairs are really there.
//!
//! **The constants are derived, not tabulated.** The cryoscopic and
//! ebullioscopic constants of water are not independent facts; they follow
//! from its enthalpies of fusion and vaporisation:
//!
//! ```text
//! K_f = R · T_f² · M / ΔH_fus     K_b = R · T_b² · M / ΔH_vap
//! ```
//!
//! which give 1.86 and 0.513 K·kg·mol⁻¹ against the literature's 1.86 and
//! 0.512. So the only curated inputs are two enthalpies, and the numbers a
//! learner is normally told to memorise come out of them.

use serde::{Deserialize, Serialize};

const R: f64 = crate::constants::GAS_CONSTANT;

/// Water's normal melting point at 1 atm, K.
pub const WATER_FREEZING_K: f64 = 273.15;
/// Water's normal boiling point at 1 atm, K.
pub const WATER_BOILING_K: f64 = 373.15;
/// Molar mass of water, kg/mol.
const WATER_MOLAR_MASS_KG: f64 = 0.018_015;
/// Enthalpy of fusion of water, J/mol (CRC Handbook).
pub const WATER_H_FUS: f64 = 6010.0;
/// Enthalpy of vaporisation of water at the boiling point, J/mol (CRC).
pub const WATER_H_VAP: f64 = 40650.0;
/// Lowest temperature at which the linear colligative partial-freezing model
/// is allowed to claim a liquid/ice split.
///
/// 252 K is approximately the sodium-chloride/water eutectic temperature,
/// but this is deliberately a *model boundary*, not a claim that every brine
/// shares that eutectic. Below it the identity and composition of the solid
/// salt phases matter and the linear dilute-solution relation is no longer an
/// adequate phase diagram.
pub const BRINE_MODEL_MIN_K: f64 = 252.0;

/// Cryoscopic constant of water, K·kg·mol⁻¹ — derived, not looked up.
pub fn cryoscopic_constant() -> f64 {
    R * WATER_FREEZING_K.powi(2) * WATER_MOLAR_MASS_KG / WATER_H_FUS
}

/// Particle molality at the stated low-temperature boundary.
pub fn brine_model_max_particle_molality() -> f64 {
    (WATER_FREEZING_K - BRINE_MODEL_MIN_K) / cryoscopic_constant()
}

/// Ebullioscopic constant of water, K·kg·mol⁻¹ — likewise derived.
pub fn ebullioscopic_constant() -> f64 {
    R * WATER_BOILING_K.powi(2) * WATER_MOLAR_MASS_KG / WATER_H_VAP
}

/// The temperatures at which this solution changes state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transitions {
    /// Freezing point, K, depressed by the dissolved particles.
    pub freezing_k: f64,
    /// Boiling point, K, elevated by them.
    pub boiling_k: f64,
    /// Total solute molality, mol per kg of water — the particle count that
    /// drives both shifts.
    pub solute_molality: f64,
    /// BRD-032: how much of `boiling_k`'s offset came from the vessel's
    /// pressure rather than from what is dissolved.
    ///
    /// Kept separate because the two shifts answer different questions and
    /// the prose says so: "higher than pure water because of the salt" is a
    /// different sentence from "lower because the pressure is". Defaulted
    /// on deserialization so a scene saved before this field existed still
    /// loads as the atmospheric case it was.
    #[serde(default)]
    pub pressure_shift_k: f64,
}

impl Transitions {
    /// How far the freezing point has been pushed down, K.
    pub fn freezing_depression(&self) -> f64 {
        WATER_FREEZING_K - self.freezing_k
    }
    /// How far the boiling point has been pushed up by dissolved particles,
    /// K. Deliberately **not** the total offset: a vessel under vacuum has a
    /// lower boiling point without anything being dissolved in it, and
    /// folding that into this number would make the colligative prose lie.
    pub fn boiling_elevation(&self) -> f64 {
        self.boiling_k - WATER_BOILING_K - self.pressure_shift_k
    }

    /// How far the vessel's pressure moved the boiling point, K. Negative
    /// under vacuum, positive under pressure, zero at one atmosphere.
    pub fn boiling_pressure_shift(&self) -> f64 {
        self.pressure_shift_k
    }
}

/// Where this solution freezes and boils at one atmosphere, given the total
/// molality of dissolved particles.
///
/// Colligative properties depend on how *many* particles are dissolved and
/// not at all on what they are — which is the whole point, and the reason
/// this takes a molality rather than a composition.
///
/// BRD-032 note: this is the 1 atm answer, which is what every caller that
/// only wants a liquidus should ask for. A vessel under pressure wants
/// [`transitions_at`].
pub fn transitions(solute_molality: f64) -> Transitions {
    transitions_at(solute_molality, ATMOSPHERE_KPA).0
}

/// Standard atmospheric pressure, kPa — the pressure this module assumed
/// silently until BRD-032 made it an argument.
pub const ATMOSPHERE_KPA: f64 = kerotakis_thermo::vle::ATMOSPHERE_KPA;

/// The registry key of the solvent whose phase behaviour this module owns.
/// The *identity* join to a parameter row is by InChIKey (see
/// [`solvent_row`]); this key only says which species to ask the registry
/// about.
const SOLVENT_KEY: &str = "water";

/// Which model set the boiling temperature, so `explain` can say.
///
/// BRD-032 forbids a silent fall-through: where the cleared correlation
/// cannot answer, the bench keeps the curated normal boiling point and
/// *names* the fact, rather than extrapolating a local fit into a pressure
/// it was never given data for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilingRoute {
    /// The vessel is at one atmosphere. The curated normal boiling point is
    /// the answer and no correlation was consulted.
    NormalBoilingPoint,
    /// The BRD-031 pack's cleared saturation-pressure correlation was
    /// inverted at the vessel's own pressure.
    ClearedCorrelation,
    /// The pressure is known and sits outside the window the cleared
    /// correlation spans. Water's shipped fit stops at 100 °C, so a vacuum
    /// flask routes and a pressure cooker lands here.
    PressureOutsideClearedWindow,
    /// The vessel reports no positive finite pressure to route on — a
    /// sealed vessel with nothing in its headspace, for instance.
    NoUsablePressure,
    /// No pack row carries the solvent's identity.
    SolventNotInPack,
}

impl BoilingRoute {
    /// Did a cleared parameter set actually answer?
    pub const fn routed(self) -> bool {
        matches!(self, Self::ClearedCorrelation)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NormalBoilingPoint => "normal-boiling-point",
            Self::ClearedCorrelation => "cleared-correlation",
            Self::PressureOutsideClearedWindow => "pressure-outside-cleared-window",
            Self::NoUsablePressure => "no-usable-pressure",
            Self::SolventNotInPack => "solvent-not-in-pack",
        }
    }
}

/// The solvent's row in the BRD-031 pack, reached by InChIKey.
///
/// The registry key picks the *species*; the species' InChIKey picks the
/// *parameters*. That two-step is the seam BRD-031 landed, and it is why
/// renaming a species cannot silently disconnect it from its correlation.
pub fn solvent_row() -> Option<&'static kerotakis_thermo::pack::FluidRow> {
    let data = crate::species::lookup(&crate::species::SpeciesId::new(SOLVENT_KEY))?;
    kerotakis_thermo::pack::row_by_inchikey(data.inchikey)
}

/// How far the vessel's pressure moves the solvent's boiling point, K.
///
/// The cleared correlation supplies the **shift** and the registry's
/// reviewed normal boiling point supplies the **anchor**:
///
/// ```text
/// T_b(P) = T_b(1 atm) + [ T_fit(P) − T_fit(1 atm) ]
/// ```
///
/// That composition is worth its sentence. Stull's water fit reproduces the
/// normal boiling point to 0.003 K, not to zero; taking the fit's own value
/// at one atmosphere would move every open beaker on this bench by that
/// much for no gain in truth, and would make a 1 atm result depend on which
/// correlation happened to be installed. Anchoring makes the answer at one
/// atmosphere *exactly* the curated measurement, so nothing that is not
/// actually under pressure changes at all — and leaves the correlation
/// doing the one job it is better at than a table, which is saying how far
/// the boiling point moves when the pressure does.
pub fn boiling_shift_from_pressure_k(pressure_kpa: f64) -> (f64, BoilingRoute) {
    let Some(data) = crate::species::lookup(&crate::species::SpeciesId::new(SOLVENT_KEY)) else {
        return (0.0, BoilingRoute::SolventNotInPack);
    };
    boiling_shift_for_k(data.inchikey, pressure_kpa)
}

/// [`boiling_shift_from_pressure_k`] for any fluid the pack knows by
/// InChIKey — the boiling-point apparatus asks this for whatever pure
/// liquid is in its flask, with the same anchoring and the same named
/// refusals as the solvent route.
pub fn boiling_shift_for_k(inchikey: &str, pressure_kpa: f64) -> (f64, BoilingRoute) {
    if !pressure_kpa.is_finite() || pressure_kpa <= 0.0 {
        return (0.0, BoilingRoute::NoUsablePressure);
    }
    if (pressure_kpa - ATMOSPHERE_KPA).abs() <= AMBIENT_TOLERANCE_KPA {
        return (0.0, BoilingRoute::NormalBoilingPoint);
    }
    let Some(row) = kerotakis_thermo::pack::row_by_inchikey(inchikey) else {
        return (0.0, BoilingRoute::SolventNotInPack);
    };
    let (Ok(here), Ok(reference)) = (
        row.boiling_point_c_at(pressure_kpa),
        row.boiling_point_c_at(ATMOSPHERE_KPA),
    ) else {
        return (0.0, BoilingRoute::PressureOutsideClearedWindow);
    };
    let shift = here - reference;
    if shift.is_finite() {
        (shift, BoilingRoute::ClearedCorrelation)
    } else {
        (0.0, BoilingRoute::PressureOutsideClearedWindow)
    }
}

/// Pressures this close to one atmosphere are one atmosphere.
///
/// Not a fudge factor for the physics — the anchored form above is
/// continuous through 1 atm, so widening or narrowing this changes no
/// answer by more than the amount the pressure itself changed. It exists so
/// an open vessel, whose pressure is the atmospheric constant exactly,
/// takes the `NormalBoilingPoint` label rather than reporting a routed
/// correlation that shifted it by zero.
const AMBIENT_TOLERANCE_KPA: f64 = 1e-9;

/// Where this solution freezes and boils in a vessel at `pressure_kpa`,
/// and which model said so.
///
/// Freezing is unmoved: the pressure dependence of a melting point is tiny
/// (about −0.0074 K per atmosphere for water) and this bench has no model
/// for it, so claiming one would be worse than the silence.
pub fn transitions_at(solute_molality: f64, pressure_kpa: f64) -> (Transitions, BoilingRoute) {
    let m = solute_molality.max(0.0);
    let (shift, route) = boiling_shift_from_pressure_k(pressure_kpa);
    (
        Transitions {
            freezing_k: WATER_FREEZING_K - cryoscopic_constant() * m,
            boiling_k: WATER_BOILING_K + ebullioscopic_constant() * m + shift,
            solute_molality: m,
            pressure_shift_k: shift,
        },
        route,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_constants_come_out_of_the_enthalpies() {
        // Against the literature values a student is told to memorise.
        assert!(
            (cryoscopic_constant() - 1.86).abs() < 0.01,
            "K_f = {}",
            cryoscopic_constant()
        );
        assert!(
            (ebullioscopic_constant() - 0.512).abs() < 0.005,
            "K_b = {}",
            ebullioscopic_constant()
        );
    }

    #[test]
    fn pure_water_freezes_at_zero() {
        let t = transitions(0.0);
        assert!((t.freezing_k - WATER_FREEZING_K).abs() < 1e-9);
        assert!((t.boiling_k - WATER_BOILING_K).abs() < 1e-9);
    }

    #[test]
    fn salt_water_freezes_below_zero() {
        // 1 mol/kg of NaCl dissolves into ~2 mol/kg of particles, so the
        // depression is about 2 × 1.86 K. The factor of two is not applied
        // here — it arrives as molality, because the caller counted Na+ and
        // Cl- separately.
        let t = transitions(2.0);
        assert!(
            (t.freezing_depression() - 3.72).abs() < 0.02,
            "{} K",
            t.freezing_depression()
        );
        assert!(t.freezing_k < WATER_FREEZING_K);
    }

    #[test]
    fn seawater_is_in_the_right_place() {
        // Seawater is ~1.1 mol/kg of dissolved ions and freezes near -1.9 C.
        let t = transitions(1.1);
        let celsius = t.freezing_k - 273.15;
        assert!(
            (-2.3..-1.6).contains(&celsius),
            "seawater freezes at {celsius:.2} C"
        );
    }

    #[test]
    fn boiling_rises_much_less_than_freezing_falls() {
        // K_b is about a quarter of K_f, which is why cooks salting pasta
        // water for a higher boiling point are wasting their time.
        let t = transitions(2.0);
        assert!(t.boiling_elevation() < t.freezing_depression() / 3.0);
    }

    #[test]
    fn brine_boundary_is_finite_and_matches_its_declared_temperature() {
        let maximum = brine_model_max_particle_molality();
        assert!(maximum.is_finite() && maximum > 10.0);
        assert!((transitions(maximum).freezing_k - BRINE_MODEL_MIN_K).abs() < 1e-12);
    }
}
