//! The metallic state: displacement, the activity series, and the
//! potential a metal electrode pins (PLAN.md, P3e / L3e).
//!
//! A metal in a beaker used to be booked as its cation the moment it
//! touched water, so two moles of electrons per mole of magnesium ribbon
//! ceased to exist before any electron balance saw them, and the most
//! famous displacement reaction in school chemistry came back as copper
//! *hydroxide*. Commit 17bea5b made the bench say so. This module makes
//! that announcement obsolete.
//!
//! # Why an own module rather than PHREEQC phases
//!
//! Checked before designing (2026-08-20, `grep -A3 -E '^(Ag|Cu|Zn|Fe|Mg)Metal'`
//! over `vendor/iphreeqc/database/*.dat` — a first grep for "Silver" and
//! "Copper" missed them, and a peer's fuzz test found them the hard way):
//! wateq4f defines `AgMetal`, `CuMetal` (on the Cu⁺ couple) and `ZnMetal`;
//! it has no iron and no magnesium; minteq.v4, pitzer and phreeqc.dat
//! define no metal phase at all. So `EQUILIBRIUM_PHASES` could carry three
//! metals on one route, and not the magnesium ribbon the flagship reaction
//! is built on, nor the iron that zinc protects. The metallic state is
//! therefore this lab's own model, stated as such — and the database's
//! own `log_k` for the metals it *does* know is the independent check on
//! it (tests/displacement.rs: agreement to within a millivolt).
//!
//! # The model, and its domain
//!
//! Each couple `Ox + n e⁻ → Red` carries a curated standard potential E°
//! with its source. Two couples react when the oxidant's Nernst potential
//! is above the reductant's:
//!
//! ```text
//! E = E° + (RT ln 10 / nF) · log₁₀ a(Ox)      (metal: a(Red) = 1)
//! ```
//!
//! and the extent is found by bisection on the cell potential until either
//! it closes to zero (a genuine Nernst equilibrium, for couples that sit
//! close together) or a reagent runs out (the school case: Mg/Cu²⁺ is
//! 2.7 V apart, K ≈ 10⁹², and it goes to the last ion). The activities are
//! the ones PHREEQC has just computed for this vessel — the free-ion
//! activity, which is what Nernst wants — with molality standing in only
//! where the speciation has not named the ion.
//!
//! What this does **not** model, said here and in the beaker where it
//! bites: kinetics (a passivated aluminium sheet, the slow reaction of
//! magnesium with water itself), oxidising acids (nitric acid dissolves
//! copper by a different couple), overpotentials, and air oxidising a metal
//! that is merely sitting in solution. E° values are taken at 25 °C and
//! used at the vessel temperature without a dE°/dT correction.
//!
//! Heat comes from the same data a thermochemistry table would use: the
//! standard enthalpies of formation of the aqueous ions, metals and H₂ at
//! zero. Zinc into copper sulfate then releases the textbook −217 kJ/mol
//! as a computed difference, not a number written next to the reaction.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Kelvin, Moles};
use crate::vessel::{ThermalMode, Vessel};

/// Charge on a mole of electrons, C/mol.
///
/// Exact since the 2019 SI redefinition — the elementary charge and the
/// mole are both defined, so this is a product of two definitions rather
/// than a measurement, unlike every other constant in this file.
pub const FARADAY: f64 = 96_485.332_12;
const GAS_CONSTANT: f64 = 8.314_462_618;
/// Below this, an amount is bookkeeping noise rather than a reagent.
const TRACE: f64 = 1e-12;
/// The acid pseudo-species. Free acid has no portion of its own in the
/// inventory: it is the vessel's unspent acidity, `−solute_charge`.
const HYDROGEN_ION: &str = "H+";
const HYDROGEN_GAS: &str = "H2";

/// One redox couple `ν Ox + n e⁻ → Red`, with the data that makes it
/// computable and the provenance that makes it checkable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Couple {
    /// Registry key of the oxidised member, dissolved (`"Cu+2"`), or
    /// `"H+"` for the acid couple.
    pub oxidised: &'static str,
    /// How many of the oxidised species per reduced one: 2 for 2 H⁺ → H₂.
    pub oxidised_per_reduced: f64,
    /// Registry key of the reduced member: a metal, or hydrogen gas.
    pub reduced: &'static str,
    pub reduced_phase: Phase,
    /// Electrons transferred per reduced member formed.
    pub electrons: f64,
    /// Standard reduction potential, V vs SHE, 25 °C.
    pub e0_volts: f64,
    /// Standard enthalpy of formation of the oxidised member, kJ/mol
    /// (aqueous ion, infinite dilution). The reduced member — a metal in
    /// its standard state, or H₂ — is zero by definition.
    pub dfh_oxidised_kj: f64,
    pub source: &'static str,
}

const CRC: &str = "E°: CRC Handbook of Chemistry and Physics, 'Electrochemical Series' (Vanýsek), 25 °C vs SHE; ΔfH°(aq ion): NBS Tables of Chemical Thermodynamic Properties (Wagman et al., 1982)";

