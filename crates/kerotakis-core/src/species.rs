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

#[derive(PartialOrd, Ord, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Solid,
    Liquid,
    Aqueous,
    Gas,
}

/// Registry entry for one species.
/// The shelf swatch: what a bottle of this substance LOOKS like, from the
/// same curated data the bench renders with. Reflective colour for the
/// substance itself (scattering); where a solution spectrum exists, also
/// the transmitted tint of a 0.1 mol/L solution through 1 cm — the glance
/// into a reagent bottle, computed rather than painted.
pub fn shelf_swatch(s: &SpeciesData) -> (Option<[u8; 3]>, Option<[u8; 3]>) {
    let reflective = s.colour.map(|c| [c.r, c.g, c.b]);
    let solution = s.spectrum.map(|eps| {
        let mut a = [0.0f64; crate::spectrum::BANDS];
        for (band, e) in a.iter_mut().zip(eps.iter()) {
            *band = e * 0.1 * 1.0;
        }
        let rgb = crate::spectrum::transmitted_colour(&a);
        [rgb.r, rgb.g, rgb.b]
    });
    (reflective, solution)
}

/// EXP-33: where a pure substance changes state, and what it does instead
/// when it has no sharp point to change state at.
///
/// This is the data behind the melting-point apparatus, and it is curated
/// per record rather than computed: a melting point is a measured constant,
/// not something a Gibbs minimiser on this bench can find. Every field is
/// optional because "we do not know" and "it does not happen" are different
/// answers and both are common — potassium permanganate has no melting
/// point to record because it decomposes first, and saying so is the
/// chemistry, not a gap.
///
/// All temperatures are kelvin, at 1 atm.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransitions {
    /// Normal melting point. `None` where the substance decomposes or
    /// sublimes instead, or where no value is curated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melting_k: Option<f64>,
    /// Normal boiling point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boiling_k: Option<f64>,
    /// The temperature at which the solid passes straight to vapour at
    /// 1 atm — set only where sublimation, not melting, is what a bench
    /// actually sees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublimation_k: Option<f64>,
    /// The substance comes apart here instead of melting. Recording it is
    /// the honest answer to "what is its melting point?" for every salt of
    /// an oxidising anion and every sugar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decomposition_k: Option<f64>,
    /// Hydrates only: where this bench drives the waters of crystallisation
    /// off. See `crate::phase_route` for what that claim does and does not
    /// include.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dehydration_k: Option<f64>,
    /// Per-record provenance for the temperatures above — not the species'
    /// general citation, because these values have their own source.
    pub source: &'static str,
    /// What this row does NOT claim, shown at lv3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<&'static str>,
}

impl PhaseTransitions {
    /// The temperature a melting-point apparatus would report, and whether
    /// it is really a melting point at all.
    pub fn melting_reading(&self) -> Option<(f64, TransitionOutcome)> {
        if let Some(k) = self.melting_k {
            return Some((k, TransitionOutcome::Melts));
        }
        if let Some(k) = self.sublimation_k {
            return Some((k, TransitionOutcome::Sublimes));
        }
        if let Some(k) = self.decomposition_k {
            return Some((k, TransitionOutcome::Decomposes));
        }
        self.dehydration_k
            .map(|k| (k, TransitionOutcome::LosesWater))
    }

    /// The temperature a boiling apparatus would report.
    pub fn boiling_reading(&self) -> Option<(f64, TransitionOutcome)> {
        if let Some(k) = self.boiling_k {
            return Some((k, TransitionOutcome::Boils));
        }
        if let Some(k) = self.sublimation_k {
            return Some((k, TransitionOutcome::Sublimes));
        }
        self.decomposition_k
            .map(|k| (k, TransitionOutcome::Decomposes))
    }
}

/// What actually happens when a pure sample is heated past its recorded
/// temperature. A melting point that is really a decomposition is one of
/// the commonest things a school table hides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionOutcome {
    Melts,
    Boils,
    Sublimes,
    Decomposes,
    LosesWater,
}

impl TransitionOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Melts => "melts",
            Self::Boils => "boils",
            Self::Sublimes => "sublimes",
            Self::Decomposes => "decomposes",
            Self::LosesWater => "loses its water of crystallisation",
        }
    }
}

/// A dry solid's bulk electrical resistivity, with the citation that has
/// to travel with it.
///
/// This is a different claim from anything in [`PhaseTransitions`] and a
/// different claim from the aqueous conductivity the meter usually reports:
/// a solution conducts because ions move through it, and a metal conducts
/// because its electrons are not attached to any one atom. Sharing a field
/// between the two would let a reader mistake one for the other, so they
/// do not share one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Resistivity {
    /// Ohm-metres at 293.15 K.
    pub ohm_m: f64,
    /// Per-record provenance - this value's own source, not the species'
    /// general citation.
    pub source: &'static str,
    /// What this row does NOT claim: purity, temper, alloy, anisotropy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<&'static str>,
}

