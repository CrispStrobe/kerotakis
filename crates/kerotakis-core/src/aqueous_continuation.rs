//! Atomic post-speciation continuation shared by native and browser hosts.
//!
//! No reaction is selected here. A caller supplies a separately admitted,
//! conservative material transformation and a fresh aqueous recharacterizer.
//! Unknown reaction heat makes the remaining aqueous thermal balance of this
//! operator unavailable; it is never reconstructed from the original snapshot.

use crate::delta::StateDelta;
use crate::ledger::ConservedLedger;
use crate::ops::{Event, NotModelledCause};
use crate::phrase::Phrase;
use crate::vessel::{AqueousHeatOwnership, Headspace, SolutionScope, Vessel};
use crate::{Phase, SolveError};
use std::collections::BTreeMap;

/// In-process binding emitted by a trusted concrete aqueous adapter after a
/// computed/cache-backed answer. It proves state association, not chemistry.
#[derive(Clone, Debug)]
pub struct CurrentAqueousAnswer {
    question: String,
    answer: String,
}

fn question_key(vessel: &Vessel) -> String {
    let mut question = vessel.clone();
    question.solution = None;
    question.resolved.invalidate();
    question.free_proton = 0.0;
    question.free_hydroxide = 0.0;
    format!("{question:?}")
}

impl CurrentAqueousAnswer {
    /// Trusted-adapter boundary: call only after an actual answer for `input`.
    /// Generic Equilibrator output and Some(solution) are not this contract.
    /// Implementers (including test doubles) explicitly own that assertion.
    pub fn attest_computed(input: &Vessel, output: &Vessel) -> Result<Self, SolveError> {
        if !StateDelta::validate_state(input).is_empty()
            || !StateDelta::validate_state(output).is_empty()
            || !characterized(output)
        {
            return Err(refused(
                "adapter did not supply a finite complete aqueous answer",
            ));
        }
        Ok(Self {
            question: question_key(input),
            answer: format!("{output:?}"),
        })
    }

    fn is_current(&self, vessel: &Vessel) -> bool {
        self.answer == format!("{vessel:?}")
    }
}

/// A trusted concrete adapter must actually answer the supplied current input,
/// including its native/hook/cache binding. An arbitrary Equilibrator that
/// merely restores an old SolutionInfo is insufficient. Receipt construction
/// is an explicit trusted assertion; it is not an engine correctness proof.
pub trait TrustedAqueousReadback {
    fn characterize_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError>;
}

fn refused(detail: &str) -> SolveError {
    SolveError::NotConverged {
        solver: "aqueous-continuation".into(),
        detail: detail.into(),
    }
}

/// Whether an aqueous solver still owns a whole-step heat calculation.
/// This policy is shared by every subsequent pass, including phase coupling.
pub fn whole_step_heat_available(vessel: &Vessel) -> bool {
    vessel.aqueous_heat == AqueousHeatOwnership::WholeStep
}

fn characterized(vessel: &Vessel) -> bool {
    vessel.solution.as_ref().is_some_and(|solution| {
        solution.scope == SolutionScope::Complete
            && solution.ph.is_finite()
            && solution.ionic_strength.is_finite()
            && solution.ionic_strength >= 0.0
    })
}

fn validate_change(before: &Vessel, after: &Vessel, events: &[Event]) -> Result<(), SolveError> {
    if before.id != after.id || before.label != after.label {
        return Err(refused("continuation changed vessel identity"));
    }
    if before.thermal_mode != after.thermal_mode
        || before.heat_input != after.heat_input
        || before.headspace != after.headspace
        || before.ignition_trial != after.ignition_trial
        || before.ignition_feed_temperature != after.ignition_feed_temperature
        || before.pending_co2_transfer_mol != after.pending_co2_transfer_mol
    {
        return Err(refused(
            "continuation changed thermal input or boundary configuration",
        ));
    }
    if !StateDelta::validate_state(after).is_empty()
        || !StateDelta::validate_conservation(before, after, events, 1e-7).is_empty()
    {
        return Err(refused(
            "continuation failed finite-state or material conservation checks",
        ));
    }
    let initial = ConservedLedger::from_vessel(before).charge;
    let final_charge = ConservedLedger::from_vessel(after).charge;
    if (initial - final_charge).abs() > 1e-9 + 1e-7 * initial.abs().max(final_charge.abs()) {
        return Err(refused("continuation failed charge conservation"));
    }
    // GasContained is narration rather than an external element transfer.
    // Validate its aggregate against actual new owned gas, so duplicated or
    // invented events cannot inflate an otherwise conservative transaction.
    let mut contained: BTreeMap<String, f64> = BTreeMap::new();
    for event in events {
        if matches!(before.headspace, Headspace::Sealed { .. })
            && matches!(event, Event::GasEvolved { .. } | Event::GasAbsorbed { .. })
        {
            return Err(refused(
                "fixed sealed continuation cannot transfer external gas",
            ));
        }
        if let Event::GasContained { species, moles, .. } = event {
            if !after.owns_headspace_gas() {
                return Err(refused("contained gas requires an owned headspace"));
            }
            *contained.entry(species.0.clone()).or_default() += moles.0;
        }
    }
    for vessel in [before, after] {
        for portion in &vessel.contents {
            if portion.phase == Phase::Gas {
                contained.entry(portion.species.0.clone()).or_default();
            }
        }
    }
    for (species, reported) in contained {
        let gas = |v: &Vessel| {
            v.contents
                .iter()
                .filter(|p| p.species.0 == species && p.phase == Phase::Gas)
                .map(|p| p.moles.0)
                .sum::<f64>()
        };
        let initial = gas(before);
        let final_gas = gas(after);
        let growth = (final_gas - initial).max(0.0);
        let rounding = 64.0 * f64::EPSILON * initial.abs().max(final_gas.abs());
        let tolerance = reported.abs().max(growth) * 1e-7 + rounding;
        if !reported.is_finite() || (reported - growth).abs() > tolerance {
            return Err(refused(
                "contained gas events must match newly retained gas",
            ));
        }
    }
    Ok(())
}

