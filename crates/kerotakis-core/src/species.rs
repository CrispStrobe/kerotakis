//! Species identity and the seed property registry.
//!
//! Canonical identity is the InChIKey (PLAN.md, L1); until the Indigo FFI
//! lands, entries carry their InChIKey as data and are looked up by a short
//! human key. Property values are individual published constants with the
//! source recorded per entry (atomic weights: IUPAC/CIAAW; heat capacities:
//! CODATA/standard reference values in the open literature).

use serde::{Deserialize, Serialize};

use crate::units::{Grams, Liters, Moles};

/// Stable species identifier. Currently a registry key; becomes the InChIKey
/// once L1 identity is wired through Indigo.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpeciesId(pub String);

impl SpeciesId {
    pub fn new(key: &str) -> Self {
        SpeciesId(key.to_string())
    }
}

impl std::fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Solid,
    Liquid,
    Aqueous,
    Gas,
}

/// Registry entry for one species.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesData {
    pub key: &'static str,
    pub name: &'static str,
    pub formula: &'static str,
    pub inchikey: &'static str,
    /// g/mol
    pub molar_mass: f64,
    /// Molar heat capacity of the phase it is added as, J/(mol·K).
    pub heat_capacity: f64,
    /// Density of the pure substance at ~25 °C, g/mL (used for volume of
    /// liquids; approximate, additive-volume assumption is surfaced to the
    /// renderer as such).
    pub density: f64,
    /// Phase the pure substance is in at room conditions.
    pub standard_phase: Phase,
    /// Colour word for observations ("white", "colourless", …), if curated.
    pub appearance: Option<&'static str>,
    /// One-line provenance for the constants above.
    pub provenance: &'static str,
}

impl SpeciesData {
    pub fn moles_from_grams(&self, g: Grams) -> Moles {
        Moles(g.0 / self.molar_mass)
    }

    pub fn grams_from_moles(&self, n: Moles) -> Grams {
        Grams(n.0 * self.molar_mass)
    }

    pub fn liters_from_moles(&self, n: Moles) -> Liters {
        // g / (g/mL) = mL
        Liters(self.grams_from_moles(n).0 / self.density / 1000.0)
    }

    pub fn moles_from_liters(&self, v: Liters) -> Moles {
        self.moles_from_grams(Grams(v.0 * 1000.0 * self.density))
    }
}

/// Seed registry. Grows into the PubChem/Wikidata build-time export (L1).
///
/// Dissolved ions carry `heat_capacity: 0.0`: partial molar heat capacities
/// of aqueous ions are small (often negative) and are not modelled at this
/// stage — the solution's heat capacity is carried by its water. Ion
/// densities are unused (solution volume is carried by the liquid phase).
pub const REGISTRY: &[SpeciesData] = &[
    SpeciesData {
        key: "water",
        name: "water",
        formula: "H2O",
        inchikey: "XLYOFNOQVPJJNP-UHFFFAOYSA-N",
        molar_mass: 18.015,
        heat_capacity: 75.3,
        density: 0.997,
        standard_phase: Phase::Liquid,
        appearance: Some("colourless"),
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(l), density: CODATA/standard reference values",
    },
    SpeciesData {
        key: "ethanol",
        name: "ethanol",
        formula: "C2H5OH",
        inchikey: "LFQSCWFLJHTTHZ-UHFFFAOYSA-N",
        molar_mass: 46.069,
        heat_capacity: 112.3,
        density: 0.789,
        standard_phase: Phase::Liquid,
        appearance: Some("colourless"),
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(l), density: standard reference values",
    },
    SpeciesData {
        key: "NaCl",
        name: "sodium chloride",
        formula: "NaCl",
        inchikey: "FAPWRFPIFSIZLT-UHFFFAOYSA-M",
        molar_mass: 58.443,
        heat_capacity: 50.5,
        density: 2.17,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "AgNO3",
        name: "silver nitrate",
        formula: "AgNO3",
        inchikey: "SQGYOTSLMSWVJD-UHFFFAOYSA-N",
        molar_mass: 169.87,
        heat_capacity: 93.1,
        density: 4.35,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "AgCl",
        name: "silver chloride",
        formula: "AgCl",
        inchikey: "HKZLPVFGJNLROG-UHFFFAOYSA-M",
        molar_mass: 143.32,
        heat_capacity: 50.8,
        density: 5.56,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "Na+",
        name: "sodium ion",
        formula: "Na+",
        inchikey: "FKNQFGJONOIPTF-UHFFFAOYSA-N",
        molar_mass: 22.990,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "Cl-",
        name: "chloride ion",
        formula: "Cl-",
        inchikey: "VEXZGXHMUGYJMC-UHFFFAOYSA-M",
        molar_mass: 35.453,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "Ag+",
        name: "silver ion",
        formula: "Ag+",
        inchikey: "FOIXSVOLVBLSDH-UHFFFAOYSA-N",
        molar_mass: 107.868,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "NO3-",
        name: "nitrate ion",
        formula: "NO3-",
        inchikey: "NHNBFGGVMKEFGY-UHFFFAOYSA-N",
        molar_mass: 62.004,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
];

pub fn lookup(id: &SpeciesId) -> Option<&'static SpeciesData> {
    REGISTRY.iter().find(|s| s.key == id.0)
}

pub fn lookup_key(key: &str) -> Option<&'static SpeciesData> {
    REGISTRY.iter().find(|s| s.key == key)
}
