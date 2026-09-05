//! BRD-023 / BRD-014: which metal corrodes when two of them share an
//! electrolyte, and what a coating does about it.
//!
//! # What was already here, and what was missing
//!
//! The bench could already rust iron. `kinetics::REGISTRY` carries
//! `iron-corrosion` — `4 Fe + 3 O₂ → 2 Fe₂O₃`, gated on liquid water by
//! its locality, stopping when the oxygen runs out, and accelerated by
//! chloride through a curated catalyst so that brine visibly beats tap
//! water. Three of the four things a corrosion question asks were
//! therefore already modelled, with provenance, on the slow clock where
//! they belong. **This module does not add a second rate model**, because
//! a bench with two opinions about how fast a nail rusts is worse than a
//! bench with one.
//!
//! What was missing is the part `iron-corrosion` says in its own
//! uncertainty note that it does not resolve: *"the two half-reactions,
//! their separation on the metal, and the differential-aeration cell that
//! drives real corrosion are not resolved."* Without that, iron beside
//! zinc in salt water rusted exactly as fast as iron alone and the zinc
//! sat there doing nothing — which is the opposite of what galvanising
//! is for, and it is what the curiosity corpus kept asking about.
//!
//! # The model
//!
//! **1. Two metals in one electrolyte are a cell, and the cell decides.**
//! The lower-E° metal is the anode: it gives up the electrons for both,
//! and the nobler one is cathodically protected while any of it is left.
//! The potentials are `displacement::SERIES`, the same CRC table the
//! displacement route computes with, so the bench holds one activity
//! series and not two. That single rule is galvanising, the scratched
//! galvanised sheet, the sacrificial anode on a hull, and a copper
//! fitting eating a steel pipe.
//!
//! **2. A barrier is a claim about an object.** [`BARRIERS`] carries the
//! chromium(III) passive film of stainless steel and the paint film of
//! painted iron, keyed on the `MaterialLot` source the material route
//! stamps, and honoured only when *every* lot of that metal in the vessel
//! arrived under a barrier. All three metal recipes said in their own
//! `lot_assumptions` that they resolve to iron the bench will attack with
//! no representation of the film or coat that is the whole point of the
//! object; this is where the bench keeps the sentence they could not.
//!
//! **3. Both rules are enforced where the metal is actually eaten.**
//! [`allows_reaction`] is consulted wherever a corrosion rate is
//! computed — `kinetics_integrator`'s `expression_rate`, which is the one
//! the slow clock uses, its vessel-state twin in `kinetics`, and
//! `can_run`, which is what `applicable` and the honesty pass read. A
//! protected metal does not merely get TOLD it is protected: its
//! corrosion rate is zero. `zinc-corrosion` is the companion entry that
//! makes the sacrifice real, so the zinc is consumed while the iron is
//! not — and every protection test is paired with the unprotected
//! control that rusts under the same script, because a verdict over a
//! beaker that contradicts it is worse than no verdict.
//!
//! **4. Everything it decides, it says.** [`verdicts`] turns the same
//! state into one `Event::Corroded` per metal, positive or negative, so
//! "this is not rusting, and here is which of the requirements is
//! missing or what is protecting it" is a computed result rather than a
//! silence.
//!
//! # What this route does not claim
//!
//! * **Protection is all-or-nothing.** A real galvanic couple leaves the
//!   cathode a small residual current, and its magnitude depends on the
//!   anode-to-cathode area ratio — a small zinc spot on a large steel
//!   sheet protects far less than a full coat. This bench has no areas,
//!   so protection here is a switch, and the switch is honest about being
//!   one.
//! * **The anode does not speed up when it takes on the cathode's work.**
//!   A sacrificial zinc carrying the iron's current corrodes faster than
//!   zinc alone. `zinc-corrosion` runs at its own free-corrosion rate
//!   either way, so the zinc lasts too long by the factor the coupling
//!   would have cost it.
//! * **Oxygen has to be in the vessel.** Deliberately the same rule
//!   `iron-corrosion` already states in its validity note: an open beaker
//!   does not yet draw oxygen from the room into its water. So a nail in
//!   plain water with no `add O2` gets a typed *nothing rusts here, and
//!   this is why* rather than a rate, and the two halves of the bench
//!   agree about the same beaker.
//! * **No pitting, no crevice corrosion, no differential aeration.** The
//!   cell here is metal-versus-metal; the cell that rusts a lone nail
//!   fastest at the waterline is a difference in oxygen, and neither this
//!   module nor `iron-corrosion` resolves it.
//! * **No atmospheric weathering.** The green of an old copper contact is
//!   a patina of basic copper carbonate and sulfate, grown over years from
//!   CO₂ and SO₂. There is no gaseous weathering route on this bench and
//!   the copper verdict names that rather than inventing one.
//! * **Acid corrosion belongs to `displacement`.** Where free acidity or a
//!   dissolved noble-metal ion is present the cathode is not oxygen, and
//!   this route stands aside so the two never both narrate one beaker.

