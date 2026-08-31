//! # kerotakis-safety
//!
//! The L0 reactivity screen: runs before any chemistry, on the *prospective*
//! vessel state (PLAN.md, L0).
//!
//! This is a pedagogical tool, so known hazards produce a **strong warning
//! and then proceed** — being precise about what would happen is the lesson,
//! and the virtual lab is the one place it can be watched safely. The actual
//! outcome (chloramine gas, chlorine gas) is computed by the curated
//! reaction layer; this screen's job is to make sure the warning always
//! comes first. `Veto` is reserved for the product-safety boundary.
//!
//! The matrix is our own reimplementation of the reactive-group
//! compatibility methodology published by NOAA's Office of Response and
//! Restoration (see PLAN.md for sourcing and the legal position). Every
//! entry is our own encoding of published, non-copyrightable reactivity
//! facts.
//!
//! CAP-11: every registry species has an explicit reactive-group row
//! (totality enforced by CI); the incompatibility matrix covers
//! the groups those species populate.

use kerotakis_core::{SafetyScreen, SafetyVerdict, Severity, Vessel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactiveGroup {
    AcidStrong,
    BaseStrong,
    OxidizerStrong,
    OxidizerHypochlorite,
    ReducingAgent,
    ActiveMetal,
    FlammableLiquid,
    FlammableGas,
    WaterReactive,
    AmmoniaAmines,
    Carbonate,
    /// A salt whose solution is acidic by hydrolysis rather than by
    /// dissociation: iron(III) chloride reaches about pH 2 at 0.1 mol/L
    /// and etches metals, but it is not a strong acid and does not carry
    /// `AcidStrong`'s rules. The group states what the substance is; the
    /// mixture rules stay with the chemistry that is modelled.
    AcidicSalt,
    /// A soluble salt whose *dissolved cation* is an acute systemic
    /// toxicant. NOAA's methodology screens mixtures, not intrinsic
    /// toxicity, so this row is the same kind of extension
    /// `AmmoniaAmines`'s "toxic" label already is: our own encoding of a
    /// published, non-copyrightable fact about the substance, carried so
    /// the shelf cannot show a soluble barium salt with no hazard on it.
    /// Membership follows solubility — barite is the counter-example, and
    /// deliberately not a member.
    ToxicSoluble,
}

pub fn hazard_labels(species_key: &str) -> Vec<&'static str> {
    let (labels, _) = hazard_assessment(species_key);
    labels
}

#[derive(Debug, Clone, Serialize)]
pub struct HazardInfo {
    pub assessed: bool,
    pub classes: Vec<&'static str>,
}

pub fn hazard_assessment(species_key: &str) -> (Vec<&'static str>, bool) {
    let assessed = COVERED_KEYS.contains(&species_key);
    let labels: Vec<&'static str> = groups(species_key)
        .iter()
        .map(|g| match g {
            ReactiveGroup::AcidStrong => "corrosive",
            ReactiveGroup::BaseStrong => "corrosive",
            ReactiveGroup::OxidizerStrong => "oxidiser",
            ReactiveGroup::OxidizerHypochlorite => "oxidiser",
            ReactiveGroup::ReducingAgent => "reducing_agent",
            ReactiveGroup::ActiveMetal => "flammable_solid",
            ReactiveGroup::FlammableLiquid => "flammable",
            ReactiveGroup::FlammableGas => "flammable",
            ReactiveGroup::WaterReactive => "water_reactive",
            ReactiveGroup::AmmoniaAmines => "toxic",
            ReactiveGroup::Carbonate => "irritant",
            ReactiveGroup::AcidicSalt => "irritant",
            ReactiveGroup::ToxicSoluble => "toxic",
        })
        .collect();
    (labels, assessed)
}

