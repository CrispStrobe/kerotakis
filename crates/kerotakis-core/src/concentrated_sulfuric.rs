//! Concentrated oxidising and dehydrating acids — and hydrogen sulfide
//! oxidised by peroxide.
//!
//! Before this route, a beaker of concentrated H2SO4 was inert: the aqueous
//! model rightly refuses the medium (`not-modeled.concentrated-acid-medium`,
//! water a minority of the liquid), and nothing else claimed it. So sugar did
//! not char, table salt gave no hydrogen chloride, and copper, carbon and
//! sulfur sat unchanged in boiling acid (C1 E067, E084, E143, E144, E147,
//! E148, E333, E364).
//!
//! Every rule is a reaction CLASS driven by data rows, not a list of
//! reactions (see `docs/reaction-routes.md`):
//!
//! * **Dehydration** of every registry carbohydrate, recognised by its
//!   formula class Cn(H2O)m (n ≥ 5, m = n or n − 1: the pentoses, hexoses,
//!   disaccharides and the C6H10O5 unit of starch and cellulose), to carbon
//!   and water, e.g. C12H22O11 → 12 C + 11 H2O. Short Cn(H2O)n formulas
//!   (acetic and lactic acid, formaldehyde) are carboxylic acids or
//!   aldehydes, not carbohydrates, and are excluded by the n ≥ 5 bound.
//! * **Volatile-acid displacement**: a non-volatile strong acid
//!   ([`NONVOLATILE_ACIDS`]) and any solid salt of a volatile acid
//!   ([`VOLATILE_ACIDS`]) give the volatile acid and the acid's (hydrogen)
//!   salt, NaCl + H2SO4 → NaHSO4 + HCl↑, MgCl2 + H2SO4 → MgSO4 + 2 HCl↑,
//!   CH3COONa + H2SO4 → NaHSO4 + CH3COOH. The salt is recognised from the
//!   registry by formula (metal cation + the volatile acid's anion); the
//!   volatile acid leaves as a gas above its normal boiling point.
//! * **Oxidation by the acid's reduction couple** ([`OXIDISING_ACIDS`]):
//!   the acid half (SO4²⁻ + 4 H⁺ + 2 e⁻ → SO2 + 2 H2O; NO3⁻ + 4 H⁺ + 3 e⁻ →
//!   NO + 2 H2O; NO3⁻ + 2 H⁺ + e⁻ → NO2 + H2O) is balanced against each
//!   admitted reductant half (Cu → Cu²⁺; C → CO2; S → SO2) by electron
//!   count, and the protons and the metal's counter-anion are supplied by
//!   the acid itself: Cu + 2 H2SO4 → CuSO4 + SO2↑ + 2 H2O,
//!   3 Cu + 8 HNO3 → 3 Cu(NO3)2 + 2 NO↑ + 4 H2O. The same code serves hot
//!   concentrated sulfuric acid and nitric acid; a row whose acid or
//!   products the registry does not carry yet (nitric acid, NO, NO2) is
//!   dormant and says so with a typed note.
//! * H2S + H2O2 → S↓ + 2 H2O in water.
//!
//! WHAT IS NOT CLAIMED: no rate, no reaction heat and no measured yield.
//! "Concentrated" is the bench's own existing editorial admission boundary
//! (water below half the liquid, [`crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR`]),
//! and the dehydrating and oxidising rows stop where their own product
//! water would carry the acid across that boundary. Sulfuric acid in that
//! medium is far from ideal, so its oxidations are admitted by the medium
//! (concentrated AND hot, [`HOT_ONSET_K`]), not by a Nernst crossing: at
//! unit activity its couple (SO4²⁻/H2SO3, 0.172 V) is below Cu²⁺/Cu
//! (0.3419 V), which is exactly why dilute sulfuric acid leaves copper
//! alone. Nitric acid's couples lie above Cu²⁺/Cu at unit activity, so its
//! aqueous row is admitted by the standard potentials. Every firing says so
//! through a typed `NotYetModeled` event.

use crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR;
use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::Phase;
use crate::stoich::Formula;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;
use std::collections::BTreeMap;

/// Editorial onset of the "hot concentrated sulfuric acid" oxidations: the
/// boiling point of water at one atmosphere, i.e. acid heated well past
/// anything a water bath gives. No reviewed onset temperature exists for
/// these heterogeneous oxidations (they are rate-limited, not
/// thermodynamically switched), so this is one named constant for the
/// mechanism, said in the typed note every time it fires. Not a measured
/// threshold and not a rate law.
pub const HOT_ONSET_K: f64 = 373.15;

const TRACE: f64 = 1e-12;
const ACID: &str = "H2SO4";

/// A non-volatile strong acid that displaces volatile acids from their
/// salts: (key, anion, normal boiling point K, source).
pub const NONVOLATILE_ACIDS: &[(&str, &str, f64, &str)] = &[(
    "H2SO4",
    "SO4-2",
    610.15,
    "CRC Handbook of Chemistry and Physics, Physical Constants of Inorganic Compounds: H2SO4 bp 337 °C",
)];

/// A volatile acid whose salts a non-volatile acid decomposes.
#[derive(Debug, Clone, Copy)]
pub struct VolatileAcid {
    pub acid: &'static str,
    pub anion: &'static str,
    /// Normal boiling point, K, used when the registry carries none.
    pub boiling_k: f64,
    /// The halogen hot concentrated sulfuric acid partly oxidises this acid
    /// to, if any (HBr → Br2, HI → I2). `Some` with `refused` set means the
    /// oxidation dominates and the displacement is not booked at all.
    pub oxidised_to: Option<&'static str>,
    pub refused: bool,
    pub source: &'static str,
}

const CRC_BP: &str = "CRC Handbook of Chemistry and Physics, Physical Constants of Inorganic/Organic Compounds (normal boiling points)";

pub const VOLATILE_ACIDS: &[VolatileAcid] = &[
    VolatileAcid {
        acid: "HCl",
        anion: "Cl-",
        boiling_k: 188.1,
        oxidised_to: None,
        refused: false,
        source: CRC_BP,
    },
    VolatileAcid {
        acid: "HBr",
        anion: "Br-",
        boiling_k: 206.4,
        oxidised_to: Some("Br2"),
        refused: false,
        source: CRC_BP,
    },
    VolatileAcid {
        acid: "HI",
        anion: "I-",
        boiling_k: 237.6,
        oxidised_to: Some("I2"),
        refused: true,
        source: CRC_BP,
    },
    VolatileAcid {
        acid: "HNO3",
        anion: "NO3-",
        boiling_k: 356.15,
        oxidised_to: None,
        refused: false,
        source: CRC_BP,
    },
    VolatileAcid {
        acid: "CH3COOH",
        anion: "CH3COO-",
        boiling_k: 391.1,
        oxidised_to: None,
        refused: false,
        source: CRC_BP,
    },
];

