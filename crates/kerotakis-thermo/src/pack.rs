//! BRD-031: the cleared fluid parameter pack, and the identity seam that
//! binds one of its rows to a substance.
//!
//! ## What this module is for
//!
//! `vle.rs` holds vapour-pressure correlations as free constants — `WATER`,
//! `ETHANOL`, `METHANOL` — and a caller picks one by naming the Rust item.
//! That is fine inside the crate and useless at the boundary: nothing binds
//! `vle::WATER` to the registry's water, so a bench that wanted "the
//! correlation for whatever is in this beaker" had to get there by matching
//! a display name or by knowing a position in an array. BRD-030's spike
//! recorded that positional/name-shaped seam as the thing to fix before
//! BRD-032 routes anything, and this module is the fix:
//!
//! * every row carries a [`ComponentIdentity`] whose **join key is the
//!   Standard InChIKey**. `species_key` rides along for display and for the
//!   runtime's own tables, but [`row_by_inchikey`] is the only lookup this
//!   module offers, so there is no path from a name to a correlation;
//! * every cleared number carries its own [`ParameterProvenance`] —
//!   publication, locator, licence, rights lane and the date the rights
//!   position was recorded — with one provenance per correlation *segment*,
//!   because a piecewise fit is two sources, not one;
//! * every parameter that is **not** cleared is present as a named
//!   [`ParameterGap`] rather than absent, so asking for it produces a
//!   refusal that says which audit recorded the block.
//!
//! ## What this module does not do
//!
//! It clears nothing new. Every correlation here was already reviewed and
//! shipped in `vle.rs`; what is new is that the rights position of each one
//! is now written down, machine-checkable, and attached to an identity.
//! In particular:
//!
//! * **no PC-SAFT, ePC-SAFT or other residual-EOS parameter is present.**
//!   `provenance/brd-031-pilot-source-audit.md` recorded a `no-go` on the
//!   two candidate parameter repositories because neither states a
//!   path-level data right, and that decision stands. Every row says so.
//! * **no liquid-density correlation is present.** The bench's per-species
//!   density is a single value near 25 °C, not a `ρ(T, P)` model, and no
//!   source for one has cleared the gate. Asking for it refuses by name.
//! * a [`RightsLane::PrimaryLiteratureCoefficientsPendingReview`] row is
//!   **not** a cleared row. The lane exists to make the outstanding
//!   question visible per row instead of invisible, and a reader who treats
//!   it as permission has misread it.

use crate::vle::{Antoine, VapourPressure};
use std::fmt;

/// How a parameter's numbers may be redistributed — recorded, never
/// inferred from the licence of the software that happened to carry them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RightsLane {
    /// The numerical values themselves carry an explicit open licence, and
    /// [`ParameterProvenance::licence`] names it as an SPDX identifier.
    OpenLicensedData,
    /// A handful of correlation coefficients cited exactly to the primary
    /// publication and reviewed one at a time.
    ///
    /// The publication grants nothing by being citable. This lane records
    /// that fact for the rows this repository already carries rather than
    /// hiding it behind an empty licence field; an independent rights
    /// review still owes these rows an answer.
    PrimaryLiteratureCoefficientsPendingReview,
}

impl RightsLane {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenLicensedData => "open-licensed-data",
            Self::PrimaryLiteratureCoefficientsPendingReview => {
                "primary-literature-coefficients-pending-review"
            }
        }
    }
}

/// SPDX identifiers a [`RightsLane::OpenLicensedData`] row may claim.
///
/// The same list `provenance/brd-031-pilot-source-audit.md` names as the
/// only terms that clear promotion. A row claiming the open lane under
/// anything else is a lint failure, not a judgement call at the call site.
pub const OPEN_DATA_LICENCES: &[&str] = &[
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "CC-BY-4.0",
    "CC0-1.0",
    "LicenseRef-US-Public-Domain",
    "MIT",
];

