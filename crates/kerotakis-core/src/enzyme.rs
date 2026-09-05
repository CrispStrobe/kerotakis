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
    /// The gastric protease. It is listed beside the generic `Protease`
    /// rather than inside it because the whole point of pepsin at this
    /// bench is that it works where the generic one does not: its acidity
    /// window is a different number, and a row that asks about the stomach
    /// deserves to be told which catalyst answered.
    Pepsin,
    /// The pineapple protease. Same reason: it is the one a food carries
    /// rather than one an operator weighs out.
    Bromelain,
}

/// The reviewed macromolecule class a catalyst cuts.
///
/// The substrate table and the catalyst table used to be joined by
/// `EnzymeFamily`, which silently made "which enzyme" and "which food
/// molecule" the same question. They are not: three different proteases cut
/// the same peptide bonds and differ only in where they do it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateClass {
    Lactose,
    Protein,
    Triglyceride,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnzymeProfile {
    pub family: EnzymeFamily,
    pub species: &'static str,
    pub acts_on: &'static str,
    pub products: &'static str,
    /// The bounded-activity substrate class, or `None` for a catalyst whose
    /// chemistry is owned by a curated stoichiometric reaction instead.
    pub substrate: Option<SubstrateClass>,
    pub optimum_temperature_k: f64,
    pub temperature_width_k: f64,
    /// Above this temperature the protein is treated as irreversibly
    /// denatured: cooling the beaker down again does not bring it back.
    pub denatures_above_k: f64,
    pub optimum_ph: f64,
    pub ph_width: f64,
}

/// The catalyst table.
///
/// The two envelopes are Gaussian in temperature and in pH, each with an
/// optimum and a width. Both are DELIBERATELY smooth teaching envelopes
/// rather than fitted curves: the optima are the textbook values below and
/// the widths are editorial half-widths chosen so that the envelope is near
/// full strength across the range each enzyme is usually described as
/// working over, and near zero outside it. What is claimed is the ORDERING
/// and the position of the optimum — pepsin faster in acid than in base,
/// every one of them slower in a refrigerator — never a rate constant.
///
/// Optima, from standard biochemistry teaching values:
///
/// * pepsin, pH about 1.5–2 and inactive by pH 6, which is the whole of
///   bio-049 and bio-050. Above pH 7 real pepsin is destroyed rather than
///   merely slowed; this model only slows it, and says so here.
/// * bromelain, pH about 5–6 over a broad plateau, optimum temperature
///   around 50 °C, and irreversibly destroyed by cooking — the reason
///   canned pineapple sets a jelly and fresh pineapple does not.
/// * beta-galactosidase (lactase), neutral to slightly acid, about pH 6.5.
/// * pancreatic lipase, alkaline, about pH 8, which is what the bile-salt
///   and small-intestine rows are about.
/// * catalase, near neutral, about pH 7.
/// * the bench's generic `protease` is not one enzyme and is given a wide
///   near-neutral window to say so.
///
/// Every optimum temperature is body temperature except bromelain's, which
/// is a plant enzyme; the widths are the ones the bounded activity model
/// already used before this table existed, so the previously reviewed
/// milk/gelatine/oil pairs keep exactly the behaviour they had.
pub const FAMILIES: &[EnzymeProfile] = &[
    EnzymeProfile {
        family: EnzymeFamily::Lactase,
        species: "lactase",
        acts_on: "lactose",
        products: "glucose and galactose",
        substrate: Some(SubstrateClass::Lactose),
        optimum_temperature_k: 310.15,
        temperature_width_k: 16.0,
        denatures_above_k: 333.15,
        optimum_ph: 6.5,
        ph_width: 2.0,
    },
    EnzymeProfile {
        family: EnzymeFamily::Protease,
        species: "protease",
        acts_on: "protein peptide bonds",
        products: "shorter peptides and amino acids",
        substrate: Some(SubstrateClass::Protein),
        optimum_temperature_k: 310.15,
        temperature_width_k: 18.0,
        denatures_above_k: 343.15,
        optimum_ph: 7.0,
        ph_width: 3.0,
    },
    EnzymeProfile {
        family: EnzymeFamily::Lipase,
        species: "lipase",
        acts_on: "triglycerides",
        products: "glycerol and fatty acids",
        substrate: Some(SubstrateClass::Triglyceride),
        optimum_temperature_k: 310.15,
        temperature_width_k: 18.0,
        denatures_above_k: 333.15,
        optimum_ph: 8.0,
        ph_width: 2.5,
    },
    EnzymeProfile {
        family: EnzymeFamily::Catalase,
        species: "catalase",
        acts_on: "hydrogen peroxide",
        products: "water and oxygen",
        // Catalase owns a curated stoichiometric reaction with its own
        // rate law. It carries its envelope here so the shelf can state
        // one, and deliberately no bounded-activity substrate.
        substrate: None,
        optimum_temperature_k: 310.15,
        temperature_width_k: 15.0,
        denatures_above_k: 328.15,
        optimum_ph: 7.0,
        ph_width: 2.5,
    },
    EnzymeProfile {
        family: EnzymeFamily::Pepsin,
        species: "pepsin",
        acts_on: "protein peptide bonds in acid",
        products: "shorter peptides and amino acids",
        substrate: Some(SubstrateClass::Protein),
        optimum_temperature_k: 310.15,
        temperature_width_k: 18.0,
        denatures_above_k: 333.15,
        optimum_ph: 1.8,
        ph_width: 1.2,
    },
    EnzymeProfile {
        family: EnzymeFamily::Bromelain,
        species: "bromelain",
        acts_on: "protein peptide bonds",
        products: "shorter peptides and amino acids",
        substrate: Some(SubstrateClass::Protein),
        optimum_temperature_k: 323.15,
        temperature_width_k: 18.0,
        denatures_above_k: 343.15,
        optimum_ph: 5.5,
        ph_width: 1.8,
    },
];

pub fn profile(species: &str) -> Option<&'static EnzymeProfile> {
    FAMILIES.iter().find(|profile| profile.species == species)
}

/// The catalyst a material recipe's enzyme-source role names.
pub fn family_of(species: &str) -> Option<EnzymeFamily> {
    profile(species).map(|profile| profile.family)
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
        electrical_resistivity: None,
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
    enzyme_species(
        "pepsin",
        34_500.0,
        "Enzyme-family teaching catalyst; approximate porcine pepsin dose mass (about 34.5 kDa), with no molecular-formula claim. The acidity window that makes it pepsin rather than a generic protease is in enzyme::FAMILIES, not here",
    ),
    enzyme_species(
        "bromelain",
        33_000.0,
        "Enzyme-family teaching catalyst; approximate stem-bromelain dose mass (about 33 kDa), with no molecular-formula claim. Fresh pineapple carries its own activity through a material recipe's enzyme-source role and does not weigh out this species",
    ),
];

pub(crate) fn species_data(key: &str) -> Option<&'static SpeciesData> {
    ADDITIONAL_ENZYME_SPECIES
        .iter()
        .find(|species| species.key == key)
}
