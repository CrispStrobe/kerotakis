//! Apparatus models — powered heat sources, condensers, separating funnels,
//! and recrystallization vessels (APP-001 through APP-005).
//!
//! Each apparatus is a state machine that operates on one or more vessels
//! with explicit energy and mass accounting.

use crate::species::{Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Moles, Seconds, Watts};
use crate::vessel::Vessel;
use serde::{Deserialize, Serialize};

/// A heat source with explicit power, heat loss, and a temperature
/// ceiling (APP-001).
///
/// The ceiling is the physics this bench was missing. Energy crosses into
/// a vessel only while the vessel is colder than the thing heating it, so
/// nothing on a burner ends up hotter than the burner's own flame. Without
/// that bound, `heat v1 40kJ` on ten grams of chalk was arithmetic and
/// nothing else — 40 kJ divided by 8.2 J/K put the crucible at 4913 °C,
/// three times the hottest laboratory flame and hot enough that the Gibbs
/// minimiser, correctly, then reported carbon monoxide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeatSource {
    /// What the source is called, for the log.
    pub name: String,
    /// Power input in watts (J/s).
    pub power: Watts,
    /// Heat loss coefficient to surroundings, W/K. Loss = h·(T - T_ambient).
    #[serde(default)]
    pub heat_loss_w_per_k: f64,
    /// Ambient temperature for heat loss calculation.
    #[serde(default = "ambient_default")]
    pub ambient: Kelvin,
    /// The hottest this source can drive a vessel, K. Beyond it the source
    /// is no longer the warmer body and nothing more flows.
    pub ceiling: Kelvin,
    /// Where the ceiling and the power come from.
    pub provenance: String,
}

fn ambient_default() -> Kelvin {
    Kelvin::STANDARD
}

/// Hottest point of a Bunsen flame with the air collar open, K (1500 °C).
///
/// Source: A. Agliolo Gallitto, V. Pace, R. Zingales (Dept. of Physics and
/// Chemistry, University of Palermo), "Multidisciplinary learning at the
/// University scientific museums: the Bunsen burner", arXiv:1712.08737,
/// Fig. 3 caption: "the hottest zone of the flame can reach a temperature
/// of about 1500°C". Corroborated by L. Nichols, "1.4D: Bunsen Burners",
/// Organic Chemistry Lab Techniques, Chemistry LibreTexts: "they can reach
/// temperatures of approximately 1500° C", with "the tip of the blue cone"
/// named as the hottest part.
///
/// Published figures do disagree: Arizona State University EHS's burner
/// safety sheet gives 1200 °C for the same point. 1500 °C is taken as the
/// ceiling because a ceiling should be the best the equipment can do, and
/// it stays well under the hard physical bound — the adiabatic flame
/// temperature of stoichiometric methane in air is 2224 K = 1951 °C
/// (Marzouk, Eng. Technol. Appl. Sci. Res. 13(4) 2023, 11437–11444,
/// doi:10.48084/etasr.6132) — as a real burner must, running lean and
/// losing heat to the room and the tripod.
pub const BUNSEN_CEILING_K: f64 = 1773.15;

/// Maximum temperature in a paraffin candle flame, K (1400 °C).
///
/// Source: A. Hamins, M. Bundy (NIST) and S. E. Dillon, "Characterization
/// of Candle Flames", Journal of Fire Protection Engineering 15 (2005)
/// 265–285, Table 1 "Properties of paraffin candle wax from the
/// literature", row "Maximum flame temperature 1400 °C", citing Gaydon &
/// Wolfhard, "Flames: Their Structure, Radiation and Temperature", 4th ed.
/// 1979. The same figure is used as the candle flame temperature by
/// Sunderland et al., Proc. Combust. Inst. 33 (2011) 2489–2496.
///
/// The often-quoted ~1000 °C for the luminous yellow zone — where anything
/// a learner holds in a candle actually sits — is deliberately NOT used:
/// no primary source for it could be opened, and a number without one is
/// not provenance. The maximum is therefore generous to the candle, and
/// the gap is a known overestimate rather than a hidden one.
pub const CANDLE_CEILING_K: f64 = 1673.15;

