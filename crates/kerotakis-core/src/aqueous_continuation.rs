//! Atomic post-speciation continuation shared by native and browser hosts.
//!
//! No reaction is selected here. A caller supplies a separately admitted,
//! conservative material transformation and a fresh aqueous recharacterizer.
//! Unknown reaction heat makes the remaining aqueous thermal balance of this
//! operator unavailable; it is never reconstructed from the original snapshot.

use crate::delta::StateDelta;
use crate::ledger::ConservedLedger;
use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
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
        {
            return Err(refused(
                ContinuationRefusal::StateOrMaterial,
                "adapter did not supply a finite complete aqueous answer",
            ));
        }
        if !characterized(output) {
            return Err(refused(
                ContinuationRefusal::AnswerIncomplete,
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
    /// One ordinary aqueous pass for optional endpoint admission. A concrete
    /// adapter may retain an honest partial result without issuing a receipt.
    /// The strict continuation readback below still requires a complete answer.
    fn characterize_optional_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<(Vec<Event>, Option<CurrentAqueousAnswer>), SolveError> {
        self.characterize_current(vessel)
            .map(|(events, answer)| (events, Some(answer)))
    }

    fn characterize_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError>;
}

/// Obtain a current answer through an explicitly opted-in erased/wrapped
/// adapter. This characterizes the aqueous state only; it does not run other
/// wrapper chemistry or install a post-speciation reaction route.
pub fn characterize_via(
    aqueous: &mut dyn crate::Equilibrator,
    vessel: &mut Vessel,
) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
    aqueous
        .trusted_aqueous_readback()
        .ok_or_else(|| {
            refused(
                ContinuationRefusal::AdapterUnavailable,
                "equilibrator has no trusted aqueous readback adapter",
            )
        })?
        .characterize_current(vessel)
}

/// Ask once without upgrading a concrete adapter's partial answer to complete.
pub fn characterize_optional_via(
    aqueous: &mut dyn crate::Equilibrator,
    vessel: &mut Vessel,
) -> Result<(Vec<Event>, Option<CurrentAqueousAnswer>), SolveError> {
    aqueous
        .trusted_aqueous_readback()
        .ok_or_else(|| {
            refused(
                ContinuationRefusal::AdapterUnavailable,
                "equilibrator has no trusted aqueous readback adapter",
            )
        })?
        .characterize_optional_current(vessel)
}

/// Shared host bridge for the atomic primitive. A wrapper that does not
/// explicitly expose a trusted adapter refuses before invoking `transform`.
pub fn continue_via(
    vessel: &mut Vessel,
    initial_answer: &CurrentAqueousAnswer,
    aqueous: &mut dyn crate::Equilibrator,
    transform: impl FnOnce(&mut Vessel) -> Result<Vec<Event>, SolveError>,
) -> Result<Vec<Event>, SolveError> {
    let adapter = aqueous.trusted_aqueous_readback().ok_or_else(|| {
        refused(
            ContinuationRefusal::AdapterUnavailable,
            "equilibrator has no trusted aqueous readback adapter",
        )
    })?;
    continue_without_reaction_heat(vessel, initial_answer, adapter, transform)
}

enum ContinuationRefusal<'a> {
    AdapterUnavailable,
    AnswerIncomplete,
    VesselIdentity,
    BoundaryConfiguration,
    StateOrMaterial,
    ChargeConservation {
        before: f64,
        after: f64,
    },
    SealedGasCrossing {
        species: &'a str,
    },
    GasWithoutHeadspace {
        species: &'a str,
    },
    GasAmountInvalid {
        species: &'a str,
    },
    GasAmountMismatch {
        species: &'a str,
        reported: f64,
        retained: f64,
    },
    StaleAnswer,
    UnsupportedBoundary,
    MissingSnapshot,
    SnapshotChanged,
    ThermalOwnershipChanged,
    ThermalMutation,
    ReceiptQuestionMismatch,
    ReceiptStateMismatch,
}

fn diagnostic_species_slot(key: &str) -> Slot {
    if crate::species::lookup_key(key).is_some() {
        Slot::term("species", key)
    } else {
        Slot::text(key)
    }
}

