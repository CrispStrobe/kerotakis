//! S(IV) reductant coupling: sulfur dioxide, hydrogen sulfite and sulfite
//! as a reductant pool that iodine and hydrogen peroxide oxidise to
//! sulfate.
//!
//! KI-011 found the gap: thiosulfate's acid decomposition
//! (`S₂O₃²⁻ + 2 H⁺ → S + SO₂ + H₂O`) left its SO₂ beside iodine and
//! hydrogen peroxide for 100 s. Both oxidise S(IV) in milliseconds, so an
//! unreacted S(IV) pool next to either is not a state a beaker can hold,
//! and the decomposition is not a pure loss of sink capacity: each S(IV)
//! still reduces one I₂.
//!
//! One generic writer, for every S(IV) form in the vessel (`SO2` dissolved
//! or held in the vessel's own gas phase while it has liquid water, `HSO3-`,
//! `SO3-2`, and every Na/K salt of the ions the registry carries), against
//! the two oxidants:
//!
//! * iodine (I₂ and the I₃⁻ it forms with iodide):
//!   `S(IV) + I₂ (+ H₂O) → SO₄²⁻ + 2 I⁻ + n H⁺`. Second-order rate
//!   constants at 25.0 °C, μ = 0.50 M: I₂ + HSO₃⁻ 1.7·10⁹, I₃⁻ + HSO₃⁻
//!   1.5·10⁷, I₂ + SO₃²⁻ 3.1·10⁹, I₃⁻ + SO₃²⁻ 2.9·10⁸ M⁻¹ s⁻¹.
//!   Source: B. S. Yiin and D. W. Margerum, "Non-metal redox kinetics:
//!   reactions of iodine and triiodide with sulfite and hydrogen sulfite
//!   and the hydrolysis of iodosulfate", Inorg. Chem. 29 (1990) 1559-1564,
//!   doi:10.1021/ic00333a023 (values as quoted from the paper's abstract;
//!   the full text was not readable here).
//! * hydrogen peroxide: `S(IV) + H₂O₂ → SO₄²⁻ + n H⁺ (+ H₂O)`, with the
//!   McArdle-Hoffmann law
//!   `rate = k₁ [H₂O₂][HSO₃⁻] · R[H⁺] / (1 + R[H⁺])`,
//!   k₁ = 2.6·10⁶ M⁻¹ s⁻¹ and R = k₂/k₋₁ = 16 M⁻¹ at 15 °C,
//!   ΔH‡₁ = 37 kJ/mol, measured over pH 0.0-4.5. Source: J. V. McArdle
//!   and M. R. Hoffmann, "Kinetics and mechanism of the oxidation of
//!   aquated sulfur dioxide by hydrogen peroxide at low pH", J. Phys.
//!   Chem. 87 (1983) 5425-5429, doi:10.1021/j150644a024 (abstract,
//!   CaltechAUTHORS record 59779). The earlier acid-solution study is
//!   M. R. Hoffmann and J. O. Edwards, J. Phys. Chem. 79 (1975) 2096-2098,
//!   doi:10.1021/j100587a005.
//!
//! Both oxidations are treated as instantaneous on the bench clock (S(IV)
//! lifetime milliseconds at the concentrations the bench holds); the
//! peroxide path is admitted only when its pseudo-first-order lifetime is
//! at most [`MAX_INSTANT_S`] and inside the measured acidity range.
//!
//! When both oxidants are present they COMPETE for the S(IV). Both rate
//! laws are first order in the same hydrogen sulfite fraction α₁, so α₁
//! and the volume cancel, and with peroxide in excess (constant) the
//! competition integrates in closed form:
//!
//! `s₀ = (i₀ - i) + β ln(i₀ / i)`, `β = k_H·h₀ / k_I` (moles),
//!
//! where `i` is the iodine left when S(IV) is exhausted, `k_I` the
//! triiodide-weighted iodine constant (Palmer et al. 1984 K = 698 L/mol,
//! [`crate::iodine_chemistry::split`]) and `k_H` the McArdle-Hoffmann
//! constant at the vessel's acidity and temperature. Limitations stated in
//! the coverage table `docs/sulfur-iv-coupling-coverage-20261111.md`: the
//! I₂/I₃⁻ split is the one at the start of the coupling, a direct SO₂(aq)
//! + I₂ path is not separately priced, Yiin-Margerum constants are 25 °C
//! values, and thiosulfate's own iodine titration (the curated row) runs
//! first.
//!
//! Every gap is a typed refusal, never a silent pool:
//! `not-modeled.sulfite-peroxide-outside-measured-acid` and
//! `not-modeled.sulfite-peroxide-slow`.

