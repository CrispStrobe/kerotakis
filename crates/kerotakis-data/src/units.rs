//! BRD-003: deterministic unit normalization for quarantined external data.
//!
//! Upstream sources spell the same physical unit a dozen ways: `g/mL`,
//! `g·cm⁻³`, `g/cc`; `°C` and `K`; `kJ/mol` and `kcal/mol`; `mg/L`, `%`, `ppm`.
//! This module converges those spellings onto the unit and [`Dimension`]
//! vocabulary `kerotakis-data` already defines — it does not invent a second
//! one, and it never guesses.
//!
//! Three rules hold everywhere:
//!
//! 1. **Only listed spellings normalize.** An unrecognised spelling is
//!    [`UnitNormalizationError::UnknownUnit`], which carries the original
//!    string so a reviewer sees exactly what the source emitted. There is no
//!    prefix arithmetic, no fuzzy matching, and no fallback to
//!    [`Dimension::Other`].
//! 2. **The table is case-sensitive.** `mg` is a milligram and `Mg` is
//!    unregistered rather than quietly a megagram.
//! 3. **A unit fixes the physical quantity, not the semantic field.** `g/L`
//!    is unambiguously mass per volume, but only the target field knows
//!    whether that is a [`Dimension::MassDensity`] or a
//!    [`Dimension::MassConcentration`]. [`normalize_unit`] returns the
//!    spelling's declared default; [`normalize_unit_for`] takes the dimension
//!    the target field actually carries and refuses a mismatch.
//!
//! Deliberate gaps: bare mass (`g`, `mg`), bare energy (`kJ`, `kcal`) and
//! wavenumbers (`cm-1`) have no dimension in this schema, so they are typed
//! rejections rather than a coerced approximation. An importer that needs them
//! extends [`Dimension`] under review first.

use crate::schema::{Dimension, Unit};

/// A physical quantity this module can canonicalize, with exactly one
/// canonical spelling each. Distinct from [`Dimension`] only where the schema
/// draws a semantic distinction the unit string cannot resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Quantity {
    Dimensionless,
    MassPerMass,
    Amount,
    MolarMass,
    Temperature,
    Pressure,
    MassDensity,
    MassConcentration,
    Concentration,
    MolarEnergy,
    MolarHeatCapacity,
    MolarEntropy,
    Diffusivity,
    DynamicViscosity,
    ThermalConductivity,
    SurfaceTension,
    ElectricalConductivity,
    MolecularLength,
    Wavelength,
    MolarAbsorptivity,
    Time,
    Area,
    Volume,
}

impl Quantity {
    const ALL: &'static [Self] = &[
        Self::Dimensionless,
        Self::MassPerMass,
        Self::Amount,
        Self::MolarMass,
        Self::Temperature,
        Self::Pressure,
        Self::MassDensity,
        Self::MassConcentration,
        Self::Concentration,
        Self::MolarEnergy,
        Self::MolarHeatCapacity,
        Self::MolarEntropy,
        Self::Diffusivity,
        Self::DynamicViscosity,
        Self::ThermalConductivity,
        Self::SurfaceTension,
        Self::ElectricalConductivity,
        Self::MolecularLength,
        Self::Wavelength,
        Self::MolarAbsorptivity,
        Self::Time,
        Self::Area,
        Self::Volume,
    ];

    /// The one spelling a normalized value is expressed in. These match the
    /// spellings the checked-in registry already carries.
    const fn canonical(self) -> &'static str {
        match self {
            // A mass fraction is a ratio; the registry writes ratios as "1".
            Self::Dimensionless | Self::MassPerMass => "1",
            Self::Amount => "mol",
            Self::MolarMass => "g/mol",
            Self::Temperature => "K",
            Self::Pressure => "Pa",
            Self::MassDensity => "g/mL",
            Self::MassConcentration => "g/L",
            Self::Concentration => "mol/L",
            Self::MolarEnergy => "kJ/mol",
            Self::MolarHeatCapacity | Self::MolarEntropy => "J/(mol.K)",
            Self::Diffusivity => "m2/s",
            Self::DynamicViscosity => "Pa.s",
            Self::ThermalConductivity => "W/(m.K)",
            Self::SurfaceTension => "N/m",
            Self::ElectricalConductivity => "S/m",
            Self::MolecularLength => "Ang",
            Self::Wavelength => "nm",
            Self::MolarAbsorptivity => "L/(mol.cm)",
            Self::Time => "s",
            Self::Area => "m2",
            Self::Volume => "L",
        }
    }

    fn dimension(self) -> Dimension {
        match self {
            Self::Dimensionless => Dimension::Dimensionless,
            Self::MassPerMass => Dimension::MassPerMass,
            Self::Amount => Dimension::Amount,
            Self::MolarMass => Dimension::MolarMass,
            Self::Temperature => Dimension::Temperature,
            Self::Pressure => Dimension::Pressure,
            Self::MassDensity => Dimension::MassDensity,
            Self::MassConcentration => Dimension::MassConcentration,
            Self::Concentration => Dimension::Concentration,
            Self::MolarEnergy => Dimension::MolarEnergy,
            Self::MolarHeatCapacity => Dimension::MolarHeatCapacity,
            Self::MolarEntropy => Dimension::MolarEntropy,
            Self::Diffusivity => Dimension::Diffusivity,
            Self::DynamicViscosity => Dimension::DynamicViscosity,
            Self::ThermalConductivity => Dimension::ThermalConductivity,
            Self::SurfaceTension => Dimension::SurfaceTension,
            Self::ElectricalConductivity => Dimension::ElectricalConductivity,
            Self::MolecularLength => Dimension::MolecularLength,
            Self::Wavelength => Dimension::Wavelength,
            Self::MolarAbsorptivity => Dimension::MolarAbsorptivity,
            Self::Time => Dimension::Time,
            Self::Area => Dimension::Area,
            Self::Volume => Dimension::Volume,
        }
    }

    fn unit(self) -> Unit {
        Unit {
            symbol: self.canonical().to_owned(),
            dimension: self.dimension(),
        }
    }
}

