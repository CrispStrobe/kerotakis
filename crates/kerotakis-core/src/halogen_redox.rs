//! Halogen / oxyhalogen redox: one generic, stoichiometry-checked writer.
//!
//! Every reaction here is BUILT, not listed: an oxidant reduction half and
//! a reductant reduction half (run backwards) are scaled to a common
//! electron count, their protons and water netted, and the result is
//! element- and charge-checked before it may touch a vessel. The halves'
//! standard potentials are derived from `vendor/iphreeqc/database/llnl.dat`
//! (USGS conversion of LLNL thermo.com.V8.R6; public domain) by
//! `tools/halogen_redox_coverage.py`, which also writes the coverage table
//! `docs/halogen-redox-coverage-20261027.{md,json}` that
//! `halogen_redox_coverage_is_current` holds this module to.
//!
//! A pair is applied only when ALL of these hold, and each failure is a
//! named status in the table, never a silent partial reaction:
//!
//! * both members exist in the registry as the ion or a Na/K salt of it;
//! * no curated row or kinetic law already owns a form of both (the
//!   Landolt iodate/bisulfite law and the curated iodometric rows stay
//!   theirs);
//! * both couples passed their cross-check against an independent value;
//! * `ΔE° ≥ COMPLETE_MARGIN_V`, so a limiting-reagent writer is honest
//!   (Fe³⁺/I⁻ at 0.23 V is an equilibrium, not a completion);
//! * the reductant's oxidised product is fixed for this oxidant
//!   (thiosulfate → tetrathionate only with iodine);
//! * a reviewed source says the pair is fast on the bench (`FAST`).
//!   Thermodynamics alone never admits a pair: chlorate and iodate are
//!   powerful and slow.
//!
//! Admission at run time adds: liquid water present, at least one reactant
//! dissolved, reactants counted and withdrawn phase-specifically from
//! `REACTANT_PHASES` (dissolved, then solid in contact; never gas),
//! temperature inside `DOMAIN_K`, and, when the net equation consumes
//! protons, represented acidity — which limits the extent exactly as
//! `CuratedReaction::acid_protons` does. Protons the net equation releases
//! are left in the charge balance, the ledger's convention for strong acid.
//! No reaction heat is applied (no reviewed enthalpies for these rows).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::ops::Event;
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

/// Minimum ΔE° (V) for a writer that runs to completion. At 0.30 V even a
/// one-electron pair has K ≈ 1.2·10⁵, so a millimolar vessel ends within
/// a fraction of a percent of the limiting reagent.
pub const COMPLETE_MARGIN_V: f64 = 0.30;

/// Temperatures (K) for which the 25 °C potentials are used. A bench
/// aqueous range; outside it the writer stays out of the vessel.
pub const DOMAIN_K: (f64, f64) = (273.15, 333.15);

const TRACE: f64 = 1e-12;

/// One reduction half: `oxidised + electrons e⁻ + protons H⁺ → reduced + water H₂O`.
#[derive(Debug, Clone, Copy)]
pub struct Half {
    pub id: &'static str,
    pub oxidised: &'static str,
    pub reduced: &'static str,
    pub reduced_per_oxidised: f64,
    pub electrons: f64,
    pub protons: f64,
    pub water: f64,
    /// Standard potential at 25 °C, pH 0, as derived in the coverage JSON.
    pub e0_v: f64,
    /// Whether the derived value passed its independent cross-check.
    pub cross_check: bool,
}