/// Where one number came from and under what terms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParameterProvenance {
    /// The original publication or dataset, cited exactly enough to find
    /// the table again.
    pub source: &'static str,
    /// A stable locator for that publication — a DOI URL by preference.
    pub locator: &'static str,
    /// SPDX identifier, or a `LicenseRef-` name for a lane that has none.
    pub licence: &'static str,
    /// Which rights lane the numbers sit in.
    pub lane: RightsLane,
    /// ISO-8601 date on which this rights position was recorded.
    pub recorded: &'static str,
}

/// A model-neutral component identity.
///
/// The InChIKey is the join. `species_key` is the runtime registry key and
/// `name` is for prose; neither is allowed to select a row, which is why
/// this module exposes no lookup that takes either.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentIdentity {
    /// Standard InChIKey, `XXXXXXXXXXXXXX-YYYYYYYYYY-Z`.
    pub inchikey: &'static str,
    /// The runtime registry key this identity resolves to.
    pub species_key: &'static str,
    /// Display name, for messages only.
    pub name: &'static str,
}

/// The parameters a fluid row can carry, cleared or not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FluidParameter {
    /// Saturation (vapour) pressure as a function of temperature.
    SaturationPressure,
    /// Saturated-liquid density as a function of temperature.
    LiquidDensity,
    /// Residual equation-of-state parameters — PC-SAFT segment number,
    /// segment diameter, dispersion energy, association terms.
    ResidualEos,
}

impl FluidParameter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SaturationPressure => "saturation pressure",
            Self::LiquidDensity => "liquid density",
            Self::ResidualEos => "residual equation of state",
        }
    }
}

/// A parameter this row deliberately does not carry, and the audit that
/// recorded why. A gap is data: it is what `explain` says instead of a
/// number, and it is what a future clearance has to discharge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParameterGap {
    pub parameter: FluidParameter,
    /// One sentence, in the terms of the audit rather than of the physics.
    pub reason: &'static str,
    /// The checked-in record that decided it.
    pub audit: &'static str,
}

/// A saturation-pressure correlation with one provenance per segment.
///
/// The pairing is positional and the lint enforces the arity, because a
/// two-segment piecewise fit whose second segment came from a different
/// paper under a different licence is exactly the case a single `source`
/// string loses.
#[derive(Clone, Copy, Debug)]
pub struct ClearedVapourPressure {
    pub correlation: VapourPressure,
    /// In segment order, same length as `correlation.segments()`.
    pub provenance: &'static [ParameterProvenance],
}

impl ClearedVapourPressure {
    /// Provenance for the segment that owns `t_celsius`, if any does.
    pub fn provenance_at(&self, t_celsius: f64) -> Option<&'static ParameterProvenance> {
        let provenance: &'static [ParameterProvenance] = self.provenance;
        self.correlation
            .segments()
            .iter()
            .position(|segment| t_celsius >= segment.valid_c.0 && t_celsius <= segment.valid_c.1)
            .and_then(|index| provenance.get(index))
    }
}

/// One pure fluid, as the pack knows it.
#[derive(Clone, Copy, Debug)]
pub struct FluidRow {
    pub identity: ComponentIdentity,
    /// `None` where no correlation has been cleared for this fluid; the
    /// matching [`ParameterGap`] then says why.
    pub vapour_pressure: Option<ClearedVapourPressure>,
    /// Everything this row does not carry.
    pub gaps: &'static [ParameterGap],
}

/// Why the pack declined to answer. Every variant names the fluid and the
/// parameter, so a refusal can be rendered without the caller re-deriving
/// what it asked for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PackRefusal {
    /// No row carries this identity at all.
    UnknownIdentity { parameter: FluidParameter },
    /// The row exists; the parameter is not cleared.
    NotCleared {
        inchikey: &'static str,
        parameter: FluidParameter,
        reason: &'static str,
        audit: &'static str,
    },
    /// The parameter is cleared, and the request is outside the interval
    /// it was fitted over. Extrapolating a local correlation is how a
    /// plausible wrong number gets made.
    OutsideValidity {
        inchikey: &'static str,
        parameter: FluidParameter,
        requested_c: f64,
        valid_c: (f64, f64),
    },
    /// The parameter is cleared and in range, and the numerics found no
    /// solution. Distinct from every case above on purpose.
    NoSolution {
        inchikey: &'static str,
        parameter: FluidParameter,
    },
}

