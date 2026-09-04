//! Enzyme-family catalogue shared by curated reaction and material bridges.
//!
//! Enzymes are catalysts rather than stoichiometric protein molecules in the
//! ledger. Their `SpeciesData` formula is consequently the same explicit
//! carbon placeholder already used by catalase; the approximate molar mass is
//! only a dose conversion and must not be read as a molecular formula claim.

use serde::{Deserialize, Serialize};

use crate::species::{Phase, SpeciesData};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnzymeFamily {
    Lactase,
    Protease,
    Lipase,
    Catalase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnzymeProfile {
    pub family: EnzymeFamily,
    pub species: &'static str,
    pub acts_on: &'static str,
    pub products: &'static str,
}

pub const FAMILIES: &[EnzymeProfile] = &[
    EnzymeProfile {
        family: EnzymeFamily::Lactase,
        species: "lactase",
        acts_on: "lactose",
        products: "glucose and galactose",
    },
    EnzymeProfile {
        family: EnzymeFamily::Protease,
        species: "protease",
        acts_on: "protein peptide bonds",
        products: "shorter peptides and amino acids",
    },
    EnzymeProfile {
        family: EnzymeFamily::Lipase,
        species: "lipase",
        acts_on: "triglycerides",
        products: "glycerol and fatty acids",
    },
    EnzymeProfile {
        family: EnzymeFamily::Catalase,
        species: "catalase",
        acts_on: "hydrogen peroxide",
        products: "water and oxygen",
    },
];

pub fn profile(species: &str) -> Option<&'static EnzymeProfile> {
    FAMILIES.iter().find(|profile| profile.species == species)
}

const fn enzyme_species(key: &'static str, mass: f64, provenance: &'static str) -> SpeciesData {
    SpeciesData {
        key,
        name: key,
        formula: "C",
        inchikey: "",
        molar_mass: mass,
        heat_capacity: 0.0,
        density: 1.35,
        standard_phase: Phase::Aqueous,
        appearance: Some("colourless"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        dissolves_without_speciation: true,
        aqueous_solubility_g_per_100_ml: None,
        aqueous_solubility_g_per_100_ml_at_100c: None,
        forms_only_above_k: None,
        magnetic: false,
        transitions: None,
        provenance,
    }
}

pub static ADDITIONAL_ENZYME_SPECIES: &[SpeciesData] = &[
    enzyme_species(
        "lactase",
        120_000.0,
        "Enzyme-family teaching catalyst; approximate beta-galactosidase dose mass, with no molecular-formula claim",
    ),
    enzyme_species(
        "protease",
        30_000.0,
        "Enzyme-family teaching catalyst; representative protease dose mass, with no specific enzyme or molecular-formula claim",
    ),
    enzyme_species(
        "lipase",
        33_000.0,
        "Enzyme-family teaching catalyst; representative lipase dose mass, with no specific enzyme or molecular-formula claim",
    ),
];

pub(crate) fn species_data(key: &str) -> Option<&'static SpeciesData> {
    ADDITIONAL_ENZYME_SPECIES
        .iter()
        .find(|species| species.key == key)
}
