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
//!
//! **But ΔT = K·m is the DILUTE-solution law, and the bench no longer uses
//! it** (2026-09-11). Both constants are the limiting slope of a relation
//! whose actual variable is the *solvent's activity*, not the solute's
//! count:
//!
//! ```text
//! 1/T_f = 1/T_f° − (R/ΔH_fus)·ln a_w      1/T_b = 1/T_b° + (R/ΔH_vap)·ln a_w
//! ```
//!
//! Those are the two lines that fall out of equating the chemical potential
//! of the solvent in solution with that of ice, and with that of steam,
//! taking the enthalpy of the transition as constant across the small
//! interval it is moved through. Put `ln a_w ≈ −M·m` into either and the
//! textbook form comes back, which is exactly the approximation that made
//! one molal brine read −3.72 °C against a real −3.4: at a solute molality
//! of two, the solvent's activity is no longer its mole fraction and the
//! logarithm is no longer its argument.
//!
//! **Where a_w comes from is the whole of the honesty here**, and it
//! differs by database — see [`SolventActivity`]. One sentence of it
//! belongs at the top: only an ion-interaction (Pitzer) speciation computes
//! a solvent activity worth the name; the Debye–Hückel datasets report
//! PHREEQC's fixed `1 − 0.017·Σm` placeholder, which is the dilute law
//! wearing a different constant, and this module declines it.

use crate::heat_capacity::CpPolynomial;
use crate::species::{Phase, SpeciesData, SpeciesId};
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
/// Molar heat capacity of ice, J/(mol·K).
///
/// 2.09 J/(g·K) × 18.015 g/mol at the melting point (CRC Handbook of
/// Chemistry and Physics, specific heat of ice at 0 °C; NIST Chemistry
/// WebBook gives the same figure for H₂O(cr) at 273 K).
pub const ICE_HEAT_CAPACITY: f64 = 37.7;

/// Molar heat capacity of steam, J/(mol·K).
///
/// 33.6 J/(mol·K) — NIST Chemistry WebBook, Cp° of H₂O(g); the Shomate
/// value at 298 K and at the normal boiling point agree to within 0.2.
pub const STEAM_HEAT_CAPACITY: f64 = 33.6;

/// Molar heat capacity of liquid water, J/(mol·K), for the phases table.
///
/// The registry's own figure, restated here so the three phases read as one
/// set rather than two constants and a lookup.
pub const LIQUID_WATER_HEAT_CAPACITY: f64 = 75.3;

/// Heat capacity of a species in the phase it is actually in, J/(mol·K).
///
/// `SpeciesData::heat_capacity` is the figure for the phase a species is
/// normally added as — liquid, for water — and the bench spent it whatever
/// phase the portion had since become. Cooling 100 mL of water with 60 kJ
/// therefore chilled ICE at liquid water's 75.3 J/(mol·K) and reported
/// −39 °C; ice's own 37.7 puts the same experiment at −78 °C, which is
/// where a freezing-mixture demonstration actually lands. The plateau at
/// 0 °C was right either way, which is why the error survived: the
/// observation the curve is drawn for was never the one that was wrong.
///
/// Water only, deliberately. Every phase transition this bench models is
/// water's (`solve::SOLVENT`), and inventing per-phase figures for species
/// whose transitions are not modelled would be data with nothing to check
/// it. Anything else keeps its registry value.
pub fn heat_capacity_in(species: &SpeciesId, phase: Phase, registry: f64) -> f64 {
    constant_heat_capacity_in(&species.0, phase, registry)
}

/// The same lookup keyed by the registry key rather than an owned
/// `SpeciesId`, so a caller that already holds a `&SpeciesData` does not
/// have to allocate a `String` per portion per step to ask this question.
pub fn constant_heat_capacity_in(key: &str, phase: Phase, registry: f64) -> f64 {
    if key != "water" {
        return registry;
    }
    match phase {
        Phase::Solid => ICE_HEAT_CAPACITY,
        Phase::Gas => STEAM_HEAT_CAPACITY,
        Phase::Liquid | Phase::Aqueous => registry,
    }
}

/// The curated Cp(T) curve for this species in this phase, if there is one.
///
/// A dissolved portion is charged against the LIQUID curve, because that is
/// what the aqueous convention already does: solutes carry a heat capacity
/// of zero and the solution's heat capacity is its water's. A solute that
/// happens to own a solid curve therefore does not get charged for it while
/// it is in solution, which is the same answer the constant gave.
pub fn heat_capacity_curve(data: &SpeciesData, phase: Phase) -> Option<&'static CpPolynomial> {
    let wanted = match phase {
        Phase::Aqueous => Phase::Liquid,
        other => other,
    };
    crate::heat_capacity::polynomial_for(data.heat_capacity_polys, wanted)
}