impl PackRefusal {
    /// The parameter that was asked for, whatever the reason.
    pub fn parameter(&self) -> FluidParameter {
        match self {
            Self::UnknownIdentity { parameter }
            | Self::NotCleared { parameter, .. }
            | Self::OutsideValidity { parameter, .. }
            | Self::NoSolution { parameter, .. } => *parameter,
        }
    }
}

impl fmt::Display for PackRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownIdentity { parameter } => write!(
                f,
                "no cleared fluid row carries this identity, so {} is unavailable",
                parameter.as_str()
            ),
            Self::NotCleared {
                inchikey,
                parameter,
                reason,
                audit,
            } => write!(
                f,
                "{}: {} is not cleared — {reason} (see {audit})",
                inchikey,
                parameter.as_str()
            ),
            Self::OutsideValidity {
                inchikey,
                parameter,
                requested_c,
                valid_c,
            } => write!(
                f,
                "{}: {} is cleared only over {:.2}..{:.2} °C, and {requested_c:.2} °C is outside it",
                inchikey,
                parameter.as_str(),
                valid_c.0,
                valid_c.1
            ),
            Self::NoSolution {
                inchikey,
                parameter,
            } => write!(
                f,
                "{}: {} has no numerical solution here",
                inchikey,
                parameter.as_str()
            ),
        }
    }
}

impl std::error::Error for PackRefusal {}

impl FluidRow {
    /// The recorded gap for one parameter, if this row declares one.
    pub fn gap(&self, parameter: FluidParameter) -> Option<&'static ParameterGap> {
        let gaps: &'static [ParameterGap] = self.gaps;
        gaps.iter().find(|gap| gap.parameter == parameter)
    }

    fn refuse(&self, parameter: FluidParameter) -> PackRefusal {
        match self.gap(parameter) {
            Some(gap) => PackRefusal::NotCleared {
                inchikey: self.identity.inchikey,
                parameter,
                reason: gap.reason,
                audit: gap.audit,
            },
            // A row with neither a value nor a gap is a lint failure, not a
            // runtime condition; refuse anyway rather than answer.
            None => PackRefusal::NotCleared {
                inchikey: self.identity.inchikey,
                parameter,
                reason: "this row declares neither a cleared value nor a gap",
                audit: "crates/kerotakis-thermo/tests/fluid_pack.rs",
            },
        }
    }

    /// Saturation pressure in kPa, refusing outside the fitted interval.
    pub fn saturation_pressure_kpa(&self, t_celsius: f64) -> Result<f64, PackRefusal> {
        let cleared = self
            .vapour_pressure
            .as_ref()
            .ok_or_else(|| self.refuse(FluidParameter::SaturationPressure))?;
        if !t_celsius.is_finite() {
            return Err(PackRefusal::NoSolution {
                inchikey: self.identity.inchikey,
                parameter: FluidParameter::SaturationPressure,
            });
        }
        cleared
            .correlation
            .pressure_kpa(t_celsius)
            .filter(|kpa| kpa.is_finite() && *kpa > 0.0)
            .ok_or_else(|| self.outside_saturation_validity(t_celsius))
    }

    fn outside_saturation_validity(&self, t_celsius: f64) -> PackRefusal {
        match self
            .vapour_pressure
            .as_ref()
            .and_then(|cleared| cleared.correlation.valid_range())
        {
            Some(valid_c) => PackRefusal::OutsideValidity {
                inchikey: self.identity.inchikey,
                parameter: FluidParameter::SaturationPressure,
                requested_c: t_celsius,
                valid_c,
            },
            None => PackRefusal::NoSolution {
                inchikey: self.identity.inchikey,
                parameter: FluidParameter::SaturationPressure,
            },
        }
    }

    /// Saturated-liquid density in g/mL. Nothing is cleared, so this
    /// always refuses; it exists so the refusal is named rather than the
    /// question being unaskable.
    pub fn liquid_density_g_per_ml(&self, _t_celsius: f64) -> Result<f64, PackRefusal> {
        Err(self.refuse(FluidParameter::LiquidDensity))
    }
}