/// The activity series this lab computes with, most noble first.
///
/// The ordering is what the German curriculum calls *die Spannungsreihe*.
/// Where a shipped dataset has an opinion it agrees: wateq4f's `ZnMetal`
/// and `AgMetal` reproduce these E° to < 1 mV, and llnl.dat (not shipped)
/// gives the same order for all five with values 40–70 mV lower on its own
/// O₂ convention. The values used are the CRC ones because that is the
/// table a school checks against.
pub const SERIES: &[Couple] = &[
    Couple {
        oxidised: "Ag+",
        oxidised_per_reduced: 1.0,
        reduced: "Ag",
        reduced_phase: Phase::Solid,
        electrons: 1.0,
        e0_volts: 0.7996,
        dfh_oxidised_kj: 105.579,
        source: CRC,
    },
    Couple {
        oxidised: "Cu+2",
        oxidised_per_reduced: 1.0,
        reduced: "Cu",
        reduced_phase: Phase::Solid,
        electrons: 2.0,
        e0_volts: 0.3419,
        dfh_oxidised_kj: 64.77,
        source: CRC,
    },
    Couple {
        oxidised: HYDROGEN_ION,
        oxidised_per_reduced: 2.0,
        reduced: HYDROGEN_GAS,
        reduced_phase: Phase::Gas,
        electrons: 2.0,
        e0_volts: 0.0,
        dfh_oxidised_kj: 0.0,
        source:
            "the standard hydrogen electrode, E° = 0 by definition; ΔfH°(H⁺, aq) = 0 by convention",
    },
    Couple {
        oxidised: "Pb+2",
        oxidised_per_reduced: 1.0,
        reduced: "Pb",
        reduced_phase: Phase::Solid,
        electrons: 2.0,
        e0_volts: -0.1262,
        dfh_oxidised_kj: -1.7,
        source: CRC,
    },
    Couple {
        oxidised: "Fe+2",
        oxidised_per_reduced: 1.0,
        reduced: "Fe",
        reduced_phase: Phase::Solid,
        electrons: 2.0,
        e0_volts: -0.447,
        dfh_oxidised_kj: -89.1,
        source: CRC,
    },
    Couple {
        oxidised: "Zn+2",
        oxidised_per_reduced: 1.0,
        reduced: "Zn",
        reduced_phase: Phase::Solid,
        electrons: 2.0,
        e0_volts: -0.7618,
        dfh_oxidised_kj: -153.89,
        source: CRC,
    },
    Couple {
        oxidised: "Mg+2",
        oxidised_per_reduced: 1.0,
        reduced: "Mg",
        reduced_phase: Phase::Solid,
        electrons: 2.0,
        e0_volts: -2.372,
        dfh_oxidised_kj: -466.85,
        source: CRC,
    },
];

/// Hydrogen overpotential on each metal, V — the extra push hydrogen gas
/// needs before it will form on that surface, at a bench-scale current
/// density (~1 mA/cm²).
///
/// This is a curated claim about *rates*, in the same spirit as
/// `FAST_REDOX` in the aqueous engine, and it is the reason real
/// batteries exist: thermodynamics says lead should fizz its way out of
/// sulfuric acid, and a lead-acid accumulator sits in a car for years
/// because hydrogen on lead costs 0.88 V that the 0.13 V driving force
/// cannot pay. For the metals this lab already has it changes no outcome
/// — magnesium reacts hard, zinc and iron in strong acid clear the barrier
/// by a few hundredths of a volt (which is why they fizz so much less
/// enthusiastically than magnesium), copper and silver were inert
/// already — and that is the point: the model earns its place by
/// predicting the five outcomes already observed and getting the margins
/// right.
///
/// Its limit, stated: overpotential depends on current density and the
/// bench has none, so one number per metal can say "blocked on the
/// timescale of a lesson" and "marginal"; it cannot say "four hours".
pub const HYDROGEN_OVERPOTENTIAL: &[(&str, f64)] = &[
    // ESTIMATE. Magnesium corrodes too fast to hold a steady Tafel line
    // and is not reliably tabulated at all; its driving force is 2.37 V
    // against any plausible barrier, so nothing turns on this number.
    ("Mg", 0.70),
    ("Fe", 0.40),
    ("Cu", 0.60),
    // Compilations spread 0.30–0.75 V. Silver is thermodynamically inert
    // in acid regardless, so no outcome depends on which is right.
    ("Ag", 0.48),
    ("Zn", 0.72),
    ("Pb", 0.88),
];

/// Where the overpotentials come from — stated as what it is. A metal not
/// in the table gets no barrier, which is the honest default: no claim.
pub const HYDROGEN_OVERPOTENTIAL_SOURCE: &str = "Hydrogen overpotentials at roughly 1 mA/cm² on the bare metal in acid, as commonly tabulated in electrochemistry texts. Values vary between compilations by 0.1-0.2 V and rise with current density — lead is quoted anywhere from 0.5 to 1.1 V — so these are the right order and the right ORDERING, not measurements. Curated, uncited, and labelled as such because no primary source was consulted.";

/// Below this margin over the barrier the reaction runs, but how fast is
/// a question the bench cannot answer, and it says so.
const MARGINAL_VOLTS: f64 = 0.1;

pub fn hydrogen_overpotential(metal: &str) -> f64 {
    HYDROGEN_OVERPOTENTIAL
        .iter()
        .find(|(m, _)| *m == metal)
        .map(|(_, eta)| *eta)
        .unwrap_or(0.0)
}

/// The couple a metal (or hydrogen) is the reduced member of.
pub fn couple_of_metal(key: &str) -> Option<&'static Couple> {
    SERIES.iter().find(|c| c.reduced == key)
}

/// The couple a dissolved ion is the oxidised member of.
pub fn couple_of_ion(key: &str) -> Option<&'static Couple> {
    SERIES.iter().find(|c| c.oxidised == key)
}

/// A registry species that is one metallic element, solid, and nothing
/// else — a ribbon of magnesium rather than a salt of it. Oxidation state
/// zero by definition, which is exactly what booking it as its cation
/// loses.
///
/// Carbon and sulfur are single-element solids too, and are not metals;
/// they are excluded by name because "metal" is not something a formula
/// can tell you.
pub fn is_elemental_metal(key: &str) -> bool {
    const NONMETALS: &[&str] = &["C", "S", "P", "I", "B", "Se", "Si", "Br"];
    let Some(d) = species::lookup_key(key) else {
        return false;
    };
    if d.standard_phase != Phase::Solid {
        return false;
    }
    crate::stoich::parse_formula(d.formula)
        .map(|f| {
            f.counts.len() == 1
                && f.counts.values().all(|n| *n == 1.0)
                && f.charge == 0.0
                && !f.counts.keys().any(|el| NONMETALS.contains(&el.as_str()))
        })
        .unwrap_or(false)
}