/// Maximum surface set point of a laboratory hot plate, K (550 °C).
///
/// Source: published specifications for general-purpose digital laboratory
/// hotplates and hotplate stirrers, whose top set point runs from 380 °C
/// (500 W, ceramic-coated top) to 550 °C (1000 W, glass-ceramic top), with
/// over-temperature cut-outs at 420 °C and 580 °C respectively. Quoted as a
/// class figure rather than any one product's: 550 °C is the top of the
/// range a school or teaching laboratory would own.
pub const HOT_PLATE_CEILING_K: f64 = 823.15;

impl HeatSource {
    /// A bare power source with no named flame. The ceiling is the
    /// laboratory burner's, because that is what the bench reaches for
    /// when a script says `heat` and names nothing.
    pub fn new(power: Watts) -> Self {
        Self {
            name: "unnamed heat source".to_string(),
            power,
            heat_loss_w_per_k: 0.0,
            ambient: Kelvin::STANDARD,
            ceiling: Kelvin(BUNSEN_CEILING_K),
            provenance: "ceiling: laboratory Bunsen burner, air collar open".to_string(),
        }
    }

    /// The bench default: a laboratory burner with the collar open.
    ///
    /// `heat v1 40kJ` names no apparatus, and a script that names none is
    /// standing at a school bench, where the thing under the tripod is a
    /// Bunsen burner. Making the default explicit is the point: the old
    /// implicit answer was "a source of unbounded temperature", which is
    /// not equipment anyone owns.
    pub fn bunsen_burner() -> Self {
        Self {
            name: "Bunsen burner".to_string(),
            // A standard laboratory burner on mains natural gas.
            power: Watts(1500.0),
            heat_loss_w_per_k: 0.0,
            ambient: Kelvin::STANDARD,
            ceiling: Kelvin(BUNSEN_CEILING_K),
            provenance: "Agliolo Gallitto, Pace & Zingales, arXiv:1712.08737 — the hottest zone of a Bunsen flame reaches about 1500 °C"
                .to_string(),
        }
    }

    /// A paraffin candle.
    pub fn candle() -> Self {
        Self {
            name: "candle".to_string(),
            power: Watts(80.0),
            heat_loss_w_per_k: 0.0,
            ambient: Kelvin::STANDARD,
            ceiling: Kelvin(CANDLE_CEILING_K),
            provenance: "Hamins, Bundy & Dillon, J. Fire Prot. Eng. 15 (2005) 265–285, Table 1 — maximum paraffin candle flame temperature 1400 °C"
                .to_string(),
        }
    }

    /// A laboratory stirring hot plate.
    pub fn hot_plate() -> Self {
        Self {
            name: "hot plate".to_string(),
            power: Watts(600.0),
            heat_loss_w_per_k: 0.0,
            ambient: Kelvin::STANDARD,
            ceiling: Kelvin(HOT_PLATE_CEILING_K),
            provenance: "published laboratory hotplate specifications — top set point 380–550 °C for general-purpose digital models"
                .to_string(),
        }
    }

    /// Look a source up by the word a script or a client uses for it.
    pub fn by_name(word: &str) -> Option<Self> {
        match word.trim().to_ascii_lowercase().as_str() {
            "burner" | "bunsen" | "bunsenburner" | "bunsen-burner" | "brenner"
            | "bunsenbrenner" => Some(Self::bunsen_burner()),
            "candle" | "kerze" => Some(Self::candle()),
            "hotplate" | "hot-plate" | "plate" | "heizplatte" | "kochplatte" => {
                Some(Self::hot_plate())
            }
            _ => None,
        }
    }

    pub fn with_loss(mut self, h: f64, ambient: Kelvin) -> Self {
        self.heat_loss_w_per_k = h;
        self.ambient = ambient;
        self
    }

    /// Net power into the vessel at the given temperature.
    ///
    /// Zero once the vessel has reached the source: heat does not flow
    /// from the colder body to the warmer one, so a burner stops
    /// delivering when the crucible is as hot as its flame.
    pub fn net_power(&self, vessel_temp: Kelvin) -> f64 {
        if vessel_temp.0 >= self.ceiling.0 {
            return 0.0;
        }
        self.power.0 - self.heat_loss_w_per_k * (vessel_temp.0 - self.ambient.0)
    }