/// Look a row up by Standard InChIKey. The only lookup this module has.
pub fn row_by_inchikey(inchikey: &str) -> Option<&'static FluidRow> {
    CLEARED_FLUIDS
        .iter()
        .find(|row| row.identity.inchikey == inchikey)
}

/// Every row, in InChIKey order.
pub fn rows() -> &'static [FluidRow] {
    CLEARED_FLUIDS
}

// ---------------------------------------------------------------------------
// Provenance records
// ---------------------------------------------------------------------------

/// The date this pack recorded each row's rights position. Not the date the
/// coefficients were first put in the tree, which is older, and not a claim
/// that anything was re-derived on it.
const RECORDED: &str = "2026-09-05";

const AUDIT: &str = "provenance/brd-031-pilot-source-audit.md";

/// Stull's 1947 vapour-pressure compilation, which is where five of this
/// repository's six shipped Antoine sets come from.
const fn stull_1947(table: &'static str) -> ParameterProvenance {
    ParameterProvenance {
        source: table,
        locator: "https://doi.org/10.1021/ie50448a022",
        licence: "LicenseRef-Primary-Literature-Coefficients",
        lane: RightsLane::PrimaryLiteratureCoefficientsPendingReview,
        recorded: RECORDED,
    }
}

const WATER_PROVENANCE: &[ParameterProvenance] = &[stull_1947(
    "Stull, D. R., Ind. Eng. Chem. 39(4), 517-540 (1947), Table I — water, \
     log10(P/mmHg) = 8.07131 - 1730.63/(T/°C + 233.426), 1-100 °C",
)];

const ETHANOL_PROVENANCE: &[ParameterProvenance] = &[
    stull_1947(
        "Stull, D. R., Ind. Eng. Chem. 39(4), 517-540 (1947), Table I — ethanol, \
         log10(P/mmHg) = 8.20417 - 1642.89/(T/°C + 230.300), -57 to 80 °C",
    ),
    ParameterProvenance {
        source: "Susial Badajoz, P., Garcia Montesdeoca, I., & Santiago, D. E., \
                 ACS Omega 11, 48295-48312 (2026) — measured pure-ethanol \
                 saturation pressures over 107-1015 kPa, fitted as \
                 log10(P/kPa) = 6.99161 - 1460.701/(T/K - 58.477)",
        locator: "https://doi.org/10.1021/acsomega.6c04827",
        licence: "CC-BY-4.0",
        lane: RightsLane::OpenLicensedData,
        recorded: RECORDED,
    },
];

const METHANOL_PROVENANCE: &[ParameterProvenance] = &[stull_1947(
    "Stull, D. R., Ind. Eng. Chem. 39(4), 517-540 (1947), Table I — methanol, \
     log10(P/mmHg) = 8.08097 - 1582.271/(T/°C + 239.726), 15-84 °C",
)];

const PROPANONE_PROVENANCE: &[ParameterProvenance] = &[stull_1947(
    "Stull, D. R., Ind. Eng. Chem. 39(4), 517-540 (1947), Table I — propanone, \
     log10(P/mmHg) = 7.02447 - 1161.0/(T/°C + 224.0), -20 to 77 °C",
)];

const ETHANOIC_ACID_PROVENANCE: &[ParameterProvenance] = &[stull_1947(
    "Stull, D. R., Ind. Eng. Chem. 39(4), 517-540 (1947), Table I — ethanoic acid, \
     log10(P/mmHg) = 7.38782 - 1533.313/(T/°C + 222.309), 17-157 °C",
)];

/// Isopropanol is the one shipped set whose in-tree citation names a
/// *rendering* of Stull rather than Stull: the coefficients were taken in
/// the NIST WebBook's bar/kelvin form. The primary publication is the same
/// one, and the audit rejects NIST WebBook as a source class, so this row
/// is recorded with the primary citation and the detour written down.
const ISOPROPANOL_PROVENANCE: &[ParameterProvenance] = &[ParameterProvenance {
    source: "Stull, D. R., Ind. Eng. Chem. 39(4), 517-540 (1947), Table I — \
             isopropyl alcohol, 329.92-362.41 K; the shipped coefficients were \
             transcribed from the NIST Chemistry WebBook (SRD 69) rendering of \
             that fit, log10(P/bar) = 4.8610 - 1357.427/(T/K - 75.814), and NIST \
             WebBook is a rejected source class in the BRD-031 audit",
    locator: "https://doi.org/10.1021/ie50448a022",
    licence: "LicenseRef-Primary-Literature-Coefficients",
    lane: RightsLane::PrimaryLiteratureCoefficientsPendingReview,
    recorded: RECORDED,
}];

