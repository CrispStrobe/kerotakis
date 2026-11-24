//! Chlorine as a reagent: made, dissolved, hydrolysed, and spent.
//!
//! C1 audit class `C1A-chlorine`. Before this module the bench could hold
//! chlorine (`Cl2` as a gas portion) and could make it from hypochlorite and
//! acid (the curated bleach rows), but nothing else ever touched it: it never
//! formed from the classic oxidant + hydrochloric-acid preparations, never
//! dissolved, never bleached, never oxidised iron(II) or iodide, and never
//! burned a metal. Every rule below is a reaction CLASS, written once and
//! applied to every registry species that fits it.
//!
//! The rules, in the order [`apply`] runs them:
//!
//! 1. **Preparation from an oxidant and chloride in acid**
//!    (`MnO₂ + 4 H⁺ + 2 Cl⁻ → Mn²⁺ + Cl₂ + 2 H₂O`,
//!    `2 MnO₄⁻ + 16 H⁺ + 10 Cl⁻ → 2 Mn²⁺ + 5 Cl₂ + 8 H₂O`). Run until the
//!    ideal-solution Nernst potentials of the two couples meet, so dilute
//!    acid stays inert and concentrated acid evolves chlorine, which is the
//!    whole teaching point of "warm the manganese dioxide with CONCENTRATED
//!    hydrochloric acid". Standard potentials: CRC Handbook of Chemistry and
//!    Physics, "Electrochemical Series" (P. Vanýsek): MnO₂/Mn²⁺ 1.224 V,
//!    MnO₄⁻/Mn²⁺ 1.507 V, Cl₂(g)/Cl⁻ 1.35827 V. Activity coefficients are
//!    not modelled (molality stands in for activity), and no rate is
//!    claimed: both are said in a typed note every time the rule runs.
//! 2. **Absorption in alkali** (`Cl₂ + 2 OH⁻ → ClO⁻ + Cl⁻ + H₂O`) against the
//!    vessel's base: base equivalents, then NaOH, KOH and Ca(OH)₂ in any
//!    condensed phase (bleaching powder / bleach liquor).
//! 3. **Halide displacement and iron(II)** (`Cl₂ + 2 I⁻ → I₂ + 2 Cl⁻`,
//!    `Cl₂ + 2 Fe²⁺ → 2 Fe³⁺ + 2 Cl⁻`, `Cl₂ + 2 Br⁻ → Br₂ + 2 Cl⁻` when the
//!    registry carries Br₂), strongest reductant first.
//! 4. **Bleaching of pigments** by HClO, Cl₂ and ClO⁻ to the registry's
//!    oxidised (colourless) form, as the curated hypochlorite rows already
//!    write it (one O added). Qualitative: no reviewed rate.
//! 5. **Chlorine water**: Henry partition and hydrolysis
//!    `Cl₂(aq) + H₂O ⇌ HClO + H⁺ + Cl⁻`, solved to its constant.
//! 6. **Dry chlorination of a hot or ignited metal or hydrogen**
//!    (`2 Fe + 3 Cl₂ → 2 FeCl₃`, `Cu + Cl₂ → CuCl₂`, `H₂ + Cl₂ → 2 HCl`),
//!    complete, heat unpriced (typed note), and hydrogen chloride gas taken
//!    up by liquid water.
//!
//! ## Data
//!
//! Hydrolysis constant and its temperature dependence are DERIVED from the
//! NBS tables (D. D. Wagman et al., J. Phys. Chem. Ref. Data 11 (1982)
//! Suppl. 2, Table 10, chlorine, 298.15 K): ΔfG°/ΔfH° kJ/mol of
//! Cl₂(ao) 6.94/−23.4, HClO(ao) −79.9/−120.9, Cl⁻(ai) −131.228/−167.159,
//! H₂O(l) −237.129/−285.830. ΔrG° = 19.061 kJ/mol gives
//! K = 4.58·10⁻⁴ mol² kg⁻², ΔrH° = +21.171 kJ/mol. Henry's constant is the
//! bench's existing Cl₂ row (Sander 2015, 0.092 mol L⁻¹ atm⁻¹, 2500 K).
//!
//! ## Representation
//!
//! The registry has no aqueous molecular chlorine the aqueous engines can
//! speciate, so molecular Cl₂ — in the headspace AND dissolved — is held as
//! one `Cl2` gas-phase pool, and the partition is applied to it in the
//! hydrolysis solve: only the dissolved share hydrolyses, and the pool
//! reacts with dissolved reductants and pigments. HClO and Cl⁻ are written
//! as ordinary solutes for the aqueous engine to speciate. A displayed
//! headspace pressure therefore counts dissolved Cl₂ as gas; that is stated,
//! not hidden. An open vessel keeps only what its liquid can hold at one
//! atmosphere of chlorine; the rest escapes.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::{Headspace, Vessel};
use crate::SpeciesId;

const TRACE: f64 = 1e-12;

/// Aqueous domain (K) for the solution rules: the hydrolysis constant is
/// continued with a constant reaction enthalpy from 25 °C, which is a
/// bench-range continuation, not a high-temperature claim.
pub const AQUEOUS_DOMAIN_K: (f64, f64) = (273.15, 373.15);