use crate::ops::Event;
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

const TRACE: f64 = 1e-12;

/// Yiin & Margerum 1990, 25.0 °C, μ = 0.50 M (M⁻¹ s⁻¹): I₂ + HSO₃⁻.
pub const K_I2_HSO3: f64 = 1.7e9;
/// Yiin & Margerum 1990: I₃⁻ + HSO₃⁻.
pub const K_I3_HSO3: f64 = 1.5e7;
/// Yiin & Margerum 1990: I₂ + SO₃²⁻.
pub const K_I2_SO3: f64 = 3.1e9;
/// Yiin & Margerum 1990: I₃⁻ + SO₃²⁻.
pub const K_I3_SO3: f64 = 2.9e8;

/// McArdle & Hoffmann 1983: k₁ at 15 °C, M⁻¹ s⁻¹.
pub const MH_K1_M1_S1: f64 = 2.6e6;
/// McArdle & Hoffmann 1983: k₂/k₋₁, M⁻¹ (15 °C; held constant).
pub const MH_K2_OVER_KM1_M1: f64 = 16.0;
/// McArdle & Hoffmann 1983: ΔH‡ of k₁, J/mol.
pub const MH_DH_ACT_J_MOL: f64 = 37_000.0;
/// Temperature of the McArdle & Hoffmann constants, K.
pub const MH_T_K: f64 = 288.15;
/// Measured acidity range of the McArdle & Hoffmann law: pH 0.0-4.5,
/// as [H⁺] in mol/L (low, high).
pub const MH_PROTON_RANGE_M: (f64, f64) = (3.162_277_660_168_379_5e-5, 1.0);

/// Longest pseudo-first-order S(IV) lifetime (s) the peroxide path is
/// treated as instantaneous for. A modelling bound, not a constant of
/// nature: above it the coupling is refused rather than integrated.
pub const MAX_INSTANT_S: f64 = 1.0;

/// Aqueous bench range the 25 °C constants are used over (same as the
/// halogen writers).
pub const DOMAIN_K: (f64, f64) = crate::halogen_redox::DOMAIN_K;

const R_GAS: f64 = 8.314_462_618;

/// One S(IV) ion: its key, O and H counts and charge.
#[derive(Debug, Clone, Copy)]
pub struct SivIon {
    pub key: &'static str,
    pub oxygen: f64,
    pub hydrogen: f64,
    pub charge: f64,
}

/// The S(IV) members the writer owns.
pub const IONS: &[SivIon] = &[
    SivIon {
        key: "SO2",
        oxygen: 2.0,
        hydrogen: 0.0,
        charge: 0.0,
    },
    SivIon {
        key: "HSO3-",
        oxygen: 3.0,
        hydrogen: 1.0,
        charge: -1.0,
    },
    SivIon {
        key: "SO3-2",
        oxygen: 3.0,
        hydrogen: 0.0,
        charge: -2.0,
    },
];

/// Oxidants that take S(IV) to sulfate here.
pub const OXIDANTS: &[&str] = &["I2", "H2O2"];

/// Per mole of S(IV) of `ion` oxidised by I₂:
/// `(water consumed, protons released)`.
pub fn iodine_path(ion: &SivIon) -> (f64, f64) {
    (4.0 - ion.oxygen, 4.0 + ion.charge)
}

/// Per mole of S(IV) of `ion` oxidised by H₂O₂:
/// `(water released, protons released)`.
pub fn peroxide_path(ion: &SivIon) -> (f64, f64) {
    (ion.oxygen - 2.0, 2.0 + ion.charge)
}

/// Registry keys delivering each ion: `(form, ion)`.
pub fn forms() -> Vec<(String, &'static SivIon)> {
    let mut out = Vec::new();
    for ion in IONS {
        for form in crate::halogen_redox::registry_forms(ion.key) {
            out.push((form, ion));
        }
    }
    out
}