    /// How much energy this source can still put into a vessel of heat
    /// capacity `cp` sitting at `now` before the vessel is as hot as the
    /// source, J. Zero when the vessel has already reached the ceiling.
    pub fn headroom_j(&self, now: Kelvin, cp: f64) -> f64 {
        if cp <= 0.0 {
            return 0.0;
        }
        ((self.ceiling.0 - now.0) * cp).max(0.0)
    }

    /// Apply the heat source to a vessel for a duration.
    /// Returns (energy_in, energy_lost, final_temperature).
    pub fn apply(&self, vessel: &mut Vessel, duration: Seconds) -> (Joules, Joules, Kelvin) {
        let cp = vessel.heat_capacity();
        if cp <= 0.0 || duration.0 <= 0.0 {
            return (Joules(0.0), Joules(0.0), vessel.temperature);
        }

        let from = vessel.temperature;
        // Simple Euler integration for T(t) = T_0 + (P_net / Cp) * dt
        // For better accuracy with heat loss, use the analytical solution:
        // T(t) = T_ambient + (T_0 - T_ambient + P/h) * exp(-h*t/Cp) - P/h
        // when h > 0.
        let h = self.heat_loss_w_per_k;
        let p = self.power.0;
        let dt = duration.0;

        let final_temp = if h > 1e-15 {
            // Analytical solution for dT/dt = (P - h*(T - T_amb)) / Cp
            let t_amb = self.ambient.0;
            // The (p / h) terms below are the approach to the equilibrium
            // temperature t_amb + p/h.
            t_amb + (from.0 - t_amb) * (-h * dt / cp).exp() + (p / h) * (1.0 - (-h * dt / cp).exp())
        } else {
            // No heat loss: linear temperature rise
            from.0 + p * dt / cp
        };

        // Absolute zero below, the source's own temperature above: a
        // vessel cannot be driven past the thing heating it however long
        // the burner is left under it.
        let final_temp = final_temp.clamp(0.0, self.ceiling.0.max(from.0));
        let energy_in = Joules(p * dt);
        let energy_lost = Joules(energy_in.0 - cp * (final_temp - from.0));

        vessel.temperature = Kelvin(final_temp);
        vessel.refresh_pressure();

        (energy_in, energy_lost, Kelvin(final_temp))
    }
}

impl Default for HeatSource {
    fn default() -> Self {
        Self::bunsen_burner()
    }
}

/// A simple condenser that collects vapour from a boiling vessel (APP-002).
///
/// Takes vapour composition from a bubble-point calculation and deposits
/// the condensed liquid into a receiver vessel.
#[derive(Debug, Clone)]
pub struct Condenser {
    /// Fraction of vapour that is successfully condensed (0..=1).
    pub efficiency: f64,
}

impl Default for Condenser {
    fn default() -> Self {
        Self { efficiency: 1.0 }
    }
}

impl Condenser {
    /// Distill: remove vapour from the source vessel according to the
    /// given vapour composition, condense it, and deposit into the receiver.
    ///
    /// `vapour_moles` is the total moles of vapour to remove.
    /// `y` is the vapour composition (mole fractions, must sum to ~1).
    /// `species` maps each component index to its SpeciesId.
    ///
    /// Returns the total moles transferred.
    pub fn distill(
        &self,
        source: &mut Vessel,
        receiver: &mut Vessel,
        vapour_moles: f64,
        y: &[f64],
        species: &[SpeciesId],
    ) -> Moles {
        let mut total = 0.0;
        for (i, sid) in species.iter().enumerate() {
            let n = vapour_moles * y[i] * self.efficiency;
            if n > 1e-15 {
                source.withdraw(sid, Moles(n));
                receiver.deposit(sid.clone(), Moles(n), Phase::Liquid);
                total += n;
            }
        }
        Moles(total)
    }
}

/// A single ideal stage for distillation (APP-003).
///
/// An ideal stage achieves thermodynamic equilibrium between liquid and vapour.
/// Multiple stages in series with reflux model a distillation column.
#[derive(Debug, Clone)]
pub struct IdealStage {
    /// Liquid holdup in moles (how much liquid sits on this tray).
    pub holdup: f64,
}

/// Result of a separatory-funnel extraction (APP-004).
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Moles of solute in the aqueous phase after extraction.
    pub aqueous_moles: f64,
    /// Moles of solute in the organic phase after extraction.
    pub organic_moles: f64,
    /// Extraction efficiency (fraction removed from aqueous).
    pub efficiency: f64,
}