// ---------------------------------------------------------------------------
// Gaps
// ---------------------------------------------------------------------------

const NO_RESIDUAL_EOS: ParameterGap = ParameterGap {
    parameter: FluidParameter::ResidualEos,
    reason: "the two candidate PC-SAFT parameter repositories carry a permissive \
             licence on their code and no path-level statement covering the \
             third-party-derived numerical tables, so neither is cleared",
    audit: AUDIT,
};

const NO_LIQUID_DENSITY: ParameterGap = ParameterGap {
    parameter: FluidParameter::LiquidDensity,
    reason: "no cleared rho(T) correlation exists for any fluid; the registry's \
             per-species density is a single reviewed value near 25 °C and is not \
             a temperature-dependent model",
    audit: AUDIT,
};

const fn no_saturation_pressure(reason: &'static str) -> ParameterGap {
    ParameterGap {
        parameter: FluidParameter::SaturationPressure,
        reason,
        audit: AUDIT,
    }
}

const CORRELATION_GAPS: &[ParameterGap] = &[NO_LIQUID_DENSITY, NO_RESIDUAL_EOS];

const PERMANENT_GAS_GAPS: &[ParameterGap] = &[
    no_saturation_pressure(
        "this fluid is above its critical temperature at bench conditions, so a \
         saturation pressure is not merely uncleared but undefined; a residual \
         equation of state is the model that answers here, and none is cleared",
    ),
    NO_LIQUID_DENSITY,
    NO_RESIDUAL_EOS,
];

const UNCLEARED_SOLVENT_GAPS: &[ParameterGap] = &[
    no_saturation_pressure(
        "no vapour-pressure correlation for this fluid has been reviewed into the \
         tree under a recorded rights lane",
    ),
    NO_LIQUID_DENSITY,
    NO_RESIDUAL_EOS,
];

// ---------------------------------------------------------------------------
// The pack
// ---------------------------------------------------------------------------

