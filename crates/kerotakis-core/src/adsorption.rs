//! BRD-032: a dye held on activated charcoal, by a Langmuir isotherm.
//!
//! This module exists because of a trap, and the trap is worth stating
//! before the model. The corpus asks whether activated charcoal removes a
//! food dye from water. Adding the charcoal as a species and stopping
//! there would let the script PARSE and then answer WRONGLY: `filter`
//! keeps solids and pours the solution, so the carbon would come out and
//! every last molecule of dye would go through with the water. A
//! confident wrong answer about what removes a dye is worse than
//! `unknown-species`, which at least says the bench does not know. So the
//! row only moves when the adsorption actually happens.
//!
//! # What is claimed
//!
//! One equilibrium, on one pair, in the smallest shape that is honest:
//!
//! ```text
//!     q = q_max · K·C / (1 + K·C)
//! ```
//!
//! `q` is milligrams of dye held per gram of carbon, `C` the milligrams
//! per litre still dissolved, `q_max` the monolayer capacity and `K` the
//! Langmuir affinity. Total dye is conserved: what is not on the carbon
//! is in the water, and the split is the root of the mass balance, solved
//! by bisection because the function is monotone in `C`.
//!
//! # What is NOT claimed
//!
//! No rate. Langmuir is an equilibrium isotherm and this applies it as
//! one, so the bench reaches the final split in the step the carbon is
//! added and `wait` does not change it — a real column takes minutes to
//! hours, and none of that is here. No pore-size distribution, no
//! specific surface area, no competitive adsorption between two dyes, no
//! pH dependence (a real dye's uptake moves strongly with pH), no
//! temperature dependence, and no hysteresis. The isotherm is reversible:
//! dilute the beaker and the dye comes back off, which is what a Langmuir
//! equilibrium says and is more than most classroom demonstrations show.
//!
//! Deliberately NOT the PHREEQC surface machinery in `vessel::SurfaceSites`.
//! That is a hydrous-ferric-oxide model with a closed sorbate enum, whose
//! bound ions re-enter the aqueous element totals; no shipped database
//! speciates an azo dye, so routing one through it would be borrowing a
//! thermodynamic claim nobody made.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError, SolverRouteKind};
use crate::species::{self, Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::{AdsorbedAmount, Vessel};

/// One curated sorbent/sorbate pair.
pub struct Isotherm {
    /// Registry key of the solid that does the holding.
    pub sorbent: &'static str,
    /// Registry key of the species held.
    pub sorbate: &'static str,
    /// Monolayer capacity, milligrams of sorbate per gram of sorbent.
    pub capacity_mg_per_g: f64,
    /// Langmuir affinity, litres per milligram.
    pub affinity_l_per_mg: f64,
    /// What this row does not claim.
    pub boundary: &'static str,
    pub source: &'static str,
}

/// Hand-curated pairs. One, for now, and the shape says why a second is a
/// data change rather than a code change.
pub const ISOTHERMS: &[Isotherm] = &[Isotherm {
    sorbent: "activated_charcoal",
    sorbate: "methyl_orange",
    // PENDING REVIEW. Reported monolayer capacities for methyl orange on
    // powdered activated carbon are spread over most of an order of
    // magnitude — the number depends on the activation, the pore size and
    // the pH far more than on the dye — and 200 mg/g sits in the middle
    // of the range that is commonly tabulated for a laboratory-grade
    // powdered carbon. It is recorded as commonly tabulated and is
    // flagged for reviewer confirmation against a positively identified
    // source; nothing finer than "this order of magnitude" is claimed.
    capacity_mg_per_g: 200.0,
    // Also pending review, and in some ways the softer of the two: the
    // affinity decides the SHAPE of the curve at low concentration, where
    // no classroom experiment measures it. 0.05 L/mg puts the knee of the
    // isotherm well below the concentrations a demonstration uses, which
    // is the qualitative fact the number is here to carry.
    affinity_l_per_mg: 0.05,
    boundary: "an equilibrium and not a rate: the split is reached in the step the carbon meets the dye, and `wait` does not move it, while a real column takes minutes to hours. The capacity is a monolayer on a nominal carbon and does not resolve pore size, activation or specific area; there is no pH term, though a real azo dye's uptake moves strongly with pH; and there is no competition, so a second dye in the same beaker would be adsorbed as if it were alone",
    source: "Langmuir monolayer adsorption of methyl orange on powdered activated carbon. BOTH PARAMETERS ARE RECORDED AS COMMONLY TABULATED AND THEIR PROVENANCE LANE IS PENDING REVIEW: published monolayer capacities for this pair range over most of an order of magnitude with the carbon's activation and the solution pH, 200 mg/g is a mid-range laboratory-grade value, and the affinity is chosen so the knee of the isotherm sits below classroom concentrations. Neither is a transcription from a positively identified paper and no such provenance is claimed; both are flagged for reviewer confirmation",
}];

/// Below this the split is not worth an event: a tenth of a milligram is
/// under what a school balance resolves.
const OBSERVABLE_MG: f64 = 0.1;

/// Grams of a sorbent solid actually lying in the vessel.
fn sorbent_grams(vessel: &Vessel, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    let Some(data) = species::lookup(&id) else {
        return 0.0;
    };
    vessel
        .contents
        .iter()
        .filter(|p| p.species == id && p.phase == Phase::Solid)
        .map(|p| p.moles.0 * data.molar_mass)
        .sum()
}

/// Total sorbate in the vessel, dissolved plus already bound, in moles.
fn total_sorbate(vessel: &Vessel, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    let dissolved: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species == id && matches!(p.phase, Phase::Aqueous | Phase::Liquid))
        .map(|p| p.moles.0)
        .sum();
    dissolved + vessel.adsorbed_moles(&id).0
}