/// The halves, in the generator's order. Values are copied from the
/// generated JSON and pinned to it by `halogen_redox_coverage_is_current`.
pub const HALVES: &[Half] = &[
    half("I2/I-", "I2", "I-", 2.0, 2.0, 0.0, 0.0, 0.5355, true),
    half("I3-/I-", "I3-", "I-", 3.0, 2.0, 0.0, 0.0, 0.5404, true),
    half("IO3-/I-", "IO3-", "I-", 1.0, 6.0, 6.0, 3.0, 1.0975, true),
    half("ClO-/Cl-", "ClO-", "Cl-", 1.0, 2.0, 2.0, 1.0, 1.7186, true),
    half("HClO/Cl-", "HClO", "Cl-", 1.0, 2.0, 1.0, 1.0, 1.4947, true),
    half(
        "ClO3-/Cl-",
        "ClO3-",
        "Cl-",
        1.0,
        6.0,
        6.0,
        3.0,
        1.4421,
        true,
    ),
    half("Cl2/Cl-", "Cl2", "Cl-", 2.0, 2.0, 0.0, 0.0, 1.358, true),
    half("Br2/Br-", "Br2", "Br-", 2.0, 2.0, 0.0, 0.0, 1.1038, true),
    half("Br3-/Br-", "Br3-", "Br-", 3.0, 2.0, 0.0, 0.0, 1.0627, true),
    half("BrO-/Br-", "BrO-", "Br-", 1.0, 2.0, 2.0, 1.0, 1.5948, true),
    half(
        "BrO3-/Br-",
        "BrO3-",
        "Br-",
        1.0,
        6.0,
        6.0,
        3.0,
        1.4409,
        true,
    ),
    half(
        "Fe+3/Fe+2",
        "Fe+3",
        "Fe+2",
        1.0,
        1.0,
        0.0,
        0.0,
        0.7696,
        true,
    ),
    // llnl.dat's S4O6-2 entry is in electron form with an O2-form log_k:
    // the derived 3.84 V fails its cross-check, so no pair uses it.
    half(
        "S4O6-2/S2O3-2",
        "S4O6-2",
        "S2O3-2",
        2.0,
        2.0,
        0.0,
        0.0,
        3.8393,
        false,
    ),
    half(
        "SO4-2/H2SO3",
        "SO4-2",
        "H2SO3",
        1.0,
        2.0,
        4.0,
        1.0,
        0.1652,
        true,
    ),
    half(
        "SO4-2/SO3-2",
        "SO4-2",
        "SO3-2",
        1.0,
        2.0,
        2.0,
        1.0,
        -0.1073,
        true,
    ),
    half(
        "SO4-2/HSO3-",
        "SO4-2",
        "HSO3-",
        1.0,
        2.0,
        3.0,
        1.0,
        0.1059,
        true,
    ),
];

#[allow(clippy::too_many_arguments)]
const fn half(
    id: &'static str,
    oxidised: &'static str,
    reduced: &'static str,
    reduced_per_oxidised: f64,
    electrons: f64,
    protons: f64,
    water: f64,
    e0_v: f64,
    cross_check: bool,
) -> Half {
    Half {
        id,
        oxidised,
        reduced,
        reduced_per_oxidised,
        electrons,
        protons,
        water,
        e0_v,
        cross_check,
    }
}

/// Oxidant halves (their oxidised member is the oxidant).
pub const OXIDANTS: &[&str] = &[
    "I2/I-",
    "I3-/I-",
    "IO3-/I-",
    "ClO-/Cl-",
    "HClO/Cl-",
    "ClO3-/Cl-",
    "Cl2/Cl-",
    "Br2/Br-",
    "Br3-/Br-",
    "BrO-/Br-",
    "BrO3-/Br-",
    "Fe+3/Fe+2",
];

/// Reductant halves (their reduced member is the reductant).
pub const REDUCTANTS: &[&str] = &[
    "S4O6-2/S2O3-2",
    "SO4-2/SO3-2",
    "SO4-2/HSO3-",
    "Fe+3/Fe+2",
    "I2/I-",
];

/// Reductants the lane names that have no reviewed couple anywhere.
pub const UNCOUPLED_REDUCTANTS: &[&str] = &["ascorbic_acid"];

/// Reductant halves whose oxidised product holds only with these oxidants.
pub const ONLY_WITH: &[(&str, &[&str])] = &[("S4O6-2/S2O3-2", &["I2/I-", "I3-/I-"])];

