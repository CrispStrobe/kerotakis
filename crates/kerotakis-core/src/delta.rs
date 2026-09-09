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

/// Which conserved inventory on a named electrode changes.
#[derive(Debug, Clone, PartialEq)]
pub enum ElectrodeInventory {
    Substrate,
    Deposit {
        species: SpeciesId,
        growth: Option<crate::compartment::DepositGrowthModel>,
        effect: Option<crate::electrochemistry::PassivationEffect>,
    },
}

impl ElectrodeInventory {
    fn same_reservoir(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Substrate, Self::Substrate) => true,
            (Self::Deposit { species: left, .. }, Self::Deposit { species: right, .. }) => {
                left == right
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElectrodeMoleDelta {
    pub electrode: String,
    pub inventory: ElectrodeInventory,
    /// Positive adds material; negative consumes it.
    pub moles: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElectrodePotentialDelta {
    pub electrode: String,
    pub potential_v: f64,
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
    /// Matter transferred to or from explicit electrode inventories.
    pub electrode_changes: Vec<ElectrodeMoleDelta>,
    /// Persistent interfacial electrical state after a transient solve.
    pub electrode_potential_changes: Vec<ElectrodePotentialDelta>,
    /// Thermal state change.
    pub thermal: Option<ThermalDelta>,
    /// Which model produced this delta.
    pub source: &'static str,
}

/// One coupled state proposal after a single, uniform inventory limit.
///
/// The fraction applies to every material and relative-energy term in the
/// original proposal. Keeping it explicit lets callers report depletion and
/// choose a smaller clock step without reconstructing the limit from rounded
/// mole changes.
#[derive(Debug, Clone)]
pub struct InventoryLimitedDelta {
    pub delta: StateDelta,
    pub accepted_fraction: f64,
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
    UnknownElectrode {
        electrode: String,
    },
    InvalidElectrodeDelta {
        electrode: String,
        reason: String,
    },
    ElectrodeNegativity {
        electrode: String,
        species: String,
        available: f64,
        requested: f64,
    },
    UnscalableThermalDelta,
    UnscalableElectricalDelta,
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
            DeltaError::UnknownElectrode { electrode } => {
                write!(f, "electrode {electrode} does not exist")
            }
            DeltaError::InvalidElectrodeDelta { electrode, reason } => {
                write!(f, "cannot change electrode {electrode}: {reason}")
            }
            DeltaError::ElectrodeNegativity {
                electrode,
                species,
                available,
                requested,
            } => write!(
                f,
                "cannot withdraw {requested:.6e} mol of {species} from electrode {electrode}; only {available:.6e} available"
            ),
            DeltaError::UnscalableThermalDelta => {
                write!(f, "an absolute-temperature delta cannot be inventory-scaled")
            }
            DeltaError::UnscalableElectricalDelta => {
                write!(f, "an electrode-potential delta cannot be inventory-scaled")
            }
        }
    }
}

impl StateDelta {
    pub fn new(source: &'static str) -> Self {
        Self {
            mole_changes: Vec::new(),
            electrode_changes: Vec::new(),
            electrode_potential_changes: Vec::new(),
            thermal: None,
            source,
        }
    }

    pub fn with_electrode_moles(
        mut self,
        electrode: impl Into<String>,
        inventory: ElectrodeInventory,
        moles: f64,
    ) -> Self {
        self.electrode_changes.push(ElectrodeMoleDelta {
            electrode: electrode.into(),
            inventory,
            moles,
        });
        self
    }