/// One reading of a spelling: which quantity it measures and the affine map
/// onto that quantity's canonical unit.
#[derive(Debug, Clone, Copy)]
struct Interpretation {
    quantity: Quantity,
    scale: f64,
    offset: f64,
}

const fn to(quantity: Quantity, scale: f64) -> Interpretation {
    Interpretation {
        quantity,
        scale,
        offset: 0.0,
    }
}

const fn affine(quantity: Quantity, scale: f64, offset: f64) -> Interpretation {
    Interpretation {
        quantity,
        scale,
        offset,
    }
}

/// The canonical unit for a schema dimension, when this module can produce
/// one. [`Dimension::RateConstant`] has no dimension-independent spelling
/// (its units depend on reaction order) and [`Dimension::Other`] is by
/// definition outside the vocabulary.
pub fn canonical_symbol(dimension: &Dimension) -> Option<&'static str> {
    Quantity::ALL
        .iter()
        .find(|quantity| quantity.dimension() == *dimension)
        .map(|quantity| quantity.canonical())
}

/// An affine map from an upstream spelling onto a canonical unit.
///
/// `canonical_value = raw_value * scale + offset`. The offset is non-zero only
/// for the non-absolute temperature scales.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitConversion {
    pub canonical: Unit,
    pub scale: f64,
    pub offset: f64,
}

impl UnitConversion {
    /// Whether this spelling is already canonical.
    pub fn is_identity(&self) -> bool {
        self.scale == 1.0 && self.offset == 0.0
    }

    pub fn apply(&self, value: f64) -> f64 {
        value * self.scale + self.offset
    }

    /// The inverse map, for round-tripping a canonical value back into the
    /// spelling the source used.
    pub fn invert(&self, canonical_value: f64) -> f64 {
        (canonical_value - self.offset) / self.scale
    }
}

/// A value expressed in the canonical unit of its dimension.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedQuantity {
    pub value: f64,
    pub unit: Unit,
}

/// Every way normalization refuses. Each variant keeps the source's original
/// spelling so a rejection is reviewable without the caller re-threading it.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitNormalizationError {
    /// The source supplied no unit at all.
    EmptyUnit,
    /// The spelling is not in the reviewed table.
    UnknownUnit { original: String },
    /// The spelling is known but measures a different quantity than the
    /// target field declares.
    DimensionMismatch {
        original: String,
        expected: Dimension,
        found: Dimension,
    },
    /// The target field's dimension has no canonical spelling here.
    UnsupportedDimension {
        original: String,
        expected: Dimension,
    },
    /// The source value, or its conversion, is not a finite number.
    NonFiniteValue { original: String, value: f64 },
    /// A converted temperature falls below absolute zero.
    BelowAbsoluteZero { original: String, kelvin: f64 },
}

impl UnitNormalizationError {
    /// The upstream spelling, preserved verbatim.
    pub fn original(&self) -> &str {
        match self {
            Self::EmptyUnit => "",
            Self::UnknownUnit { original }
            | Self::DimensionMismatch { original, .. }
            | Self::UnsupportedDimension { original, .. }
            | Self::NonFiniteValue { original, .. }
            | Self::BelowAbsoluteZero { original, .. } => original,
        }
    }
}

