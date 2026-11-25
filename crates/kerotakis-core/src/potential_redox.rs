//! Potential-driven aqueous redox: one generic route, every pair a data row.
//!
//! Before this module, the only potential-driven redox route on the bench
//! was chlorine preparation (`MnO₂`/`MnO₄⁻` with acidified chloride) and the
//! chlorine displacements, both hard-wired in `chlorine.rs`. The same
//! chemistry with bromide or iodide, or with dichromate, was simply absent:
//! acidified permanganate and potassium iodide sat side by side and the
//! bench said "no reviewed writer prices this pair".
//!
//! # The route
//!
//! A [`Route`] pairs an oxidant [`Couple`] with a reductant [`Couple`], each a
//! reduction half `oxidised + n e⁻ + p H⁺ → r reduced + w H₂O` with a CRC
//! standard potential. The reductant half is run backwards, the two are
//! scaled to a common electron count, and the reaction runs until the two
//! Nernst potentials meet (ideal: molality stands in for activity, a pure
//! solid, a pure liquid or a gas at one atmosphere has unit activity) or a
//! reagent runs out, whichever is first. So dilute hydrochloric acid stays
//! inert over manganese dioxide while concentrated acid evolves chlorine,
//! and a thermodynamically complete displacement (chlorine on iodide) runs
//! to its limiting reagent. An equilibrium residue below [`RESOLUTION`] of
//! the limiting reagent is booked as complete: the bench does not resolve
//! it.
//!
//! Thermodynamics alone never admits a pair: chlorate and iodate are
//! powerful and slow. A route exists only where a reviewed source says the
//! reaction is the bench reaction (its `source`), and every firing says
//! through a typed note that the rate, the reaction heat and the activity
//! coefficients are not modelled.
//!
//! # Adding a substance or a pair
//!
//! A new oxidant or reductant is one [`Couple`] row (CRC potential, the half
//! reaction, the registry forms that deliver it); a new reaction is one
//! [`Route`] row. A route whose product the registry does not carry yet
//! (bromine, chromium(III)) is dormant: it says so with a typed
//! `not-modeled.potential-redox-product-unregistered` note naming the missing
//! species, and fires by itself the day the species joins the registry. See
//! `docs/reaction-routes.md`.
//!
//! # Data
//!
//! Standard reduction potentials at 25 °C, CRC Handbook of Chemistry and
//! Physics, "Electrochemical Series" (P. Vanýsek): MnO₂/Mn²⁺ 1.224 V,
//! MnO₄⁻/Mn²⁺ 1.507 V, Cr₂O₇²⁻/Cr³⁺ 1.232 V, Cl₂(g)/Cl⁻ 1.35827 V,
//! Br₂(l)/Br⁻ 1.066 V, I₂(s)/I⁻ 0.5355 V, Fe³⁺/Fe²⁺ 0.771 V.
//!
//! Iron(II) oxidised by permanganate is NOT a route here: the aqueous engine
//! already equilibrates Mn and Fe redox (wateq4f), and a second owner would
//! double-book it.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

const TRACE: f64 = 1e-12;

/// Relative equilibrium residue of the limiting reagent below which the
/// route is booked as complete (the bench's resolution, not a datum).
pub const RESOLUTION: f64 = 1e-9;

/// CRC "Electrochemical Series" standard potentials, V (see module notes).
pub const E0_MNO2: f64 = 1.224;
pub const E0_MNO4: f64 = 1.507;
pub const E0_CR2O7: f64 = 1.232;
pub const E0_CL2: f64 = 1.358_27;
pub const E0_BR2: f64 = 1.066;
pub const E0_I2: f64 = 0.5355;
pub const E0_FE3: f64 = 0.771;

/// One registry form that delivers a couple member, and the spectator ions
/// it releases per unit of the member.
#[derive(Debug, Clone, Copy)]
pub struct Form {
    pub key: &'static str,
    pub spectators: &'static [(&'static str, f64)],
}