use crate::displacement::{Couple, SERIES};
use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError, SolverRouteKind};
use crate::species::{self, Phase, SpeciesId};
use crate::vessel::Vessel;

/// A barrier that a named object carries and its bare metal does not.
///
/// Keyed on the lot source the material route stamps on every component
/// it deposits (`material recipe <recipe id>`), because the barrier is a
/// fact about the object and the bench resolves the object into ordinary
/// species the moment it is added.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Barrier {
    /// `MaterialLot::source` written by the material route.
    pub lot_source: &'static str,
    /// The metal it protects.
    pub metal: &'static str,
    /// The sentence the verdict carries.
    pub why: &'static str,
    pub source: &'static str,
}

pub const BARRIERS: &[Barrier] = &[
    Barrier {
        lot_source: "material recipe metal/stainless-steel",
        metal: "Fe",
        why: "this iron came in as stainless steel, and stainless steel does not rust for a reason ordinary steel cannot borrow: its chromium reacts with oxygen faster than the iron does and builds a chromium(III) oxide film a couple of nanometres thick, transparent, tightly bound, and self-repairing — scratch it and it reforms out of the same air that would have rusted the iron. Iron oxide is none of those things: it is loose, it flakes, and it exposes fresh metal underneath, which is why rust does not stop and a passive film does",
        source: "Passivity of Fe-Cr alloys and the Cr2O3 film: Uhlig & Revie, Corrosion and Corrosion Control, 4th ed., chapter on passivity; the ~11% chromium threshold for stainless behaviour is the classical Tammann result. Editorial judgement (Kerotakis): no Cr species is installed in this registry, so the film is asserted from the object's identity rather than computed from its chromium — the recipe metal/stainless-steel says exactly this in its own lot assumptions, and this row is the bench acting on it",
    },
    Barrier {
        lot_source: "material recipe metal/painted-iron",
        metal: "Fe",
        why: "this iron came in under a complete paint film, and a sound coating simply keeps the water and the oxygen off the steel: no electrolyte on the surface, no cathodic reaction, no cell. That is the whole mechanism, and it is also the whole limitation — paint is a barrier and not a cure. Break the film anywhere and the steel at the break corrodes at the bare-metal rate, with no help from the paint around it, because paint gives no cathodic protection. Zinc does, which is the difference between a chipped painted railing and a scratched galvanised one",
        source: "Barrier protection by organic coatings and the absence of cathodic protection at a defect: Jones, Principles and Prevention of Corrosion, 2nd ed., chapter on coatings. Editorial judgement (Kerotakis): the recipe metal/painted-iron holds the paint as a conserved unresolved 5% with no geometry, so 'the film is complete' is an assumption of the object rather than a state the bench can inspect — there is no scratch verb, and a broken film is described here rather than simulated",
    },
];

/// The kinetic corrosion reactions this route gates, and the metal each
/// one eats.
///
/// A reaction named here does not run while its metal is protected. Any
/// reaction not named here is none of this module's business.
pub const GATED_REACTIONS: &[(&str, &str)] = &[("iron-corrosion", "Fe"), ("zinc-corrosion", "Zn")];

/// What the route has to say about one metal in one vessel.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// Registry key of the metal.
    pub metal: &'static str,
    /// Whether it is corroding here.
    pub corroding: bool,
    pub why: String,
}

fn solid_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

fn aqueous_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

fn has_liquid_water(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > 0.0)
}