/// Molar heat capacity of a species in a phase AT a temperature, J/(mol·K).
///
/// This is `heat_capacity_in` with the temperature it was always missing.
/// Where a curve exists it is evaluated (and held at its endpoints outside
/// the tabulated range); where none does, the room-temperature constant is
/// returned unchanged, so a species without a curve behaves exactly as it
/// did before curves existed.
pub fn heat_capacity_at(data: &SpeciesData, phase: Phase, t_k: f64) -> f64 {
    heat_capacity_curve(data, phase)
        .and_then(|curve| curve.cp(t_k))
        .unwrap_or_else(|| constant_heat_capacity_in(data.key, phase, data.heat_capacity))
}

/// The heat one mole of this species in this phase absorbs going from `t0`
/// to `t1`, J/mol — `∫Cp dT`, not `Cp·ΔT`.
///
/// Signed: cooling returns a negative number. This is the function every
/// `cp * delta_t` on the bench should be written in terms of, because with a
/// curve the two are not the same quantity: chalk heated from 25 °C to
/// 1500 °C costs a third more than its room-temperature heat capacity
/// suggests, and a burner charged the smaller number delivers energy the
/// ledger cannot account for.
pub fn enthalpy_between(data: &SpeciesData, phase: Phase, t0: f64, t1: f64) -> f64 {
    match heat_capacity_curve(data, phase).and_then(|curve| curve.enthalpy_between(t0, t1)) {
        Some(joules) => joules,
        None => constant_heat_capacity_in(data.key, phase, data.heat_capacity) * (t1 - t0),
    }
}

/// The single heat capacity that would carry the same enthalpy across
/// `[t0, t1]`, J/(mol·K).
///
/// Newton's law of cooling has a closed form only for a constant heat
/// capacity. Rather than abandon the closed form, a caller can integrate the
/// real curve across the interval it is about to traverse and use the mean
/// this returns — exact in the energy it moves, and approximate only in the
/// shape of the approach. For a degenerate interval it falls back to the
/// instantaneous value, which is the limit.
pub fn mean_heat_capacity_between(data: &SpeciesData, phase: Phase, t0: f64, t1: f64) -> f64 {
    if (t1 - t0).abs() < 1e-9 {
        return heat_capacity_at(data, phase, t0);
    }
    enthalpy_between(data, phase, t0, t1) / (t1 - t0)
}

/// Lowest temperature at which the colligative partial-freezing model is
/// allowed to claim a liquid/ice split.
///
/// 252 K is approximately the sodium-chloride/water eutectic temperature,
/// but this is deliberately a *model boundary*, not a claim that every brine
/// shares that eutectic. Below it the identity and composition of the solid
/// salt phases matter and a solvent-activity relation, however good its
/// activity, is no longer an adequate phase diagram.
pub const BRINE_MODEL_MIN_K: f64 = 252.0;

/// Cryoscopic constant of water, K·kg·mol⁻¹ — derived, not looked up.
///
/// Kept, and still exact, as what it actually is: the LIMITING slope of
/// [`freezing_point_from_activity`] as the solute molality goes to zero.
/// It is no longer the relation the bench computes with; see the module
/// documentation for why, and [`SolventActivity`] for what replaced it.
pub fn cryoscopic_constant() -> f64 {
    R * WATER_FREEZING_K.powi(2) * WATER_MOLAR_MASS_KG / WATER_H_FUS
}

/// Ebullioscopic constant of water, K·kg·mol⁻¹ — likewise derived, and
/// likewise the limiting slope rather than the working relation.
pub fn ebullioscopic_constant() -> f64 {
    R * WATER_BOILING_K.powi(2) * WATER_MOLAR_MASS_KG / WATER_H_VAP
}

/// The solvent activity at which water freezes at `t_k` — the relation
/// above, read the other way round.
pub fn activity_at_freezing_point(t_k: f64) -> f64 {
    (-(1.0 / t_k - 1.0 / WATER_FREEZING_K) * WATER_H_FUS / R).exp()
}

/// Particle molality at the stated low-temperature boundary.
///
/// Inverted through the ideal relation rather than through ΔT = K_f·m,
/// which is why it is 13.81 mol/kgw where the dilute law put it at 11.37.
/// It is a cap on the freeze-concentration BOOKKEEPING — how far the liquid
/// compartment may be concentrated before the bench stops — and not a claim
/// that a 13.8 molal brine is a solution this bench understands. The route's
/// own ceiling ([`SolventActivity::ceiling_molality`]) is the other cap, and
/// the caller takes whichever bites first.
pub fn brine_model_max_particle_molality() -> f64 {
    (1.0 / activity_at_freezing_point(BRINE_MODEL_MIN_K) - 1.0) / WATER_MOLAR_MASS_KG
}

