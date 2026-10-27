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
//! Domain: liquid water, 1 bar, 0–100 C; Debye–Hückel routes (wateq4f,
//! minteq.v4) carry the router's existing above-1-molal caveat, pitzer is
//! the concentrated-brine route. Solvent freezing is out of scope here
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