/// Medium in which an oxidising acid row is admitted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Medium {
    /// Water below the admission boundary; `hot_onset_k` if it must also
    /// be hot.
    Concentrated { hot_onset_k: Option<f64> },
    /// Water-majority solution, admitted by the standard potentials.
    Aqueous,
}

/// An acid as an oxidant: its reduction half `anion + protons H⁺ +
/// electrons e⁻ → reduced + water H₂O`.
#[derive(Debug, Clone, Copy)]
pub struct OxidisingAcid {
    pub acid: &'static str,
    pub anion: &'static str,
    pub reduced: &'static str,
    pub electrons: f64,
    pub protons: f64,
    pub water: f64,
    /// Standard reduction potential, V (CRC Electrochemical Series).
    pub e0_v: f64,
    pub medium: Medium,
    /// Keys of the [`ACID_REDUCTANTS`] rows this acid is reviewed to oxidise.
    pub reductants: &'static [&'static str],
    pub source: &'static str,
}

const TEXTBOOK: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed., §15.2.5 (sulfuric acid) and §11.3.7 (nitric acid); Holleman-Wiberg, Inorganic Chemistry, 101st ed.";

pub const OXIDISING_ACIDS: &[OxidisingAcid] = &[
    OxidisingAcid {
        acid: "H2SO4",
        anion: "SO4-2",
        reduced: "SO2",
        electrons: 2.0,
        protons: 4.0,
        water: 2.0,
        e0_v: 0.172,
        medium: Medium::Concentrated {
            hot_onset_k: Some(HOT_ONSET_K),
        },
        reductants: &["Cu", "C", "activated_charcoal", "S"],
        source: TEXTBOOK,
    },
    OxidisingAcid {
        acid: "HNO3",
        anion: "NO3-",
        reduced: "NO",
        electrons: 3.0,
        protons: 4.0,
        water: 2.0,
        e0_v: 0.957,
        medium: Medium::Aqueous,
        reductants: &["Cu"],
        source: TEXTBOOK,
    },
    OxidisingAcid {
        acid: "HNO3",
        anion: "NO3-",
        reduced: "NO2",
        electrons: 1.0,
        protons: 2.0,
        water: 1.0,
        e0_v: 0.803,
        medium: Medium::Concentrated { hot_onset_k: None },
        reductants: &["Cu"],
        source: TEXTBOOK,
    },
];

/// What a reductant becomes.
#[derive(Debug, Clone, Copy)]
pub enum Oxidised {
    /// A metal cation, written as the salt of the acid's anion.
    Cation { ion: &'static str, charge: f64 },
    /// A molecule (CO2, SO2).
    Molecule { key: &'static str, phase: Phase },
}

/// A reductant half run forwards: `key (+ water H₂O) → oxidised + protons
/// H⁺ + electrons e⁻`.
#[derive(Debug, Clone, Copy)]
pub struct AcidReductant {
    pub key: &'static str,
    pub oxidised: Oxidised,
    pub electrons: f64,
    pub protons_released: f64,
    pub water_consumed: f64,
    /// Standard reduction potential of a non-metal couple, V, where one is
    /// reviewed (CRC: H2SO3/S 0.449). A metal's is read from the activity
    /// series the displacement route already carries
    /// ([`crate::displacement::SERIES`]), not repeated here. Graphite and
    /// diamond are not rows: their resistance to the acid is why, not an
    /// omission.
    pub e0_v: Option<f64>,
}

pub const ACID_REDUCTANTS: &[AcidReductant] = &[
    AcidReductant {
        key: "Cu",
        oxidised: Oxidised::Cation {
            ion: "Cu+2",
            charge: 2.0,
        },
        electrons: 2.0,
        protons_released: 0.0,
        water_consumed: 0.0,
        // Read from the metal activity series (`displacement::SERIES`).
        e0_v: None,
    },
    AcidReductant {
        key: "C",
        oxidised: Oxidised::Molecule {
            key: "CO2",
            phase: Phase::Gas,
        },
        electrons: 4.0,
        protons_released: 4.0,
        water_consumed: 2.0,
        e0_v: None,
    },
    AcidReductant {
        key: "activated_charcoal",
        oxidised: Oxidised::Molecule {
            key: "CO2",
            phase: Phase::Gas,
        },
        electrons: 4.0,
        protons_released: 4.0,
        water_consumed: 2.0,
        e0_v: None,
    },
    AcidReductant {
        key: "S",
        oxidised: Oxidised::Molecule {
            key: "SO2",
            phase: Phase::Gas,
        },
        electrons: 4.0,
        protons_released: 4.0,
        water_consumed: 2.0,
        e0_v: Some(0.449),
    },
];

// ── inventory helpers ────────────────────────────────────────────────

fn phase_moles(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && phases.contains(&p.phase))
        .map(|p| p.moles.0)
        .sum()
}

fn in_registry(key: &str) -> bool {
    crate::species::lookup_key(key).is_some()
}

fn formula_of(key: &str) -> Option<Formula> {
    let text = if key == "water" {
        "H2O"
    } else {
        crate::species::lookup_key(key)
            .map(|d| d.formula)
            .unwrap_or(key)
    };
    crate::stoich::parse_formula(text).ok()
}

/// Display spelling: the key when it spells its own formula (NaBr, CuSO4),
/// otherwise the registry formula (CH3COONa for NaOAc, C for charcoal).
fn display(key: &str) -> String {
    let own = crate::stoich::parse_formula(key).ok();
    let reg = formula_of(key);
    match (own, reg) {
        (Some(a), Some(b)) if a.counts == b.counts && a.charge == b.charge => {
            crate::halogen_redox::pretty(key)
        }
        (_, Some(_)) => crate::halogen_redox::pretty(
            crate::species::lookup_key(key)
                .map(|d| d.formula)
                .unwrap_or(key),
        ),
        _ => crate::halogen_redox::pretty(key),
    }
}

fn term(coef: f64, text: &str) -> String {
    if (coef - 1.0).abs() < 1e-9 {
        text.to_string()
    } else {
        format!("{} {text}", coef.round() as i64)
    }
}

/// The neutral registry species with exactly these counts, if any.
fn registry_with_counts(counts: &BTreeMap<String, f64>) -> Option<&'static str> {
    crate::species::registry().iter().find_map(|s| {
        let f = crate::stoich::parse_formula(s.formula).ok()?;
        (f.charge == 0.0
            && f.counts.len() == counts.len()
            && f.counts
                .iter()
                .all(|(e, n)| (counts.get(e).copied().unwrap_or(0.0) - n).abs() < 1e-9))
        .then_some(s.key)
    })
}

/// The registry's monatomic metal cation of `element` (Na → Na+), with its
/// charge. Hydrogen is not a metal: an acid is not its own salt.
fn cation_of(element: &str) -> Option<f64> {
    if element == "H" {
        return None;
    }
    crate::species::registry().iter().find_map(|s| {
        let f = crate::stoich::parse_formula(s.formula).ok()?;
        (f.charge > 0.0 && f.counts.len() == 1 && f.counts.get(element) == Some(&1.0))
            .then_some(f.charge)
    })
}