const fn form(key: &'static str, spectators: &'static [(&'static str, f64)]) -> Form {
    Form { key, spectators }
}

/// A reduction half `oxidised + electrons e⁻ + protons H⁺ →
/// reduced_per_oxidised reduced + water H₂O`.
#[derive(Debug, Clone, Copy)]
pub struct Couple {
    pub id: &'static str,
    pub oxidised: &'static str,
    /// Registry forms of the oxidised member besides the member itself and
    /// its Na/K salts (which are found in the registry).
    pub oxidised_forms: &'static [Form],
    pub oxidised_phase: Phase,
    /// Pure solid, pure liquid or gas at one atmosphere.
    pub oxidised_unit_activity: bool,
    pub reduced: &'static str,
    pub reduced_per_oxidised: f64,
    pub reduced_forms: &'static [Form],
    /// Literal acids `H_k X` booked as the molecule that deliver the
    /// reduced anion AND its protons (hydrochloric acid for chloride).
    pub reduced_acids: &'static [&'static str],
    pub reduced_phase: Phase,
    pub reduced_unit_activity: bool,
    pub electrons: f64,
    pub protons: f64,
    pub water: f64,
    pub e0_v: f64,
    pub source: &'static str,
}

const CRC: &str = "CRC Handbook of Chemistry and Physics, Electrochemical Series (P. Vanýsek)";

pub const COUPLES: &[Couple] = &[
    Couple {
        id: "MnO2/Mn+2",
        oxidised: "MnO2",
        oxidised_forms: &[],
        oxidised_phase: Phase::Solid,
        oxidised_unit_activity: true,
        reduced: "Mn+2",
        reduced_per_oxidised: 1.0,
        reduced_forms: &[],
        reduced_acids: &[],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 2.0,
        protons: 4.0,
        water: 2.0,
        e0_v: E0_MNO2,
        source: CRC,
    },
    Couple {
        id: "MnO4-/Mn+2",
        oxidised: "MnO4-",
        oxidised_forms: &[form("KMnO4", &[("K+", 1.0)])],
        oxidised_phase: Phase::Aqueous,
        oxidised_unit_activity: false,
        reduced: "Mn+2",
        reduced_per_oxidised: 1.0,
        reduced_forms: &[],
        reduced_acids: &[],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 5.0,
        protons: 8.0,
        water: 4.0,
        e0_v: E0_MNO4,
        source: CRC,
    },
    Couple {
        id: "Cr2O7-2/Cr+3",
        oxidised: "Cr2O7-2",
        oxidised_forms: &[form("K2Cr2O7", &[("K+", 2.0)])],
        oxidised_phase: Phase::Aqueous,
        oxidised_unit_activity: false,
        reduced: "Cr+3",
        reduced_per_oxidised: 2.0,
        reduced_forms: &[],
        reduced_acids: &[],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 6.0,
        protons: 14.0,
        water: 7.0,
        e0_v: E0_CR2O7,
        source: CRC,
    },
    Couple {
        id: "Cl2/Cl-",
        oxidised: "Cl2",
        oxidised_forms: &[],
        oxidised_phase: Phase::Gas,
        oxidised_unit_activity: true,
        reduced: "Cl-",
        reduced_per_oxidised: 2.0,
        reduced_forms: &[],
        reduced_acids: &["HCl"],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 2.0,
        protons: 0.0,
        water: 0.0,
        e0_v: E0_CL2,
        source: CRC,
    },
    Couple {
        id: "Br2/Br-",
        oxidised: "Br2",
        oxidised_forms: &[],
        oxidised_phase: Phase::Liquid,
        oxidised_unit_activity: true,
        reduced: "Br-",
        reduced_per_oxidised: 2.0,
        reduced_forms: &[],
        reduced_acids: &["HBr"],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 2.0,
        protons: 0.0,
        water: 0.0,
        e0_v: E0_BR2,
        source: CRC,
    },
    Couple {
        id: "I2/I-",
        oxidised: "I2",
        oxidised_forms: &[],
        // Written dissolved, as the chlorine displacement always wrote it;
        // the iodine/triiodide equilibrium takes it from there.
        oxidised_phase: Phase::Aqueous,
        oxidised_unit_activity: true,
        reduced: "I-",
        reduced_per_oxidised: 2.0,
        reduced_forms: &[],
        reduced_acids: &["HI"],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 2.0,
        protons: 0.0,
        water: 0.0,
        e0_v: E0_I2,
        source: CRC,
    },
    Couple {
        id: "Fe+3/Fe+2",
        oxidised: "Fe+3",
        oxidised_forms: &[],
        oxidised_phase: Phase::Aqueous,
        oxidised_unit_activity: false,
        reduced: "Fe+2",
        reduced_per_oxidised: 1.0,
        reduced_forms: &[form("FeSO4", &[("SO4-2", 1.0)])],
        reduced_acids: &[],
        reduced_phase: Phase::Aqueous,
        reduced_unit_activity: false,
        electrons: 1.0,
        protons: 0.0,
        water: 0.0,
        e0_v: E0_FE3,
        source: CRC,
    },
];

/// One reaction: the oxidant couple's oxidised member takes electrons from
/// the reductant couple's reduced member.
#[derive(Debug, Clone, Copy)]
pub struct Route {
    pub oxidant: &'static str,
    pub reductant: &'static str,
    /// Why this is the bench reaction (thermodynamics alone admits nothing).
    pub source: &'static str,
}

const PREP: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed., §17.2.3 (laboratory preparation of the halogens by oxidising the halide in acid)";
const DISPLACE: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed., §17.3 (each halogen displaces the heavier halides from solution)";
const IODOMETRY: &str = "Vogel's Textbook of Quantitative Chemical Analysis, 6th ed., iodometric standardisations (dichromate, permanganate on acidified iodide)";
const DICHROMATOMETRY: &str = "Vogel's Textbook of Quantitative Chemical Analysis, 6th ed., titrations with potassium dichromate (iron(II) in acid)";

/// Every route, in the order they run: halogen preparations, then the
/// chlorine displacements strongest reductant first, then the rest.
pub const ROUTES: &[Route] = &[
    Route {
        oxidant: "MnO2/Mn+2",
        reductant: "Cl2/Cl-",
        source: PREP,
    },
    Route {
        oxidant: "MnO4-/Mn+2",
        reductant: "Cl2/Cl-",
        source: PREP,
    },
    Route {
        oxidant: "Cr2O7-2/Cr+3",
        reductant: "Cl2/Cl-",
        source: PREP,
    },
    Route {
        oxidant: "Cl2/Cl-",
        reductant: "I2/I-",
        source: DISPLACE,
    },
    Route {
        oxidant: "Cl2/Cl-",
        reductant: "Fe+3/Fe+2",
        source: DISPLACE,
    },
    Route {
        oxidant: "Cl2/Cl-",
        reductant: "Br2/Br-",
        source: DISPLACE,
    },
    Route {
        oxidant: "MnO2/Mn+2",
        reductant: "Br2/Br-",
        source: PREP,
    },
    Route {
        oxidant: "MnO4-/Mn+2",
        reductant: "Br2/Br-",
        source: PREP,
    },
    Route {
        oxidant: "Cr2O7-2/Cr+3",
        reductant: "Br2/Br-",
        source: PREP,
    },
    Route {
        oxidant: "MnO2/Mn+2",
        reductant: "I2/I-",
        source: PREP,
    },
    Route {
        oxidant: "MnO4-/Mn+2",
        reductant: "I2/I-",
        source: IODOMETRY,
    },
    Route {
        oxidant: "Cr2O7-2/Cr+3",
        reductant: "I2/I-",
        source: IODOMETRY,
    },
    Route {
        oxidant: "Br2/Br-",
        reductant: "I2/I-",
        source: DISPLACE,
    },
    Route {
        oxidant: "Br2/Br-",
        reductant: "Fe+3/Fe+2",
        source: DISPLACE,
    },
    Route {
        oxidant: "Cr2O7-2/Cr+3",
        reductant: "Fe+3/Fe+2",
        source: DICHROMATOMETRY,
    },
];

pub fn couple(id: &str) -> Option<&'static Couple> {
    COUPLES.iter().find(|c| c.id == id)
}

