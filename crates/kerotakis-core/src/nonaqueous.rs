//! CAP-23, rung 1: computed answers for the single-solvent organic
//! bench. Every wired solution model assumed water; a beaker of
//! ethanol full of salts drew a wall of "not yet modelled". But
//! "sodium chloride is practically insoluble in ethanol" is not a
//! model gap — it is chemistry with a handbook value, and "zinc does
//! not react with dry ethanol at bench conditions" is knowledge, not
//! an apology. This equilibrator says those things with numbers.
//!
//! Model boundary, stated once and carried in every event: dissolved
//! amounts in an organic solvent are undissociated solute held at the
//! curated solubility limit. No speciation, no activity model, no
//! electrical conductivity claim — those need an electrolyte theory
//! for the solvent, which this deliberately is not. Mixtures of two
//! organic solvents, and water present in any amount, are outside
//! this rung: the honesty pass keeps those.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::{self, Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Solvents this rung understands: one of these as the only liquid,
/// with no water anywhere, makes the vessel a single-solvent organic
/// bench.
pub const KNOWN_SOLVENTS: &[&str] = &["ethanol", "hexane", "ethyl_acetate", "propanone"];

/// Curated solubility of a solid in an organic solvent near room
/// temperature, g of solute per 100 mL of solvent. `0.0` renders as
/// "practically insoluble" — the CRC's own "i". Values are handbook
/// magnitudes, good to the first digit; the point is the verdict and
/// its order of magnitude, and the provenance says exactly that.
pub struct OrganicSolubility {
    pub solute: &'static str,
    pub solvent: &'static str,
    pub g_per_100ml: f64,
    pub source: &'static str,
}

pub const ORGANIC_SOLUBILITY: &[OrganicSolubility] = &[
    OrganicSolubility {
        solute: "NaCl",
        solvent: "ethanol",
        g_per_100ml: 0.065,
        source: "CRC Handbook, 97th ed.: NaCl in ethanol 0.065 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "ethanol",
        g_per_100ml: 0.03,
        source: "CRC Handbook, 97th ed.: KCl sparingly soluble in ethanol, ~0.03 g/100 mL",
    },
    OrganicSolubility {
        solute: "AgCl",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: AgCl insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "ethanol",
        g_per_100ml: 2.1,
        source: "CRC Handbook, 97th ed.: AgNO3 in ethanol 2.1 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "Pb(NO3)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Pb(NO3)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "ethanol",
        g_per_100ml: 13.9,
        source: "CRC Handbook, 97th ed.: NaOH in ethanol 13.9 g/100 mL",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaCO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "NaCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaCl insoluble in hydrocarbons ('i')",
    },
];

/// Metal–solvent pairs where the computed answer is "no reaction at
/// bench conditions", with the reason a learner can check. KMnO4 is
/// deliberately NOT here and NOT in the solubility table: permanganate
/// in ethanol reacts (the classic oxidation the safety screen warns
/// about) — that entry belongs to the curated-reaction rung, and
/// tabulating it as inert or merely soluble would be a lie.
pub const INERT_IN_SOLVENT: &[(&str, &str, &str)] = &[
    (
        "Zn",
        "ethanol",
        "the activity series needs a proton source or an aqueous couple; \
         dry ethanol at bench temperature offers neither at an observable rate",
    ),
    (
        "Fe",
        "ethanol",
        "no aqueous couple and no acid: iron sits in dry ethanol unchanged",
    ),
    (
        "Cu",
        "ethanol",
        "copper is below hydrogen in the series and dry ethanol offers \
         nothing to oxidise it",
    ),
    (
        "Mg",
        "ethanol",
        "magnesium ethoxide forms only at reflux with a catalyst; at bench \
         temperature the ribbon sits unchanged",
    ),
    (
        "Pb",
        "ethanol",
        "no aqueous couple and no acid: lead sits in dry ethanol unchanged",
    ),
    (
        "Ag",
        "ethanol",
        "silver is noble against everything this solvent can offer",
    ),
];