    pub fn with_electrode_potential(
        mut self,
        electrode: impl Into<String>,
        potential_v: f64,
    ) -> Self {
        self.electrode_potential_changes
            .push(ElectrodePotentialDelta {
                electrode: electrode.into(),
                potential_v,
            });
        self
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

        // Check positivity cumulatively: two individually valid withdrawals
        // must not overdraw the same reservoir when committed together.
        let mut checked_bulk = std::collections::BTreeSet::new();
        for change in &self.mole_changes {
            if !change.moles.is_finite() {
                errors.push(DeltaError::Negativity {
                    species: change.species.0.clone(),
                    phase: change.phase,
                    available: 0.0,
                    requested: f64::NAN,
                });
                continue;
            }
            let key = (change.species.clone(), change.phase);
            if !checked_bulk.insert(key) {
                continue;
            }
            let cumulative_change: f64 = self
                .mole_changes
                .iter()
                .filter(|candidate| {
                    candidate.species == change.species && candidate.phase == change.phase
                })
                .map(|candidate| candidate.moles)
                .sum();
            let available = vessel
                .contents
                .iter()
                .filter(|portion| {
                    portion.species == change.species && portion.phase == change.phase
                })
                .map(|portion| portion.moles.0)
                .sum::<f64>();
            if cumulative_change < 0.0 && -cumulative_change > available + 1e-15 {
                errors.push(DeltaError::Negativity {
                    species: change.species.0.clone(),
                    phase: change.phase,
                    available,
                    requested: -cumulative_change,
                });
            }
        }

        let mut checked = std::collections::BTreeSet::new();
        for change in &self.electrode_changes {
            let matching_count = vessel
                .electrodes
                .iter()
                .filter(|electrode| electrode.label == change.electrode)
                .count();
            if matching_count == 0 {
                errors.push(DeltaError::UnknownElectrode {
                    electrode: change.electrode.clone(),
                });
                continue;
            }
            if matching_count > 1 {
                errors.push(DeltaError::InvalidElectrodeDelta {
                    electrode: change.electrode.clone(),
                    reason: "label is ambiguous".into(),
                });
                continue;
            }
            let electrode = vessel
                .electrodes
                .iter()
                .find(|electrode| electrode.label == change.electrode)
                .expect("counted exactly one electrode above");
            if !change.moles.is_finite() {
                errors.push(DeltaError::InvalidElectrodeDelta {
                    electrode: change.electrode.clone(),
                    reason: "mole change must be finite".into(),
                });
                continue;
            }
            let (species, available, inventory_key) = match &change.inventory {
                ElectrodeInventory::Substrate => {
                    let Some(available) = electrode.substrate_moles else {
                        errors.push(DeltaError::InvalidElectrodeDelta {
                            electrode: change.electrode.clone(),
                            reason: "external apparatus has no finite substrate inventory".into(),
                        });
                        continue;
                    };
                    (
                        electrode.material.clone(),
                        available,
                        "substrate".to_owned(),
                    )
                }
                ElectrodeInventory::Deposit {
                    species,
                    growth,
                    effect,
                } => {
                    let existing = electrode
                        .deposits
                        .iter()
                        .find(|deposit| deposit.species == species.0);
                    if existing
                        .and_then(|deposit| deposit.effect)
                        .zip(*effect)
                        .is_some_and(|(existing, proposed)| existing != proposed)
                    {
                        errors.push(DeltaError::InvalidElectrodeDelta {
                            electrode: change.electrode.clone(),
                            reason: format!("deposit {} changes kinetic effect", species.0),
                        });
                    }
                    if growth.is_some_and(|model| {
                        model
                            .geometry(
                                (existing.map_or(0.0, |deposit| deposit.moles) + change.moles)
                                    .max(0.0),
                                electrode.area_m2,
                            )
                            .is_err()
                    }) {
                        errors.push(DeltaError::InvalidElectrodeDelta {
                            electrode: change.electrode.clone(),
                            reason: format!("deposit {} has invalid growth geometry", species.0),
                        });
                    }
                    (
                        species.0.clone(),
                        electrode
                            .deposits
                            .iter()
                            .filter(|deposit| deposit.species == species.0)
                            .map(|deposit| deposit.moles)
                            .sum(),
                        format!("deposit:{}", species.0),
                    )
                }
            };
            let key = (change.electrode.clone(), inventory_key);
            if !checked.insert(key) {
                continue;
            }
            let cumulative_change: f64 = self
                .electrode_changes
                .iter()
                .filter(|candidate| {
                    candidate.electrode == change.electrode
                        && candidate.inventory.same_reservoir(&change.inventory)
                })
                .map(|candidate| candidate.moles)
                .sum();
            if cumulative_change < 0.0 && -cumulative_change > available + 1e-15 {
                errors.push(DeltaError::ElectrodeNegativity {
                    electrode: change.electrode.clone(),
                    species,
                    available,
                    requested: -cumulative_change,
                });
            }
        }

        let mut potentials = std::collections::BTreeSet::new();
        for change in &self.electrode_potential_changes {
            let matching_count = vessel
                .electrodes
                .iter()
                .filter(|electrode| electrode.label == change.electrode)
                .count();
            if matching_count != 1 || !change.potential_v.is_finite() {
                errors.push(DeltaError::InvalidElectrodeDelta {
                    electrode: change.electrode.clone(),
                    reason: "potential update requires one named electrode and a finite value"
                        .into(),
                });
            } else if !potentials.insert(change.electrode.clone()) {
                errors.push(DeltaError::InvalidElectrodeDelta {
                    electrode: change.electrode.clone(),
                    reason: "potential may be set only once per atomic delta".into(),
                });
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

        for change in &self.electrode_changes {
            let Some(electrode) = vessel
                .electrodes
                .iter_mut()
                .find(|electrode| electrode.label == change.electrode)
            else {
                continue;
            };
            match &change.inventory {
                ElectrodeInventory::Substrate => {
                    if let Some(moles) = &mut electrode.substrate_moles {
                        *moles += change.moles;
                    }
                }
                ElectrodeInventory::Deposit {
                    species,
                    growth,
                    effect,
                } => {
                    let area_m2 = electrode.area_m2;
                    if let Some(deposit) = electrode
                        .deposits
                        .iter_mut()
                        .find(|deposit| deposit.species == species.0)
                    {
                        deposit.moles += change.moles;
                        if deposit.effect.is_none() {
                            deposit.effect = *effect;
                        }
                        if let Some(model) = growth {
                            if let Ok((thickness, coverage)) =
                                model.geometry(deposit.moles.max(0.0), area_m2)
                            {
                                deposit.thickness_m = Some(thickness);
                                deposit.coverage_fraction = Some(coverage);
                            }
                        }
                    } else if change.moles > 0.0 {
                        let geometry =
                            growth.and_then(|model| model.geometry(change.moles, area_m2).ok());
                        electrode
                            .deposits
                            .push(crate::compartment::ElectrodeDeposit {
                                species: species.0.clone(),
                                moles: change.moles,
                                thickness_m: geometry.map(|value| value.0),
                                coverage_fraction: geometry.map(|value| value.1),
                                effect: *effect,
                                electrical_resistivity_ohm_m: None,
                            });
                    }
                }
            }
            electrode.deposits.retain(|deposit| deposit.moles > 1e-15);
        }

        for change in &self.electrode_potential_changes {
            if let Some(electrode) = vessel
                .electrodes
                .iter_mut()
                .find(|electrode| electrode.label == change.electrode)
            {
                electrode.interfacial_potential_v = Some(change.potential_v);
            }
        }

        if let Some(thermal) = &self.thermal {
            match thermal {
                ThermalDelta::SetTemperature(t) => vessel.temperature = *t,
                ThermalDelta::AddEnergy(j) => {
                    if vessel.heat_capacity() > 0.0 {
                        vessel.temperature.0 = vessel.temperature_after(j.0);
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

    /// Scale every coupled material change by the single factor required to
    /// avoid the first depleted reservoir. This preserves stoichiometry and
    /// competition fractions; independently clamping terms would not.
    pub fn limited_to_inventory(
        &self,
        vessel: &crate::vessel::Vessel,
    ) -> Result<Self, Vec<DeltaError>> {
        self.inventory_limited(vessel).map(|limited| limited.delta)
    }

    /// Return both the uniformly limited proposal and its accepted fraction.
    pub fn inventory_limited(
        &self,
        vessel: &crate::vessel::Vessel,
    ) -> Result<InventoryLimitedDelta, Vec<DeltaError>> {
        let errors = self.validate(vessel);
        let mut scale = 1.0_f64;
        let mut fatal = Vec::new();
        for error in errors {
            match error {
                DeltaError::Negativity {
                    available,
                    requested,
                    ..
                }
                | DeltaError::ElectrodeNegativity {
                    available,
                    requested,
                    ..
                } if requested.is_finite() && requested > 0.0 && available.is_finite() => {
                    scale = scale.min((available / requested).clamp(0.0, 1.0));
                }
                other => fatal.push(other),
            }
        }
        if !fatal.is_empty() {
            return Err(fatal);
        }
        if scale < 1.0 && matches!(self.thermal, Some(ThermalDelta::SetTemperature(_))) {
            return Err(vec![DeltaError::UnscalableThermalDelta]);
        }
        if scale < 1.0 && !self.electrode_potential_changes.is_empty() {
            return Err(vec![DeltaError::UnscalableElectricalDelta]);
        }
        let mut limited = self.clone();
        for change in &mut limited.mole_changes {
            change.moles *= scale;
        }
        for change in &mut limited.electrode_changes {
            change.moles *= scale;
        }
        if let Some(ThermalDelta::AddEnergy(energy)) = &mut limited.thermal {
            energy.0 *= scale;
        }
        let residual = limited.validate(vessel);
        if residual.is_empty() {
            Ok(InventoryLimitedDelta {
                delta: limited,
                accepted_fraction: scale,
            })
        } else {
            Err(residual)
        }
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
                    element: v
                        .quantity
                        .strip_prefix("element:")
                        .unwrap_or(&v.quantity)
                        .to_string(),
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
        self.mole_changes.is_empty()
            && self.electrode_changes.is_empty()
            && self.electrode_potential_changes.is_empty()
            && self.thermal.is_none()
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
        let delta = StateDelta::new("test").with_moles(SpeciesId::new("HCl"), Phase::Aqueous, 0.05);

        assert!(delta.validate(&v).is_empty());
        delta.apply(&mut v);

        let hcl = v.contents.iter().find(|p| p.species.0 == "HCl").unwrap();
        assert!((hcl.moles.0 - 0.05).abs() < 1e-15);
    }

    #[test]
    fn withdrawal_within_limits_succeeds() {
        let mut v = test_vessel();
        let delta =
            StateDelta::new("test").with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.05);

        assert!(delta.validate(&v).is_empty());
        delta.commit(&mut v).unwrap();

        let nacl = v.contents.iter().find(|p| p.species.0 == "NaCl").unwrap();
        assert!((nacl.moles.0 - 0.05).abs() < 1e-15);
    }

    #[test]
    fn withdrawal_beyond_limits_rejected() {
        let v = test_vessel();
        let delta =
            StateDelta::new("test").with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.2);

        let errors = delta.validate(&v);
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], DeltaError::Negativity { .. }));
    }

    #[test]
    fn cumulative_bulk_overdraw_is_rejected() {
        let vessel = test_vessel();
        let delta = StateDelta::new("test")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.06)
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.06);
        assert!(matches!(
            delta.validate(&vessel).as_slice(),
            [DeltaError::Negativity { requested, .. }] if (*requested - 0.12).abs() < 1e-12
        ));
    }

