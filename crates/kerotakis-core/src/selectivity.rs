//! EXP-50: Mechanistic selectivity rules — SN1/SN2/E1/E2 prediction.
//!
//! A curated rule table selects the mechanism for haloalkane reactions
//! based on substrate class, nucleophile strength, and temperature.
//! Conditions outside the table are refused out loud.
//!
//! Provenance: March's Advanced Organic Chemistry, 5th ed., Chapter 10.

use crate::ops::Event;
use crate::species::Phase;
use crate::units::{Kelvin, Moles};
use crate::vessel::Vessel;
use crate::SpeciesId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateClass {
    Methyl,
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NucleophileClass {
    Strong,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanism {
    Sn1,
    Sn2,
    E1,
    E2,
}

impl std::fmt::Display for Mechanism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sn1 => write!(f, "SN1"),
            Self::Sn2 => write!(f, "SN2"),
            Self::E1 => write!(f, "E1"),
            Self::E2 => write!(f, "E2"),
        }
    }
}

impl std::fmt::Display for SubstrateClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Methyl => write!(f, "methyl"),
            Self::Primary => write!(f, "primary"),
            Self::Secondary => write!(f, "secondary"),
            Self::Tertiary => write!(f, "tertiary"),
        }
    }
}

impl std::fmt::Display for NucleophileClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strong => write!(f, "strong"),
            Self::Weak => write!(f, "weak"),
        }
    }
}

struct SubstrateInfo {
    key: &'static str,
    class: SubstrateClass,
}

const SUBSTRATES: &[SubstrateInfo] = &[
    SubstrateInfo {
        key: "bromoethane",
        class: SubstrateClass::Primary,
    },
    SubstrateInfo {
        key: "tert_butyl_bromide",
        class: SubstrateClass::Tertiary,
    },
];

struct NucleophileInfo {
    key: &'static str,
    class: NucleophileClass,
}

const NUCLEOPHILES: &[NucleophileInfo] = &[
    NucleophileInfo {
        key: "NaOH",
        class: NucleophileClass::Strong,
    },
    NucleophileInfo {
        key: "water",
        class: NucleophileClass::Weak,
    },
];

const ELIMINATION_THRESHOLD_K: f64 = 353.15; // 80 °C

struct SelectivityRule {
    substrate_class: SubstrateClass,
    nucleophile_class: NucleophileClass,
    hot: bool,
    mechanism: Mechanism,
    provenance: &'static str,
}

const RULES: &[SelectivityRule] = &[
    SelectivityRule {
        substrate_class: SubstrateClass::Primary,
        nucleophile_class: NucleophileClass::Strong,
        hot: false,
        mechanism: Mechanism::Sn2,
        provenance: "March ch. 10: primary + strong nucleophile → SN2 (back-side attack, \
                     Walden inversion)",
    },
    SelectivityRule {
        substrate_class: SubstrateClass::Primary,
        nucleophile_class: NucleophileClass::Strong,
        hot: true,
        mechanism: Mechanism::E2,
        provenance: "March ch. 10: primary + strong base + heat → E2 (anti-periplanar \
                     elimination, Zaitsev product)",
    },
    SelectivityRule {
        substrate_class: SubstrateClass::Tertiary,
        nucleophile_class: NucleophileClass::Weak,
        hot: false,
        mechanism: Mechanism::Sn1,
        provenance: "March ch. 10: tertiary + weak nucleophile → SN1 (carbocation \
                     intermediate, racemisation)",
    },
    SelectivityRule {
        substrate_class: SubstrateClass::Tertiary,
        nucleophile_class: NucleophileClass::Weak,
        hot: true,
        mechanism: Mechanism::E1,
        provenance: "March ch. 10: tertiary + weak nucleophile + heat → E1 (unimolecular \
                     elimination via carbocation, Zaitsev product)",
    },
    SelectivityRule {
        substrate_class: SubstrateClass::Tertiary,
        nucleophile_class: NucleophileClass::Strong,
        hot: false,
        mechanism: Mechanism::E2,
        provenance: "March ch. 10: tertiary + strong base → E2 (SN2 blocked by steric \
                     hindrance; base-promoted elimination)",
    },
    SelectivityRule {
        substrate_class: SubstrateClass::Tertiary,
        nucleophile_class: NucleophileClass::Strong,
        hot: true,
        mechanism: Mechanism::E2,
        provenance: "March ch. 10: tertiary + strong base + heat → E2 (elimination \
                     favoured at any temperature for this combination)",
    },
];

