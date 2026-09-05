//! BRD-023: which metal corrodes when two of them share an electrolyte.
//!
//! The bench could already rust a nail — `kinetics::iron-corrosion` has
//! done that since 2026-09-02. What it could not do was notice the lump
//! of zinc lying against the nail, so galvanising was a word the bench
//! knew and a thing it did not do.
//!
//! Every assertion here therefore checks the BEAKER and not the caption.
//! A verdict saying "the zinc protects the iron" over a vessel whose iron
//! rusted anyway is worse than no verdict at all, so each protection test
//! is paired with the unprotected control that shows the same script
//! rusting when the protection is taken away.

use kerotakis_core::corrosion::{allows_reaction, anode, is_protected, CorrosionEquilibrator};
use kerotakis_core::displacement::SERIES;
use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::Moles;
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{
    Bench, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen, OBSERVABLE_MOLES,
};

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CorrosionEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn run(commands: &[&str]) -> (Bench, Vec<Event>) {
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
    (bench, events)
}

fn amount(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("v1")
        .moles_of(&SpeciesId::new(key))
        .0
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

// ── the control: with all three, iron still rusts ──────────────────

#[test]
fn iron_water_and_oxygen_together_still_rust() {
    let (bench, events) = run(&[
        "add v1 Fe 2g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert!(
        amount(&bench, "Fe2O3") > OBSERVABLE_MOLES,
        "the three-things case is the one that must keep working"
    );
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(
        corroding,
        "and the verdict has to agree with the beaker: {why}"
    );
}

#[test]
fn taking_the_oxygen_away_stops_it_and_says_so() {
    let (bench, events) = run(&["add v1 Fe 2g", "add v1 water 20mL", "wait 1h"]);
    assert_eq!(
        amount(&bench, "Fe2O3"),
        0.0,
        "no oxygen in the vessel, no rust"
    );
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(!corroding, "{why}");
    assert!(
        why.contains("oxygen"),
        "and the reason names the missing one of the three: {why}"
    );
}

#[test]
fn iron_with_no_water_gets_no_verdict_at_all() {
    let (_, events) = run(&["add v1 Fe 2g", "add v1 O2 0.01mol", "wait 1h"]);
    assert!(
        verdict_for(&events, "Fe").is_none(),
        "a dry nail is not this route's business: it has no atmospheric-humidity model"
    );
}

// ── the galvanic couple, checked in the beaker ─────────────────────

#[test]
fn zinc_beside_iron_corrodes_instead_of_it() {
    let script = [
        "add v1 Fe 1g",
        "add v1 Zn 1g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ];
    let (coupled, events) = run(&script);
    let (control, _) = run(&[
        "add v1 Fe 1g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);

    // The control rusts, so the script is capable of rusting iron.
    assert!(
        amount(&control, "Fe2O3") > OBSERVABLE_MOLES,
        "the unprotected control must rust, or this test proves nothing"
    );
    // With the zinc there, it does not.
    assert_eq!(
        amount(&coupled, "Fe2O3"),
        0.0,
        "the iron must not rust while the zinc is in contact with it"
    );
    assert!(
        amount(&coupled, "Fe") >= amount(&control, "Fe"),
        "and the iron itself must still be there"
    );
    // And the zinc is what paid for it.
    assert!(
        amount(&coupled, "Zn(OH)2") > OBSERVABLE_MOLES,
        "the zinc has to actually corrode, or 'sacrificial' is a caption"
    );
    let (before, _) = run(&[
        "add v1 Fe 1g",
        "add v1 Zn 1g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
    ]);
    assert!(
        amount(&coupled, "Zn") < amount(&before, "Zn"),
        "and the zinc metal has to be going away"
    );

    let (iron, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    let (zinc, _) = verdict_for(&events, "Zn").expect("a verdict on the zinc");
    assert!(!iron, "the iron's verdict must match its beaker: {why}");
    assert!(zinc, "and the zinc's must too");
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
    // happens when scratched galvanised steel gets wet" is the same as
    // for the unscratched sheet: the zinc protects the iron it is merely
    // next to, not only the iron it covers.
    let (bench, events) = run(&[
        "add v1 galvanized_steel 2g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert_eq!(
        amount(&bench, "Fe2O3"),
        0.0,
        "the steel under the coat is spared"
    );
    assert!(
        amount(&bench, "Zn(OH)2") > OBSERVABLE_MOLES,
        "and the coat is what goes"
    );
    let (iron, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(!iron, "{why}");
}

#[test]
fn iron_beside_copper_is_the_one_that_goes() {
    let (bench, events) = run(&[
        "add v1 Fe 1g",
        "add v1 Cu 1g",
        "add v1 water 100mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert!(
        amount(&bench, "Fe2O3") > OBSERVABLE_MOLES,
        "iron is below copper, so the iron is the anode and it corrodes"
    );
    let (copper, why) = verdict_for(&events, "Cu").expect("a verdict on the copper");
    assert!(!copper, "and the copper is the cathode: {why}");
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
            let expected = if a.e0_volts < b.e0_volts {
                a.reduced
            } else {
                b.reduced
            };
            assert_eq!(
                anode(&vessel).map(|c| c.reduced),
                Some(expected),
                "{} against {}: the lower-E° metal is the anode",
                a.reduced,
                b.reduced
            );
            assert!(
                !is_protected(&vessel, expected),
                "the anode protects nobody"
            );
            let spared = if expected == a.reduced {
                b.reduced
            } else {
                a.reduced
            };
            assert!(
                is_protected(&vessel, spared),
                "{spared} is the cathode here"
            );
        }
    }
}

// ── barriers, checked in the beaker ────────────────────────────────

#[test]
fn stainless_steel_keeps_its_iron_behind_a_passive_film() {
    let (bench, events) = run(&[
        "add v1 stainless_steel 5g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert_eq!(
        amount(&bench, "Fe2O3"),
        0.0,
        "stainless steel does not rust"
    );
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(!corroding, "{why}");
    assert!(
        why.contains("chromium"),
        "and the reason is the chromium, not the iron: {why}"
    );
}

#[test]
fn a_complete_paint_film_stops_it() {
    let (bench, events) = run(&[
        "add v1 painted_iron 2g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert_eq!(
        amount(&bench, "Fe2O3"),
        0.0,
        "a sound coating keeps the water off"
    );
    let (corroding, why) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(!corroding, "{why}");
    assert!(why.contains("paint"), "and the reason is the film: {why}");
}

#[test]
fn one_bare_lot_withdraws_the_barrier_claim() {
    // A stainless spoon and a bare nail in one beaker are one pool of
    // `Fe` to the bench. Claiming the spoon's passive film for the nail
    // would be a barrier protecting metal it was never on.
    let (bench, events) = run(&[
        "add v1 stainless_steel 5g",
        "add v1 Fe 2g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert!(
        amount(&bench, "Fe2O3") > OBSERVABLE_MOLES,
        "with bare iron in the same glass the film cannot be claimed"
    );
    let (corroding, _) = verdict_for(&events, "Fe").expect("a verdict on the iron");
    assert!(corroding);
}

// ── the metals that do not rust ────────────────────────────────────

#[test]
fn copper_does_not_rust_and_the_patina_is_named_as_unmodelled() {
    let (bench, events) = run(&[
        "add v1 Cu 2g",
        "add v1 water 20mL",
        "add v1 O2 0.01mol",
        "wait 1h",
    ]);
    assert!(
        amount(&bench, "Cu") > 0.031,
        "the copper is all still there — nothing here consumes it"
    );
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
        kerotakis_core::corrosion::verdicts(&vessel).is_empty(),
        "with acid present the cathode is hydrogen, and displacement computes that"
    );
}

#[test]
fn a_nobler_dissolved_ion_leaves_the_beaker_to_displacement() {
    let mut vessel = beaker(&[("Zn", 0.02)]);
    vessel.deposit(SpeciesId::new("Cu+2"), Moles(0.01), Phase::Aqueous);
    assert!(
        kerotakis_core::corrosion::verdicts(&vessel).is_empty(),
        "zinc in copper sulfate is a displacement, and only one route may narrate it"
    );
}

#[test]
fn a_metals_own_ion_does_not_hand_the_beaker_away() {
    // Zinc corroding is how Zn2+ gets into the water in the first place.
    // Reading that as a displacement would have the route stand aside
    // the moment it succeeded.
    let mut vessel = beaker(&[("Zn", 0.02)]);
    vessel.deposit(SpeciesId::new("Zn+2"), Moles(0.001), Phase::Aqueous);
    assert!(
        !kerotakis_core::corrosion::verdicts(&vessel).is_empty(),
        "zinc beside its own ion is still zinc corroding"
    );
}

// ── the gate itself ────────────────────────────────────────────────

#[test]
fn the_gate_speaks_only_for_the_reactions_it_names() {
    let vessel = beaker(&[("Fe", 0.02), ("Zn", 0.02)]);
    assert!(!allows_reaction("iron-corrosion", &vessel));
    assert!(allows_reaction("zinc-corrosion", &vessel));
    assert!(
        allows_reaction("peroxide-decomposition", &vessel),
        "an unrelated reaction must pass straight through the gate"
    );
}
