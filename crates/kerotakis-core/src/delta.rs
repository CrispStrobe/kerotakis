//! StateDelta — transactional state proposals (ARCH-008).
//!
//! A `StateDelta` represents a proposed change to a vessel's state.
//! Models produce deltas rather than mutating vessels directly. The
//! orchestrator validates positivity, conservation, and compatibility
//! before committing a delta atomically.

use crate::species::{Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Moles};

/// A proposed change to one species amount in a vessel.
#[derive(Debug, Clone, PartialEq)]
pub struct MoleDelta {
    pub species: SpeciesId,
    pub phase: Phase,
    /// Positive = deposit, negative = withdraw.
    pub moles: f64,
}

/// A proposed change to a vessel's thermal state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalDelta {
    /// Set the vessel temperature to an absolute value.
    SetTemperature(Kelvin),
    /// Add or remove energy (positive = heat in).
    AddEnergy(Joules),
}

/// A complete proposed state transition from a solver or model.
///
/// A delta is the unit of atomic commit: either the entire delta is
/// applied, or none of it is. The orchestrator validates the delta
/// against the current vessel state before committing.
#[derive(Debug, Clone, Default)]
pub struct StateDelta {
    /// Species amount changes (positive = deposit, negative = withdraw).
    pub mole_changes: Vec<MoleDelta>,
    /// Thermal state change.
    pub thermal: Option<ThermalDelta>,
    /// Which model produced this delta.
    pub source: &'static str,
}

/// Reasons a delta cannot be committed.
#[derive(Debug, Clone, PartialEq)]
pub enum DeltaError {
    /// Withdrawing more than available.
    Negativity {
        species: String,
        phase: Phase,
        available: f64,
        requested: f64,
    },
    /// Element totals don't balance (for reaction deltas).
    ElementImbalance {
        element: String,
        net: f64,
    },
}

impl std::fmt::Display for DeltaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeltaError::Negativity {
                species,
                phase,
                available,
                requested,
            } => write!(
                f,
                "cannot withdraw {requested:.6e} mol of {species} ({phase:?}); only {available:.6e} available"
            ),
            DeltaError::ElementImbalance { element, net } => {
                write!(f, "element {element} has net change {net:.6e} mol")
            }
        }
    }
}

impl StateDelta {
    pub fn new(source: &'static str) -> Self {
        Self {
            mole_changes: Vec::new(),
            thermal: None,
            source,
        }
    }

    /// Add a species deposit/withdrawal to this delta.
    pub fn with_moles(mut self, species: SpeciesId, phase: Phase, moles: f64) -> Self {
        self.mole_changes.push(MoleDelta {
            species,
            phase,
            moles,
        });
        self
    }

    /// Set the thermal change.
    pub fn with_thermal(mut self, thermal: ThermalDelta) -> Self {
        self.thermal = Some(thermal);
        self
    }

    /// Validate this delta against a vessel state.
    /// Returns a list of errors (empty = valid).
    pub fn validate(&self, vessel: &crate::vessel::Vessel) -> Vec<DeltaError> {
        let mut errors = Vec::new();

        // Check positivity: withdrawals must not exceed available amounts
        for change in &self.mole_changes {
            if change.moles < 0.0 {
                let available = vessel
                    .contents
                    .iter()
                    .filter(|p| p.species == change.species && p.phase == change.phase)
                    .map(|p| p.moles.0)
                    .sum::<f64>();
                if -change.moles > available + 1e-15 {
                    errors.push(DeltaError::Negativity {
                        species: change.species.0.clone(),
                        phase: change.phase,
                        available,
                        requested: -change.moles,
                    });
                }
            }
        }

        errors
    }

    /// Apply this delta to a vessel. Call `validate()` first to check
    /// for errors; this method applies unconditionally.
    pub fn apply(&self, vessel: &mut crate::vessel::Vessel) {
        for change in &self.mole_changes {
            if change.moles > 0.0 {
                vessel.deposit(change.species.clone(), Moles(change.moles), change.phase);
            } else if change.moles < 0.0 {
                let mut remaining = -change.moles;
                for portion in &mut vessel.contents {
                    if portion.species == change.species
                        && portion.phase == change.phase
                        && remaining > 0.0
                    {
                        let take = portion.moles.0.min(remaining);
                        portion.moles.0 -= take;
                        remaining -= take;
                    }
                }
                vessel.contents.retain(|p| p.moles.0 > 1e-15);
            }
        }

        if let Some(thermal) = &self.thermal {
            match thermal {
                ThermalDelta::SetTemperature(t) => vessel.temperature = *t,
                ThermalDelta::AddEnergy(j) => {
                    let cap = vessel.heat_capacity();
                    if cap > 0.0 {
                        vessel.temperature.0 += j.0 / cap;
                    }
                }
            }
        }

        vessel.refresh_pressure();
    }

