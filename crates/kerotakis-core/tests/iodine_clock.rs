//! EXP-43: iodine-clock kinetics.
//!
//! Two clock reactions beside the landed thiosulfate clock: the
//! iodide–peroxide clock (H₂O₂ + 2 KI → I₂ + 2 KOH) and the
//! iodate–bisulfite Landolt clock (KIO₃ + 3 HSO₃⁻ + 3 Na⁺ → KI + 3 NaHSO₄).
//! Each must show clock time scaling with concentration and temperature.

use kerotakis_core::kinetics::{self, advance};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{SolutionInfo, Vessel, VesselId};

// ~5.5 mol water ≈ 0.1 L vessel. Moles below are chosen to give
// school-practical molarities: 0.005 mol / 0.1 L ≈ 0.05 M.

fn iodide_peroxide_vessel(celsius: f64, ki: f64, h2o2: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
    v.deposit(SpeciesId::new("KI"), Moles(ki), Phase::Aqueous);
    v.deposit(SpeciesId::new("H2O2"), Moles(h2o2), Phase::Liquid);
    v.temperature = Kelvin(273.15 + celsius);
    v.solution = Some(SolutionInfo {
        scope: Default::default(),
        solvent_kg: None,
        redox: Vec::new(),
        pe: None,
        ph: 1.7,
        ionic_strength: 0.02,
        species: Vec::new(),
        provenance: None,
    });
    v
}

/// The vessel as it exists AFTER a solve, which is the only vessel this
/// reaction ever runs in.
///
/// It used to deposit the bottle, `NaHSO3`, and that stopped being honest
/// when `databases::minteq_v4()` borrowed sulfite's acid constant: a solved
/// beaker holds `Na+` and `HSO3-` and no `NaHSO3` at all, so a hand-built
/// vessel naming the bottle was testing a mixture the bench cannot
/// produce. The rate law is keyed on `HSO3-` now and these vessels hold
/// what it names.
///
/// **The sodium is deposited too, and it is not decoration.** A hand-built
/// vessel whose reactant has become a bare anion is not electroneutral
/// without it, and the represented-strong-acid bookkeeping derives
/// spendable acid from exactly that balance - drop the counterion and the
/// vessel reads as holding base. `NaHSO3` splits one-for-one, so one `Na+`
/// per `HSO3-` is the whole of the correction here.
///
/// `KIO3` stays as the bottle deliberately: no routed dataset defines
/// iodate, so it is what a solved vessel still holds.
fn landolt_vessel(celsius: f64, kio3: f64, nahso3: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
    v.deposit(SpeciesId::new("KIO3"), Moles(kio3), Phase::Aqueous);
    v.deposit(SpeciesId::new("HSO3-"), Moles(nahso3), Phase::Aqueous);
    v.deposit(SpeciesId::new("Na+"), Moles(nahso3), Phase::Aqueous);
    v.temperature = Kelvin(273.15 + celsius);
    v
}

