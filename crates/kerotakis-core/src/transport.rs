//! Conservative one-dimensional transport before reactive coupling.
//!
//! A transport cell is an ordinary [`Vessel`]. Liquid and aqueous portions
//! move; solids, headspaces, surfaces, exchangers, and solid solutions remain
//! owned by their cell. One step is an explicit first-order upwind update with
//! a Courant fraction in `[0, 1]`. Every outflow is snapshotted before any cell
//! mutates, so the result does not depend on iteration order.
//!
//! [`CellChain::advance`] deliberately performs no chemistry and invalidates
//! stale solution metadata after matter moves. [`CellChain::advance_reactive`]
//! adds the operator-splitting seam: transport first, then local equilibrium,
//! with whole-chain rollback if any cell solve fails.

use serde::{Deserialize, Serialize};

use crate::ops::Event;
use crate::solve::{adiabatic_mix_temperature, Equilibrator, SolveError};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Liters, Moles};
use crate::vessel::{Portion, ThermalMode, Vessel};

const VOLUME_RELATIVE_TOLERANCE: f64 = 1e-9;
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
    #[error("{location} contains a non-finite temperature, charge, or mobile amount")]
    InvalidMobileState { location: String },
    #[error("transport cell {cell} is thermostatted; conservative AQ-011 transport requires adiabatic cells")]
    ThermostattedCell { cell: usize },
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

    pub fn heat_capacity(&self) -> f64 {
        self.contents
            .iter()
            .filter_map(|portion| {
                species::lookup(&portion.species).map(|data| portion.moles.0 * data.heat_capacity)
            })
            .sum()
    }

    pub fn sensible_energy(&self) -> Joules {
        Joules(self.heat_capacity() * (self.temperature.0 - Kelvin::STANDARD.0))
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
        }
    }

    fn validate(&self, location: impl Into<String>) -> Result<(), TransportError> {
        if !self.temperature.0.is_finite()
            || self.temperature.0 <= 0.0
            || !self.solute_charge.is_finite()
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
        chain.uniform_cell_volume()?;
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

    /// Advance one explicit upwind step.
    ///
    /// `inlet` represents one complete cell volume at the boundary
    /// composition. `courant_fraction` is the fraction of each cell volume
    /// replaced during this step. The update is simultaneous and keeps cell
    /// liquid volumes fixed when the validated uniform-volume contract holds.
    pub fn advance(
        &mut self,
        inlet: &Vessel,
        courant_fraction: f64,
    ) -> Result<TransportStep, TransportError> {
        if !courant_fraction.is_finite() || !(0.0..=1.0).contains(&courant_fraction) {
            return Err(TransportError::InvalidCourant {
                fraction: courant_fraction,
            });
        }

        let cell_volume = self.uniform_cell_volume()?;
        let inlet = MobileParcel::from_vessel(inlet);
        inlet.validate("transport inlet")?;
        let inlet_volume = inlet.liquid_volume().0;
        if !same_volume(cell_volume, inlet_volume) {
            return Err(TransportError::InletVolume {
                expected_l: cell_volume,
                actual_l: inlet_volume,
            });
        }

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
        let injected = inlet.scaled(courant_fraction);
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

        for cell in &mut self.cells {
            for portion in &mut cell.contents {
                if is_mobile(portion.phase) {
                    portion.moles = Moles(portion.moles.0 * (1.0 - courant_fraction));
                }
            }
            cell.contents.retain(|portion| portion.moles.0 > 1e-15);
            cell.solute_charge *= 1.0 - courant_fraction;
            cell.solution = None;
        }

        for index in 0..self.cells.len() {
            let incoming = if index == 0 {
                &injected
            } else {
                &outgoing[index - 1]
            };
            let cell = &mut self.cells[index];
            if matches!(cell.thermal_mode, ThermalMode::Adiabatic) {
                cell.temperature = adiabatic_mix_temperature(
                    cell.temperature,
                    cell.heat_capacity(),
                    incoming.temperature,
                    incoming.heat_capacity(),
                );
            }
            for portion in &incoming.contents {
                cell.deposit(portion.species.clone(), portion.moles, portion.phase);
            }
            cell.solute_charge += incoming.solute_charge;
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
    pub fn advance_reactive<E: Equilibrator>(
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

    fn uniform_cell_volume(&self) -> Result<f64, TransportError> {
        let Some(first) = self.cells.first() else {
            return Err(TransportError::EmptyChain);
        };
        if !matches!(first.thermal_mode, ThermalMode::Adiabatic) {
            return Err(TransportError::ThermostattedCell { cell: 0 });
        }
        let expected = first.liquid_volume().0;
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
            let actual = cell.liquid_volume().0;
            if !actual.is_finite() || actual <= 0.0 {
                return Err(TransportError::InvalidCellVolume {
                    cell: index,
                    volume_l: actual,
                });
            }
            if !same_volume(expected, actual) {
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

fn is_mobile(phase: Phase) -> bool {
    matches!(phase, Phase::Liquid | Phase::Aqueous)
}

fn same_volume(expected: f64, actual: f64) -> bool {
    (expected - actual).abs()
        <= (expected.abs() * VOLUME_RELATIVE_TOLERANCE).max(VOLUME_ABSOLUTE_TOLERANCE_L)
}