/// Freezing point of a solution whose solvent activity is `water_activity`, K.
///
/// ```text
/// 1/T_f = 1/T_f° − (R/ΔH_fus)·ln a_w
/// ```
///
/// Exact for a solution in equilibrium with PURE ice, given a constant
/// enthalpy of fusion across the interval. Both assumptions are stated
/// rather than hidden: the ice this bench deposits is pure water by
/// construction (`solve.rs` puts only the solvent in the solid
/// compartment), and ΔH_fus varies by about 4 % over the 20 K this relation
/// is allowed to travel, which is under 0.1 K on the answer at the boundary
/// and far less anywhere a learner will be.
pub fn freezing_point_from_activity(water_activity: f64) -> f64 {
    let a = clamp_activity(water_activity);
    1.0 / (1.0 / WATER_FREEZING_K - (R / WATER_H_FUS) * a.ln())
}

/// Boiling point of a solution whose solvent activity is `water_activity`,
/// at one atmosphere, K.
///
/// ```text
/// 1/T_b = 1/T_b° + (R/ΔH_vap)·ln a_w
/// ```
///
/// The same derivation with the vapour on the other side, and it needs its
/// own sentence rather than a factor: the two constants differ because
/// ΔH_vap is nearly seven times ΔH_fus, and the two relations differ in
/// SIGN because a less active solvent is harder to boil and easier to keep
/// unfrozen. The assumption that is this side's alone is that the vapour is
/// pure solvent — true for the salts and sugars this bench dissolves, and
/// false for a volatile solute, which is distillation's question rather
/// than this one's.
pub fn boiling_point_from_activity(water_activity: f64) -> f64 {
    let a = clamp_activity(water_activity);
    1.0 / (1.0 / WATER_BOILING_K + (R / WATER_H_VAP) * a.ln())
}

/// An activity outside (0, 1] is not a solvent activity. Rather than
/// return a transition temperature computed from a logarithm of nonsense,
/// treat it as pure solvent — the answer the bench would have given with no
/// speciation at all.
fn clamp_activity(water_activity: f64) -> f64 {
    if water_activity.is_finite() && water_activity > 0.0 && water_activity <= 1.0 {
        water_activity
    } else {
        1.0
    }
}

/// Which model supplied the solvent's activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivityRoute {
    /// An ion-interaction (Pitzer) speciation computed it, from virial
    /// coefficients fitted to measured brines. `pitzer.dat` is the only
    /// dataset this bench loads that has one.
    IonInteraction,
    /// No solvent model ran, so Raoult's law was used: the activity of the
    /// solvent is its mole fraction. Exact as the molality goes to zero and
    /// an assumption everywhere else.
    IdealSolution,
}

impl ActivityRoute {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IonInteraction => "ion-interaction",
            Self::IdealSolution => "ideal-solution",
        }
    }
}

/// The prefix `Provenance::model` carries when the dataset that answered
/// applies an ion-interaction activity model.
///
/// A string test, and deliberately a narrow one. The provenance wire
/// carries the model's own prose description, this crate sits BELOW the
/// crate that knows which dataset is which, and inverting that dependency
/// to type one boolean would be the larger mistake. The seam is pinned from
/// the other side instead: `kerotakis-phreeqc`'s
/// `only_the_ion_interaction_model_carries_the_prefix_core_matches_on`
/// asserts that the Pitzer description starts with this and that neither
/// Debye–Hückel description does, so a reworded label fails a test rather
/// than silently returning every solution to the ideal route.
pub const ION_INTERACTION_MODEL_PREFIX: &str = "Pitzer";

/// Particle molality past which the ideal route stops claiming a transition.
///
/// **Where the number comes from: sucrose saturates here.** About 2.0 kg of
/// sugar dissolves in a kilogram of water at room temperature, which is 5.9
/// mol/kgw, so six is the edge of what the commonest customer of this route
/// can physically be. A boundary at the end of the solubility curve is one
/// the world draws rather than one this module picked.
///
/// **How wrong the route is on the way there, stated rather than implied.**
/// Raoult's law is exact as the molality goes to zero and says nothing about
/// how far a real solvent departs from it, so the departure has to be
/// measured from outside. Against freezing points of sucrose solutions —
/// 1 molal at about −1.86 °C and 60 % by weight (4.4 mol/kgw) at about
/// −10 °C — a sugar's osmotic coefficient AT THE FREEZING POINT runs 1.01 at
/// one molal, near 1.27 by 4.4, and about 1.4 at the ceiling. So this route
/// is 2 % short at one molal, near a tenth short at two, and about a third
/// short at six. It is not accurate to the ceiling and this comment does not
/// say it is.
///
/// What can be promised instead is the SIGN, which is worth more than a
/// tolerance because it never reverses: a non-electrolyte's osmotic
/// coefficient is above one and rises with concentration, so on this route a
/// sugar's depression and elevation are always UNDER-stated, never over. A
/// learner told "at least this much" is not misled the way one told "about
/// this much, give or take a third" would be.
///
/// An earlier draft of this comment put the ceiling's osmotic coefficient at
/// 1.2 and the error at a fifth. Those are the figures for about 3.5
/// mol/kgw, not for six, and they are left written here because a stated
/// bound that is itself wrong is worse than none.
pub const IDEAL_MAX_PARTICLE_MOLALITY: f64 = 6.0;

