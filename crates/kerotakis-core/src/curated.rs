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