/// Commit an unknown-heat transformation only with a valid fresh readback.
///
/// Call immediately after an aqueous answer in the shared host continuation
/// path. `transform` must perform its own chemistry/domain admission and emit
/// all material/gas events. It may not replace the operator's `StepStart` or
/// claim an applied heat by changing temperature/excess enthalpy. The original
/// material/gas snapshot remains intact; only the vessel's thermal ownership
/// changes. The caller books returned gas events through its ordinary ledger.
///
/// Both transformation and readback occur on a clone. A solver error, missing
/// cache answer, stale/invalid solution, or failed conservation commits neither
/// matter nor events. The original valid aqueous state remains available.
///
/// This deliberately conservative policy withholds *all remaining aqueous
/// reaction heat for this operator*, including associated recharacterization.
/// It does not claim that omitted heat is physically zero. Bench operator
/// entry/exit restores normal heat ownership. Embedders that
/// call solvers directly own the equivalent operator boundary explicitly.
pub fn continue_without_reaction_heat(
    vessel: &mut Vessel,
    initial_answer: &CurrentAqueousAnswer,
    aqueous: &mut dyn TrustedAqueousReadback,
    transform: impl FnOnce(&mut Vessel) -> Result<Vec<Event>, SolveError>,
) -> Result<Vec<Event>, SolveError> {
    if !initial_answer.is_current(vessel) || !characterized(vessel) {
        return Err(refused(
            "continuation requires a fresh complete aqueous answer",
        ));
    }
    if !matches!(vessel.headspace, Headspace::Open | Headspace::Sealed { .. }) {
        return Err(refused(
            "continuation currently supports open or fixed sealed boundaries",
        ));
    }
    let original_start = vessel
        .step_start
        .clone()
        .ok_or_else(|| refused("continuation requires an active operator snapshot"))?;
    let mut candidate = vessel.clone();
    let mut events = transform(&mut candidate)?;
    if candidate.step_start.as_ref() != Some(&original_start)
        || candidate.aqueous_heat != vessel.aqueous_heat
    {
        return Err(refused("transformation replaced the operator snapshot"));
    }
    if candidate.temperature != vessel.temperature
        || candidate.excess_enthalpy_j != vessel.excess_enthalpy_j
    {
        return Err(refused("unknown-heat transformation changed thermal state"));
    }
    validate_change(vessel, &candidate, &events)?;
    candidate.aqueous_heat = AqueousHeatOwnership::UnmodeledContinuation;
    candidate.solute_charge = crate::displacement::solute_charge(&candidate);
    candidate.solution = None;
    candidate.resolved.invalidate();
    candidate.free_proton = 0.0;
    candidate.free_hydroxide = 0.0;
    let before_readback = candidate.clone();
    let (readback, receipt) = aqueous.characterize_current(&mut candidate)?;
    if receipt.question != question_key(&before_readback) || !receipt.is_current(&candidate) {
        return Err(refused(
            "aqueous answer receipt does not match the current question and state",
        ));
    }
    if !characterized(&candidate)
        || candidate.step_start.as_ref() != Some(&original_start)
        || candidate.aqueous_heat != AqueousHeatOwnership::UnmodeledContinuation
    {
        return Err(refused(
            "continuation received no fresh complete aqueous answer or lost thermal ownership",
        ));
    }
    if candidate.temperature != before_readback.temperature
        || candidate.excess_enthalpy_j != before_readback.excess_enthalpy_j
    {
        return Err(refused(
            "unknown-heat recharacterization changed thermal state",
        ));
    }
    validate_change(&before_readback, &candidate, &readback)?;
    events.extend(readback);
    events.push(Event::not_modeled(
        vessel.id,
        NotModelledCause::NoReviewedDatum,
        Phrase::bare(
            "not-modeled.aqueous-continuation-heat",
            "this post-speciation transformation has no reviewed reaction heat. Its associated aqueous recharacterization and all remaining aqueous heat calculations in this operator are withheld; the reported temperature is not a complete thermal prediction for this transformation. Material and gas accounting remain explicit",
        ),
    ));
    *vessel = candidate;
    Ok(events)
}