impl std::fmt::Display for UnitNormalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyUnit => write!(formatter, "no unit was supplied"),
            Self::UnknownUnit { original } => {
                write!(formatter, "unit spelling {original:?} is not reviewed")
            }
            Self::DimensionMismatch {
                original,
                expected,
                found,
            } => write!(
                formatter,
                "unit {original:?} measures {found:?}, not the expected {expected:?}"
            ),
            Self::UnsupportedDimension { original, expected } => write!(
                formatter,
                "dimension {expected:?} has no canonical unit here (from {original:?})"
            ),
            Self::NonFiniteValue { original, value } => {
                write!(formatter, "value {value} in {original:?} is not finite")
            }
            Self::BelowAbsoluteZero { original, kelvin } => write!(
                formatter,
                "temperature {original:?} converts to {kelvin} K, below absolute zero"
            ),
        }
    }
}

impl std::error::Error for UnitNormalizationError {}

/// Normalize a unit spelling using its declared default dimension.
///
/// Prefer [`normalize_unit_for`] whenever the target field's dimension is
/// known: several spellings serve two schema dimensions (`g/L` is a mass
/// density or a mass concentration; `J/(mol.K)` is a molar heat capacity or a
/// molar entropy), and only the field can say which.
pub fn normalize_unit(spelling: &str) -> Result<UnitConversion, UnitNormalizationError> {
    let interpretations = lookup(spelling)?;
    Ok(conversion(interpretations[0]))
}

/// Normalize a unit spelling against the dimension the target field carries.
pub fn normalize_unit_for(
    spelling: &str,
    expected: &Dimension,
) -> Result<UnitConversion, UnitNormalizationError> {
    let interpretations = lookup(spelling)?;
    if let Some(interpretation) = interpretations
        .iter()
        .find(|interpretation| interpretation.quantity.dimension() == *expected)
    {
        return Ok(conversion(*interpretation));
    }
    if canonical_symbol(expected).is_none() {
        return Err(UnitNormalizationError::UnsupportedDimension {
            original: spelling.to_owned(),
            expected: expected.clone(),
        });
    }
    Err(UnitNormalizationError::DimensionMismatch {
        original: spelling.to_owned(),
        expected: expected.clone(),
        found: interpretations[0].quantity.dimension(),
    })
}

/// Normalize a value and its unit spelling using the spelling's default
/// dimension.
pub fn normalize_quantity(
    value: f64,
    spelling: &str,
) -> Result<NormalizedQuantity, UnitNormalizationError> {
    finish(value, spelling, normalize_unit(spelling)?)
}

/// Normalize a value and its unit spelling against a declared dimension.
pub fn normalize_quantity_for(
    value: f64,
    spelling: &str,
    expected: &Dimension,
) -> Result<NormalizedQuantity, UnitNormalizationError> {
    finish(value, spelling, normalize_unit_for(spelling, expected)?)
}

fn finish(
    value: f64,
    spelling: &str,
    conversion: UnitConversion,
) -> Result<NormalizedQuantity, UnitNormalizationError> {
    if !value.is_finite() {
        return Err(UnitNormalizationError::NonFiniteValue {
            original: spelling.to_owned(),
            value,
        });
    }
    let normalized = conversion.apply(value);
    if !normalized.is_finite() {
        return Err(UnitNormalizationError::NonFiniteValue {
            original: spelling.to_owned(),
            value: normalized,
        });
    }
    if conversion.canonical.dimension == Dimension::Temperature && normalized < 0.0 {
        return Err(UnitNormalizationError::BelowAbsoluteZero {
            original: spelling.to_owned(),
            kelvin: normalized,
        });
    }
    Ok(NormalizedQuantity {
        value: normalized,
        unit: conversion.canonical,
    })
}

fn conversion(interpretation: Interpretation) -> UnitConversion {
    UnitConversion {
        canonical: interpretation.quantity.unit(),
        scale: interpretation.scale,
        offset: interpretation.offset,
    }
}

fn lookup(spelling: &str) -> Result<&'static [Interpretation], UnitNormalizationError> {
    let cleaned = clean(spelling);
    if cleaned.is_empty() {
        return Err(UnitNormalizationError::EmptyUnit);
    }
    SPELLINGS
        .iter()
        .find(|(candidate, _)| *candidate == cleaned)
        .map(|(_, interpretations)| *interpretations)
        .ok_or_else(|| UnitNormalizationError::UnknownUnit {
            original: spelling.to_owned(),
        })
}