/// RT ln10 / F at this temperature: 0.05916 V at 25 °C.
fn nernst_slope(t: Kelvin) -> f64 {
    GAS_CONSTANT * t.0 * std::f64::consts::LN_10 / FARADAY
}

fn moles_in(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn kgw(vessel: &Vessel) -> f64 {
    const WATER_MOLAR_MASS: f64 = 18.015;
    moles_in(vessel, "water", Phase::Liquid) * WATER_MOLAR_MASS / 1000.0
}

/// Σ z·n over the dissolved portions — the same quantity the aqueous
/// solver carries as the vessel's unspent acidity.
fn solute_charge(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Aqueous)
        .filter_map(|p| {
            let d = species::lookup(&p.species)?;
            let f = crate::stoich::parse_formula(d.formula).ok()?;
            Some(f.charge * p.moles.0)
        })
        .sum()
}

/// How much of an oxidant is there to be reduced, in moles of it.
fn oxidant_available(vessel: &Vessel, c: &Couple) -> f64 {
    if c.oxidised == HYDROGEN_ION {
        (-vessel.solute_charge).max(0.0)
    } else {
        moles_in(vessel, c.oxidised, Phase::Aqueous)
    }
}

/// The activity of a dissolved species, and whether it came from the
/// solver's speciation or is molality standing in for it.
fn activity_of(vessel: &Vessel, key: &str) -> Option<(f64, bool)> {
    if key == HYDROGEN_ION {
        // The solver's pH is the measurement; the unspent acidity is only
        // an amount.
        if let Some(info) = &vessel.solution {
            return Some((10f64.powf(-info.ph), true));
        }
        let w = kgw(vessel);
        return (w > 0.0).then(|| (oxidant_available(vessel, &SERIES[2]) / w, false));
    }
    let formula = species::lookup_key(key).map(|d| d.formula).unwrap_or(key);
    if let Some(s) = vessel
        .solution
        .as_ref()
        .and_then(|info| info.species.iter().find(|s| s.name == formula))
    {
        if s.activity > 0.0 {
            return Some((s.activity, true));
        }
    }
    let w = kgw(vessel);
    (w > 0.0).then(|| (moles_in(vessel, key, Phase::Aqueous) / w, false))
}

/// Activity coefficient for scaling a changing amount: γ = a/m where the
/// speciation gives both, else 1.
fn gamma_of(vessel: &Vessel, key: &str) -> f64 {
    let w = kgw(vessel);
    let m = if key == HYDROGEN_ION {
        oxidant_available(vessel, &SERIES[2]) / w
    } else {
        moles_in(vessel, key, Phase::Aqueous) / w
    };
    match activity_of(vessel, key) {
        Some((a, true)) if m > 0.0 && a > 0.0 => a / m,
        _ => 1.0,
    }
}