/// Editorial threshold (Kerotakis), K: a metal or hydrogen held at or above
/// this temperature in chlorine, or ignited, is treated as burning in it to
/// completion. Not an ignition temperature of any particular metal: the
/// bench has none reviewed, and the typed note says so every time.
pub const DRY_CHLORINATION_K: f64 = 573.15;

/// CRC "Electrochemical Series" standard potentials, V.
pub const E0_CL2: f64 = 1.358_27;
pub const E0_MNO2: f64 = 1.224;
pub const E0_MNO4: f64 = 1.507;

/// NBS 1982 (Wagman et al.) formation data, kJ/mol: (ΔfG°, ΔfH°).
pub const NBS_CL2_AQ: (f64, f64) = (6.94, -23.4);
pub const NBS_HCLO_AQ: (f64, f64) = (-79.9, -120.9);
pub const NBS_CL_ION: (f64, f64) = (-131.228, -167.159);
pub const NBS_WATER_L: (f64, f64) = (-237.129, -285.830);

/// HCl uptake ceiling, mol per kg water: 72 g HCl per 100 g water at 20 °C
/// and 1 atm (CRC Handbook, aqueous solubility of hydrogen chloride).
pub const HCL_UPTAKE_MOL_PER_KG: f64 = 720.0 / 36.461;

const R_L_ATM: f64 = 0.082_057_366;
const CHLORINE: &str = "Cl2";
const CHLORIDE: &str = "Cl-";

/// Hydrolysis reaction (ΔrG°, ΔrH°), kJ/mol, from the NBS rows.
pub fn hydrolysis_reaction_kj() -> (f64, f64) {
    let g = NBS_HCLO_AQ.0 + NBS_CL_ION.0 - NBS_CL2_AQ.0 - NBS_WATER_L.0;
    let h = NBS_HCLO_AQ.1 + NBS_CL_ION.1 - NBS_CL2_AQ.1 - NBS_WATER_L.1;
    (g, h)
}

/// `[HClO][H⁺][Cl⁻]/[Cl₂(aq)]`, mol² kg⁻², at `t` K (van 't Hoff with the
/// 25 °C reaction enthalpy).
pub fn hydrolysis_k(t: f64) -> f64 {
    let r = crate::constants::GAS_CONSTANT;
    let (g, h) = hydrolysis_reaction_kj();
    let k298 = (-g * 1e3 / (r * 298.15)).exp();
    k298 * (-h * 1e3 / r * (1.0 / t - 1.0 / 298.15)).exp()
}

fn henry(t: f64) -> f64 {
    crate::properties::henry_lookup(CHLORINE)
        .map(|c| crate::properties::henry_at_t(c, t).value)
        .unwrap_or(0.0)
}

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

const CONDENSED: &[Phase] = &[Phase::Aqueous, Phase::Solid, Phase::Liquid];
const DISSOLVED_THEN_SOLID: &[Phase] = &[Phase::Aqueous, Phase::Solid];

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

fn aqueous_ok(vessel: &Vessel) -> bool {
    liquid_water(vessel) > 0.0
        && (AQUEOUS_DOMAIN_K.0..=AQUEOUS_DOMAIN_K.1).contains(&vessel.temperature.0)
}

/// Molecular chlorine pool (headspace + dissolved; see module notes).
pub fn chlorine_pool(vessel: &Vessel) -> f64 {
    amount(vessel, CHLORINE, &[Phase::Gas, Phase::Aqueous])
}

fn take_chlorine(vessel: &mut Vessel, n: f64) -> f64 {
    take(vessel, CHLORINE, n, &[Phase::Aqueous, Phase::Gas])
}

fn in_registry(key: &str) -> bool {
    crate::species::lookup_key(key).is_some()
}

fn refresh_charge(vessel: &mut Vessel) {
    vessel.solute_charge = crate::displacement::solute_charge(vessel);
}

fn free_acid(vessel: &Vessel) -> f64 {
    (-crate::displacement::solute_charge(vessel)).max(0.0)
}

/// Reaction notes, said once per step (the curated loop calls [`apply`]
/// up to eight times).
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

fn reaction(events: &mut Vec<Event>, vessel: &Vessel, equation: &str) {
    events.push(Event::ReactionOccurred {
        vessel: vessel.id,
        equation: equation.to_string(),
    });
}

/// Write chlorine made by a reaction: an owned headspace keeps it; an
/// open vessel keeps what its liquid holds at one atmosphere and the rest
/// escapes.
fn emit_chlorine(vessel: &mut Vessel, n: f64, events: &mut Vec<Event>) {
    if n <= 0.0 {
        return;
    }
    let keep = match vessel.headspace {
        Headspace::Sealed { .. } | Headspace::PressureControlled { .. } => n,
        Headspace::Open => {
            let cap = henry(vessel.temperature.0) * vessel.liquid_volume().0;
            (cap - chlorine_pool(vessel)).clamp(0.0, n)
        }
        Headspace::Swept { .. } => 0.0,
    };
    if keep > 0.0 {
        vessel.deposit(SpeciesId::new(CHLORINE), Moles(keep), Phase::Gas);
    }
    if n - keep > TRACE {
        events.push(Event::GasEvolved {
            vessel: vessel.id,
            species: SpeciesId::new(CHLORINE),
            moles: Moles(n - keep),
        });
    }
    vessel.refresh_pressure();
}