fn pair(route: &Route) -> (&'static Couple, &'static Couple) {
    (
        couple(route.oxidant).expect("route oxidant couple"),
        couple(route.reductant).expect("route reductant couple"),
    )
}

/// Which routes a caller runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Routes that make chlorine (the oxidant + acidified chloride preparations).
    MakesChlorine,
    /// Routes in which chlorine is the oxidant.
    ChlorineOxidant,
    /// Every other route.
    Rest,
}

const CHLORINE: &str = "Cl2/Cl-";

fn in_scope(route: &Route, scope: Scope) -> bool {
    match scope {
        Scope::MakesChlorine => route.reductant == CHLORINE,
        Scope::ChlorineOxidant => route.oxidant == CHLORINE,
        Scope::Rest => route.reductant != CHLORINE && route.oxidant != CHLORINE,
    }
}

/// Whether a route prices this (oxidant member, reductant member) pair, so
/// the halogen writer table's honesty tail does not call it unpriced. A
/// dormant route counts: it names its own missing product.
pub fn covers(oxidant: &str, reductant: &str) -> bool {
    ROUTES.iter().any(|r| {
        let (o, d) = pair(r);
        o.oxidised == oxidant && d.reduced == reductant
    })
}

fn in_registry(key: &str) -> bool {
    crate::species::lookup_key(key).is_some()
}

// ── inventory ────────────────────────────────────────────────────────

const PHASES: &[Phase] = &[Phase::Aqueous, Phase::Solid, Phase::Liquid];

fn amount(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    let id = SpeciesId::new(key);
    let id = id.canonical();
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == id.as_ref() && phases.contains(&p.phase))
        .map(|p| p.moles.0)
        .sum()
}

