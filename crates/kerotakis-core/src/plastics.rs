//! BRD-023: why a thermoplastic softens and a thermoset does not.
//!
//! This is the one question about plastics a school bench is actually
//! asked, and the answer is not a property — it is a difference in what
//! the object IS.
//!
//! A thermoplastic is a great many separate chains lying in a tangle. What
//! holds them to one another is nothing stronger than the attraction
//! between neighbouring chains, so there is a temperature at which those
//! give way and the chains begin to slide past each other. The object goes
//! soft, it can be pressed into a new shape, and on cooling it sets in
//! that shape. Nothing was broken and nothing was made; the same chains
//! are there, lying differently. That is why a milk bottle can be melted
//! down and blown into another milk bottle.
//!
//! A thermoset was cured, and curing built covalent bonds BETWEEN the
//! chains until there were no separate chains left. A cured epoxy block is,
//! in the only sense that matters here, one molecule. There is nothing to
//! slide, so there is no softening point — not a very high one, none at
//! all. Heat it far enough and what gives way is not the attraction
//! between chains but the bonds inside the network, and the object chars.
//! That does not undo, which is why a burnt plug socket is thrown away and
//! a squashed yoghurt pot is not.
//!
//! # What this module claims, and what it does not
//!
//! * **Three states and no fourth.** Rigid, softened, charred. There is no
//!   viscosity here, no rate of flow, no glass transition distinct from a
//!   crystalline melt, and no degree of cure. A thermoplastic's real
//!   behaviour between its glass transition and its melt is a whole
//!   subject and this bench has one threshold.
//! * **The thresholds are curated, not computed.** They come from the
//!   recipe's reviewed `PolymerHeatResponse` row with its own citation and
//!   its own boundary, in the pending-review lane, exactly as the
//!   resistivity rows do.
//! * **Charring is named, not performed.** The bench says the network has
//!   decomposed and remembers that it did; it does not turn the block into
//!   char and volatiles, because it has no formula for either and
//!   inventing the products of a pyrolysis would be a mass claim with
//!   nothing behind it. The event is the answer; the ledger is untouched.
//! * **Softening changes nothing but the sentence.** No shape is modelled,
//!   so a softened object cannot be moulded, only described as mouldable.

use crate::material::{self, MaterialBasis, MaterialRole};
use crate::ops::{Event, PolymerState};
use crate::units::Kelvin;
use crate::vessel::Vessel;

/// The reviewed heat response of one named object, and what it means at
/// the temperature the vessel is standing at.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatResponse {
    pub material: String,
    pub recipe_id: String,
    pub state: PolymerState,
    pub temperature: Kelvin,
    /// The threshold this state turns on.
    pub threshold: Kelvin,
    /// Whether cooling takes it back.
    pub reversible: bool,
    /// True where the recipe declares no softening point at all, which is
    /// the cross-linked network's whole signature.
    pub cross_linked: bool,
}

fn response(recipe: &material::MaterialRecipe) -> Option<(f64, Option<f64>, f64)> {
    recipe.roles.iter().find_map(|role| match role {
        MaterialRole::PolymerHeatResponse {
            specific_heat_j_per_g_k,
            softens_above_k,
            chars_above_k,
            ..
        } => Some((*specific_heat_j_per_g_k, *softens_above_k, *chars_above_k)),
        _ => None,
    })
}

/// J/K contributed by named objects that resolve into no species at all.
///
/// Without this a beaker holding a two-gram block of cured resin answers
/// `heat` with "heating an empty vessel", because the vessel's heat
/// capacity is a sum over resolved portions and a thermoset has none. The
/// term is deliberately narrow: it counts only the mass an unresolved
/// portion actually carries, and only where the recipe has declared a
/// specific heat, so no existing material's energy accounting moves.
pub fn unresolved_heat_capacity(vessel: &Vessel) -> f64 {
    vessel
        .unresolved_materials
        .iter()
        .filter(|portion| portion.basis == MaterialBasis::MassFraction && portion.amount > 0.0)
        .filter_map(|portion| {
            let recipe = material::lookup_versioned(&portion.recipe_id, portion.recipe_version)?;
            let (specific_heat, _, _) = response(&recipe)?;
            Some(portion.amount * specific_heat)
        })
        .sum()
}

/// What every named plastic in this vessel is doing at its temperature.
pub fn observe(vessel: &Vessel) -> Vec<HeatResponse> {
    let now = vessel.temperature;
    material::named_objects(vessel)
        .into_iter()
        .filter_map(|recipe| {
            let (_, softens_above_k, chars_above_k) = response(&recipe)?;
            let charred = now.0 >= chars_above_k
                || vessel.charred_materials.iter().any(|id| *id == recipe.id);
            let (state, threshold, reversible) = if charred {
                (PolymerState::Charred, chars_above_k, false)
            } else {
                match softens_above_k {
                    Some(softens) if now.0 >= softens => (PolymerState::Softened, softens, true),
                    Some(softens) => (PolymerState::Rigid, softens, true),
                    // A network has no softening point, so the nearest
                    // wall it has is the one it decomposes at, and that is
                    // the number its rigid sentence should quote.
                    None => (PolymerState::Rigid, chars_above_k, true),
                }
            };
            Some(HeatResponse {
                material: recipe.name.clone(),
                recipe_id: recipe.id.clone(),
                state,
                temperature: now,
                threshold: Kelvin(threshold),
                reversible,
                cross_linked: softens_above_k.is_none(),
            })
        })
        .collect()
}

/// Observe, and write down anything that has charred so that cooling does
/// not quietly undo it.
pub fn settle(vessel: &mut Vessel) -> Vec<Event> {
    let seen = observe(vessel);
    for response in &seen {
        if response.state == PolymerState::Charred
            && !vessel
                .charred_materials
                .iter()
                .any(|id| *id == response.recipe_id)
        {
            vessel.charred_materials.push(response.recipe_id.clone());
        }
    }
    let id = vessel.id;
    seen.into_iter()
        .map(|response| Event::PolymerHeated {
            vessel: id,
            material: response.material,
            state: response.state,
            temperature: response.temperature,
            threshold: response.threshold,
            reversible: response.reversible,
            cross_linked: response.cross_linked,
        })
        .collect()
}