/// Oxygen booked into this vessel, in any phase.
///
/// Deliberately the same reading `iron-corrosion`'s oxygen order term
/// makes — the vessel's own oxygen, wherever it sits — so the narration
/// and the rate never disagree about whether a beaker can rust.
fn has_oxygen(vessel: &Vessel) -> bool {
    vessel.moles_of(&SpeciesId::new("O2")).0 > crate::OBSERVABLE_MOLES
}

/// Whether the displacement route owns this vessel instead.
///
/// Two cases, and in neither is the cathode oxygen: free acidity makes it
/// `2 H⁺ + 2 e⁻ → H₂`, and a dissolved noble-metal ion makes it that metal
/// plating out. Both are computed by `displacement` with its own
/// thermodynamics and its own overpotential gate, and two solvers must not
/// both narrate one beaker.
fn displacement_owns(vessel: &Vessel) -> bool {
    if crate::displacement::unspent_acidity(vessel) > crate::OBSERVABLE_MOLES {
        return true;
    }
    // A dissolved metal ion only makes this displacement's beaker if some
    // metal standing in it sits BELOW that ion in the series. Zinc beside
    // its own Zn²⁺ is not a displacement — and that case is not
    // hypothetical, because zinc corroding is how the ion got there.
    // Reading any dissolved ion as displacement would have this route
    // stand aside the moment it succeeded.
    let solids = metals_present(vessel);
    SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| aqueous_moles(vessel, c.oxidised) > crate::OBSERVABLE_MOLES)
        .any(|ion| solids.iter().any(|metal| metal.e0_volts < ion.e0_volts))
}

fn barrier_for(vessel: &Vessel, metal: &str) -> Option<&'static Barrier> {
    let mut found: Option<&'static Barrier> = None;
    for lot in vessel.lots.iter() {
        if lot.species.0 != metal || lot.phase != Phase::Solid {
            continue;
        }
        let source = lot.source.as_deref().unwrap_or("");
        // Every lot of this metal has to have arrived under a barrier.
        // A stainless spoon dropped in beside a bare nail protects the
        // spoon and not the nail, and the bench cannot tell their iron
        // apart once it is in `contents` — so the `?` here is the claim
        // being withdrawn the moment one bare lot turns up, rather than
        // extended to metal it was never about.
        let barrier = BARRIERS
            .iter()
            .find(|b| b.metal == metal && b.lot_source == source)?;
        found = Some(barrier);
    }
    // `None` when no lot of this metal exists at all, which is the right
    // answer: a barrier is a claim about an object that was added, and
    // metal that arrived by some other road has none.
    found
}

fn display_name(key: &str) -> &str {
    species::lookup_key(key).map(|d| d.name).unwrap_or(key)
}

/// The metals present as solids that the series can speak for.
fn metals_present(vessel: &Vessel) -> Vec<&'static Couple> {
    SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| solid_moles(vessel, c.reduced) > crate::OBSERVABLE_MOLES)
        .collect()
}

/// The anode of this vessel's corrosion cell: the lowest-E° metal in
/// contact that is neither noble nor behind a barrier.
///
/// Metals above hydrogen are never the anode of an oxygen cell here — in
/// aerated neutral water copper and silver are cathodes, and their own
/// slow tarnishing is atmospheric chemistry this bench has no route for.
/// A barriered object is not in the circuit at all.
pub fn anode(vessel: &Vessel) -> Option<&'static Couple> {
    metals_present(vessel)
        .into_iter()
        .filter(|c| c.e0_volts < 0.0)
        .filter(|c| barrier_for(vessel, c.reduced).is_none())
        .min_by(|a, b| a.e0_volts.total_cmp(&b.e0_volts))
}

/// Whether this metal is spared here — behind a barrier, or cathodically
/// protected by something lower in the series sharing its electrolyte.
///
/// This is the predicate `allows_reaction` gates on, so it is the one
/// place where "the zinc protects the iron" stops being a sentence and
/// becomes a difference in what is left in the beaker.
pub fn is_protected(vessel: &Vessel, metal: &str) -> bool {
    if !has_liquid_water(vessel) {
        return false;
    }
    if barrier_for(vessel, metal).is_some() {
        return true;
    }
    anode(vessel).is_some_and(|a| a.reduced != metal)
}