/// A salt `M_a X_b` of one metal and one anion: (metal, a, charge, b).
fn split_salt(salt: &Formula, anion: &Formula) -> Option<(String, f64, f64, f64)> {
    if salt.charge != 0.0 || anion.charge >= 0.0 {
        return None;
    }
    for b in 1..=3 {
        let b = f64::from(b);
        let mut rest = salt.counts.clone();
        let mut ok = true;
        for (e, n) in &anion.counts {
            match rest.get_mut(e) {
                Some(have) if *have >= b * n - 1e-9 => *have -= b * n,
                _ => ok = false,
            }
        }
        if !ok {
            continue;
        }
        rest.retain(|_, n| n.abs() > 1e-9);
        if rest.len() != 1 {
            continue;
        }
        let (metal, a) = rest.into_iter().next()?;
        let z = cation_of(&metal)?;
        if (a * z + b * anion.charge).abs() < 1e-9 {
            return Some((metal, a, z, b));
        }
    }
    None
}

// ── medium ───────────────────────────────────────────────────────────

/// Neat (undissolved) acid in the liquid.
fn neat(vessel: &Vessel, acid: &str) -> f64 {
    phase_moles(vessel, acid, &[Phase::Liquid])
}

/// Whether the vessel holds this acid concentrated by the bench's own
/// admission boundary: the liquid acid present and water below
/// [`AQUEOUS_WATER_FRACTION_FLOOR`] of the competing liquids.
fn concentrated_acid(vessel: &Vessel, acid: &str) -> bool {
    neat(vessel, acid) > TRACE && !crate::nonaqueous::aqueous_model_admits(vessel)
}

/// Whether the vessel holds concentrated sulfuric acid.
pub fn concentrated(vessel: &Vessel) -> bool {
    concentrated_acid(vessel, ACID)
}

#[cfg(test)]
fn acid(vessel: &Vessel) -> f64 {
    neat(vessel, ACID)
}

/// Moles of product water the liquid can take before it crosses the
/// boundary: with water fraction w = W / (W + R), staying below the floor f
/// needs W ≤ R f / (1 − f).
fn water_room(vessel: &Vessel, acid: &str) -> f64 {
    let Some(fraction) = crate::nonaqueous::water_fraction_among_solvents(vessel) else {
        return 0.0;
    };
    let water = phase_moles(vessel, "water", &[Phase::Liquid, Phase::Aqueous]);
    let total = if fraction > 0.0 {
        water / fraction
    } else {
        neat(vessel, acid)
    };
    let rest = (total - water).max(0.0);
    let f = AQUEOUS_WATER_FRACTION_FLOOR;
    (rest * f / (1.0 - f) - water).max(0.0)
}

// ── dehydration ──────────────────────────────────────────────────────

/// (carbon, water) per formula unit when `formula` is in the carbohydrate
/// class Cn(H2O)m, n ≥ 5, m ∈ {n − 1, n}.
pub fn carbohydrate_class(formula: &Formula) -> Option<(f64, f64)> {
    if formula.charge != 0.0 || formula.counts.len() != 3 {
        return None;
    }
    let c = *formula.counts.get("C")?;
    let h = *formula.counts.get("H")?;
    let o = *formula.counts.get("O")?;
    let whole = |x: f64| (x - x.round()).abs() < 1e-9;
    if !(whole(c) && whole(o) && (h - 2.0 * o).abs() < 1e-9) {
        return None;
    }
    (c >= 5.0 && (o == c || o == c - 1.0)).then_some((c, o))
}

const DEHYDRATABLE_PHASES: &[Phase] = &[Phase::Solid, Phase::Liquid, Phase::Aqueous];

/// Carbohydrates in the vessel: (key, carbon, water) per formula unit.
fn carbohydrates(vessel: &Vessel) -> Vec<(String, f64, f64)> {
    let mut out: Vec<(String, f64, f64)> = Vec::new();
    for p in &vessel.contents {
        if !DEHYDRATABLE_PHASES.contains(&p.phase) || p.moles.0 <= TRACE {
            continue;
        }
        let key = p.species.0.as_str();
        if out.iter().any(|(k, ..)| k == key) {
            continue;
        }
        let Some(data) = crate::species::lookup_key(key) else {
            continue;
        };
        let Ok(f) = crate::stoich::parse_formula(data.formula) else {
            continue;
        };
        if let Some((c, w)) = carbohydrate_class(&f) {
            out.push((key.to_string(), c, w));
        }
    }
    out
}

/// Concentrated acids that dehydrate carbohydrates.
pub const DEHYDRATING_ACIDS: &[&str] = &["H2SO4"];

fn dehydrating_acid(vessel: &Vessel) -> Option<&'static str> {
    DEHYDRATING_ACIDS
        .iter()
        .copied()
        .find(|a| concentrated_acid(vessel, a))
}

fn dehydration_extent(vessel: &Vessel, acid: &str, key: &str, water: f64) -> f64 {
    phase_moles(vessel, key, DEHYDRATABLE_PHASES).min(water_room(vessel, acid) / water)
}

// ── volatile-acid displacement ───────────────────────────────────────

/// One displacement found in the vessel.
#[derive(Debug, Clone)]
struct Displacement {
    salt: String,
    acid: &'static str,
    volatile: &'static VolatileAcid,
    /// Per salt unit: acid consumed, product formed, volatile acid released.
    acid_per: f64,
    product: Option<&'static str>,
    product_per: f64,
    released: f64,
    /// The acid's salt that would be written, for a dormant note.
    wanted: String,
}

/// The acid's salt of `metal` (charge `z`): the hydrogen salt (NaHSO4)
/// when the metal takes fewer protons than the acid has, else the neutral
/// salt (Na2SO4, MgSO4). Returns (registry key, metals and acid anions per
/// formula unit), or the formula it looked for.
fn salt_product(
    metal: &str,
    z: f64,
    acid: &Formula,
    anion_key: &str,
    anion: &Formula,
) -> Result<(&'static str, f64, f64), String> {
    let k = acid.counts.get("H").copied().unwrap_or(0.0);
    let q = -anion.charge;
    let mut candidates: Vec<(f64, f64, f64)> = Vec::new(); // (metals, anions, hydrogens)
    if z < k && (k - z - (k - z).round()).abs() < 1e-9 {
        candidates.push((1.0, 1.0, k - z));
    }
    let g = gcd(q.round() as u64, z.round() as u64).max(1) as f64;
    candidates.push((q / g, z / g, 0.0));
    let mut tried = Vec::new();
    for (metals, anions, hydrogens) in candidates {
        let mut counts: BTreeMap<String, f64> = BTreeMap::new();
        for (e, n) in &anion.counts {
            *counts.entry(e.clone()).or_default() += n * anions;
        }
        *counts.entry(metal.to_string()).or_default() += metals;
        if hydrogens > 0.0 {
            *counts.entry("H".to_string()).or_default() += hydrogens;
        }
        if let Some(key) = registry_with_counts(&counts) {
            return Ok((key, metals, anions));
        }
        tried.push(salt_name(metal, metals, hydrogens, anion_key, anions));
    }
    Err(tried.join(" / "))
}

