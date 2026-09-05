//! BRD-023: rust, galvanic couples and coatings.
//!
//! The interesting assertions here are the negative ones. A bench that
//! only says "the iron is rusting" has learned a word; one that can say
//! WHICH of two metals rusts, and why the other one does not, has learned
//! the chemistry — and every one of those sentences used to be
//! `not yet modelled`.
//!
//! The stack deliberately has no aqueous engine, so `vessel.solution` is
//! never set and no verdict here carries a rate. That is the point: the
//! mechanism — which metal is the anode, what a coating does, what
//! happens with the oxygen taken away — is decided without one, and the
//! rate is the part that needs an electrolyte characterised for it.

use kerotakis_core::corrosion::{
    metal_datum, ohmic_throttle, oxygen_limiting_current_a_per_cm2, penetration_mm_per_year,
    verdicts, CorrosionEquilibrator,
};
use kerotakis_core::displacement::SERIES;
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Liters, Moles};
use kerotakis_core::vessel::{Headspace, Vessel, VesselId};
use kerotakis_core::{Bench, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen};

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CorrosionEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn run(commands: &[&str]) -> Vec<Event> {
    let mut bench = Bench::new();
    let mut solver = stack();
    let mut events = Vec::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        events.extend(
            bench
                .step_with(op, &mut solver, &PermissiveScreen)
                .unwrap_or_else(|error| panic!("{command}: {error}")),
        );
    }
    events
}

/// The LAST corrosion verdict for a metal, which is the one the finished
/// script left standing.
fn verdict_for<'a>(events: &'a [Event], key: &str) -> Option<(bool, &'a str)> {
    events.iter().rev().find_map(|event| match event {
        Event::Corroded {
            species,
            corroding,
            why,
            ..
        } if species.0 == key => Some((*corroding, why.as_str())),
        _ => None,
    })
}

fn beaker(metals: &[(&str, f64)]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    for (key, moles) in metals {
        vessel.deposit(SpeciesId::new(key), Moles(*moles), Phase::Solid);
    }
    vessel
}

// ── the three things ───────────────────────────────────────────────

#[test]
fn iron_water_and_air_together_rust() {
    let events = run(&["add v1 Fe 2g", "add v1 water 20mL", "wait 1h"]);
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(corroding, "iron in an open beaker of water corrodes: {why}");
    assert!(
        why.contains("oxygen"),
        "the verdict must name the third requirement: {why}"
    );
}

#[test]
fn iron_with_no_water_gets_no_verdict_at_all() {
    let events = run(&["add v1 Fe 2g", "wait 1h"]);
    assert!(
        verdict_for(&events, "Fe").is_none(),
        "a dry nail is not this route's business: it has no atmospheric-humidity model"
    );
}

#[test]
fn taking_the_oxygen_away_stops_it() {
    let mut sealed = beaker(&[("Fe", 0.03)]);
    sealed.headspace = Headspace::Sealed {
        volume: Liters(0.5),
    };
    let spoken = verdicts(&sealed);
    let iron = spoken
        .iter()
        .find(|v| v.metal == "Fe")
        .expect("a verdict on the iron");
    assert!(
        !iron.corroding,
        "with no oxygen in a closed vessel nothing rusts: {}",
        iron.why
    );
    assert!(
        iron.why.contains("oxygen"),
        "and the reason has to name the oxygen: {}",
        iron.why
    );
    assert!(iron.penetration_mm_per_year.is_none());

    // The same vessel with oxygen in it turns over again — so the
    // negative above is about the oxygen and not about being sealed.
    let mut aerated = sealed.clone();
    aerated.deposit(SpeciesId::new("O2"), Moles(0.01), Phase::Gas);
    let iron = verdicts(&aerated)
        .into_iter()
        .find(|v| v.metal == "Fe")
        .expect("a verdict on the iron");
    assert!(iron.corroding, "oxygen restores the cathode: {}", iron.why);
}

// ── the galvanic couple ────────────────────────────────────────────