/// Ionic strength past which the ion-interaction route stops claiming one.
///
/// **This was 6.5 for one day and that was wrong**, so the argument for the
/// number it is now matters more than the number.
///
/// 6.5 came from halite: it saturates at I = 6.11 mol/kgw in this dataset
/// (`README.md`'s four-dataset comparison), which is the concentrated case
/// school chemistry reaches with the salt school chemistry uses. But halite
/// is one salt, ionic strength is not, and a sodium-chloride number applied
/// to every electrolyte bites first for anything that is not 1:1 — a 2:1
/// chloride reaches I = 6.5 at a third of the particle molality a 1:1 one
/// does. It refused a calcium chloride brine (`th-005`, "why does calcium
/// chloride help melt road ice?") a whole answer it can give, and did it
/// four degrees short of the eutectic boundary that should have stopped it.
///
/// The evidence for a wider range is in the shipped file rather than in a
/// citation. `vendor/iphreeqc/database/pitzer.dat` carries the evaporite
/// sequence out to its end — `MgCl2_2H2O`, `MgCl2_4H2O` and `Carnallite`
/// beside `Halite` and `Kieserite` — and bischofite saturates near 5.8
/// molal MgCl2, which is I ≈ 17 mol/kgw. A dataset is not given phases it
/// cannot reach, so its virial coefficients are fitted through there. 20
/// sits just past that.
///
/// **What the ceiling is FOR, after the correction.** On the freezing path
/// the 252 K eutectic boundary now bites first for every salt this bench
/// dissolves, so this is an entry gate against a solution that is already
/// past the model before anything is asked of it, rather than a cap on
/// freeze-concentration. That is the job it should have had: 252 K is a
/// statement about the phase diagram and this is a statement about the
/// activity model, and the first is the tighter one for a brine being
/// cooled.
pub const ION_INTERACTION_MAX_IONIC_STRENGTH: f64 = 20.0;

/// The solvent's activity, and how far it may be believed.
///
/// **The whole of the colligative honesty is in this type.** The relation
/// the bench computes with — [`freezing_point_from_activity`] — is exact
/// thermodynamics; every approximation left is in the number handed to it,
/// so that number arrives with its provenance and its range attached rather
/// than as a bare `f64`.
///
/// It is carried as the OSMOTIC COEFFICIENT φ rather than as a_w, because φ
/// is the transferable quantity: a speciation solves at one molality, and
/// the freezing pass then asks about a liquid compartment that has just
/// given up some of its water to ice and is more concentrated than the
/// solve was. φ re-applies; a_w does not.
///
/// ```text
/// ln a_w = −φ · M_w · Σm
/// ```
///
/// M_w is the registry's own molar mass of water, 0.018015 kg/mol. PHREEQC
/// writes the same relation with its reciprocal — `pitzer.cpp`: `AW =
/// exp(-OSUM * COSMOT / 55.50837)` — and 55.50837 is 1/0.0180153, so the
/// two differ by 1.7 ppm, which is 1e-4 K on a freezing point. One molar
/// mass governs every place this bench converts water's amount, so φ is
/// recovered with ours rather than with theirs.
///
/// **Which database supplies φ, and which does not.**
///
/// - `pitzer.dat` computes it. `pitzer.cpp` writes `AW = exp(-OSUM *
///   COSMOT / 55.50837)`, where `COSMOT` is the osmotic coefficient the
///   specific-ion-interaction model solved for. At one molal NaCl it
///   returns a_w = 0.9668, which is φ = 0.937 against a measured 0.936.
/// - `wateq4f.dat` and `minteq.v4.dat` do NOT, and the water activity they
///   report must not be mistaken for one. With no Pitzer or SIT block,
///   PHREEQC closes its water-activity unknown with a hard-coded linear
///   placeholder — `model.cpp`, `AH2O_FACTOR 0.017`, giving a_w = 1 −
///   0.017·Σm. That is the dilute law with a different constant. It is
///   measurably GOOD for common electrolytes, because 0.017/0.018015 =
///   0.944 sits near their osmotic coefficients — 0.1 molal NaCl comes out
///   at −0.351 °C against a measured −0.346 — and it is exactly as
///   confidently wrong for a non-electrolyte, whose φ is above one, where
///   it would under-predict a sugar's depression by six per cent. A
///   constant that carries no information about what is dissolved is not a
///   solvent model, so this bench declines it and says ideal instead.
///
/// The consequence, stated rather than hidden: a solution the router sends
/// to a Debye–Hückel dataset takes the ideal route even where an
/// ion-interaction model would have done better. Below about 0.5 molal the
/// two agree to a few hundredths of a kelvin; between there and the
/// router's 1 mol/kgw threshold the ideal route is a few per cent
/// optimistic in the same direction the dilute law was. Closing that needs
/// a solvent activity asked of `pitzer.dat` independently of which dataset
/// answered the speciation, which is a second solve per step and its own
/// piece of work.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SolventActivity {
    /// The osmotic coefficient an ion-interaction speciation computed for
    /// this solution, or `None` when none did and Raoult's law is standing
    /// in for it.
    #[serde(default)]
    pub osmotic_coefficient: Option<f64>,
    /// Particle molality beyond which this route's stated range stops and
    /// the bench declines to name a transition.
    pub ceiling_molality: f64,
}