fn liquid_water(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

fn in_medium(vessel: &Vessel) -> bool {
    (DOMAIN_K.0..=DOMAIN_K.1).contains(&vessel.temperature.0) && liquid_water(vessel) > 0.0
}

/// Phases a form is drawn from. `SO2` booked as gas by a rate law (the
/// thiosulfate-acid law) is still held by the vessel over its liquid
/// water and counts; salts are drawn dissolved first, then solid.
fn phases_of(form: &str) -> &'static [Phase] {
    if form == "SO2" {
        &[Phase::Aqueous, Phase::Gas]
    } else {
        &[Phase::Aqueous, Phase::Solid]
    }
}

fn amount(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    let id = SpeciesId::new(key);
    let id = id.canonical();
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == id.as_ref() && phases.contains(&p.phase))
        .map(|p| p.moles.0.max(0.0))
        .sum()
}

/// S(IV) moles in this vessel that the writer can reach, per form.
pub fn pool(vessel: &Vessel) -> Vec<(String, &'static SivIon, f64)> {
    if !in_medium(vessel) {
        return Vec::new();
    }
    forms()
        .into_iter()
        .map(|(f, ion)| {
            let n = amount(vessel, &f, phases_of(&f));
            (f, ion, n)
        })
        .filter(|(_, _, n)| *n > TRACE)
        .collect()
}

/// Remaining S(IV) reducing capacity in moles of I₂ (two electrons per
/// S(IV)) — the sink capacity S(IV) adds beside thiosulfate.
pub fn reducing_capacity_i2_equivalents(vessel: &Vessel) -> f64 {
    pool(vessel).iter().map(|(_, _, n)| n).sum()
}

const IODINE_PHASES: &[Phase] = &[Phase::Aqueous, Phase::Solid];
const PEROXIDE_PHASES: &[Phase] = &[Phase::Liquid, Phase::Aqueous];

fn litres(vessel: &Vessel) -> f64 {
    vessel.liquid_volume().0.max(1e-9)
}

/// Free proton concentration (mol/L): the ledger's H⁺ row, or the
/// anion charge excess where released protons are booked as charge (the
/// halogen writers' convention).
fn proton_molar(vessel: &Vessel) -> f64 {
    let row = amount(vessel, "H+", &[Phase::Aqueous]);
    let charge = (-crate::displacement::solute_charge(vessel)).max(0.0);
    row.max(charge) / litres(vessel)
}

/// Whether the halogen table's admitted iodine/sulfite writers own this
/// ion when no peroxide competes (they hold I₂ + HSO₃⁻/SO₃²⁻ and their
/// salts; SO₂ has no half there).
fn halogen_owned(ion: &SivIon) -> bool {
    ion.key != "SO2"
}

/// McArdle-Hoffmann k₁ at `t_k` (Eyring with ΔH‡₁ from 15 °C).
pub fn mh_k1(t_k: f64) -> f64 {
    MH_K1_M1_S1 * (t_k / MH_T_K) * (-(MH_DH_ACT_J_MOL / R_GAS) * (1.0 / t_k - 1.0 / MH_T_K)).exp()
}

/// Effective second-order peroxide constant on HSO₃⁻ (M⁻¹ s⁻¹), or
/// `None` outside the measured acidity range.
pub fn peroxide_k(t_k: f64, proton_m: f64) -> Option<f64> {
    if !(MH_PROTON_RANGE_M.0..=MH_PROTON_RANGE_M.1).contains(&proton_m) {
        return None;
    }
    let r = MH_K2_OVER_KM1_M1 * proton_m;
    Some(mh_k1(t_k) * r / (1.0 + r))
}

/// Triiodide-weighted iodine constant on HSO₃⁻ (M⁻¹ s⁻¹) for this
/// vessel's iodine/iodide split.
pub fn iodine_k(vessel: &Vessel) -> f64 {
    match crate::iodine_chemistry::speciate(vessel) {
        Some(s) if s.iodine + s.triiodide > 0.0 => {
            (s.iodine * K_I2_HSO3 + s.triiodide * K_I3_HSO3) / (s.iodine + s.triiodide)
        }
        _ => K_I2_HSO3,
    }
}