/// A readable formula for a salt the registry lacks (KHSO₄, K₂SO₄).
fn salt_name(metal: &str, metals: f64, hydrogens: f64, anion_key: &str, anions: f64) -> String {
    let count = |n: f64| {
        if (n - 1.0).abs() < 1e-9 {
            String::new()
        } else {
            format!("{}", n.round() as i64)
        }
    };
    let body = anion_key
        .rfind(['+', '-'])
        .map_or(anion_key, |i| &anion_key[..i]);
    let anion = if (anions - 1.0).abs() < 1e-9 {
        body.to_string()
    } else {
        format!("({body}){}", count(anions))
    };
    let h = if hydrogens > 0.0 {
        format!("H{}", count(hydrogens))
    } else {
        String::new()
    };
    crate::halogen_redox::pretty(&format!("{metal}{}{h}{anion}", count(metals)))
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn displacements(vessel: &Vessel) -> Vec<Displacement> {
    let mut out = Vec::new();
    for &(acid, acid_anion, _, _) in NONVOLATILE_ACIDS {
        if !concentrated_acid(vessel, acid) {
            continue;
        }
        let (Some(acid_f), Some(acid_anion_f)) = (formula_of(acid), formula_of(acid_anion)) else {
            continue;
        };
        for p in &vessel.contents {
            if p.phase != Phase::Solid || p.moles.0 <= TRACE {
                continue;
            }
            let key = p.species.0.as_str();
            if out.iter().any(|d: &Displacement| d.salt == key) {
                continue;
            }
            let Some(salt_f) = formula_of(key) else {
                continue;
            };
            for volatile in VOLATILE_ACIDS {
                let Some(anion_f) = formula_of(volatile.anion) else {
                    continue;
                };
                let Some((metal, a, z, b)) = split_salt(&salt_f, &anion_f) else {
                    continue;
                };
                let (product, product_per, acid_per, wanted) =
                    match salt_product(&metal, z, &acid_f, acid_anion, &acid_anion_f) {
                        Ok((salt, metals, anions)) => {
                            (Some(salt), a / metals, a / metals * anions, String::new())
                        }
                        Err(wanted) => (None, 0.0, 0.0, wanted),
                    };
                out.push(Displacement {
                    salt: key.to_string(),
                    acid,
                    volatile,
                    acid_per,
                    product,
                    product_per,
                    released: b,
                    wanted,
                });
                break;
            }
        }
    }
    out
}

fn admitted(d: &Displacement) -> bool {
    d.product.is_some() && in_registry(d.volatile.acid) && !d.volatile.refused
}

fn displacement_extent(vessel: &Vessel, d: &Displacement) -> f64 {
    if !admitted(d) {
        return 0.0;
    }
    phase_moles(vessel, &d.salt, &[Phase::Solid]).min(neat(vessel, d.acid) / d.acid_per)
}

fn volatile_boiling_k(v: &VolatileAcid) -> f64 {
    crate::species::lookup_key(v.acid)
        .and_then(|d| d.transitions)
        .and_then(|t| t.boiling_k)
        .unwrap_or(v.boiling_k)
}

fn whole_scale(values: &[f64]) -> f64 {
    (1..=6)
        .map(f64::from)
        .find(|s| {
            values
                .iter()
                .all(|v| ((s * v) - (s * v).round()).abs() < 1e-9)
        })
        .unwrap_or(1.0)
}

fn displacement_equation(d: &Displacement, gas: bool) -> String {
    let product = d.product.unwrap_or_default();
    let s = whole_scale(&[d.acid_per, d.product_per, d.released]);
    let arrow = if gas { "↑" } else { "" };
    format!(
        "{} + {} → {} + {}{arrow}",
        term(s, &display(&d.salt)),
        term(s * d.acid_per, &display(d.acid)),
        term(s * d.product_per, &display(product)),
        term(s * d.released, &display(d.volatile.acid)),
    )
}

// ── oxidation by the acid ────────────────────────────────────────────

/// One balanced oxidation, per unit of reductant.
#[derive(Debug, Clone)]
pub struct AcidOxidation {
    pub acid: &'static OxidisingAcid,
    pub reductant: &'static AcidReductant,
    /// Acid consumed (as the neutral acid) per reductant.
    pub acid_per: f64,
    /// (key, amount, phase) products per reductant, merged by key; water
    /// is listed last when produced.
    pub products: Vec<(String, f64, Phase)>,
    /// Water consumed per reductant (0 when produced).
    pub water_in: f64,
    /// A product the registry lacks (dormant row).
    pub missing: Option<String>,
}

/// Balance one acid half against one reductant half.
pub fn balance(acid: &'static OxidisingAcid, r: &'static AcidReductant) -> AcidOxidation {
    let q = r.electrons / acid.electrons; // acid halves per reductant
    let anion_q = formula_of(acid.anion).map(|f| -f.charge).unwrap_or(1.0);
    let mut products: Vec<(String, f64, Phase)> = Vec::new();
    let mut missing = None;
    let push =
        |products: &mut Vec<(String, f64, Phase)>, key: &str, n: f64, phase: Phase| match products
            .iter_mut()
            .find(|(k, ..)| k == key)
        {
            Some(slot) => slot.1 += n,
            None => products.push((key.to_string(), n, phase)),
        };
    let mut counter_anions = 0.0;
    match r.oxidised {
        Oxidised::Cation { ion, charge } => {
            counter_anions = charge / anion_q;
            // The metal's salt of the acid anion, found by formula.
            let salt = formula_of(ion)
                .zip(formula_of(acid.anion))
                .and_then(|(c, a)| {
                    let mut counts = c.counts.clone();
                    for (e, n) in a.counts {
                        *counts.entry(e).or_default() += n * counter_anions;
                    }
                    registry_with_counts(&counts)
                });
            match salt {
                Some(key) => {
                    let phase = if acid.medium == Medium::Aqueous {
                        Phase::Aqueous
                    } else {
                        Phase::Solid
                    };
                    push(&mut products, key, 1.0, phase)
                }
                None => {
                    missing = Some(format!(
                        "{}({})",
                        crate::halogen_redox::pretty(ion),
                        crate::halogen_redox::pretty(acid.anion)
                    ))
                }
            }
        }
        Oxidised::Molecule { key, phase } => {
            push(&mut products, key, 1.0, phase);
            if !in_registry(key) {
                missing = Some(key.to_string());
            }
        }
    }
    let reduced_phase = Phase::Gas;
    push(&mut products, acid.reduced, q, reduced_phase);
    if missing.is_none() && !in_registry(acid.reduced) {
        missing = Some(acid.reduced.to_string());
    }
    if missing.is_none() && !in_registry(acid.acid) {
        missing = Some(acid.acid.to_string());
    }
    let water = q * acid.water - r.water_consumed;
    if water > 0.0 {
        push(&mut products, "water", water, Phase::Liquid);
    }
    AcidOxidation {
        acid,
        reductant: r,
        acid_per: q + counter_anions,
        products,
        water_in: (-water).max(0.0),
        missing,
    }
}

/// Every (oxidising acid, reductant) row, balanced.
pub fn acid_oxidations() -> Vec<AcidOxidation> {
    let mut out = Vec::new();
    for acid in OXIDISING_ACIDS {
        for key in acid.reductants {
            if let Some(r) = ACID_REDUCTANTS.iter().find(|r| r.key == *key) {
                out.push(balance(acid, r));
            }
        }
    }
    out
}

/// The reductant couple's standard potential: a metal's from the activity
/// series, a non-metal's from its row.
pub fn reductant_e0(r: &AcidReductant) -> Option<f64> {
    match r.oxidised {
        Oxidised::Cation { ion, .. } => crate::displacement::SERIES
            .iter()
            .find(|c| c.oxidised == ion)
            .map(|c| c.e0_volts),
        Oxidised::Molecule { .. } => r.e0_v,
    }
}

fn medium_admits(vessel: &Vessel, acid: &OxidisingAcid, r: &AcidReductant) -> bool {
    match acid.medium {
        Medium::Concentrated { hot_onset_k } => {
            concentrated_acid(vessel, acid.acid)
                && hot_onset_k.is_none_or(|t| vessel.temperature.0 >= t)
        }
        Medium::Aqueous => {
            phase_moles(vessel, "water", &[Phase::Liquid]) > TRACE
                && crate::nonaqueous::aqueous_model_admits(vessel)
                && reductant_e0(r).is_some_and(|e| acid.e0_v > e)
        }
    }
}

fn acid_amount(vessel: &Vessel, acid: &OxidisingAcid) -> f64 {
    match acid.medium {
        Medium::Concentrated { .. } => neat(vessel, acid.acid),
        Medium::Aqueous => phase_moles(vessel, acid.acid, &[Phase::Liquid, Phase::Aqueous]),
    }
}

fn oxidation_extent(vessel: &Vessel, row: &AcidOxidation) -> f64 {
    if row.missing.is_some() || !medium_admits(vessel, row.acid, row.reductant) {
        return 0.0;
    }
    let mut x = phase_moles(vessel, row.reductant.key, &[Phase::Solid, Phase::Liquid])
        .min(acid_amount(vessel, row.acid) / row.acid_per);
    if row.water_in > 0.0 {
        x = x.min(phase_moles(vessel, "water", &[Phase::Liquid]) / row.water_in);
    }
    if let Medium::Concentrated { .. } = row.acid.medium {
        let made = row
            .products
            .iter()
            .find(|(k, ..)| k == "water")
            .map(|(_, n, _)| *n)
            .unwrap_or(0.0);
        if made > 0.0 {
            x = x.min(water_room(vessel, row.acid.acid) / made);
        }
    }
    x
}

/// The smallest whole-number equation for a balanced acid oxidation.
pub fn oxidation_equation(row: &AcidOxidation) -> String {
    let mut values = vec![1.0, row.acid_per, row.water_in];
    values.extend(row.products.iter().map(|(_, n, _)| *n));
    let s = whole_scale(&values);
    let hot = matches!(
        row.acid.medium,
        Medium::Concentrated {
            hot_onset_k: Some(_)
        }
    );
    let mut left = vec![
        term(s, &display(row.reductant.key)),
        term(s * row.acid_per, &display(row.acid.acid)),
    ];
    if row.water_in > 0.0 {
        left.push(term(s * row.water_in, "H₂O"));
    }
    let right: Vec<String> = row
        .products
        .iter()
        .map(|(k, n, phase)| {
            let text = if k == "water" {
                "H₂O".to_string()
            } else {
                display(k)
            };
            let arrow = if *phase == Phase::Gas { "↑" } else { "" };
            term(s * n, &format!("{text}{arrow}"))
        })
        .collect();
    format!(
        "{} {} {}",
        left.join(" + "),
        if hot { "→Δ" } else { "→" },
        right.join(" + ")
    )
}

// ── peroxide on hydrogen sulfide ─────────────────────────────────────

fn peroxide_extent(vessel: &Vessel) -> f64 {
    if phase_moles(vessel, "water", &[Phase::Liquid]) <= TRACE {
        return 0.0;
    }
    phase_moles(vessel, "H2S(aq)", &[Phase::Aqueous]).min(phase_moles(
        vessel,
        "H2O2",
        &[Phase::Liquid, Phase::Aqueous],
    ))
}

/// Whether any row can fire in this vessel now.
pub fn fires(vessel: &Vessel) -> bool {
    if peroxide_extent(vessel) > TRACE {
        return true;
    }
    if let Some(acid) = dehydrating_acid(vessel) {
        if carbohydrates(vessel)
            .iter()
            .any(|(key, _, water)| dehydration_extent(vessel, acid, key, *water) > TRACE)
        {
            return true;
        }
    }
    displacements(vessel)
        .iter()
        .any(|d| displacement_extent(vessel, d) > TRACE)
        || acid_oxidations()
            .iter()
            .any(|row| oxidation_extent(vessel, row) > TRACE)
}

fn withdraw(vessel: &mut Vessel, key: &str, moles: f64, phases: &[Phase]) {
    let id = SpeciesId::new(key);
    let mut left = moles;
    for phase in phases {
        if left <= 0.0 {
            break;
        }
        left -= vessel.withdraw_phase(&id, Moles(left), *phase).0;
    }
}

fn release_gas(vessel: &mut Vessel, key: &str, moles: f64, events: &mut Vec<Event>) {
    let species = SpeciesId::new(key);
    let n = Moles(moles);
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
}

fn concentrated_note(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.concentrated-sulfuric-qualitative",
            "this reaction of concentrated sulfuric acid is a qualitative curated route, not calibrated: the textbook stoichiometry is booked to the limiting reagent, the acid counts as concentrated only while water stays below half of the liquid (an editorial boundary, which also ends the dehydration once the product water has diluted it), \"hot\" is an editorial onset at the boiling point of water, and no rate, no reaction heat and no measured yield are claimed",
        ),
    )
}

