//! What is inside a battery, and what grows on the outside of one
//! (curiosity corpus mat-058 and mat-071).
//!
//! Both rows used to fail at the parser: neither `alkaline_battery` nor
//! `battery_terminal` was a name this bench knew. The interesting part is
//! not that they now parse — it is what they had to be in order to answer
//! honestly.
//!
//! A sealed cell has to be a COHERENT OBJECT. A recipe that dispensed its
//! zinc, its manganese dioxide and its caustic paste into the beaker the
//! moment it was put down would describe a battery that had been cut open,
//! and every route on this bench would then narrate the wrong experiment —
//! the zinc corroding in the alkali, the water making a solution. Kept
//! whole, it weighs what it weighs, and that is the fact the row is about:
//! a flat cell weighs exactly what a fresh one does.
//!
//! A terminal has to be the opposite. The crust is a corrosion verdict and
//! the corrosion route reads metal in the vessel, so the post is dispensed
//! as lead — while its acid film is deliberately NOT resolved, because
//! sulfuric acid put in as a species would make this "lead in acid", which
//! is a different experiment.

use kerotakis_core::corrosion::{creep_for, CorrosionEquilibrator};
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::SolverStack;
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{Bench, Event, HonestyEquilibrator, MixingEquilibrator, PermissiveScreen};

fn run(commands: &[&str]) -> (Bench, Vec<Event>, Vec<Event>) {
    let mut bench = Bench::new();
    let mut solver = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CorrosionEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    let mut all = Vec::new();
    let mut last = Vec::new();
    for command in commands {
        let op = parse_op(command)
            .unwrap_or_else(|error| panic!("parse {command}: {error}"))
            .expect("operator");
        last = bench
            .step_with(op, &mut solver, &PermissiveScreen)
            .unwrap_or_else(|error| panic!("{command}: {error}"));
        all.extend(last.iter().cloned());
    }
    (bench, all, last)
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).expect("v1")
}

// ── mat-058: the sealed cell ─────────────────────────────────────────────

/// The whole of mat-058's evidence: twenty grams in, twenty grams on the
/// balance. A sealed cell lets nothing out, so nothing about discharging
/// it can change what it weighs — and the object has to stay whole for
/// that to be true rather than a coincidence.
#[test]
fn a_sealed_cell_keeps_its_mass_and_its_insides() {
    let (bench, _, _) = run(&["add v1 alkaline_battery 20g"]);
    let v = vessel(&bench);
    assert!(
        (v.mass().0 - 20.0).abs() < 1e-9,
        "the balance reads twenty grams, got {}",
        v.mass().0
    );
    // Nothing was emptied into the beaker: no zinc to corrode, no caustic
    // paste to make a solution, no water to be a solvent.
    assert!(v.contents.is_empty(), "{:?}", v.contents);
    assert!(v.unresolved_materials.is_empty());
    assert_eq!(v.material_objects.len(), 1);
    assert!(v.liquid_volume().0 <= 0.0);
}

/// And it says what is moving inside, which is the question. The reaction
/// is curated prose written down beside the balance reading — not run,
/// because neither of its products is an installed species.
#[test]
fn the_cell_names_its_own_discharge() {
    let (_, all, _) = run(&["add v1 alkaline_battery 20g"]);
    let cell = all
        .iter()
        .find_map(|event| match event {
            Event::SealedCell {
                open_circuit_volts,
                reaction,
                why,
                material,
                ..
            } => Some((
                *open_circuit_volts,
                reaction.clone(),
                why.clone(),
                material.clone(),
            )),
            _ => None,
        })
        .expect("a sealed-cell statement");
    assert!((cell.0 - 1.5).abs() < 1e-9, "nominal 1.5 V, got {}", cell.0);
    assert_eq!(cell.1, "Zn + 2 MnO2 -> ZnO + Mn2O3");
    assert!(cell.2.contains("anode") && cell.2.contains("cathode"));
    assert!(
        cell.2.contains("hydroxide"),
        "the electrolyte carries the ions and is not used up: {}",
        cell.2
    );
    // No brand, anywhere: the object is described by its chemistry.
    assert!(cell.3.contains("alkaline"));
    // The products are named and not made. Nothing entered the ledger.
    assert!(!all
        .iter()
        .any(|event| matches!(event, Event::Reacted { .. } | Event::Precipitated { .. })));
}

/// The reaction is honest about being prose: neither product exists as a
/// species here, which is exactly why it is not run. A test that only
/// checked the sentence would not notice if someone later wired it up
/// against species that were never installed.
#[test]
fn neither_discharge_product_is_an_installed_species() {
    assert!(kerotakis_core::species::lookup_key("ZnO").is_none());
    assert!(kerotakis_core::species::lookup_key("Mn2O3").is_none());
    // The reactants ARE installed, which is what makes the equation
    // checkable rather than decorative.
    assert!(kerotakis_core::species::lookup_key("Zn").is_some());
    assert!(kerotakis_core::species::lookup_key("MnO2").is_some());
}