impl Default for SolventActivity {
    fn default() -> Self {
        Self::ideal()
    }
}

impl SolventActivity {
    /// Raoult's law on whatever particles the caller counted.
    pub const fn ideal() -> Self {
        Self {
            osmotic_coefficient: None,
            ceiling_molality: IDEAL_MAX_PARTICLE_MOLALITY,
        }
    }

    /// The solvent activity an ion-interaction speciation reported, turned
    /// into the osmotic coefficient that produced it.
    ///
    /// `particle_molality` and `ionic_strength` must be the ones that
    /// speciation itself solved at, because φ is `−ln a_w / (M_w·Σm)` at
    /// THAT molality and the ceiling is the molality at which this
    /// solution's ionic strength would reach
    /// [`ION_INTERACTION_MAX_IONIC_STRENGTH`] — a ratio of the two, so a
    /// 1:1 chloride gets twice the headroom of a 2:1 one, which is right.
    ///
    /// Anything that is not a solvent activity — a non-positive or
    /// supra-unity a_w, an empty solution, an osmotic coefficient outside
    /// the range any aqueous electrolyte occupies — falls back to
    /// [`Self::ideal`] rather than propagating. A silent `NaN` in a
    /// freezing point is the failure this whole module exists to stop.
    pub fn from_speciation(
        water_activity: f64,
        particle_molality: f64,
        ionic_strength: f64,
    ) -> Self {
        if !water_activity.is_finite()
            || water_activity <= 0.0
            || water_activity > 1.0
            || !particle_molality.is_finite()
            || particle_molality <= 0.0
        {
            return Self::ideal();
        }
        let phi = -water_activity.ln() / (WATER_MOLAR_MASS_KG * particle_molality);
        // 0.6 is below the most strongly hydrating acid at any molality a
        // beaker reaches and 2.5 above the most strongly repelling salt;
        // outside that the number did not come from an osmotic model.
        //
        // It is also what makes a DILUTE solution safe on this path, and
        // that is the case worth spelling out. PHREEQC's species table
        // prints four significant figures, so a_w arrives quantised to
        // about 1e-4 — 0.01 K on a freezing point at one molal, and
        // meaningless below about 0.05 molal, where a_w rounds to 1.000 and
        // φ computes as 0. Zero is outside this band, so an activity too
        // coarse to carry information returns the ideal route instead of a
        // confident zero. The guard is a precision floor as much as a
        // plausibility check.
        if !(0.6..=2.5).contains(&phi) {
            return Self::ideal();
        }
        let ceiling = if ionic_strength.is_finite() && ionic_strength > 0.0 {
            particle_molality * ION_INTERACTION_MAX_IONIC_STRENGTH / ionic_strength
        } else {
            IDEAL_MAX_PARTICLE_MOLALITY
        };
        Self {
            osmotic_coefficient: Some(phi),
            ceiling_molality: ceiling,
        }
    }

    /// Fold in particles no speciation ever saw.
    ///
    /// Sucrose is the case: it is a non-electrolyte, no database this bench
    /// loads carries it, and the registry's `dissolves_without_speciation`
    /// is what counts it. Those particles were not in the solve that
    /// produced φ, so they are added as IDEAL ones — φ = 1 for them — and
    /// the blend is exact under the exponential form, which is also why it
    /// survives freeze-concentration: both populations concentrate by the
    /// same factor when water leaves.
    ///
    /// The ideal route already treats everything ideally and is returned
    /// unchanged.
    pub fn with_unspeciated(self, speciated_molality: f64, unspeciated_molality: f64) -> Self {
        let Some(phi) = self.osmotic_coefficient else {
            return self;
        };
        if !unspeciated_molality.is_finite() || unspeciated_molality <= 0.0 {
            return self;
        }
        let total = speciated_molality.max(0.0) + unspeciated_molality;
        if total <= 0.0 {
            return self;
        }
        Self {
            osmotic_coefficient: Some(
                (phi * speciated_molality.max(0.0) + unspeciated_molality) / total,
            ),
            // The ion-interaction range is a statement about the IONS; a
            // dissolved sugar beside them neither extends nor spends it.
            ceiling_molality: self.ceiling_molality + unspeciated_molality,
        }
    }

    /// Which model this is.
    pub const fn route(&self) -> ActivityRoute {
        match self.osmotic_coefficient {
            Some(_) => ActivityRoute::IonInteraction,
            None => ActivityRoute::IdealSolution,
        }
    }