/// Whether the galvanic rule lets a kinetic corrosion reaction run.
///
/// Called from `KineticReaction::expression_rate`, so a blocked reaction
/// has a rate of zero rather than merely being filtered out of a list,
/// and from `can_run` so the two agree. A reaction this module does not
/// name is always allowed: the gate speaks only for the corrosion
/// entries it lists in [`GATED_REACTIONS`].
pub fn allows_reaction(reaction_id: &str, vessel: &Vessel) -> bool {
    match GATED_REACTIONS.iter().find(|(id, _)| *id == reaction_id) {
        Some((_, metal)) => !is_protected(vessel, metal),
        None => true,
    }
}

/// Every corrosion verdict this vessel earns, one per metal present.
///
/// Empty when the route has nothing to say: no metal, no liquid water, or
/// a beaker the displacement route owns.
pub fn verdicts(vessel: &Vessel) -> Vec<Verdict> {
    if !has_liquid_water(vessel) || displacement_owns(vessel) {
        return Vec::new();
    }
    let present = metals_present(vessel);
    if present.is_empty() {
        return Vec::new();
    }

    let anode = anode(vessel);
    // Which metals the anode is holding up, named once so its own
    // sentence can say so. Barriered and noble metals are excluded:
    // neither owes its survival to the anode.
    let protected: Vec<&'static str> = present
        .iter()
        .copied()
        .filter(|c| c.e0_volts < 0.0)
        .filter(|c| barrier_for(vessel, c.reduced).is_none())
        .filter(|c| anode.is_some_and(|a| a.reduced != c.reduced))
        .map(|c| c.reduced)
        .collect();
    let protecting = if protected.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = protected.iter().copied().map(display_name).collect();
        format!(", and the {} beside it is spared", names.join(" and "))
    };
    let oxygen = has_oxygen(vessel);

    let mut out = Vec::new();
    for couple in present.iter().copied() {
        let metal = couple.reduced;
        let name = display_name(metal);

        if let Some(barrier) = barrier_for(vessel, metal) {
            out.push(Verdict {
                metal,
                corroding: false,
                why: barrier.why.to_string(),
            });
            continue;
        }

        if couple.e0_volts > 0.0 {
            let patina = if metal == "Cu" {
                ". The green on an old copper contact is not rust and is not this reaction: it is a patina of basic copper carbonate and sulfate, grown over years from carbon dioxide and sulfur dioxide in the air on a first film of Cu2O. That is atmospheric weathering, it needs a gas phase this bench does not carry, and no route here claims it"
            } else {
                ""
            };
            out.push(Verdict {
                metal,
                corroding: false,
                why: format!(
                    "{name} sits above hydrogen in the activity series (E° {:+.3} V), so in aerated neutral water it is the cathode of any corrosion cell rather than the anode: oxygen takes electrons at its surface and it stays as the metal{patina}",
                    couple.e0_volts
                ),
            });
            continue;
        }

        let Some(anode) = anode else {
            continue;
        };

        if anode.reduced != metal {
            let other = display_name(anode.reduced);
            out.push(Verdict {
                metal,
                corroding: false,
                why: format!(
                    "{name} does not corrode while {other} is in contact with it here. Two metals in one electrolyte are a cell, and the cell decides: {other} sits below {name} in the series (E° {:+.3} V against {:+.3} V), so {other} is the anode and gives up the electrons for both while {name} is the cathode and is spared. This is what galvanising is, and it is why a scratch does not undo it — the zinc protects the iron it is merely NEXT to, not only the iron it covers, and it goes on doing so until the zinc is gone",
                    anode.e0_volts, couple.e0_volts
                ),
            });
            continue;
        }

        if !oxygen {
            let and_then = if protected.is_empty() {
                String::new()
            } else {
                let names: Vec<&str> = protected.iter().copied().map(display_name).collect();
                format!(
                    ". When there is oxygen it is the {name} that goes and not the {}, because {name} is the lowest-E° metal in contact",
                    names.join(" or the ")
                )
            };
            out.push(Verdict {
                metal,
                corroding: false,
                why: format!(
                    "nothing rusts in this vessel, because there is no oxygen in it. Corrosion needs three things at once — the metal, liquid water and oxygen — since it is a circuit: {name} would give up electrons at the anode, oxygen would take them at the cathode, and the water carries the ions between the two. This bench does not yet draw oxygen from the room into an open beaker, so the oxygen has to be put in the vessel for the slow clock to see it{and_then}"
                ),
            });
            continue;
        }

        out.push(Verdict {
            metal,
            corroding: true,
            why: format!(
                "{name}, liquid water and oxygen are all three here, which is everything corrosion needs: {name} gives up electrons at the anode, oxygen takes them at the cathode, and the water between them carries the ions{protecting}. {name} is the lowest-E° metal in contact, so it is the anode of every cell in this vessel and it is the one that goes"
            ),
        });
    }
    out
}