/// Single-stage liquid-liquid extraction (APP-004).
///
/// Given a partition coefficient K = [solute]_organic / [solute]_aqueous,
/// compute how a solute distributes between two immiscible phases.
pub fn extract(
    solute_moles: f64,
    aqueous_volume_l: f64,
    organic_volume_l: f64,
    partition_coefficient: f64,
) -> ExtractionResult {
    if aqueous_volume_l <= 0.0 || organic_volume_l <= 0.0 || partition_coefficient <= 0.0 {
        return ExtractionResult {
            aqueous_moles: solute_moles,
            organic_moles: 0.0,
            efficiency: 0.0,
        };
    }
    // At equilibrium: K = (n_org / V_org) / (n_aq / V_aq)
    // n_org + n_aq = n_total
    // n_org = K * V_org / V_aq * n_aq
    // n_total = n_aq * (1 + K * V_org / V_aq)
    let ratio = partition_coefficient * organic_volume_l / aqueous_volume_l;
    let n_aq = solute_moles / (1.0 + ratio);
    let n_org = solute_moles - n_aq;
    ExtractionResult {
        aqueous_moles: n_aq,
        organic_moles: n_org,
        efficiency: n_org / solute_moles,
    }
}

/// Repeated extraction: n stages with equal portions of organic solvent.
/// More efficient than a single extraction with the same total solvent.
pub fn extract_repeated(
    solute_moles: f64,
    aqueous_volume_l: f64,
    organic_volume_per_stage_l: f64,
    partition_coefficient: f64,
    stages: usize,
) -> ExtractionResult {
    let mut remaining = solute_moles;
    let mut total_organic = 0.0;
    for _ in 0..stages {
        let result = extract(
            remaining,
            aqueous_volume_l,
            organic_volume_per_stage_l,
            partition_coefficient,
        );
        remaining = result.aqueous_moles;
        total_organic += result.organic_moles;
    }
    ExtractionResult {
        aqueous_moles: remaining,
        organic_moles: total_organic,
        efficiency: total_organic / solute_moles,
    }
}

/// Recrystallization result (APP-005).
#[derive(Debug, Clone)]
pub struct RecrystallizationResult {
    /// Moles of product recovered as crystals.
    pub recovered_moles: f64,
    /// Moles remaining in the mother liquor.
    pub mother_liquor_moles: f64,
    /// Recovery fraction.
    pub recovery: f64,
    /// Cooling energy removed in joules.
    pub cooling_energy_j: f64,
}

