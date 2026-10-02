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

#[derive(Debug, Clone, PartialEq)]
pub struct ElectrodeInterfacialSpeciesDelta {
    pub electrode: String,
    pub reaction_id: String,
    pub species: String,
    pub surface_concentration_mol_per_m3: f64,
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
    /// Persistent near-surface concentration changes after a transient slice.
    pub electrode_interfacial_species_changes: Vec<ElectrodeInterfacialSpeciesDelta>,
    /// Signed changes to material bound on sorbents.
    pub adsorbed_changes: Vec<AdsorbedDelta>,
    /// Thermal state change.
    pub thermal: Option<ThermalDelta>,
    /// Replacement temperature limitations, if changed by the proposal.
    pub unpriced_heat: Option<Vec<SpeciesId>>,
    /// Which model produced this delta.
    pub source: &'static str,
    /// Complete legacy-solver result, including interfaces and derived state.
    /// Private so native deltas cannot accidentally bypass their signed changes.
    replacement: Option<Box<crate::vessel::Vessel>>,
    proposal_base: Option<(String, String)>,
}

/// Signed transfer to a (sorbent, sorbate) inventory; bulk changes are separate.
#[derive(Debug, Clone, PartialEq)]
pub struct AdsorbedDelta {
    pub sorbent: SpeciesId,
    pub sorbate: SpeciesId,
    pub moles: f64,
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
    InvalidState {
        field: String,
    },
    StaleProposal,
    UnscalableSnapshot,
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
            DeltaError::InvalidState { field } => write!(f, "invalid proposed state: {field}"),
            DeltaError::StaleProposal => write!(f, "vessel changed after this proposal was prepared"),
            DeltaError::UnscalableSnapshot => write!(f, "a complete solver snapshot cannot be inventory-scaled"),
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
            adsorbed_changes: Vec::new(),
            electrode_changes: Vec::new(),
            electrode_potential_changes: Vec::new(),
            electrode_interfacial_species_changes: Vec::new(),
            thermal: None,
            unpriced_heat: None,
            source,
            replacement: None,
            proposal_base: None,
        }
    }

    /// Attach the entire result of a solver that works on a clone. The base
    /// prevents a delayed proposal from overwriting an intervening operation.
    pub(crate) fn with_snapshot(
        mut self,
        before: &crate::vessel::Vessel,
        after: &crate::vessel::Vessel,
    ) -> Self {
        let base = Self::state_key(before);
        if base != Self::state_key(after) {
            self.proposal_base = Some((base, self.terms_key()));
            self.replacement = Some(Box::new(after.clone()));
        }
        self
    }

    // Debug includes transient fields deliberately omitted from persistence.
    // This guard compares in-process snapshots, never a public wire format.
    fn state_key(vessel: &crate::vessel::Vessel) -> String {
        format!("{vessel:?}")
    }

    fn terms_key(&self) -> String {
        format!(
            "{:?}",
            (
                &self.mole_changes,
                &self.adsorbed_changes,
                &self.electrode_changes,
                &self.electrode_potential_changes,
                &self.electrode_interfacial_species_changes,
                &self.thermal,
                &self.unpriced_heat
            )
        )
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

    pub fn with_electrode_interfacial_species(
        mut self,
        electrode: impl Into<String>,
        reaction_id: impl Into<String>,
        species: impl Into<String>,
        surface_concentration_mol_per_m3: f64,
    ) -> Self {
        self.electrode_interfacial_species_changes
            .push(ElectrodeInterfacialSpeciesDelta {
                electrode: electrode.into(),
                reaction_id: reaction_id.into(),
                species: species.into(),
                surface_concentration_mol_per_m3,
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

    /// Add a bound-inventory change (positive binds, negative releases).
    pub fn with_adsorbed(mut self, sorbent: SpeciesId, sorbate: SpeciesId, moles: f64) -> Self {
        self.adsorbed_changes.push(AdsorbedDelta {
            sorbent,
            sorbate,
            moles,
        });
        self
    }

    /// Set the thermal change.
    pub fn with_thermal(mut self, thermal: ThermalDelta) -> Self {
        self.thermal = Some(thermal);
        self
    }

    /// Numerical domain checks shared by cloned and direct solver commits.
    pub(crate) fn validate_state(candidate: &crate::vessel::Vessel) -> Vec<DeltaError> {
        let mut errors = Vec::new();
        if let Some(input) = &candidate.heat_input {
            if !input.temperature.0.is_finite()
                || input.temperature.0 <= 0.0
                || !input.delivered_j.is_finite()
                || input.delivered_j < 0.0
                || input
                    .contents
                    .iter()
                    .any(|p| !p.moles.0.is_finite() || p.moles.0 < 0.0)
            {
                errors.push(DeltaError::InvalidState {
                    field: "heat input proposal".into(),
                });
            }
        }
        let ledger = crate::ledger::ConservedLedger::from_vessel(candidate);
        if !candidate.liquid_volume().0.is_finite()
            || !candidate.heat_capacity().is_finite()
            || !ledger.mass.is_finite()
            || !ledger.energy.is_finite()
            || !ledger.charge.is_finite()
            || ledger.elements.values().any(|n| !n.is_finite())
        {
            errors.push(DeltaError::InvalidState {
                field: "aggregate inventory/energy".into(),
            });
        }
        if candidate
            .ignition_feed_temperature
            .is_some_and(|t| !t.0.is_finite() || t.0 <= 0.0)
            || !candidate.temperature.0.is_finite()
            // COOL exposes zero only as a disclosed mathematical floor.
            // Native thermochemistry and ignition retain positive domains.
            || candidate.temperature.0 < 0.0
            || !candidate.pressure.0.is_finite()
            || candidate.pressure.0 < 0.0
        {
            errors.push(DeltaError::InvalidState {
                field: "temperature/pressure".into(),
            });
        }
        let invalid_boundary = match candidate.headspace {
            crate::vessel::Headspace::Open => false,
            crate::vessel::Headspace::Sealed { volume } => !volume.0.is_finite() || volume.0 <= 0.0,
            crate::vessel::Headspace::PressureControlled { pressure, volume } => {
                !pressure.0.is_finite()
                    || pressure.0 <= 0.0
                    || !volume.0.is_finite()
                    || volume.0 < 0.0
            }
            crate::vessel::Headspace::Swept { pressure } => {
                !pressure.0.is_finite() || pressure.0 <= 0.0
            }
        };
        if invalid_boundary
            || matches!(candidate.thermal_mode, crate::vessel::ThermalMode::Thermostatted(t) if !t.0.is_finite() || t.0 <= 0.0)
        {
            errors.push(DeltaError::InvalidState {
                field: "boundary conditions".into(),
            });
        }
        for (field, value) in [
            ("solute charge", candidate.solute_charge),
            ("excess enthalpy", candidate.excess_enthalpy_j),
            ("pending CO2 transfer", candidate.pending_co2_transfer_mol),
        ] {
            if !value.is_finite() {
                errors.push(DeltaError::InvalidState {
                    field: field.into(),
                });
            }
        }
        if candidate
            .co2_partial_pressure_atm
            .is_some_and(|p| !p.is_finite() || p < 0.0)
        {
            errors.push(DeltaError::InvalidState {
                field: "CO2 driving pressure".into(),
            });
        }
        for portion in &candidate.contents {
            if !portion.moles.0.is_finite() || portion.moles.0 < 0.0 {
                errors.push(DeltaError::InvalidState {
                    field: format!("inventory {}", portion.species.0),
                });
            }
        }
        let amounts = candidate
            .adsorbed
            .iter()
            .map(|p| p.moles.0)
            .chain(
                candidate
                    .surfaces
                    .iter()
                    .flat_map(|s| s.occupancy.iter().map(|p| p.moles.0)),
            )
            .chain(
                candidate
                    .exchanges
                    .iter()
                    .flat_map(|s| s.occupancy.iter().map(|p| p.moles.0)),
            )
            .chain(
                candidate
                    .solid_solutions
                    .iter()
                    .flat_map(|s| s.components.iter().map(|p| p.moles.0)),
            )
            .chain(candidate.electrodes.iter().flat_map(|s| {
                s.substrate_moles
                    .into_iter()
                    .chain(s.deposits.iter().map(|p| p.moles))
            }))
            .chain(candidate.unresolved_materials.iter().map(|p| p.amount))
            .chain(candidate.material_objects.iter().map(|p| p.mass_g))
            .chain(
                candidate
                    .material_objects
                    .iter()
                    .flat_map(|p| p.components.iter().map(|c| c.moles.0)),
            )
            .chain(candidate.surfaces.iter().flat_map(|p| {
                [
                    p.mass.0,
                    p.specific_area_m2_per_g,
                    p.strong_capacity.0,
                    p.weak_capacity.0,
                ]
            }))
            .chain(
                candidate
                    .exchanges
                    .iter()
                    .flat_map(|p| [p.dry_mass.0, p.capacity.0]),
            );
        if amounts.into_iter().any(|n| !n.is_finite() || n < 0.0) {
            errors.push(DeltaError::InvalidState {
                field: "interface/material inventory".into(),
            });
        }
        for (field, value) in [
            ("free proton", candidate.free_proton),
            ("free hydroxide", candidate.free_hydroxide),
            ("elapsed time", candidate.elapsed_seconds),
        ] {
            if !value.is_finite() || value < 0.0 {
                errors.push(DeltaError::InvalidState {
                    field: field.into(),
                });
            }
        }
        for electrode in &candidate.electrodes {
            if !electrode.area_m2.is_finite()
                || electrode.area_m2 < 0.0
                || !electrode.roughness.is_finite()
                || electrode.roughness < 0.0
                || !(electrode.area_m2 * electrode.roughness).is_finite()
                || electrode.double_layer_capacitance_f_per_m2.is_some()
                    != electrode.interfacial_potential_v.is_some()
                || electrode
                    .double_layer_capacitance_f_per_m2
                    .is_some_and(|c| {
                        !c.is_finite() || c <= 0.0 || !(c * electrode.area_m2).is_finite()
                    })
                || electrode
                    .interfacial_potential_v
                    .is_some_and(|p| !p.is_finite())
                || electrode.interfacial_species.iter().any(|p| {
                    !p.surface_concentration_mol_per_m3.is_finite()
                        || p.surface_concentration_mol_per_m3 < 0.0
                })
                || electrode.deposits.iter().any(|p| {
                    p.thickness_m.is_some_and(|x| !x.is_finite() || x < 0.0)
                        || p.coverage_fraction
                            .is_some_and(|x| !x.is_finite() || !(0.0..=1.0).contains(&x))
                        || p.electrical_resistivity_ohm_m
                            .is_some_and(|x| !x.is_finite() || x < 0.0)
                })
                || !electrode
                    .deposits
                    .iter()
                    .filter_map(|p| Some(p.thickness_m? * p.electrical_resistivity_ohm_m?))
                    .sum::<f64>()
                    .is_finite()
                || electrode.diagnostics.as_ref().is_some_and(|d| {
                    !d.seconds.is_finite()
                        || d.seconds < 0.0
                        || [
                            d.balance.electrode_potential_v,
                            d.balance.terminal_potential_v,
                        ]
                        .into_iter()
                        .any(|x| !x.is_finite())
                        || [
                            d.balance.net_current_density_a_per_m2,
                            d.balance.capacitive_current_density_a_per_m2,
                            d.balance.total_current_density_a_per_m2,
                        ]
                        .into_iter()
                        .any(|x| !x.is_finite() || !(x * electrode.area_m2).is_finite())
                        || d.balance.partial_currents.iter().any(|p| {
                            !p.current_density_a_per_m2.is_finite()
                                || !(p.current_density_a_per_m2 * electrode.area_m2).is_finite()
                        })
                        || !d
                            .balance
                            .partial_currents
                            .iter()
                            .map(|p| p.current_density_a_per_m2)
                            .sum::<f64>()
                            .is_finite()
                        || d.interfacial_conditions.iter().any(|p| {
                            [
                                p.bulk_concentration_mol_per_m3,
                                p.surface_concentration_mol_per_m3,
                                p.bulk_activity,
                                p.surface_activity,
                            ]
                            .into_iter()
                            .any(|x| !x.is_finite() || x < 0.0)
                                || [
                                    p.bulk_equilibrium_potential_v,
                                    p.surface_equilibrium_potential_v,
                                ]
                                .into_iter()
                                .any(|x| !x.is_finite())
                                || p.surface_ph.is_some_and(|x| !x.is_finite())
                                || p.migration_potential_v.is_some_and(|x| !x.is_finite())
                        })
                        || d.applied_parameters.iter().any(|p| {
                            p.nominal.validate().is_err()
                                || p.reference.validate().is_err()
                                || p.relative_uncertainty
                                    .is_some_and(|x| !x.is_finite() || x < 0.0)
                        })
                })
            {
                errors.push(DeltaError::InvalidState {
                    field: "electrode interface".into(),
                });
            }
        }
        // These persisted fractions and geometries are not extra matter,
        // but they still feed optical/transport models and public scenes.
        // A complete solver snapshot must not bypass their numeric domains.
        let nonnegative = |x: f64| x.is_finite() && x >= 0.0;
        let fraction = |x: f64| x.is_finite() && (0.0..=1.0).contains(&x);
        if candidate.unresolved_materials.iter().any(|p| {
            !fraction(p.protein_denatured_fraction)
                || p.enzyme_hydrolysis
                    .as_ref()
                    .is_some_and(|e| !fraction(e.converted_fraction))
        }) || candidate.material_objects.iter().any(|p| {
            !nonnegative(p.state.elapsed_seconds)
                // Water exchange is signed (inward/outward), unlike time.
                || !p.state.exchanged_water_moles.is_finite()
                || !fraction(p.state.browned_fraction)
        }) || [
            candidate.foam.trapped_gas_liters,
            candidate.foam.volume_liters,
            candidate.foam.peak_volume_liters,
        ]
        .into_iter()
        .any(|x| !nonnegative(x))
            || candidate
                .surface_particles
                .as_ref()
                .is_some_and(|p| !fraction(p.coverage_fraction) || !fraction(p.cleared_fraction))
            || candidate
                .surface_colours
                .iter()
                .any(|p| !nonnegative(p.moles.0) || !fraction(p.spread_fraction))
            || candidate.emulsion.as_ref().is_some_and(|p| {
                !nonnegative(p.dispersed_volume_l)
                    || !p.half_life_seconds.is_finite()
                    || p.half_life_seconds <= 0.0
            })
            || candidate.soap_scum.as_ref().is_some_and(|p| {
                [
                    p.aggregate_mass_g,
                    p.divalent_ion_moles,
                    p.soap_equivalent_moles,
                ]
                .into_iter()
                .any(|x| !nonnegative(x))
            })
            || candidate.lemon_paper_mark.as_ref().is_some_and(|p| {
                !nonnegative(p.lemon_amount_g)
                    || !nonnegative(p.paper_amount_g)
                    || !fraction(p.browned_fraction)
            })
        {
            errors.push(DeltaError::InvalidState {
                field: "material progress/geometry".into(),
            });
        }
        for solution in candidate
            .solution
            .iter()
            .chain(candidate.resolved.solution.iter())
        {
            if !solution.ph.is_finite()
                || solution.pe.is_some_and(|pe| !pe.is_finite())
                || solution
                    .redox
                    .iter()
                    .any(|s| !s.molality.is_finite() || s.molality < 0.0)
                || solution.solvent_activity.as_ref().is_some_and(|s| {
                    [s.water_activity, s.particle_molality, s.ionic_strength]
                        .iter()
                        .any(|v| !v.is_finite() || *v < 0.0)
                })
                || !solution.ionic_strength.is_finite()
                || solution.ionic_strength < 0.0
                || solution
                    .solvent_kg
                    .is_some_and(|mass| !mass.is_finite() || mass <= 0.0)
                || solution.species.iter().any(|s| {
                    !s.molality.is_finite()
                        || s.molality < 0.0
                        || !s.activity.is_finite()
                        || s.activity < 0.0
                })
            {
                errors.push(DeltaError::InvalidState {
                    field: "solution characterisation".into(),
                });
            }
        }
        errors
    }

    /// Gas narration is also the numeric boundary ledger used by later stages.
    pub(crate) fn validate_gas_events(
        vessel: &crate::vessel::Vessel,
        events: &[crate::ops::Event],
    ) -> Vec<DeltaError> {
        events
            .iter()
            .filter_map(|event| {
                let (id, species, moles) = match event {
                    crate::ops::Event::GasEvolved {
                        vessel,
                        species,
                        moles,
                        ..
                    }
                    | crate::ops::Event::GasAbsorbed {
                        vessel,
                        species,
                        moles,
                        ..
                    }
                    | crate::ops::Event::GasContained {
                        vessel,
                        species,
                        moles,
                        ..
                    } => (*vessel, species, moles.0),
                    _ => return None,
                };
                if id != vessel.id || !moles.is_finite() || moles < 0.0 {
                    return Some(DeltaError::InvalidState {
                        field: "gas event identity/amount".into(),
                    });
                }
                if crate::species::lookup(species)
                    .and_then(|data| crate::stoich::parse_formula(data.formula).ok())
                    .is_none()
                {
                    return Some(DeltaError::InvalidState {
                        field: format!("unaccounted gas {}", species.0),
                    });
                }
                None
            })
            .collect()
    }

    /// Shared element ledger for native and delta solver commits. GasContained
    /// reports owned matter; only transfers extend the conserved boundary.
    pub(crate) fn validate_conservation(
        before: &crate::vessel::Vessel,
        after: &crate::vessel::Vessel,
        events: &[crate::ops::Event],
        tolerance: f64,
    ) -> Vec<DeltaError> {
        let mut errors = Self::validate_gas_events(after, events);
        if !tolerance.is_finite() || tolerance < 0.0 {
            errors.push(DeltaError::InvalidState {
                field: "conservation tolerance".into(),
            });
        }
        if !errors.is_empty() {
            return errors;
        }
        let mut ledger_before = crate::ledger::ConservedLedger::from_vessel(before);
        let mut ledger_after = crate::ledger::ConservedLedger::from_vessel(after);
        for event in events {
            let (species, moles) = match event {
                crate::ops::Event::GasEvolved { species, moles, .. } => (species, moles.0),
                crate::ops::Event::GasAbsorbed { species, moles, .. } => (species, -moles.0),
                _ => continue,
            };
            let formula = crate::species::lookup(species)
                .and_then(|data| crate::stoich::parse_formula(data.formula).ok())
                .expect("gas event validated");
            for (element, count) in formula.counts {
                if moles >= 0.0 {
                    *ledger_after.elements.entry(element).or_default() += moles * count;
                } else {
                    *ledger_before.elements.entry(element).or_default() -= moles * count;
                }
            }
        }
        if ledger_before
            .elements
            .values()
            .chain(ledger_after.elements.values())
            .any(|v| !v.is_finite())
        {
            return vec![DeltaError::InvalidState {
                field: "aggregate element ledger".into(),
            }];
        }
        ledger_before
            .check_against(&ledger_after, tolerance, 1e-15)
            .into_iter()
            .filter(|v| v.quantity.starts_with("element:"))
            .map(|v| DeltaError::ElementImbalance {
                element: v.quantity.strip_prefix("element:").unwrap().into(),
                net: v.delta,
            })
            .collect()
    }

    /// Validate this delta against a vessel state.
    /// Returns a list of errors (empty = valid).
    pub fn validate(&self, vessel: &crate::vessel::Vessel) -> Vec<DeltaError> {
        let mut errors = Vec::new();
        if let Some((base, terms)) = &self.proposal_base {
            if *base != Self::state_key(vessel) {
                errors.push(DeltaError::StaleProposal);
            }
            if *terms != self.terms_key() {
                errors.push(DeltaError::InvalidState {
                    field: "complete snapshot proposal was edited".into(),
                });
            }
        }
        if let Some(candidate) = &self.replacement {
            if candidate.id != vessel.id || candidate.label != vessel.label {
                errors.push(DeltaError::InvalidState {
                    field: "vessel identity".into(),
                });
            }
            errors.extend(Self::validate_state(candidate));
        }

        if self.thermal.is_some_and(|thermal| match thermal {
            ThermalDelta::SetTemperature(t) => !t.0.is_finite() || t.0 <= 0.0,
            ThermalDelta::AddEnergy(j) => !j.0.is_finite(),
        }) {
            errors.push(DeltaError::InvalidState {
                field: "thermal delta".into(),
            });
        }

        // Validate each applied prefix; a later deposit cannot repair an
        // earlier withdrawal that would have been silently clamped.
        let mut bulk_prefix = std::collections::BTreeMap::new();
        for change in &self.mole_changes {
            let available = vessel
                .contents
                .iter()
                .filter(|p| p.species == change.species && p.phase == change.phase)
                .map(|p| p.moles.0)
                .sum::<f64>();
            let net = bulk_prefix
                .entry((change.species.0.clone(), change.phase))
                .or_insert(0.0);
            *net += change.moles;
            if !net.is_finite() || !available.is_finite() || available + *net < -1e-15 {
                errors.push(DeltaError::Negativity {
                    species: change.species.0.clone(),
                    phase: change.phase,
                    available,
                    requested: -*net,
                });
            }
        }

        // Check every prefix, matching apply order. A later deposit must
        // not mask an earlier withdrawal that apply would otherwise clamp.
        let mut cumulative = std::collections::BTreeMap::new();
        for change in &self.adsorbed_changes {
            let available: f64 = vessel
                .adsorbed
                .iter()
                .filter(|entry| entry.sorbent == change.sorbent && entry.sorbate == change.sorbate)
                .map(|entry| entry.moles.0)
                .sum();
            let net = cumulative
                .entry((change.sorbent.0.clone(), change.sorbate.0.clone()))
                .or_insert(0.0);
            *net += change.moles;
            if !change.moles.is_finite() || !net.is_finite() || !available.is_finite() {
                errors.push(DeltaError::Negativity {
                    species: change.sorbate.0.clone(),
                    phase: Phase::Aqueous,
                    available,
                    requested: f64::NAN,
                });
            } else if available + *net < -1e-15 {
                errors.push(DeltaError::Negativity {
                    species: change.sorbate.0.clone(),
                    phase: Phase::Aqueous,
                    available,
                    requested: -*net,
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

        let mut interface_keys = std::collections::BTreeSet::new();
        for change in &self.electrode_interfacial_species_changes {
            let matching_count = vessel
                .electrodes
                .iter()
                .filter(|electrode| electrode.label == change.electrode)
                .count();
            let key = (
                change.electrode.as_str(),
                change.reaction_id.as_str(),
                change.species.as_str(),
            );
            if matching_count != 1
                || change.reaction_id.trim().is_empty()
                || change.species.trim().is_empty()
                || !change.surface_concentration_mol_per_m3.is_finite()
                || change.surface_concentration_mol_per_m3 < 0.0
            {
                errors.push(DeltaError::InvalidElectrodeDelta {
                    electrode: change.electrode.clone(),
                    reason: "interfacial update requires one named electrode, named reaction/species and a non-negative concentration".into(),
                });
            } else if !interface_keys.insert(key) {
                errors.push(DeltaError::InvalidElectrodeDelta {
                    electrode: change.electrode.clone(),
                    reason: "one interfacial species may be set only once per atomic delta".into(),
                });
            }
        }

        errors
    }

    /// Apply this delta to a vessel. Call `validate()` first to check
    /// for errors; this method applies unconditionally.
    pub fn apply(&self, vessel: &mut crate::vessel::Vessel) {
        if let Some(snapshot) = &self.replacement {
            *vessel = (**snapshot).clone();
            return;
        }
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
                vessel.contents.retain(|p| p.moles.0 > 0.0);
            }
        }

        for change in &self.adsorbed_changes {
            if change.moles > 0.0 {
                if let Some(entry) = vessel.adsorbed.iter_mut().find(|entry| {
                    entry.sorbent == change.sorbent && entry.sorbate == change.sorbate
                }) {
                    entry.moles.0 += change.moles;
                } else {
                    vessel.adsorbed.push(crate::vessel::AdsorbedAmount {
                        sorbent: change.sorbent.clone(),
                        sorbate: change.sorbate.clone(),
                        moles: Moles(change.moles),
                    });
                }
            } else if change.moles < 0.0 {
                let mut remaining = -change.moles;
                for entry in &mut vessel.adsorbed {
                    if entry.sorbent == change.sorbent && entry.sorbate == change.sorbate {
                        let take = remaining.min(entry.moles.0);
                        entry.moles.0 -= take;
                        remaining -= take;
                    }
                }
                vessel.adsorbed.retain(|entry| entry.moles.0 > 0.0);
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
            electrode.deposits.retain(|deposit| deposit.moles > 0.0);
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

        for change in &self.electrode_interfacial_species_changes {
            if let Some(electrode) = vessel
                .electrodes
                .iter_mut()
                .find(|electrode| electrode.label == change.electrode)
            {
                if let Some(state) = electrode.interfacial_species.iter_mut().find(|state| {
                    state.reaction_id == change.reaction_id && state.species == change.species
                }) {
                    state.surface_concentration_mol_per_m3 =
                        change.surface_concentration_mol_per_m3;
                } else {
                    electrode.interfacial_species.push(
                        crate::compartment::ElectrodeInterfacialSpecies {
                            reaction_id: change.reaction_id.clone(),
                            species: change.species.clone(),
                            surface_concentration_mol_per_m3: change
                                .surface_concentration_mol_per_m3,
                        },
                    );
                }
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

        if let Some(species) = &self.unpriced_heat {
            vessel.unpriced_heat = species.clone();
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
        let mut candidate = vessel.clone();
        self.apply(&mut candidate);
        let errors = Self::validate_state(&candidate);
        if !errors.is_empty() {
            return Err(errors);
        }
        *vessel = candidate;
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
        if self.replacement.is_some() {
            return Err(vec![DeltaError::UnscalableSnapshot]);
        }
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
        if scale < 1.0
            && (!self.electrode_potential_changes.is_empty()
                || !self.electrode_interfacial_species_changes.is_empty())
        {
            return Err(vec![DeltaError::UnscalableElectricalDelta]);
        }
        let mut limited = self.clone();
        for change in &mut limited.mole_changes {
            change.moles *= scale;
        }
        for change in &mut limited.adsorbed_changes {
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
        self.commit_conserved_with_events(vessel, tolerance, &[])
    }

    /// Commit a reaction with explicit gas outlets/inlets included in the
    /// conserved system. Rejected proposals commit neither state nor events.
    pub fn commit_conserved_with_events(
        &self,
        vessel: &mut crate::vessel::Vessel,
        tolerance: f64,
        events: &[crate::ops::Event],
    ) -> Result<(), Vec<DeltaError>> {
        // Step 1: positivity
        let errors = self.validate(vessel);
        if !errors.is_empty() {
            return Err(errors);
        }

        // Step 2: snapshot before
        let snapshot = vessel.clone();

        // Step 3: apply
        self.apply(vessel);

        let result_errors = Self::validate_state(vessel);
        if !result_errors.is_empty() {
            *vessel = snapshot;
            return Err(result_errors);
        }

        let errors = Self::validate_conservation(&snapshot, vessel, events, tolerance);
        if !errors.is_empty() {
            *vessel = snapshot;
            return Err(errors);
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
            && self.adsorbed_changes.is_empty()
            && self.electrode_changes.is_empty()
            && self.electrode_potential_changes.is_empty()
            && self.electrode_interfacial_species_changes.is_empty()
            && self.thermal.is_none()
            && self.unpriced_heat.is_none()
            && self.replacement.is_none()
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
            interfacial_species: Vec::new(),
            diagnostics: None,
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