    #[test]
    fn one_inventory_scale_preserves_a_coupled_reaction() {
        let vessel = test_vessel();
        let delta = StateDelta::new("test")
            .with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.2)
            .with_moles(SpeciesId::new("HCl"), Phase::Aqueous, 0.4);
        let limited = delta.inventory_limited(&vessel).unwrap();
        assert!((limited.accepted_fraction - 0.5).abs() < 1e-12);
        assert!(
            (limited
                .delta
                .net_moles(&SpeciesId::new("NaCl"), Phase::Aqueous)
                + 0.1)
                .abs()
                < 1e-12
        );
        assert!(
            (limited
                .delta
                .net_moles(&SpeciesId::new("HCl"), Phase::Aqueous)
                - 0.2)
                .abs()
                < 1e-12
        );
    }

    fn zinc_electrode() -> crate::compartment::ElectrodeState {
        crate::compartment::ElectrodeState {
            label: "anode".into(),
            material: "Zn".into(),
            surface_preparation: None,
            substrate_moles: Some(0.01),
            area_m2: 0.001,
            roughness: 1.0,
            double_layer_capacitance_f_per_m2: None,
            interfacial_potential_v: None,
            deposits: Vec::new(),
        }
    }

    #[test]
    fn electrode_and_bulk_changes_commit_as_one_conserved_delta() {
        let mut vessel = Vessel::new(VesselId(0), "cell");
        vessel.electrodes.push(zinc_electrode());
        let delta = StateDelta::new("electrode reaction")
            .with_electrode_moles("anode", ElectrodeInventory::Substrate, -0.002)
            .with_moles(SpeciesId::new("Zn+2"), Phase::Aqueous, 0.002);

        delta.commit_conserved(&mut vessel, 1e-12).unwrap();
        assert!((vessel.electrodes[0].substrate_moles.unwrap() - 0.008).abs() < 1e-12);
        assert!((vessel.moles_of(&SpeciesId::new("Zn+2")).0 - 0.002).abs() < 1e-12);
    }

