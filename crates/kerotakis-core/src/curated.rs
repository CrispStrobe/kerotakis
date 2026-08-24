//! The seed of L4: curated reactions, hand-verified, applied at the
//! compound level. This is deliberately the *first* chemistry stage in the
//! stack — curated knowledge outranks generic solvers where it exists
//! (PLAN.md, "L4 is a cascade").
//!
//! Gas products leave a reservoir/swept boundary but remain in a materially
//! closed headspace. The balance notices either way, which makes conservation
//! of mass across the boundary explicit. Reaction enthalpies for these entries
//! are not yet curated; no heat effect is applied (honestly).

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

pub struct CuratedReaction {
    /// Shown at student register and above.
    pub equation: &'static str,
    pub reactants: &'static [(&'static str, f64)],
    /// Products with the phase they appear in. `Phase::Gas` products escape
    /// an external boundary or remain in a material-closed headspace.
    pub products: &'static [(&'static str, f64, Phase)],
    /// When set, this reaction fires only in a single-organic-solvent
    /// bench of the named solvent (no water). Extent is computed from
    /// the dissolved fraction only — undissolved solid on the bottom
    /// does not participate.
    pub solvent: Option<&'static str>,
    /// When set, the reaction fires only when the vessel temperature
    /// is at or above this threshold (in kelvin).
    pub min_temp_k: Option<f64>,
    /// When set, this species must be present for the reaction to fire,
    /// but is NOT consumed (enzyme/catalyst).
    pub catalyst: Option<&'static str>,
}