fn moles_of(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

// ── Iodide–peroxide clock ──────────────────────────────────────────

#[test]
fn iodide_peroxide_is_in_the_registry() {
    assert!(
        kinetics::lookup("iodide-peroxide-clock").is_some(),
        "reaction must be in the kinetics REGISTRY"
    );
}

#[test]
fn iodide_peroxide_produces_iodine() {
    let mut v = iodide_peroxide_vessel(25.0, 0.005, 0.005);
    advance(&mut v, 60.0).unwrap();
    assert!(
        moles_of(&v, "I2") > 1e-6,
        "I₂ must be produced: {}",
        moles_of(&v, "I2")
    );
}

#[test]
fn iodide_peroxide_concentration_doubles_rate() {
    let mut a = iodide_peroxide_vessel(25.0, 0.0025, 0.01);
    let mut b = iodide_peroxide_vessel(25.0, 0.005, 0.01);
    advance(&mut a, 2.0).unwrap();
    advance(&mut b, 2.0).unwrap();
    let ratio = moles_of(&b, "I2") / moles_of(&a, "I2");
    assert!(
        (1.8..2.2).contains(&ratio),
        "first order: doubling [KI] doubles I₂ production, got ×{ratio:.3}"
    );
}

#[test]
fn iodide_peroxide_warmer_is_faster() {
    let mut cold = iodide_peroxide_vessel(20.0, 0.005, 0.005);
    let mut warm = iodide_peroxide_vessel(40.0, 0.005, 0.005);
    advance(&mut cold, 5.0).unwrap();
    advance(&mut warm, 5.0).unwrap();
    let ratio = moles_of(&warm, "I2") / moles_of(&cold, "I2");
    assert!(
        ratio > 2.5,
        "20 °C warmer must speed up the reaction: ×{ratio:.2}"
    );
}

#[test]
fn iodide_peroxide_clock_time_in_practical_range() {
    let mut v = iodide_peroxide_vessel(25.0, 0.005, 0.005);
    let mut seconds = 0.0;
    let threshold = 1e-4;
    while moles_of(&v, "I2") < threshold && seconds < 300.0 {
        advance(&mut v, 1.0).unwrap();
        seconds += 1.0;
    }
    assert!(
        (10.0..120.0).contains(&seconds),
        "clock time {seconds} s is outside practical range"
    );
}

#[test]
fn iodide_peroxide_products_form() {
    let mut v = iodide_peroxide_vessel(25.0, 0.005, 0.005);
    advance(&mut v, 60.0).unwrap();
    assert!(moles_of(&v, "I2") > 0.0, "must produce I₂");
    assert!(moles_of(&v, "KOH") > 0.0, "must produce KOH");
}

// ── Iodate–bisulfite (Landolt) clock ───────────────────────────────

#[test]
fn landolt_is_in_the_registry() {
    assert!(
        kinetics::lookup("iodate-bisulfite-clock").is_some(),
        "reaction must be in the kinetics REGISTRY"
    );
}

#[test]
fn landolt_consumes_bisulfite() {
    let mut v = landolt_vessel(25.0, 0.002, 0.006);
    advance(&mut v, 120.0).unwrap();
    assert!(
        moles_of(&v, "HSO3-") < 0.001,
        "bisulfite must be substantially consumed: {}",
        moles_of(&v, "HSO3-")
    );
}

#[test]
fn landolt_concentration_doubles_rate() {
    let mut a = landolt_vessel(25.0, 0.001, 0.006);
    let mut b = landolt_vessel(25.0, 0.002, 0.006);
    advance(&mut a, 2.0).unwrap();
    advance(&mut b, 2.0).unwrap();
    let consumed_a = 0.006 - moles_of(&a, "HSO3-");
    let consumed_b = 0.006 - moles_of(&b, "HSO3-");
    let ratio = consumed_b / consumed_a;
    assert!(
        (1.8..2.2).contains(&ratio),
        "first order: doubling [KIO₃] doubles rate, got ×{ratio:.3}"
    );
}

#[test]
fn landolt_warmer_is_faster() {
    let mut cold = landolt_vessel(20.0, 0.002, 0.006);
    let mut warm = landolt_vessel(40.0, 0.002, 0.006);
    advance(&mut cold, 3.0).unwrap();
    advance(&mut warm, 3.0).unwrap();
    let consumed_cold = 0.006 - moles_of(&cold, "HSO3-");
    let consumed_warm = 0.006 - moles_of(&warm, "HSO3-");
    let ratio = consumed_warm / consumed_cold;
    assert!(
        ratio > 2.0,
        "20 °C warmer must speed up the reaction: ×{ratio:.2}"
    );
}

#[test]
fn landolt_produces_ki_and_bisulfate() {
    let mut v = landolt_vessel(25.0, 0.002, 0.006);
    advance(&mut v, 120.0).unwrap();
    assert!(moles_of(&v, "KI") > 0.0, "must produce KI");
    // `NaHSO4`, with the proton still bound. Writing the products as
    // sulfate plus free `H+` balances on paper and perturbs the shared
    // represented-acid bookkeeping - see the note on the reaction.
    assert!(moles_of(&v, "NaHSO4") > 0.0, "must produce NaHSO₄");
}

/// One long step must cost the same as ten short ones.
///
/// This is the guard the re-keyed stoichiometry needs, and it is named
/// after a real failure rather than a hypothetical: when #536 made
/// thiosulfate a bare anion, a hand-built vessel lost its counterion, the
/// represented-strong-acid bookkeeping read the vessel as holding base,
/// and one long step disagreed with ten short ones by 58 %.
///
/// This reaction now MINTS PROTONS - `3 H+` per iodate, which is real
/// chemistry, a running Landolt mixture does acidify - and that is a new
/// thing for that bookkeeping. If the protons it produces were to feed
/// back into the acid inventory inconsistently, the disagreement would
/// show up here first and nowhere else.
#[test]
fn the_clock_agrees_with_itself_over_one_step_and_ten() {
    let mut one = landolt_vessel(25.0, 0.002, 0.006);
    advance(&mut one, 20.0).unwrap();

    let mut ten = landolt_vessel(25.0, 0.002, 0.006);
    for _ in 0..10 {
        advance(&mut ten, 2.0).unwrap();
    }

    for key in ["HSO3-", "KIO3", "KI", "NaHSO4", "Na+"] {
        let a = moles_of(&one, key);
        let b = moles_of(&ten, key);
        let scale = a.abs().max(b.abs()).max(1e-9);
        assert!(
            (a - b).abs() / scale < 0.02,
            "{key}: one 20 s step gives {a}, ten 2 s steps give {b}"
        );
    }
}

// ── Network-level checks ───────────────────────────────────────────

#[test]
fn all_clock_reactions_pass_the_conservation_lint() {
    kinetics::lint_network(&kinetics::NETWORK)
        .expect("the built-in network must pass the conservation lint");
}