/// The single organic solvent of a water-free vessel, if there is
/// exactly one and it is on the known list.
pub fn single_organic_solvent(vessel: &Vessel) -> Option<&'static str> {
    let mut found: Option<&'static str> = None;
    for p in &vessel.contents {
        match p.phase {
            Phase::Aqueous => return None,
            Phase::Liquid => {
                if p.species.0 == "water" {
                    return None;
                }
                // A dissolved liquid solute (e.g. previously dissolved
                // NaOH) must not be mistaken for a second solvent; only
                // known solvents count.
                if let Some(s) = KNOWN_SOLVENTS.iter().find(|s| **s == p.species.0.as_str()) {
                    match found {
                        None => found = Some(s),
                        Some(prev) if prev == *s => {}
                        Some(_) => return None,
                    }
                }
            }
            _ => {}
        }
    }
    found
}

/// True when this rung has a computed verdict for the pair — the
/// honesty pass consults this so a spoken answer is not followed by an
/// apology for the same solid.
pub fn verdict_exists(species: &SpeciesId, solvent: &str) -> bool {
    ORGANIC_SOLUBILITY
        .iter()
        .any(|r| r.solute == species.0 && r.solvent == solvent)
        || INERT_IN_SOLVENT
            .iter()
            .any(|(m, s, _)| *m == species.0 && *s == solvent)
}

/// Applies curated organic-solvent verdicts: dissolution to the
/// handbook limit, and computed inertness. Sits between the curated
/// reactions and the aqueous engine in every stack.
pub struct NonAqueousEquilibrator;

impl Equilibrator for NonAqueousEquilibrator {
    fn name(&self) -> &'static str {
        "nonaqueous-curated"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        let Some(solvent) = single_organic_solvent(vessel) else {
            return false;
        };
        vessel
            .contents
            .iter()
            .any(|p| p.phase == Phase::Solid && verdict_exists(&p.species, solvent))
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        let Some(solvent) = single_organic_solvent(vessel) else {
            return Ok(events);
        };
        let solvent_id = SpeciesId::new(solvent);
        let solvent_data =
            species::lookup(&solvent_id).expect("known solvents are registry species");
        let solvent_moles: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species == solvent_id && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum();
        let solvent_ml = solvent_moles * solvent_data.molar_mass / solvent_data.density;

        // Collect verdicts first; mutate after (no aliasing games).
        let mut dissolutions: Vec<(SpeciesId, Moles, Moles)> = Vec::new();
        let mut inerts: Vec<(SpeciesId, &'static str)> = Vec::new();
        for p in &vessel.contents {
            if p.phase != Phase::Solid || p.moles.0 <= 0.0 {
                continue;
            }
            if let Some(row) = ORGANIC_SOLUBILITY
                .iter()
                .find(|r| r.solute == p.species.0 && r.solvent == solvent)
            {
                let data = species::lookup(&p.species);
                let limit_mol = match data {
                    Some(d) => row.g_per_100ml * (solvent_ml / 100.0) / d.molar_mass,
                    None => 0.0,
                };
                // Already-dissolved portion counts against the limit.
                let already: f64 = vessel
                    .contents
                    .iter()
                    .filter(|q| q.species == p.species && q.phase == Phase::Liquid)
                    .map(|q| q.moles.0)
                    .sum();
                let room = (limit_mol - already).max(0.0);
                let dissolve = p.moles.0.min(room);
                // A species already settled at its limit was verdicted on
                // a previous pass; repeating "0.0000 mol dissolved" every
                // step is noise. (A fully insoluble solid has no dissolved
                // marker to remember, so its verdict does repeat — the
                // price of statelessness, and the honest default.)
                if dissolve <= 0.0 && already > 0.0 {
                    continue;
                }
                dissolutions.push((
                    p.species.clone(),
                    Moles(dissolve),
                    Moles(p.moles.0 - dissolve),
                ));
            } else if let Some((_, _, why)) = INERT_IN_SOLVENT
                .iter()
                .find(|(m, s, _)| *m == p.species.0 && *s == solvent)
            {
                inerts.push((p.species.clone(), *why));
            }
        }

        for (species_id, dissolve, remaining) in dissolutions {
            if dissolve.0 > 0.0 {
                vessel.withdraw(&species_id, dissolve);
                vessel.deposit(species_id.clone(), dissolve, Phase::Liquid);
            }
            events.push(Event::DissolvedInSolvent {
                vessel: vessel.id,
                species: species_id,
                solvent: solvent_id.clone(),
                dissolved: dissolve,
                undissolved: remaining,
            });
        }
        for (species_id, why) in inerts {
            events.push(Event::InertInSolvent {
                vessel: vessel.id,
                species: species_id,
                solvent: solvent_id.clone(),
                why: why.to_string(),
            });
        }
        Ok(events)
    }
}