/// Model recrystallization from a saturated solution (APP-005).
///
/// A hot saturated solution is cooled; the difference in solubility between
/// hot and cold determines how much crystallizes out.
pub fn recrystallize(
    dissolved_moles: f64,
    // Unused until the model checks that the hot solution was actually
    // unsaturated; kept in the signature because that check belongs here.
    _solubility_hot_mol_per_l: f64,
    solubility_cold_mol_per_l: f64,
    volume_l: f64,
    solution_cp_j_per_k: f64,
    delta_t_k: f64,
) -> RecrystallizationResult {
    let can_hold_cold = solubility_cold_mol_per_l * volume_l;
    let precipitated = (dissolved_moles - can_hold_cold).max(0.0);
    let cooling_energy = solution_cp_j_per_k * delta_t_k.abs();

    RecrystallizationResult {
        recovered_moles: precipitated,
        mother_liquor_moles: dissolved_moles - precipitated,
        recovery: if dissolved_moles > 0.0 {
            precipitated / dissolved_moles
        } else {
            0.0
        },
        cooling_energy_j: cooling_energy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::VesselId;

    fn water_vessel(celsius: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin::from_celsius(celsius);
        v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
        v
    }

    // ── APP-001: powered heat source ──────────────────────────────

    #[test]
    fn heat_source_raises_temperature() {
        let mut v = water_vessel(25.0);
        let source = HeatSource::new(Watts(100.0));
        let (energy_in, _, final_t) = source.apply(&mut v, Seconds(10.0));
        assert!(final_t.0 > Kelvin::from_celsius(25.0).0);
        assert!((energy_in.0 - 1000.0).abs() < 1e-10); // 100W * 10s = 1000J
    }

    #[test]
    fn heat_source_with_loss_reaches_equilibrium() {
        let mut v = water_vessel(25.0);
        let source = HeatSource::new(Watts(100.0)).with_loss(10.0, Kelvin::from_celsius(25.0));
        // Equilibrium: P = h * (T_eq - T_amb) → T_eq = T_amb + P/h = 25 + 10 = 35°C
        source.apply(&mut v, Seconds(100000.0)); // very long time
        assert!(
            (v.temperature.to_celsius() - 35.0).abs() < 0.5,
            "should reach ~35°C, got {:.1}",
            v.temperature.to_celsius()
        );
    }

    #[test]
    fn heat_source_zero_duration_no_change() {
        let mut v = water_vessel(25.0);
        let source = HeatSource::new(Watts(1000.0));
        source.apply(&mut v, Seconds(0.0));
        assert!((v.temperature.to_celsius() - 25.0).abs() < 1e-10);
    }

    // ── APP-002: condenser ────────────────────────────────────────

    #[test]
    fn condenser_transfers_matter() {
        let mut source = water_vessel(100.0);
        let mut receiver = Vessel::new(VesselId(1), "receiver");
        receiver.temperature = Kelvin::from_celsius(25.0);

        let condenser = Condenser::default();
        let transferred = condenser.distill(
            &mut source,
            &mut receiver,
            0.5, // 0.5 mol of vapour
            &[1.0],
            &[SpeciesId::new("water")],
        );

        assert!((transferred.0 - 0.5).abs() < 1e-15);
        // Source lost 0.5 mol
        let source_water: f64 = source
            .contents
            .iter()
            .filter(|p| p.species.0 == "water")
            .map(|p| p.moles.0)
            .sum();
        assert!((source_water - 5.0).abs() < 1e-10);
        // Receiver gained 0.5 mol
        let receiver_water: f64 = receiver
            .contents
            .iter()
            .filter(|p| p.species.0 == "water")
            .map(|p| p.moles.0)
            .sum();
        assert!((receiver_water - 0.5).abs() < 1e-10);
    }

    // ── APP-004: extraction ───────────────────────────────────────

    #[test]
    fn single_extraction() {
        let result = extract(0.1, 0.1, 0.1, 10.0);
        // K=10, V_aq=V_org → n_org = 10/(1+10) * 0.1 ≈ 0.0909
        assert!(result.efficiency > 0.9);
        assert!((result.aqueous_moles + result.organic_moles - 0.1).abs() < 1e-12);
    }

    #[test]
    fn repeated_extraction_beats_single() {
        let total_organic = 0.3; // same total solvent
        let single = extract(0.1, 0.1, total_organic, 3.0);
        let repeated = extract_repeated(0.1, 0.1, total_organic / 3.0, 3.0, 3);
        assert!(
            repeated.efficiency > single.efficiency,
            "3 × {:.1} mL should beat 1 × {:.1} mL: {:.4} vs {:.4}",
            total_organic / 3.0 * 1000.0,
            total_organic * 1000.0,
            repeated.efficiency,
            single.efficiency
        );
    }

    #[test]
    fn extraction_conserves_mass() {
        let result = extract_repeated(0.1, 0.1, 0.05, 5.0, 4);
        assert!(
            (result.aqueous_moles + result.organic_moles - 0.1).abs() < 1e-12,
            "mass not conserved: {} + {} ≠ 0.1",
            result.aqueous_moles,
            result.organic_moles
        );
    }

    // ── APP-005: recrystallization ────────────────────────────────

    #[test]
    fn recrystallization_recovers_excess() {
        let result = recrystallize(
            0.1,   // 0.1 mol dissolved
            1.0,   // hot solubility: 1.0 mol/L
            0.2,   // cold solubility: 0.2 mol/L
            0.1,   // 100 mL of solution
            418.0, // water Cp ≈ 418 J/K for 100 mL
            50.0,  // cooled by 50 K
        );
        // Can hold 0.2 * 0.1 = 0.02 mol cold → precipitate 0.08 mol
        assert!((result.recovered_moles - 0.08).abs() < 1e-12);
        assert!((result.mother_liquor_moles - 0.02).abs() < 1e-12);
        assert!((result.recovery - 0.8).abs() < 1e-12);
        assert!(result.cooling_energy_j > 0.0);
    }

    #[test]
    fn recrystallization_nothing_if_undersaturated() {
        let result = recrystallize(
            0.01, // only 0.01 mol
            1.0,  // hot solubility
            0.5,  // cold solubility
            0.1,  // 100 mL → can hold 0.05 mol cold
            418.0, 50.0,
        );
        assert_eq!(result.recovered_moles, 0.0);
        assert_eq!(result.recovery, 0.0);
    }

    // ── KIN-012: batch and plug-flow reactor models ───────────────

    #[test]
    fn batch_reactor_conversion_increases_with_time() {
        // First-order: X = 1 - exp(-k*t)
        let x1 = batch_conversion(0.1, 1.0);
        let x2 = batch_conversion(0.1, 10.0);
        assert!(x2 > x1);
    }

    #[test]
    fn pfr_conversion_equals_batch_for_first_order() {
        // For first-order reactions, batch and PFR give the same conversion
        // at the same k*tau product.
        let k = 0.1;
        let tau = 10.0;
        let x_batch = batch_conversion(k, tau);
        let x_pfr = pfr_conversion(k, tau);
        assert!(
            (x_batch - x_pfr).abs() < 1e-10,
            "batch {:.6} vs PFR {:.6}",
            x_batch,
            x_pfr
        );
    }

    #[test]
    fn cstr_conversion_less_than_pfr() {
        // For the same residence time, CSTR gives less conversion than PFR
        let k = 0.5;
        let tau = 5.0;
        let x_pfr = pfr_conversion(k, tau);
        let x_cstr = cstr_conversion(k, tau);
        assert!(
            x_cstr < x_pfr,
            "CSTR {:.4} should be less than PFR {:.4}",
            x_cstr,
            x_pfr
        );
    }
}

// ── KIN-012: Batch and plug-flow apparatus models ─────────────────

/// First-order batch reactor conversion: X = 1 - exp(-k*t).
pub fn batch_conversion(rate_constant: f64, time_s: f64) -> f64 {
    1.0 - (-rate_constant * time_s).exp()
}

/// First-order PFR conversion: X = 1 - exp(-k*τ).
/// For first-order reactions, PFR and batch are equivalent.
pub fn pfr_conversion(rate_constant: f64, residence_time_s: f64) -> f64 {
    1.0 - (-rate_constant * residence_time_s).exp()
}

/// First-order CSTR (well-mixed) conversion: X = k*τ / (1 + k*τ).
pub fn cstr_conversion(rate_constant: f64, residence_time_s: f64) -> f64 {
    let kt = rate_constant * residence_time_s;
    kt / (1.0 + kt)
}

/// Result of a reactor design calculation.
#[derive(Debug, Clone)]
pub struct ReactorResult {
    /// Fractional conversion of the limiting reactant.
    pub conversion: f64,
    /// Exit concentration of the limiting reactant, mol/L.
    pub exit_concentration: f64,
    /// Space time (residence time), s.
    pub residence_time_s: f64,
}

/// Design a batch reactor for a first-order reaction.
pub fn batch_reactor(initial_concentration: f64, rate_constant: f64, time_s: f64) -> ReactorResult {
    let x = batch_conversion(rate_constant, time_s);
    ReactorResult {
        conversion: x,
        exit_concentration: initial_concentration * (1.0 - x),
        residence_time_s: time_s,
    }
}

/// Design a PFR for a first-order reaction.
pub fn pfr_reactor(
    initial_concentration: f64,
    rate_constant: f64,
    residence_time_s: f64,
) -> ReactorResult {
    let x = pfr_conversion(rate_constant, residence_time_s);
    ReactorResult {
        conversion: x,
        exit_concentration: initial_concentration * (1.0 - x),
        residence_time_s,
    }
}

/// Design a CSTR for a first-order reaction.
pub fn cstr_reactor(
    initial_concentration: f64,
    rate_constant: f64,
    residence_time_s: f64,
) -> ReactorResult {
    let x = cstr_conversion(rate_constant, residence_time_s);
    ReactorResult {
        conversion: x,
        exit_concentration: initial_concentration * (1.0 - x),
        residence_time_s,
    }
}
