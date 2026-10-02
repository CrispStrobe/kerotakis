//! Neutral-partition column predictions with explicit sample coverage.
//!
//! Missing retention data must remain visible even when another component
//! produces a modeled peak. Ions outside this method and neutral species
//! outside the available model are deliberately separate inventories.

use std::collections::{BTreeMap, BTreeSet};

use crate::instrument::{curated_partition_k, ChromatographyColumn, PaperPlate};
use crate::ops::ElutedPeak;
use crate::species::{Phase, SpeciesId};
use crate::vessel::Vessel;

#[derive(Debug, Clone)]
pub struct ChromatographySample {
    /// Modeled peaks only; areas are relative to the largest modeled peak.
    pub peaks: Vec<ElutedPeak>,
    /// Ions that this neutral-partition method does not separate.
    pub outside_method: Vec<SpeciesId>,
    /// Neutral sample components with no usable retention prediction.
    pub unparameterised: Vec<SpeciesId>,
}

pub fn sample(vessel: &Vessel) -> ChromatographySample {
    let mut injectable = BTreeMap::<SpeciesId, f64>::new();
    let mut outside = BTreeSet::new();
    let mut unparameterised = BTreeSet::new();
    for portion in &vessel.contents {
        let dissolved = portion.phase == Phase::Aqueous
            || (portion.phase == Phase::Liquid && portion.species.0 != "water");
        if !dissolved || portion.moles.0 <= 0.0 {
            continue;
        }
        if crate::bench::partition_groups(&portion.species).is_some()
            || curated_partition_k(&portion.species.0).is_some()
        {
            *injectable.entry(portion.species.clone()).or_default() += portion.moles.0;
        } else if crate::stoich::parse_formula(&portion.species.0)
            .is_ok_and(|formula| formula.charge != 0.0)
        {
            outside.insert(portion.species.clone());
        } else {
            unparameterised.insert(portion.species.clone());
        }
    }
    let column = ChromatographyColumn::school();
    let plate = PaperPlate::school();
    let mut peaks = Vec::new();
    for (species, moles) in injectable {
        let k = if let Some(solute) = crate::bench::partition_groups(&species) {
            kerotakis_thermo::lle::infinite_dilution_gamma(
                &solute,
                &crate::bench::water_groups(),
                vessel.temperature.0,
            ) / kerotakis_thermo::lle::infinite_dilution_gamma(
                &solute,
                &crate::bench::hexane_groups(),
                vessel.temperature.0,
            )
        } else {
            curated_partition_k(&species.0)
                .expect("selected curated parameter")
                .0
        };
        let retention = column.retention_time(k);
        let width = column.peak_width(retention);
        let rf = plate.rf(k);
        if !moles.is_finite()
            || !k.is_finite()
            || k < 0.0
            || !retention.is_finite()
            || !width.is_finite()
            || !rf.is_finite()
        {
            unparameterised.insert(species);
            continue;
        }
        peaks.push(ElutedPeak {
            species,
            retention_time_s: retention,
            width_s: width,
            relative_area: moles,
            partition_k: k,
            rf,
        });
    }
    peaks.sort_by(|a, b| a.retention_time_s.total_cmp(&b.retention_time_s));
    let largest = peaks
        .iter()
        .map(|peak| peak.relative_area)
        .fold(0.0_f64, f64::max);
    for peak in &mut peaks {
        peak.relative_area /= largest;
    }
    ChromatographySample {
        peaks,
        outside_method: outside.into_iter().collect(),
        unparameterised: unparameterised.into_iter().collect(),
    }
}