/// Registry-key → reactive groups. Assignment is total over the registry;
/// the CI totality test (`tests/totality.rs`) fails if a species is added
/// without a row here. Inert species (solvents, spectator ions, salts,
/// indicators) get `&[]`.
///
/// Provenance: each assignment follows from the species' chemical identity
/// and the published NOAA reactive-group definitions. Acids dissociate
/// completely ⇒ AcidStrong; metals above hydrogen in the activity series
/// ⇒ ActiveMetal; etc. No database exports are used.
pub fn groups(species_key: &str) -> &'static [ReactiveGroup] {
    use ReactiveGroup::*;
    match species_key {
        // ── strong acids ──────────────────────────────────────────
        "HCl" | "HI" | "HBr" => &[AcidStrong],
        "H2SO4" | "NaHSO4" => &[AcidStrong],

        // ── strong bases ──────────────────────────────────────────
        "NaOH" => &[BaseStrong],
        "KOH" => &[BaseStrong],
        "OH-" => &[BaseStrong],
        "Ca(OH)2" => &[BaseStrong],
        // Barium hydroxide is both: a strong base and a soluble barium
        // salt. Both rows fire.
        "Ba(OH)2" => &[BaseStrong, ToxicSoluble],

        // ── strong oxidizers ──────────────────────────────────────
        "H2O2" => &[OxidizerStrong],
        "KMnO4" | "KIO3" => &[OxidizerStrong],
        "Cl2" | "I2" => &[OxidizerStrong],
        "MnO4-" => &[OxidizerStrong],

        // ── hypochlorite (specific sub-class of oxidizer) ─────────
        "NaOCl" => &[OxidizerHypochlorite],

        // ── reducing agents ───────────────────────────────────────
        "Na2SO3" | "NaHSO3" => &[ReducingAgent],
        "Na2S2O3" => &[ReducingAgent],
        "KI" => &[ReducingAgent],
        "ascorbic_acid" => &[ReducingAgent],

        // ── active metals (above H in activity series) ────────────
        "Al" => &[ActiveMetal],
        "Mg" => &[ActiveMetal],
        "Zn" => &[ActiveMetal],
        "Fe" => &[ActiveMetal],
        "Pb" => &[ActiveMetal],

        // ── flammable liquids ─────────────────────────────────────
        "ethanol" | "isopropanol" => &[FlammableLiquid],
        "methanol" => &[FlammableLiquid],
        "hexane" => &[FlammableLiquid],
        "propanone" => &[FlammableLiquid],
        "ethyl_acetate" => &[FlammableLiquid],
        "bromoethane" | "tert_butyl_bromide" => &[FlammableLiquid],
        "tert_butanol" => &[FlammableLiquid],

        // ── flammable gas ─────────────────────────────────────────
        "H2" => &[FlammableGas],
        "ethene" | "isobutylene" => &[FlammableGas],

        // ── water-reactive ────────────────────────────────────────
        "CaO" => &[WaterReactive],

        // ── ammonia / amines ──────────────────────────────────────
        "NH3" => &[AmmoniaAmines],

        // ── carbonates (vigorous CO₂ release with strong acid) ────
        "CaCO3" => &[Carbonate],
        "Na2CO3" => &[Carbonate],
        "NaHCO3" => &[Carbonate],

        // ── acidic salts (hydrolysis, not dissociation) ───────────
        "FeCl3" => &[AcidicSalt],

        // ── soluble barium: acutely toxic by ingestion ────────────
        // BRD-012's P2 gate. These are virtual-lab reagents; nothing in
        // the material packs presents them as household supplies, and the
        // shelf shows them as toxic wherever it shows hazards at all.
        // BaSO4 is deliberately absent: the insoluble sulfate is the form
        // people swallow for a radiograph, and that contrast is the
        // pedagogical point of the sulfate test.
        "BaCl2" => &[ToxicSoluble],
        "Ba+2" => &[ToxicSoluble],

        // ── inert: solvents, salts, oxides, ions, indicators ──────
        "water"
        | "NaCl"
        | "AgNO3"
        | "AgCl"
        | "catalase"
        | "MnO2"
        | "S"
        | "SO2"
        | "Cu(OH)2"
        | "Fe(OH)2"
        | "Fe(OH)3"
        | "Fe2O3"
        | "Mg(OH)2"
        | "Zn(OH)2"
        | "CuO"
        | "Na+"
        | "Br-"
        | "Cl-"
        | "Ag+"
        | "NO3-"
        | "NH2Cl"
        | "CH3COOH"
        | "NaOAc"
        | "CH3COO-"
        | "CO2"
        | "HCO3-"
        | "H3PO4"
        | "H2PO4-"
        | "KCl"
        | "CaCl2"
        | "MgSO4"
        | "epsomite"
        | "gypsum"
        | "K+"
        | "Ca+2"
        | "Mg+2"
        | "Sr+2"
        | "SO4-2"
        | "Cu"
        | "Ag"
        | "MgO"
        | "C"
        | "O2"
        | "N2"
        | "CuSO4"
        | "Cu+2"
        | "FeSO4"
        | "Fe+2"
        | "Fe+3"
        | "Cu+1"
        | "Mn+2"
        | "MnO4-2"
        | "Mn+3"
        | "phenolphthalein"
        | "methyl_orange"
        | "bromothymol_blue"
        | "Zn+2"
        | "ZnSO4"
        | "Pb+2"
        | "Pb(NO3)2"
        | "PE"
        | "PP"
        | "PET"
        | "PS"
        | "betanin"
        | "betanin_ox"
        | "curcumin"
        | "curcumin_ox"
        | "indigo_carmine"
        | "indigo_carmine_ox"
        | "NaNO3"
        | "KNO3"
        | "dehydroascorbic_acid"
        | "starch"
        | "sucrose"
        | "amylase"
        | "maltose"
        | "SiO2"
        // Ammonium chloride and sodium sulfate are the school shelf's
        // neutral-to-mildly-acidic salts: no NOAA reactive group of their
        // own. Ammonium salts DO liberate ammonia with a strong base and
        // chloramines with bleach; neither rule is in the matrix yet, so
        // neither is claimed here rather than approximated by filing them
        // under `AmmoniaAmines`, whose rules describe ammonia itself.
        | "NH4Cl"
        | "NH4+"
        | "Na2SO4"
        // The insoluble sulfate: barium that cannot be absorbed.
        | "BaSO4"
        | "NaBr" => &[],

        _ => &[],
    }
}