/// Every fluid the corpus and Kids Lab actually reach for, in InChIKey
/// order. A row with `vapour_pressure: None` is here on purpose: knowing
/// the identity and refusing by name is the whole point of the seam.
pub const CLEARED_FLUIDS: &[FluidRow] = &[
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "CSCPPACGZOOCGX-UHFFFAOYSA-N",
            species_key: "propanone",
            name: "propanone",
        },
        vapour_pressure: Some(ClearedVapourPressure {
            correlation: crate::vle::PROPANONE,
            provenance: PROPANONE_PROVENANCE,
        }),
        gaps: CORRELATION_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "CURLTUGMZLYLDI-UHFFFAOYSA-N",
            species_key: "CO2",
            name: "carbon dioxide",
        },
        vapour_pressure: None,
        gaps: PERMANENT_GAS_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "IJGRMHOSHXDMSA-UHFFFAOYSA-N",
            species_key: "N2",
            name: "nitrogen",
        },
        vapour_pressure: None,
        gaps: PERMANENT_GAS_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "KFZMGEQAYNKOFK-UHFFFAOYSA-N",
            species_key: "isopropanol",
            name: "isopropanol",
        },
        vapour_pressure: Some(ClearedVapourPressure {
            correlation: crate::vle::ISOPROPANOL,
            provenance: ISOPROPANOL_PROVENANCE,
        }),
        gaps: CORRELATION_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "LFQSCWFLJHTTHZ-UHFFFAOYSA-N",
            species_key: "ethanol",
            name: "ethanol",
        },
        vapour_pressure: Some(ClearedVapourPressure {
            correlation: crate::vle::ETHANOL,
            provenance: ETHANOL_PROVENANCE,
        }),
        gaps: CORRELATION_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "MYMOFIZGZYHOMD-UHFFFAOYSA-N",
            species_key: "O2",
            name: "oxygen",
        },
        vapour_pressure: None,
        gaps: PERMANENT_GAS_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "OKKJLVBELUTLKV-UHFFFAOYSA-N",
            species_key: "methanol",
            name: "methanol",
        },
        vapour_pressure: Some(ClearedVapourPressure {
            correlation: crate::vle::METHANOL,
            provenance: METHANOL_PROVENANCE,
        }),
        gaps: CORRELATION_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "QTBSBXVTEAMEQO-UHFFFAOYSA-N",
            species_key: "CH3COOH",
            name: "ethanoic acid",
        },
        vapour_pressure: Some(ClearedVapourPressure {
            correlation: crate::vle::ETHANOIC_ACID,
            provenance: ETHANOIC_ACID_PROVENANCE,
        }),
        gaps: CORRELATION_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "VLKZOEOYAKHREP-UHFFFAOYSA-N",
            species_key: "hexane",
            name: "hexane",
        },
        vapour_pressure: None,
        gaps: UNCLEARED_SOLVENT_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "XEKOWRVHYACXOJ-UHFFFAOYSA-N",
            species_key: "ethyl_acetate",
            name: "ethyl acetate",
        },
        vapour_pressure: None,
        gaps: UNCLEARED_SOLVENT_GAPS,
    },
    FluidRow {
        identity: ComponentIdentity {
            inchikey: "XLYOFNOQVPJJNP-UHFFFAOYSA-N",
            species_key: "water",
            name: "water",
        },
        vapour_pressure: Some(ClearedVapourPressure {
            correlation: crate::vle::WATER,
            provenance: WATER_PROVENANCE,
        }),
        gaps: CORRELATION_GAPS,
    },
];

// ---------------------------------------------------------------------------
// The lint
// ---------------------------------------------------------------------------