fn oxidising_acid_note(vessel: &Vessel, acid: &str) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::new(
            "not-modeled.oxidising-acid-qualitative",
            "{acid} oxidised the metal or non-metal here by its own reduction couple (CRC standard potentials): the stoichiometry is booked to the limiting reagent with one reduction product, although real mixtures give several, and no rate, no reaction heat and no measured yield are claimed",
            vec![("acid".to_string(), Slot::text(display(acid)))],
        ),
    )
}

fn peroxide_note(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.sulfide-peroxide-qualitative",
            "hydrogen peroxide oxidising dissolved hydrogen sulfide to sulfur is a qualitative curated route, not calibrated: it is booked to completion of the limiting reagent as elemental sulfur, although further oxidation to sulfate is possible, and no rate, no reaction heat and no measured yield are claimed",
        ),
    )
}

fn partly_oxidised_note(vessel: &Vessel, d: &Displacement, halogen: &str) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::new(
            "not-modeled.volatile-acid-partly-oxidised",
            "{volatile} set free from {salt} by concentrated {acid} is partly oxidised by the acid to {halogen} and sulfur dioxide; no reviewed split between the two is available, so all of it is booked as {volatile}",
            vec![
                ("volatile".to_string(), Slot::text(display(d.volatile.acid))),
                ("salt".to_string(), Slot::text(display(&d.salt))),
                ("acid".to_string(), Slot::text(display(d.acid))),
                ("halogen".to_string(), Slot::text(display(halogen))),
            ],
        ),
    )
}