/// Milligrams held per gram of carbon at an equilibrium concentration of
/// `c_mg_per_l` milligrams per litre.
fn langmuir(isotherm: &Isotherm, c_mg_per_l: f64) -> f64 {
    let k_c = isotherm.affinity_l_per_mg * c_mg_per_l.max(0.0);
    isotherm.capacity_mg_per_g * k_c / (1.0 + k_c)
}

/// The equilibrium split: milligrams of sorbate left in solution, given
/// the total present and the carbon available.
///
/// `f(C) = C·V + q(C)·m − total` is strictly increasing in `C` and
/// changes sign between 0 and `total/V`, so bisection finds the root
/// without a derivative and cannot diverge. Fifty halvings take the
/// bracket below a part in 10^15 of its width, which is far finer than
/// either curated parameter deserves.
fn dissolved_at_equilibrium(isotherm: &Isotherm, total_mg: f64, litres: f64, grams: f64) -> f64 {
    if litres <= 0.0 {
        return 0.0;
    }
    let residual = |c: f64| c * litres + langmuir(isotherm, c) * grams - total_mg;
    let (mut low, mut high) = (0.0, total_mg / litres);
    if residual(high) <= 0.0 {
        return high;
    }
    for _ in 0..50 {
        let mid = 0.5 * (low + high);
        if residual(mid) > 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }
    0.5 * (low + high)
}

/// Bring every curated pair in this vessel to its isotherm, moving the
/// difference between the solution and the bound ledger.
pub fn equilibrate(vessel: &mut Vessel) -> Vec<Event> {
    let mut events = Vec::new();
    let litres = vessel.liquid_volume().0;
    for isotherm in ISOTHERMS {
        let sorbate_id = SpeciesId::new(isotherm.sorbate);
        let Some(sorbate) = species::lookup(&sorbate_id) else {
            continue;
        };
        let grams = sorbent_grams(vessel, isotherm.sorbent);
        let total_moles = total_sorbate(vessel, isotherm.sorbate);
        if total_moles <= 0.0 {
            continue;
        }
        let total_mg = total_moles * sorbate.molar_mass * 1_000.0;
        // No carbon, or no water for the dye to be in: everything the
        // ledger holds is released, everything dissolved stays dissolved.
        let bound_target_mg = if grams <= 0.0 || litres <= 0.0 {
            // With no liquid there is nothing to desorb INTO, so what is
            // already held stays held; with no carbon there is nothing to
            // hold it and it all returns.
            if grams <= 0.0 {
                0.0
            } else {
                vessel.adsorbed_moles(&sorbate_id).0 * sorbate.molar_mass * 1_000.0
            }
        } else {
            let dissolved_mg = dissolved_at_equilibrium(isotherm, total_mg, litres, grams);
            (total_mg - dissolved_mg).max(0.0)
        };
        let bound_target = bound_target_mg / (sorbate.molar_mass * 1_000.0);
        let bound_now = vessel.adsorbed_moles(&sorbate_id).0;
        let delta = bound_target - bound_now;
        if delta.abs() * sorbate.molar_mass * 1_000.0 < OBSERVABLE_MG {
            continue;
        }
        if delta > 0.0 {
            let taken = vessel.withdraw(&sorbate_id, Moles(delta));
            if taken.0 <= 0.0 {
                continue;
            }
            bind(vessel, isotherm.sorbent, &sorbate_id, taken.0);
        } else {
            let released = release(vessel, isotherm.sorbent, &sorbate_id, -delta);
            if released <= 0.0 {
                continue;
            }
            vessel.deposit(sorbate_id.clone(), Moles(released), Phase::Aqueous);
        }
        let held = vessel.adsorbed_moles(&sorbate_id).0;
        events.push(Event::Adsorbed {
            vessel: vessel.id,
            sorbate: sorbate_id.clone(),
            sorbent: SpeciesId::new(isotherm.sorbent),
            held: Moles(held),
            loading_mg_per_g: if grams > 0.0 {
                held * sorbate.molar_mass * 1_000.0 / grams
            } else {
                0.0
            },
            still_dissolved: vessel.moles_of(&sorbate_id),
            boundary: isotherm.boundary.to_string(),
        });
        vessel.resolved.invalidate();
    }
    events
}

