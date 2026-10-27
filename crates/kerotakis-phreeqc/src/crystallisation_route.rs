//! Who owns a registry solid's crystallisation on cooling, per route.
//!
//! The primary solubility source is the ROUTED DATABASE'S OWN PHASE: the
//! aqueous engine already poses it in `EQUILIBRIUM_PHASES`, so its
//! saturation at the vessel's temperature comes out of the same PHREEQC
//! solve that speciates the solution — mixed electrolytes and common ions
//! included, with the database's own activity model. Nothing here
//! re-implements a solubility; this module only CLASSIFIES, for every
//! registry solid on every route, whether that path:
//!
//! * `crystallises-with-priced-heat` — the route defines a phase for the
//!   solid, the phase states a dissolution enthalpy (so its log K moves
//!   with temperature: van 't Hoff from 25 C, or the database's analytic
//!   law), and the solid is priced by [`crate::enthalpy::species_enthalpy`],
//!   so the aqueous step's enthalpy balance charges the crystallisation
//!   heat;
//! * `solubility-only` — the phase exists but either states no enthalpy
//!   (a temperature-independent log K cannot resolve crystallisation on
//!   cooling) or the solid is unpriced (crystals would come out without
//!   their heat);
//! * `refused` — the route defines no phase for the solid.
//!
//! Domain: liquid water, 1 bar, 0–100 C. Debye–Hückel routes (wateq4f,
//! minteq.v4) may claim saturation only up to
//! [`DEBYE_HUECKEL_IONIC_STRENGTH_BOUND`]; a phase whose saturation lies
//! above it is `refused` on those routes here, and at runtime the
//! router's concentrated-load fallback states that posed phases are
//! outside the route's activity domain. pitzer is admitted where it
//! defines the system. Solvent freezing is out of scope here
//! (owned by `kerotakis_core::solve::StateEquilibrator`, #852/#857).
//!
//! Measured curves (`kerotakis_core::crystallisation::CURVES`, Berkeley
//! 1904) are VALIDATION references only: `tests/crystallisation_validation.rs`
//! compares PHREEQC-derived solubilities against them and reports the
//! deviation.

use crate::{derived, enthalpy};
use kerotakis_core::{species, Phase};

/// One registry solid on one route.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RouteRow {
    pub solid: String,
    pub route: String,
    /// `crystallises-with-priced-heat`, `solubility-only` or `refused`.
    pub class: String,
    pub phase: Option<String>,
    pub reason: String,
}

/// The ionic-strength bound within which a Debye-Hueckel route (wateq4f,
/// minteq.v4) may claim saturation, mol/kgw: the router's own
/// concentrated-solution threshold. Hosted validation of b720949 showed
/// why it cannot be wider: at saturated-brine strength (~6 mol/kgw) Halite
/// on minteq.v4 missed the measured solubility by -39.98% (wateq4f +3.71%,
/// pitzer -0.49%). One route agreeing for one salt does not validate the
/// activity model, so no Debye-Hueckel route is admitted above it.
pub const DEBYE_HUECKEL_IONIC_STRENGTH_BOUND: f64 = 1.0;

fn charge_of(species: &str) -> Option<f64> {
    let tail = &species[species.rfind(['+', '-'])?..];
    let magnitude = match &tail[1..] {
        "" => 1.0,
        n => n.parse::<f64>().ok()?,
    };
    Some(if tail.starts_with('+') {
        magnitude
    } else {
        -magnitude
    })
}

/// The ionic strength of a pure-water solution saturated with this phase
/// if every activity coefficient were one. Real coefficients are below one
/// at these strengths, so the true saturating ionic strength is higher:
/// this is a lower bound. `None` when the reaction is not a plain
/// dissociation into charged products (H+/OH- terms, neutral products).
fn ideal_saturation_ionic_strength(info: &crate::dbindex::PhaseInfo) -> Option<f64> {
    let log_k = info.log_k?;
    let ions: Vec<(f64, f64)> = info
        .products
        .iter()
        .map(|(species, nu)| Some((*nu, charge_of(species)?)))
        .collect::<Option<_>>()?;
    if ions.is_empty()
        || ions.iter().any(|(nu, _)| *nu <= 0.0)
        || info.products.iter().any(|(s, _)| s == "H+" || s == "OH-")
    {
        return None;
    }
    let total: f64 = ions.iter().map(|(nu, _)| nu).sum();
    let ln_prod: f64 = ions.iter().map(|(nu, _)| nu * nu.ln()).sum();
    let molality = ((log_k * std::f64::consts::LN_10 - ln_prod) / total).exp();
    Some(0.5 * ions.iter().map(|(nu, z)| nu * z * z).sum::<f64>() * molality)
}

/// Classify one registry solid on one routed database.
pub fn route_row(solid: &str, db_tag: &str) -> RouteRow {
    let row = |class: &str, phase: Option<&str>, reason: String| RouteRow {
        solid: solid.to_string(),
        route: db_tag.to_string(),
        class: class.to_string(),
        phase: phase.map(str::to_string),
        reason,
    };
    let Some(phase) = derived::routed_phase_for_solid(solid, db_tag) else {
        return row(
            "refused",
            None,
            "the routed database defines no phase for this solid".to_string(),
        );
    };
    let info = derived::index_for(db_tag).phases.get(phase);
    if db_tag != "pitzer" {
        if let Some(ionic_strength) = info.and_then(ideal_saturation_ionic_strength) {
            if ionic_strength > DEBYE_HUECKEL_IONIC_STRENGTH_BOUND {
                return row(
                    "refused",
                    Some(phase),
                    format!(
                        "the phase saturates at ionic strength >= {ionic_strength:.2} mol/kgw \
                         (ideal lower bound), above this Debye-Hueckel route's validated \
                         {DEBYE_HUECKEL_IONIC_STRENGTH_BOUND} mol/kgw; saturated-brine \
                         crystallisation needs the pitzer route"
                    ),
                );
            }
        }
    }
    let Some(delta_h) = info.and_then(|p| p.delta_h_kj) else {
        return row(
            "solubility-only",
            Some(phase),
            "the phase states no dissolution enthalpy: its log K does not move with \
             temperature, so crystallisation on cooling is not temperature-resolved"
                .to_string(),
        );
    };
    match enthalpy::species_enthalpy(solid, Phase::Solid, db_tag) {
        Ok(_) => row(
            "crystallises-with-priced-heat",
            Some(phase),
            format!("PHREEQC phase saturation, delta_h {delta_h:.3} kJ/mol; solid priced"),
        ),
        Err(unpriced) => row(
            "solubility-only",
            Some(phase),
            format!(
                "PHREEQC phase saturation, but the solid is unpriced ({}: {})",
                unpriced.species, unpriced.why
            ),
        ),
    }
}

/// Every registry solid × every routed database, sorted.
pub fn coverage() -> Vec<RouteRow> {
    let mut solids: Vec<&str> = species::all_species()
        .into_iter()
        .filter(|d| d.standard_phase == Phase::Solid)
        .map(|d| d.key)
        .collect();
    solids.sort_unstable();
    solids
        .into_iter()
        .flat_map(|solid| {
            derived::DB_TAGS
                .iter()
                .map(move |tag| route_row(solid, tag))
        })
        .collect()
}