/// Fold the typographic variation upstream sources emit — spaces, superscript
/// exponents, middle dots, micro signs, degree glyphs — without touching
/// letter case, which distinguishes real units.
fn clean(spelling: &str) -> String {
    let mut cleaned = String::with_capacity(spelling.len());
    for character in spelling.chars() {
        match character {
            // Whitespace (including the non-breaking kinds) and explicit
            // exponent carets carry no meaning in a unit symbol.
            ' ' | '\t' | '\n' | '\r' | '\u{a0}' | '\u{2007}' | '\u{202f}' | '^' => {}
            '·' | '⋅' | '∙' | '×' | '*' => cleaned.push('.'),
            '−' | '–' | '—' | '⁻' => cleaned.push('-'),
            'μ' | 'µ' => cleaned.push('u'),
            'º' => cleaned.push('°'),
            '℃' => cleaned.push_str("°C"),
            '℉' => cleaned.push_str("°F"),
            'Å' | '\u{212b}' => cleaned.push_str("Ang"),
            '⁰' => cleaned.push('0'),
            '¹' => cleaned.push('1'),
            '²' => cleaned.push('2'),
            '³' => cleaned.push('3'),
            '⁴' => cleaned.push('4'),
            '⁵' => cleaned.push('5'),
            '⁶' => cleaned.push('6'),
            '⁷' => cleaned.push('7'),
            '⁸' => cleaned.push('8'),
            '⁹' => cleaned.push('9'),
            other => cleaned.push(other),
        }
    }
    cleaned
}

/// Every reviewed spelling, in the cleaned form [`clean`] produces.
pub fn known_unit_spellings() -> impl Iterator<Item = &'static str> {
    SPELLINGS.iter().map(|(spelling, _)| *spelling)
}

/// Every schema dimension this module can normalize onto.
pub fn normalizable_dimensions() -> Vec<Dimension> {
    Quantity::ALL
        .iter()
        .map(|quantity| quantity.dimension())
        .collect()
}

use Quantity as Q;

