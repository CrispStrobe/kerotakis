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
//!    These, and rule 3, are rows of the generic potential-driven route in
//!    [`crate::potential_redox`] (which also prepares bromine and iodine
//!    the same way); this module only keeps their place in the rule order.
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

/// CRC "Electrochemical Series" standard potentials, V (the rows live in
/// `crate::potential_redox`).
pub const E0_CL2: f64 = crate::potential_redox::E0_CL2;
pub const E0_MNO2: f64 = crate::potential_redox::E0_MNO2;
pub const E0_MNO4: f64 = crate::potential_redox::E0_MNO4;

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
}

/// The hydrolysis constant's continuation range (see [`AQUEOUS_DOMAIN_K`]).
fn hydrolysis_domain(vessel: &Vessel) -> bool {
    aqueous_ok(vessel) && (AQUEOUS_DOMAIN_K.0..=AQUEOUS_DOMAIN_K.1).contains(&vessel.temperature.0)
}

/// Molecular chlorine pool (headspace + dissolved; see module notes).
pub fn chlorine_pool(vessel: &Vessel) -> f64 {
    amount(vessel, CHLORINE, &[Phase::Gas, Phase::Aqueous])
}

pub(crate) fn take_chlorine(vessel: &mut Vessel, n: f64) -> f64 {
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
pub(crate) fn emit_chlorine(vessel: &mut Vessel, n: f64, events: &mut Vec<Event>) {
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
//
// The oxidant + acidified chloride preparations are rows of the generic
// potential-driven route (`crate::potential_redox`): MnO₂/Mn²⁺,
// MnO₄⁻/Mn²⁺ and Cr₂O₇²⁻/Cr³⁺ against Cl₂/Cl⁻, run to the crossing of the
// two Nernst potentials. This module keeps their place in its rule order.

fn prepare(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    crate::potential_redox::apply_in(vessel, events, crate::potential_redox::Scope::MakesChlorine)
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
            vessel.deposit(SpeciesId::new(c), Moles(got), Phase::Aqueous);
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
//
// Halide displacement and iron(II): rows of `crate::potential_redox` with
// Cl₂/Cl⁻ as the oxidant couple, strongest reductant first.

/// Whether this module prices an oxidant/reductant pair the halogen
/// writer table lists (so its honesty tail does not call it unpriced).
pub fn covers(oxidant: &str, reductant: &str) -> bool {
    oxidant == CHLORINE && crate::potential_redox::covers(oxidant, reductant)
}

fn reduce(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !aqueous_ok(vessel) {
        return false;
    }
    crate::potential_redox::apply_in(
        vessel,
        events,
        crate::potential_redox::Scope::ChlorineOxidant,
    )
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
        vessel.deposit(SpeciesId::new(product), Moles(spent), Phase::Aqueous);
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
    if !hydrolysis_domain(vessel) {
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
                if dy > crate::OBSERVABLE_MOLES {
                    reaction(events, vessel, "Cl₂ + H₂O ⇌ HClO + H⁺ + Cl⁻");
                }
            } else {
                let back = -dy;
                take(vessel, "HClO", back, &[Phase::Aqueous]);
                take(vessel, CHLORIDE, back, &[Phase::Aqueous]);
                refresh_charge(vessel);
                crate::displacement::consume_acidity(vessel, back);
                vessel.deposit(SpeciesId::new(CHLORINE), Moles(back), Phase::Gas);
                vessel.deposit(SpeciesId::new("water"), Moles(back), Phase::Liquid);
                if back > crate::OBSERVABLE_MOLES {
                    reaction(events, vessel, "HClO + H⁺ + Cl⁻ ⇌ Cl₂ + H₂O");
                }
            }
            refresh_charge(vessel);
            let (f, _) = dissolved_share(vessel);
            if dy.abs() > crate::OBSERVABLE_MOLES {
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
    hydrolysis_domain(vessel) && chlorine_pool(vessel) > TRACE
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
        vessel.deposit(SpeciesId::new(chloride), Moles(n), Phase::Solid);
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
    if crate::potential_redox::fires_in(vessel, crate::potential_redox::Scope::MakesChlorine) {
        return true;
    }
    if wet && pool > TRACE && base_available(vessel) > TRACE {
        return true;
    }
    if wet
        && pool > TRACE
        && crate::potential_redox::fires_in(vessel, crate::potential_redox::Scope::ChlorineOxidant)
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
