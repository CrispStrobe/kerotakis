//! Conservative one-dimensional transport before reactive coupling.
//!
//! A transport cell is an ordinary [`Vessel`]. Liquid and aqueous portions
//! move; solids, headspaces, surfaces, exchangers, and solid solutions remain
//! owned by their cell. One step is an explicit first-order upwind update with
//! a Courant fraction in `[0, 1]`. Every outflow is snapshotted before any cell
//! mutates, so the result does not depend on iteration order.
//!
//! The boundary is either a finite feed vessel, debited by exactly the parcel
//! it injects ([`CellChain::advance_from_feed`], the default for the bench
//! verb), or an explicitly declared constant-composition reservoir
//! ([`CellChain::advance`], `transport ... reservoir`). In both cases the
//! per-step ledger is `old + injected = new + effluent` for every species.
//!
//! [`CellChain::advance`] deliberately performs no chemistry and invalidates
//! stale solution metadata after matter moves. [`CellChain::advance_reactive`]
//! adds the operator-splitting seam: transport first, then local equilibrium,
//! with whole-chain rollback if any cell solve fails.

use serde::{Deserialize, Serialize};

use crate::ops::Event;
use crate::solve::{adiabatic_mix_into, Equilibrator, SolveError};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Liters, Moles};
use crate::vessel::{Portion, ThermalMode, Vessel, VesselId};

// Hydraulic geometry tolerance, relative to the first cell's liquid volume.
//
// `Vessel::liquid_volume` is an additive solvent-volume proxy and deliberately
// excludes solute volume. Ordinary chemistry moves it without changing the
// physical cell: aqueous readback shifts it by ppm, and a dissolving hydrate
// (gypsum releases 2 H2O per CaSO4) adds a few hundred ppm to a dilute cell.
// The ledger does not depend on uniform volumes - every outflow is a fraction
// of the cell's OWN mobile inventory and every inflow is exactly what left the
// neighbour or the inlet - so matter and charge close exactly whatever the
// mismatch. The tolerance therefore only bounds the hydraulic interpretation
// of one Courant number for the whole column: a 0.1 % volume mismatch changes
// the replaced fraction of a cell by at most 0.1 % per step, far below any
// observable the column is asked to show. A cell that differs by more than
// that is a different geometry and is still refused.
pub const HYDRAULIC_VOLUME_RELATIVE_TOLERANCE: f64 = 1e-3;
const VOLUME_ABSOLUTE_TOLERANCE_L: f64 = 1e-12;

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("a 1-D cell chain needs at least one cell")]
    EmptyChain,
    #[error("Courant fraction must be finite and between 0 and 1, got {fraction}")]
    InvalidCourant { fraction: f64 },
    #[error("transport cell {cell} has no finite positive liquid volume (got {volume_l} L)")]
    InvalidCellVolume { cell: usize, volume_l: f64 },
    #[error(
        "transport cell {cell} has {actual_l} L of liquid; the uniform chain requires {expected_l} L"
    )]
    NonUniformCellVolume {
        cell: usize,
        expected_l: f64,
        actual_l: f64,
    },
    #[error(
        "the inlet represents {actual_l} L of liquid; one full transport cell requires {expected_l} L"
    )]
    InletVolume { expected_l: f64, actual_l: f64 },
    #[error(
        "the feed vessel holds {available_l} L of liquid but transport step {step} needs {required_l} L; \
         a feed vessel is finite and is drained as it flows (add more feed, or write 'reservoir' to \
         declare a constant-composition source)"
    )]
    InletExhausted {
        step: u32,
        required_l: f64,
        available_l: f64,
    },
    #[error("{location} contains a non-finite temperature, charge, or mobile amount")]
    InvalidMobileState { location: String },
    #[error("transport cell {cell} is thermostatted; conservative AQ-011 transport requires adiabatic cells")]
    ThermostattedCell { cell: usize },
    #[error("duplicate vessel {vessel} appears twice in transport chain")]
    DuplicateChainCell { vessel: VesselId },
    #[error("inlet {vessel} is also a chain cell; inlet must be an external reservoir")]
    InletIsChainCell { vessel: VesselId },
    #[error("receiver {vessel} is also a chain cell; receiver must be distinct from the chain")]
    ReceiverIsChainCell { vessel: VesselId },
    #[error("receiver {vessel} is also the inlet; receiver and inlet must be distinct")]
    ReceiverIsInlet { vessel: VesselId },
}

