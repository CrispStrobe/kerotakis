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

/// A curated colour: sRGB plus how strongly it tints a solution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    /// Tinting strength, roughly "absorbance per mol/L through 1 cm".
    /// Permanganate is enormous (you can see 10⁻⁵ M); copper sulfate is
    /// mild (you need tenths of a mol).
    pub strength: f64,
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
    /// The colour this substance gives a flame, where it has a
    /// characteristic one. Curated: atomic emission is spectroscopy, not
    /// something the thermodynamic data knows.
    #[serde(default)]
    pub flame_colour: Option<&'static str>,
    /// Reflective colour: what a powder or lump *looks* like. This is
    /// scattering, not transmission, so it stays a plain sRGB value.
    #[serde(default)]
    pub colour: Option<Colour>,
    /// Absorption spectrum of the dissolved species, ε(λ) in
    /// L·mol⁻¹·cm⁻¹ across `spectrum::BAND_NM`. Where a species has one,
    /// solution colour is computed from Beer–Lambert and the CIE observer
    /// rather than tinted — so mixtures compose, concentration changes
    /// hue, and path length matters.
    #[serde(skip, default)]
    pub spectrum: Option<fn() -> crate::spectrum::Spectrum>,
    /// Enthalpy of dissolution in water, kJ/mol, positive = endothermic.
    /// Feeds the vessel energy balance: dissolving NaOH warms the beaker,
    /// dissolving ammonium nitrate would cool it. `None` = not curated yet
    /// (no heat effect is applied, honestly).
    pub dissolution_enthalpy_kj: Option<f64>,
    /// One-line provenance for the constants above.
    /// Some solids are the *stable* phase and still do not appear on a
    /// bench, because the metastable one nucleates first and then sits
    /// there — Ostwald's rule of stages. Copper(II) hydroxide is the
    /// classic case: tenorite (CuO) is more stable by ~1.0 log unit, yet
    /// adding lye to copper sulfate gives the pale blue hydroxide gel, and
    /// it is *heating* that turns it black.
    ///
    /// A Gibbs-minimising engine cannot discover that, because it is a
    /// statement about rates. So it is recorded here as data with its own
    /// provenance rather than special-cased in a solver: below this
    /// temperature the phase is not offered, and above it the engine is
    /// free to find it. `None` means no kinetic barrier is claimed.
    #[serde(default)]
    pub forms_only_above_k: Option<f64>,
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
        flame_colour: None,
        colour: Some(Colour { r: 255, g: 255, b: 255, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
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
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
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
        flame_colour: Some("bright yellow"),
        colour: Some(Colour { r: 250, g: 250, b: 250, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: Some(3.88),
        forms_only_above_k: None,
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
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: Some(22.6),
        forms_only_above_k: None,
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
        flame_colour: None,
        colour: Some(Colour { r: 248, g: 248, b: 246, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: Some(65.7),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "Ca(OH)2",
        name: "slaked lime (calcium hydroxide)",
        formula: "CaH2O2",
        inchikey: "AXCZMVOFGPJBDE-UHFFFAOYSA-L",
        molar_mass: 74.09,
        heat_capacity: 87.5,
        density: 2.21,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: Some("brick red"),
        colour: Some(Colour { r: 246, g: 246, b: 244, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: Some(-16.7),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, enthalpy of solution: standard reference values. Solubility and speciation computed by PHREEQC (Portlandite in wateq4f.dat/minteq.v4.dat)",
    },
    SpeciesData {
        key: "Cu(OH)2",
        name: "copper(II) hydroxide",
        formula: "CuH2O2",
        inchikey: "PTTPXKJBFFKCEK-UHFFFAOYSA-N",
        molar_mass: 97.56,
        heat_capacity: 96.0,
        density: 3.37,
        standard_phase: Phase::Solid,
        appearance: Some("pale blue"),
        flame_colour: None,
        colour: Some(Colour { r: 137, g: 199, b: 214, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values. Colour: the pale blue gelatinous precipitate of the classic school demonstration",
    },
    SpeciesData {
        key: "CuO",
        name: "copper(II) oxide",
        formula: "CuO",
        inchikey: "QPLDLSVMHZLSFG-UHFFFAOYSA-N",
        molar_mass: 79.55,
        heat_capacity: 42.3,
        density: 6.32,
        standard_phase: Phase::Solid,
        appearance: Some("black"),
        flame_colour: None,
        colour: Some(Colour { r: 26, g: 22, b: 20, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        // Tenorite is the thermodynamically stable copper(II) solid in
        // water - by 1.03 log units against Cu(OH)2 in minteq.v4 - and it
        // is still not what a beaker gives you at room temperature. The
        // hydroxide nucleates first and persists; boiling the suspension
        // is what turns it black, which is the demonstration. 340 K is the
        // temperature at which that conversion becomes brisk enough to
        // watch, and it is an editorial reading of a qualitative
        // observation rather than a measured rate constant.
        forms_only_above_k: Some(340.0),
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values. log_k for Cu(OH)2 (8.674) and Tenorite (7.644) from minteq.v4.dat (USGS/MINTEQA2). The 340 K kinetic threshold is Editorial judgement (Kerotakis): the blue-to-black conversion on warming is qualitative in every source we have, so the number is a stated stand-in for a rate we do not model, not a measurement",
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
        flame_colour: Some("bright yellow"),
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
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
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
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
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
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
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "HCl",
        name: "hydrochloric acid",
        formula: "HCl",
        inchikey: "VEXZGXHMUGYJMC-UHFFFAOYSA-N",
        molar_mass: 36.461,
        heat_capacity: 75.3,
        density: 1.19,
        standard_phase: Phase::Liquid,
        appearance: Some("colourless"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; modelled as concentrated aqueous acid, dilution heat not curated",
    },
    SpeciesData {
        key: "NaOH",
        name: "sodium hydroxide",
        formula: "NaOH",
        inchikey: "HEMHJVSKTPXQMS-UHFFFAOYSA-M",
        molar_mass: 39.997,
        heat_capacity: 59.5,
        density: 2.13,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: Some(-44.5),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy: standard reference values",
    },
    SpeciesData {
        key: "NH3",
        name: "ammonia solution",
        formula: "NH3(aq)",
        inchikey: "QGZKDVFQNNGYKY-UHFFFAOYSA-N",
        molar_mass: 17.031,
        heat_capacity: 80.0,
        density: 0.91,
        standard_phase: Phase::Liquid,
        appearance: Some("colourless, sharp smell"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; modelled as household ammonia solution (safety screening only at this stage)",
    },
    SpeciesData {
        key: "NaOCl",
        name: "bleach (sodium hypochlorite)",
        formula: "NaOCl(aq)",
        inchikey: "SUKJFIGYRHOWBL-UHFFFAOYSA-N",
        molar_mass: 74.442,
        heat_capacity: 75.0,
        density: 1.1,
        standard_phase: Phase::Liquid,
        appearance: Some("pale yellow-green"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; modelled as household bleach solution (safety screening only at this stage)",
    },
    SpeciesData {
        key: "NH2Cl",
        name: "chloramine",
        formula: "NH2Cl",
        inchikey: "QDHHCQZDFGDHMP-UHFFFAOYSA-N",
        molar_mass: 51.476,
        heat_capacity: 35.0,
        density: 1.0,
        standard_phase: Phase::Gas,
        appearance: Some("sharp-smelling, toxic"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; product of the curated bleach+ammonia entry",
    },
    SpeciesData {
        key: "Cl2",
        name: "chlorine gas",
        formula: "Cl2",
        inchikey: "KZBUYRJDOAKODT-UHFFFAOYSA-N",
        molar_mass: 70.906,
        heat_capacity: 33.9,
        density: 1.0,
        standard_phase: Phase::Gas,
        appearance: Some("yellow-green, toxic"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(g): standard reference values",
    },
    SpeciesData {
        key: "CH3COOH",
        name: "acetic acid",
        formula: "CH3COOH",
        inchikey: "QTBSBXVTEAMEQO-UHFFFAOYSA-N",
        molar_mass: 60.052,
        heat_capacity: 123.1,
        density: 1.049,
        standard_phase: Phase::Liquid,
        appearance: Some("colourless, vinegar smell"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(l), density: standard reference values",
    },
    SpeciesData {
        key: "NaOAc",
        name: "sodium acetate",
        formula: "CH3COONa",
        inchikey: "VMHLLURERBWHNL-UHFFFAOYSA-M",
        molar_mass: 82.034,
        heat_capacity: 79.9,
        density: 1.528,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: Some(-17.3),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy: standard reference values",
    },
    SpeciesData {
        key: "CH3COO-",
        name: "acetate ion",
        formula: "CH3COO-",
        inchikey: "QTBSBXVTEAMEQO-UHFFFAOYSA-M",
        molar_mass: 59.045,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "NaHCO3",
        name: "baking soda (sodium bicarbonate)",
        formula: "NaHCO3",
        inchikey: "UIIMBOGNXHQVGW-UHFFFAOYSA-M",
        molar_mass: 84.007,
        heat_capacity: 87.6,
        density: 2.20,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: Some(16.7),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy: standard reference values",
    },
    SpeciesData {
        key: "Na2CO3",
        name: "washing soda (sodium carbonate)",
        formula: "Na2CO3",
        inchikey: "CDBYLPFSWZWCQE-UHFFFAOYSA-L",
        molar_mass: 105.988,
        heat_capacity: 112.3,
        density: 2.54,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values; dissolution enthalpy not yet curated",
    },
    SpeciesData {
        key: "CO2",
        name: "carbon dioxide",
        formula: "CO2",
        inchikey: "CURLTUGMZLYLDI-UHFFFAOYSA-N",
        molar_mass: 44.009,
        heat_capacity: 37.1,
        density: 1.0,
        standard_phase: Phase::Gas,
        appearance: Some("colourless"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(g): standard reference values",
    },
    SpeciesData {
        key: "HCO3-",
        name: "bicarbonate ion",
        formula: "HCO3-",
        inchikey: "BVKZGUZCCUSVTD-UHFFFAOYSA-M",
        molar_mass: 61.017,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs); books total dissolved carbonate",
    },
    SpeciesData {
        key: "H3PO4",
        name: "phosphoric acid",
        formula: "H3PO4",
        inchikey: "NBIIXXVUZAFLBC-UHFFFAOYSA-N",
        molar_mass: 97.994,
        heat_capacity: 106.1,
        density: 1.88,
        standard_phase: Phase::Liquid,
        appearance: Some("colourless, syrupy"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; modelled as the concentrated syrupy acid",
    },
    SpeciesData {
        key: "H2PO4-",
        name: "dihydrogen phosphate ion",
        formula: "H2PO4-",
        inchikey: "NBIIXXVUZAFLBC-UHFFFAOYSA-M",
        molar_mass: 96.987,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs); books total dissolved phosphate",
    },
    SpeciesData {
        key: "KCl",
        name: "potassium chloride",
        formula: "KCl",
        inchikey: "WCUXLLCKKVVCTQ-UHFFFAOYSA-M",
        molar_mass: 74.551,
        heat_capacity: 51.3,
        density: 1.984,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: Some("lilac"),
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: Some(17.2),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy: standard reference values",
    },
    SpeciesData {
        key: "CaCl2",
        name: "calcium chloride",
        formula: "CaCl2",
        inchikey: "UXVMQQNJUSDDNG-UHFFFAOYSA-L",
        molar_mass: 110.98,
        heat_capacity: 72.9,
        density: 2.15,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: Some(-82.8),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy (anhydrous): standard reference values",
    },
    SpeciesData {
        key: "CaCO3",
        name: "chalk (calcium carbonate)",
        formula: "CaCO3",
        inchikey: "VTYYLEPIZMXCLO-UHFFFAOYSA-L",
        molar_mass: 100.087,
        heat_capacity: 81.9,
        density: 2.711,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: Some(Colour { r: 250, g: 250, b: 248, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density (calcite): standard reference values",
    },
    SpeciesData {
        key: "MgSO4",
        name: "magnesium sulfate",
        formula: "MgSO4",
        inchikey: "CSNNHWWHGAXBCP-UHFFFAOYSA-L",
        molar_mass: 120.366,
        heat_capacity: 96.5,
        density: 2.66,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values; dissolution enthalpy not curated (hydrate-dependent)",
    },
    SpeciesData {
        key: "gypsum",
        name: "gypsum (calcium sulfate dihydrate)",
        formula: "CaSO4·2H2O",
        inchikey: "PASHVRUKOFIRIK-UHFFFAOYSA-L",
        molar_mass: 172.171,
        heat_capacity: 186.0,
        density: 2.32,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "K+",
        name: "potassium ion",
        formula: "K+",
        inchikey: "NPYPAHLBTDXSSS-UHFFFAOYSA-N",
        molar_mass: 39.098,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: Some("lilac"),
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "Ca+2",
        name: "calcium ion",
        formula: "Ca+2",
        inchikey: "BHPQYMZQTOCNFJ-UHFFFAOYSA-N",
        molar_mass: 40.078,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: Some("orange-red"),
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "Mg+2",
        name: "magnesium ion",
        formula: "Mg+2",
        inchikey: "JLVVSXFLKOJNIY-UHFFFAOYSA-N",
        molar_mass: 24.305,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "SO4-2",
        name: "sulfate ion",
        formula: "SO4-2",
        inchikey: "QAOWNCQODCNURD-UHFFFAOYSA-L",
        molar_mass: 96.066,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: None,
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; ion Cp not modelled (see module docs)",
    },
    SpeciesData {
        key: "CaO",
        name: "quicklime (calcium oxide)",
        formula: "CaO",
        inchikey: "ODINCKMPIJJUCX-UHFFFAOYSA-N",
        molar_mass: 56.077,
        heat_capacity: 42.0,
        density: 3.34,
        standard_phase: Phase::Solid,
        appearance: Some("white"),
        flame_colour: None,
        colour: Some(Colour { r: 252, g: 250, b: 245, strength: 0.0 }),
        spectrum: None,
        // KNOWN GAP, stated rather than papered over. Slaking is violently
        // exothermic - CaO + H2O -> Ca(OH)2 releases about 82 kJ/mol, which
        // is why a bucket of quicklime steams and why it is a burn hazard.
        // The bench currently shows the vessel *cooling* instead, because
        // the energy balance only reads Dissolved and Precipitated events
        // and a solid turning into a different solid emits neither. Putting
        // the number here would not fix that; it would just look fixed.
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values. The heat of slaking is NOT modelled: see the note on dissolution_enthalpy_kj",
    },
    SpeciesData {
        key: "Mg",
        name: "magnesium",
        formula: "Mg",
        inchikey: "FYYHWMGAXLPEAU-UHFFFAOYSA-N",
        molar_mass: 24.305,
        heat_capacity: 24.9,
        density: 1.738,
        standard_phase: Phase::Solid,
        appearance: Some("silvery"),
        flame_colour: Some("a blinding white"),
        colour: Some(Colour { r: 200, g: 202, b: 205, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "MgO",
        name: "magnesium oxide",
        formula: "MgO",
        inchikey: "CPLXHLVBOLITMK-UHFFFAOYSA-N",
        molar_mass: 40.304,
        heat_capacity: 37.2,
        density: 3.58,
        standard_phase: Phase::Solid,
        appearance: Some("brilliant white"),
        flame_colour: None,
        colour: Some(Colour { r: 255, g: 255, b: 255, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density: standard reference values",
    },
    SpeciesData {
        key: "C",
        name: "carbon (charcoal)",
        formula: "C",
        inchikey: "OKTJSMMVPCPJKN-UHFFFAOYSA-N",
        molar_mass: 12.011,
        heat_capacity: 8.5,
        density: 2.26,
        standard_phase: Phase::Solid,
        appearance: Some("black"),
        flame_colour: None,
        colour: Some(Colour { r: 24, g: 24, b: 26, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s, graphite), density: standard reference values",
    },
    SpeciesData {
        key: "O2",
        name: "oxygen",
        formula: "O2",
        inchikey: "MYMOFIZGZYHOMD-UHFFFAOYSA-N",
        molar_mass: 31.998,
        heat_capacity: 29.4,
        density: 1.0,
        standard_phase: Phase::Gas,
        appearance: Some("colourless"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(g): standard reference values",
    },
    SpeciesData {
        key: "N2",
        name: "nitrogen",
        formula: "N2",
        inchikey: "IJGRMHOSHXDMSA-UHFFFAOYSA-N",
        molar_mass: 28.014,
        heat_capacity: 29.1,
        density: 1.0,
        standard_phase: Phase::Gas,
        appearance: Some("colourless"),
        flame_colour: None,
        colour: None,
        spectrum: None,
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(g): standard reference values",
    },
    SpeciesData {
        key: "CuSO4",
        name: "copper sulfate",
        formula: "CuSO4",
        inchikey: "ARUVKPQLZAKDPS-UHFFFAOYSA-L",
        molar_mass: 159.609,
        heat_capacity: 100.0,
        density: 3.60,
        standard_phase: Phase::Solid,
        appearance: Some("white when dry"),
        flame_colour: Some("blue-green"),
        colour: Some(Colour { r: 245, g: 245, b: 240, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: Some(-73.1),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy (anhydrous): standard reference values",
    },
    SpeciesData {
        key: "Cu+2",
        name: "copper(II) ion",
        formula: "Cu+2",
        inchikey: "JPVYNHNXODAKFH-UHFFFAOYSA-N",
        molar_mass: 63.546,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: Some("blue"),
        flame_colour: Some("blue-green"),
        colour: Some(Colour { r: 40, g: 110, b: 210, strength: 60.0 }),
        // [Cu(H2O)6]2+ absorbs through a broad, weak d–d band peaking in
        // the near infrared (~810 nm, ε ≈ 12). Only its tail reaches the
        // visible, which is why the solution is *pale* blue and why you
        // need tenths of a mole to see anything at all.
        spectrum: Some(|| crate::spectrum::edge(0.6, 11.0)),
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; colour curated, strength set from the published molar absorptivity of [Cu(H2O)6]2+ (ε ≈ 12 L/mol/cm) over a beaker-sized path",
    },
    SpeciesData {
        key: "KMnO4",
        name: "potassium permanganate",
        formula: "KMnO4",
        inchikey: "VZJVWSHVAAUDKD-UHFFFAOYSA-N",
        molar_mass: 158.034,
        heat_capacity: 119.2,
        density: 2.70,
        standard_phase: Phase::Solid,
        appearance: Some("dark purple crystals"),
        flame_colour: Some("lilac"),
        colour: Some(Colour { r: 60, g: 20, b: 70, strength: 0.0 }),
        spectrum: None,
        dissolution_enthalpy_kj: Some(16.2),
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; Cp(s), density, dissolution enthalpy: standard reference values",
    },
    SpeciesData {
        key: "MnO4-",
        name: "permanganate ion",
        formula: "MnO4-",
        inchikey: "VLTRZXGMWDSKGL-UHFFFAOYSA-M",
        molar_mass: 118.934,
        heat_capacity: 0.0,
        density: 1.0,
        standard_phase: Phase::Aqueous,
        appearance: Some("intense purple"),
        flame_colour: None,
        colour: Some(Colour { r: 120, g: 10, b: 140, strength: 12000.0 }),
        // MnO4- absorbs by ligand-to-metal charge transfer: an intense
        // band at ~525 nm (ε ≈ 2400) with the characteristic vibrational
        // shoulders either side. Absorbing green is exactly why it looks
        // purple, and the intensity is why it is visible at 1e-5 M — which
        // is what makes it a self-indicating titrant.
        spectrum: Some(|| {
            crate::spectrum::bands(&[
                (525.0, 2400.0, 22.0),
                (546.0, 1900.0, 20.0),
                (505.0, 1500.0, 18.0),
                (567.0, 900.0, 18.0),
            ])
        }),
        dissolution_enthalpy_kj: None,
        forms_only_above_k: None,
        provenance: "M from IUPAC/CIAAW 2021 atomic weights; colour curated, strength set from the published molar absorptivity of MnO4- (ε ≈ 2400 L/mol/cm at 525 nm) — visible at 1e-5 M, which is why it is the classic titration indicator",
    },
];

pub fn lookup(id: &SpeciesId) -> Option<&'static SpeciesData> {
    REGISTRY.iter().find(|s| s.key == id.0)
}

pub fn lookup_key(key: &str) -> Option<&'static SpeciesData> {
    REGISTRY.iter().find(|s| s.key == key)
}
