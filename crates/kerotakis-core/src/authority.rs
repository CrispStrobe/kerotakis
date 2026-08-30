//! BRD-070: the one-way boundary between scene physics and chemistry.
//!
//! A renderer may propose an action, but it never edits [`crate::Bench`] or
//! invents an [`crate::Event`].  The host converts a proposal to an
//! [`Operator`], submits that operator once, and replaces its picture from the
//! returned scene.  In particular, a fluid animation visualises a transfer
//! already accepted by the engine; it is not a second material ledger.

use serde::{Deserialize, Serialize};

use crate::{Event, Operator, VesselId};

/// A stable seed chosen when an interaction starts and persisted in replay.
/// It is for visual variation only and must never alter an operator amount.
pub type ReplaySeed = u64;

/// The typed messages scene physics is allowed to send toward the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "proposal", rename_all = "snake_case")]
pub enum SceneProposal {
    Transfer(TransferProposal),
    Gesture {
        affordance: String,
        vessel: VesselId,
        replay_seed: ReplaySeed,
    },
    Collision(CollisionProposal),
}

/// A cumulative pour target, measured against the amount present when the
/// interaction began. Cumulative targets make cancellation/interruption exact:
/// frames can be dropped or coalesced without losing or duplicating material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferProposal {
    pub from: VesselId,
    pub to: TransferDestination,
    pub cumulative_fraction: f64,
    pub replay_seed: ReplaySeed,
}

/// Destinations are explicit. The chemistry engine never infers a spill from
/// particle positions or silently discards liquid that missed a vessel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "destination", rename_all = "snake_case")]
pub enum TransferDestination {
    Vessel {
        vessel: VesselId,
    },
    /// Reserved for BRD-073's material-holding spill compartments.
    Spill(SpillDestination),
}

/// Stable spill identities. BRD-073 will make these real compartments and
/// perform the safety rerun; BRD-070 only prevents ambiguous scene-owned loss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "surface", rename_all = "snake_case")]
pub enum SpillDestination {
    Bench { zone: String },
    Tray { tray: String },
    Floor { zone: String },
}

/// A collision is evidence proposed by physics, not permission to mutate.
/// BRD-073 owns the accepted `ContainerBroken`/spill events and consequences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionProposal {
    pub vessel: VesselId,
    pub impulse_ns: f64,
    pub destination_if_broken: SpillDestination,
    pub replay_seed: ReplaySeed,
}

/// Chemistry-owned event shapes reserved for BRD-073's accepted collision
/// path. Defining them here lets scene/host implementations integrate without
/// deciding breakage or spill semantics themselves; they are not emitted until
/// the corresponding operator mutates real spill-compartment state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "physical_event", rename_all = "snake_case")]
pub enum PhysicalEventContract {
    ContainerBroken {
        vessel: VesselId,
        destination: SpillDestination,
        replay_seed: ReplaySeed,
    },
    SpillCreated {
        destination: SpillDestination,
        source: VesselId,
        /// Fraction of the source's pre-event charge reconciled into the spill.
        fraction: f64,
        replay_seed: ReplaySeed,
    },
}

/// Presentation policy has no chemistry-bearing fields. Background hosts may
/// stop painting and polling, but must finish an accepted atomic engine step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionPolicy {
    Animated,
    ReducedMotion,
    Headless,
    Background,
}

impl MotionPolicy {
    /// Whether intermediate visual frames should be produced. All policies
    /// still consume the same accepted endpoint scene and event ledger.
    pub const fn paints_intermediate_frames(self) -> bool {
        matches!(self, Self::Animated)
    }
}

/// Reconciles cumulative scene targets with sequential `Decant` operators.
/// `committed_fraction` is the authoritative fraction of the initial charge
/// already transferred, reconstructed only from accepted events.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransferReconciler {
    from: VesselId,
    to: VesselId,
    replay_seed: ReplaySeed,
    committed_fraction: f64,
    pending_fraction: Option<f64>,
}