/// Every species key covered by `groups()`. The totality test checks
/// this set against `species::registry()` — a new species without an
/// entry here fails CI.
pub const COVERED_KEYS: &[&str] = &[
    "Ag",
    "Ag+",
    "AgCl",
    "Al",
    "betanin",
    "betanin_ox",
    "AgNO3",
    "Ba+2",
    "Ba(OH)2",
    "BaCl2",
    "BaSO4",
    "Br-",
    "C",
    "CO2",
    "Ca(OH)2",
    "Ca+2",
    "CaCO3",
    "CaCl2",
    "CaO",
    "CH3COO-",
    "CH3COOH",
    "Cl-",
    "Cl2",
    "Cu",
    "Cu+1",
    "curcumin",
    "curcumin_ox",
    "Cu+2",
    "CuO",
    "Cu(OH)2",
    "CuSO4",
    "Fe",
    "Fe+2",
    "Fe+3",
    "Fe(OH)2",
    "Fe(OH)3",
    "Fe2O3",
    "FeCl3",
    "FeSO4",
    "H2",
    "H2O2",
    "H2PO4-",
    "H2SO4",
    "H3PO4",
    "HCl",
    "HBr",
    "HCO3-",
    "HI",
    "I2",
    "K+",
    "KCl",
    "KI",
    "KIO3",
    "KMnO4",
    "KNO3",
    "Mg",
    "Mg+2",
    "Mg(OH)2",
    "MgO",
    "MgSO4",
    "Mn+2",
    "Mn+3",
    "MnO2",
    "MnO4-",
    "MnO4-2",
    "N2",
    "NH2Cl",
    "NH3",
    "NH4+",
    "NH4Cl",
    "NO3-",
    "Na+",
    "NaBr",
    "Na2CO3",
    "Na2S2O3",
    "Na2SO3",
    "Na2SO4",
    "NaCl",
    "NaNO3",
    "NaHCO3",
    "NaHSO3",
    "NaHSO4",
    "NaOAc",
    "NaOCl",
    "KOH",
    "NaOH",
    "OH-",
    "O2",
    "Pb",
    "Pb(NO3)2",
    "Pb+2",
    "PE",
    "PET",
    "PP",
    "PS",
    "S",
    "SiO2",
    "SO2",
    "SO4-2",
    "Sr+2",
    "Zn",
    "Zn(OH)2",
    "Zn+2",
    "ZnSO4",
    "amylase",
    "ascorbic_acid",
    "bromoethane",
    "bromothymol_blue",
    "catalase",
    "dehydroascorbic_acid",
    "ethanol",
    "ethene",
    "ethyl_acetate",
    "epsomite",
    "gypsum",
    "hexane",
    "indigo_carmine",
    "indigo_carmine_ox",
    "isobutylene",
    "isopropanol",
    "maltose",
    "methanol",
    "methyl_orange",
    "phenolphthalein",
    "propanone",
    "starch",
    "sucrose",
    "tert_butanol",
    "tert_butyl_bromide",
    "water",
];

