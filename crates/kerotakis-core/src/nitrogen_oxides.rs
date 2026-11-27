//! Nitrogen oxides and nitrous acid: the chemistry after the copper.
//!
//! Copper with dilute nitric acid gives colourless nitrogen monoxide, with
//! concentrated acid brown nitrogen dioxide (both rows of
//! [`crate::concentrated_sulfuric`]). This module owns what those gases
//! and nitrite do next. Every rule is a reaction CLASS over the registry's
//! existing species; nothing here is a scenario.
//!
//! The rules, in the order [`apply`] runs them:
//!
//! 1. **Nitrogen monoxide meets oxygen** (`2 NO + O₂ → 2 NO₂`) in a vessel
//!    that owns its headspace (a sealed jar traps room air). ΔrG° =
//!    −70.48 kJ/mol (K ≈ 2·10¹²), so it is booked to the limiting gas.
//!    The rate is termolecular (IUPAC: k = 3.3·10⁻³⁹ exp(530/T)
//!    cm⁶ molecule⁻² s⁻¹, Atkinson et al., Atmos. Chem. Phys. 4 (2004)
//!    1461, evaluation of NO + NO + O₂); it is not integrated, and a typed
//!    note says so. Without oxygen (a nitrogen purge, or a jar whose oxygen
//!    is spent) nitrogen monoxide stays colourless. From an OPEN vessel the
//!    gas escapes into room air, which is outside the ledger: the brown at
//!    the mouth of the vessel is said in a note, not booked.
//! 2. **Nitrogen dioxide into water**
//!    (`2 NO₂ + H₂O ⇌ H⁺ + NO₃⁻ + HNO₂`), solved to its constant against
//!    the headspace partial pressure of the MONOMER. Nitrous acid is held
//!    as its ions H⁺ + NO₂⁻ (the registry has no molecular HNO₂); its
//!    protonated share is computed from its acid constant wherever a rule
//!    needs it.
//! 3. **Nitrous acid decomposes** (`3 HNO₂ ⇌ H⁺ + NO₃⁻ + 2 NO + H₂O`,
//!    booked on the ledger as `3 NO₂⁻ + 2 H⁺ → NO₃⁻ + 2 NO + H₂O`), solved
//!    to its constant against the nitrogen monoxide pressure of an owned
//!    headspace, or against one atmosphere in an open vessel (the bench's
//!    bubbling convention: gas leaves when its equilibrium pressure would
//!    exceed the atmosphere). Rule 2 followed by rule 3 is the textbook
//!    overall `3 NO₂ + H₂O → 2 HNO₃ + NO`. No rate is claimed: acidified
//!    nitrite decomposes over minutes to hours, and the note says so.
//!
//! **Dimerisation** `2 NO₂ ⇌ N₂O₄` is not a writer: the registry has no
//! N₂O₄ row, so the headspace `NO2` portion is one pool counted in NO₂
//! units (mass and nitrogen exact), and [`monomer_moles`] splits it by the
//! equilibrium constant at the vessel's temperature whenever the monomer
//! matters: the headspace colour ([`crate::gas_colour`]) and the monomer
//! pressure of rule 2. N₂O₄ is colourless in the visible, so cooling a
//! brown jar fades it. The shown headspace pressure still counts the pool
//! as monomer; that, the unpriced dimerisation heat and the condensation
//! of N₂O₄ below its boiling point (21 °C at 1 atm) are said in a note.
//!
//! ## Data
//!
//! All constants are DERIVED from the NBS tables (D. D. Wagman et al.,
//! J. Phys. Chem. Ref. Data 11 (1982) Suppl. 2, Table 18, nitrogen,
//! 298.15 K, 1 bar standard state): ΔfG°/ΔfH° kJ/mol of NO(g) 86.55/90.25,
//! NO₂(g) 51.31/33.18, N₂O₄(g) 97.89/9.16, HNO₂(ao) −50.6/−119.2,
//! NO₂⁻(ai) −32.2/−104.6, NO₃⁻(ai) −108.74/−205.0, H₂O(l)
//! −237.129/−285.830. The nitrous acid constant they give, pKa 3.22,
//! agrees with the pinned llnl.dat row `NO2- + H+ = HNO2`, `log_k 3.2206`.
//! Temperature dependence is van 't Hoff with the 25 °C reaction enthalpy,
//! a bench-range continuation. Molality stands in for activity.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::{Headspace, Vessel};
use crate::SpeciesId;