impl ContinuationRefusal<'_> {
    fn phrase(&self) -> Phrase {
        match self {
            Self::AdapterUnavailable => Phrase::bare(
                "solver-reason.continuation.adapter-unavailable",
                "This solver wrapper does not expose a trusted aqueous readback adapter.",
            ),
            Self::AnswerIncomplete => Phrase::bare(
                "solver-reason.continuation.answer-incomplete",
                "The trusted adapter did not supply a finite complete aqueous answer.",
            ),
            Self::VesselIdentity => Phrase::bare(
                "solver-reason.continuation.vessel-identity",
                "The continuation changed the vessel identity or label.",
            ),
            Self::BoundaryConfiguration => Phrase::bare(
                "solver-reason.continuation.boundary-configuration",
                "The continuation changed a thermal input or gas boundary state.",
            ),
            Self::StateOrMaterial => Phrase::bare(
                "solver-reason.continuation.state-material",
                "The continuation failed a finite-state or material-conservation check.",
            ),
            Self::ChargeConservation { before, after } => Phrase::new(
                "solver-reason.continuation.charge-conservation",
                "Owned charge changed from {before} to {after} mol-equivalents during continuation.",
                vec![
                    ("before".into(), Slot::number(before.to_string())),
                    ("after".into(), Slot::number(after.to_string())),
                ],
            ),
            Self::SealedGasCrossing { species } => Phrase::new(
                "solver-reason.continuation.sealed-gas-crossing",
                "A fixed sealed boundary cannot transfer {species} to or from the external gas space.",
                vec![("species".into(), diagnostic_species_slot(species))],
            ),
            Self::GasWithoutHeadspace { species } => Phrase::new(
                "solver-reason.continuation.gas-without-headspace",
                "The continuation reported contained {species}, but this vessel has no owned headspace.",
                vec![("species".into(), diagnostic_species_slot(species))],
            ),
            Self::GasAmountInvalid { species } => Phrase::new(
                "solver-reason.continuation.gas-amount-invalid",
                "The continuation reported a non-finite amount of contained {species}.",
                vec![("species".into(), diagnostic_species_slot(species))],
            ),
            Self::GasAmountMismatch {
                species,
                reported,
                retained,
            } => Phrase::new(
                "solver-reason.continuation.gas-amount-mismatch",
                "The continuation reported {reported} mol of contained {species}, but {retained} mol was newly retained.",
                vec![
                    ("species".into(), diagnostic_species_slot(species)),
                    ("reported".into(), Slot::number(reported.to_string())),
                    ("retained".into(), Slot::number(retained.to_string())),
                ],
            ),
            Self::StaleAnswer => Phrase::bare(
                "solver-reason.continuation.stale-answer",
                "The continuation requires a fresh complete aqueous answer for the current vessel state.",
            ),
            Self::UnsupportedBoundary => Phrase::bare(
                "solver-reason.continuation.unsupported-boundary",
                "This continuation supports only open or fixed sealed boundaries.",
            ),
            Self::MissingSnapshot => Phrase::bare(
                "solver-reason.continuation.missing-snapshot",
                "The continuation requires an active operator snapshot.",
            ),
            Self::SnapshotChanged => Phrase::bare(
                "solver-reason.continuation.snapshot-changed",
                "The transformation replaced the active operator snapshot.",
            ),
            Self::ThermalOwnershipChanged => Phrase::bare(
                "solver-reason.continuation.thermal-ownership-changed",
                "The transformation changed ownership of the aqueous heat calculation.",
            ),
            Self::ThermalMutation => Phrase::bare(
                "solver-reason.continuation.thermal-mutation",
                "A transformation with unreviewed heat changed the vessel thermal state.",
            ),
            Self::ReceiptQuestionMismatch => Phrase::bare(
                "solver-reason.continuation.receipt-question-mismatch",
                "The aqueous answer receipt belongs to a different input question.",
            ),
            Self::ReceiptStateMismatch => Phrase::bare(
                "solver-reason.continuation.receipt-state-mismatch",
                "The aqueous answer receipt does not attest to the returned vessel state.",
            ),
        }
    }
}