#[test]
fn zinc_beside_iron_is_the_one_that_goes() {
    let events = run(&[
        "add v1 Fe 1g",
        "add v1 Zn 1g",
        "add v1 water 20mL",
        "wait 1h",
    ]);
    let (zinc, _) = verdict_for(&events, "Zn").expect("a verdict on the zinc");
    let (iron, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(zinc, "the zinc is the anode and corrodes");
    assert!(!iron, "the iron is the cathode and is spared: {why}");
    assert!(
        why.contains("anode"),
        "the iron's sentence has to say what is protecting it: {why}"
    );
}

#[test]
fn galvanised_steel_protects_its_own_iron() {
    // The recipe resolves the coat as a bulk 3% of zinc rather than as a
    // layer, and says so in its own lot assumptions. That is exactly the
    // geometry a scratch removes — which is why the answer to "what
    // happens when scratched galvanised steel gets wet" is the same
    // answer as for the unscratched sheet: the zinc protects the iron it
    // is merely next to, not only the iron it covers.
    let events = run(&["add v1 galvanized_steel 2g", "add v1 water 20mL", "wait 1h"]);
    let (zinc, _) = verdict_for(&events, "Zn").expect("a verdict on the zinc");
    let (iron, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(zinc, "the coat is the anode");
    assert!(!iron, "the steel under it is spared: {why}");
}

#[test]
fn iron_beside_copper_is_the_one_that_goes() {
    let events = run(&[
        "add v1 Fe 1g",
        "add v1 Cu 1g",
        "add v1 water 100mL",
        "wait 1h",
    ]);
    let (iron, _) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    let (copper, why) = verdict_for(&events, "Cu").expect("a verdict on the copper");
    assert!(iron, "iron is below copper, so iron is the anode");
    assert!(!copper, "and the copper is spared: {why}");
}

#[test]
fn the_anode_is_always_the_lower_potential_metal() {
    let couples: Vec<_> = SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| c.e0_volts < 0.0)
        .collect();
    for a in &couples {
        for b in &couples {
            if a.reduced == b.reduced {
                continue;
            }
            let vessel = beaker(&[(a.reduced, 0.02), (b.reduced, 0.02)]);
            let spoken = verdicts(&vessel);
            let corroding: Vec<&str> = spoken
                .iter()
                .filter(|v| v.corroding)
                .map(|v| v.metal)
                .collect();
            let expected = if a.e0_volts < b.e0_volts {
                a.reduced
            } else {
                b.reduced
            };
            assert_eq!(
                corroding,
                [expected],
                "{} against {}: exactly the lower-E° metal corrodes",
                a.reduced,
                b.reduced
            );
        }
    }
}

// ── barriers ───────────────────────────────────────────────────────

#[test]
fn stainless_steel_keeps_its_iron_behind_a_passive_film() {
    let events = run(&[
        "add v1 stainless_steel 5g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(!corroding, "stainless steel does not rust: {why}");
    assert!(
        why.contains("chromium"),
        "and the reason is the chromium, not the iron: {why}"
    );
}

#[test]
fn a_complete_paint_film_stops_it() {
    let events = run(&["add v1 painted_iron 2g", "add v1 water 20mL", "wait 1h"]);
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(!corroding, "a sound coating keeps the water off: {why}");
    assert!(why.contains("paint"), "and the reason is the film: {why}");
}

#[test]
fn one_bare_lot_withdraws_the_barrier_claim() {
    // A stainless spoon and a bare nail in one beaker are one pool of
    // `Fe` to the bench. Claiming the spoon's passive film for the nail
    // would be the barrier protecting metal it was never on.
    let events = run(&[
        "add v1 stainless_steel 5g",
        "add v1 Fe 2g",
        "add v1 water 20mL",
        "wait 1h",
    ]);
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(
        corroding,
        "with bare iron in the same glass the film cannot be claimed: {why}"
    );
}

// ── the metals that do not rust ────────────────────────────────────

#[test]
fn copper_does_not_rust_and_the_patina_is_named_as_unmodelled() {
    let events = run(&[
        "add v1 Cu 2g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    let (corroding, why) = verdict_for(&events, "Cu").expect("a verdict on the copper");
    assert!(
        !corroding,
        "copper is above hydrogen and does not rust: {why}"
    );
    assert!(
        why.contains("patina"),
        "the green is a different question and has to be named: {why}"
    );
    assert!(
        why.contains("no route here claims it"),
        "and named as one this bench does not model: {why}"
    );
}

// ── who owns the beaker ────────────────────────────────────────────

#[test]
fn free_acid_leaves_the_beaker_to_displacement() {
    let mut vessel = beaker(&[("Mg", 0.04)]);
    // Unspent acidity is `-solute_charge` plus the ledger acids; a
    // strongly negative solute charge is a beaker with free protons in it.
    vessel.solute_charge = -0.02;
    assert!(
        verdicts(&vessel).is_empty(),
        "with acid present the cathode is hydrogen, and displacement computes that"
    );
}

#[test]
fn a_dissolved_noble_ion_leaves_the_beaker_to_displacement() {
    let mut vessel = beaker(&[("Zn", 0.02)]);
    vessel.deposit(SpeciesId::new("Cu+2"), Moles(0.01), Phase::Aqueous);
    assert!(
        verdicts(&vessel).is_empty(),
        "zinc in copper sulfate is a displacement, and only one route may narrate it"
    );
}

// ── the numbers ────────────────────────────────────────────────────

#[test]
fn salt_raises_the_corrosion_current_monotonically_towards_a_ceiling() {
    let ceiling = oxygen_limiting_current_a_per_cm2();
    let distilled = ceiling * ohmic_throttle(1.0);
    let tap = ceiling * ohmic_throttle(500.0);
    let sea = ceiling * ohmic_throttle(50_000.0);
    assert!(distilled < tap && tap < sea, "more salt, more current");
    assert!(
        sea < ceiling,
        "and never past the oxygen the water can deliver"
    );
    assert!(
        sea / tap < 3.0,
        "brine is about twice tap water, not ten times: the cap is the same for both"
    );
}

#[test]
fn the_penetration_rate_is_faradays_law_on_the_named_metal() {
    let iron = metal_datum("Fe").expect("iron datum");
    let current = oxygen_limiting_current_a_per_cm2();
    let mm = penetration_mm_per_year(current, iron);
    // i M / (n F rho), in cm/s, times a year, times 10 mm per cm.
    let expected = current * iron.molar_mass
        / (iron.electrons * kerotakis_core::displacement::FARADAY * iron.density_g_per_cm3)
        * 31_557_600.0
        * 10.0;
    assert!((mm - expected).abs() < 1e-12, "{mm} vs {expected}");
    assert!(mm > 0.0);
}