#[derive(Debug, thiserror::Error)]
pub enum ReactiveTransportError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error("reaction solve failed in transport cell {cell}: {source}")]
    Reaction {
        cell: usize,
        #[source]
        source: SolveError,
    },
}

/// A mobile boundary parcel reported by one transport step.
///
/// The inlet is a representative full-cell composition; [`TransportStep`]
/// records only the fraction actually injected. The effluent is the matching
/// fraction removed from the last cell. Keeping both makes an open-column
/// conservation equation explicit: `old + injected = new + effluent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileParcel {
    pub contents: Vec<Portion>,
    pub temperature: Kelvin,
    /// Net analytical solute charge, in moles of charge equivalents.
    pub solute_charge: f64,
    /// Free hydroxide measured by the last aqueous solve, in moles. Zero
    /// means no measured free base was present. Scaled with the aqueous
    /// fraction so a transported parcel does not leave its donor's measured
    /// state behind.
    #[serde(default)]
    pub free_hydroxide: f64,
    /// Free proton measured by the last aqueous solve, in moles.
    #[serde(default)]
    pub free_proton: f64,
}

impl MobileParcel {
    /// Extract every liquid/aqueous portion from a vessel without mutation.
    pub fn from_vessel(vessel: &Vessel) -> Self {
        Self {
            contents: vessel
                .contents
                .iter()
                .filter(|portion| is_mobile(portion.phase))
                .cloned()
                .collect(),
            temperature: vessel.temperature,
            solute_charge: vessel.solute_charge,
            free_hydroxide: vessel.free_hydroxide,
            free_proton: vessel.free_proton,
        }
    }

    pub fn moles_of(&self, species: &SpeciesId) -> Moles {
        Moles(
            self.contents
                .iter()
                .filter(|portion| &portion.species == species)
                .map(|portion| portion.moles.0)
                .sum(),
        )
    }

    pub fn liquid_volume(&self) -> Liters {
        Liters(
            self.contents
                .iter()
                .filter(|portion| portion.phase == Phase::Liquid)
                .filter_map(|portion| {
                    species::lookup(&portion.species)
                        .map(|data| data.liters_from_moles(portion.moles).0)
                })
                .sum(),
        )
    }

    /// Heat capacity of the parcel where it currently is, J/K.
    ///
    /// Per phase and along each species' curve, exactly as `Vessel` charges
    /// its own contents. It used to read the registry constant regardless of
    /// phase, which meant a parcel of ice mixed into a cell at liquid
    /// water's price - the same defect `states::heat_capacity_in` was
    /// written to close inside a vessel.
    pub fn heat_capacity(&self) -> f64 {
        self.heat_capacity_at(self.temperature.0)
    }

    /// The same, at a stated temperature.
    pub fn heat_capacity_at(&self, t_k: f64) -> f64 {
        self.contents
            .iter()
            .filter_map(|portion| {
                species::lookup(&portion.species).map(|data| {
                    portion.moles.0 * crate::states::heat_capacity_at(data, portion.phase, t_k)
                })
            })
            .sum()
    }

    /// The heat this parcel absorbs going from `t0` to `t1`, J.
    pub fn energy_between(&self, t0: f64, t1: f64) -> f64 {
        crate::solve::portions_enthalpy(
            self.contents
                .iter()
                .map(|portion| (&portion.species, portion.moles.0, portion.phase)),
            t0,
            t1,
        )
    }

    pub fn sensible_energy(&self) -> Joules {
        Joules(self.energy_between(Kelvin::STANDARD.0, self.temperature.0))
    }

    fn scaled(&self, fraction: f64) -> Self {
        Self {
            contents: self
                .contents
                .iter()
                .filter_map(|portion| {
                    let moles = portion.moles.0 * fraction;
                    (moles > 0.0).then(|| Portion {
                        species: portion.species.clone(),
                        moles: Moles(moles),
                        phase: portion.phase,
                    })
                })
                .collect(),
            temperature: self.temperature,
            solute_charge: self.solute_charge * fraction,
            free_hydroxide: self.free_hydroxide * fraction,
            free_proton: self.free_proton * fraction,
        }
    }

