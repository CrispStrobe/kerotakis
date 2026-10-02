//! Authoritative material-holding spill compartments.

use serde::{Deserialize, Serialize};

use crate::authority::SpillDestination;
use crate::units::Kelvin;
use crate::vessel::{Portion, UnresolvedMaterialPortion, Vessel, VesselId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpillCompartment {
    pub destination: SpillDestination,
    pub contents: Vec<Portion>,
    pub unresolved_materials: Vec<UnresolvedMaterialPortion>,
    pub temperature: Kelvin,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unpriced_heat: Vec<crate::SpeciesId>,
    #[serde(default)]
    pub sources: Vec<VesselId>,
}

impl SpillCompartment {
    pub fn new(destination: SpillDestination, temperature: Kelvin) -> Self {
        Self {
            destination,
            contents: Vec::new(),
            unresolved_materials: Vec::new(),
            temperature,
            unpriced_heat: Vec::new(),
            sources: Vec::new(),
        }
    }

    /// Read-only shape consumed by the existing safety-screen contract.
    pub fn as_vessel_probe(&self) -> Vessel {
        let mut probe = Vessel::new(VesselId(usize::MAX), "spill");
        probe.contents = self.contents.clone();
        probe.unresolved_materials = self.unresolved_materials.clone();
        probe.temperature = self.temperature;
        probe.unpriced_heat = self.unpriced_heat.clone();
        probe.refresh_pressure();
        probe
    }
}