/// Pairs a reviewed source calls fast and quantitative on the bench.
pub const FAST: &[(&str, &str)] = &[
    ("I2/I-", "S4O6-2/S2O3-2"),
    ("I3-/I-", "S4O6-2/S2O3-2"),
    ("I2/I-", "SO4-2/SO3-2"),
    ("I2/I-", "SO4-2/HSO3-"),
    ("I3-/I-", "SO4-2/SO3-2"),
    ("I3-/I-", "SO4-2/HSO3-"),
    ("ClO-/Cl-", "I2/I-"),
    ("HClO/Cl-", "I2/I-"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairStatus {
    SelfPair,
    NoRegistryForm,
    NoReviewedCouple,
    OwnedElsewhere,
    CoupleRejected,
    NotFavourable,
    EquilibriumLimited,
    ProductNotDetermined,
    RateNotReviewed,
    Admitted,
}

impl PairStatus {
    pub fn key(self) -> &'static str {
        match self {
            PairStatus::SelfPair => "self",
            PairStatus::NoRegistryForm => "no_registry_form",
            PairStatus::NoReviewedCouple => "no_reviewed_couple",
            PairStatus::OwnedElsewhere => "owned_elsewhere",
            PairStatus::CoupleRejected => "couple_rejected",
            PairStatus::NotFavourable => "not_favourable",
            PairStatus::EquilibriumLimited => "equilibrium_limited",
            PairStatus::ProductNotDetermined => "product_not_determined",
            PairStatus::RateNotReviewed => "rate_not_reviewed",
            PairStatus::Admitted => "admitted",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pair {
    pub oxidant: &'static str,
    pub reductant: &'static str,
    pub delta_e_v: Option<f64>,
    pub status: PairStatus,
    pub oxidant_forms: Vec<String>,
    pub reductant_forms: Vec<String>,
}

pub fn half_by_id(id: &str) -> Option<&'static Half> {
    HALVES.iter().find(|h| h.id == id)
}

/// Registry keys that deliver `ion` in solution: the ion itself, and every
/// neutral salt whose formula minus the ion is only Na/K balancing its
/// charge. Derived from the registry, so a new salt joins automatically.
pub fn registry_forms(ion: &str) -> Vec<String> {
    let mut forms = Vec::new();
    if crate::species::REGISTRY.iter().any(|s| s.key == ion) {
        forms.push(ion.to_string());
    }
    let Ok(ion_f) = crate::stoich::parse_formula(ion) else {
        return forms;
    };
    if ion_f.charge >= 0.0 {
        return forms;
    }
    let mut salts: Vec<String> = crate::species::REGISTRY
        .iter()
        .filter(|s| s.key != ion)
        .filter_map(|s| {
            let f = crate::stoich::parse_formula(s.formula).ok()?;
            if f.charge != 0.0 {
                return None;
            }
            spectators(&f.counts, &ion_f)?;
            Some(s.key.to_string())
        })
        .collect();
    salts.sort();
    forms.extend(salts);
    forms
}

/// The Na⁺/K⁺ a salt releases beside one `ion`, or `None` if the salt is
/// not exactly that ion plus monovalent alkali counter-ions.
fn spectators(
    salt: &BTreeMap<String, f64>,
    ion: &crate::stoich::Formula,
) -> Option<Vec<(&'static str, f64)>> {
    let mut rest = salt.clone();
    for (el, n) in &ion.counts {
        let have = rest.remove(el)?;
        if (have - n).abs() > 1e-9 {
            return None;
        }
    }
    if rest.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    let mut charge = 0.0;
    for (el, n) in rest {
        let cation = match el.as_str() {
            "Na" => "Na+",
            "K" => "K+",
            _ => return None,
        };
        charge += n;
        out.push((cation, n));
    }
    ((charge + ion.charge).abs() < 1e-9).then_some(out)
}

fn spectators_of(form: &str, ion: &str) -> Vec<(&'static str, f64)> {
    if form == ion {
        return Vec::new();
    }
    let (Some(data), Ok(ion_f)) = (
        crate::species::lookup_key(form),
        crate::stoich::parse_formula(ion),
    ) else {
        return Vec::new();
    };
    crate::stoich::parse_formula(data.formula)
        .ok()
        .and_then(|f| spectators(&f.counts, &ion_f))
        .unwrap_or_default()
}

/// Reactant key sets of every curated row and every kinetic law.
fn owner_sets() -> Vec<Vec<&'static str>> {
    let mut sets: Vec<Vec<&'static str>> = crate::curated::REACTIONS
        .iter()
        .map(|r| r.reactants.iter().map(|(k, _)| *k).collect())
        .collect();
    sets.extend(crate::kinetics::REGISTRY.iter().map(|r| {
        r.stoichiometry
            .iter()
            .filter(|t| t.coefficient < 0.0)
            .map(|t| t.species)
            .collect()
    }));
    sets
}

/// Classify every oxidant × reductant pair. The precedence is the
/// generator's: the first failing condition names the status.
pub fn pairs() -> &'static [Pair] {
    static PAIRS: OnceLock<Vec<Pair>> = OnceLock::new();
    PAIRS.get_or_init(|| {
        let owners = owner_sets();
        let mut out = Vec::new();
        for &o in OXIDANTS {
            let ho = half_by_id(o).expect("oxidant half");
            let ox_forms = registry_forms(ho.oxidised);
            for &r in REDUCTANTS {
                let hr = half_by_id(r).expect("reductant half");
                let red_forms = registry_forms(hr.reduced);
                let de = ((ho.e0_v - hr.e0_v) * 1e4).round() / 1e4;
                let owned = owners.iter().any(|set| {
                    set.iter().any(|k| ox_forms.iter().any(|f| f == k))
                        && set.iter().any(|k| red_forms.iter().any(|f| f == k))
                });
                let status = if o == r {
                    PairStatus::SelfPair
                } else if ox_forms.is_empty() || red_forms.is_empty() {
                    PairStatus::NoRegistryForm
                } else if owned {
                    PairStatus::OwnedElsewhere
                } else if !(ho.cross_check && hr.cross_check) {
                    PairStatus::CoupleRejected
                } else if de <= 0.0 {
                    PairStatus::NotFavourable
                } else if de < COMPLETE_MARGIN_V {
                    PairStatus::EquilibriumLimited
                } else if ONLY_WITH
                    .iter()
                    .any(|(red, oxs)| *red == r && !oxs.contains(&o))
                {
                    PairStatus::ProductNotDetermined
                } else if !FAST.contains(&(o, r)) {
                    PairStatus::RateNotReviewed
                } else {
                    PairStatus::Admitted
                };
                out.push(Pair {
                    oxidant: o,
                    reductant: r,
                    delta_e_v: Some(de),
                    status,
                    oxidant_forms: ox_forms.clone(),
                    reductant_forms: red_forms,
                });
            }
            for &r in UNCOUPLED_REDUCTANTS {
                out.push(Pair {
                    oxidant: o,
                    reductant: r,
                    delta_e_v: None,
                    status: if ox_forms.is_empty() {
                        PairStatus::NoRegistryForm
                    } else {
                        PairStatus::NoReviewedCouple
                    },
                    oxidant_forms: ox_forms.clone(),
                    reductant_forms: vec![r.to_string()],
                });
            }
        }
        out
    })
}

/// One concrete writer: an admitted pair on one oxidant form and one
/// reductant form, balanced and checked.
#[derive(Debug, Clone)]
pub struct Row {
    pub equation: String,
    /// Aqueous reactants (form key, moles per extent).
    pub reactants: Vec<(String, f64)>,
    /// Liquid water consumed per extent (0 if none).
    pub water_in: f64,
    /// Protons drawn from represented acidity per extent.
    pub acid_protons: f64,
    /// Products: aqueous species, plus water as liquid.
    pub products: Vec<(String, f64, Phase)>,
}

fn lcm(a: f64, b: f64) -> f64 {
    let (a, b) = (a.round() as u64, b.round() as u64);
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    (a / x * b) as f64
}

/// Display spelling: subscript counts and superscript charge.
pub fn pretty(key: &str) -> String {
    if key == "water" {
        return "H₂O".into();
    }
    const SUB: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
    const SUP: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let (body, charge) = match key.rfind(['+', '-']) {
        Some(i) if i > 0 => (&key[..i], &key[i..]),
        _ => (key, ""),
    };
    let mut out: String = body
        .chars()
        .map(|c| c.to_digit(10).map(|d| SUB[d as usize]).unwrap_or(c))
        .collect();
    if let Some(sign) = charge.chars().next() {
        for c in charge[1..].chars() {
            if let Some(d) = c.to_digit(10) {
                out.push(SUP[d as usize]);
            }
        }
        out.push(if sign == '+' { '⁺' } else { '⁻' });
    }
    out
}

fn term(coef: f64, key: &str) -> String {
    if (coef - 1.0).abs() < 1e-9 {
        pretty(key)
    } else if (coef - coef.round()).abs() < 1e-9 {
        format!("{} {}", coef.round() as i64, pretty(key))
    } else {
        format!("{coef} {}", pretty(key))
    }
}

/// Build and balance-check the writer for one pair and two forms.
pub fn build_row(ho: &Half, hr: &Half, ox_form: &str, red_form: &str) -> Option<Row> {
    let l = lcm(ho.electrons, hr.electrons);
    let a = l / ho.electrons; // oxidant halves
    let b = l / hr.electrons; // reductant halves (run backwards)
    let red_n = b * hr.reduced_per_oxidised; // moles of reductant ion
    let h_net = a * ho.protons - b * hr.protons; // >0 consumed
    let w_net = a * ho.water - b * hr.water; // >0 produced

    let mut reactants = vec![(ox_form.to_string(), a), (red_form.to_string(), red_n)];
    if reactants[0].0 == reactants[1].0 {
        return None;
    }
    let mut products: Vec<(String, f64, Phase)> = vec![
        (
            ho.reduced.to_string(),
            a * ho.reduced_per_oxidised,
            Phase::Aqueous,
        ),
        (hr.oxidised.to_string(), b, Phase::Aqueous),
    ];
    let mut spect: Vec<(&'static str, f64)> = Vec::new();
    for (cat, n) in spectators_of(ox_form, ho.oxidised)
        .into_iter()
        .map(|(c, n)| (c, n * a))
        .chain(
            spectators_of(red_form, hr.reduced)
                .into_iter()
                .map(|(c, n)| (c, n * red_n)),
        )
    {
        match spect.iter_mut().find(|(c, _)| *c == cat) {
            Some(slot) => slot.1 += n,
            None => spect.push((cat, n)),
        }
    }
    // Merge products that share a key (I⁻ from both sides cannot occur in
    // an admitted pair, but a merge keeps the check honest if it did).
    for (cat, n) in spect {
        products.push((cat.to_string(), n, Phase::Aqueous));
    }
    let water_in = (-w_net).max(0.0);
    if w_net > 0.0 {
        products.push(("water".into(), w_net, Phase::Liquid));
    }
    let acid_protons = h_net.max(0.0);

    // Element and charge closure, with consumed protons on the left and
    // released protons on the right (the ledger keeps released ones as
    // charge, so they are not deposited).
    let mut left: Vec<(String, f64)> = reactants.clone();
    if water_in > 0.0 {
        left.push(("water".into(), water_in));
    }
    if acid_protons > 0.0 {
        left.push(("H+".into(), acid_protons));
    }
    let mut right: Vec<(String, f64)> = products.iter().map(|(k, n, _)| (k.clone(), *n)).collect();
    if h_net < 0.0 {
        right.push(("H+".into(), -h_net));
    }
    if !balanced(&left, &right) {
        return None;
    }

    let mut lhs: Vec<String> = reactants.iter().map(|(k, n)| term(*n, k)).collect();
    if water_in > 0.0 {
        lhs.push(term(water_in, "water"));
    }
    if acid_protons > 0.0 {
        lhs.push(term(acid_protons, "H+"));
    }
    let mut rhs: Vec<String> = products.iter().map(|(k, n, _)| term(*n, k)).collect();
    if h_net < 0.0 {
        rhs.push(term(-h_net, "H+"));
    }
    reactants.retain(|(_, n)| *n > 0.0);
    Some(Row {
        equation: format!("{} → {}", lhs.join(" + "), rhs.join(" + ")),
        reactants,
        water_in,
        acid_protons,
        products,
    })
}

fn formula_of(key: &str) -> Option<crate::stoich::Formula> {
    let text = if key == "water" {
        "H2O"
    } else {
        crate::species::lookup_key(key)
            .map(|d| d.formula)
            .unwrap_or(key)
    };
    crate::stoich::parse_formula(text).ok()
}

fn balanced(left: &[(String, f64)], right: &[(String, f64)]) -> bool {
    let mut el: BTreeMap<String, f64> = BTreeMap::new();
    let mut charge = 0.0;
    for (side, sign) in [(left, -1.0), (right, 1.0)] {
        for (key, n) in side {
            let Some(f) = formula_of(key) else {
                return false;
            };
            for (e, c) in f.counts {
                *el.entry(e).or_default() += sign * n * c;
            }
            charge += sign * n * f.charge;
        }
    }
    charge.abs() < 1e-9 && el.values().all(|v| v.abs() < 1e-9)
}

/// Every writer for every admitted pair. Rows that fail the balance check
/// are absent here and counted by `rejected_rows`.
pub fn rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut out = Vec::new();
        for pair in pairs().iter().filter(|p| p.status == PairStatus::Admitted) {
            let (ho, hr) = (
                half_by_id(pair.oxidant).unwrap(),
                half_by_id(pair.reductant).unwrap(),
            );
            for of in &pair.oxidant_forms {
                for rf in &pair.reductant_forms {
                    if let Some(row) = build_row(ho, hr, of, rf) {
                        out.push(row);
                    }
                }
            }
        }
        out
    })
}