    /// The solvent's activity at this particle molality.
    pub fn water_activity(&self, particle_molality: f64) -> f64 {
        let m = particle_molality.max(0.0);
        match self.osmotic_coefficient {
            // ln a_w = −φ·M·Σm, the ion-interaction form.
            Some(phi) => (-phi * WATER_MOLAR_MASS_KG * m).exp(),
            // a_w = x_w, Raoult. Deliberately NOT exp(−M·Σm), which is the
            // same thing only to first order and is what φ = 1 would mean:
            // the ideal statement is about the mole fraction, and at two
            // molal the two forms are 0.06 K apart.
            None => 1.0 / (1.0 + WATER_MOLAR_MASS_KG * m),
        }
    }

    /// The particle molality at which THIS solution's freezing point reaches
    /// `t_k` — how far freeze-concentration may be taken before a declared
    /// low-temperature boundary.
    ///
    /// Route-dependent, and it has to be: a brine whose osmotic coefficient
    /// is 1.27 reaches 252 K at a lower molality than an ideal solution
    /// does, and capping both at the ideal figure would let the concentrated
    /// one past the boundary it is the whole point of.
    pub fn particle_molality_freezing_at(&self, t_k: f64) -> f64 {
        let a = activity_at_freezing_point(t_k);
        match self.osmotic_coefficient {
            Some(phi) if phi > 0.0 => -a.ln() / (phi * WATER_MOLAR_MASS_KG),
            _ => (1.0 / a - 1.0) / WATER_MOLAR_MASS_KG,
        }
    }

    /// Does this route's stated range cover a solution this concentrated?
    pub fn covers(&self, particle_molality: f64) -> bool {
        particle_molality <= self.ceiling_molality
    }

    /// The sentence a refusal should carry.
    pub fn out_of_range_reason(&self, particle_molality: f64) -> String {
        match self.route() {
            ActivityRoute::IonInteraction => format!(
                "the stated range of the solvent-activity model this transition rests on ({}): the ion-interaction dataset is fitted to about I = {ION_INTERACTION_MAX_IONIC_STRENGTH} mol/kgw, which this solution reaches at {:.1} mol/kgw of dissolved particles, and it is at {particle_molality:.1}",
                ActivityRoute::IonInteraction.as_str(),
                self.ceiling_molality
            ),
            ActivityRoute::IdealSolution => format!(
                "the stated range of the solvent-activity model this transition rests on ({}): with no activity model covering this solution the bench is using Raoult's law, which it does not carry past {IDEAL_MAX_PARTICLE_MOLALITY} mol/kgw of dissolved particles, and this one is at {particle_molality:.1}",
                ActivityRoute::IdealSolution.as_str()
            ),
        }
    }
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
    /// Where the solvent's activity came from, and how far it may be
    /// believed. Defaulted on deserialization to the ideal route, which is
    /// the assumption a scene saved before this field existed was computed
    /// under.
    #[serde(default)]
    pub solvent: SolventActivity,
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

    /// The solvent activity both temperatures were computed from.
    pub fn water_activity(&self) -> f64 {
        self.solvent.water_activity(self.solute_molality)
    }

    /// The osmotic coefficient, computed or assumed: the single number that
    /// says how far this solution departs from ideality. One on the ideal
    /// route by construction, where it is an assumption rather than a
    /// result — [`Self::activity_route`] is how a reader tells which.
    pub fn osmotic_coefficient(&self) -> f64 {
        self.solvent.osmotic_coefficient.unwrap_or(1.0)
    }

    /// Which model supplied the activity.
    pub fn activity_route(&self) -> ActivityRoute {
        self.solvent.route()
    }

    /// Is this solution inside the stated range of the model that answered?
    ///
    /// False means these two temperatures are an extrapolation, and a
    /// caller must say so rather than act on them: `StateEquilibrator`
    /// declines the transition and names the boundary.
    pub fn within_model_range(&self) -> bool {
        self.solvent.covers(self.solute_molality)
    }