impl<'de> Deserialize<'de> for TransferReconciler {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            from: VesselId,
            to: VesselId,
            replay_seed: ReplaySeed,
            committed_fraction: f64,
            pending_fraction: Option<f64>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if !wire.committed_fraction.is_finite()
            || !(0.0..=1.0).contains(&wire.committed_fraction)
            || wire
                .pending_fraction
                .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(serde::de::Error::custom(
                "invalid transfer reconciliation state",
            ));
        }
        Ok(Self {
            from: wire.from,
            to: wire.to,
            replay_seed: wire.replay_seed,
            committed_fraction: wire.committed_fraction,
            pending_fraction: wire.pending_fraction,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ReconcileError {
    #[error("transfer proposal does not belong to this interaction")]
    WrongInteraction,
    #[error("cumulative transfer fraction must be finite and within 0..=1")]
    BadFraction,
    #[error("spill transfer execution belongs to BRD-073")]
    SpillNotImplemented,
    #[error("accepted event ledger did not contain the expected transfer")]
    MissingReceipt,
    #[error("the previous transfer proposal is still awaiting its engine receipt")]
    AwaitingReceipt,
    #[error("accepted transfer receipt does not match the proposed operator")]
    UnexpectedReceipt,
}

impl TransferReconciler {
    pub fn new(from: VesselId, to: VesselId, replay_seed: ReplaySeed) -> Self {
        Self {
            from,
            to,
            replay_seed,
            committed_fraction: 0.0,
            pending_fraction: None,
        }
    }

    pub fn committed_fraction(&self) -> f64 {
        self.committed_fraction
    }

    /// Abandon an unaccepted proposal after the engine vetoes or fails it.
    /// This never changes committed material authority.
    pub fn cancel_pending(&mut self) -> bool {
        self.pending_fraction.take().is_some()
    }

    /// Compile a scene target into the fraction of the *remaining* liquid the
    /// existing chemistry operator must move. No state is committed here.
    pub fn propose(
        &mut self,
        proposal: &TransferProposal,
    ) -> Result<Option<Operator>, ReconcileError> {
        if self.pending_fraction.is_some() {
            return Err(ReconcileError::AwaitingReceipt);
        }
        let TransferDestination::Vessel { vessel: to } = &proposal.to else {
            return Err(ReconcileError::SpillNotImplemented);
        };
        if proposal.from != self.from || *to != self.to || proposal.replay_seed != self.replay_seed
        {
            return Err(ReconcileError::WrongInteraction);
        }
        let target = proposal.cumulative_fraction;
        if !target.is_finite() || !(0.0..=1.0).contains(&target) || target < self.committed_fraction
        {
            return Err(ReconcileError::BadFraction);
        }
        let delta = target - self.committed_fraction;
        if delta <= f64::EPSILON {
            return Ok(None);
        }
        let remaining = 1.0 - self.committed_fraction;
        let fraction = delta / remaining;
        self.pending_fraction = Some(fraction);
        Ok(Some(Operator::Decant {
            from: self.from,
            to: self.to,
            fraction,
        }))
    }

    /// Commit only the engine's receipt. A veto or failed step has no
    /// `Transferred` event and therefore cannot advance the interaction.
    pub fn reconcile(&mut self, events: &[Event]) -> Result<(), ReconcileError> {
        let expected = self
            .pending_fraction
            .ok_or(ReconcileError::UnexpectedReceipt)?;
        let mut matches = events.iter().filter_map(|event| match event {
            Event::Transferred { from, to, fraction } if *from == self.from && *to == self.to => {
                Some(*fraction)
            }
            _ => None,
        });
        let fraction = matches.next().ok_or(ReconcileError::MissingReceipt)?;
        if matches.next().is_some() {
            return Err(ReconcileError::UnexpectedReceipt);
        }
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(ReconcileError::BadFraction);
        }
        if (fraction - expected).abs() > 1e-12 * expected.abs().max(1.0) {
            return Err(ReconcileError::UnexpectedReceipt);
        }
        self.pending_fraction = None;
        self.committed_fraction += (1.0 - self.committed_fraction) * fraction;
        if (1.0 - self.committed_fraction).abs() < 1e-15 {
            self.committed_fraction = 1.0;
        }
        Ok(())
    }
}