const TRACE: f64 = 1e-12;

/// L·bar/(mol·K): the NBS standard state is 1 bar.
const R_L_BAR: f64 = 0.083_144_626;
/// One atmosphere in bar (the open-vessel bubbling pressure).
const ATM_BAR: f64 = 1.013_25;

/// Aqueous domain (K) of rules 2 and 3 (constant-enthalpy continuation).
pub const AQUEOUS_DOMAIN_K: (f64, f64) = (273.15, 373.15);

/// NBS 1982 (Wagman et al.) formation data, kJ/mol: (ΔfG°, ΔfH°).
pub const NBS_NO_G: (f64, f64) = (86.55, 90.25);
pub const NBS_NO2_G: (f64, f64) = (51.31, 33.18);
pub const NBS_N2O4_G: (f64, f64) = (97.89, 9.16);
pub const NBS_HNO2_AQ: (f64, f64) = (-50.6, -119.2);
pub const NBS_NO2_ION: (f64, f64) = (-32.2, -104.6);
pub const NBS_NO3_ION: (f64, f64) = (-108.74, -205.0);
pub const NBS_WATER_L: (f64, f64) = (-237.129, -285.830);

const NO: &str = "NO";
const NO2: &str = "NO2";
const O2: &str = "O2";
const NITRITE: &str = "NO2-";
const NITRATE: &str = "NO3-";

/// Nitrite carriers in solution: (registry key, cation left dissolved).
const NITRITE_CARRIERS: &[(&str, Option<&str>)] = &[
    (NITRITE, None),
    ("NaNO2", Some("Na+")),
    ("KNO2", Some("K+")),
];

fn sum(terms: &[((f64, f64), f64)]) -> (f64, f64) {
    terms
        .iter()
        .fold((0.0, 0.0), |(g, h), ((dg, dh), n)| (g + n * dg, h + n * dh))
}

/// `2 NO₂ ⇌ N₂O₄`: (ΔrG°, ΔrH°) kJ/mol.
pub fn dimerisation_kj() -> (f64, f64) {
    sum(&[(NBS_N2O4_G, 1.0), (NBS_NO2_G, -2.0)])
}

/// `2 NO + O₂ → 2 NO₂`: (ΔrG°, ΔrH°) kJ/mol.
pub fn oxidation_kj() -> (f64, f64) {
    sum(&[(NBS_NO2_G, 2.0), (NBS_NO_G, -2.0)])
}

/// `2 NO₂(g) + H₂O(l) ⇌ H⁺ + NO₃⁻ + HNO₂(ao)`: (ΔrG°, ΔrH°) kJ/mol.
pub fn absorption_kj() -> (f64, f64) {
    sum(&[
        (NBS_NO3_ION, 1.0),
        (NBS_HNO2_AQ, 1.0),
        (NBS_NO2_G, -2.0),
        (NBS_WATER_L, -1.0),
    ])
}

/// `3 HNO₂(ao) ⇌ H⁺ + NO₃⁻ + 2 NO(g) + H₂O(l)`: (ΔrG°, ΔrH°) kJ/mol.
pub fn decomposition_kj() -> (f64, f64) {
    sum(&[
        (NBS_NO3_ION, 1.0),
        (NBS_NO_G, 2.0),
        (NBS_WATER_L, 1.0),
        (NBS_HNO2_AQ, -3.0),
    ])
}

/// `HNO₂(ao) ⇌ H⁺ + NO₂⁻`: (ΔrG°, ΔrH°) kJ/mol.
pub fn nitrous_acid_kj() -> (f64, f64) {
    sum(&[(NBS_NO2_ION, 1.0), (NBS_HNO2_AQ, -1.0)])
}

/// Equilibrium constant at `t` K from (ΔrG°, ΔrH°) at 298.15 K.
pub fn constant(reaction: (f64, f64), t: f64) -> f64 {
    let r = crate::constants::GAS_CONSTANT;
    let (g, h) = reaction;
    let k298 = (-g * 1e3 / (r * 298.15)).exp();
    k298 * (-h * 1e3 / r * (1.0 / t - 1.0 / 298.15)).exp()
}

/// `p(N₂O₄)/p(NO₂)²`, bar⁻¹.
pub fn dimerisation_kp(t: f64) -> f64 {
    constant(dimerisation_kj(), t)
}