// ── mat-071: the white crust ─────────────────────────────────────────────

/// mat-071. A lead post with a film of its own electrolyte on it grows
/// lead sulfate, and it does so whether or not there is oxygen in the
/// beaker — because its cathode is the cell it is part of, not the air.
#[test]
fn a_battery_post_grows_lead_sulfate_and_says_where_it_came_from() {
    let (bench, all, _) = run(&["add v1 battery_terminal 2g", "add v1 water 5mL"]);
    let verdict = all
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::Corroded {
                species,
                corroding,
                why,
                ..
            } if species.0 == "Pb" => Some((*corroding, why.clone())),
            _ => None,
        })
        .expect("a verdict about the lead");
    assert!(verdict.0, "the post is corroding");
    assert!(
        verdict.1.contains("lead(II) sulfate"),
        "the crust is named: {}",
        verdict.1
    );
    assert!(
        verdict.1.contains("comes out of the battery"),
        "and it does not come out of the water: {}",
        verdict.1
    );
    assert!(
        verdict.1.contains("NOTHING IS ADDED TO THE LEDGER"),
        "a named product is not a weighed one: {}",
        verdict.1
    );
    // The balance is the lead plus the water and nothing else: no crust
    // was weighed into the vessel.
    assert!(
        (vessel(&bench).mass().0 - 7.0).abs() < 0.05,
        "two grams of lead and five of water, got {}",
        vessel(&bench).mass().0
    );
}

/// The claim is about the OBJECT, not about lead. A plain lead sheet in
/// water grows nothing of the kind, and the acid film is asserted from
/// the post's identity rather than resolved into the beaker — which is
/// the whole reason the displacement route does not own this vessel.
#[test]
fn a_bare_lead_lump_grows_no_crust() {
    let (bench, _, _) = run(&["add v1 Pb 2g", "add v1 water 5mL"]);
    assert!(creep_for(vessel(&bench), "Pb").is_none());
    let (post, _, _) = run(&["add v1 battery_terminal 2g", "add v1 water 5mL"]);
    assert!(creep_for(vessel(&post), "Pb").is_some());
    // One bare lot beside the post withdraws the claim, the same rule a
    // stainless spoon dropped in beside a bare nail lives under.
    let (mixed, _, _) = run(&[
        "add v1 battery_terminal 2g",
        "add v1 Pb 2g",
        "add v1 water 5mL",
    ]);
    assert!(creep_for(vessel(&mixed), "Pb").is_none());
    // And no acid is in the vessel: the film is a fact about the object.
    assert!(post
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .all(|portion| portion.species.0 != "H2SO4" && portion.species.0 != "SO4-2"));
}

/// A dry post says nothing at all. The corrosion route needs an
/// electrolyte to be a circuit, and a film that has not been wetted is
/// not one.
#[test]
fn a_dry_post_is_not_a_cell() {
    let (bench, all, _) = run(&["add v1 battery_terminal 2g"]);
    assert!(kerotakis_core::corrosion::verdicts(vessel(&bench)).is_empty());
    assert!(!all
        .iter()
        .any(|event| matches!(event, Event::Corroded { .. })));
}

/// Every reviewed row on either side carries its citation and says the
/// lane is not cleared.
#[test]
fn both_battery_rows_carry_their_provenance() {
    for creep in kerotakis_core::corrosion::ELECTROLYTE_CREEP {
        assert!(creep.source.contains("PENDING REVIEW"));
        assert!(!creep.why.is_empty());
    }
    use kerotakis_core::material::{self, MaterialRole};
    let mut cells = 0;
    for recipe in material::all() {
        for role in &recipe.roles {
            if let MaterialRole::SealedCell {
                open_circuit_volts,
                reaction,
                why,
                boundary,
                source,
            } = role
            {
                cells += 1;
                assert!(*open_circuit_volts > 0.0);
                assert!(!reaction.is_empty() && !why.is_empty() && !boundary.is_empty());
                assert!(source.contains("PENDING REVIEW"));
                assert!(
                    source.contains("NO BRAND IS NAMED OR IMPLIED"),
                    "a battery is described by its chemistry"
                );
                assert!(
                    boundary.contains("NAMED AND NOT RUN"),
                    "the row must say the reaction is prose"
                );
                // A sealed cell must be sealed: the coherent-object role is
                // the case, and without it the components would be emptied
                // into the beaker.
                assert!(recipe
                    .roles
                    .iter()
                    .any(|other| matches!(other, MaterialRole::CoherentObject)));
            }
        }
    }
    assert_eq!(cells, 1, "one sealed cell on the shelf");
}