    /// Validate and apply atomically. Returns errors if validation fails
    /// (vessel is unchanged). Returns Ok(()) if applied successfully.
    pub fn commit(&self, vessel: &mut crate::vessel::Vessel) -> Result<(), Vec<DeltaError>> {
        let errors = self.validate(vessel);
        if !errors.is_empty() {
            return Err(errors);
        }
        self.apply(vessel);
        Ok(())
    }

    /// ARCH-009: Transactional commit with conservation audit and rollback.
    ///
    /// 1. Validate positivity (withdrawal limits)
    /// 2. Snapshot the vessel's conserved quantities (ConservedLedger)
    /// 3. Apply the delta
    /// 4. Re-snapshot and check element conservation
    /// 5. If conservation is violated, roll back to the snapshot and return errors
    ///
    /// `tolerance` is the maximum acceptable relative element drift.
    /// Operations that legitimately add/remove matter (Add, Evaporate) should
    /// use `commit()` instead — this is for internal transformations only.
    pub fn commit_conserved(
        &self,
        vessel: &mut crate::vessel::Vessel,
        tolerance: f64,
    ) -> Result<(), Vec<DeltaError>> {
        // Step 1: positivity
        let errors = self.validate(vessel);
        if !errors.is_empty() {
            return Err(errors);
        }

        // Step 2: snapshot before
        let snapshot = vessel.clone();
        let ledger_before = crate::ledger::ConservedLedger::from_vessel(vessel);

        // Step 3: apply
        self.apply(vessel);

        // Step 4: check conservation
        let ledger_after = crate::ledger::ConservedLedger::from_vessel(vessel);
        let violations = ledger_before.check_against(&ledger_after, tolerance, 1e-15);

        // Only element violations are conservation errors; mass drift from
        // molar-mass table precision is expected and not a rollback reason.
        let element_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.quantity.starts_with("element:"))
            .collect();

        if !element_violations.is_empty() {
            // Step 5: rollback
            *vessel = snapshot;
            return Err(element_violations
                .into_iter()
                .map(|v| DeltaError::ElementImbalance {
                    element: v.quantity.strip_prefix("element:").unwrap_or(&v.quantity).to_string(),
                    net: v.delta,
                })
                .collect());
        }

