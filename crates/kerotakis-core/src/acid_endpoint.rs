//! Bounded sulfate-acidified permanganate/peroxide stoichiometric endpoint.
//! This is neither a rate law nor a complete thermal/product-domain model.
use crate::aqueous_continuation::{continue_via, CurrentAqueousAnswer};
use crate::curated::{CuratedReaction, PostAqueousEndpoint, REACTIONS};
use crate::ledger::ConservedLedger;
use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::vessel::{Headspace, SolutionScope, Vessel};
use crate::{Equilibrator, Moles, Phase, SolveError, SpeciesId};

const TRACE: f64 = 1e-12;
// Editorial implementation domain, not an experimentally determined threshold.
const MAX_PH: f64 = 3.0;

fn pool(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && phases.contains(&p.phase))
        .map(|p| p.moles.0)
        .sum()
}

/// Structural hint only: it cannot authorize consumption or assert fresh pH.
pub(crate) fn may_apply(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| matches!(p.species.0.as_str(), "MnO4-" | "KMnO4") && p.moles.0 > TRACE)
        && vessel
            .contents
            .iter()
            .any(|p| p.species.0 == "H2O2" && p.moles.0 > TRACE)
}

/// Cheap raw-stock filter, before choosing trusted optional characterization.
/// It includes complete sulfate and potassium bottles before their first solve.
pub(crate) fn supported_input(vessel: &Vessel) -> bool {
    let mut water = 0.0;
    let mut sulfate = 0.0;
    let mut potassium = 0.0;
    for p in &vessel.contents {
        if !p.moles.0.is_finite() || p.moles.0 < 0.0 {
            return false;
        }
        if p.moles.0 == 0.0 {
            continue;
        }
        match p.species.0.as_str() {
            "water" if p.phase == Phase::Liquid => water += p.moles.0,
            "SO4-2" | "H2SO4" => sulfate += p.moles.0,
            "K+" | "KMnO4" => potassium += p.moles.0,
            "MnO4-" | "Mn+2" | "H2O2" | "H+" | "O2" | "N2" => {}
            _ => return false,
        }
    }
    [water, sulfate, potassium]
        .iter()
        .all(|n| n.is_finite() && *n > TRACE)
}

/// The same finite-domain gate is used by eligibility queries and application.
/// Application additionally requires the continuation's trusted answer receipt.
pub(crate) fn admitted_extent(vessel: &Vessel, reaction: &CuratedReaction) -> Option<f64> {
    if reaction.post_aqueous != Some(PostAqueousEndpoint::AcidicPermanganatePeroxide)
        || !matches!(vessel.headspace, Headspace::Open | Headspace::Sealed { .. })
        || !vessel.surfaces.is_empty()
        || !vessel.electrodes.is_empty()
        || !vessel.exchanges.is_empty()
        || !vessel.adsorbed.is_empty()
        || !vessel.solid_solutions.is_empty()
        || !vessel.material_objects.is_empty()
        || !vessel.unresolved_materials.is_empty()
    {
        return None;
    }
    let answer = vessel.solution.as_ref()?;
    if answer.scope != SolutionScope::Complete || !answer.ph.is_finite() || answer.ph > MAX_PH {
        return None;
    }
    // Explicit closed medium: chloride and other competing reagents cannot
    // become inert spectators merely by meeting the pH gate. H+ is the owned
    // analytical coordinate, not free_proton or an inferred amount from pH.
    for p in &vessel.contents {
        if !p.moles.0.is_finite() || p.moles.0 < 0.0 {
            return None;
        }
        if p.moles.0 == 0.0 {
            continue;
        }
        let allowed = match p.species.0.as_str() {
            "water" => p.phase == Phase::Liquid,
            "H+" | "K+" | "MnO4-" | "Mn+2" | "SO4-2" => p.phase == Phase::Aqueous,
            "H2O2" => matches!(p.phase, Phase::Aqueous | Phase::Liquid),
            "O2" => matches!(p.phase, Phase::Aqueous | Phase::Gas),
            "N2" => p.phase == Phase::Gas,
            _ => false,
        };
        if !allowed {
            return None;
        }
    }
    let aqueous = &[Phase::Aqueous];
    let water = pool(vessel, "water", &[Phase::Liquid]);
    let sulfate = pool(vessel, "SO4-2", aqueous);
    let potassium = pool(vessel, "K+", aqueous);
    let permanganate = pool(vessel, "MnO4-", aqueous);
    let peroxide = pool(vessel, "H2O2", &[Phase::Aqueous, Phase::Liquid]);
    let acid = pool(vessel, "H+", aqueous);
    let free = (-vessel.solute_charge).max(0.0);
    let unspent = crate::displacement::unspent_acidity(vessel);
    if ![
        water,
        sulfate,
        potassium,
        permanganate,
        peroxide,
        acid,
        free,
        unspent,
    ]
    .iter()
    .all(|n| n.is_finite())
        || water <= TRACE
        || sulfate <= TRACE
        || potassium <= TRACE
        || !vessel.solute_charge.is_finite()
    {
        return None;
    }
    // Require a complete counterion inventory, not a bare ionic analytical
    // problem whose pH solver supplies missing charge by compensation.
    let ledger = ConservedLedger::from_vessel(vessel);
    let charge_scale: f64 = vessel.contents.iter().map(|p| p.moles.0).sum();
    if !ledger.charge.is_finite()
        || !charge_scale.is_finite()
        || ledger.charge.abs() > 1e-10 + 64.0 * f64::EPSILON * charge_scale
    {
        return None;
    }
    let x = (permanganate / 2.0)
        .min(peroxide / 5.0)
        .min(acid / 6.0)
        .min(free / 6.0)
        .min(unspent / 6.0);
    (x.is_finite() && x > TRACE).then_some(x)
}