/// One displacement that happened, for the record and the equation.
#[derive(Debug, Clone, PartialEq)]
pub struct Displacement {
    pub reductant: &'static Couple,
    pub oxidant: &'static Couple,
    /// Moles of electrons transferred.
    pub electrons: f64,
    /// Whether it stopped at a Nernst equilibrium (true) or because a
    /// reagent ran out (false).
    pub equilibrium: bool,
    /// Heat released into the vessel, J (positive = exothermic).
    pub heat_joules: f64,
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

fn coefficient(n: u64) -> String {
    if n == 1 {
        String::new()
    } else {
        format!("{n} ")
    }
}

impl Displacement {
    /// The balanced equation, as the symbols a learner writes.
    pub fn equation(&self) -> String {
        let (r, o) = (self.reductant, self.oxidant);
        let (nr, no) = (r.electrons as u64, o.electrons as u64);
        let l = nr * no / gcd(nr, no);
        // l electrons in total: l/nr of the reductant, l/no of the oxidant.
        let red_units = l / nr;
        let ox_units = l / no;
        let nu_r = r.oxidised_per_reduced as u64;
        let nu_o = o.oxidised_per_reduced as u64;
        let arrow = if self.equilibrium { "⇌" } else { "→" };
        format!(
            "{}{} + {}{} {arrow} {}{} + {}{}",
            coefficient(red_units),
            r.reduced,
            coefficient(ox_units * nu_o),
            o.oxidised,
            coefficient(red_units * nu_r),
            r.oxidised,
            coefficient(ox_units),
            o.reduced,
        )
    }
}

/// Cell potential between an oxidant couple and a reductant couple after
/// `xi` moles of electrons have moved, V. Positive means the reaction
/// still wants to go.
#[allow(clippy::too_many_arguments)]
fn cell_potential(
    slope: f64,
    ox: &Couple,
    red: &Couple,
    xi: f64,
    w: f64,
    ox_n0: f64,
    ox_gamma: f64,
    red_ion_n0: f64,
    red_gamma: f64,
) -> f64 {
    let ox_left = (ox_n0 - ox.oxidised_per_reduced * xi / ox.electrons).max(0.0);
    let red_ion = (red_ion_n0 + red.oxidised_per_reduced * xi / red.electrons).max(0.0);
    let a_ox = (ox_gamma * ox_left / w).max(f64::MIN_POSITIVE);
    let a_red = (red_gamma * red_ion / w).max(f64::MIN_POSITIVE);
    let e_ox = ox.e0_volts + slope / ox.electrons * ox.oxidised_per_reduced * a_ox.log10();
    let e_red = red.e0_volts + slope / red.electrons * red.oxidised_per_reduced * a_red.log10();
    e_ox - e_red
}

/// Run every displacement the series allows, strongest reductant against
/// strongest oxidant first, until no pair has a positive cell potential.
///
/// Sequential pairwise equilibria rather than one joint minimisation:
/// exact when the couples involved are well separated (every pair in
/// `SERIES` is ≥ 0.3 V apart, K ≥ 10¹⁰), and stated as the approximation
/// it would be for a closer pair.
pub fn displace(vessel: &mut Vessel) -> (Vec<Event>, Vec<Displacement>) {
    let mut events = Vec::new();
    let mut done: Vec<Displacement> = Vec::new();
    let w = kgw(vessel);
    if w <= 0.0 {
        return (events, done);
    }
    let slope = nernst_slope(vessel.temperature);
    // Activity coefficients are read once, from the speciation the solver
    // produced for the state as it stood. The first transfer makes that
    // speciation stale, and dividing a stale activity by a fresh molality
    // manufactured a γ of 10⁸ for the last trace of silver — which then
    // reacted a second time on paper.
    let gammas: Vec<f64> = SERIES
        .iter()
        .map(|c| gamma_of(vessel, c.oxidised))
        .collect();
    let gamma = |key: &str| {
        SERIES
            .iter()
            .position(|c| c.oxidised == key)
            .map(|i| gammas[i])
            .unwrap_or(1.0)
    };
    // A pair that has reached its Nernst root is settled for this pass;
    // asking it again would only chase the bisection's own residual.
    let mut settled: Vec<(&str, &str)> = Vec::new();

    for _ in 0..SERIES.len() * SERIES.len() {
        // The most reactive metal that is actually there.
        let Some(red) = SERIES
            .iter()
            .filter(|c| c.reduced_phase == Phase::Solid)
            .filter(|c| moles_in(vessel, c.reduced, Phase::Solid) > TRACE)
            .min_by(|a, b| a.e0_volts.total_cmp(&b.e0_volts))
        else {
            break;
        };
        // The most noble oxidant present that sits above it.
        let Some(ox) = SERIES
            .iter()
            .filter(|c| c.e0_volts > red.e0_volts)
            .filter(|c| !settled.contains(&(red.reduced, c.oxidised)))
            .filter(|c| oxidant_available(vessel, c) > TRACE)
            .max_by(|a, b| a.e0_volts.total_cmp(&b.e0_volts))
        else {
            break;
        };

        let ox_n0 = oxidant_available(vessel, ox);
        let red_n0 = moles_in(vessel, red.reduced, Phase::Solid);
        let red_ion_n0 = moles_in(vessel, red.oxidised, Phase::Aqueous);
        let ox_gamma = gamma(ox.oxidised);
        let red_gamma = gamma(red.oxidised);
        // Electron capacity of each side.
        let xi_max = (ox_n0 * ox.electrons / ox.oxidised_per_reduced).min(red_n0 * red.electrons);
        if xi_max <= TRACE {
            break;
        }
        let f = |xi: f64| {
            cell_potential(
                slope, ox, red, xi, w, ox_n0, ox_gamma, red_ion_n0, red_gamma,
            )
        };
        // Already at equilibrium (or past it) before anything moves.
        if f(0.0) <= 0.0 {
            settled.push((red.reduced, ox.oxidised));
            continue;
        }
        // Hydrogen has to *form* on the metal, and that costs an
        // overpotential the thermodynamics knows nothing about. The
        // driving force is the Nernst cell potential as the beaker stands
        // — vinegar at pH 3 puts the hydrogen line at −0.18 V, which is
        // why zinc in vinegar is an overnight job — with the metal's own
        // ion at its activity where it is present and at unit activity
        // where it is not, since a metal freshly dropped in has no ion to
        // speak of and −∞ is not a driving force.
        if ox.oxidised == HYDROGEN_ION {
            let eta = hydrogen_overpotential(red.reduced);
            let (a_h, _) = activity_of(vessel, HYDROGEN_ION).unwrap_or((1.0, false));
            let e_h = ox.e0_volts + slope * a_h.max(f64::MIN_POSITIVE).log10();
            let a_red = if red_ion_n0 > crate::OBSERVABLE_MOLES {
                red_gamma * red_ion_n0 / w
            } else {
                1.0
            };
            let e_red = red.e0_volts + slope / red.electrons * a_red.log10();
            let driving = e_h - e_red;
            let name = species::lookup_key(red.reduced)
                .map(|d| d.name)
                .unwrap_or(red.reduced);
            if driving <= eta {
                // Blocked by rate, not by thermodynamics — a different
                // sentence, because a learner needs to know which. The
                // codex entry `charging-fights-the-series` quotes the
                // sentence below verbatim as its thesis: reword it and the
                // entry's prose moves with it (its lint will say so).
                events.push(Event::Inert {
                    vessel: vessel.id,
                    species: SpeciesId::new(red.reduced),
                    why: format!(
                        "{name} should dissolve in this acid by the series (driving force {driving:+.2} V), but hydrogen has to form on {name}, and on that surface it costs an overpotential of about {eta:.2} V. Kinetically blocked on the timescale of a lesson, not thermodynamically inert — the difference between a bench and a battery"
                    ),
                });
                settled.push((red.reduced, ox.oxidised));
                continue;
            }
            if driving - eta < MARGINAL_VOLTS {
                events.push(Event::NotYetModeled {
                    vessel: vessel.id,
                    what: format!(
                        "how fast {name} fizzes: the driving force clears the hydrogen overpotential on {name} by only {:.2} V, and a rate that close to its barrier is not something this lab computes — it reacts, slowly",
                        driving - eta
                    ),
                });
            }
        }
        // Complete if the potential is still positive with the last
        // representable trace of reagent left; otherwise bisect for the
        // Nernst root.
        let xi = if f(xi_max * (1.0 - 1e-9)) > 0.0 {
            xi_max
        } else {
            let (mut lo, mut hi) = (0.0, xi_max);
            for _ in 0..200 {
                let mid = 0.5 * (lo + hi);
                if f(mid) > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
                if hi - lo < 1e-15 * xi_max {
                    break;
                }
            }
            0.5 * (lo + hi)
        };
        settled.push((red.reduced, ox.oxidised));
        // A computed transfer below the lab's material detection boundary is
        // not a reaction this model owns. In neutral brine, the Nernst root
        // can otherwise consume tens of picomoles of magnesium as though it
        // had displaced free acid, while the kinetic layer correctly says
        // that reaction with water is not modelled. Keep the metal unchanged
        // and let that explicit bystander diagnosis be the result.
        if xi <= crate::OBSERVABLE_MOLES {
            continue;
        }
        // "Equilibrium" is a claim a learner can test — both sides still
        // there in amounts that can be seen. Copper into silver nitrate
        // has a root too, leaving 6e-10 mol of silver in the glass, and
        // writing ⇌ over that teaches a hesitation the beaker does not
        // show. The trace is kept in the books; the arrow says what the
        // eye sees.
        let equilibrium = xi_max - xi >= crate::OBSERVABLE_MOLES;

        // ΔH per mole of electrons, from formation enthalpies: products
        // minus reactants, metals and H₂ at zero.
        let dh_per_electron_kj = red.dfh_oxidised_kj * red.oxidised_per_reduced / red.electrons
            - ox.dfh_oxidised_kj * ox.oxidised_per_reduced / ox.electrons;
        let heat_joules = -dh_per_electron_kj * 1000.0 * xi;

        // Enthalpy is balanced across the inventory change, not
        // temperature: the metal leaving takes its heat capacity with it
        // and the metal arriving brings its own.
        let t_ref = Kelvin::STANDARD.0;
        let cp_before = vessel.heat_capacity();
        let t0 = vessel.temperature.0;

        // Reductant: metal out, its ion in.
        let metal_gone = xi / red.electrons;
        vessel.withdraw(&SpeciesId::new(red.reduced), Moles(metal_gone));
        vessel.deposit(
            SpeciesId::new(red.oxidised),
            Moles(metal_gone * red.oxidised_per_reduced),
            Phase::Aqueous,
        );
        // Oxidant: ion out, metal (or gas) in.
        let reduced_made = xi / ox.electrons;
        if ox.oxidised == HYDROGEN_ION {
            // Free acid has no portion; it is spent through the charge
            // balance, which is recomputed below.
            let species = SpeciesId::new(ox.reduced);
            let moles = Moles(reduced_made);
            if vessel.retain_gas(species.clone(), moles) {
                events.push(Event::GasContained {
                    vessel: vessel.id,
                    species,
                    moles,
                });
            } else {
                events.push(Event::GasEvolved {
                    vessel: vessel.id,
                    species,
                    moles,
                });
            }
        } else {
            vessel.withdraw(
                &SpeciesId::new(ox.oxidised),
                Moles(reduced_made * ox.oxidised_per_reduced),
            );
            vessel.deposit(
                SpeciesId::new(ox.reduced),
                Moles(reduced_made),
                ox.reduced_phase,
            );
            events.push(Event::Plated {
                vessel: vessel.id,
                species: SpeciesId::new(ox.reduced),
                onto: SpeciesId::new(red.reduced),
                moles: Moles(reduced_made),
            });
        }
        // The acidity the next aqueous solve starts from is what the
        // inventory now says, so the acid a metal consumed is not
        // re-counted as a neutralisation.
        vessel.solute_charge = solute_charge(vessel);

        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
            let cp_after = vessel.heat_capacity();
            if cp_after > 0.0 {
                let t_new = t_ref + (cp_before * (t0 - t_ref) + heat_joules) / cp_after;
                vessel.temperature = Kelvin(t_new.max(0.0));
            }
        }

        let record = Displacement {
            reductant: red,
            oxidant: ox,
            electrons: xi,
            equilibrium,
            heat_joules,
        };
        events.insert(
            events.len() - 1,
            Event::ReactionOccurred {
                vessel: vessel.id,
                equation: record.equation(),
            },
        );
        events.insert(
            events.len() - 1,
            Event::Consumed {
                vessel: vessel.id,
                species: SpeciesId::new(red.reduced),
                moles: Moles(metal_gone),
                remaining: Some(Moles(moles_in(vessel, red.reduced, Phase::Solid))),
            },
        );
        done.push(record);
    }