        Ok(())
    }

    /// Total moles of a given species across all changes in this delta.
    pub fn net_moles(&self, species: &SpeciesId, phase: Phase) -> f64 {
        self.mole_changes
            .iter()
            .filter(|c| c.species == *species && c.phase == phase)
            .map(|c| c.moles)
            .sum()
    }

    /// Whether this delta has no changes at all.
    pub fn is_empty(&self) -> bool {
        self.mole_changes.is_empty() && self.thermal.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Kelvin;
    use crate::vessel::{Vessel, VesselId};

    fn test_vessel() -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin(298.15);
        v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
        v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
        v
    }

    #[test]
    fn empty_delta_is_valid() {
        let v = test_vessel();
        let delta = StateDelta::new("test");
        assert!(delta.validate(&v).is_empty());
        assert!(delta.is_empty());
    }

    #[test]
    fn deposit_delta_validates_and_applies() {
        let mut v = test_vessel();
        let delta = StateDelta::new("test")
            .with_moles(SpeciesId::new("HCl"), Phase::Aqueous, 0.05);

        assert!(delta.validate(&v).is_empty());
        delta.apply(&mut v);

        let hcl = v
            .contents
            .iter()
            .find(|p| p.species.0 == "HCl")
            .unwrap();
        assert!((hcl.moles.0 - 0.05).abs() < 1e-15);
    }

    #[test]
    fn withdrawal_within_limits_succeeds() {
        let mut v = test_vessel();
        let delta = StateDelta::new("test")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.05);

        assert!(delta.validate(&v).is_empty());
        delta.commit(&mut v).unwrap();

        let nacl = v
            .contents
            .iter()
            .find(|p| p.species.0 == "NaCl")
            .unwrap();
        assert!((nacl.moles.0 - 0.05).abs() < 1e-15);
    }

    #[test]
    fn withdrawal_beyond_limits_rejected() {
        let v = test_vessel();
        let delta = StateDelta::new("test")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.2);

        let errors = delta.validate(&v);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], DeltaError::Negativity { .. }));
    }

    #[test]
    fn commit_rejects_invalid_delta() {
        let mut v = test_vessel();
        let delta = StateDelta::new("test")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.2);

        let result = delta.commit(&mut v);
        assert!(result.is_err());
        // Vessel should be unchanged
        let nacl = v
            .contents
            .iter()
            .find(|p| p.species.0 == "NaCl")
            .unwrap();
        assert!((nacl.moles.0 - 0.1).abs() < 1e-15);
    }

    #[test]
    fn thermal_delta_changes_temperature() {
        let mut v = test_vessel();
        let delta = StateDelta::new("test")
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(373.15)));

        delta.commit(&mut v).unwrap();
        assert!((v.temperature.0 - 373.15).abs() < 1e-10);
    }

    #[test]
    fn reaction_delta_with_conservation() {
        // A -> B, 0.01 mol: withdraw 0.01 A, deposit 0.01 B
        let mut v = test_vessel();
        v.deposit(SpeciesId::new("A"), Moles(0.1), Phase::Aqueous);

        let delta = StateDelta::new("reaction")
            .with_moles(SpeciesId::new("A"), Phase::Aqueous, -0.01)
            .with_moles(SpeciesId::new("B"), Phase::Aqueous, 0.01);

        delta.commit(&mut v).unwrap();

        let a_moles: f64 = v
            .contents
            .iter()
            .filter(|p| p.species.0 == "A")
            .map(|p| p.moles.0)
            .sum();
        let b_moles: f64 = v
            .contents
            .iter()
            .filter(|p| p.species.0 == "B")
            .map(|p| p.moles.0)
            .sum();
        assert!((a_moles - 0.09).abs() < 1e-15);
        assert!((b_moles - 0.01).abs() < 1e-15);
    }

    // ── ARCH-009: transactional commit/rollback tests ──────────────

    #[test]
    fn commit_conserved_accepts_balanced_reaction() {
        // NaCl dissolution: NaCl -> Na+ + Cl-
        // Elements: Na:1,Cl:1 -> Na:1 + Cl:1 — balanced
        let mut v = test_vessel();

        let delta = StateDelta::new("dissolution")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.01)
            .with_moles(SpeciesId::new("Na+"), Phase::Aqueous, 0.01)
            .with_moles(SpeciesId::new("Cl-"), Phase::Aqueous, 0.01);

        let result = delta.commit_conserved(&mut v, 1e-8);
        assert!(result.is_ok(), "balanced reaction should commit: {:?}", result);

        let nacl: f64 = v
            .contents
            .iter()
            .filter(|p| p.species.0 == "NaCl")
            .map(|p| p.moles.0)
            .sum();
        assert!((nacl - 0.09).abs() < 1e-14);
    }

    #[test]
    fn commit_conserved_rejects_unbalanced_and_rolls_back() {
        // Unbalanced: withdraw NaCl but deposit only Na+ (Cl lost)
        let mut v = test_vessel();
        let before = v.clone();

        let delta = StateDelta::new("broken")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.01)
            .with_moles(SpeciesId::new("Na+"), Phase::Aqueous, 0.01);
        // Missing Cl- deposit → Cl element conservation violation

        let result = delta.commit_conserved(&mut v, 1e-8);
        assert!(result.is_err(), "unbalanced should fail");

        // Vessel must be byte-equivalent to pre-step state
        assert_eq!(v.contents.len(), before.contents.len());
        for (a, b) in v.contents.iter().zip(before.contents.iter()) {
            assert_eq!(a.species, b.species);
            assert_eq!(a.moles.0, b.moles.0);
            assert_eq!(a.phase, b.phase);
        }
    }

    #[test]
    fn commit_conserved_rejects_negativity_without_applying() {
        let mut v = test_vessel();
        let before = v.clone();

        // Withdraw more NaCl than available
        let delta = StateDelta::new("overdraw")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.2)
            .with_moles(SpeciesId::new("Na+"), Phase::Aqueous, 0.2)
            .with_moles(SpeciesId::new("Cl-"), Phase::Aqueous, 0.2);

        let result = delta.commit_conserved(&mut v, 1e-8);
        assert!(result.is_err());

        // Vessel unchanged
        for (a, b) in v.contents.iter().zip(before.contents.iter()) {
            assert_eq!(a.moles.0, b.moles.0);
        }
    }

    #[test]
    fn rollback_preserves_temperature() {
        let mut v = test_vessel();
        let original_temp = v.temperature;

        // Unbalanced delta with thermal change
        let delta = StateDelta::new("broken")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.01)
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(500.0)));

        let result = delta.commit_conserved(&mut v, 1e-8);
        assert!(result.is_err());

        // Temperature must be restored
        assert_eq!(v.temperature, original_temp);
    }
}