/// Whether the slow clock is actually corroding this metal in this
/// vessel: a gated corrosion reaction names it, and that reaction can
/// run here.
///
/// `displacement::bystanders` asks before writing its "{metal} stays as
/// the metal … its slow reaction with water itself is a rate this lab
/// does not model" apology, and the predicate is deliberately this one
/// rather than "the corrosion route has a verdict". The apology is about
/// the metal's OWN reaction with water, and it stays true in every case
/// this module does not actually eat the metal in:
///
/// * magnesium in brine — no corrosion entry names magnesium, so nothing
///   here models what it does in water and the apology is the honest
///   answer (a phreeqc test pins exactly this);
/// * iron in de-aerated water — the entry exists and cannot run, so the
///   slow clock is not corroding it either;
/// * iron beside zinc — protected, so again nothing is eating it, and a
///   stack without `CorrosionEquilibrator` would otherwise lose the
///   apology and gain no verdict in its place.
///
/// Only where `iron-corrosion` or `zinc-corrosion` is genuinely running
/// does the sentence become false, and only there is it withdrawn.
pub fn kinetics_corrodes(vessel: &Vessel, metal: &str) -> bool {
    crate::kinetics::REGISTRY.iter().any(|reaction| {
        GATED_REACTIONS
            .iter()
            .any(|(id, gated)| *id == reaction.id && *gated == metal)
            && reaction.can_run(vessel)
    })
}

/// Corrosion as a solver: it decides and it narrates, and the slow clock
/// is what moves the matter.
///
/// It changes no inventory itself on purpose. The corrosion reactions
/// live in `kinetics::REGISTRY`, where `wait` drives them with a rate and
/// conserves the ledger; this pass is the part that says which of them
/// may run and why, and its `Event::Corroded` is the sentence for the
/// step the learner is looking at.
#[derive(Debug, Default, Clone, Copy)]
pub struct CorrosionEquilibrator;

impl Equilibrator for CorrosionEquilibrator {
    fn name(&self) -> &'static str {
        "corrosion"
    }

    fn route_kind(&self) -> SolverRouteKind {
        SolverRouteKind::Computed
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        !verdicts(vessel).is_empty()
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let id = vessel.id;
        Ok(verdicts(vessel)
            .into_iter()
            .map(|v| Event::Corroded {
                vessel: id,
                species: SpeciesId::new(v.metal),
                corroding: v.corroding,
                why: v.why,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_barrier_names_a_series_metal() {
        for barrier in BARRIERS {
            assert!(
                SERIES.iter().any(|c| c.reduced == barrier.metal),
                "{} is not in the activity series",
                barrier.metal
            );
        }
    }

    #[test]
    fn every_gated_reaction_exists_and_eats_the_metal_it_names() {
        for (id, metal) in GATED_REACTIONS {
            let reaction = crate::kinetics::REGISTRY
                .iter()
                .find(|r| r.id == *id)
                .unwrap_or_else(|| panic!("{id} is not a kinetic reaction"));
            assert!(
                reaction
                    .reactants()
                    .any(|term| term.species == *metal && term.phase == Phase::Solid),
                "{id} does not consume solid {metal}"
            );
        }
    }

    #[test]
    fn an_ungated_reaction_is_never_blocked() {
        let mut vessel = Vessel::new(crate::vessel::VesselId(0), "beaker");
        vessel.deposit(
            SpeciesId::new("water"),
            crate::units::Moles(1.0),
            Phase::Liquid,
        );
        vessel.deposit(
            SpeciesId::new("Zn"),
            crate::units::Moles(0.02),
            Phase::Solid,
        );
        vessel.deposit(
            SpeciesId::new("Fe"),
            crate::units::Moles(0.02),
            Phase::Solid,
        );
        assert!(allows_reaction("peroxide-decomposition", &vessel));
        assert!(allows_reaction("zinc-corrosion", &vessel));
        assert!(!allows_reaction("iron-corrosion", &vessel));
    }
}
