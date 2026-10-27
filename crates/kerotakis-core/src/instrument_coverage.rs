//! One shared coverage disclosure for instruments: for the species in the
//! sample, which the method measures (`supported`), which it would measure
//! but has no reviewed parameter for (`missing_parameters`), and which lie
//! outside the method (`outside_method`). A reading is only complete when
//! `missing_parameters` is empty; an instrument with a non-empty list
//! refuses with a typed not-modelled reason instead of reporting a partial
//! value as the whole.
//!
//! This is the shape #829 gave chromatography (peaks / missing neutral
//! parameters / ions outside the method). `disclosure_of` enumerates which
//! instruments report through this type and which still carry their own
//! equivalent, so the remaining migrations are visible rather than implied.

/// Coverage of one instrument reading over one sample.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstrumentCoverage {
    pub instrument: &'static str,
    pub supported: Vec<String>,
    pub missing_parameters: Vec<String>,
    pub outside_method: Vec<String>,
}

impl InstrumentCoverage {
    pub fn new(
        instrument: &'static str,
        mut supported: Vec<String>,
        mut missing_parameters: Vec<String>,
        mut outside_method: Vec<String>,
    ) -> Self {
        for list in [&mut supported, &mut missing_parameters, &mut outside_method] {
            list.sort();
            list.dedup();
        }
        Self {
            instrument,
            supported,
            missing_parameters,
            outside_method,
        }
    }

    /// A complete reading needs every in-method species parameterised.
    pub fn is_complete(&self) -> bool {
        self.missing_parameters.is_empty()
    }
}

/// How each instrument discloses coverage today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disclosure {
    /// Reports through [`InstrumentCoverage`].
    Shared,
    /// Has its own supported/missing/outside split, not yet migrated.
    OwnEquivalent,
    /// Measures a bulk property every sample has; no species coverage.
    NotSpeciesResolved,
}

/// Every bench instrument and its coverage disclosure, 2026-10-27. The
/// match is exhaustive on purpose: a new instrument does not compile until
/// it says how it discloses coverage.
pub fn disclosure_of(instrument: crate::Instrument) -> (Disclosure, &'static str) {
    use crate::Instrument as I;
    match instrument {
        I::Spectrophotometer => (
            Disclosure::Shared,
            "solution_optics::disclosure: registry spectra and indicator tables supported; coloured species without reviewed molar absorptivity, missing native spectra and missing pH are missing parameters; colourless or unclassified species are outside the visible method",
        ),
        I::Chromatograph => (
            Disclosure::OwnEquivalent,
            "Event::Chromatographed peaks / missing neutral parameters (warning) / outside_method ions (#829); migration to InstrumentCoverage pending",
        ),
        I::PhMeter | I::ConductivityMeter => (
            Disclosure::OwnEquivalent,
            "refuses without a solved aqueous solution; no per-species split yet",
        ),
        I::Eyes => (
            Disclosure::OwnEquivalent,
            "appearance spectral_gaps / optical_ph_gaps notes",
        ),
        I::MeltingPointApparatus | I::BoilingPointApparatus | I::Densitometer => (
            Disclosure::OwnEquivalent,
            "pure-sample admission with typed refusal for mixtures or missing curated data",
        ),
        I::Thermometer
        | I::Balance
        | I::PressureGauge
        | I::VolumeMeter
        | I::Calorimeter
        | I::GeigerCounter => (Disclosure::NotSpeciesResolved, "bulk property"),
    }
}