struct Incompatibility {
    a: ReactiveGroup,
    b: ReactiveGroup,
    severity: Severity,
    /// Stable machine identity, carried through `ExposureFinding` and
    /// `SafetyVerdict::Warn` into the `HazardWarning` event so contracts
    /// and tests can recognise WHICH rule fired without matching
    /// localized prose.
    rule: &'static str,
    hazard: &'static str,
    real_world: &'static str,
}

/// One precisely attributed hazard found across exposed material.
///
/// Unlike [`SafetyVerdict`], this record retains the species and locations
/// that triggered the rule. Spill handling uses that attribution to explain
/// whether the danger was already present in one puddle or was created when
/// material reached an occupied spill compartment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureFinding {
    pub severity: Severity,
    /// Stable machine identity of the rule (untranslated across locales;
    /// additive, defaults empty on older serialized findings).
    #[serde(default)]
    pub rule: String,
    pub hazard: String,
    pub real_world: String,
    /// The two species participating in the rule. For a water-reactive rule,
    /// these are the water-reactive species and water, in that order.
    pub species: [String; 2],
    /// Stable caller-provided location labels corresponding to `species`.
    pub locations: [String; 2],
}

#[derive(Debug, Clone)]
struct PresentSpecies<'a> {
    key: &'a str,
    location: &'a str,
    groups: &'static [ReactiveGroup],
}

/// Assess all material sharing an exposed area, including incompatibilities
/// whose two reactants came from different vessels or spill deposits.
///
/// `location` is deliberately an opaque stable label: core can pass a spill
/// destination while tests and other clients can use their own identifiers.
/// Findings are deterministic, deduplicated, and retain enough attribution
/// for a precise safety event or notebook entry.
pub fn assess_exposures<'a>(
    exposures: impl IntoIterator<Item = (&'a str, &'a Vessel)>,
) -> Vec<ExposureFinding> {
    let mut present = Vec::new();
    for (location, vessel) in exposures {
        for portion in &vessel.contents {
            if portion.moles.0 <= 1e-12 {
                continue;
            }
            present.push(PresentSpecies {
                key: portion.species.0.as_str(),
                location,
                groups: groups(portion.species.0.as_str()),
            });
        }
    }

    let mut findings = Vec::new();
    // Preserve the historical screen priority: water-reactive exposure is
    // reported first even if another incompatible pair appears earlier in
    // vessel order.
    for (index, a) in present.iter().enumerate() {
        for b in present.iter().skip(index + 1) {
            let water_reactive =
                if a.key == "water" && b.groups.contains(&ReactiveGroup::WaterReactive) {
                    Some((b, a))
                } else if b.key == "water" && a.groups.contains(&ReactiveGroup::WaterReactive) {
                    Some((a, b))
                } else {
                    None
                };
            if let Some((reactive, water)) = water_reactive {
                push_unique(
                    &mut findings,
                    ExposureFinding {
                        severity: Severity::Caution,
                        rule: "water-reactive-slaking".to_string(),
                        hazard: "this substance reacts violently with water, releasing a large amount of heat".to_string(),
                        real_world: "Quicklime (CaO) in water can reach 100 °C and cause severe burns. Always add the solid to the water slowly, never the reverse.".to_string(),
                        species: [reactive.key.to_string(), water.key.to_string()],
                        locations: [reactive.location.to_string(), water.location.to_string()],
                    },
                );
            }
        }
    }

    for (index, a) in present.iter().enumerate() {
        for b in present.iter().skip(index + 1) {
            for inc in INCOMPATIBLE {
                if a.groups.contains(&inc.a) && b.groups.contains(&inc.b)
                    || a.groups.contains(&inc.b) && b.groups.contains(&inc.a)
                {
                    push_unique(
                        &mut findings,
                        ExposureFinding {
                            severity: inc.severity,
                            rule: inc.rule.to_string(),
                            hazard: inc.hazard.to_string(),
                            real_world: inc.real_world.to_string(),
                            species: [a.key.to_string(), b.key.to_string()],
                            locations: [a.location.to_string(), b.location.to_string()],
                        },
                    );
                }
            }
        }
    }
    findings
}