    #[test]
    fn electrodeposition_updates_conserved_matter_and_computed_geometry() {
        let mut vessel = Vessel::new(VesselId(0), "plating cell");
        let mut electrode = zinc_electrode();
        electrode.material = "Pt".into();
        electrode.substrate_moles = None;
        electrode.area_m2 = 0.01;
        vessel.electrodes.push(electrode);
        vessel.deposit(SpeciesId::new("Cu"), Moles(2e-6), Phase::Solid);
        let growth = crate::compartment::DepositGrowthModel::IslandCoalescence {
            molar_volume_m3_per_mol: 1e-5,
            coalescence_thickness_m: 1e-6,
        };
        let delta = StateDelta::new("plating")
            .with_moles(SpeciesId::new("Cu"), Phase::Solid, -2e-6)
            .with_electrode_moles(
                "anode",
                ElectrodeInventory::Deposit {
                    species: SpeciesId::new("Cu"),
                    growth: Some(growth),
                    effect: Some(crate::electrochemistry::PassivationEffect::Conductive),
                },
                2e-6,
            );
        delta.commit_conserved(&mut vessel, 1e-12).unwrap();
        let deposit = &vessel.electrodes[0].deposits[0];
        assert!((deposit.thickness_m.unwrap() - 1e-6).abs() < 1e-15);
        assert!((deposit.coverage_fraction.unwrap() - 0.002).abs() < 1e-15);
        assert_eq!(
            deposit.effect,
            Some(crate::electrochemistry::PassivationEffect::Conductive)
        );
    }

