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
}

/// Hand-verified seed set. Grows into the codex (P4).
pub const REACTIONS: &[CuratedReaction] = &[
    CuratedReaction {
        equation: "NH3 + NaOCl → NH2Cl↑ + NaOH",
        reactants: &[("NH3", 1.0), ("NaOCl", 1.0)],
        products: &[("NH2Cl", 1.0, Phase::Gas), ("NaOH", 1.0, Phase::Aqueous)],
    },
    CuratedReaction {
        equation: "NaOCl + 2 HCl → Cl2↑ + NaCl + H2O",
        reactants: &[("NaOCl", 1.0), ("HCl", 2.0)],
        products: &[
            ("Cl2", 1.0, Phase::Gas),
            ("NaCl", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
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

/// Applies every curated reaction whose reactants are present, to
/// completion, before the generic solvers run.
pub struct CuratedEquilibrator;

impl Equilibrator for CuratedEquilibrator {
    fn name(&self) -> &'static str {
        "curated-reactions"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        REACTIONS.iter().any(|r| extent(vessel, r) > TRACE)
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        // A product of one reaction can feed another; a few passes settle it.
        for _ in 0..8 {
            let mut progressed = false;
            for reaction in REACTIONS {
                let x = extent(vessel, reaction);
                if x <= TRACE {
                    continue;
                }
                progressed = true;
                for (key, coeff) in reaction.reactants {
                    vessel.withdraw(&SpeciesId::new(key), Moles(x * coeff));
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