/// Typed notes for rows that would fire but cannot be written: a salt whose
/// volatile acid or acid salt the registry lacks, an iodide the acid
/// oxidises instead of displacing, an oxidation whose product is missing.
/// Read by the honesty tail every step (see `halogen_redox::unpriced_notes`).
pub fn dormant_notes(vessel: &Vessel) -> Vec<Event> {
    let mut out = Vec::new();
    for d in displacements(vessel) {
        if admitted(&d) {
            continue;
        }
        if d.volatile.refused {
            out.push(Event::not_modeled(
                vessel.id,
                NotModelledCause::NoReviewedDatum,
                Phrase::new(
                    "not-modeled.volatile-acid-oxidised-instead",
                    "concentrated {acid} oxidises the anion of {salt} (to {halogen}, sulfur dioxide, sulfur and hydrogen sulfide) instead of setting {volatile} free; no reviewed product split is available, so it is left unreacted. That is a gap in the model, not evidence that nothing happens",
                    vec![
                        ("acid".to_string(), Slot::text(display(d.acid))),
                        ("salt".to_string(), Slot::text(display(&d.salt))),
                        (
                            "halogen".to_string(),
                            Slot::text(display(d.volatile.oxidised_to.unwrap_or("I2"))),
                        ),
                        ("volatile".to_string(), Slot::text(display(d.volatile.acid))),
                    ],
                ),
            ));
            continue;
        }
        let missing = if !in_registry(d.volatile.acid) {
            crate::halogen_redox::pretty(d.volatile.acid)
        } else {
            d.wanted.clone()
        };
        out.push(acid_unregistered_note(
            vessel,
            d.acid,
            &display(&d.salt),
            &missing,
        ));
    }
    for row in acid_oxidations() {
        let Some(missing) = &row.missing else {
            continue;
        };
        let present = phase_moles(vessel, row.reductant.key, &[Phase::Solid, Phase::Liquid])
            > TRACE
            && acid_amount(vessel, row.acid) > TRACE;
        if present && medium_admits(vessel, row.acid, row.reductant) {
            out.push(acid_unregistered_note(
                vessel,
                row.acid.acid,
                &display(row.reductant.key),
                missing,
            ));
        }
    }
    out
}

fn acid_unregistered_note(vessel: &Vessel, acid: &str, with: &str, missing: &str) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::PhaseNotInRegistry,
        Phrase::new(
            "not-modeled.acid-route-product-unregistered",
            "concentrated {acid} would react with {reactant} here, but the registry has no {product} to write, so it is left unreacted. That is a gap in the model, not evidence that nothing happens",
            vec![
                ("acid".to_string(), Slot::text(display(acid))),
                ("reactant".to_string(), Slot::text(with.to_string())),
                ("product".to_string(), Slot::text(missing.to_string())),
            ],
        ),
    )
}