struct ProductEntry {
    substrate: &'static str,
    nucleophile: &'static str,
    mechanism: Mechanism,
    equation: &'static str,
    reactants: &'static [(&'static str, f64)],
    products: &'static [(&'static str, f64, Phase)],
}

const PRODUCT_TABLE: &[ProductEntry] = &[
    // ── SN2: bromoethane + NaOH → ethanol + NaBr ─────────────────
    ProductEntry {
        substrate: "bromoethane",
        nucleophile: "NaOH",
        mechanism: Mechanism::Sn2,
        equation: "CH₃CH₂Br + NaOH → C₂H₅OH + NaBr",
        reactants: &[("bromoethane", 1.0), ("NaOH", 1.0)],
        products: &[
            ("ethanol", 1.0, Phase::Liquid),
            ("NaBr", 1.0, Phase::Aqueous),
        ],
    },
    // ── E2: bromoethane + NaOH (hot) → ethene + NaBr + H₂O ──────
    ProductEntry {
        substrate: "bromoethane",
        nucleophile: "NaOH",
        mechanism: Mechanism::E2,
        equation: "CH₃CH₂Br + NaOH →Δ CH₂=CH₂↑ + NaBr + H₂O",
        reactants: &[("bromoethane", 1.0), ("NaOH", 1.0)],
        products: &[
            ("ethene", 1.0, Phase::Gas),
            ("NaBr", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
    },
    // ── SN1: tert-butyl bromide + H₂O → tert-butanol + HBr ──────
    ProductEntry {
        substrate: "tert_butyl_bromide",
        nucleophile: "water",
        mechanism: Mechanism::Sn1,
        equation: "(CH₃)₃CBr + H₂O → (CH₃)₃COH + HBr",
        reactants: &[("tert_butyl_bromide", 1.0), ("water", 1.0)],
        products: &[
            ("tert_butanol", 1.0, Phase::Liquid),
            ("HBr", 1.0, Phase::Aqueous),
        ],
    },
    // ── E1: tert-butyl bromide (hot, water as solvent) → isobutylene + HBr
    ProductEntry {
        substrate: "tert_butyl_bromide",
        nucleophile: "water",
        mechanism: Mechanism::E1,
        equation: "(CH₃)₃CBr →Δ (CH₃)₂C=CH₂↑ + HBr",
        reactants: &[("tert_butyl_bromide", 1.0)],
        products: &[
            ("isobutylene", 1.0, Phase::Gas),
            ("HBr", 1.0, Phase::Aqueous),
        ],
    },
    // ── E2: tert-butyl bromide + NaOH → isobutylene + NaBr + H₂O
    ProductEntry {
        substrate: "tert_butyl_bromide",
        nucleophile: "NaOH",
        mechanism: Mechanism::E2,
        equation: "(CH₃)₃CBr + NaOH → (CH₃)₂C=CH₂↑ + NaBr + H₂O",
        reactants: &[("tert_butyl_bromide", 1.0), ("NaOH", 1.0)],
        products: &[
            ("isobutylene", 1.0, Phase::Gas),
            ("NaBr", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
    },
];

fn find_substrate(vessel: &Vessel) -> Option<&'static SubstrateInfo> {
    SUBSTRATES
        .iter()
        .find(|s| vessel.moles_of(&SpeciesId::new(s.key)).0 > 1e-12)
}

fn find_nucleophile(vessel: &Vessel) -> Option<&'static NucleophileInfo> {
    NUCLEOPHILES
        .iter()
        .find(|n| vessel.moles_of(&SpeciesId::new(n.key)).0 > 1e-12)
}

fn find_rule(
    substrate_class: SubstrateClass,
    nuc_class: NucleophileClass,
    hot: bool,
) -> Option<&'static SelectivityRule> {
    RULES.iter().find(|r| {
        r.substrate_class == substrate_class && r.nucleophile_class == nuc_class && r.hot == hot
    })
}

fn find_products(
    substrate: &str,
    nucleophile: &str,
    mechanism: Mechanism,
) -> Option<&'static ProductEntry> {
    PRODUCT_TABLE.iter().find(|p| {
        p.substrate == substrate && p.nucleophile == nucleophile && p.mechanism == mechanism
    })
}