    (events, done)
}

/// What the series has to say about metals that did *not* react.
///
/// Copper in dilute acid does nothing, and that is a computed result with
/// a reason, not a gap: it is said as `Inert`. Magnesium in brine also
/// does nothing here, but for a reason this lab does not compute — its
/// slow reaction with water itself — and that is said as `NotYetModeled`.
/// Conflating the two would be the silent-filter fault in a new coat.
pub fn bystanders(vessel: &Vessel, just_plated: &[&str]) -> Vec<Event> {
    let mut events = Vec::new();
    if kgw(vessel) <= 0.0 || vessel.solution.is_none() {
        // With no characterised solution the honesty pass already names
        // every solid in contact with liquid.
        return events;
    }
    let acid = oxidant_available(vessel, &SERIES[2]) > crate::OBSERVABLE_MOLES;
    for c in SERIES.iter().filter(|c| c.reduced_phase == Phase::Solid) {
        if moles_in(vessel, c.reduced, Phase::Solid) <= crate::OBSERVABLE_MOLES {
            continue;
        }
        // A metal that was plated out this very step has just been the
        // whole story; saying it also does nothing would be true and
        // bewildering.
        if just_plated.contains(&c.reduced) {
            continue;
        }
        let name = species::lookup_key(c.reduced)
            .map(|d| d.name)
            .unwrap_or(c.reduced);
        // The most noble ion of *another* metal in the glass, if any.
        let idle_against = SERIES
            .iter()
            .filter(|o| o.reduced_phase == Phase::Solid && o.reduced != c.reduced)
            .filter(|o| oxidant_available(vessel, o) > crate::OBSERVABLE_MOLES)
            .max_by(|a, b| a.e0_volts.total_cmp(&b.e0_volts));
        if acid && c.e0_volts > 0.0 {
            events.push(Event::Inert {
                vessel: vessel.id,
                species: SpeciesId::new(c.reduced),
                why: format!(
                    "{name} sits above hydrogen in the activity series (E° {:+.3} V against 0.000 V for 2H⁺/H₂), so dilute acid cannot take its electrons. An oxidising acid such as nitric would, by a different couple, and that is not modelled",
                    c.e0_volts
                ),
            });
        } else if let Some(o) = idle_against {
            // The series grid: which metal displaces which. The negative
            // cells are as much the result as the positive ones.
            let other = species::lookup_key(o.reduced)
                .map(|d| d.name)
                .unwrap_or(o.reduced);
            events.push(Event::Inert {
                vessel: vessel.id,
                species: SpeciesId::new(c.reduced),
                why: format!(
                    "{name} sits above {other} in the activity series (E° {:+.3} V against {:+.3} V), so the electrons would have to flow uphill: the less reactive metal does not displace the more reactive one",
                    c.e0_volts, o.e0_volts
                ),
            });
        } else if !acid && c.e0_volts < 0.0 {
            // Nothing below it to displace, no acid to dissolve in: the
            // remaining question is water itself.
            events.push(Event::NotYetModeled {
                vessel: vessel.id,
                what: format!(
                    "{name} stays as the metal: nothing dissolved here sits below it in the activity series. Its slow reaction with water itself — hydrogen over hours, a passivating hydroxide skin — is a rate this lab does not model"
                ),
            });
        }
    }
    events
}