fn refused(reason: ContinuationRefusal<'_>, detail: &str) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(reason.phrase())),
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
        return Err(refused(
            ContinuationRefusal::VesselIdentity,
            "continuation changed vessel identity",
        ));
    }
    if before.thermal_mode != after.thermal_mode
        || before.heat_input != after.heat_input
        || before.headspace != after.headspace
        || before.pending_co2_transfer_mol != after.pending_co2_transfer_mol
    {
        return Err(refused(
            ContinuationRefusal::BoundaryConfiguration,
            "continuation changed thermal input or boundary configuration",
        ));
    }
    if !StateDelta::validate_state(after).is_empty()
        || !StateDelta::validate_conservation(before, after, events, 1e-7).is_empty()
    {
        return Err(refused(
            ContinuationRefusal::StateOrMaterial,
            "continuation failed finite-state or material conservation checks",
        ));
    }
    let initial = ConservedLedger::from_vessel(before).charge;
    let final_charge = ConservedLedger::from_vessel(after).charge;
    if (initial - final_charge).abs() > 1e-9 + 1e-7 * initial.abs().max(final_charge.abs()) {
        return Err(refused(
            ContinuationRefusal::ChargeConservation {
                before: initial,
                after: final_charge,
            },
            "continuation failed charge conservation",
        ));
    }
    // GasContained is narration rather than an external element transfer.
    // Validate its aggregate against actual new owned gas, so duplicated or
    // invented events cannot inflate an otherwise conservative transaction.
    let mut contained: BTreeMap<String, f64> = BTreeMap::new();
    for event in events {
        if matches!(before.headspace, Headspace::Sealed { .. }) {
            let species = match event {
                Event::GasEvolved { species, .. } | Event::GasAbsorbed { species, .. } => {
                    Some(species.0.as_str())
                }
                _ => None,
            };
            if let Some(species) = species {
                return Err(refused(
                    ContinuationRefusal::SealedGasCrossing { species },
                    "fixed sealed continuation cannot transfer external gas",
                ));
            }
        }
        if let Event::GasContained { species, moles, .. } = event {
            if !after.owns_headspace_gas() {
                return Err(refused(
                    ContinuationRefusal::GasWithoutHeadspace {
                        species: species.0.as_str(),
                    },
                    "contained gas requires an owned headspace",
                ));
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
        if !reported.is_finite() {
            return Err(refused(
                ContinuationRefusal::GasAmountInvalid { species: &species },
                "contained gas events must match newly retained gas",
            ));
        }
        if (reported - growth).abs() > tolerance {
            return Err(refused(
                ContinuationRefusal::GasAmountMismatch {
                    species: &species,
                    reported,
                    retained: growth,
                },
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
    if !initial_answer.is_current(vessel) {
        return Err(refused(
            ContinuationRefusal::StaleAnswer,
            "continuation requires a fresh complete aqueous answer",
        ));
    }
    if !characterized(vessel) {
        return Err(refused(
            ContinuationRefusal::AnswerIncomplete,
            "continuation requires a fresh complete aqueous answer",
        ));
    }
    if !matches!(vessel.headspace, Headspace::Open | Headspace::Sealed { .. }) {
        return Err(refused(
            ContinuationRefusal::UnsupportedBoundary,
            "continuation currently supports open or fixed sealed boundaries",
        ));
    }
    let original_start = vessel.step_start.clone().ok_or_else(|| {
        refused(
            ContinuationRefusal::MissingSnapshot,
            "continuation requires an active operator snapshot",
        )
    })?;
    let mut candidate = vessel.clone();
    let mut events = transform(&mut candidate)?;
    if candidate.step_start.as_ref() != Some(&original_start) {
        return Err(refused(
            ContinuationRefusal::SnapshotChanged,
            "transformation replaced the operator snapshot",
        ));
    }
    if candidate.aqueous_heat != vessel.aqueous_heat {
        return Err(refused(
            ContinuationRefusal::ThermalOwnershipChanged,
            "transformation replaced the operator snapshot",
        ));
    }
    if candidate.temperature != vessel.temperature
        || candidate.excess_enthalpy_j != vessel.excess_enthalpy_j
    {
        return Err(refused(
            ContinuationRefusal::ThermalMutation,
            "unknown-heat transformation changed thermal state",
        ));
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
    if receipt.question != question_key(&before_readback) {
        return Err(refused(
            ContinuationRefusal::ReceiptQuestionMismatch,
            "aqueous answer receipt does not match the current question and state",
        ));
    }
    if !receipt.is_current(&candidate) {
        return Err(refused(
            ContinuationRefusal::ReceiptStateMismatch,
            "aqueous answer receipt does not match the current question and state",
        ));
    }
    if !characterized(&candidate) {
        return Err(refused(
            ContinuationRefusal::AnswerIncomplete,
            "continuation received no fresh complete aqueous answer or lost thermal ownership",
        ));
    }
    if candidate.step_start.as_ref() != Some(&original_start) {
        return Err(refused(
            ContinuationRefusal::SnapshotChanged,
            "continuation received no fresh complete aqueous answer or lost thermal ownership",
        ));
    }
    if candidate.aqueous_heat != AqueousHeatOwnership::UnmodeledContinuation {
        return Err(refused(
            ContinuationRefusal::ThermalOwnershipChanged,
            "continuation received no fresh complete aqueous answer or lost thermal ownership",
        ));
    }
    if candidate.temperature != before_readback.temperature
        || candidate.excess_enthalpy_j != before_readback.excess_enthalpy_j
    {
        return Err(refused(
            ContinuationRefusal::ThermalMutation,
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

#[cfg(test)]
mod refusal_tests {
    use super::{refused, ContinuationRefusal};

    #[test]
    fn continuation_gas_refusal_keeps_reason_slots_and_raw_detail() {
        let detail = "contained gas events must match newly retained gas";
        let error = refused(
            ContinuationRefusal::GasAmountMismatch {
                species: "O2",
                reported: 0.25,
                retained: 0.2,
            },
            detail,
        );

        assert_eq!(
            error.to_string(),
            format!("aqueous-continuation could not solve this state: {detail}")
        );
        let reason = error.owned_reason().unwrap();
        assert_eq!(reason.key, "solver-reason.continuation.gas-amount-mismatch");
        assert_eq!(
            reason
                .slots
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["species", "reported", "retained"]
        );
    }
}