/// Everything a row must satisfy to sit in the pack.
///
/// A function rather than a test body so it can be run in both directions:
/// the checked-in pack must pass it, and a deliberately incomplete row must
/// fail it. That is the same shape as `kerotakis-data`'s `lint_promotion`,
/// and for the same reason — a gate only proves something if something is
/// shown being refused by it.
pub fn lint_row(row: &FluidRow) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    let at = row.identity.species_key;

    if !well_formed_inchikey(row.identity.inchikey) {
        problems.push(format!(
            "{at}: '{}' is not a Standard InChIKey",
            row.identity.inchikey
        ));
    }
    if row.identity.species_key.is_empty() {
        problems.push(format!("{at}: species_key is empty"));
    }
    if row.identity.name.is_empty() {
        problems.push(format!("{at}: name is empty"));
    }

    if let Some(cleared) = row.vapour_pressure.as_ref() {
        let segments = cleared.correlation.segments();
        if segments.len() != cleared.provenance.len() {
            problems.push(format!(
                "{at}: {} correlation segments carry {} provenance records",
                segments.len(),
                cleared.provenance.len()
            ));
        }
        if segments.is_empty() {
            problems.push(format!("{at}: cleared correlation has no segments"));
        }
        for (index, segment) in segments.iter().enumerate() {
            lint_segment(at, index, segment, &mut problems);
        }
        for (index, provenance) in cleared.provenance.iter().enumerate() {
            lint_provenance(at, index, provenance, &mut problems);
        }
        if row.gap(FluidParameter::SaturationPressure).is_some() {
            problems.push(format!(
                "{at}: saturation pressure is both cleared and declared a gap"
            ));
        }
    } else if row.gap(FluidParameter::SaturationPressure).is_none() {
        problems.push(format!(
            "{at}: saturation pressure is neither cleared nor declared a gap"
        ));
    }

    // Nothing may be silently absent. Density and the residual EOS are not
    // cleared for any fluid, so every row owes both a gap.
    for parameter in [FluidParameter::LiquidDensity, FluidParameter::ResidualEos] {
        if row.gap(parameter).is_none() {
            problems.push(format!(
                "{at}: {} is neither cleared nor declared a gap",
                parameter.as_str()
            ));
        }
    }
    for gap in row.gaps {
        if gap.reason.is_empty() {
            problems.push(format!(
                "{at}: the {} gap carries no reason",
                gap.parameter.as_str()
            ));
        }
        if gap.audit.is_empty() {
            problems.push(format!(
                "{at}: the {} gap names no audit record",
                gap.parameter.as_str()
            ));
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

fn lint_segment(at: &str, index: usize, segment: &Antoine, problems: &mut Vec<String>) {
    if segment.source.is_empty() {
        problems.push(format!("{at}: segment {index} carries no source"));
    }
    if !segment.a.is_finite() || !segment.b.is_finite() || !segment.c.is_finite() {
        problems.push(format!("{at}: segment {index} has a nonfinite coefficient"));
    }
    let (lo, hi) = segment.valid_c;
    if !lo.is_finite() || !hi.is_finite() || lo >= hi {
        problems.push(format!(
            "{at}: segment {index} has no usable validity range ({lo}..{hi})"
        ));
    } else {
        let mid = 0.5 * (lo + hi);
        match segment.pressure_kpa(mid) {
            Some(kpa) if kpa.is_finite() && kpa > 0.0 => {}
            _ => problems.push(format!(
                "{at}: segment {index} gives no positive pressure at {mid} °C"
            )),
        }
    }
}

fn lint_provenance(
    at: &str,
    index: usize,
    provenance: &ParameterProvenance,
    problems: &mut Vec<String>,
) {
    if provenance.source.is_empty() {
        problems.push(format!("{at}: provenance {index} carries no source"));
    }
    if provenance.locator.is_empty() {
        problems.push(format!("{at}: provenance {index} carries no locator"));
    }
    if provenance.licence.is_empty() {
        problems.push(format!("{at}: provenance {index} carries no licence"));
    }
    if !iso_date(provenance.recorded) {
        problems.push(format!(
            "{at}: provenance {index} records '{}' rather than an ISO date",
            provenance.recorded
        ));
    }
    if provenance.lane == RightsLane::OpenLicensedData
        && !OPEN_DATA_LICENCES.contains(&provenance.licence)
    {
        problems.push(format!(
            "{at}: provenance {index} claims the open-data lane under '{}', \
             which is not one of the licences that clear promotion",
            provenance.licence
        ));
    }
    if provenance.lane == RightsLane::PrimaryLiteratureCoefficientsPendingReview
        && OPEN_DATA_LICENCES.contains(&provenance.licence)
    {
        problems.push(format!(
            "{at}: provenance {index} names the open licence '{}' but sits in the \
             pending-review lane; move it to the open lane or correct the licence",
            provenance.licence
        ));
    }
}

/// `AAAAAAAAAAAAAA-BBBBBBBBBB-C`, uppercase ASCII letters and two hyphens.
fn well_formed_inchikey(key: &str) -> bool {
    let blocks: Vec<&str> = key.split('-').collect();
    blocks.len() == 3
        && blocks[0].len() == 14
        && blocks[1].len() == 10
        && blocks[2].len() == 1
        && blocks
            .iter()
            .all(|block| block.bytes().all(|b| b.is_ascii_uppercase()))
}

/// `YYYY-MM-DD`, checked for shape rather than for being a real day.
fn iso_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts
            .iter()
            .all(|part| part.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_pack_passes_its_own_lint() {
        for row in rows() {
            if let Err(problems) = lint_row(row) {
                panic!("{}: {}", row.identity.species_key, problems.join("; "));
            }
        }
    }

    #[test]
    fn rows_are_unique_and_ordered_by_inchikey() {
        let keys: Vec<&str> = rows().iter().map(|row| row.identity.inchikey).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            keys, sorted,
            "pack rows must be unique and InChIKey-ordered"
        );
    }

    #[test]
    fn a_row_is_found_by_its_key_and_never_by_its_name() {
        let water = row_by_inchikey("XLYOFNOQVPJJNP-UHFFFAOYSA-N").expect("water row");
        assert_eq!(water.identity.species_key, "water");
        // The display name and the registry key are carried, not consulted.
        assert!(row_by_inchikey("water").is_none());
        assert!(row_by_inchikey("XLYOFNOQVPJJNP-UHFFFAOYSA-X").is_none());
    }

    #[test]
    fn the_ethanol_row_keeps_one_provenance_per_segment() {
        let ethanol = row_by_inchikey("LFQSCWFLJHTTHZ-UHFFFAOYSA-N").expect("ethanol row");
        let cleared = ethanol.vapour_pressure.as_ref().expect("cleared ethanol");
        assert_eq!(cleared.correlation.segments().len(), 2);
        assert_eq!(cleared.provenance.len(), 2);
        // The two segments really do sit in different rights lanes, which is
        // the case a single `source` string on the row would have lost.
        assert_eq!(
            cleared.provenance[0].lane,
            RightsLane::PrimaryLiteratureCoefficientsPendingReview
        );
        assert_eq!(cleared.provenance[1].lane, RightsLane::OpenLicensedData);
        assert_eq!(cleared.provenance[1].licence, "CC-BY-4.0");
        assert_eq!(
            cleared.provenance_at(120.0).map(|p| p.lane),
            Some(RightsLane::OpenLicensedData)
        );
        assert_eq!(
            cleared.provenance_at(20.0).map(|p| p.lane),
            Some(RightsLane::PrimaryLiteratureCoefficientsPendingReview)
        );
    }

    #[test]
    fn no_row_claims_a_residual_equation_of_state_or_a_density() {
        for row in rows() {
            let density = row
                .liquid_density_g_per_ml(25.0)
                .expect_err("no density is cleared");
            assert!(matches!(density, PackRefusal::NotCleared { .. }));
            assert!(row.gap(FluidParameter::ResidualEos).is_some());
        }
    }

    #[test]
    fn a_permanent_gas_refuses_saturation_pressure_by_name() {
        let nitrogen = row_by_inchikey("IJGRMHOSHXDMSA-UHFFFAOYSA-N").expect("nitrogen row");
        let refusal = nitrogen
            .saturation_pressure_kpa(20.0)
            .expect_err("nitrogen has no cleared saturation pressure");
        assert_eq!(refusal.parameter(), FluidParameter::SaturationPressure);
        assert!(refusal.to_string().contains("not cleared"));
    }

    #[test]
    fn outside_the_fitted_interval_is_a_refusal_and_not_an_extrapolation() {
        let water = row_by_inchikey("XLYOFNOQVPJJNP-UHFFFAOYSA-N").expect("water row");
        assert!(water.saturation_pressure_kpa(50.0).is_ok());
        let refusal = water
            .saturation_pressure_kpa(180.0)
            .expect_err("water's fit stops at 100 °C");
        assert!(matches!(refusal, PackRefusal::OutsideValidity { .. }));
    }

    #[test]
    fn the_lint_refuses_a_row_with_no_source() {
        const SOURCELESS: Antoine = Antoine {
            a: 7.0,
            b: 1700.0,
            c: 230.0,
            valid_c: (0.0, 100.0),
            source: "",
        };
        const NO_PROVENANCE: &[ParameterProvenance] = &[ParameterProvenance {
            source: "",
            locator: "",
            licence: "",
            lane: RightsLane::OpenLicensedData,
            recorded: "yesterday",
        }];
        let bad = FluidRow {
            identity: ComponentIdentity {
                inchikey: "not-a-key",
                species_key: "invented",
                name: "invented fluid",
            },
            vapour_pressure: Some(ClearedVapourPressure {
                correlation: VapourPressure::Antoine(SOURCELESS),
                provenance: NO_PROVENANCE,
            }),
            gaps: &[],
        };
        let problems = lint_row(&bad).expect_err("a sourceless row must be refused");
        let joined = problems.join("; ");
        assert!(joined.contains("carries no source"), "{joined}");
        assert!(joined.contains("carries no licence"), "{joined}");
        assert!(joined.contains("ISO date"), "{joined}");
        assert!(joined.contains("not a Standard InChIKey"), "{joined}");
        assert!(joined.contains("liquid density"), "{joined}");
        assert!(joined.contains("residual equation of state"), "{joined}");
    }
}