/// A metal in contact with its own ion is an electrode, and it — not the
/// air above the beaker — sets the potential. Pin pe from the Nernst
/// equation over the computed activity, and say so in the provenance.
///
/// Takes the most reactive metal present whose ion the speciation can
/// see, because after a displacement that is the couple the solution has
/// settled against; the nobler metal's ion is then below anything the
/// solver resolves.
/// A metal standing in a solution of its own ion, and the potential it
/// holds there.
#[derive(Debug, Clone, PartialEq)]
pub struct Electrode {
    pub couple: &'static Couple,
    /// Activity of the ion, as the speciation reported it.
    pub activity: f64,
    /// Electrode potential vs SHE, V, by Nernst at the vessel temperature.
    pub volts: f64,
}

impl Electrode {
    /// `Zn | Zn+2` — the half-cell as it is written.
    pub fn label(&self) -> String {
        format!("{} | {}", self.couple.reduced, self.couple.oxidised)
    }
}

/// The electrode a vessel presents, if it presents one: a metal of the
/// series in contact with an observable amount of its own ion, whose
/// activity the speciation can see. None is an answer too — a copper
/// strip in brine is not a half-cell, and neither is a copper sulfate
/// solution with no copper in it.
pub fn electrode(vessel: &Vessel) -> Option<Electrode> {
    let slope = nernst_slope(vessel.temperature);
    let couple = SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| moles_in(vessel, c.reduced, Phase::Solid) > crate::OBSERVABLE_MOLES)
        .filter(|c| moles_in(vessel, c.oxidised, Phase::Aqueous) > crate::OBSERVABLE_MOLES)
        .filter(|c| matches!(activity_of(vessel, c.oxidised), Some((a, true)) if a > 0.0))
        .min_by(|a, b| a.e0_volts.total_cmp(&b.e0_volts))?;
    let (activity, _) = activity_of(vessel, couple.oxidised)?;
    let volts = couple.e0_volts + slope / couple.electrons * activity.log10();
    Some(Electrode {
        couple,
        activity,
        volts,
    })
}

/// What a current moves through a vessel, and what it cannot.
pub struct Electrolysis {
    pub species: SpeciesId,
    /// The ion the metal came out of, and how many of it per metal atom,
    /// so the caller can take it out of solution without re-deriving the
    /// couple.
    pub ion: SpeciesId,
    pub ion_per_metal: f64,
    pub coulombs: f64,
    pub electrons: f64,
    /// Moles of substance actually deposited — the demand, or the supply,
    /// whichever ran out first.
    pub moles: f64,
    pub grams: f64,
    pub per_ion: f64,
    /// Moles the charge *asked* for. Larger than `moles` when the solution
    /// ran out of ion before the charge ran out.
    pub demanded: f64,
}

/// Faraday's law over a vessel's own electrode.
///
/// n = Q / (z·F), and every term is read rather than assumed: Q from the
/// current and the clock, z from the couple the vessel actually holds, and
/// the ion's supply from the speciation. The law is one division; the
/// chemistry is in knowing which z, and that is what the bench supplies.
///
/// Returns `None` when the vessel is not a half-cell — `why_no_electrode`
/// says so in words.
pub fn electrolyse(vessel: &Vessel, amps: f64, seconds: f64) -> Option<Electrolysis> {
    let e = electrode(vessel)?;
    let coulombs = amps * seconds;
    let electrons = coulombs / FARADAY;
    let demanded = electrons / e.couple.electrons;
    // A current cannot deposit an ion that is not there. Past that point a
    // real cell starts electrolysing the water instead, which this bench
    // does not model — so it stops at the supply and says so rather than
    // inventing metal.
    let available =
        moles_in(vessel, e.couple.oxidised, Phase::Aqueous) / e.couple.oxidised_per_reduced;
    let moles = demanded.min(available).max(0.0);
    let grams = species::lookup_key(e.couple.reduced)
        .map(|d| moles * d.molar_mass)
        .unwrap_or(0.0);
    Some(Electrolysis {
        species: SpeciesId::new(e.couple.reduced),
        ion: SpeciesId::new(e.couple.oxidised),
        ion_per_metal: e.couple.oxidised_per_reduced,
        coulombs,
        electrons,
        moles,
        grams,
        per_ion: e.couple.electrons,
        demanded,
    })
}