/// The reviewed spelling table. Each row lists every quantity the spelling can
/// measure; the first is the default [`normalize_unit`] uses.
static SPELLINGS: &[(&str, &[Interpretation])] = &[
    // ── ratios ──────────────────────────────────────────────────────────
    ("1", &[to(Q::Dimensionless, 1.0), to(Q::MassPerMass, 1.0)]),
    (
        "unitless",
        &[to(Q::Dimensionless, 1.0), to(Q::MassPerMass, 1.0)],
    ),
    (
        "dimensionless",
        &[to(Q::Dimensionless, 1.0), to(Q::MassPerMass, 1.0)],
    ),
    ("mol/mol", &[to(Q::Dimensionless, 1.0)]),
    ("g/g", &[to(Q::MassPerMass, 1.0), to(Q::Dimensionless, 1.0)]),
    (
        "kg/kg",
        &[to(Q::MassPerMass, 1.0), to(Q::Dimensionless, 1.0)],
    ),
    (
        "mg/mg",
        &[to(Q::MassPerMass, 1.0), to(Q::Dimensionless, 1.0)],
    ),
    ("%", &[to(Q::Dimensionless, 0.01), to(Q::MassPerMass, 0.01)]),
    (
        "percent",
        &[to(Q::Dimensionless, 0.01), to(Q::MassPerMass, 0.01)],
    ),
    (
        "pct",
        &[to(Q::Dimensionless, 0.01), to(Q::MassPerMass, 0.01)],
    ),
    (
        "%(w/w)",
        &[to(Q::MassPerMass, 0.01), to(Q::Dimensionless, 0.01)],
    ),
    (
        "%w/w",
        &[to(Q::MassPerMass, 0.01), to(Q::Dimensionless, 0.01)],
    ),
    (
        "w/w%",
        &[to(Q::MassPerMass, 0.01), to(Q::Dimensionless, 0.01)],
    ),
    (
        "wt%",
        &[to(Q::MassPerMass, 0.01), to(Q::Dimensionless, 0.01)],
    ),
    ("%(v/v)", &[to(Q::Dimensionless, 0.01)]),
    ("%v/v", &[to(Q::Dimensionless, 0.01)]),
    ("vol%", &[to(Q::Dimensionless, 0.01)]),
    (
        "ppm",
        &[to(Q::Dimensionless, 1e-6), to(Q::MassPerMass, 1e-6)],
    ),
    (
        "ppb",
        &[to(Q::Dimensionless, 1e-9), to(Q::MassPerMass, 1e-9)],
    ),
    (
        "g/100g",
        &[to(Q::MassPerMass, 0.01), to(Q::Dimensionless, 0.01)],
    ),
    (
        "mg/100g",
        &[to(Q::MassPerMass, 1e-5), to(Q::Dimensionless, 1e-5)],
    ),
    (
        "mg/g",
        &[to(Q::MassPerMass, 1e-3), to(Q::Dimensionless, 1e-3)],
    ),
    (
        "mg/kg",
        &[to(Q::MassPerMass, 1e-6), to(Q::Dimensionless, 1e-6)],
    ),
    (
        "ug/g",
        &[to(Q::MassPerMass, 1e-6), to(Q::Dimensionless, 1e-6)],
    ),
    (
        "ug/kg",
        &[to(Q::MassPerMass, 1e-9), to(Q::Dimensionless, 1e-9)],
    ),
    (
        "g/kg",
        &[to(Q::MassPerMass, 1e-3), to(Q::Dimensionless, 1e-3)],
    ),
    // ── amount of substance ─────────────────────────────────────────────
    ("mol", &[to(Q::Amount, 1.0)]),
    ("mole", &[to(Q::Amount, 1.0)]),
    ("moles", &[to(Q::Amount, 1.0)]),
    ("kmol", &[to(Q::Amount, 1e3)]),
    ("mmol", &[to(Q::Amount, 1e-3)]),
    ("umol", &[to(Q::Amount, 1e-6)]),
    ("nmol", &[to(Q::Amount, 1e-9)]),
    ("pmol", &[to(Q::Amount, 1e-12)]),
    // ── molar mass ──────────────────────────────────────────────────────
    ("g/mol", &[to(Q::MolarMass, 1.0)]),
    ("g/mole", &[to(Q::MolarMass, 1.0)]),
    ("g.mol-1", &[to(Q::MolarMass, 1.0)]),
    ("gmol-1", &[to(Q::MolarMass, 1.0)]),
    ("kg/kmol", &[to(Q::MolarMass, 1.0)]),
    ("kg/mol", &[to(Q::MolarMass, 1e3)]),
    ("Da", &[to(Q::MolarMass, 1.0)]),
    ("dalton", &[to(Q::MolarMass, 1.0)]),
    ("Dalton", &[to(Q::MolarMass, 1.0)]),
    ("amu", &[to(Q::MolarMass, 1.0)]),
    ("kDa", &[to(Q::MolarMass, 1e3)]),
    // ── temperature ─────────────────────────────────────────────────────
    ("K", &[to(Q::Temperature, 1.0)]),
    ("°K", &[to(Q::Temperature, 1.0)]),
    ("kelvin", &[to(Q::Temperature, 1.0)]),
    ("Kelvin", &[to(Q::Temperature, 1.0)]),
    ("°C", &[affine(Q::Temperature, 1.0, 273.15)]),
    ("degC", &[affine(Q::Temperature, 1.0, 273.15)]),
    ("celsius", &[affine(Q::Temperature, 1.0, 273.15)]),
    ("Celsius", &[affine(Q::Temperature, 1.0, 273.15)]),
    (
        "°F",
        &[affine(
            Q::Temperature,
            0.555_555_555_555_555_6,
            255.372_222_222_222_2,
        )],
    ),
    (
        "degF",
        &[affine(
            Q::Temperature,
            0.555_555_555_555_555_6,
            255.372_222_222_222_2,
        )],
    ),
    (
        "fahrenheit",
        &[affine(
            Q::Temperature,
            0.555_555_555_555_555_6,
            255.372_222_222_222_2,
        )],
    ),
    (
        "Fahrenheit",
        &[affine(
            Q::Temperature,
            0.555_555_555_555_555_6,
            255.372_222_222_222_2,
        )],
    ),
    // ── pressure ────────────────────────────────────────────────────────
    ("Pa", &[to(Q::Pressure, 1.0)]),
    ("pascal", &[to(Q::Pressure, 1.0)]),
    ("Pascal", &[to(Q::Pressure, 1.0)]),
    ("N/m2", &[to(Q::Pressure, 1.0)]),
    ("mPa", &[to(Q::Pressure, 1e-3)]),
    ("hPa", &[to(Q::Pressure, 1e2)]),
    ("kPa", &[to(Q::Pressure, 1e3)]),
    ("MPa", &[to(Q::Pressure, 1e6)]),
    ("GPa", &[to(Q::Pressure, 1e9)]),
    ("bar", &[to(Q::Pressure, 1e5)]),
    ("mbar", &[to(Q::Pressure, 1e2)]),
    ("atm", &[to(Q::Pressure, 101_325.0)]),
    ("mmHg", &[to(Q::Pressure, 133.322_387_415)]),
    ("Torr", &[to(Q::Pressure, 133.322_368_421_052_63)]),
    ("torr", &[to(Q::Pressure, 133.322_368_421_052_63)]),
    ("psi", &[to(Q::Pressure, 6_894.757_293_168_361)]),
    ("dyn/cm2", &[to(Q::Pressure, 0.1)]),
    // ── mass per volume ─────────────────────────────────────────────────
    (
        "g/mL",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "g/ml",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "g/cm3",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "g.cm-3",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "gcm-3",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "g/cc",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "kg/L",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "kg/l",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "kg/dm3",
        &[to(Q::MassDensity, 1.0), to(Q::MassConcentration, 1e3)],
    ),
    (
        "g/L",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "g/l",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "g/dm3",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "kg/m3",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "kgm-3",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "mg/mL",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "mg/ml",
        &[to(Q::MassConcentration, 1.0), to(Q::MassDensity, 1e-3)],
    ),
    (
        "mg/L",
        &[to(Q::MassConcentration, 1e-3), to(Q::MassDensity, 1e-6)],
    ),
    (
        "mg/l",
        &[to(Q::MassConcentration, 1e-3), to(Q::MassDensity, 1e-6)],
    ),
    (
        "ug/mL",
        &[to(Q::MassConcentration, 1e-3), to(Q::MassDensity, 1e-6)],
    ),
    (
        "ug/L",
        &[to(Q::MassConcentration, 1e-6), to(Q::MassDensity, 1e-9)],
    ),
    (
        "g/dL",
        &[to(Q::MassConcentration, 10.0), to(Q::MassDensity, 0.01)],
    ),
    (
        "g/100mL",
        &[to(Q::MassConcentration, 10.0), to(Q::MassDensity, 0.01)],
    ),
    (
        "g/100ml",
        &[to(Q::MassConcentration, 10.0), to(Q::MassDensity, 0.01)],
    ),
    (
        "mg/100mL",
        &[to(Q::MassConcentration, 0.01), to(Q::MassDensity, 1e-5)],
    ),
    // ── amount concentration ────────────────────────────────────────────
    ("mol/L", &[to(Q::Concentration, 1.0)]),
    ("mol/l", &[to(Q::Concentration, 1.0)]),
    ("mol.L-1", &[to(Q::Concentration, 1.0)]),
    ("molL-1", &[to(Q::Concentration, 1.0)]),
    ("mol/dm3", &[to(Q::Concentration, 1.0)]),
    ("M", &[to(Q::Concentration, 1.0)]),
    ("mmol/L", &[to(Q::Concentration, 1e-3)]),
    ("mmol/dm3", &[to(Q::Concentration, 1e-3)]),
    ("mM", &[to(Q::Concentration, 1e-3)]),
    ("mol/m3", &[to(Q::Concentration, 1e-3)]),
    ("umol/L", &[to(Q::Concentration, 1e-6)]),
    ("uM", &[to(Q::Concentration, 1e-6)]),
    ("nmol/L", &[to(Q::Concentration, 1e-9)]),
    ("nM", &[to(Q::Concentration, 1e-9)]),
    ("pM", &[to(Q::Concentration, 1e-12)]),
    // ── molar energy ────────────────────────────────────────────────────
    ("kJ/mol", &[to(Q::MolarEnergy, 1.0)]),
    ("kJ/mole", &[to(Q::MolarEnergy, 1.0)]),
    ("kJ.mol-1", &[to(Q::MolarEnergy, 1.0)]),
    ("kJmol-1", &[to(Q::MolarEnergy, 1.0)]),
    ("J/mol", &[to(Q::MolarEnergy, 1e-3)]),
    ("J.mol-1", &[to(Q::MolarEnergy, 1e-3)]),
    ("Jmol-1", &[to(Q::MolarEnergy, 1e-3)]),
    ("kJ/kmol", &[to(Q::MolarEnergy, 1e-3)]),
    ("kcal/mol", &[to(Q::MolarEnergy, 4.184)]),
    ("cal/mol", &[to(Q::MolarEnergy, 4.184e-3)]),
    // ── molar heat capacity and molar entropy ───────────────────────────
    (
        "J/(mol.K)",
        &[to(Q::MolarHeatCapacity, 1.0), to(Q::MolarEntropy, 1.0)],
    ),
    (
        "J/(molK)",
        &[to(Q::MolarHeatCapacity, 1.0), to(Q::MolarEntropy, 1.0)],
    ),
    (
        "J/(K.mol)",
        &[to(Q::MolarHeatCapacity, 1.0), to(Q::MolarEntropy, 1.0)],
    ),
    (
        "J/mol/K",
        &[to(Q::MolarHeatCapacity, 1.0), to(Q::MolarEntropy, 1.0)],
    ),
    (
        "J.mol-1.K-1",
        &[to(Q::MolarHeatCapacity, 1.0), to(Q::MolarEntropy, 1.0)],
    ),
    (
        "Jmol-1K-1",
        &[to(Q::MolarHeatCapacity, 1.0), to(Q::MolarEntropy, 1.0)],
    ),
    (
        "kJ/(mol.K)",
        &[to(Q::MolarHeatCapacity, 1e3), to(Q::MolarEntropy, 1e3)],
    ),
    (
        "cal/(mol.K)",
        &[to(Q::MolarHeatCapacity, 4.184), to(Q::MolarEntropy, 4.184)],
    ),
    // ── transport and interface ─────────────────────────────────────────
    ("m2/s", &[to(Q::Diffusivity, 1.0)]),
    ("cm2/s", &[to(Q::Diffusivity, 1e-4)]),
    ("mm2/s", &[to(Q::Diffusivity, 1e-6)]),
    ("Pa.s", &[to(Q::DynamicViscosity, 1.0)]),
    ("Pas", &[to(Q::DynamicViscosity, 1.0)]),
    ("kg/(m.s)", &[to(Q::DynamicViscosity, 1.0)]),
    ("mPa.s", &[to(Q::DynamicViscosity, 1e-3)]),
    ("mPas", &[to(Q::DynamicViscosity, 1e-3)]),
    ("cP", &[to(Q::DynamicViscosity, 1e-3)]),
    ("P", &[to(Q::DynamicViscosity, 0.1)]),
    ("poise", &[to(Q::DynamicViscosity, 0.1)]),
    ("W/(m.K)", &[to(Q::ThermalConductivity, 1.0)]),
    ("W/(mK)", &[to(Q::ThermalConductivity, 1.0)]),
    ("W/m/K", &[to(Q::ThermalConductivity, 1.0)]),
    ("mW/(m.K)", &[to(Q::ThermalConductivity, 1e-3)]),
    ("W/(cm.K)", &[to(Q::ThermalConductivity, 1e2)]),
    ("N/m", &[to(Q::SurfaceTension, 1.0)]),
    ("mN/m", &[to(Q::SurfaceTension, 1e-3)]),
    ("dyn/cm", &[to(Q::SurfaceTension, 1e-3)]),
    ("mJ/m2", &[to(Q::SurfaceTension, 1e-3)]),
    ("erg/cm2", &[to(Q::SurfaceTension, 1e-3)]),
    ("S/m", &[to(Q::ElectricalConductivity, 1.0)]),
    ("mho/m", &[to(Q::ElectricalConductivity, 1.0)]),
    ("S/cm", &[to(Q::ElectricalConductivity, 1e2)]),
    ("mS/cm", &[to(Q::ElectricalConductivity, 0.1)]),
    ("uS/cm", &[to(Q::ElectricalConductivity, 1e-4)]),
    // ── optical ─────────────────────────────────────────────────────────
    // Shared spellings keep wavelength first, preserving historical untyped
    // normalization. Model parameters must declare MolecularLength.
    ("m", &[to(Q::MolecularLength, 1e10)]),
    ("meter", &[to(Q::MolecularLength, 1e10)]),
    ("metre", &[to(Q::MolecularLength, 1e10)]),
    (
        "nm",
        &[to(Q::Wavelength, 1.0), to(Q::MolecularLength, 10.0)],
    ),
    ("nanometer", &[to(Q::MolecularLength, 10.0)]),
    ("nanometre", &[to(Q::MolecularLength, 10.0)]),
    ("pm", &[to(Q::Wavelength, 1e-3)]),
    ("um", &[to(Q::Wavelength, 1e3)]),
    (
        "Ang",
        &[to(Q::Wavelength, 0.1), to(Q::MolecularLength, 1.0)],
    ),
    ("angstrom", &[to(Q::MolecularLength, 1.0)]),
    ("Angstrom", &[to(Q::MolecularLength, 1.0)]),
    ("L/(mol.cm)", &[to(Q::MolarAbsorptivity, 1.0)]),
    ("L.mol-1.cm-1", &[to(Q::MolarAbsorptivity, 1.0)]),
    ("Lmol-1cm-1", &[to(Q::MolarAbsorptivity, 1.0)]),
    ("M-1.cm-1", &[to(Q::MolarAbsorptivity, 1.0)]),
    ("M-1cm-1", &[to(Q::MolarAbsorptivity, 1.0)]),
    ("1/(M.cm)", &[to(Q::MolarAbsorptivity, 1.0)]),
    ("m2/mol", &[to(Q::MolarAbsorptivity, 10.0)]),
    // ── time ────────────────────────────────────────────────────────────
    ("s", &[to(Q::Time, 1.0)]),
    ("sec", &[to(Q::Time, 1.0)]),
    ("second", &[to(Q::Time, 1.0)]),
    ("seconds", &[to(Q::Time, 1.0)]),
    ("ms", &[to(Q::Time, 1e-3)]),
    ("us", &[to(Q::Time, 1e-6)]),
    ("ns", &[to(Q::Time, 1e-9)]),
    ("min", &[to(Q::Time, 60.0)]),
    ("minute", &[to(Q::Time, 60.0)]),
    ("minutes", &[to(Q::Time, 60.0)]),
    ("h", &[to(Q::Time, 3_600.0)]),
    ("hr", &[to(Q::Time, 3_600.0)]),
    ("hour", &[to(Q::Time, 3_600.0)]),
    ("hours", &[to(Q::Time, 3_600.0)]),
    ("d", &[to(Q::Time, 86_400.0)]),
    ("day", &[to(Q::Time, 86_400.0)]),
    ("days", &[to(Q::Time, 86_400.0)]),
    // ── area and volume ─────────────────────────────────────────────────
    ("m2", &[to(Q::Area, 1.0)]),
    ("km2", &[to(Q::Area, 1e6)]),
    ("cm2", &[to(Q::Area, 1e-4)]),
    ("mm2", &[to(Q::Area, 1e-6)]),
    ("L", &[to(Q::Volume, 1.0)]),
    ("l", &[to(Q::Volume, 1.0)]),
    ("dm3", &[to(Q::Volume, 1.0)]),
    ("m3", &[to(Q::Volume, 1e3)]),
    ("dL", &[to(Q::Volume, 0.1)]),
    ("cL", &[to(Q::Volume, 0.01)]),
    ("mL", &[to(Q::Volume, 1e-3)]),
    ("ml", &[to(Q::Volume, 1e-3)]),
    ("cm3", &[to(Q::Volume, 1e-3)]),
    ("cc", &[to(Q::Volume, 1e-3)]),
    ("uL", &[to(Q::Volume, 1e-6)]),
    ("ul", &[to(Q::Volume, 1e-6)]),
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn every_spelling_is_listed_once_and_already_cleaned() {
        let mut seen = BTreeSet::new();
        for (spelling, interpretations) in SPELLINGS {
            assert!(seen.insert(*spelling), "duplicate spelling {spelling}");
            assert_eq!(
                clean(spelling),
                *spelling,
                "table key {spelling} is not in cleaned form"
            );
            assert!(!interpretations.is_empty(), "{spelling} has no reading");
            let mut quantities = BTreeSet::new();
            for interpretation in *interpretations {
                assert!(
                    quantities.insert(interpretation.quantity),
                    "{spelling} repeats a quantity"
                );
                assert!(
                    interpretation.scale.is_finite() && interpretation.scale != 0.0,
                    "{spelling} has an unusable scale"
                );
            }
        }
    }

    #[test]
    fn every_canonical_spelling_normalizes_to_itself() {
        for quantity in Quantity::ALL {
            let dimension = quantity.dimension();
            let conversion = normalize_unit_for(quantity.canonical(), &dimension)
                .unwrap_or_else(|error| panic!("{:?}: {error}", quantity.canonical()));
            assert_eq!(conversion.canonical.symbol, quantity.canonical());
            assert_eq!(conversion.canonical.dimension, dimension);
            assert!(
                conversion.is_identity(),
                "{:?} is not an identity conversion",
                quantity.canonical()
            );
        }
    }

    #[test]
    fn normalization_is_idempotent() {
        for (spelling, _) in SPELLINGS {
            let once = normalize_quantity(2.5, spelling).expect("listed spelling normalizes");
            let twice = normalize_quantity_for(once.value, &once.unit.symbol, &once.unit.dimension)
                .expect("canonical spelling normalizes");
            assert_eq!(once, twice, "{spelling} is not idempotent");
        }
    }

    #[test]
    fn conversions_round_trip_through_their_inverse() {
        for (spelling, _) in SPELLINGS {
            let conversion = normalize_unit(spelling).expect("listed spelling normalizes");
            let original = 3.25_f64;
            let back = conversion.invert(conversion.apply(original));
            assert!(
                (back - original).abs() <= 1e-9 * original.abs().max(1.0),
                "{spelling} did not round-trip: {back}"
            );
        }
    }
}