enum EndpointRefusal<'a> {
    Domain,
    ReactantDebit {
        species: &'a str,
        requested_moles: f64,
        remaining_moles: f64,
    },
    ProtonDebit {
        requested_moles: f64,
        observed_moles: f64,
    },
}

impl EndpointRefusal<'_> {
    fn phrase(&self) -> Phrase {
        match self {
            Self::Domain => Phrase::bare(
                "solver-reason.endpoint.domain",
                "This mixture is outside the admitted acidic permanganate/peroxide endpoint domain.",
            ),
            Self::ReactantDebit {
                species,
                requested_moles,
                remaining_moles,
            } => Phrase::new(
                "solver-reason.endpoint.reactant-debit",
                "The endpoint could not debit {species}: {remaining} mol remained after requesting {requested} mol.",
                vec![
                    ("species".into(), Slot::term("species", *species)),
                    ("remaining".into(), Slot::number(remaining_moles.to_string())),
                    ("requested".into(), Slot::number(requested_moles.to_string())),
                ],
            ),
            Self::ProtonDebit {
                requested_moles,
                observed_moles,
            } => Phrase::new(
                "solver-reason.endpoint.proton-debit",
                "The endpoint consumed {observed} mol of the owned proton inventory; {requested} mol was required.",
                vec![
                    ("observed".into(), Slot::number(observed_moles.to_string())),
                    ("requested".into(), Slot::number(requested_moles.to_string())),
                ],
            ),
        }
    }
}

fn refused(reason: EndpointRefusal<'_>, detail: &str) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(reason.phrase())),
        solver: "acidic-permanganate-peroxide".into(),
        detail: detail.into(),
    }
}

fn debit(vessel: &mut Vessel, key: &str, amount: f64, phases: &[Phase]) -> Result<(), SolveError> {
    let mut remaining = amount;
    for phase in phases {
        let taken = vessel
            .withdraw_phase(&SpeciesId::new(key), Moles(remaining), *phase)
            .0;
        remaining = (remaining - taken).max(0.0);
    }
    if remaining > 64.0 * f64::EPSILON * amount {
        return Err(refused(
            EndpointRefusal::ReactantDebit {
                species: key,
                requested_moles: amount,
                remaining_moles: remaining,
            },
            "eligible reactant debit did not match admitted extent",
        ));
    }
    Ok(())
}