fn take(vessel: &mut Vessel, key: &str, n: f64, phases: &[Phase]) -> f64 {
    let mut left = n;
    for phase in phases {
        if left <= 0.0 {
            break;
        }
        left -= vessel
            .withdraw_phase(&SpeciesId::new(key), Moles(left), *phase)
            .0;
    }
    n - left.max(0.0)
}

fn liquid_water(vessel: &Vessel) -> f64 {
    amount(vessel, "water", &[Phase::Liquid])
}

fn kgw(vessel: &Vessel) -> f64 {
    liquid_water(vessel) * crate::constants::WATER_MOLAR_MASS_KG_PER_MOL
}

/// Every form of a member with the spectators it releases per unit: the
/// member itself, the couple's explicit forms, and (for an anion) every
/// Na/K salt of it in the registry.
fn forms_of(member: &str, explicit: &[Form]) -> Vec<(String, Vec<(&'static str, f64)>)> {
    let mut out: Vec<(String, Vec<(&'static str, f64)>)> = Vec::new();
    for f in crate::halogen_redox::registry_forms(member) {
        let spect = crate::halogen_redox::spectators_of(&f, member);
        out.push((f, spect));
    }
    for f in explicit {
        if !out.iter().any(|(k, _)| k == f.key) {
            out.push((f.key.to_string(), f.spectators.to_vec()));
        }
    }
    out
}

fn is_chlorine(key: &str) -> bool {
    key == "Cl2"
}

fn oxidant_available(vessel: &Vessel, c: &Couple) -> f64 {
    if is_chlorine(c.oxidised) {
        return crate::chlorine::chlorine_pool(vessel);
    }
    forms_of(c.oxidised, c.oxidised_forms)
        .iter()
        .map(|(f, _)| amount(vessel, f, PHASES))
        .sum()
}

fn take_oxidant(vessel: &mut Vessel, c: &Couple, n: f64) {
    if is_chlorine(c.oxidised) {
        crate::chlorine::take_chlorine(vessel, n);
        return;
    }
    let mut left = n;
    for (f, spect) in forms_of(c.oxidised, c.oxidised_forms) {
        if left <= TRACE {
            break;
        }
        let got = take(vessel, &f, left, PHASES);
        for (ion, k) in spect {
            vessel.deposit(SpeciesId::new(ion), Moles(got * k), Phase::Aqueous);
        }
        left -= got;
    }
}

fn literal_acid(vessel: &Vessel, c: &Couple) -> f64 {
    c.reduced_acids
        .iter()
        .map(|a| amount(vessel, a, &[Phase::Aqueous, Phase::Liquid]))
        .sum()
}

fn reductant_available(vessel: &Vessel, c: &Couple) -> f64 {
    forms_of(c.reduced, c.reduced_forms)
        .iter()
        .map(|(f, _)| amount(vessel, f, &[Phase::Aqueous, Phase::Solid]))
        .sum::<f64>()
        + literal_acid(vessel, c)
}

/// Book a literal `HX` as the ions a solve would, so the route can spend its
/// anion and its proton separately.
fn ionise_literal_acids(vessel: &mut Vessel, c: &Couple) {
    for acid in c.reduced_acids {
        let n = take(
            vessel,
            acid,
            amount(vessel, acid, &[Phase::Aqueous, Phase::Liquid]),
            &[Phase::Aqueous, Phase::Liquid],
        );
        if n > 0.0 {
            vessel.deposit(SpeciesId::new("H+"), Moles(n), Phase::Aqueous);
            vessel.deposit(SpeciesId::new(c.reduced), Moles(n), Phase::Aqueous);
            refresh_charge(vessel);
        }
    }
}

fn take_reductant(vessel: &mut Vessel, c: &Couple, n: f64) {
    let mut left = n;
    for (f, spect) in forms_of(c.reduced, c.reduced_forms) {
        if left <= TRACE {
            break;
        }
        let got = take(vessel, &f, left, &[Phase::Aqueous, Phase::Solid]);
        for (ion, k) in spect {
            vessel.deposit(SpeciesId::new(ion), Moles(got * k), Phase::Aqueous);
        }
        left -= got;
    }
}

fn refresh_charge(vessel: &mut Vessel) {
    vessel.solute_charge = crate::displacement::solute_charge(vessel);
}

fn acid_available(vessel: &Vessel) -> f64 {
    crate::displacement::unspent_acidity(vessel)
        + crate::strong_reserve::certified_literal_acid_protons(vessel)
}

fn spend_acid(vessel: &mut Vessel, n: f64) {
    refresh_charge(vessel);
    let ledger = crate::displacement::unspent_acidity(vessel).min(n);
    crate::displacement::consume_acidity(vessel, ledger);
    let rest = (n - ledger).max(0.0);
    if rest > TRACE {
        crate::strong_reserve::consume_literal_acid(vessel, rest);
    }
}

// ── stoichiometry ────────────────────────────────────────────────────

/// Per mole of oxidant reduced: (reductant units oxidised, reductant
/// consumed, net protons consumed, net water produced).
fn per_oxidant(ox: &Couple, red: &Couple) -> (f64, f64, f64, f64) {
    let k = ox.electrons / red.electrons;
    (
        k,
        k * red.reduced_per_oxidised,
        ox.protons - k * red.protons,
        ox.water - k * red.water,
    )
}

fn lcm(a: f64, b: f64) -> f64 {
    let (a, b) = (a.round() as u64, b.round() as u64);
    let (mut x, mut y) = (a, b);
    while y != 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    (a / x * b) as f64
}

fn term(coef: f64, key: &str, gas: bool) -> String {
    let name = crate::halogen_redox::pretty(key);
    let arrow = if gas { "↑" } else { "" };
    if (coef - 1.0).abs() < 1e-9 {
        format!("{name}{arrow}")
    } else {
        format!("{} {name}{arrow}", coef.round() as i64)
    }
}

fn charge_of(key: &str) -> f64 {
    crate::stoich::parse_formula(key)
        .map(|f| f.charge)
        .unwrap_or(0.0)
}

/// The smallest whole-number equation: oxidant, consumed protons, reductant
/// on the left; cations, neutral products, anions, then water on the right.
pub fn equation(route: &Route) -> String {
    let (ox, red) = pair(route);
    let l = lcm(ox.electrons, red.electrons);
    let scale = (1..=8)
        .map(f64::from)
        .find(|s| {
            let n = s * l / red.electrons * red.reduced_per_oxidised;
            let m = s * l / ox.electrons * ox.reduced_per_oxidised;
            (n - n.round()).abs() < 1e-9 && (m - m.round()).abs() < 1e-9
        })
        .unwrap_or(1.0);
    let a = scale * l / ox.electrons;
    let (k, red_n, h, w) = per_oxidant(ox, red);
    let mut left = vec![term(a, ox.oxidised, false)];
    if h > 0.0 {
        left.push(term(a * h, "H+", false));
    }
    left.push(term(a * red_n, red.reduced, false));
    if w < 0.0 {
        left.push(term(-a * w, "water", false));
    }
    let mut products = [
        (
            ox.reduced,
            a * ox.reduced_per_oxidised,
            ox.reduced_phase == Phase::Gas,
        ),
        (red.oxidised, a * k, red.oxidised_phase == Phase::Gas),
    ];
    products.sort_by(|x, y| charge_of(y.0).total_cmp(&charge_of(x.0)));
    let mut right: Vec<String> = products
        .iter()
        .map(|(key, n, gas)| term(*n, key, *gas))
        .collect();
    if w > 0.0 {
        right.push(term(a * w, "water", false));
    }
    if h < 0.0 {
        right.push(term(-a * h, "H+", false));
    }
    format!("{} → {}", left.join(" + "), right.join(" + "))
}

// ── the potential crossing ───────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
struct Start {
    ox: f64,
    ox_reduced: f64,
    red: f64,
    red_oxidised: f64,
    protons: f64,
}

fn start(vessel: &Vessel, ox: &Couple, red: &Couple) -> Start {
    Start {
        ox: oxidant_available(vessel, ox),
        ox_reduced: amount(vessel, ox.reduced, &[Phase::Aqueous]),
        red: reductant_available(vessel, red),
        red_oxidised: if is_chlorine(red.oxidised) {
            crate::chlorine::chlorine_pool(vessel)
        } else {
            amount(vessel, red.oxidised, PHASES)
        },
        protons: acid_available(vessel) - crate::displacement::room_air_acidity_mol(vessel),
    }
}

fn molal(n: f64, w: f64) -> f64 {
    (n / w).max(1e-300)
}

/// Oxidant-couple potential minus reductant-couple potential (V) after
/// extent `x` (mol oxidant reduced).
fn delta_e(vessel: &Vessel, ox: &Couple, red: &Couple, x: f64, s0: Start) -> f64 {
    let w = kgw(vessel);
    let slope = crate::relations::nernst_slope(vessel.temperature);
    let (k, red_n, h_net, _) = per_oxidant(ox, red);
    let h = molal(s0.protons - h_net * x, w);
    let ox_m = if ox.oxidised_unit_activity {
        1.0
    } else {
        molal(s0.ox - x, w)
    };
    let oxr_m = if ox.reduced_unit_activity {
        1.0
    } else {
        molal(s0.ox_reduced + ox.reduced_per_oxidised * x, w)
    };
    let red_m = if red.reduced_unit_activity {
        1.0
    } else {
        molal(s0.red - red_n * x, w)
    };
    let redo_m = if red.oxidised_unit_activity {
        1.0
    } else {
        molal(s0.red_oxidised + k * x, w)
    };
    let e_ox = ox.e0_v
        - slope / ox.electrons
            * (oxr_m.powf(ox.reduced_per_oxidised) / (ox_m * h.powf(ox.protons))).log10();
    let e_red = red.e0_v
        - slope / red.electrons
            * (red_m.powf(red.reduced_per_oxidised) / (redo_m * h.powf(red.protons))).log10();
    e_ox - e_red
}

/// Extent (mol oxidant reduced) to the potential crossing or the limiting
/// reagent, whichever is first; zero when the route does not apply.
pub fn extent(vessel: &Vessel, route: &Route) -> f64 {
    let (ox, red) = pair(route);
    if liquid_water(vessel) <= 0.0 {
        return 0.0;
    }
    if !in_registry(ox.reduced) || !in_registry(red.oxidised) {
        return 0.0;
    }
    let s0 = start(vessel, ox, red);
    let (_, red_n, h_net, _) = per_oxidant(ox, red);
    if s0.ox <= TRACE || s0.red <= TRACE {
        return 0.0;
    }
    let mut xmax = s0.ox.min(s0.red / red_n);
    if h_net > 0.0 {
        if s0.protons <= crate::OBSERVABLE_MOLES {
            return 0.0;
        }
        xmax = xmax.min(s0.protons / h_net);
    }
    if delta_e(vessel, ox, red, 0.0, s0) <= 0.0 {
        return 0.0;
    }
    // ΔE falls monotonically with extent.
    if delta_e(vessel, ox, red, xmax * (1.0 - RESOLUTION), s0) >= 0.0 {
        return xmax;
    }
    let (mut lo, mut hi) = (0.0, xmax);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if delta_e(vessel, ox, red, mid, s0) >= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Smallest extent worth booking: an acid-driven crossing near zero
/// (dilute acid) is booked as nothing; a proton-neutral displacement runs
/// on whatever is there.
fn floor(route: &Route) -> f64 {
    let (ox, red) = pair(route);
    if per_oxidant(ox, red).2 > 0.0 {
        crate::OBSERVABLE_MOLES
    } else {
        TRACE
    }
}

fn worth(route: &Route, x: f64) -> bool {
    let (ox, red) = pair(route);
    x > TRACE && x * per_oxidant(ox, red).0 > floor(route)
}

// ── notes and events ─────────────────────────────────────────────────

fn note(events: &mut Vec<Event>, vessel: &Vessel, cause: NotModelledCause, phrase: Phrase) {
    let dup = events.iter().any(|e| match e {
        Event::NotYetModeled {
            reason: Some(r), ..
        } => r.key == phrase.key,
        _ => false,
    });
    if !dup {
        events.push(Event::not_modeled(vessel.id, cause, phrase));
    }
}

fn pretty(key: &str) -> String {
    crate::halogen_redox::pretty(key)
}

fn unregistered_note(events: &mut Vec<Event>, vessel: &Vessel, route: &Route, product: &str) {
    let (ox, red) = pair(route);
    let phrase = if is_chlorine(ox.oxidised) {
        Phrase::new(
            "not-modeled.chlorine-product-unregistered",
            "chlorine would oxidise {reductant} here, but the registry has no {product} to write, so it is left unreacted. That is a gap in the model, not evidence that nothing happens",
            vec![
                ("reductant".to_string(), Slot::text(pretty(red.reduced))),
                ("product".to_string(), Slot::text(pretty(product))),
            ],
        )
    } else {
        Phrase::new(
            "not-modeled.potential-redox-product-unregistered",
            "{oxidant} would oxidise {reductant} here (its standard potential is the higher), but the registry has no {product} to write, so they are left unreacted. That is a gap in the model, not evidence that nothing happens",
            vec![
                ("oxidant".to_string(), Slot::text(pretty(ox.oxidised))),
                ("reductant".to_string(), Slot::text(pretty(red.reduced))),
                ("product".to_string(), Slot::text(pretty(product))),
            ],
        )
    };
    note(events, vessel, NotModelledCause::PhaseNotInRegistry, phrase);
}

fn ideal_note(events: &mut Vec<Event>, vessel: &Vessel, route: &Route) {
    let (ox, red) = pair(route);
    if is_chlorine(ox.oxidised) {
        // The chlorine displacements run to completion; the hydrolysis
        // note says what the chlorine pool is.
        return;
    }
    let phrase = if is_chlorine(red.oxidised) {
        Phrase::new(
            "not-modeled.chlorine-preparation-ideal",
            "{oxidant} made chlorine from chloride in acid until the two electrode potentials met (CRC standard potentials, molality standing in for activity, chlorine at one atmosphere); activity coefficients in concentrated acid, the rate and the heat of this reaction are not modelled",
            vec![("oxidant".to_string(), Slot::text(pretty(ox.oxidised)))],
        )
    } else {
        Phrase::new(
            "not-modeled.potential-redox-ideal",
            "{oxidant} oxidised {reductant} until the two electrode potentials met or a reagent ran out (CRC standard potentials, molality standing in for activity); activity coefficients, the rate and the heat of this reaction are not modelled",
            vec![
                ("oxidant".to_string(), Slot::text(pretty(ox.oxidised))),
                ("reductant".to_string(), Slot::text(pretty(red.reduced))),
            ],
        )
    };
    note(events, vessel, NotModelledCause::RateNotModelled, phrase);
}

fn emit_product(vessel: &mut Vessel, key: &str, n: f64, phase: Phase, events: &mut Vec<Event>) {
    if n <= 0.0 {
        return;
    }
    if is_chlorine(key) {
        crate::chlorine::emit_chlorine(vessel, n, events);
        return;
    }
    if phase == Phase::Gas {
        let species = SpeciesId::new(key);
        if vessel.retain_gas(species.clone(), Moles(n)) {
            events.push(Event::GasContained {
                vessel: vessel.id,
                species,
                moles: Moles(n),
            });
        } else {
            events.push(Event::GasEvolved {
                vessel: vessel.id,
                species,
                moles: Moles(n),
            });
        }
        return;
    }
    vessel.deposit(SpeciesId::new(key), Moles(n), phase);
}

/// The product a dormant route lacks, if its reagents are both here.
fn missing_product(vessel: &Vessel, route: &Route) -> Option<&'static str> {
    let (ox, red) = pair(route);
    if liquid_water(vessel) <= 0.0 {
        return None;
    }
    if oxidant_available(vessel, ox) <= TRACE || reductant_available(vessel, red) <= TRACE {
        return None;
    }
    if per_oxidant(ox, red).2 > 0.0
        && acid_available(vessel) - crate::displacement::room_air_acidity_mol(vessel)
            <= crate::OBSERVABLE_MOLES
    {
        return None;
    }
    [ox.reduced, red.oxidised]
        .into_iter()
        .find(|k| !in_registry(k))
}

fn run_route(vessel: &mut Vessel, route: &Route, events: &mut Vec<Event>) -> bool {
    if let Some(product) = missing_product(vessel, route) {
        // Chlorine's own displacements say it in the step (as they always
        // have); every other dormant route is said by the honesty tail
        // through `dormant_notes`.
        if in_scope(route, Scope::ChlorineOxidant) {
            unregistered_note(events, vessel, route, product);
        }
        return false;
    }
    let x = extent(vessel, route);
    if !worth(route, x) {
        return false;
    }
    let (ox, red) = pair(route);
    ionise_literal_acids(vessel, red);
    let x = x.min(extent(vessel, route));
    if x <= TRACE {
        return false;
    }
    let (k, red_n, h_net, w_net) = per_oxidant(ox, red);
    if h_net > 0.0 {
        spend_acid(vessel, x * h_net);
    }
    take_oxidant(vessel, ox, x);
    take_reductant(vessel, red, x * red_n);
    if w_net < 0.0 {
        take(vessel, "water", -x * w_net, &[Phase::Liquid]);
    }
    events.push(Event::ReactionOccurred {
        vessel: vessel.id,
        equation: equation(route),
    });
    vessel.deposit(
        SpeciesId::new(ox.reduced),
        Moles(x * ox.reduced_per_oxidised),
        ox.reduced_phase,
    );
    if w_net > 0.0 {
        vessel.deposit(SpeciesId::new("water"), Moles(x * w_net), Phase::Liquid);
    }
    if h_net < 0.0 {
        vessel.deposit(SpeciesId::new("H+"), Moles(-x * h_net), Phase::Aqueous);
    }
    emit_product(vessel, red.oxidised, x * k, red.oxidised_phase, events);
    vessel.refresh_pressure();
    refresh_charge(vessel);
    ideal_note(events, vessel, route);
    true
}

/// Typed notes for dormant routes whose reagents are both here (and, for
/// an acid-driven route, acid too) but whose product the registry lacks.
/// Read by the honesty tail every step (see `halogen_redox::unpriced_notes`).
pub fn dormant_notes(vessel: &Vessel) -> Vec<Event> {
    let mut events = Vec::new();
    for route in ROUTES
        .iter()
        .filter(|r| !in_scope(r, Scope::ChlorineOxidant))
    {
        if let Some(product) = missing_product(vessel, route) {
            unregistered_note(&mut events, vessel, route, product);
        }
    }
    events
}

/// Whether any route in `scope` would react in this vessel.
pub fn fires_in(vessel: &Vessel, scope: Scope) -> bool {
    ROUTES
        .iter()
        .filter(|r| in_scope(r, scope))
        .any(|r| worth(r, extent(vessel, r)))
}

/// Run every route in `scope` once, in table order. Returns whether
/// anything reacted; the caller refreshes `solute_charge`.
pub fn apply_in(vessel: &mut Vessel, events: &mut Vec<Event>, scope: Scope) -> bool {
    let mut progressed = false;
    for route in ROUTES.iter().filter(|r| in_scope(r, scope)) {
        progressed |= run_route(vessel, route, events);
    }
    progressed
}

/// The routes chlorine does not own (`chlorine::apply` runs those).
pub fn fires(vessel: &Vessel) -> bool {
    fires_in(vessel, Scope::Rest)
}

pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    apply_in(vessel, events, Scope::Rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(o: &str, r: &str) -> &'static Route {
        ROUTES
            .iter()
            .find(|x| x.oxidant == o && x.reductant == r)
            .unwrap()
    }

    fn formula(key: &str) -> crate::stoich::Formula {
        let text = if key == "water" {
            "H2O"
        } else {
            crate::species::lookup_key(key)
                .map(|d| d.formula)
                .unwrap_or(key)
        };
        crate::stoich::parse_formula(text).unwrap()
    }

    #[test]
    fn every_route_closes_atoms_and_charge() {
        for r in ROUTES {
            let (ox, red) = pair(r);
            let (k, red_n, h, w) = per_oxidant(ox, red);
            let mut el: std::collections::BTreeMap<String, f64> = Default::default();
            let mut q = 0.0;
            let mut add = |key: &str, n: f64| {
                let f = formula(key);
                for (e, c) in f.counts {
                    *el.entry(e).or_default() += n * c;
                }
                q += n * f.charge;
            };
            add(ox.oxidised, -1.0);
            add(red.reduced, -red_n);
            add("H+", -h);
            add(ox.reduced, ox.reduced_per_oxidised);
            add(red.oxidised, k);
            add("water", w);
            assert!(q.abs() < 1e-9, "{}: charge {q}", equation(r));
            assert!(
                el.values().all(|v| v.abs() < 1e-9),
                "{}: {el:?}",
                equation(r)
            );
            // Not every route is downhill at standard state: MnO₂ and
            // dichromate make chlorine only from concentrated acid, which is
            // what the Nernst crossing decides.
        }
    }

    #[test]
    fn equations_read_like_the_textbook() {
        assert_eq!(
            equation(route("MnO2/Mn+2", "Cl2/Cl-")),
            "MnO₂ + 4 H⁺ + 2 Cl⁻ → Mn²⁺ + Cl₂↑ + 2 H₂O"
        );
        assert_eq!(
            equation(route("MnO4-/Mn+2", "Cl2/Cl-")),
            "2 MnO₄⁻ + 16 H⁺ + 10 Cl⁻ → 2 Mn²⁺ + 5 Cl₂↑ + 8 H₂O"
        );
        assert_eq!(
            equation(route("Cl2/Cl-", "Fe+3/Fe+2")),
            "Cl₂ + 2 Fe²⁺ → 2 Fe³⁺ + 2 Cl⁻"
        );
        assert_eq!(
            equation(route("Cl2/Cl-", "I2/I-")),
            "Cl₂ + 2 I⁻ → I₂ + 2 Cl⁻"
        );
        assert_eq!(
            equation(route("Cl2/Cl-", "Br2/Br-")),
            "Cl₂ + 2 Br⁻ → Br₂ + 2 Cl⁻"
        );
        assert_eq!(
            equation(route("Cr2O7-2/Cr+3", "Fe+3/Fe+2")),
            "Cr₂O₇²⁻ + 14 H⁺ + 6 Fe²⁺ → 2 Cr³⁺ + 6 Fe³⁺ + 7 H₂O"
        );
        assert_eq!(
            equation(route("MnO4-/Mn+2", "I2/I-")),
            "2 MnO₄⁻ + 16 H⁺ + 10 I⁻ → 2 Mn²⁺ + 5 I₂ + 8 H₂O"
        );
    }

    #[test]
    fn covered_pairs_are_the_table_rows() {
        assert!(covers("Cl2", "I-"));
        assert!(covers("MnO4-", "I-"));
        assert!(covers("Cr2O7-2", "Fe+2"));
        assert!(!covers("MnO4-", "Fe+2"), "the aqueous engine owns it");
        assert!(!covers("I2", "Fe+2"));
        assert!(!covers("I2", "Cl-"));
    }
}