impl Resistivity {
    /// The reciprocal, S/m - what a conductance meter reads directly.
    /// Resistivity is what the handbooks tabulate, so it is what the
    /// registry stores; the meter inverts it rather than storing both.
    pub fn conductivity_s_per_m(&self) -> f64 {
        1.0 / self.ohm_m
    }
}

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
    pub spectrum: Option<&'static crate::spectrum::Spectrum>,
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
    /// Conservative room-temperature aqueous solubility limit in grams of
    /// solute per 100 mL liquid water. `None` means no finite dissolution
    /// model is installed; this is deliberately separate from speciation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aqueous_solubility_g_per_100_ml: Option<f64>,
    /// KID-7: the same limit at 100 °C, where a reviewed second point
    /// exists. Two points make a line, and a line is the whole of the
    /// temperature dependence this bench claims — enough for the one
    /// observation the crystal experiments are run to make, which is that
    /// hot water holds far more sugar than cold water does.
    ///
    /// `None` means the solute is modelled as temperature-independent, and
    /// the bench says so rather than extrapolating from one point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aqueous_solubility_g_per_100_ml_at_100c: Option<f64>,
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
    #[serde(default)]
    pub magnetic: bool,
    /// EXP-33: melting, boiling, sublimation, decomposition, dehydration —
    /// with their own citation. `None` means no transition temperature has
    /// been curated for this species and the apparatus says so rather than
    /// guessing one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transitions: Option<PhaseTransitions>,
    /// Bulk electrical resistivity of the dry solid, with its own citation.
    /// `None` means none is curated and the conductance meter says so
    /// rather than guessing one - which is the whole difference between a
    /// bench that can say why wire is copper and one that cannot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electrical_resistivity: Option<Resistivity>,
    pub provenance: &'static str,
}

/// KID-2 / KID-10: are these two species the same Brønsted acid, one of
/// them having lost a proton?
///
/// A Brønsted acid and its conjugate base differ only in hydrogen count and
/// charge, so the family is the set of species sharing a non-hydrogen
/// composition. This exists because the aqueous solver rewrites the ledger:
/// pour vinegar into water and what remains is `CH3COO-`, not `CH3COOH`.
/// Any model that looks up an acid *by key* therefore stops working the
/// moment the solver runs, and it does so silently.
///
/// That has now been the cause of three separate defects — milk that would
/// not curdle (KID-2), vinegar that had no smell (KID-10), and it is worth
/// keeping in one place so the fourth is a lookup rather than a rediscovery.
pub fn same_bronsted_family(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let skeleton = |key: &str| -> Option<Vec<(String, f64)>> {
        let formula = crate::stoich::parse_formula(lookup_key(key).map(|d| d.formula)?).ok()?;
        Some(
            formula
                .counts
                .iter()
                .filter(|(element, _)| element.as_str() != "H")
                .map(|(element, count)| (element.clone(), *count))
                .collect(),
        )
    };
    match (skeleton(a), skeleton(b)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

impl SpeciesData {
    /// KID-7: how much of this solute 100 mL of water holds at `t_k`.
    ///
    /// Linear between the reviewed 20 °C and 100 °C points and held flat
    /// outside them: sucrose's real curve bends upward, so the midpoint is
    /// about 5% low, which is inside what a school experiment can see and
    /// far inside the difference this exists to show. A solute with only one
    /// reviewed point is returned unchanged at every temperature.
    pub fn aqueous_solubility_at(&self, t_k: f64) -> Option<f64> {
        let cold = self.aqueous_solubility_g_per_100_ml?;
        let Some(hot) = self.aqueous_solubility_g_per_100_ml_at_100c else {
            return Some(cold);
        };
        const COLD_K: f64 = 293.15;
        const HOT_K: f64 = 373.15;
        let f = ((t_k - COLD_K) / (HOT_K - COLD_K)).clamp(0.0, 1.0);
        Some(cold + f * (hot - cold))
    }

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

// ── DATA-010: the loaded-species overlay ────────────────────────────
// Pack-loaded species live beside the compiled REGISTRY: the static
// table stays zero-cost, and loaded entries are leaked once per pack
// per session. Built-ins always win a key collision — a pack cannot
// silently redefine the chemistry the tests pinned.

fn loaded_index(
) -> &'static std::sync::RwLock<std::collections::HashMap<String, &'static SpeciesData>> {
    use std::sync::OnceLock;
    static LOADED: OnceLock<
        std::sync::RwLock<std::collections::HashMap<String, &'static SpeciesData>>,
    > = OnceLock::new();
    LOADED.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()))
}

/// Register pack-loaded species. Returns (added, skipped): built-ins
/// and already-loaded keys are skipped, never replaced.
pub fn register_loaded(list: Vec<SpeciesData>) -> (usize, usize) {
    let mut added = 0;
    let mut skipped = 0;
    let mut map = loaded_index().write().expect("loaded-species lock");
    for s in list {
        if lookup_index().contains_key(s.key) || map.contains_key(s.key) {
            skipped += 1;
            continue;
        }
        let leaked: &'static SpeciesData = Box::leak(Box::new(s));
        map.insert(leaked.key.to_string(), leaked);
        added += 1;
    }
    (added, skipped)
}

/// Every species the lab knows: the compiled registry plus everything
/// loaded from packs — the shelf's honest inventory.
pub fn all_species() -> Vec<&'static SpeciesData> {
    let map = loaded_index().read().expect("loaded-species lock");
    registry()
        .iter()
        .chain(crate::enzyme::ADDITIONAL_ENZYME_SPECIES.iter())
        .chain(map.values().copied())
        .collect()
}

/// How many pack-loaded species are active.
pub fn loaded_count() -> usize {
    loaded_index().read().expect("loaded-species lock").len()
}

pub fn lookup(id: &SpeciesId) -> Option<&'static SpeciesData> {
    lookup_key(id.0.as_str())
}

pub fn lookup_key(key: &str) -> Option<&'static SpeciesData> {
    if let Some(s) = lookup_index().get(key).copied() {
        return Some(s);
    }
    if let Some(s) = crate::enzyme::species_data(key) {
        return Some(s);
    }
    loaded_index()
        .read()
        .expect("loaded-species lock")
        .get(key)
        .copied()
}