/// Why a vessel is not a half-cell, for the reader who wired it up.
pub fn why_no_electrode(vessel: &Vessel) -> String {
    let metals: Vec<&str> = SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| moles_in(vessel, c.reduced, Phase::Solid) > crate::OBSERVABLE_MOLES)
        .map(|c| c.reduced)
        .collect();
    let ions: Vec<&str> = SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| moles_in(vessel, c.oxidised, Phase::Aqueous) > crate::OBSERVABLE_MOLES)
        .map(|c| c.oxidised)
        .collect();
    match (metals.is_empty(), ions.is_empty()) {
        (true, true) => format!(
            "{} holds neither a metal of the series nor a dissolved metal ion, so there is nothing to be an electrode",
            vessel.id
        ),
        (true, false) => format!(
            "{} has {} in solution but no metal standing in it — an ion alone is not a half-cell; it needs its own metal as the electrode",
            vessel.id,
            ions.join(", ")
        ),
        (false, true) => format!(
            "{} has {} but none of its ion in solution: a metal in water that does not contain its own ion has no defined potential here",
            vessel.id,
            metals.join(", ")
        ),
        (false, false) => format!(
            "{} has {} and {} but no metal is standing in a solution of its *own* ion, so no half-cell is defined — or the solution has not been characterised yet",
            vessel.id,
            metals.join(", "),
            ions.join(", ")
        ),
    }
}

/// An open-circuit galvanic cell between two half-cells.
///
/// No current flows, so nothing in either beaker changes: this is the
/// voltmeter reading the moment the wires touch, which is what the
/// activity series predicts and what a school cell is built to measure.
/// The salt bridge is assumed ideal — no liquid-junction potential — and
/// that is stated rather than corrected for.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub anode: Electrode,
    pub cathode: Electrode,
    /// E(cathode) − E(anode), V; positive by construction.
    pub volts: f64,
    /// The standard cell potential the series alone predicts, E°c − E°a.
    pub standard_volts: f64,
    /// Whether the first vessel handed to [`cell`] is the anode.
    pub anode_is_first: bool,
}

impl Cell {
    /// The reaction that would run if the circuit were closed.
    pub fn equation(&self) -> String {
        Displacement {
            reductant: self.anode.couple,
            oxidant: self.cathode.couple,
            electrons: 0.0,
            equilibrium: false,
            heat_joules: 0.0,
        }
        .equation()
    }

    /// Cell notation, anode on the left as it is written.
    pub fn notation(&self) -> String {
        format!(
            "{} | {} ‖ {} | {}",
            self.anode.couple.reduced,
            self.anode.couple.oxidised,
            self.cathode.couple.oxidised,
            self.cathode.couple.reduced
        )
    }
}

/// Wire two vessels as a cell. The more negative electrode is the anode —
/// that is not a choice, it is what the potentials say.
pub fn cell(a: &Vessel, b: &Vessel) -> Result<Cell, String> {
    let ea = electrode(a).ok_or_else(|| why_no_electrode(a))?;
    let eb = electrode(b).ok_or_else(|| why_no_electrode(b))?;
    if ea.couple.reduced == eb.couple.reduced {
        // A concentration cell. Real, small, and not nothing: two copper
        // electrodes in copper sulfate of different strengths give a few
        // tens of millivolts. It is the same arithmetic.
        if (ea.volts - eb.volts).abs() < 1e-12 {
            return Err(format!(
                "both vessels present the same {} electrode at the same activity: a cell of two identical half-cells reads 0 V",
                ea.label()
            ));
        }
    }
    let anode_is_first = ea.volts <= eb.volts;
    let (anode, cathode) = if anode_is_first { (ea, eb) } else { (eb, ea) };
    let volts = cathode.volts - anode.volts;
    let standard_volts = cathode.couple.e0_volts - anode.couple.e0_volts;
    Ok(Cell {
        anode,
        cathode,
        volts,
        standard_volts,
        anode_is_first,
    })
}

pub fn pin_electrode(vessel: &mut Vessel) -> Option<(&'static Couple, f64)> {
    let slope = nernst_slope(vessel.temperature);
    // Both members at observable amounts, or there is no electrode. Zinc
    // into an exactly equivalent amount of copper sulfate leaves 2e-10 mol
    // of copper(II) one way round and none the other, and pinning on that
    // trace made the reported potential depend on addition order —
    // +0.02 V against the open-air +0.77 V. At equivalence there is no
    // couple left to set a potential, which is the same chemistry the
    // titration endpoint is built on, and the same rule: withhold rather
    // than publish a bracket edge.
    let Electrode {
        couple,
        activity,
        volts: e,
    } = electrode(vessel)?;
    let pe = e / slope;
    let info = vessel.solution.as_mut()?;
    // Water's own hydrogen line at this pH (P(H₂) = 1 atm). A metal whose
    // equilibrium potential lies below it is not at equilibrium with the
    // water it stands in — it corrodes — and what a voltmeter reads then is
    // a mixed potential set by rates, which is why zinc works as a
    // Daniell-cell electrode (hydrogen is slow on zinc) and magnesium
    // fizzes. The Nernst value is still the number the series is built
    // on, so it is reported, with that said beside it.
    let hydrogen_line = -slope * info.ph;
    let caveat = if e < hydrogen_line {
        format!(
            " This lies below water's own hydrogen line ({hydrogen_line:+.3} V at pH {:.2}): {} is not at equilibrium with the water it stands in, and a real voltmeter would read a mixed potential set by rates this lab does not model",
            info.ph, couple.reduced
        )
    } else {
        String::new()
    };
    info.pe = Some(pe);
    if let Some(prov) = info.provenance.as_mut() {
        prov.routing = format!(
            "{}. The potential reported is the {}/{} electrode's, by Nernst over the computed activity ({:.3e}; E° {:+.4} V, CRC) — a metal in contact with its ion sets the potential, not the air above the beaker. The speciation itself was solved at the open-air pe.{caveat}",
            prov.routing, couple.reduced, couple.oxidised, activity, couple.e0_volts
        );
    }
    Some((couple, e))
}