fn push_unique(findings: &mut Vec<ExposureFinding>, finding: ExposureFinding) {
    let duplicate = findings.iter().any(|existing| {
        existing.hazard == finding.hazard
            && ((existing.species == finding.species && existing.locations == finding.locations)
                || (existing.species == [finding.species[1].clone(), finding.species[0].clone()]
                    && existing.locations
                        == [finding.locations[1].clone(), finding.locations[0].clone()]))
    });
    if !duplicate {
        findings.push(finding);
    }
}

/// The incompatibility matrix. Symmetric; checked both ways.
const INCOMPATIBLE: &[Incompatibility] = &[
    // ── toxic-gas producers ───────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::OxidizerHypochlorite,
        b: ReactiveGroup::AmmoniaAmines,
        severity: Severity::Danger,
        rule: "bleach-ammonia-chloramine",
        hazard: "mixing bleach with ammonia makes chloramine, a toxic gas",
        real_world: "People are hospitalised every year from mixing \
                     these two household cleaners.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerHypochlorite,
        b: ReactiveGroup::AcidStrong,
        severity: Severity::Danger,
        rule: "bleach-acid-chlorine",
        hazard: "mixing bleach with acid releases chlorine, a toxic gas",
        real_world: "Chlorine gas was used as a chemical weapon; even \
                     small amounts damage lungs.",
    },
    // ── fire / explosion risks ────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::OxidizerStrong,
        b: ReactiveGroup::FlammableLiquid,
        severity: Severity::Danger,
        rule: "oxidizer-flammable-liquid",
        hazard: "a strong oxidizer mixed with a flammable liquid \
                 can ignite or explode",
        real_world: "Potassium permanganate and glycerol ignite on \
                     contact; similar mixtures cause laboratory fires.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerStrong,
        b: ReactiveGroup::FlammableGas,
        severity: Severity::Danger,
        rule: "oxidizer-flammable-gas",
        hazard: "a strong oxidizer in the presence of a flammable gas \
                 creates an explosion risk",
        real_world: "Hydrogen and chlorine mixtures can detonate when \
                     exposed to light or a spark.",
    },
    Incompatibility {
        a: ReactiveGroup::OxidizerStrong,
        b: ReactiveGroup::ReducingAgent,
        severity: Severity::Danger,
        rule: "oxidizer-reducer",
        hazard: "mixing a strong oxidizer with a reducing agent can \
                 cause a violent, potentially explosive reaction",
        real_world: "Permanganate and sulfite solutions react vigorously; \
                     at scale, such mixtures can detonate.",
    },
    // ── acid + metal → hydrogen gas ───────────────────────────────
    Incompatibility {
        a: ReactiveGroup::AcidStrong,
        b: ReactiveGroup::ActiveMetal,
        severity: Severity::Caution,
        rule: "acid-metal-hydrogen",
        hazard: "strong acid dissolves this metal, releasing hydrogen \
                 gas which is flammable",
        real_world: "Magnesium ribbon in hydrochloric acid produces \
                     enough hydrogen to pop with a lit splint — a \
                     familiar school demo, but the gas is genuinely \
                     flammable.",
    },
    // ── acid + carbonate → CO₂ ────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::AcidStrong,
        b: ReactiveGroup::Carbonate,
        severity: Severity::Caution,
        rule: "acid-carbonate-co2",
        hazard: "strong acid and carbonate fizz vigorously, releasing \
                 carbon dioxide — the mixture can spatter",
        real_world: "Adding acid to chalk or baking soda foams over \
                     if the vessel is too small. CO₂ displaces air in \
                     enclosed spaces.",
    },
    // ── water-reactive ────────────────────────────────────────────
    Incompatibility {
        a: ReactiveGroup::WaterReactive,
        b: ReactiveGroup::BaseStrong,
        severity: Severity::Caution,
        rule: "water-reactive-base",
        hazard: "this water-reactive substance already reacted with \
                 water; combining with a strong base adds further heat",
        real_world: "Quicklime (CaO) generates enough heat on contact \
                     with water to cause burns; adding caustic soda \
                     on top is reckless.",
    },
];