/// Hand-verified seed set. Grows into the codex (P4).
pub const REACTIONS: &[CuratedReaction] = &[
    CuratedReaction {
        equation: "NH3 + NaOCl → NH2Cl↑ + NaOH",
        reactants: &[("NH3", 1.0), ("NaOCl", 1.0)],
        products: &[("NH2Cl", 1.0, Phase::Gas), ("NaOH", 1.0, Phase::Aqueous)],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "NaOCl + 2 HCl → Cl2↑ + NaCl + H2O",
        reactants: &[("NaOCl", 1.0), ("HCl", 2.0)],
        products: &[
            ("Cl2", 1.0, Phase::Gas),
            ("NaCl", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "4 KMnO4 + 3 C₂H₅OH → 4 MnO₂↓ + 3 CH₃COOH + 4 KOH + H₂O",
        reactants: &[("KMnO4", 4.0), ("ethanol", 3.0)],
        products: &[
            ("MnO2", 4.0, Phase::Solid),
            ("CH3COOH", 3.0, Phase::Liquid),
            ("KOH", 4.0, Phase::Liquid),
            ("water", 1.0, Phase::Liquid),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "4 MnO₄⁻ + 3 C₂H₅OH → 4 MnO₂↓ + 3 CH₃COOH + 4 OH⁻ + H₂O",
        reactants: &[("MnO4-", 4.0), ("ethanol", 3.0)],
        products: &[
            ("MnO2", 4.0, Phase::Solid),
            ("CH3COOH", 3.0, Phase::Aqueous),
            ("OH-", 4.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // ── silver metathesis in ethanol (CAP-23 rung 2b) ────────────
    // PHREEQC handles these in water; the curated entries fire only
    // in the organic solvent, drawing from the dissolved fraction.
    CuratedReaction {
        equation: "AgNO₃ + NaCl → AgCl↓ + NaNO₃",
        reactants: &[("AgNO3", 1.0), ("NaCl", 1.0)],
        products: &[("AgCl", 1.0, Phase::Solid), ("NaNO3", 1.0, Phase::Solid)],
        solvent: Some("ethanol"),
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "AgNO₃ + KCl → AgCl↓ + KNO₃",
        reactants: &[("AgNO3", 1.0), ("KCl", 1.0)],
        products: &[("AgCl", 1.0, Phase::Solid), ("KNO3", 1.0, Phase::Solid)],
        solvent: Some("ethanol"),
        min_temp_k: None,
        catalyst: None,
    },
    // ── iodine decolorisation (EXP-13: Vitamin C) ─────────────
    // Ascorbic acid reduces molecular iodine to iodide; this is
    // the basis of the iodometric vitamin C assay.
    CuratedReaction {
        equation: "C₆H₈O₆ + I₂ → C₆H₆O₆ + 2 HI",
        reactants: &[("ascorbic_acid", 1.0), ("I2", 1.0)],
        products: &[
            ("dehydroascorbic_acid", 1.0, Phase::Aqueous),
            ("HI", 2.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // ── thermal decomposition (EXP-2: Backpulver) ───────────────
    // Onset ~50 °C, classroom-observable above ~80 °C (CRC Handbook
    // 97th ed.; Merck Index 15th ed.). Threshold set at 353 K.
    CuratedReaction {
        equation: "2 NaHCO₃ →Δ Na₂CO₃ + H₂O + CO₂↑",
        reactants: &[("NaHCO3", 2.0)],
        products: &[
            ("Na2CO3", 1.0, Phase::Solid),
            ("water", 1.0, Phase::Liquid),
            ("CO2", 1.0, Phase::Gas),
        ],
        solvent: None,
        min_temp_k: Some(353.0),
        catalyst: None,
    },
    // ── enzymatic hydrolysis (EXP-14: Das süße Brot) ────────────
    // Amylase catalyses starch hydrolysis to maltose. The enzyme
    // is not consumed. Simplified: 2 monomer units + H₂O → maltose.
    CuratedReaction {
        equation: "2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁",
        reactants: &[("starch", 2.0), ("water", 1.0)],
        products: &[("maltose", 1.0, Phase::Aqueous)],
        solvent: None,
        min_temp_k: None,
        catalyst: Some("amylase"),
    },
];

/// A named organic transformation the `react` verb applies on command.
///
/// Deliberately NOT registered with `CuratedEquilibrator`: acid and
/// alcohol standing in one beaker at room temperature do not visibly
/// esterify — the reaction wants its conditions and its push, which is
/// what makes it a *verb*. The stoichiometry here is proven at the
/// identity level against the atom-mapped SMIRKS templates in
/// `kerotakis-org` (its differential test maps each product SMILES to
/// an InChIKey and requires the registry species named here to carry
/// that same key), so this table and the templates cannot drift apart.
pub struct OrgReaction {
    /// The name the verb takes: `react v1 esterification`.
    pub name: &'static str,
    pub equation: &'static str,
    pub reactants: &'static [(&'static str, f64)],
    pub products: &'static [(&'static str, f64, Phase)],
    /// What this entry does NOT claim — stated at lv3, because an
    /// equilibrium driven to completion on command is a modelling
    /// choice, not a measurement.
    pub boundary: &'static str,
    pub source: &'static str,
}

pub const ORG_REACTIONS: &[OrgReaction] = &[
    OrgReaction {
        name: "esterification",
        equation: "CH3COOH + C2H5OH ⇌ CH3COOC2H5 + H2O",
        reactants: &[("CH3COOH", 1.0), ("ethanol", 1.0)],
        products: &[
            ("ethyl_acetate", 1.0, Phase::Liquid),
            ("water", 1.0, Phase::Liquid),
        ],
        boundary: "Fischer esterification is an equilibrium (K ≈ 4 for this                    pair); the verb drives the requested extent to completion                    and says so — no yield claim is made, and the acid                    catalyst and heat the real reaction wants are assumed,                    not modelled",
        source: "Fischer esterification, March's Advanced Organic Chemistry;                  stoichiometry proven against the kerotakis-org SMIRKS                  template at the InChIKey level",
    },
    OrgReaction {
        name: "saponification",
        equation: "CH3COOC2H5 + NaOH → NaOAc + C2H5OH",
        reactants: &[("ethyl_acetate", 1.0), ("NaOH", 1.0)],
        products: &[
            ("NaOAc", 1.0, Phase::Aqueous),
            ("ethanol", 1.0, Phase::Liquid),
        ],
        boundary: "alkaline hydrolysis is driven by the carboxylate sink and                    goes to completion honestly; the heat of reaction is not                    yet curated and no thermal effect is applied",
        source: "Alkaline ester hydrolysis, March's Advanced Organic                  Chemistry; stoichiometry proven against the kerotakis-org                  SMIRKS template at the InChIKey level",
    },
];

const TRACE: f64 = 1e-12;

fn extent(vessel: &Vessel, reaction: &CuratedReaction) -> f64 {
    reaction
        .reactants
        .iter()
        .map(|(key, coeff)| vessel.moles_of(&SpeciesId::new(key)).0 / coeff)
        .fold(f64::INFINITY, f64::min)
}

/// Moles of a species available in solution for a solvent-gated
/// reaction: the liquid (already-dissolved) fraction plus whatever
/// solid could still dissolve up to the handbook solubility limit.
fn available_dissolved(vessel: &Vessel, key: &str, solvent: &str) -> f64 {
    let id = SpeciesId::new(key);
    let liquid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species == id && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum();
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species == id && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    if solid <= 0.0 {
        return liquid;
    }
    if let Some(row) = crate::nonaqueous::ORGANIC_SOLUBILITY
        .iter()
        .find(|r| r.solute == key && r.solvent == solvent)
    {
        let solvent_id = SpeciesId::new(solvent);
        let solvent_data =
            crate::species::lookup(&solvent_id).expect("known solvents are registry species");
        let solvent_moles: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species == solvent_id && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum();
        let solvent_ml = solvent_moles * solvent_data.molar_mass / solvent_data.density;
        let species_data = match crate::species::lookup(&id) {
            Some(d) => d,
            None => return liquid,
        };
        let limit_mol = row.g_per_100ml * (solvent_ml / 100.0) / species_data.molar_mass;
        let room = (limit_mol - liquid).max(0.0);
        liquid + solid.min(room)
    } else {
        liquid + solid
    }
}

fn extent_in_solvent(vessel: &Vessel, reaction: &CuratedReaction, solvent: &str) -> f64 {
    reaction
        .reactants
        .iter()
        .map(|(key, coeff)| available_dissolved(vessel, key, solvent) / coeff)
        .fold(f64::INFINITY, f64::min)
}

/// Withdraw from liquid phase first, then solid — ensures only the
/// dissolved (or would-dissolve) fraction is consumed.
fn withdraw_from_solution(vessel: &mut Vessel, species: &SpeciesId, moles: Moles) {
    let mut remaining = moles.0;
    for p in vessel.contents.iter_mut() {
        if &p.species == species && p.phase == Phase::Liquid && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    for p in vessel.contents.iter_mut() {
        if &p.species == species && p.phase == Phase::Solid && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    vessel.contents.retain(|p| p.moles.0 > 1e-15);
}

/// Applies every curated reaction whose reactants are present, to
/// completion, before the generic solvers run.
pub struct CuratedEquilibrator;

impl Equilibrator for CuratedEquilibrator {
    fn name(&self) -> &'static str {
        "curated-reactions"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        let solvent = crate::nonaqueous::single_organic_solvent(vessel);
        REACTIONS.iter().any(|r| {
            if let Some(min_t) = r.min_temp_k {
                if vessel.temperature.0 < min_t {
                    return false;
                }
            }
            if let Some(cat) = r.catalyst {
                if vessel.moles_of(&SpeciesId::new(cat)).0 <= TRACE {
                    return false;
                }
            }
            if let Some(req) = r.solvent {
                solvent == Some(req) && extent_in_solvent(vessel, r, req) > TRACE
            } else {
                extent(vessel, r) > TRACE
            }
        })
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        // Solvent-gated reactions fire at most once per equilibration:
        // the dissolved fraction reacts, and that is all until the next
        // step re-equilibrates (Le Chatelier pull from the solid is a
        // multi-step kinetic, not an instant cascade).
        let mut solvent_gated_done = [false; REACTIONS.len()];
        for _ in 0..8 {
            let mut progressed = false;
            let solvent = crate::nonaqueous::single_organic_solvent(vessel);
            for (i, reaction) in REACTIONS.iter().enumerate() {
                if let Some(min_t) = reaction.min_temp_k {
                    if vessel.temperature.0 < min_t {
                        continue;
                    }
                }
                if let Some(cat) = reaction.catalyst {
                    if vessel.moles_of(&SpeciesId::new(cat)).0 <= TRACE {
                        continue;
                    }
                }
                let x = if let Some(req) = reaction.solvent {
                    if solvent != Some(req) || solvent_gated_done[i] {
                        continue;
                    }
                    extent_in_solvent(vessel, reaction, req)
                } else {
                    extent(vessel, reaction)
                };
                if x <= TRACE {
                    continue;
                }
                progressed = true;
                if reaction.solvent.is_some() {
                    solvent_gated_done[i] = true;
                }
                for (key, coeff) in reaction.reactants {
                    if reaction.solvent.is_some() {
                        withdraw_from_solution(vessel, &SpeciesId::new(key), Moles(x * coeff));
                    } else {
                        vessel.withdraw(&SpeciesId::new(key), Moles(x * coeff));
                    }
                }
                events.push(Event::ReactionOccurred {
                    vessel: vessel.id,
                    equation: reaction.equation.to_string(),
                });
                for (key, coeff, phase) in reaction.products {
                    let n = Moles(x * coeff);
                    if *phase == Phase::Gas {
                        let species = SpeciesId::new(key);
                        if vessel.retain_gas(species.clone(), n) {
                            events.push(Event::GasContained {
                                vessel: vessel.id,
                                species,
                                moles: n,
                            });
                        } else {
                            events.push(Event::GasEvolved {
                                vessel: vessel.id,
                                species,
                                moles: n,
                            });
                        }
                    } else {
                        vessel.deposit(SpeciesId::new(key), n, *phase);
                    }
                }
            }
            if !progressed {
                break;
            }
        }
        Ok(events)
    }
}