    /// The van 't Hoff factor a textbook would have looked up, reported
    /// back rather than assumed.
    ///
    /// Neither the counted particles per formula unit nor the osmotic
    /// coefficient is *i* on its own; it is the ratio of the depression
    /// actually computed to the one an ideal non-electrolyte of the same
    /// formula molality would give, which is what a table means when it
    /// prints 1.85 for sodium chloride. This bench can therefore print the
    /// number it was never allowed to assume.
    pub fn effective_vant_hoff_factor(&self, formula_molality: f64) -> Option<f64> {
        if formula_molality <= 0.0 {
            return None;
        }
        Some(self.freezing_depression() / (cryoscopic_constant() * formula_molality))
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
    transitions_with(SolventActivity::ideal(), solute_molality, pressure_kpa)
}

/// The same, with the solvent's activity supplied rather than assumed.
///
/// This is the entry point that carries P3s's colligative correction:
/// `activity` decides what a_w is at this molality, and the two exact
/// relations decide what that does to the temperatures. One molal brine
/// reaches it with an ion-interaction φ of 0.937 and freezes at −3.44 °C;
/// the same call with [`SolventActivity::ideal`] gives −3.61, and the
/// dilute law this replaced gave −3.72 against a measured −3.4.
///
/// The pressure shift is added to the boiling point AFTER the activity has
/// moved it, and the two are kept separable because they answer different
/// questions — see [`Transitions::pressure_shift_k`]. That composition is
/// an approximation of its own, and a small one: the exact form would
/// invert the saturation correlation against the SOLUTION's vapour
/// pressure rather than the pure solvent's, which at one molal moves a
/// pressure-cooker answer by under a hundredth of a kelvin.
pub fn transitions_with(
    activity: SolventActivity,
    solute_molality: f64,
    pressure_kpa: f64,
) -> (Transitions, BoilingRoute) {
    let m = solute_molality.max(0.0);
    let (shift, route) = boiling_shift_from_pressure_k(pressure_kpa);
    let a_w = activity.water_activity(m);
    (
        Transitions {
            freezing_k: freezing_point_from_activity(a_w),
            boiling_k: boiling_point_from_activity(a_w) + shift,
            solute_molality: m,
            pressure_shift_k: shift,
            solvent: activity,
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
        // 1 mol/kg of NaCl dissolves into ~2 mol/kg of particles. The
        // factor of two is not applied here — it arrives as molality,
        // because the caller counted Na+ and Cl- separately.
        //
        // With no activity model this is the IDEAL answer, 3.61 K, and the
        // dilute law's 3.72 is what it becomes if the logarithm is
        // linearised. The measured 3.4 needs the solvent's real activity,
        // which is the ion-interaction route below and not available here.
        let t = transitions(2.0);
        assert!(
            (t.freezing_depression() - 3.61).abs() < 0.02,
            "{} K",
            t.freezing_depression()
        );
        assert!(t.freezing_depression() < cryoscopic_constant() * 2.0);
        assert!(t.freezing_k < WATER_FREEZING_K);
        assert_eq!(t.activity_route(), ActivityRoute::IdealSolution);
    }

    #[test]
    fn the_dilute_law_is_the_limit_of_the_relation_that_replaced_it() {
        // The two agree where the textbook says they do and part company
        // where it stops saying so. At a millimolal they are indis-
        // tinguishable; at two molal the dilute law is 3 % steeper, and
        // the sign of the disagreement is always the same — a logarithm
        // linearised at zero over-states its own descent.
        for (m, tolerance) in [(1e-3, 1e-5), (0.01, 1e-4), (0.1, 2e-3)] {
            let dilute = cryoscopic_constant() * m;
            let exact = transitions(m).freezing_depression();
            assert!(
                (dilute - exact).abs() < tolerance,
                "at {m} molal: dilute {dilute} against {exact}"
            );
        }
        let far = transitions(2.0);
        assert!(far.freezing_depression() < cryoscopic_constant() * 2.0);
    }

    #[test]
    fn one_molal_brine_lands_where_a_thermometer_does() {
        // pitzer.dat reports a_w = 0.9668 for one molal NaCl, which is an
        // osmotic coefficient of 0.937 against a measured 0.936. Through
        // the exact relation that is −3.44 °C, where the dilute law said
        // −3.72 and a thermometer says about −3.4.
        let activity = SolventActivity::from_speciation(0.96680, 2.0, 1.0);
        assert_eq!(activity.route(), ActivityRoute::IonInteraction);
        let phi = activity.osmotic_coefficient.unwrap();
        assert!((phi - 0.937).abs() < 0.002, "phi = {phi}");
        let (t, _) = transitions_with(activity, 2.0, ATMOSPHERE_KPA);
        let celsius = t.freezing_k - WATER_FREEZING_K;
        assert!((celsius + 3.44).abs() < 0.02, "{celsius} C");
        // And the ceiling this solution earns. The range is stated in
        // IONIC STRENGTH, so each solution converts it at its own ratio:
        // I = 1 at Σm = 2 here, so a 1:1 chloride gets 40 mol/kgw of
        // particles where a 2:1 one would get 20. That ratio is the whole
        // reason the constant is an ionic strength and not a molality.
        assert!((activity.ceiling_molality - 40.0).abs() < 1e-9);
        assert!(t.within_model_range());
    }

    #[test]
    fn the_osmotic_coefficient_survives_freeze_concentration() {
        // The reason φ is carried rather than a_w: a speciation solves at
        // one molality and the freezing pass asks about a liquid
        // compartment that has since given up water to ice. Re-applying φ
        // at the new molality is the whole point, and it must not silently
        // return the old answer.
        let activity = SolventActivity::from_speciation(0.96680, 2.0, 1.0);
        let concentrated = activity.water_activity(4.0);
        assert!(concentrated < activity.water_activity(2.0));
        assert!(
            (concentrated - 0.93470).abs() < 1e-4,
            "a_w at 4 molal = {concentrated}"
        );
    }

    #[test]
    fn a_number_that_is_not_an_activity_falls_back_to_ideal() {
        for bad in [0.0, -1.0, 1.5, f64::NAN, f64::INFINITY] {
            assert_eq!(
                SolventActivity::from_speciation(bad, 2.0, 1.0).route(),
                ActivityRoute::IdealSolution,
                "a_w = {bad}"
            );
        }
        // An activity that IS in range but implies an impossible osmotic
        // coefficient is refused the same way: 0.5 at a millimolal would be
        // φ = 38.
        assert_eq!(
            SolventActivity::from_speciation(0.5, 1e-3, 1e-3).route(),
            ActivityRoute::IdealSolution
        );
        // And a pure-water speciation, with nothing dissolved to divide by.
        assert_eq!(
            SolventActivity::from_speciation(1.0, 0.0, 0.0).route(),
            ActivityRoute::IdealSolution
        );
    }

    #[test]
    fn each_route_states_where_it_stops() {
        let ideal = SolventActivity::ideal();
        assert!(ideal.covers(IDEAL_MAX_PARTICLE_MOLALITY));
        assert!(!ideal.covers(IDEAL_MAX_PARTICLE_MOLALITY + 1e-9));
        assert!(ideal
            .out_of_range_reason(9.0)
            .contains("Raoult"));
        // A saturated chloride brine is not near the ion-interaction
        // range's edge, and should not be: halite saturates at I = 6.11 and
        // `pitzer.dat` carries the evaporite sequence out to bischofite near
        // I = 17. At I = 6 with 12 mol/kgw of particles the ceiling is 40.
        let saturated = SolventActivity::from_speciation(0.7594, 12.0, 6.0);
        assert_eq!(saturated.route(), ActivityRoute::IonInteraction);
        assert!(saturated.covers(12.0), "{}", saturated.ceiling_molality);
        assert!(saturated.covers(39.9));
        assert!(!saturated.covers(40.1));
        // And the 2:1 salt that the old halite-shaped ceiling refused: for
        // a chloride whose ionic strength EQUALS its particle molality the
        // same range is 20, and a road-salt brine at 6 is inside it.
        let two_to_one = SolventActivity::from_speciation(0.85, 6.0, 6.0);
        assert_eq!(two_to_one.route(), ActivityRoute::IonInteraction);
        assert!(two_to_one.covers(6.0));
        assert!((two_to_one.ceiling_molality - 20.0).abs() < 1e-9);
    }

    #[test]
    fn a_saturated_brine_boils_where_a_cook_finds_it() {
        // 6 molal NaCl — near halite saturation — reports a_w = 0.7594
        // under pitzer.dat, an osmotic coefficient of 1.273 against a
        // measured 1.271. The exact relation puts its boil at 108.0 °C
        // where a measurement finds 108.7; the dilute law said 106.1, so
        // the correction is worth nearly two degrees and it is on the
        // OPPOSITE side from the freezing case, because above about three
        // molal the osmotic coefficient of a chloride passes one.
        let activity = SolventActivity::from_speciation(0.75940, 12.0, 6.0);
        let (t, _) = transitions_with(activity, 12.0, ATMOSPHERE_KPA);
        let celsius = t.boiling_k - WATER_FREEZING_K;
        assert!((celsius - 108.0).abs() < 0.1, "{celsius} C");
        assert!(t.boiling_elevation() > ebullioscopic_constant() * 12.0);
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
        assert!((transitions(maximum).freezing_k - BRINE_MODEL_MIN_K).abs() < 1e-9);
        // Inverted through the ideal relation, not through K_f·m: the
        // dilute law put this at 11.37 mol/kgw and the relation the bench
        // actually computes with puts it at 13.81.
        assert!((maximum - 13.805).abs() < 0.01, "{maximum}");
    }

    #[test]
    fn the_seam_between_freezing_and_boiling_is_one_activity() {
        // Both temperatures come from the same a_w, so nothing can move one
        // without moving the other, and the directions are opposite. This
        // is the invariant that the two relations are one derivation.
        let activity = SolventActivity::from_speciation(0.96680, 2.0, 1.0);
        let (t, _) = transitions_with(activity, 2.0, ATMOSPHERE_KPA);
        assert!((t.water_activity() - 0.96680).abs() < 1e-9);
        assert!(t.freezing_k < WATER_FREEZING_K);
        assert!(t.boiling_k > WATER_BOILING_K);
        // and the van 't Hoff factor a table would print for one molal
        // NaCl, computed rather than looked up.
        let i = t.effective_vant_hoff_factor(1.0).unwrap();
        assert!((i - 1.85).abs() < 0.02, "i = {i}");
    }
}