fn transform(vessel: &mut Vessel, reaction: &CuratedReaction) -> Result<Vec<Event>, SolveError> {
    let x = admitted_extent(vessel, reaction).ok_or_else(|| {
        refused(
            EndpointRefusal::Domain,
            "endpoint is outside its admitted domain",
        )
    })?;
    let acid_before = pool(vessel, "H+", &[Phase::Aqueous]);
    // Admission guarantees this legacy helper's free-acid branch has an
    // actual exclusively aqueous H+ inventory and needs no weak-acid donor.
    crate::displacement::consume_acidity(vessel, 6.0 * x);
    let spent = acid_before - pool(vessel, "H+", &[Phase::Aqueous]);
    if (spent - 6.0 * x).abs() > 64.0 * f64::EPSILON * acid_before {
        return Err(refused(
            EndpointRefusal::ProtonDebit {
                requested_moles: 6.0 * x,
                observed_moles: spent,
            },
            "owned proton debit did not match admitted extent",
        ));
    }
    debit(vessel, "MnO4-", 2.0 * x, &[Phase::Aqueous])?;
    debit(vessel, "H2O2", 5.0 * x, &[Phase::Aqueous, Phase::Liquid])?;
    vessel.deposit(SpeciesId::new("Mn+2"), Moles(2.0 * x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("water"), Moles(8.0 * x), Phase::Liquid);
    let species = SpeciesId::new("O2");
    let moles = Moles(5.0 * x);
    let gas = if vessel.retain_gas(species.clone(), moles) {
        Event::GasContained {
            vessel: vessel.id,
            species,
            moles,
        }
    } else {
        Event::GasEvolved {
            vessel: vessel.id,
            species,
            moles,
        }
    };
    Ok(vec![Event::ReactionOccurred { vessel: vessel.id, equation: reaction.equation.into() }, gas,
        Event::not_modeled(vessel.id, NotModelledCause::ModelBoundary, Phrase::bare(
            "not-modeled.acid-permanganate-endpoint",
            "this sulfate-acidified permanganate/peroxide calculation is a bounded stoichiometric endpoint. No reaction timescale or reaction heat is adopted; chloride, competing reagents and other product domains are outside this model",
        ))])
}

pub(crate) fn outside_domain(vessel: &Vessel) -> Event {
    Event::not_modeled(vessel.id, NotModelledCause::ModelBoundary, Phrase::bare(
        "not-modeled.acid-permanganate-domain",
        "no acidic permanganate/peroxide endpoint was applied: this route requires a trusted fresh complete aqueous answer at pH at most 3, an owned aqueous proton inventory, a complete sulfate/potassium medium and an open or fixed sealed boundary; unsupported mixtures and inaccessible reactants are left to their existing models",
    ))
}

pub(crate) fn run(
    vessel: &mut Vessel,
    answer: &CurrentAqueousAnswer,
    aqueous: &mut dyn Equilibrator,
) -> Result<Option<Vec<Event>>, SolveError> {
    let reaction = REACTIONS
        .iter()
        .find(|r| r.post_aqueous == Some(PostAqueousEndpoint::AcidicPermanganatePeroxide))
        .expect("typed acidic endpoint is installed");
    if admitted_extent(vessel, reaction).is_none() {
        return Ok(None);
    }
    // Keep an outer transaction until the computed product domain is known.
    // Acid exhaustion with BOTH reactants left can enter competing MnO2
    // chemistry. Exhausted MnO4/peroxide does not impose a blanket acidic
    // final pH on the already formed Mn(II) endpoint products.
    let mut candidate = vessel.clone();
    let events = continue_via(&mut candidate, answer, aqueous, |v| transform(v, reaction))?;
    let residual_mn = pool(&candidate, "MnO4-", &[Phase::Aqueous]);
    let residual_peroxide = pool(&candidate, "H2O2", &[Phase::Aqueous, Phase::Liquid]);
    if residual_mn > 2.0 * TRACE
        && residual_peroxide > 5.0 * TRACE
        && candidate.solution.as_ref().is_some_and(|s| s.ph > MAX_PH)
    {
        return Ok(None);
    }
    *vessel = candidate;
    Ok(Some(events))
}

#[cfg(test)]
mod refusal_tests {
    use super::{refused, EndpointRefusal};
    use crate::SolveError;

    #[test]
    fn endpoint_refusals_keep_typed_reason_and_raw_detail() {
        let cases = [
            (
                EndpointRefusal::Domain,
                "endpoint is outside its admitted domain",
                "solver-reason.endpoint.domain",
            ),
            (
                EndpointRefusal::ReactantDebit {
                    species: "MnO4-",
                    requested_moles: 0.25,
                    remaining_moles: 0.01,
                },
                "eligible reactant debit did not match admitted extent",
                "solver-reason.endpoint.reactant-debit",
            ),
            (
                EndpointRefusal::ProtonDebit {
                    requested_moles: 1.5,
                    observed_moles: 1.4,
                },
                "owned proton debit did not match admitted extent",
                "solver-reason.endpoint.proton-debit",
            ),
        ];

        for (reason, detail, key) in cases {
            let error = refused(reason, detail);
            assert_eq!(
                error.to_string(),
                format!("acidic-permanganate-peroxide could not solve this state: {detail}")
            );
            assert_eq!(error.owned_reason().unwrap().key, key);
            assert!(!matches!(
                error,
                SolveError::NotConverged { reason: None, .. }
            ));
        }
    }
}