// ── 1. preparation ───────────────────────────────────────────────────

/// One oxidant that turns acidified chloride into chlorine.
#[derive(Debug, Clone, Copy)]
pub struct ChlorineOxidant {
    /// The oxidising ion or solid as the Nernst term counts it.
    pub key: &'static str,
    /// Registry forms that deliver it, with the spectator cation each
    /// leaves (form, cation, cation per oxidant).
    pub forms: &'static [(&'static str, Option<&'static str>)],
    /// Mn²⁺ per oxidant, H⁺ per oxidant, Cl⁻ per oxidant, H₂O per oxidant,
    /// Cl₂ per oxidant, electrons per oxidant.
    pub mn: f64,
    pub protons: f64,
    pub chloride: f64,
    pub water: f64,
    pub chlorine: f64,
    pub electrons: f64,
    pub e0_v: f64,
    /// Whether the oxidant is a solid (unit activity) or dissolved.
    pub solid: bool,
    pub equation: &'static str,
}

pub const OXIDANTS: &[ChlorineOxidant] = &[
    ChlorineOxidant {
        key: "MnO2",
        forms: &[("MnO2", None)],
        mn: 1.0,
        protons: 4.0,
        chloride: 2.0,
        water: 2.0,
        chlorine: 1.0,
        electrons: 2.0,
        e0_v: E0_MNO2,
        solid: true,
        equation: "MnO₂ + 4 H⁺ + 2 Cl⁻ → Mn²⁺ + Cl₂↑ + 2 H₂O",
    },
    ChlorineOxidant {
        key: "MnO4-",
        forms: &[("MnO4-", None), ("KMnO4", Some("K+"))],
        mn: 1.0,
        protons: 8.0,
        chloride: 5.0,
        water: 4.0,
        chlorine: 2.5,
        electrons: 5.0,
        e0_v: E0_MNO4,
        solid: false,
        equation: "2 MnO₄⁻ + 16 H⁺ + 10 Cl⁻ → 2 Mn²⁺ + 5 Cl₂↑ + 8 H₂O",
    },
];

fn chloride_forms() -> Vec<String> {
    crate::halogen_redox::registry_forms(CHLORIDE)
}

fn chloride_available(vessel: &Vessel) -> f64 {
    chloride_forms()
        .iter()
        .map(|f| amount(vessel, f, DISSOLVED_THEN_SOLID))
        .sum()
}

