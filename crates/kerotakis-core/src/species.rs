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
    /// A salt that goes into solution but whose solution chemistry no
    /// wired engine models.
    ///
    /// Sodium thiosulfate is in no PHREEQC database we ship, so the aqueous
    /// engine cannot speciate it — but it is freely soluble, and leaving it
    /// sitting at the bottom of the beaker as a white solid is a *visibly*
    /// wrong observation about one of the commonest rate practicals. This
    /// flag says: it dissolves, and that is all we claim. No speciation, no
    /// contribution to pH or ionic strength, and the lab says so.
    #[serde(default)]
    pub dissolves_without_speciation: bool,
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

// Dissolved ions carry heat_capacity 0.0: partial molar heat capacities
// of aqueous ions are small (often negative) and are not modelled at this
// stage — the solution's heat capacity is carried by its water. Ion
// densities are unused (solution volume is carried by the liquid phase).
// The registry table is generated at build time from
// data/registry/registry-source-v1.json (CAP-21): the pack is the source
// of truth, the table stays `static` with `&'static str` fields at zero
// runtime cost, and adding a species is a data change, not a code
// change. tests/registry_snapshot.rs pins the generated table to the
// golden captured from the hand-written one it replaced; see build.rs
// for the join.
include!(concat!(env!("OUT_DIR"), "/species_generated.rs"));

/// The active registry. Currently the static REGISTRY; when the pack
/// loading path is wired (DATA-010), this will return pack-loaded data
/// with the same `&'static` lifetime via `OnceLock` + `Box::leak`.
pub fn registry() -> &'static [SpeciesData] {
    REGISTRY
}

/// OPT-4: O(1) species lookup via a lazily-built index.
///
/// The first call builds a HashMap from key → &SpeciesData; subsequent
/// calls are a single hash probe instead of a linear scan of 75+ entries.
fn lookup_index() -> &'static std::collections::HashMap<&'static str, &'static SpeciesData> {
    use std::sync::OnceLock;
    static INDEX: OnceLock<std::collections::HashMap<&'static str, &'static SpeciesData>> =
        OnceLock::new();
    INDEX.get_or_init(|| registry().iter().map(|s| (s.key, s)).collect())
}

pub fn lookup(id: &SpeciesId) -> Option<&'static SpeciesData> {
    lookup_index().get(id.0.as_str()).copied()
}

pub fn lookup_key(key: &str) -> Option<&'static SpeciesData> {
    lookup_index().get(key).copied()
}