/// The L0 screen: warns (strongly, precisely) on states whose species carry
/// incompatible reactive groups; the simulation then shows what happens.
///
/// Water-reactive species are also checked against the presence of water
/// in the vessel — this is not a group-vs-group rule but a special case.
pub struct ReactiveGroupScreen;

impl SafetyScreen for ReactiveGroupScreen {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        if let Some(finding) = assess_exposures([("vessel", vessel)]).into_iter().next() {
            return SafetyVerdict::Warn {
                severity: finding.severity,
                rule: finding.rule,
                hazard: finding.hazard,
                real_world: finding.real_world,
            };
        }
        SafetyVerdict::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::*;

    fn vessel_with(keys: &[&str]) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "test");
        for k in keys {
            v.deposit(SpeciesId::new(k), Moles(0.1), Phase::Liquid);
        }
        v
    }

    // ── existing tests ────────────────────────────────────────────

    #[test]
    fn bleach_and_ammonia_warns_and_proceeds() {
        let v = vessel_with(&["water", "NaOCl", "NH3"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn { hazard, .. } => assert!(hazard.contains("chloramine")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn bleach_and_acid_warns() {
        let v = vessel_with(&["NaOCl", "HCl"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn { hazard, .. } => assert!(hazard.contains("chlorine")),
            other => panic!("expected Warn, got {other:?}"),
        }
    }

    #[test]
    fn exposed_material_is_assessed_across_locations_with_precise_sources() {
        let bleach = vessel_with(&["water", "NaOCl"]);
        let acid = vessel_with(&["HCl"]);
        let findings = assess_exposures([("bench", &bleach), ("tray", &acid)]);

        let chlorine = findings
            .iter()
            .find(|finding| finding.hazard.contains("chlorine"))
            .expect("cross-location chlorine hazard");
        assert_eq!(chlorine.severity, Severity::Danger);
        assert_eq!(chlorine.species, ["NaOCl", "HCl"]);
        assert_eq!(chlorine.locations, ["bench", "tray"]);
    }

    #[test]
    fn exposure_findings_round_trip_for_save_and_notebook_evidence() {
        let lime = vessel_with(&["CaO"]);
        let water = vessel_with(&["water"]);
        let finding = assess_exposures([("new spill", &lime), ("old spill", &water)])
            .into_iter()
            .next()
            .expect("water-reactive finding");

        assert_eq!(finding.species, ["CaO", "water"]);
        let encoded = serde_json::to_string(&finding).expect("serialize finding");
        let decoded: ExposureFinding = serde_json::from_str(&encoded).expect("restore finding");
        assert_eq!(decoded, finding);
    }

    #[test]
    fn acid_and_base_is_allowed() {
        let v = vessel_with(&["water", "HCl", "NaOH"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    #[test]
    fn benign_mixtures_pass() {
        let v = vessel_with(&["water", "NaCl", "AgNO3"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    // ── CAP-11 new rules ──────────────────────────────────────────

    #[test]
    fn permanganate_and_ethanol_warns() {
        let v = vessel_with(&["KMnO4", "ethanol"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("oxidizer"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn peroxide_and_isopropanol_warns() {
        let v = vessel_with(&["H2O2", "isopropanol"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("flammable"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn peroxide_and_hexane_warns() {
        let v = vessel_with(&["H2O2", "hexane"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("flammable"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn chlorine_and_hydrogen_warns() {
        let v = vessel_with(&["Cl2", "H2"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("explosion"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn oxidizer_and_reducing_agent_warns() {
        let v = vessel_with(&["KMnO4", "Na2SO3"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Danger);
                assert!(hazard.contains("violent"));
            }
            other => panic!("expected Danger, got {other:?}"),
        }
    }

    #[test]
    fn acid_and_metal_warns() {
        let v = vessel_with(&["HCl", "Mg"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Caution);
                assert!(hazard.contains("hydrogen"));
            }
            other => panic!("expected Caution, got {other:?}"),
        }
    }

    #[test]
    fn acid_and_carbonate_warns() {
        let v = vessel_with(&["HCl", "CaCO3"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Caution);
                assert!(hazard.contains("carbonate"));
            }
            other => panic!("expected Caution, got {other:?}"),
        }
    }

    #[test]
    fn quicklime_and_water_warns() {
        let v = vessel_with(&["CaO", "water"]);
        match ReactiveGroupScreen.assess(&v) {
            SafetyVerdict::Warn {
                severity, hazard, ..
            } => {
                assert_eq!(severity, Severity::Caution);
                assert!(hazard.contains("water"));
            }
            other => panic!("expected Caution, got {other:?}"),
        }
    }

    #[test]
    fn all_inert_species_allow() {
        let v = vessel_with(&["water", "NaCl", "KCl", "CaCl2", "phenolphthalein", "N2"]);
        assert_eq!(ReactiveGroupScreen.assess(&v), SafetyVerdict::Allow);
    }

    // ── BRD-012.S02: the barium gate ──────────────────────────────

    #[test]
    fn soluble_barium_is_labelled_toxic_and_barite_is_not() {
        for key in ["BaCl2", "Ba(OH)2", "Ba+2"] {
            let (labels, assessed) = hazard_assessment(key);
            assert!(assessed, "{key} must carry a safety row");
            assert!(
                labels.contains(&"toxic"),
                "{key} must show as toxic on every shelf that shows hazards, got {labels:?}"
            );
        }
        // The precipitate is the form that is swallowed for a radiograph.
        let (labels, assessed) = hazard_assessment("BaSO4");
        assert!(assessed, "barite must still be assessed, not unknown");
        assert!(
            !labels.contains(&"toxic"),
            "the insoluble sulfate is not the toxic form: {labels:?}"
        );
    }

    #[test]
    fn barium_hydroxide_is_a_strong_base_as_well() {
        let (labels, _) = hazard_assessment("Ba(OH)2");
        assert!(labels.contains(&"corrosive"), "{labels:?}");
        assert!(labels.contains(&"toxic"), "{labels:?}");
    }

    /// The GUI-080 safety-audit mission recognises hazards by these rule
    /// ids (`outcomeMission.ts`), and they cross the locale boundary
    /// untranslated. Renaming one silently breaks a shipped mission
    /// contract — this test is the tripwire.
    #[test]
    fn rule_ids_are_stable_contract_surface() {
        let cases: &[(&[&str], &str)] = &[
            (&["NaOCl", "NH3"], "bleach-ammonia-chloramine"),
            (&["NaOCl", "HCl"], "bleach-acid-chlorine"),
            (&["KMnO4", "ethanol"], "oxidizer-flammable-liquid"),
            (&["HCl", "Mg"], "acid-metal-hydrogen"),
            (&["HCl", "CaCO3"], "acid-carbonate-co2"),
            (&["CaO", "water"], "water-reactive-slaking"),
        ];
        for (keys, expected) in cases {
            match ReactiveGroupScreen.assess(&vessel_with(keys)) {
                SafetyVerdict::Warn { rule, .. } => {
                    assert_eq!(rule, *expected, "for {keys:?}");
                }
                other => panic!("expected a warning for {keys:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn iron_iii_chloride_is_not_silently_harmless() {
        let (labels, assessed) = hazard_assessment("FeCl3");
        assert!(assessed);
        assert!(labels.contains(&"irritant"), "{labels:?}");
    }

    #[test]
    fn totality_of_covered_keys() {
        let registry_keys: std::collections::HashSet<&str> = kerotakis_core::species::registry()
            .iter()
            .map(|s| s.key)
            .collect();
        let safety_keys: std::collections::HashSet<&str> = COVERED_KEYS.iter().copied().collect();

        let missing: Vec<&&str> = registry_keys.difference(&safety_keys).collect();
        assert!(
            missing.is_empty(),
            "registry species without safety row: {missing:?}"
        );

        let extra: Vec<&&str> = safety_keys.difference(&registry_keys).collect();
        assert!(
            extra.is_empty(),
            "safety rows for species not in registry: {extra:?}"
        );
    }
}