fn take_ion(vessel: &mut Vessel, ion: &str, forms: &[String], n: f64) -> f64 {
    let mut left = n;
    for form in forms {
        if left <= TRACE {
            break;
        }
        let got = take(vessel, form, left, DISSOLVED_THEN_SOLID);
        for (cation, k) in crate::halogen_redox::spectators_of(form, ion) {
            vessel.deposit(SpeciesId::new(cation), Moles(got * k), Phase::Aqueous);
        }
        left -= got;
    }
    n - left.max(0.0)
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

fn oxidant_available(vessel: &Vessel, ox: &ChlorineOxidant) -> f64 {
    ox.forms
        .iter()
        .map(|(f, _)| amount(vessel, f, DISSOLVED_THEN_SOLID))
        .sum()
}

/// ΔE (V) between the oxidant couple and Cl₂/Cl⁻ after extent `x` (mol
/// oxidant reduced), ideal molality as activity, Cl₂ at one atmosphere.
fn delta_e(vessel: &Vessel, ox: &ChlorineOxidant, x: f64, start: (f64, f64, f64, f64)) -> f64 {
    let (ox0, h0, cl0, mn0) = start;
    let w = kgw(vessel);
    let s = crate::relations::nernst_slope(vessel.temperature);
    let h = ((h0 - ox.protons * x) / w).max(1e-300);
    let cl = ((cl0 - ox.chloride * x) / w).max(1e-300);
    let mn = ((mn0 + ox.mn * x) / w).max(1e-300);
    let ox_m = if ox.solid {
        1.0
    } else {
        ((ox0 - x) / w).max(1e-300)
    };
    let e_ox = ox.e0_v - s / ox.electrons * (mn / (ox_m * h.powf(ox.protons))).log10();
    let e_cl = E0_CL2 - s / 2.0 * (cl * cl).log10();
    e_ox - e_cl
}

fn preparation_extent(vessel: &Vessel, ox: &ChlorineOxidant) -> f64 {
    if !aqueous_ok(vessel) {
        return 0.0;
    }
    let ox0 = oxidant_available(vessel, ox);
    let h0 = acid_available(vessel) - crate::displacement::room_air_acidity_mol(vessel);
    let cl0 = chloride_available(vessel);
    if ox0 <= TRACE || h0 <= crate::OBSERVABLE_MOLES || cl0 <= TRACE {
        return 0.0;
    }
    let mn0 = amount(vessel, "Mn+2", &[Phase::Aqueous]);
    let xmax = ox0.min(h0 / ox.protons).min(cl0 / ox.chloride);
    let start = (ox0, h0, cl0, mn0);
    if delta_e(vessel, ox, 0.0, start) <= 0.0 {
        return 0.0;
    }
    // ΔE falls monotonically with extent; the reaction runs to the
    // potential crossing or to the limiting reagent, whichever is first.
    if delta_e(vessel, ox, xmax * (1.0 - 1e-12), start) >= 0.0 {
        return xmax;
    }
    let (mut lo, mut hi) = (0.0, xmax);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if delta_e(vessel, ox, mid, start) >= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

fn prepare(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut progressed = false;
    for ox in OXIDANTS {
        let x = preparation_extent(vessel, ox);
        if x <= TRACE || x * ox.chlorine <= crate::OBSERVABLE_MOLES {
            continue;
        }
        progressed = true;
        spend_acid(vessel, x * ox.protons);
        let mut left = x;
        for (form, cation) in ox.forms {
            if left <= TRACE {
                break;
            }
            let got = take(vessel, form, left, DISSOLVED_THEN_SOLID);
            if let Some(c) = cation {
                vessel.deposit(SpeciesId::new(*c), Moles(got), Phase::Aqueous);
            }
            left -= got;
        }
        let forms = chloride_forms();
        take_ion(vessel, CHLORIDE, &forms, x * ox.chloride);
        vessel.deposit(SpeciesId::new("Mn+2"), Moles(x * ox.mn), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("water"), Moles(x * ox.water), Phase::Liquid);
        reaction(events, vessel, ox.equation);
        emit_chlorine(vessel, x * ox.chlorine, events);
        refresh_charge(vessel);
        note(
            events,
            vessel,
            NotModelledCause::RateNotModelled,
            Phrase::new(
                "not-modeled.chlorine-preparation-ideal",
                "{oxidant} made chlorine from chloride in acid until the two electrode potentials met (CRC standard potentials, molality standing in for activity, chlorine at one atmosphere); activity coefficients in concentrated acid, the rate and the heat of this reaction are not modelled",
                vec![("oxidant".to_string(), Slot::text(crate::halogen_redox::pretty(ox.key)))],
            ),
        );
    }
    progressed
}

// ── 2. absorption in alkali ──────────────────────────────────────────

/// Base carriers and the hydroxide each delivers, with the cation it
/// leaves dissolved.
const BASES: &[(&str, f64, Option<&str>)] = &[
    (crate::species::BASE_EQUIVALENTS, 1.0, None),
    ("OH-", 1.0, None),
    ("NaOH", 1.0, Some("Na+")),
    ("KOH", 1.0, Some("K+")),
    ("Ca(OH)2", 2.0, Some("Ca+2")),
];

fn base_available(vessel: &Vessel) -> f64 {
    BASES
        .iter()
        .map(|(k, n, _)| n * amount(vessel, k, CONDENSED))
        .sum()
}

fn absorb_in_alkali(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !aqueous_ok(vessel) {
        return false;
    }
    let x = chlorine_pool(vessel).min(base_available(vessel) / 2.0);
    if x <= TRACE {
        return false;
    }
    take_chlorine(vessel, x);
    let mut need = 2.0 * x;
    for (key, per, cation) in BASES {
        if need <= TRACE {
            break;
        }
        let got = take(vessel, key, need / per, CONDENSED);
        if let Some(c) = cation {
            vessel.deposit(SpeciesId::new(*c), Moles(got), Phase::Aqueous);
        }
        need -= got * per;
    }
    vessel.deposit(SpeciesId::new("ClO-"), Moles(x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new(CHLORIDE), Moles(x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("water"), Moles(x), Phase::Liquid);
    reaction(events, vessel, "Cl₂ + 2 OH⁻ → ClO⁻ + Cl⁻ + H₂O");
    vessel.refresh_pressure();
    refresh_charge(vessel);
    true
}

// ── 3. reductants ────────────────────────────────────────────────────

/// (reduced ion, oxidised product, product phase, reductant per Cl₂,
/// equation). Strongest reductant first.
pub const REDUCTANTS: &[(&str, &str, Phase, f64, &str)] = &[
    ("I-", "I2", Phase::Aqueous, 2.0, "Cl₂ + 2 I⁻ → I₂ + 2 Cl⁻"),
    (
        "Fe+2",
        "Fe+3",
        Phase::Aqueous,
        2.0,
        "Cl₂ + 2 Fe²⁺ → 2 Fe³⁺ + 2 Cl⁻",
    ),
    (
        "Br-",
        "Br2",
        Phase::Liquid,
        2.0,
        "Cl₂ + 2 Br⁻ → Br₂ + 2 Cl⁻",
    ),
];

/// Whether this module prices an oxidant/reductant pair the halogen
/// writer table lists (so its honesty tail does not call it unpriced).
pub fn covers(oxidant: &str, reductant: &str) -> bool {
    oxidant == CHLORINE && REDUCTANTS.iter().any(|(r, ..)| *r == reductant)
}

fn ion_forms(ion: &str) -> Vec<String> {
    crate::halogen_redox::registry_forms(ion)
}

fn reduce(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !aqueous_ok(vessel) {
        return false;
    }
    let mut progressed = false;
    for (ion, product, phase, per, equation) in REDUCTANTS {
        let forms = ion_forms(ion);
        let have: f64 = forms
            .iter()
            .map(|f| amount(vessel, f, DISSOLVED_THEN_SOLID))
            .sum();
        if have <= TRACE || chlorine_pool(vessel) <= TRACE {
            continue;
        }
        if !in_registry(product) {
            note(
                events,
                vessel,
                NotModelledCause::PhaseNotInRegistry,
                Phrase::new(
                    "not-modeled.chlorine-product-unregistered",
                    "chlorine would oxidise {reductant} here, but the registry has no {product} to write, so it is left unreacted. That is a gap in the model, not evidence that nothing happens",
                    vec![
                        ("reductant".to_string(), Slot::text(crate::halogen_redox::pretty(ion))),
                        ("product".to_string(), Slot::text(crate::halogen_redox::pretty(product))),
                    ],
                ),
            );
            continue;
        }
        let x = chlorine_pool(vessel).min(have / per);
        if x <= TRACE {
            continue;
        }
        progressed = true;
        take_chlorine(vessel, x);
        take_ion(vessel, ion, &forms, x * per);
        vessel.deposit(SpeciesId::new(*product), Moles(x * per), *phase);
        vessel.deposit(SpeciesId::new(CHLORIDE), Moles(2.0 * x), Phase::Aqueous);
        reaction(events, vessel, equation);
        vessel.refresh_pressure();
        refresh_charge(vessel);
    }
    progressed
}

// ── 4. bleaching ─────────────────────────────────────────────────────

/// Pigments and the oxidised (colourless) form the registry writes for
/// each: the product is the pigment plus one oxygen, the simplified
/// stoichiometry of the curated hypochlorite rows.
pub const PIGMENTS: &[(&str, &str)] = &[
    ("anthocyanin", "anthocyanin_ox"),
    ("betanin", "betanin_ox"),
    ("curcumin", "curcumin_ox"),
    ("indigo_carmine", "indigo_carmine_ox"),
    ("hair_pigment", "hair_pigment_ox"),
    ("litmus", "litmus_ox"),
];

fn oxidising_chlorine(vessel: &Vessel) -> f64 {
    chlorine_pool(vessel)
        + amount(vessel, "HClO", &[Phase::Aqueous])
        + amount(vessel, "ClO-", &[Phase::Aqueous])
}

fn bleach(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !aqueous_ok(vessel) || oxidising_chlorine(vessel) <= TRACE {
        return false;
    }
    let mut progressed = false;
    let mut bleached: Vec<String> = Vec::new();
    for (dye, product) in PIGMENTS {
        let have = amount(vessel, dye, CONDENSED);
        if have <= TRACE {
            continue;
        }
        if !in_registry(product) {
            note(
                events,
                vessel,
                NotModelledCause::PhaseNotInRegistry,
                Phrase::new(
                    "not-modeled.chlorine-bleach-unregistered",
                    "chlorine or hypochlorite would bleach {pigment} here, but the registry has no oxidised form of it to write, so it is left unbleached. That is a gap in the model, not evidence that nothing happens",
                    vec![("pigment".to_string(), Slot::text(crate::halogen_redox::pretty(dye)))],
                ),
            );
            continue;
        }
        let mut left = have;
        // HClO first (the bleaching species), then Cl₂, then ClO⁻.
        let hclo = amount(vessel, "HClO", &[Phase::Aqueous]).min(left);
        if hclo > TRACE {
            take(vessel, "HClO", hclo, &[Phase::Aqueous]);
            vessel.deposit(SpeciesId::new(CHLORIDE), Moles(hclo), Phase::Aqueous);
            vessel.deposit(SpeciesId::new("H+"), Moles(hclo), Phase::Aqueous);
            left -= hclo;
            reaction(
                events,
                vessel,
                &format!(
                    "{} + HClO → {} + H⁺ + Cl⁻",
                    pretty_dye(dye),
                    pretty_dye(product)
                ),
            );
        }
        let cl2 = chlorine_pool(vessel).min(left).min(liquid_water(vessel));
        if cl2 > TRACE {
            take_chlorine(vessel, cl2);
            take(vessel, "water", cl2, &[Phase::Liquid]);
            vessel.deposit(SpeciesId::new(CHLORIDE), Moles(2.0 * cl2), Phase::Aqueous);
            vessel.deposit(SpeciesId::new("H+"), Moles(2.0 * cl2), Phase::Aqueous);
            left -= cl2;
            reaction(
                events,
                vessel,
                &format!(
                    "{} + Cl₂ + H₂O → {} + 2 H⁺ + 2 Cl⁻",
                    pretty_dye(dye),
                    pretty_dye(product)
                ),
            );
        }
        let clo = amount(vessel, "ClO-", &[Phase::Aqueous]).min(left);
        if clo > TRACE {
            take(vessel, "ClO-", clo, &[Phase::Aqueous]);
            vessel.deposit(SpeciesId::new(CHLORIDE), Moles(clo), Phase::Aqueous);
            left -= clo;
            reaction(
                events,
                vessel,
                &format!("{} + ClO⁻ → {} + Cl⁻", pretty_dye(dye), pretty_dye(product)),
            );
        }
        let spent = have - left;
        if spent <= TRACE {
            continue;
        }
        take(vessel, dye, spent, CONDENSED);
        vessel.deposit(SpeciesId::new(*product), Moles(spent), Phase::Aqueous);
        progressed = true;
        bleached.push(pretty_dye(dye));
    }
    if progressed {
        vessel.refresh_pressure();
        refresh_charge(vessel);
        note(
            events,
            vessel,
            NotModelledCause::RateNotModelled,
            Phrase::new(
                "not-modeled.chlorine-bleach-uncalibrated",
                "{pigments}: bleached by chlorine, hypochlorous acid or hypochlorite as a complete one-oxygen oxidation to a colourless form. Not calibrated: the bench has no reviewed rate or product identity for this bleaching, so the extent is stoichiometric and the time it takes is not modelled",
                vec![("pigments".to_string(), Slot::text(bleached.join(", ")))],
            ),
        );
    }
    progressed
}

fn pretty_dye(key: &str) -> String {
    crate::species::lookup_key(key)
        .map(|d| d.name.to_string())
        .unwrap_or_else(|| key.to_string())
}

// ── 5. chlorine water ────────────────────────────────────────────────

/// Fraction of the molecular pool that is dissolved, and the cap on the
/// dissolved amount for an open vessel (`None` for an owned headspace).
fn dissolved_share(vessel: &Vessel) -> (f64, Option<f64>) {
    let h = henry(vessel.temperature.0);
    let v_l = vessel.liquid_volume().0;
    match vessel.headspace_volume() {
        Some(v_hs) => {
            let held = h * v_l;
            let gas = (v_hs.0 / (R_L_ATM * vessel.temperature.0)).max(0.0);
            if held + gas <= 0.0 {
                (0.0, None)
            } else {
                (held / (held + gas), None)
            }
        }
        None => (1.0, Some(h * v_l)),
    }
}

/// Equilibrium HClO amount (mol) for the vessel's current pool, or
/// `None` when the rule does not apply.
fn hydrolysis_target(vessel: &Vessel) -> Option<(f64, f64)> {
    if !aqueous_ok(vessel) {
        return None;
    }
    let z0 = chlorine_pool(vessel);
    let y0 = amount(vessel, "HClO", &[Phase::Aqueous]);
    if z0 <= TRACE && y0 <= TRACE {
        return None;
    }
    // Only a vessel that HOLDS molecular chlorine is this rule's: HClO
    // with acid and chloride and no Cl₂ yet is the curated bleach rows'.
    if z0 <= TRACE {
        return None;
    }
    let w = kgw(vessel);
    if w <= 0.0 {
        return None;
    }
    let p = z0 + y0;
    let h0 = free_acid(vessel);
    let c0 = amount(vessel, CHLORIDE, &[Phase::Aqueous]);
    let (f, cap) = dissolved_share(vessel);
    let k = hydrolysis_k(vessel.temperature.0);
    let aq = |y: f64| {
        let z = p - y;
        match cap {
            Some(c) => z.min(c),
            None => z * f,
        }
    };
    let g = |y: f64| {
        let h = (h0 + y - y0).max(0.0) / w;
        let c = (c0 + y - y0).max(0.0) / w;
        (y / w) * h * c - k * aq(y) / w
    };
    let lo0 = 0.0f64.max(y0 - h0).max(y0 - c0);
    let (mut lo, mut hi) = (lo0, p);
    if g(lo) >= 0.0 {
        hi = lo;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if g(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let y = 0.5 * (lo + hi);
    Some((y, y0))
}

fn tolerance(vessel: &Vessel) -> f64 {
    (1e-9 * (chlorine_pool(vessel) + amount(vessel, "HClO", &[Phase::Aqueous]))).max(TRACE)
}

fn open_excess(vessel: &Vessel) -> f64 {
    if !aqueous_ok(vessel) {
        return 0.0;
    }
    match dissolved_share(vessel) {
        (_, Some(cap)) => (chlorine_pool(vessel) - cap).max(0.0),
        _ => 0.0,
    }
}

fn chlorine_water(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut progressed = false;
    if let Some((y, y0)) = hydrolysis_target(vessel) {
        let dy = y - y0;
        if dy.abs() > tolerance(vessel) {
            progressed = true;
            if dy > 0.0 {
                take_chlorine(vessel, dy);
                take(vessel, "water", dy, &[Phase::Liquid]);
                vessel.deposit(SpeciesId::new("HClO"), Moles(dy), Phase::Aqueous);
                vessel.deposit(SpeciesId::new(CHLORIDE), Moles(dy), Phase::Aqueous);
                vessel.deposit(SpeciesId::new("H+"), Moles(dy), Phase::Aqueous);
                reaction(events, vessel, "Cl₂ + H₂O ⇌ HClO + H⁺ + Cl⁻");
            } else {
                let back = -dy;
                take(vessel, "HClO", back, &[Phase::Aqueous]);
                take(vessel, CHLORIDE, back, &[Phase::Aqueous]);
                refresh_charge(vessel);
                crate::displacement::consume_acidity(vessel, back);
                vessel.deposit(SpeciesId::new(CHLORINE), Moles(back), Phase::Gas);
                vessel.deposit(SpeciesId::new("water"), Moles(back), Phase::Liquid);
                reaction(events, vessel, "HClO + H⁺ + Cl⁻ ⇌ Cl₂ + H₂O");
            }
            refresh_charge(vessel);
            let (f, _) = dissolved_share(vessel);
            note(
                events,
                vessel,
                NotModelledCause::ModelBoundary,
                Phrase::new(
                    "not-modeled.chlorine-water-pool",
                    "chlorine water: dissolved molecular chlorine is held with the headspace chlorine as one pool ({share} of it in solution by Henry's law); hydrolysis Cl₂ + H₂O ⇌ HClO + H⁺ + Cl⁻ is solved to its NBS constant with molality standing in for activity, and the shown headspace pressure counts the dissolved share",
                    vec![("share".to_string(), Slot::number(format!("{:.0} %", f * 100.0)))],
                ),
            );
        }
    }
    let excess = open_excess(vessel);
    if excess > tolerance(vessel) {
        progressed = true;
        let got = take_chlorine(vessel, excess);
        events.push(Event::GasEvolved {
            vessel: vessel.id,
            species: SpeciesId::new(CHLORINE),
            moles: Moles(got),
        });
        vessel.refresh_pressure();
    }
    progressed
}

/// Whether the hydrolysis is this module's in this vessel (so the curated
/// `HClO + H⁺ + Cl⁻ → Cl₂` rows must not also run it to completion).
pub fn owns_hypochlorous_equilibrium(vessel: &Vessel) -> bool {
    aqueous_ok(vessel) && chlorine_pool(vessel) > TRACE
}

// ── 6. dry chlorination ──────────────────────────────────────────────

/// (element, chloride, Cl per element atom).
pub const METAL_CHLORIDES: &[(&str, &str, f64)] = &[
    ("Na", "NaCl", 1.0),
    ("K", "KCl", 1.0),
    ("Mg", "MgCl2", 2.0),
    ("Ca", "CaCl2", 2.0),
    ("Zn", "ZnCl2", 2.0),
    ("Al", "AlCl3", 3.0),
    ("Fe", "FeCl3", 3.0),
    ("Cu", "CuCl2", 2.0),
];

fn hot(vessel: &Vessel) -> bool {
    vessel.ignition || vessel.temperature.0 >= DRY_CHLORINATION_K
}

fn chlorine_gas(vessel: &Vessel) -> f64 {
    amount(vessel, CHLORINE, &[Phase::Gas])
}

fn metal_equation(metal: &str, chloride: &str, cl: f64) -> String {
    // n M + (n·cl/2) Cl₂ → n MClcl, smallest integers.
    let (n, m) = if (cl as i64) % 2 == 0 {
        (1, (cl as i64) / 2)
    } else {
        (2, cl as i64)
    };
    let c = |k: i64| {
        if k == 1 {
            String::new()
        } else {
            format!("{k} ")
        }
    };
    format!(
        "{}{} + {}Cl₂ → {}{}",
        c(n),
        metal,
        c(m),
        c(n),
        crate::halogen_redox::pretty(chloride)
    )
}

fn dry_chlorination(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !hot(vessel) || chlorine_gas(vessel) <= TRACE {
        return false;
    }
    let mut progressed = false;
    let mut burned: Vec<String> = Vec::new();
    for (metal, chloride, cl) in METAL_CHLORIDES {
        let have = amount(vessel, metal, &[Phase::Solid, Phase::Liquid]);
        if have <= TRACE {
            continue;
        }
        if !in_registry(chloride) {
            note(
                events,
                vessel,
                NotModelledCause::PhaseNotInRegistry,
                Phrase::new(
                    "not-modeled.chlorine-product-unregistered",
                    "chlorine would oxidise {reductant} here, but the registry has no {product} to write, so it is left unreacted. That is a gap in the model, not evidence that nothing happens",
                    vec![
                        ("reductant".to_string(), Slot::text(crate::halogen_redox::pretty(metal))),
                        ("product".to_string(), Slot::text(crate::halogen_redox::pretty(chloride))),
                    ],
                ),
            );
            continue;
        }
        let n = have.min(2.0 * chlorine_gas(vessel) / cl);
        if n <= TRACE {
            continue;
        }
        take(vessel, metal, n, &[Phase::Solid, Phase::Liquid]);
        take(vessel, CHLORINE, n * cl / 2.0, &[Phase::Gas]);
        vessel.deposit(SpeciesId::new(*chloride), Moles(n), Phase::Solid);
        reaction(events, vessel, &metal_equation(metal, chloride, *cl));
        burned.push(metal.to_string());
        progressed = true;
    }
    let h2 = amount(vessel, "H2", &[Phase::Gas]);
    let x = h2.min(chlorine_gas(vessel));
    if x > TRACE && in_registry("HCl") {
        take(vessel, "H2", x, &[Phase::Gas]);
        take(vessel, CHLORINE, x, &[Phase::Gas]);
        vessel.deposit(SpeciesId::new("HCl"), Moles(2.0 * x), Phase::Gas);
        reaction(events, vessel, "H₂ + Cl₂ → 2 HCl");
        burned.push("H2".to_string());
        progressed = true;
    }
    if progressed {
        vessel.refresh_pressure();
        note(
            events,
            vessel,
            NotModelledCause::NoReviewedDatum,
            Phrase::new(
                "not-modeled.chlorine-combustion-heat-unpriced",
                "{fuels} burned in chlorine to completion (hot or ignited); heat unpriced: no reviewed reaction enthalpy or ignition temperature is applied, so the flame's heat and the temperature it would reach are not modelled",
                vec![("fuels".to_string(), Slot::text(burned.join(", ")))],
            ),
        );
    }
    progressed
}

/// Hydrogen chloride gas over liquid water dissolves (very soluble) up to
/// [`HCL_UPTAKE_MOL_PER_KG`].
fn dissolve_hydrogen_chloride(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !aqueous_ok(vessel) {
        return false;
    }
    let gas = amount(vessel, "HCl", &[Phase::Gas]);
    if gas <= TRACE {
        return false;
    }
    let held = amount(vessel, CHLORIDE, &[Phase::Aqueous]);
    let room = (HCL_UPTAKE_MOL_PER_KG * kgw(vessel) - held).max(0.0);
    let x = gas.min(room);
    if x <= TRACE {
        return false;
    }
    take(vessel, "HCl", x, &[Phase::Gas]);
    vessel.deposit(SpeciesId::new("H+"), Moles(x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new(CHLORIDE), Moles(x), Phase::Aqueous);
    reaction(events, vessel, "HCl(g) → H⁺ + Cl⁻");
    vessel.refresh_pressure();
    refresh_charge(vessel);
    true
}

// ── entry points ─────────────────────────────────────────────────────

/// Whether any chlorine rule would change this vessel.
pub fn fires(vessel: &Vessel) -> bool {
    let wet = aqueous_ok(vessel);
    let pool = chlorine_pool(vessel);
    if OXIDANTS
        .iter()
        .any(|ox| preparation_extent(vessel, ox) * ox.chlorine > crate::OBSERVABLE_MOLES)
    {
        return true;
    }
    if wet && pool > TRACE && base_available(vessel) > TRACE {
        return true;
    }
    if wet
        && pool > TRACE
        && REDUCTANTS.iter().any(|(ion, product, ..)| {
            in_registry(product)
                && ion_forms(ion)
                    .iter()
                    .any(|f| amount(vessel, f, DISSOLVED_THEN_SOLID) > TRACE)
        })
    {
        return true;
    }
    if wet
        && oxidising_chlorine(vessel) > TRACE
        && PIGMENTS
            .iter()
            .any(|(d, p)| in_registry(p) && amount(vessel, d, CONDENSED) > TRACE)
    {
        return true;
    }
    if let Some((y, y0)) = hydrolysis_target(vessel) {
        if (y - y0).abs() > tolerance(vessel) {
            return true;
        }
    }
    if open_excess(vessel) > tolerance(vessel) {
        return true;
    }
    if hot(vessel)
        && chlorine_gas(vessel) > TRACE
        && (METAL_CHLORIDES.iter().any(|(m, c, _)| {
            in_registry(c) && amount(vessel, m, &[Phase::Solid, Phase::Liquid]) > TRACE
        }) || amount(vessel, "H2", &[Phase::Gas]) > TRACE)
    {
        return true;
    }
    wet && amount(vessel, "HCl", &[Phase::Gas]) > TRACE
        && HCL_UPTAKE_MOL_PER_KG * kgw(vessel) > amount(vessel, CHLORIDE, &[Phase::Aqueous])
}

/// Run every chlorine rule once, in order. Returns whether anything
/// happened; the caller refreshes `solute_charge`.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut progressed = false;
    progressed |= dry_chlorination(vessel, events);
    progressed |= dissolve_hydrogen_chloride(vessel, events);
    progressed |= prepare(vessel, events);
    progressed |= absorb_in_alkali(vessel, events);
    progressed |= reduce(vessel, events);
    progressed |= bleach(vessel, events);
    progressed |= chlorine_water(vessel, events);
    progressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrolysis_constant_is_the_nbs_derivation() {
        let (g, h) = hydrolysis_reaction_kj();
        assert!((g - 19.061).abs() < 1e-9, "{g}");
        assert!((h - 21.171).abs() < 1e-9, "{h}");
        let k = hydrolysis_k(298.15);
        assert!((k - 4.58e-4).abs() < 0.01e-4, "{k}");
        // Endothermic: more hydrolysis when warmer.
        assert!(hydrolysis_k(318.15) > k);
    }

    #[test]
    fn metal_equations_balance_in_smallest_integers() {
        assert_eq!(metal_equation("Fe", "FeCl3", 3.0), "2 Fe + 3 Cl₂ → 2 FeCl₃");
        assert_eq!(metal_equation("Cu", "CuCl2", 2.0), "Cu + Cl₂ → CuCl₂");
        assert_eq!(metal_equation("Na", "NaCl", 1.0), "2 Na + Cl₂ → 2 NaCl");
    }

    #[test]
    fn pairs_this_module_prices_are_covered() {
        assert!(covers("Cl2", "I-"));
        assert!(covers("Cl2", "Fe+2"));
        assert!(!covers("Cl2", "SO3-2"));
        assert!(!covers("I2", "Fe+2"));
    }
}