/// Acid constant of nitrous acid, mol/kg.
pub fn nitrous_acid_ka(t: f64) -> f64 {
    constant(nitrous_acid_kj(), t)
}

/// Monomer moles in a pool of `pool` mol NO₂ units held in `litres` at
/// `t` K: `n_M + 2 n_D = pool`, `n_D / n_M² = Kp·R·T / V`.
pub fn monomer_moles(pool: f64, t: f64, litres: f64) -> f64 {
    if !(pool > 0.0 && t > 0.0 && litres > 0.0) {
        return pool.max(0.0);
    }
    let k = dimerisation_kp(t) * R_L_BAR * t / litres;
    if !(k.is_finite() && k > 0.0) {
        return pool;
    }
    // Stable root of 2k n² + n − pool = 0.
    2.0 * pool / (1.0 + (1.0 + 8.0 * k * pool).sqrt())
}

/// Moles of a headspace gas that absorb as the registry species: the
/// NO₂ pool's monomer share, every other gas whole.
pub fn coloured_moles(vessel: &Vessel, key: &str, moles: f64) -> f64 {
    match (key, vessel.headspace_volume()) {
        (NO2, Some(v)) => monomer_moles(moles, vessel.temperature.0, v.0),
        _ => moles,
    }
}

/// Share of the headspace NO₂ pool (in NO₂ units) bound as N₂O₄, or
/// `None` when the vessel owns no NO₂.
pub fn dimer_share(vessel: &Vessel) -> Option<f64> {
    let pool = gas(vessel, NO2);
    let v = vessel.headspace_volume()?.0;
    (pool > TRACE).then(|| 1.0 - monomer_moles(pool, vessel.temperature.0, v) / pool)
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

fn gas(vessel: &Vessel, key: &str) -> f64 {
    amount(vessel, key, &[Phase::Gas])
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

fn in_registry(key: &str) -> bool {
    crate::species::lookup_key(key).is_some()
}

fn kgw(vessel: &Vessel) -> f64 {
    amount(vessel, "water", &[Phase::Liquid]) * crate::constants::WATER_MOLAR_MASS_KG_PER_MOL
}

fn aqueous_domain(vessel: &Vessel) -> bool {
    kgw(vessel) > 0.0 && (AQUEOUS_DOMAIN_K.0..=AQUEOUS_DOMAIN_K.1).contains(&vessel.temperature.0)
}

fn free_acid(vessel: &Vessel) -> f64 {
    (-crate::displacement::solute_charge(vessel)).max(0.0)
}

fn refresh_charge(vessel: &mut Vessel) {
    vessel.solute_charge = crate::displacement::solute_charge(vessel);
}

fn nitrite(vessel: &Vessel) -> f64 {
    NITRITE_CARRIERS
        .iter()
        .filter(|(k, _)| in_registry(k))
        .map(|(k, _)| amount(vessel, k, &[Phase::Aqueous]))
        .sum()
}

fn take_nitrite(vessel: &mut Vessel, n: f64) {
    let mut left = n;
    for (key, cation) in NITRITE_CARRIERS {
        if left <= TRACE || !in_registry(key) {
            continue;
        }
        let got = take(vessel, key, left, &[Phase::Aqueous]);
        if let Some(c) = cation {
            if got > 0.0 {
                vessel.deposit(SpeciesId::new(c), Moles(got), Phase::Aqueous);
            }
        }
        left -= got;
    }
}

fn nitrate(vessel: &Vessel) -> f64 {
    amount(vessel, NITRATE, &[Phase::Aqueous]) + amount(vessel, "HNO3", &[Phase::Aqueous])
}

/// Protonated nitrous acid (molal) from total protons `ht` and total
/// nitrite `n3` (molal): `y = n3·h/(h + Ka)`, `h = ht − y`.
fn hno2_molal(ht: f64, n3: f64, ka: f64) -> f64 {
    if !(ht > 0.0 && n3 > 0.0) {
        return 0.0;
    }
    let b = ht + ka + n3;
    let disc = (b * b - 4.0 * n3 * ht).max(0.0);
    // Stable form of (b − √disc)/2.
    (2.0 * n3 * ht / (b + disc.sqrt())).min(n3.min(ht))
}

fn note(events: &mut Vec<Event>, vessel: &Vessel, phrase: Phrase) {
    let dup = events.iter().any(|e| match e {
        Event::NotYetModeled {
            reason: Some(r), ..
        } => r.key == phrase.key,
        _ => false,
    });
    if !dup {
        events.push(Event::not_modeled(
            vessel.id,
            NotModelledCause::ModelBoundary,
            phrase,
        ));
    }
}

fn reaction(events: &mut Vec<Event>, vessel: &Vessel, equation: &str) {
    events.push(Event::ReactionOccurred {
        vessel: vessel.id,
        equation: equation.to_string(),
    });
}

fn tolerance(scale: f64) -> f64 {
    (1e-9 * scale).max(TRACE)
}

/// Solve `f(x) = 0` on `[0, hi]` for an `f` that rises through zero;
/// returns `hi` when `f` stays below zero there.
fn bisect(hi: f64, f: impl Fn(f64) -> f64) -> f64 {
    if hi.is_nan() || hi <= 0.0 {
        return 0.0;
    }
    let f0 = f(0.0);
    if f0.is_nan() || f0 >= 0.0 {
        return 0.0;
    }
    if f(hi) < 0.0 {
        return hi;
    }
    let (mut lo, mut up) = (0.0, hi);
    for _ in 0..200 {
        let mid = 0.5 * (lo + up);
        if f(mid) < 0.0 {
            lo = mid;
        } else {
            up = mid;
        }
    }
    lo
}

// ── 1. NO + O2 ───────────────────────────────────────────────────────

fn oxidation_extent(vessel: &Vessel) -> f64 {
    if !vessel.owns_headspace_gas() {
        return 0.0;
    }
    (gas(vessel, NO) / 2.0).min(gas(vessel, O2))
}

fn oxidise(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let x = oxidation_extent(vessel);
    let mut progressed = false;
    if x > TRACE {
        take(vessel, NO, 2.0 * x, &[Phase::Gas]);
        take(vessel, O2, x, &[Phase::Gas]);
        vessel.deposit(SpeciesId::new(NO2), Moles(2.0 * x), Phase::Gas);
        vessel.refresh_pressure();
        reaction(events, vessel, "2 NO + O₂ → 2 NO₂");
        note(
            events,
            vessel,
            Phrase::new(
                "not-modeled.nitric-oxide-oxidation",
                "colourless nitrogen monoxide met the oxygen in the headspace and {amount} mol of brown nitrogen dioxide formed (2 NO + O₂ → 2 NO₂, ΔrG° −70.5 kJ/mol from the NBS tables); it is booked to completion: the termolecular rate (IUPAC) is fast at these amounts but not integrated, and the reaction heat is not priced",
                vec![("amount".to_string(), Slot::number(format!("{:.3e}", 2.0 * x)))],
            ),
        );
        progressed = true;
    }
    if open_nitric_oxide(vessel, events) {
        note(
            events,
            vessel,
            Phrase::bare(
                "not-modeled.nitric-oxide-meets-air",
                "the nitrogen monoxide leaving this open vessel is colourless, but it turns brown where it meets room air at the mouth of the vessel (2 NO + O₂ → 2 NO₂); room air is outside the ledger, so that nitrogen dioxide is not booked",
            ),
        );
    }
    progressed
}

fn open_nitric_oxide(vessel: &Vessel, events: &[Event]) -> bool {
    matches!(vessel.headspace, Headspace::Open)
        && events.iter().any(|e| {
            matches!(e, Event::GasEvolved { vessel: id, species, moles }
                if *id == vessel.id && species.0 == NO && moles.0 > TRACE)
        })
}

// ── 2. NO2 into water ────────────────────────────────────────────────

fn absorption_extent(vessel: &Vessel) -> f64 {
    if !aqueous_domain(vessel) {
        return 0.0;
    }
    let Some(litres) = vessel.headspace_volume().map(|v| v.0) else {
        return 0.0;
    };
    let pool = gas(vessel, NO2);
    if !(pool > TRACE && litres > 0.0) {
        return 0.0;
    }
    let t = vessel.temperature.0;
    let w = kgw(vessel);
    let (p0, n0, a0) = (free_acid(vessel), nitrite(vessel), nitrate(vessel));
    let ka = nitrous_acid_ka(t);
    let ln_k = constant(absorption_kj(), t).ln();
    let hi = (pool / 2.0).min(amount(vessel, "water", &[Phase::Liquid]));
    bisect(hi * (1.0 - 1e-12), |x| {
        let ht = (p0 + 2.0 * x) / w;
        let n3 = (n0 + x) / w;
        let y = hno2_molal(ht, n3, ka);
        let h = ht - y;
        let p = monomer_moles(pool - 2.0 * x, t, litres) * R_L_BAR * t / litres;
        let q = h * ((a0 + x) / w) * y / (p * p);
        q.ln() - ln_k
    })
}

fn absorb(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let x = absorption_extent(vessel);
    if x <= tolerance(gas(vessel, NO2)) {
        return false;
    }
    take(vessel, NO2, 2.0 * x, &[Phase::Gas]);
    take(vessel, "water", x, &[Phase::Liquid]);
    vessel.deposit(SpeciesId::new("H+"), Moles(2.0 * x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new(NITRATE), Moles(x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new(NITRITE), Moles(x), Phase::Aqueous);
    refresh_charge(vessel);
    vessel.refresh_pressure();
    if x > crate::OBSERVABLE_MOLES {
        reaction(events, vessel, "2 NO₂ + H₂O ⇌ HNO₃ + HNO₂");
    }
    note(
        events,
        vessel,
        Phrase::new(
            "not-modeled.nitrogen-dioxide-absorption",
            "nitrogen dioxide dissolved in the water as nitric and nitrous acid ({amount} mol NO₂; 2 NO₂ + H₂O ⇌ HNO₃ + HNO₂), solved to its NBS constant against the monomer pressure with molality standing in for activity; nitrous acid is held as its ions H⁺ + NO₂⁻ and the uptake rate and reaction heat are not modelled",
            vec![("amount".to_string(), Slot::number(format!("{:.3e}", 2.0 * x)))],
        ),
    );
    true
}

// ── 3. nitrous acid decomposition ────────────────────────────────────

fn decomposition_extent(vessel: &Vessel) -> f64 {
    if !aqueous_domain(vessel) {
        return 0.0;
    }
    let (p0, n0, a0) = (free_acid(vessel), nitrite(vessel), nitrate(vessel));
    if !(p0 > TRACE && n0 > TRACE) {
        return 0.0;
    }
    let t = vessel.temperature.0;
    let w = kgw(vessel);
    let ka = nitrous_acid_ka(t);
    let ln_k = constant(decomposition_kj(), t).ln();
    // Open: gas leaves once its equilibrium pressure passes 1 atm.
    // Swept: the purge carries it away at its own pressure.
    let owned = match (vessel.owns_headspace_gas(), vessel.headspace_volume()) {
        (true, Some(v)) if v.0 > 0.0 => Some((gas(vessel, NO), v.0)),
        _ => None,
    };
    let pressure = |x: f64| match owned {
        Some((n_no, litres)) => (n_no + 2.0 * x) * R_L_BAR * t / litres,
        None => ATM_BAR,
    };
    let hi = (n0 / 3.0).min(p0 / 2.0);
    bisect(hi * (1.0 - 1e-12), |x| {
        let ht = (p0 - 2.0 * x) / w;
        let n3 = (n0 - 3.0 * x) / w;
        let y = hno2_molal(ht, n3, ka);
        let h = ht - y;
        let p = pressure(x);
        let q = h * ((a0 + x) / w) * p * p / (y * y * y);
        q.ln() - ln_k
    })
}

fn decompose(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let x = decomposition_extent(vessel);
    if x <= tolerance(nitrite(vessel)) {
        return false;
    }
    // Protons first, while the charge still counts the nitrite.
    refresh_charge(vessel);
    crate::displacement::consume_acidity(vessel, 2.0 * x);
    take_nitrite(vessel, 3.0 * x);
    vessel.deposit(SpeciesId::new(NITRATE), Moles(x), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("water"), Moles(x), Phase::Liquid);
    refresh_charge(vessel);
    let n = Moles(2.0 * x);
    if vessel.retain_gas(SpeciesId::new(NO), n) {
        events.push(Event::GasContained {
            vessel: vessel.id,
            species: SpeciesId::new(NO),
            moles: n,
        });
    } else {
        events.push(Event::GasEvolved {
            vessel: vessel.id,
            species: SpeciesId::new(NO),
            moles: n,
        });
    }
    if x > crate::OBSERVABLE_MOLES {
        reaction(events, vessel, "3 HNO₂ → HNO₃ + 2 NO + H₂O");
    }
    note(
        events,
        vessel,
        Phrase::new(
            "not-modeled.nitrous-acid-decomposition",
            "nitrous acid (pKa 3.2) is unstable: {amount} mol of it decomposed to nitric acid and nitrogen monoxide (3 HNO₂ → HNO₃ + 2 NO + H₂O), solved to its NBS constant against the nitrogen monoxide pressure (one atmosphere in an open vessel) with molality standing in for activity; the real decomposition takes minutes to hours and no rate or reaction heat is claimed",
            vec![("amount".to_string(), Slot::number(format!("{:.3e}", 3.0 * x)))],
        ),
    );
    true
}

// ── dimer note ───────────────────────────────────────────────────────

fn dimer_note(vessel: &Vessel, events: &mut Vec<Event>) {
    if let Some(share) = dimer_share(vessel) {
        if gas(vessel, NO2) > crate::OBSERVABLE_MOLES {
            note(
                events,
                vessel,
                Phrase::new(
                    "not-modeled.nitrogen-dioxide-dimer-pool",
                    "nitrogen dioxide in the headspace is in equilibrium with colourless dinitrogen tetroxide (2 NO₂ ⇌ N₂O₄, NBS constant, van 't Hoff from 25 °C): {share} of it is the dimer at this temperature, more when colder, so the brown fades on cooling; the gas is held as one NO₂ pool, the shown pressure counts it as monomer, and the dimerisation heat and any condensation of N₂O₄ (boiling point 21 °C) are not modelled",
                    vec![("share".to_string(), Slot::number(format!("{:.0} %", share * 100.0)))],
                ),
            );
        }
    }
}

/// Whether any rule would act.
pub fn fires(vessel: &Vessel) -> bool {
    oxidation_extent(vessel) > TRACE
        || absorption_extent(vessel) > tolerance(gas(vessel, NO2))
        || decomposition_extent(vessel) > tolerance(nitrite(vessel))
}

/// Run every rule once, in order. Returns whether anything happened; the
/// caller refreshes `solute_charge`.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut progressed = false;
    progressed |= oxidise(vessel, events);
    progressed |= absorb(vessel, events);
    progressed |= decompose(vessel, events);
    dimer_note(vessel, events);
    progressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_the_nbs_derivations() {
        let (g, h) = dimerisation_kj();
        assert!(
            (g + 4.73).abs() < 1e-9 && (h + 57.20).abs() < 1e-9,
            "{g} {h}"
        );
        // Kp(298) about 6.7 bar⁻¹.
        let kp = dimerisation_kp(298.15);
        assert!((kp - 6.74).abs() < 0.05, "{kp}");
        let (g, _) = oxidation_kj();
        assert!((g + 70.48).abs() < 1e-9, "{g}");
        let (g, h) = absorption_kj();
        assert!(
            (g + 24.831).abs() < 1e-9 && (h + 104.73).abs() < 1e-9,
            "{g} {h}"
        );
        let (g, h) = decomposition_kj();
        assert!(
            (g + 20.969).abs() < 1e-9 && (h - 47.27).abs() < 1e-9,
            "{g} {h}"
        );
        // pKa 3.22, matching pinned llnl.dat log_k 3.2206.
        let pka = -nitrous_acid_ka(298.15).log10();
        assert!((pka - 3.2206).abs() < 0.01, "{pka}");
    }

    #[test]
    fn exothermic_dimerisation_favours_the_dimer_when_cold() {
        assert!(dimerisation_kp(273.15) > 5.0 * dimerisation_kp(298.15));
        let warm = monomer_moles(0.02, 323.15, 1.0) / 0.02;
        let cold = monomer_moles(0.02, 273.15, 1.0) / 0.02;
        assert!(cold < warm, "{cold} {warm}");
        // The split closes: n_M + 2 n_D = pool with n_D/n_M² = Kp R T / V.
        let (pool, t, v) = (0.02, 298.15, 1.0);
        let m = monomer_moles(pool, t, v);
        let d = (pool - m) / 2.0;
        let k = dimerisation_kp(t) * R_L_BAR * t / v;
        assert!((d / (m * m) - k).abs() / k < 1e-9);
    }

    #[test]
    fn nitrous_acid_split_solves_its_quadratic() {
        let ka = nitrous_acid_ka(298.15);
        let (ht, n3) = (0.01, 0.005);
        let y = hno2_molal(ht, n3, ka);
        let h = ht - y;
        assert!(((n3 - y) * h / y - ka).abs() / ka < 1e-9);
        assert_eq!(hno2_molal(0.0, n3, ka), 0.0);
    }
}