    #[test]
    fn cumulative_electrode_overdraw_is_atomic() {
        let mut vessel = Vessel::new(VesselId(0), "cell");
        vessel.electrodes.push(zinc_electrode());
        let delta = StateDelta::new("bad electrode reaction")
            .with_electrode_moles("anode", ElectrodeInventory::Substrate, -0.006)
            .with_electrode_moles("anode", ElectrodeInventory::Substrate, -0.006);

        assert!(matches!(
            delta.commit(&mut vessel),
            Err(errors) if errors.iter().any(|error| matches!(error, DeltaError::ElectrodeNegativity { .. }))
        ));
        assert_eq!(vessel.electrodes[0].substrate_moles, Some(0.01));
    }

    #[test]
    fn commit_rejects_invalid_delta() {
        let mut v = test_vessel();
        let delta =
            StateDelta::new("test").with_moles(SpeciesId::new("NaCl"), Phase::Aqueous, -0.2);

        let result = delta.commit(&mut v);
        assert!(result.is_err());
        // Vessel should be unchanged
        let nacl = v.contents.iter().find(|p| p.species.0 == "NaCl").unwrap();
        assert!((nacl.moles.0 - 0.1).abs() < 1e-15);
    }

    #[test]
    fn thermal_delta_changes_temperature() {
        let mut v = test_vessel();
        let delta =
            StateDelta::new("test").with_thermal(ThermalDelta::SetTemperature(Kelvin(373.15)));

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
        assert!(
            result.is_ok(),
            "balanced reaction should commit: {:?}",
            result
        );

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