/// Apply every row to its limiting reagent. Returns whether anything reacted.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut concentrated_fired = false;
    let mut peroxide_fired = false;
    let mut oxidising: Vec<&'static str> = Vec::new();

    let x = peroxide_extent(vessel);
    if x > TRACE {
        withdraw(vessel, "H2S(aq)", x, &[Phase::Aqueous]);
        withdraw(vessel, "H2O2", x, &[Phase::Aqueous, Phase::Liquid]);
        vessel.deposit(SpeciesId::new("S"), Moles(x), Phase::Solid);
        vessel.deposit(SpeciesId::new("water"), Moles(2.0 * x), Phase::Liquid);
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: "H₂S + H₂O₂ → S↓ + 2 H₂O".to_string(),
        });
        peroxide_fired = true;
    }

    if let Some(acid) = dehydrating_acid(vessel) {
        for (key, carbon, water) in carbohydrates(vessel) {
            let x = dehydration_extent(vessel, acid, &key, water);
            if x <= TRACE {
                continue;
            }
            withdraw(vessel, &key, x, DEHYDRATABLE_PHASES);
            vessel.deposit(SpeciesId::new("C"), Moles(carbon * x), Phase::Solid);
            vessel.deposit(SpeciesId::new("water"), Moles(water * x), Phase::Liquid);
            let formula = crate::species::lookup_key(&key)
                .map(|d| d.formula)
                .unwrap_or(key.as_str());
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: format!("{formula} →({}) {carbon} C + {water} H₂O", display(acid)),
            });
            concentrated_fired = true;
        }
    }

    for d in displacements(vessel) {
        let x = displacement_extent(vessel, &d);
        if x <= TRACE {
            continue;
        }
        let product = d.product.expect("admitted displacement has a product");
        withdraw(vessel, &d.salt, x, &[Phase::Solid]);
        withdraw(vessel, d.acid, x * d.acid_per, &[Phase::Liquid]);
        vessel.deposit(
            SpeciesId::new(product),
            Moles(x * d.product_per),
            Phase::Solid,
        );
        let gas = vessel.temperature.0 >= volatile_boiling_k(d.volatile);
        if gas {
            release_gas(vessel, d.volatile.acid, x * d.released, events);
        } else {
            vessel.deposit(
                SpeciesId::new(d.volatile.acid),
                Moles(x * d.released),
                Phase::Liquid,
            );
        }
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: displacement_equation(&d, gas),
        });
        if let Some(halogen) = d.volatile.oxidised_to {
            events.push(partly_oxidised_note(vessel, &d, halogen));
        }
        concentrated_fired = true;
    }

    for row in acid_oxidations() {
        let x = oxidation_extent(vessel, &row);
        if x <= TRACE {
            continue;
        }
        let acid_phases: &[Phase] = match row.acid.medium {
            Medium::Concentrated { .. } => &[Phase::Liquid],
            Medium::Aqueous => &[Phase::Aqueous, Phase::Liquid],
        };
        withdraw(vessel, row.reductant.key, x, &[Phase::Solid, Phase::Liquid]);
        withdraw(vessel, row.acid.acid, row.acid_per * x, acid_phases);
        if row.water_in > 0.0 {
            withdraw(vessel, "water", row.water_in * x, &[Phase::Liquid]);
        }
        for (key, coeff, phase) in &row.products {
            if *phase == Phase::Gas {
                release_gas(vessel, key, coeff * x, events);
            } else {
                vessel.deposit(SpeciesId::new(key.as_str()), Moles(coeff * x), *phase);
            }
        }
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: oxidation_equation(&row),
        });
        if row.acid.acid == "H2SO4" {
            concentrated_fired = true;
        } else if !oxidising.contains(&row.acid.acid) {
            oxidising.push(row.acid.acid);
        }
    }

    if peroxide_fired {
        events.push(peroxide_note(vessel));
    }
    if concentrated_fired {
        events.push(concentrated_note(vessel));
    }
    for acid in &oxidising {
        events.push(oxidising_acid_note(vessel, acid));
    }
    peroxide_fired || concentrated_fired || !oxidising.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Kelvin;
    use crate::VesselId;
    use std::collections::BTreeMap;

    fn beaker() -> Vessel {
        Vessel::new(VesselId(0), "beaker")
    }

    fn noted(events: &[Event], key: &str) -> bool {
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { reason: Some(r), .. } if r.key == key))
    }

    fn element_total(v: &Vessel, el: &str) -> f64 {
        v.contents
            .iter()
            .map(|p| {
                let formula = crate::species::lookup_key(&p.species.0)
                    .map(|d| d.formula)
                    .unwrap_or_else(|| panic!("{}", p.species.0));
                p.moles.0
                    * crate::stoich::parse_formula(formula)
                        .unwrap()
                        .counts
                        .get(el)
                        .copied()
                        .unwrap_or(0.0)
            })
            .sum()
    }

    #[test]
    fn concentrated_acid_chars_sugar_until_its_own_water_dilutes_it() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("sucrose"), Moles(0.005), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.0375), Phase::Liquid);
        let before = ["C", "H", "O", "S"].map(|e| element_total(&v, e));
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        // 0.0375 mol acid takes at most 0.0375 mol water before water is half
        // the liquid: 0.0375 / 11 of sucrose chars, the rest is left.
        let charred = 0.0375 / 11.0;
        assert!((phase_moles(&v, "C", &[Phase::Solid]) - 12.0 * charred).abs() < 1e-12);
        assert!((phase_moles(&v, "sucrose", &[Phase::Solid]) - (0.005 - charred)).abs() < 1e-12);
        assert!(noted(
            &events,
            "not-modeled.concentrated-sulfuric-qualitative"
        ));
        let after = ["C", "H", "O", "S"].map(|e| element_total(&v, e));
        for (b, a) in before.iter().zip(after) {
            assert!((b - a).abs() < 1e-12, "{before:?} {after:?}");
        }
        // Now diluted to the boundary: nothing more fires.
        assert!(!fires(&v));
    }

    #[test]
    fn dilute_acid_neither_chars_nor_releases_hydrogen_chloride() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        v.deposit(SpeciesId::new("sucrose"), Moles(0.005), Phase::Solid);
        v.deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.02), Phase::Liquid);
        v.temperature = Kelvin(380.0);
        v.deposit(SpeciesId::new("Cu"), Moles(0.01), Phase::Solid);
        assert!(!fires(&v));
    }

    #[test]
    fn salt_and_concentrated_acid_give_hydrogen_chloride() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("NaCl"), Moles(0.05), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.03), Phase::Liquid);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        let hcl: f64 = events
            .iter()
            .map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == "HCl" => moles.0,
                _ => 0.0,
            })
            .sum();
        assert!((hcl - 0.03).abs() < 1e-12);
        assert!((phase_moles(&v, "NaHSO4", &[Phase::Solid]) - 0.03).abs() < 1e-12);
        assert!((phase_moles(&v, "NaCl", &[Phase::Solid]) - 0.02).abs() < 1e-12);
    }

    #[test]
    fn copper_needs_hot_acid_and_gives_sulfur_dioxide() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("Cu"), Moles(0.005), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        assert!(!fires(&v), "cold concentrated acid leaves copper");
        v.temperature = Kelvin(420.0);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!(phase_moles(&v, "Cu", &[Phase::Solid]) < 1e-12);
        assert!((phase_moles(&v, "CuSO4", &[Phase::Solid]) - 0.005).abs() < 1e-12);
        let so2: f64 = events
            .iter()
            .map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == "SO2" => moles.0,
                _ => 0.0,
            })
            .sum();
        assert!((so2 - 0.005).abs() < 1e-12);
        assert!((acid(&v) - 0.04).abs() < 1e-12);
    }

    #[test]
    fn hot_acid_oxidises_carbon_and_sulfur_but_not_graphite() {
        for (key, so2) in [("C", 2.0), ("activated_charcoal", 2.0), ("S", 3.0)] {
            let mut v = beaker();
            v.deposit(SpeciesId::new(key), Moles(0.002), Phase::Solid);
            v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
            v.temperature = Kelvin(420.0);
            let mut events = Vec::new();
            assert!(apply(&mut v, &mut events), "{key}");
            let made: f64 = events
                .iter()
                .map(|e| match e {
                    Event::GasEvolved { species, moles, .. } if species.0 == "SO2" => moles.0,
                    _ => 0.0,
                })
                .sum();
            assert!((made - so2 * 0.002).abs() < 1e-12, "{key}: {made}");
        }
        let mut v = beaker();
        v.deposit(SpeciesId::new("graphite"), Moles(0.002), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        v.temperature = Kelvin(420.0);
        assert!(!fires(&v));
    }

    #[test]
    fn peroxide_oxidises_dissolved_hydrogen_sulfide_to_sulfur() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(0.55), Phase::Liquid);
        v.deposit(SpeciesId::new("H2S(aq)"), Moles(0.001), Phase::Aqueous);
        v.deposit(SpeciesId::new("H2O2"), Moles(0.005), Phase::Liquid);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!((phase_moles(&v, "S", &[Phase::Solid]) - 0.001).abs() < 1e-12);
        assert!((phase_moles(&v, "H2O2", &[Phase::Liquid]) - 0.004).abs() < 1e-12);
        assert!(noted(&events, "not-modeled.sulfide-peroxide-qualitative"));
        // No peroxide, no sulfur.
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(0.55), Phase::Liquid);
        v.deposit(SpeciesId::new("H2S(aq)"), Moles(0.001), Phase::Aqueous);
        assert!(!fires(&v));
    }

    // ── generic rows ─────────────────────────────────────────────────

    fn gas(events: &[Event], key: &str) -> f64 {
        events
            .iter()
            .map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == key => moles.0,
                _ => 0.0,
            })
            .sum()
    }

    fn equations(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|e| match e {
                Event::ReactionOccurred { equation, .. } => Some(equation.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn carbohydrates_are_a_formula_class_not_a_name_list() {
        let class = |f: &str| carbohydrate_class(&crate::stoich::parse_formula(f).unwrap());
        assert_eq!(class("C12H22O11"), Some((12.0, 11.0)));
        assert_eq!(class("C6H12O6"), Some((6.0, 6.0)));
        assert_eq!(class("C6H10O5"), Some((6.0, 5.0)));
        assert_eq!(class("C5H10O5"), Some((5.0, 5.0)));
        // Acetic and lactic acid and formaldehyde are Cn(H2O)n but not sugars.
        assert_eq!(class("C2H4O2"), None);
        assert_eq!(class("C3H6O3"), None);
        assert_eq!(class("CH2O"), None);
        // Not Cn(H2O)m at all.
        assert_eq!(class("C6H8O7"), None);
        assert_eq!(class("C6H8O6"), None);
    }

    #[test]
    fn every_registry_carbohydrate_chars_and_acids_do_not() {
        for key in ["fructose", "starch", "cellulose", "maltose"] {
            let mut v = beaker();
            v.deposit(SpeciesId::new(key), Moles(0.001), Phase::Solid);
            v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
            let mut events = Vec::new();
            assert!(apply(&mut v, &mut events), "{key}");
            assert!(phase_moles(&v, "C", &[Phase::Solid]) > 0.0, "{key}");
        }
        for key in ["lactic_acid", "CH3COOH", "citric_acid"] {
            let mut v = beaker();
            v.deposit(SpeciesId::new(key), Moles(0.001), Phase::Solid);
            v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
            assert!(!fires(&v), "{key} is not a carbohydrate");
        }
    }

    #[test]
    fn concentrated_acid_sets_acetic_acid_free_from_sodium_acetate() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("NaOAc"), Moles(0.01), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        let before = ["C", "H", "O", "S", "Na"].map(|e| element_total(&v, e));
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        // Below its boiling point the acid stays in the liquid.
        assert!((phase_moles(&v, "CH3COOH", &[Phase::Liquid]) - 0.01).abs() < 1e-12);
        assert!((phase_moles(&v, "NaHSO4", &[Phase::Solid]) - 0.01).abs() < 1e-12);
        assert!((acid(&v) - 0.04).abs() < 1e-12);
        assert!(equations(&events).contains(&"CH₃COONa + H₂SO₄ → NaHSO₄ + CH₃COOH".to_string()));
        let after = ["C", "H", "O", "S", "Na"].map(|e| element_total(&v, e));
        for (b, a) in before.iter().zip(after) {
            assert!((b - a).abs() < 1e-12, "{before:?} {after:?}");
        }
    }

    #[test]
    fn a_divalent_chloride_gives_two_hydrogen_chloride_per_formula_unit() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("MgCl2"), Moles(0.004), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!((gas(&events, "HCl") - 0.008).abs() < 1e-12);
        assert!((phase_moles(&v, "MgSO4", &[Phase::Solid]) - 0.004).abs() < 1e-12);
        assert!(equations(&events).contains(&"MgCl₂ + H₂SO₄ → MgSO₄ + 2 HCl↑".to_string()));
    }

    #[test]
    fn bromide_gives_hydrogen_bromide_with_a_typed_oxidation_note() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("NaBr"), Moles(0.01), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!((gas(&events, "HBr") - 0.01).abs() < 1e-12);
        assert!(noted(&events, "not-modeled.volatile-acid-partly-oxidised"));
    }

    #[test]
    fn salts_whose_products_are_unregistered_stay_and_say_so() {
        // No KHSO4/K2SO4 in the registry; no HNO3 either.
        for key in ["KCl", "KNO3", "NaNO3"] {
            let mut v = beaker();
            v.deposit(SpeciesId::new(key), Moles(0.01), Phase::Solid);
            v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
            assert!(!fires(&v), "{key}");
            assert!(
                noted(
                    &dormant_notes(&v),
                    "not-modeled.acid-route-product-unregistered"
                ),
                "{key}"
            );
        }
        // Iodide is oxidised, not displaced.
        let mut v = beaker();
        v.deposit(SpeciesId::new("KI"), Moles(0.01), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        assert!(!fires(&v));
        assert!(noted(
            &dormant_notes(&v),
            "not-modeled.volatile-acid-oxidised-instead"
        ));
        // Dilute acid: neither fires nor notes.
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        v.deposit(SpeciesId::new("NaOAc"), Moles(0.01), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.02), Phase::Liquid);
        assert!(!fires(&v));
        assert!(dormant_notes(&v).is_empty());
    }

    #[test]
    fn every_acid_oxidation_closes_atoms_and_charge() {
        let rows = acid_oxidations();
        assert!(rows.len() >= 6);
        for row in &rows {
            let mut el: BTreeMap<String, f64> = BTreeMap::new();
            let mut add = |key: &str, n: f64| {
                for (e, c) in formula_of(key).unwrap().counts {
                    *el.entry(e).or_default() += n * c;
                }
            };
            add(row.reductant.key, -1.0);
            add(row.acid.acid, -row.acid_per);
            add("water", -row.water_in);
            // A dormant metal row has no salt to count; its anion and
            // cation are still balanced by construction.
            if row.missing.is_some() && matches!(row.reductant.oxidised, Oxidised::Cation { .. }) {
                continue;
            }
            for (k, n, _) in &row.products {
                add(k, *n);
            }
            assert!(
                el.values().all(|v| v.abs() < 1e-9),
                "{}: {el:?}",
                oxidation_equation(row)
            );
        }
    }

    #[test]
    fn acid_oxidation_equations_are_built_from_the_halves() {
        let rows = acid_oxidations();
        let eq = |acid: &str, red: &str, product: &str| {
            rows.iter()
                .find(|r| {
                    r.acid.acid == acid && r.reductant.key == red && r.acid.reduced == product
                })
                .map(oxidation_equation)
                .unwrap()
        };
        assert_eq!(
            eq("H2SO4", "Cu", "SO2"),
            "Cu + 2 H₂SO₄ →Δ CuSO₄ + SO₂↑ + 2 H₂O"
        );
        assert_eq!(
            eq("H2SO4", "C", "SO2"),
            "C + 2 H₂SO₄ →Δ CO₂↑ + 2 SO₂↑ + 2 H₂O"
        );
        assert_eq!(eq("H2SO4", "S", "SO2"), "S + 2 H₂SO₄ →Δ 3 SO₂↑ + 2 H₂O");
        assert_eq!(
            eq("H2SO4", "activated_charcoal", "SO2"),
            "C + 2 H₂SO₄ →Δ CO₂↑ + 2 SO₂↑ + 2 H₂O"
        );
        // Nitric acid rows are dormant until the registry carries HNO3, NO,
        // NO2 and copper(II) nitrate; their balance is already fixed.
        let dilute = rows
            .iter()
            .find(|r| r.acid.acid == "HNO3" && r.acid.reduced == "NO")
            .unwrap();
        assert!((dilute.acid_per - 8.0 / 3.0).abs() < 1e-12);
        assert!(dilute.missing.is_some());
        // Copper's potential comes from the activity series, below both
        // nitrate couples and above sulfate/sulfite at unit activity.
        let cu = ACID_REDUCTANTS.iter().find(|r| r.key == "Cu").unwrap();
        let e_cu = reductant_e0(cu).unwrap();
        assert!((e_cu - 0.3419).abs() < 1e-9);
        assert!(OXIDISING_ACIDS[1].e0_v > e_cu && OXIDISING_ACIDS[2].e0_v > e_cu);
        assert!(
            OXIDISING_ACIDS[0].e0_v < e_cu,
            "dilute sulfuric acid leaves copper"
        );
    }
}