/// The aqueous solver with the metallic state on top: solve, let the
/// series move electrons over the activities that solve produced, then
/// solve again so the products are speciated, and pin the potential to
/// whatever electrode is left standing in the beaker.
pub struct DisplacementEquilibrator {
    inner: Box<dyn Equilibrator>,
}

impl DisplacementEquilibrator {
    pub fn wrapping(inner: Box<dyn Equilibrator>) -> Self {
        DisplacementEquilibrator { inner }
    }

    pub fn has_series_metal(vessel: &Vessel) -> bool {
        vessel.contents.iter().any(|p| {
            p.phase == Phase::Solid && p.moles.0 > TRACE && couple_of_metal(&p.species.0).is_some()
        })
    }
}

impl Equilibrator for DisplacementEquilibrator {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        self.inner.applies(vessel)
    }

    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        self.inner.chemistry_applies(vessel)
            || (Self::has_series_metal(vessel) && kgw(vessel) > 0.0)
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        over(&mut *self.inner, vessel)
    }
}

/// Run an aqueous solver with the metallic state on top. This is the
/// wrapper's whole behaviour as a free function, so a host that holds its
/// aqueous solver by value (the browser does, to feed it shipped results)
/// can apply the same pass without boxing it.
pub fn over(inner: &mut dyn Equilibrator, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
    let t_start = vessel.temperature;
    let mut events = inner.equilibrate(vessel)?;
    if !DisplacementEquilibrator::has_series_metal(vessel) {
        return Ok(events);
    }
    let (more, reacted) = displace(vessel);
    if !reacted.is_empty() {
        // The first solve described a state the step has now moved past:
        // its "this solution is supersaturated against …" notes are about
        // copper that is no longer in solution. What it *did* — salts
        // dissolved, heat moved — stays in the ledger.
        let id = vessel.id;
        events.retain(|e| !matches!(e, Event::NotYetModeled { vessel, .. } if *vessel == id));
    }
    events.extend(more);
    if !reacted.is_empty() {
        events.extend(inner.equilibrate(vessel)?);
    }
    let plated: Vec<&str> = reacted.iter().map(|d| d.oxidant.reduced).collect();
    events.extend(bystanders(vessel, &plated));
    pin_electrode(vessel);

    // One step, one temperature story and one pH story. Two solves in a
    // row each announce theirs; collapse them to the vessel's actual path
    // from where the step began to where it ended.
    if !reacted.is_empty() {
        let id = vessel.id;
        let last_solution = events.iter().rposition(
            |e| matches!(e, Event::SolutionCharacterized { vessel, .. } if *vessel == id),
        );
        let mut i = 0;
        events.retain(|e| {
            let keep = match e {
                Event::TemperatureChanged { vessel, .. } if *vessel == id => false,
                Event::SolutionCharacterized { vessel, .. } if *vessel == id => {
                    Some(i) == last_solution
                }
                _ => true,
            };
            i += 1;
            keep
        });
        let t_end = vessel.temperature;
        if (t_end.0 - t_start.0).abs() > 0.01 {
            events.push(Event::TemperatureChanged {
                vessel: id,
                from: t_start,
                to: t_end,
            });
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_series_is_ordered_most_noble_first() {
        for pair in SERIES.windows(2) {
            assert!(
                pair[0].e0_volts > pair[1].e0_volts,
                "{} must sit above {}",
                pair[0].reduced,
                pair[1].reduced
            );
        }
    }

    #[test]
    fn every_couple_names_a_registry_species_or_the_acid() {
        for c in SERIES {
            assert!(
                c.oxidised == HYDROGEN_ION || species::lookup_key(c.oxidised).is_some(),
                "{} is not in the registry",
                c.oxidised
            );
            assert!(
                species::lookup_key(c.reduced).is_some(),
                "{} is not in the registry",
                c.reduced
            );
        }
    }

    #[test]
    fn equations_balance_electrons() {
        let d = Displacement {
            reductant: couple_of_metal("Mg").unwrap(),
            oxidant: couple_of_ion("Ag+").unwrap(),
            electrons: 1.0,
            equilibrium: false,
            heat_joules: 0.0,
        };
        assert_eq!(d.equation(), "Mg + 2 Ag+ → Mg+2 + 2 Ag");
        let d = Displacement {
            reductant: couple_of_metal("Zn").unwrap(),
            oxidant: couple_of_ion(HYDROGEN_ION).unwrap(),
            electrons: 1.0,
            equilibrium: false,
            heat_joules: 0.0,
        };
        assert_eq!(d.equation(), "Zn + 2 H+ → Zn+2 + H2");
    }

    #[test]
    fn a_metal_is_a_metal_and_charcoal_is_not() {
        assert!(is_elemental_metal("Mg"));
        assert!(!is_elemental_metal("C"));
        assert!(!is_elemental_metal("S"));
        assert!(!is_elemental_metal("MgO"));
        assert!(!is_elemental_metal("Mg+2"));
    }
}