/// Iodine left (mol) when `s0` mol of S(IV) has been shared between `i0`
/// mol of iodine and peroxide with competition parameter `beta` (mol),
/// from `s0 = (i0 - i) + beta ln(i0 / i)`.
pub fn iodine_left(s0: f64, i0: f64, beta: f64) -> f64 {
    if i0 <= 0.0 || s0 <= 0.0 {
        return i0.max(0.0);
    }
    if beta <= 0.0 {
        return (i0 - s0).max(0.0);
    }
    let f = |i: f64| (i0 - i) + beta * (i0 / i).ln() - s0;
    // f is decreasing in i, +inf at 0 and -s0 at i0.
    let (mut lo, mut hi) = ((i0 - s0).max(0.0).max(i0 * 1e-300), i0);
    if f(lo) <= 0.0 {
        return lo;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Why the peroxide path is not run in this vessel, if it is not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PeroxideGap {
    OutsideMeasuredAcid { proton_m: f64 },
    Slow { lifetime_s: f64 },
}

/// The planned split: S(IV) moles to iodine and to peroxide, and any
/// peroxide refusal.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub s_iv: f64,
    /// S(IV) moles the iodine share is spread over: all of it when
    /// peroxide competes, otherwise only the forms the halogen writers do
    /// not own.
    pub iodine_basis: f64,
    /// Whether the peroxide path is admitted (so this writer owns every
    /// S(IV) form).
    pub peroxide_competes: bool,
    pub to_iodine: f64,
    pub to_peroxide: f64,
    pub peroxide_gap: Option<PeroxideGap>,
}

/// Plan the coupling for this vessel, or `None` when there is no S(IV)
/// or no oxidant present.
pub fn plan(vessel: &Vessel) -> Option<Plan> {
    let pool = pool(vessel);
    let s0: f64 = pool.iter().map(|(_, _, n)| n).sum();
    if s0 <= TRACE {
        return None;
    }
    let i0 = amount(vessel, "I2", IODINE_PHASES);
    let h0 = amount(vessel, "H2O2", PEROXIDE_PHASES);
    if i0 <= TRACE && h0 <= TRACE {
        return None;
    }
    let mut gap = None;
    let mut k_h = 0.0;
    if h0 > TRACE {
        let proton = proton_molar(vessel);
        match peroxide_k(vessel.temperature.0, proton) {
            None => gap = Some(PeroxideGap::OutsideMeasuredAcid { proton_m: proton }),
            Some(k) => {
                let lifetime = 1.0 / (k * h0 / litres(vessel));
                if lifetime <= MAX_INSTANT_S {
                    k_h = k;
                } else {
                    gap = Some(PeroxideGap::Slow {
                        lifetime_s: lifetime,
                    });
                }
            }
        }
    }
    let competes = k_h > 0.0;
    let basis: f64 = if competes {
        s0
    } else {
        pool.iter()
            .filter(|(_, ion, _)| !halogen_owned(ion))
            .map(|(_, _, n)| n)
            .sum()
    };
    let (to_iodine, to_peroxide) = if !competes {
        (basis.min(i0), 0.0)
    } else if i0 <= TRACE {
        (0.0, s0.min(h0))
    } else {
        let beta = k_h * h0 / iodine_k(vessel);
        let di = i0 - iodine_left(s0, i0, beta);
        let dh = (s0 - di).max(0.0);
        if dh <= h0 {
            (di, dh)
        } else {
            // Peroxide not in excess after all: it is spent and the rest
            // goes to iodine as far as iodine reaches.
            ((s0 - h0).min(i0), h0)
        }
    };
    Some(Plan {
        s_iv: s0,
        iodine_basis: basis,
        peroxide_competes: competes,
        to_iodine,
        to_peroxide,
        peroxide_gap: gap,
    })
}

/// Whether the writer would change this vessel now.
pub fn fires(vessel: &Vessel) -> bool {
    plan(vessel).is_some_and(|p| p.to_iodine + p.to_peroxide > TRACE)
}

fn pretty(key: &str) -> String {
    crate::halogen_redox::pretty(key)
}

fn term(coef: f64, key: &str) -> String {
    if (coef - 1.0).abs() < 1e-9 {
        pretty(key)
    } else {
        format!("{} {}", coef.round() as i64, pretty(key))
    }
}

/// Display equation for one ion and oxidant.
pub fn equation(ion: &SivIon, oxidant: &str) -> String {
    let mut lhs = vec![pretty(oxidant), pretty(ion.key)];
    let mut rhs = Vec::new();
    if oxidant == "I2" {
        let (w, h) = iodine_path(ion);
        if w > 0.0 {
            lhs.push(term(w, "water"));
        }
        rhs.push(term(2.0, "I-"));
        rhs.push(pretty("SO4-2"));
        if h > 0.0 {
            rhs.push(term(h, "H+"));
        }
    } else {
        rhs.push(pretty("SO4-2"));
        let (w, h) = peroxide_path(ion);
        if h > 0.0 {
            rhs.push(term(h, "H+"));
        }
        if w > 0.0 {
            rhs.push(term(w, "water"));
        }
    }
    format!("{} → {}", lhs.join(" + "), rhs.join(" + "))
}

fn withdraw(vessel: &mut Vessel, key: &str, moles: f64, phases: &[Phase]) {
    let mut left = moles;
    for phase in phases {
        if left <= 0.0 {
            break;
        }
        left -= vessel
            .withdraw_phase(&SpeciesId::new(key), Moles(left), *phase)
            .0;
    }
}

/// Run the coupling to completion. Returns whether anything changed. The
/// caller refreshes `solute_charge`.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let Some(plan) = plan(vessel) else {
        return false;
    };
    if plan.to_iodine + plan.to_peroxide <= TRACE {
        return false;
    }
    let pool = pool(vessel);
    let s0 = plan.s_iv;
    let mut said: Vec<String> = Vec::new();
    for (form, ion, n) in pool {
        let share = n / s0;
        let iodine_share = if plan.peroxide_competes {
            share
        } else if halogen_owned(ion) || plan.iodine_basis <= 0.0 {
            0.0
        } else {
            n / plan.iodine_basis
        };
        let spectators = crate::halogen_redox::spectators_of(&form, ion.key);
        for (oxidant, x) in [
            ("I2", plan.to_iodine * iodine_share),
            ("H2O2", plan.to_peroxide * share),
        ] {
            if x <= TRACE {
                continue;
            }
            withdraw(vessel, &form, x, phases_of(&form));
            let (water_in, water_out, _protons) = if oxidant == "I2" {
                withdraw(vessel, "I2", x, IODINE_PHASES);
                let (w, h) = iodine_path(ion);
                (w, 0.0, h)
            } else {
                withdraw(vessel, "H2O2", x, PEROXIDE_PHASES);
                let (w, h) = peroxide_path(ion);
                (0.0, w, h)
            };
            if water_in > 0.0 {
                vessel.withdraw_phase(&SpeciesId::new("water"), Moles(x * water_in), Phase::Liquid);
            }
            if water_out > 0.0 {
                vessel.deposit(SpeciesId::new("water"), Moles(x * water_out), Phase::Liquid);
            }
            vessel.deposit(SpeciesId::new("SO4-2"), Moles(x), Phase::Aqueous);
            if oxidant == "I2" {
                vessel.deposit(SpeciesId::new("I-"), Moles(2.0 * x), Phase::Aqueous);
            }
            // Released protons stay in the charge balance (the ledger's
            // strong-acid convention, as in `halogen_redox`); `protons` is
            // the equation's count and is checked by the closure test.
            for (cation, k) in &spectators {
                vessel.deposit(SpeciesId::new(*cation), Moles(x * k), Phase::Aqueous);
            }
            let eq = equation(ion, oxidant);
            if !said.contains(&eq) {
                events.push(Event::ReactionOccurred {
                    vessel: vessel.id,
                    equation: eq.clone(),
                });
                said.push(eq);
            }
        }
    }
    true
}