pub const VERB_NAME: &str = "haloalkane";

pub fn dispatch(vessel: &mut Vessel) -> Vec<Event> {
    let substrate = match find_substrate(vessel) {
        Some(s) => s,
        None => {
            return vec![Event::NotYetModeled {
                vessel: vessel.id,
                what: "no haloalkane substrate in the vessel — the selectivity \
                       table covers bromoethane (primary) and tert-butyl \
                       bromide (tertiary)"
                    .to_string(),
            }];
        }
    };

    let nucleophile = match find_nucleophile(vessel) {
        Some(n) => n,
        None => {
            return vec![Event::NotYetModeled {
                vessel: vessel.id,
                what: "no recognised nucleophile in the vessel — the selectivity \
                       table covers NaOH (strong) and water (weak)"
                    .to_string(),
            }];
        }
    };

    let hot = vessel.temperature >= Kelvin(ELIMINATION_THRESHOLD_K);

    let rule = match find_rule(substrate.class, nucleophile.class, hot) {
        Some(r) => r,
        None => {
            return vec![Event::NotYetModeled {
                vessel: vessel.id,
                what: format!(
                    "no selectivity rule for {} substrate + {} nucleophile \
                     at {:.0}°C — outside the curated table",
                    substrate.class,
                    nucleophile.class,
                    vessel.temperature.0 - 273.15
                ),
            }];
        }
    };

    let entry = match find_products(substrate.key, nucleophile.key, rule.mechanism) {
        Some(e) => e,
        None => {
            return vec![Event::NotYetModeled {
                vessel: vessel.id,
                what: format!(
                    "selectivity predicts {} for {} + {} but no product table \
                     entry exists — this is an implementation gap",
                    rule.mechanism, substrate.key, nucleophile.key
                ),
            }];
        }
    };

    let extent = entry
        .reactants
        .iter()
        .map(|(key, coeff)| vessel.moles_of(&SpeciesId::new(key)).0 / coeff)
        .fold(f64::INFINITY, f64::min);

    if !(extent.is_finite() && extent > 1e-12) {
        let needs: Vec<&str> = entry.reactants.iter().map(|(k, _)| *k).collect();
        return vec![Event::NotYetModeled {
            vessel: vessel.id,
            what: format!(
                "nothing for {} to work on — it needs {} together in the vessel",
                VERB_NAME,
                needs.join(" and ")
            ),
        }];
    }

    for (key, coeff) in entry.reactants {
        vessel.withdraw(&SpeciesId::new(key), Moles(extent * coeff));
    }

    let mut events = Vec::new();

    for (key, coeff, phase) in entry.products {
        let n = Moles(extent * coeff);
        if *phase == Phase::Gas {
            let species = SpeciesId::new(key);
            if vessel.retain_gas(species.clone(), n) {
                events.push(Event::GasContained {
                    vessel: vessel.id,
                    species,
                    moles: n,
                });
            } else {
                events.push(Event::GasEvolved {
                    vessel: vessel.id,
                    species,
                    moles: n,
                });
            }
        } else {
            vessel.deposit(SpeciesId::new(key), n, *phase);
        }
    }

    events.push(Event::OrgReacted {
        vessel: vessel.id,
        name: format!("haloalkane:{}", rule.mechanism),
        equation: entry.equation.to_string(),
        extent: Moles(extent),
        boundary: format!(
            "Mechanism {} selected by the selectivity rule table: {} substrate ({}) \
             + {} nucleophile ({}) at {:.0}°C. {}. \
             The reaction is driven to completion on command — no yield claim is \
             made, and the kinetic selectivity ratios are not modelled",
            rule.mechanism,
            substrate.class,
            substrate.key,
            nucleophile.class,
            nucleophile.key,
            vessel.temperature.0 - 273.15,
            rule.provenance
        ),
    });

    events
}

pub fn is_selectivity_verb(name: &str) -> bool {
    name == VERB_NAME
}