/// Number of (admitted pair, form, form) combinations that failed closure.
pub fn rejected_rows() -> usize {
    pairs()
        .iter()
        .filter(|p| p.status == PairStatus::Admitted)
        .map(|p| {
            let (ho, hr) = (
                half_by_id(p.oxidant).unwrap(),
                half_by_id(p.reductant).unwrap(),
            );
            p.oxidant_forms
                .iter()
                .flat_map(|of| p.reductant_forms.iter().map(move |rf| (of, rf)))
                .filter(|(of, rf)| build_row(ho, hr, of, rf).is_none())
                .count()
        })
        .sum()
}

/// Phases a reactant is drawn from, in withdrawal order: dissolved first,
/// then undissolved solid in contact with the solution (it dissolves as the
/// dissolved share is consumed). Never gas or a separate liquid phase.
pub const REACTANT_PHASES: &[Phase] = &[Phase::Aqueous, Phase::Solid];

fn available(vessel: &Vessel, key: &str) -> f64 {
    let id = SpeciesId::new(key);
    let id = id.canonical();
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == id.as_ref() && REACTANT_PHASES.contains(&p.phase))
        .map(|p| p.moles.0)
        .sum()
}

fn liquid_water(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

fn in_domain(vessel: &Vessel) -> bool {
    (DOMAIN_K.0..=DOMAIN_K.1).contains(&vessel.temperature.0)
}

/// Extent this row could run to now, from aqueous inventory only.
pub fn extent(vessel: &Vessel, row: &Row) -> f64 {
    // An aqueous mechanism needs an aqueous medium, and at least one
    // reactant must already be dissolved: two dry powders do not titrate.
    if !in_domain(vessel) || liquid_water(vessel) <= 0.0 {
        return 0.0;
    }
    let dissolved = |k: &str| {
        let id = SpeciesId::new(k);
        let id = id.canonical();
        vessel
            .contents
            .iter()
            .any(|p| &p.species == id.as_ref() && p.phase == Phase::Aqueous && p.moles.0 > 0.0)
    };
    if !row.reactants.iter().any(|(k, _)| dissolved(k)) {
        return 0.0;
    }
    let mut x = row
        .reactants
        .iter()
        .map(|(k, n)| available(vessel, k) / n)
        .fold(f64::INFINITY, f64::min);
    if row.water_in > 0.0 {
        x = x.min(liquid_water(vessel) / row.water_in);
    }
    if row.acid_protons > 0.0 {
        x = x.min(crate::displacement::unspent_acidity(vessel) / row.acid_protons);
    }
    if x.is_finite() {
        x
    } else {
        0.0
    }
}

/// Whether any admitted writer can run in this vessel.
pub fn fires(vessel: &Vessel) -> bool {
    rows().iter().any(|row| extent(vessel, row) > TRACE)
}

/// Apply every writer that can run, once each, to completion. Returns
/// whether anything happened. The caller refreshes `solute_charge`.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut progressed = false;
    for row in rows() {
        let x = extent(vessel, row);
        if x <= TRACE {
            continue;
        }
        progressed = true;
        if row.acid_protons > 0.0 {
            crate::displacement::consume_acidity(vessel, x * row.acid_protons);
        }
        for (key, n) in &row.reactants {
            let mut left = x * n;
            for phase in REACTANT_PHASES {
                left -= vessel
                    .withdraw_phase(&SpeciesId::new(key), Moles(left), *phase)
                    .0;
            }
        }
        if row.water_in > 0.0 {
            vessel.withdraw_phase(
                &SpeciesId::new("water"),
                Moles(x * row.water_in),
                Phase::Liquid,
            );
        }
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: row.equation.clone(),
        });
        for (key, n, phase) in &row.products {
            vessel.deposit(SpeciesId::new(key), Moles(x * n), *phase);
        }
    }
    progressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_admitted_writer_closes_atoms_and_charge() {
        assert!(!rows().is_empty());
        assert_eq!(rejected_rows(), 0);
    }

    #[test]
    fn pretty_spells_ions_like_the_curated_rows() {
        assert_eq!(pretty("S2O3-2"), "S₂O₃²⁻");
        assert_eq!(pretty("I-"), "I⁻");
        assert_eq!(pretty("Na+"), "Na⁺");
        assert_eq!(pretty("Na2SO3"), "Na₂SO₃");
        assert_eq!(pretty("water"), "H₂O");
    }

    #[test]
    fn sulfite_iodimetry_row_is_the_textbook_equation() {
        let row = rows()
            .iter()
            .find(|r| r.reactants.iter().any(|(k, _)| k == "SO3-2"))
            .expect("I2 + SO3-2 admitted");
        assert_eq!(row.equation, "I₂ + SO₃²⁻ + H₂O → 2 I⁻ + SO₄²⁻ + 2 H⁺");
        assert_eq!(row.acid_protons, 0.0);
        assert_eq!(row.water_in, 1.0);
    }

    #[test]
    fn hypochlorite_iodide_needs_acid() {
        let row = rows()
            .iter()
            .find(|r| {
                r.reactants.iter().any(|(k, _)| k == "ClO-")
                    && r.reactants.iter().any(|(k, _)| k == "I-")
            })
            .expect("ClO- + I- admitted");
        assert_eq!(row.equation, "ClO⁻ + 2 I⁻ + 2 H⁺ → Cl⁻ + I₂ + H₂O");
        assert_eq!(row.acid_protons, 2.0);
    }

    #[test]
    fn slow_or_equilibrium_pairs_stay_out() {
        let status = |o: &str, r: &str| {
            pairs()
                .iter()
                .find(|p| p.oxidant == o && p.reductant == r)
                .unwrap()
                .status
        };
        // Dushman reaction: favourable, but a measured rate law, not instant.
        assert_eq!(status("IO3-/I-", "I2/I-"), PairStatus::RateNotReviewed);
        // Fe3+/I-: 0.23 V is an equilibrium, not a completion.
        assert_eq!(status("Fe+3/Fe+2", "I2/I-"), PairStatus::EquilibriumLimited);
        // Landolt: the kinetic law owns iodate/bisulfite.
        assert_eq!(status("IO3-/I-", "SO4-2/HSO3-"), PairStatus::OwnedElsewhere);
        // Curated iodometry owns iodine/thiosulfate.
        assert_eq!(status("I2/I-", "S4O6-2/S2O3-2"), PairStatus::OwnedElsewhere);
        // Iodine cannot oxidise iron(II).
        assert_eq!(status("I2/I-", "Fe+3/Fe+2"), PairStatus::NotFavourable);
        // No registry bromine species at all.
        assert_eq!(status("Br2/Br-", "I2/I-"), PairStatus::NoRegistryForm);
    }
}