    fn validate(&self, location: impl Into<String>) -> Result<(), TransportError> {
        if !self.temperature.0.is_finite()
            || self.temperature.0 <= 0.0
            || !self.solute_charge.is_finite()
            || !self.free_hydroxide.is_finite()
            || self.free_hydroxide < 0.0
            || !self.free_proton.is_finite()
            || self.free_proton < 0.0
            || self.contents.iter().any(|portion| {
                !is_mobile(portion.phase) || !portion.moles.0.is_finite() || portion.moles.0 < 0.0
            })
        {
            return Err(TransportError::InvalidMobileState {
                location: location.into(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStep {
    pub courant_fraction: f64,
    pub injected: MobileParcel,
    pub effluent: MobileParcel,
}

/// Solver output associated with one cell after a reactive transport step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellReaction {
    pub cell: usize,
    pub events: Vec<Event>,
}

/// One conservative transport step followed by local cell equilibria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactiveTransportStep {
    pub transport: TransportStep,
    pub reactions: Vec<CellReaction>,
}

/// A uniform one-dimensional finite-volume chain.
#[derive(Debug, Clone)]
pub struct CellChain {
    cells: Vec<Vessel>,
}

impl CellChain {
    pub fn new(cells: Vec<Vessel>) -> Result<Self, TransportError> {
        let chain = Self { cells };
        chain.uniform_cell_volume(false)?;
        Ok(chain)
    }

    pub fn cells(&self) -> &[Vessel] {
        &self.cells
    }

    /// Mutable cell access is the coupling seam for AQ-012. Geometry is
    /// validated again before every transport step, so a reaction cannot
    /// silently leave an invalid chain behind.
    pub fn cells_mut(&mut self) -> &mut [Vessel] {
        &mut self.cells
    }

    pub fn total_moles(&self, species: &SpeciesId) -> Moles {
        Moles(self.cells.iter().map(|cell| cell.moles_of(species).0).sum())
    }

    pub fn total_solute_charge(&self) -> f64 {
        self.cells.iter().map(|cell| cell.solute_charge).sum()
    }

    pub fn total_sensible_energy(&self) -> Joules {
        Joules(self.cells.iter().map(|cell| cell.enthalpy().0).sum())
    }

    /// Advance one explicit upwind step from a constant-composition reservoir.
    ///
    /// `inlet` is a representative boundary composition and is NOT consumed:
    /// it is scaled to exactly `courant_fraction` of one cell volume, whatever
    /// its own volume. This is the explicit `reservoir` boundary. A finite
    /// feed vessel that is drained as it flows uses
    /// [`CellChain::advance_from_feed`].
    pub fn advance(
        &mut self,
        inlet: &Vessel,
        courant_fraction: f64,
    ) -> Result<TransportStep, TransportError> {
        check_courant(courant_fraction)?;
        let cell_volume = self.uniform_cell_volume(true)?;
        let inlet = MobileParcel::from_vessel(inlet);
        inlet.validate("transport inlet")?;
        let inlet_volume = inlet.liquid_volume().0;
        if !inlet_volume.is_finite() || inlet_volume <= 0.0 {
            return Err(TransportError::InletVolume {
                expected_l: cell_volume,
                actual_l: inlet_volume,
            });
        }
        // A representative inlet within the hydraulic tolerance of one cell
        // volume IS one cell volume (the historical contract, unchanged);
        // any other volume is rescaled so the column still receives exactly
        // `courant_fraction` of a cell volume of that composition.
        let surface_reference_tolerance = self
            .cells
            .first()
            .map(surface_reference_volume)
            .unwrap_or(0.0);
        let scale = if same_volume(cell_volume, inlet_volume, surface_reference_tolerance) {
            courant_fraction
        } else {
            courant_fraction * (cell_volume / inlet_volume)
        };
        let injected = inlet.scaled(scale);
        self.advance_parcel(injected, courant_fraction)
    }

    /// Advance one explicit upwind step from a finite feed vessel.
    ///
    /// The injected parcel is `courant_fraction` of one cell volume taken
    /// from the feed's mobile (liquid + aqueous) inventory in proportion to
    /// its composition, and exactly that parcel is debited from `feed`. Solids
    /// and interfaces in the feed stay there. A feed that cannot supply the
    /// step (beyond [`HYDRAULIC_VOLUME_RELATIVE_TOLERANCE`]) is refused before
    /// anything mutates; `step` only labels the refusal.
    pub fn advance_from_feed(
        &mut self,
        feed: &mut Vessel,
        courant_fraction: f64,
        step: u32,
    ) -> Result<TransportStep, TransportError> {
        check_courant(courant_fraction)?;
        let cell_volume = self.uniform_cell_volume(true)?;
        let parcel = MobileParcel::from_vessel(feed);
        parcel.validate("transport inlet")?;
        let available = parcel.liquid_volume().0;
        let required = courant_fraction * cell_volume;
        if courant_fraction == 0.0 {
            return self.advance_parcel(parcel.scaled(0.0), courant_fraction);
        }
        if !available.is_finite()
            || available <= 0.0
            || required > available * (1.0 + HYDRAULIC_VOLUME_RELATIVE_TOLERANCE)
        {
            return Err(TransportError::InletExhausted {
                step,
                required_l: required,
                available_l: available,
            });
        }
        let fraction = (required / available).min(1.0);
        let injected = parcel.scaled(fraction);
        let result = self.advance_parcel(injected.clone(), courant_fraction)?;
        debit(feed, &injected, fraction >= 1.0);
        Ok(result)
    }

    fn advance_parcel(
        &mut self,
        injected: MobileParcel,
        courant_fraction: f64,
    ) -> Result<TransportStep, TransportError> {
        let outgoing: Vec<MobileParcel> = self
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                let parcel = MobileParcel::from_vessel(cell);
                parcel.validate(format!("transport cell {index}"))?;
                Ok(parcel.scaled(courant_fraction))
            })
            .collect::<Result<_, TransportError>>()?;
        let effluent = outgoing
            .last()
            .cloned()
            .expect("uniform_cell_volume rejects an empty chain");

        if courant_fraction == 0.0 {
            return Ok(TransportStep {
                courant_fraction,
                injected,
                effluent,
            });
        }

        // Debit exactly the parcel that leaves, so `old = kept + outgoing`
        // holds portion by portion rather than through a second rounding.
        for (cell, out) in self.cells.iter_mut().zip(&outgoing) {
            debit(cell, out, courant_fraction >= 1.0);
        }

        for index in 0..self.cells.len() {
            let incoming = if index == 0 {
                &injected
            } else {
                &outgoing[index - 1]
            };
            let cell = &mut self.cells[index];
            if matches!(cell.thermal_mode, ThermalMode::Adiabatic) {
                let settled = adiabatic_mix_into(cell, incoming.temperature, |t| {
                    incoming.energy_between(incoming.temperature.0, t)
                });
                cell.temperature = settled;
            }
            for portion in &incoming.contents {
                cell.deposit(portion.species.clone(), portion.moles, portion.phase);
            }
            cell.solute_charge += incoming.solute_charge;
            cell.free_hydroxide += incoming.free_hydroxide;
            cell.free_proton += incoming.free_proton;
        }

        Ok(TransportStep {
            courant_fraction,
            injected,
            effluent,
        })
    }

    /// Transport mobile matter, then equilibrate every stationary cell.
    ///
    /// This is first-order operator splitting: the conservative AQ-011 step
    /// runs once, then the supplied local solver sees cells from inlet to
    /// outlet. If any solve fails, the complete chain is restored to its
    /// pre-step state; solver-internal caches are outside this state contract.
    pub fn advance_reactive<E: Equilibrator + ?Sized>(
        &mut self,
        inlet: &Vessel,
        courant_fraction: f64,
        equilibrator: &mut E,
    ) -> Result<ReactiveTransportStep, ReactiveTransportError> {
        let before = self.cells.clone();
        let transport = self.advance(inlet, courant_fraction)?;
        let mut reactions = Vec::with_capacity(self.cells.len());

        for (cell, vessel) in self.cells.iter_mut().enumerate() {
            match equilibrator.equilibrate(vessel) {
                Ok(events) => reactions.push(CellReaction { cell, events }),
                Err(source) => {
                    self.cells = before;
                    return Err(ReactiveTransportError::Reaction { cell, source });
                }
            }
        }

        Ok(ReactiveTransportStep {
            transport,
            reactions,
        })
    }

    fn uniform_cell_volume(
        &self,
        allow_surface_reference_transfer: bool,
    ) -> Result<f64, TransportError> {
        let Some(first) = self.cells.first() else {
            return Err(TransportError::EmptyChain);
        };
        if !matches!(first.thermal_mode, ThermalMode::Adiabatic) {
            return Err(TransportError::ThermostattedCell { cell: 0 });
        }
        let expected = hydraulic_liquid_volume(first).0;
        if !expected.is_finite() || expected <= 0.0 {
            return Err(TransportError::InvalidCellVolume {
                cell: 0,
                volume_l: expected,
            });
        }
        for (index, cell) in self.cells.iter().enumerate().skip(1) {
            if !matches!(cell.thermal_mode, ThermalMode::Adiabatic) {
                return Err(TransportError::ThermostattedCell { cell: index });
            }
            let actual = hydraulic_liquid_volume(cell).0;
            if !actual.is_finite() || actual <= 0.0 {
                return Err(TransportError::InvalidCellVolume {
                    cell: index,
                    volume_l: actual,
                });
            }
            let surface_reference_tolerance = if allow_surface_reference_transfer {
                surface_reference_volume(first) + surface_reference_volume(cell)
            } else {
                0.0
            };
            if !same_volume(expected, actual, surface_reference_tolerance) {
                return Err(TransportError::NonUniformCellVolume {
                    cell: index,
                    expected_l: expected,
                    actual_l: actual,
                });
            }
        }
        Ok(expected)
    }
}

fn check_courant(courant_fraction: f64) -> Result<(), TransportError> {
    if !courant_fraction.is_finite() || !(0.0..=1.0).contains(&courant_fraction) {
        return Err(TransportError::InvalidCourant {
            fraction: courant_fraction,
        });
    }
    Ok(())
}

/// Remove exactly `parcel` (a scaled copy of `vessel`'s own mobile phase)
/// from `vessel`. With `whole`, the mobile phase leaves completely and no
/// rounding residue is left behind. Measured solution state is stale.
fn debit(vessel: &mut Vessel, parcel: &MobileParcel, whole: bool) {
    for portion in &mut vessel.contents {
        if !is_mobile(portion.phase) {
            continue;
        }
        if whole {
            portion.moles = Moles(0.0);
            continue;
        }
        let leaving: f64 = parcel
            .contents
            .iter()
            .filter(|p| p.species == portion.species && p.phase == portion.phase)
            .map(|p| p.moles.0)
            .sum();
        portion.moles = Moles((portion.moles.0 - leaving).max(0.0));
    }
    vessel
        .contents
        .retain(|portion| !is_mobile(portion.phase) || portion.moles.0 > 0.0);
    if whole {
        vessel.solute_charge = 0.0;
        vessel.free_hydroxide = 0.0;
        vessel.free_proton = 0.0;
    } else {
        vessel.solute_charge -= parcel.solute_charge;
        vessel.free_hydroxide = (vessel.free_hydroxide - parcel.free_hydroxide).max(0.0);
        vessel.free_proton = (vessel.free_proton - parcel.free_proton).max(0.0);
    }
    vessel.solution = None;
    vessel.resolved.invalidate();
}

fn is_mobile(phase: Phase) -> bool {
    matches!(phase, Phase::Liquid | Phase::Aqueous)
}

/// Carrier-liquid volume, excluding water currently released from stationary
/// surface reference sites. That water is mobile matter, but its matching
/// deficit remains on the interface ledger; it must not look like a change in
/// the fixed hydraulic cell geometry.
fn hydraulic_liquid_volume(vessel: &Vessel) -> Liters {
    let released_water = Moles(
        vessel
            .surfaces
            .iter()
            .map(|surface| surface.water_release.0)
            .sum(),
    );
    let released_volume = species::lookup_key("water")
        .map(|water| water.liters_from_moles(released_water).0)
        .unwrap_or(0.0);
    Liters(vessel.liquid_volume().0 - released_volume)
}

/// Maximum solvent volume that a cell's finite surface sites can exchange
/// with the mobile phase while moving between neutral reference states.
fn surface_reference_volume(vessel: &Vessel) -> f64 {
    let capacity = Moles(
        vessel
            .surfaces
            .iter()
            .map(|surface| surface.strong_capacity.0 + surface.weak_capacity.0)
            .sum(),
    );
    species::lookup_key("water")
        .map(|water| water.liters_from_moles(capacity).0)
        .unwrap_or(0.0)
}

fn same_volume(expected: f64, actual: f64, additional_absolute_tolerance_l: f64) -> bool {
    (expected - actual).abs()
        <= (expected.abs() * HYDRAULIC_VOLUME_RELATIVE_TOLERANCE).max(VOLUME_ABSOLUTE_TOLERANCE_L)
            + additional_absolute_tolerance_l
}