fn bind(vessel: &mut Vessel, sorbent: &str, sorbate: &SpeciesId, moles: f64) {
    let sorbent_id = SpeciesId::new(sorbent);
    if let Some(entry) = vessel
        .adsorbed
        .iter_mut()
        .find(|entry| entry.sorbent == sorbent_id && &entry.sorbate == sorbate)
    {
        entry.moles = Moles(entry.moles.0 + moles);
    } else {
        vessel.adsorbed.push(AdsorbedAmount {
            sorbent: sorbent_id,
            sorbate: sorbate.clone(),
            moles: Moles(moles),
        });
    }
}

fn release(vessel: &mut Vessel, sorbent: &str, sorbate: &SpeciesId, moles: f64) -> f64 {
    let sorbent_id = SpeciesId::new(sorbent);
    let mut released = 0.0;
    for entry in vessel.adsorbed.iter_mut() {
        if entry.sorbent == sorbent_id && &entry.sorbate == sorbate {
            let take = entry.moles.0.min(moles - released);
            entry.moles = Moles(entry.moles.0 - take);
            released += take;
        }
    }
    vessel.adsorbed.retain(|entry| entry.moles.0 > 1e-15);
    released
}

/// Whether any curated pair in this vessel is away from its isotherm.
pub fn applies(vessel: &Vessel) -> bool {
    let litres = vessel.liquid_volume().0;
    ISOTHERMS.iter().any(|isotherm| {
        let Some(sorbate) = species::lookup(&SpeciesId::new(isotherm.sorbate)) else {
            return false;
        };
        let grams = sorbent_grams(vessel, isotherm.sorbent);
        let total = total_sorbate(vessel, isotherm.sorbate);
        if total <= 0.0 {
            return false;
        }
        let bound_now_mg = vessel.adsorbed_moles(&SpeciesId::new(isotherm.sorbate)).0
            * sorbate.molar_mass
            * 1_000.0;
        // The sorbent has gone — poured into another vessel, or consumed
        // by something else. What it was holding has nothing to hold it,
        // so this rung has work to do even though no adsorption can
        // happen: it has to give the dye back.
        if grams <= 0.0 {
            return bound_now_mg >= OBSERVABLE_MG;
        }
        if litres <= 0.0 {
            return false;
        }
        let total_mg = total * sorbate.molar_mass * 1_000.0;
        let bound_target_mg =
            total_mg - dissolved_at_equilibrium(isotherm, total_mg, litres, grams);
        (bound_target_mg - bound_now_mg).abs() >= OBSERVABLE_MG
    })
}

/// The solver rung. Registered after the aqueous tail, for a reason worth
/// stating: PHREEQC's readback rebuilds `contents` and re-dissolves any
/// `dissolves_without_speciation` solid it finds there, so a rung that
/// moved the dye before the tail ran would have its work undone every
/// step. Running after it, the bound ledger is a field the tail does not
/// touch.
#[derive(Debug, Default, Clone, Copy)]
pub struct AdsorptionEquilibrator;

impl Equilibrator for AdsorptionEquilibrator {
    fn name(&self) -> &'static str {
        "adsorption"
    }

    fn route_kind(&self) -> SolverRouteKind {
        SolverRouteKind::Computed
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        applies(vessel)
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(equilibrate(vessel))
    }
}