/// Whether this module covers the halogen table's `oxidant + reductant`
/// group (members as the table spells them).
pub fn covers(oxidant: &str, reductant: &str) -> bool {
    OXIDANTS.contains(&oxidant) && IONS.iter().any(|i| i.key == reductant)
}

/// Typed notes for S(IV) left beside peroxide that the writer refuses.
pub fn unpriced_notes(vessel: &Vessel) -> Vec<Event> {
    use crate::phrase::{Phrase, Slot};
    let Some(plan) = plan(vessel) else {
        return Vec::new();
    };
    let left = plan.s_iv - plan.to_iodine - plan.to_peroxide;
    let mut events = Vec::new();
    match plan.peroxide_gap {
        Some(PeroxideGap::OutsideMeasuredAcid { proton_m }) => {
            events.push(Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::NoReviewedDatum,
                Phrase::new(
                    "not-modeled.sulfite-peroxide-outside-measured-acid",
                    "sulfur(IV) and hydrogen peroxide are together here, but the reviewed rate law for this oxidation was measured between pH 0 and 4.5 and here (H+) is {proton} mol/L, so the peroxide oxidation of {amount} mol of sulfur(IV) is not priced and it is left unreacted. That is a gap in the model, not evidence that nothing happens",
                    vec![
                        ("proton".to_string(), Slot::text(format!("{proton_m:.3e}"))),
                        ("amount".to_string(), Slot::text(format!("{left:.3e}"))),
                    ],
                ),
            ));
        }
        Some(PeroxideGap::Slow { lifetime_s }) => {
            events.push(Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::ModelBoundary,
                Phrase::new(
                    "not-modeled.sulfite-peroxide-slow",
                    "sulfur(IV) and hydrogen peroxide are together here, but at these concentrations the reviewed rate law gives sulfur(IV) a lifetime of about {lifetime} s, too slow to treat as instantaneous, and no time-resolved writer integrates it, so {amount} mol of sulfur(IV) is left unreacted",
                    vec![
                        ("lifetime".to_string(), Slot::text(format!("{lifetime_s:.3e}"))),
                        ("amount".to_string(), Slot::text(format!("{left:.3e}"))),
                    ],
                ),
            ));
        }
        None => {}
    }
    if left <= TRACE {
        events.clear();
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formula(key: &str) -> crate::stoich::Formula {
        let text = if key == "water" { "H2O" } else { key };
        crate::stoich::parse_formula(text).expect(key)
    }

    fn closes(left: &[(&str, f64)], right: &[(&str, f64)]) -> bool {
        let mut el: std::collections::BTreeMap<String, f64> = Default::default();
        let mut charge = 0.0;
        for (side, sign) in [(left, -1.0), (right, 1.0)] {
            for (k, n) in side {
                let f = formula(k);
                for (e, c) in f.counts {
                    *el.entry(e).or_default() += sign * n * c;
                }
                charge += sign * n * f.charge;
            }
        }
        charge.abs() < 1e-9 && el.values().all(|v| v.abs() < 1e-9)
    }

    #[test]
    fn every_s_iv_path_closes_atoms_and_charge() {
        for ion in IONS {
            let (w, h) = iodine_path(ion);
            assert!(closes(
                &[(ion.key, 1.0), ("I2", 1.0), ("water", w)],
                &[("SO4-2", 1.0), ("I-", 2.0), ("H+", h)]
            ));
            let (w, h) = peroxide_path(ion);
            assert!(closes(
                &[(ion.key, 1.0), ("H2O2", 1.0)],
                &[("SO4-2", 1.0), ("H+", h), ("water", w)]
            ));
            let f = formula(ion.key);
            assert_eq!(f.counts.get("O").copied().unwrap_or(0.0), ion.oxygen);
            assert_eq!(f.counts.get("H").copied().unwrap_or(0.0), ion.hydrogen);
            assert_eq!(f.charge, ion.charge);
        }
    }

    #[test]
    fn competition_root_closes_the_integrated_balance() {
        let (s0, i0, beta) = (2.5e-5, 1.5e-5, 1.7e-5);
        let i = iodine_left(s0, i0, beta);
        assert!(i > 0.0 && i < i0);
        let back = (i0 - i) + beta * (i0 / i).ln();
        assert!((back - s0).abs() < 1e-12 * s0.max(1.0) + 1e-18);
        assert!((iodine_left(1e-5, 3e-5, 0.0) - 2e-5).abs() < 1e-18);
    }

    #[test]
    fn peroxide_constant_is_the_measured_one_at_15_c() {
        let k = peroxide_k(MH_T_K, 0.1).unwrap();
        assert!((k - 2.6e6 * 1.6 / 2.6).abs() < 1e-6 * k);
        assert!(peroxide_k(298.15, 1e-6).is_none());
    }
}
