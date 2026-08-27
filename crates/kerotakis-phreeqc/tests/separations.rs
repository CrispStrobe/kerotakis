//! Separations: filtering a precipitate and evaporating to crystallisation
//! — real lab workflow over computed chemistry.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, stack: &mut SolverStack, op: Operator) -> Vec<Event> {
    bench
        .step_with(op, stack, &ReactiveGroupScreen)
        .expect("step")
}

fn add(bench: &mut Bench, stack: &mut SolverStack, v: VesselId, key: &str, moles: f64) {
    step(
        bench,
        stack,
        Operator::Add {
            vessel: v,
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        },
    );
}

#[test]
fn filtering_separates_the_precipitate_from_the_filtrate() {
    // Make AgCl in solution, then filter: the solid stays, the ions pass.
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let (a, b) = (VesselId(0), VesselId(1));
    add(&mut bench, &mut stack, a, "water", 55.51);
    add(&mut bench, &mut stack, a, "NaCl", 0.01);
    add(&mut bench, &mut stack, a, "AgNO3", 0.01);
    let total_before = bench.vessel(a).unwrap().mass().0 + bench.vessel(b).unwrap().mass().0;

    let events = step(&mut bench, &mut stack, Operator::Filter { from: a, to: b });
    assert!(events.iter().any(|e| matches!(e, Event::Filtered { .. })));

    let residue = bench.vessel(a).unwrap();
    let filtrate = bench.vessel(b).unwrap();
    // Residue: only the solid AgCl.
    assert!(residue.contents.iter().all(|p| p.phase == Phase::Solid));
    assert!(residue.moles_of(&SpeciesId::new("AgCl")).0 > 0.0098);
    // Filtrate: water + spectator ions, no solid.
    assert!(filtrate.contents.iter().all(|p| p.phase != Phase::Solid));
    assert!(filtrate.moles_of(&SpeciesId::new("Na+")).0 > 0.0098);
    assert!(filtrate.moles_of(&SpeciesId::new("NO3-")).0 > 0.0098);
    // Mass conserved across the pair.
    // Conservation across successive solver runs is bounded by PHREEQC's
    // convergence tolerance (~1e-8 relative), not exact arithmetic.
    let total_after = residue.mass().0 + filtrate.mass().0;
    assert!(
        (total_after - total_before).abs() < 1e-4,
        "mass drift across filter beyond solver tolerance: before {total_before}, after {total_after}"
    );
}

#[test]
fn evaporating_brine_crystallises_salt() {
    // 2 mol NaCl dissolved in 1 kg water is well under saturation; boil off
    // 80% of the water and the solution passes the solubility limit —
    // halite crystallises, amount computed from the database
    // (~2 − 6.1×0.2 ≈ 0.8 mol).
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaCl", 2.0);
    assert!(
        bench.vessel(v).unwrap().moles_of(&SpeciesId::new("NaCl")).0 < 1e-9,
        "2 mol/kgw is fully dissolved"
    );

    let events = step(
        &mut bench,
        &mut stack,
        Operator::Evaporate {
            vessel: v,
            fraction: 0.8,
        },
    );
    assert!(events.iter().any(|e| matches!(e, Event::Evaporated { .. })));
    let crystallised = events
        .iter()
        .find_map(|e| match e {
            Event::Precipitated { species, moles, .. } if species.0 == "NaCl" => Some(moles.0),
            _ => None,
        })
        .expect("halite must crystallise out of the concentrated brine");
    assert!(
        crystallised > 0.4 && crystallised < 1.1,
        "expected ~0.8 mol NaCl to crystallise, got {crystallised}"
    );

    // Sodium conserved: dissolved + solid = 2 mol.
    let vessel = bench.vessel(v).unwrap();
    let total_na =
        vessel.moles_of(&SpeciesId::new("Na+")).0 + vessel.moles_of(&SpeciesId::new("NaCl")).0;
    assert!((total_na - 2.0).abs() < 1e-6);
}

#[test]
fn named_seawater_has_computed_salinity_and_leaves_salt() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let op = kerotakis_core::script::parse_op("add v1 Meerwasser 100mL")
        .expect("valid localized seawater command")
        .expect("operator");
    step(&mut bench, &mut stack, op);
    let solution = bench.vessels[0]
        .solution
        .as_ref()
        .expect("computed seawater solution");
    assert!(
        solution.ionic_strength > 0.4,
        "installed major salts must produce seawater-scale ionic strength: {}",
        solution.ionic_strength
    );

    let events = step(
        &mut bench,
        &mut stack,
        Operator::Evaporate {
            vessel: VesselId(0),
            fraction: 0.95,
        },
    );
    assert!(
        events.iter().any(|event| matches!(event,
            Event::Precipitated { species, moles, .. }
                if species.0 == "NaCl" && moles.0 > 0.01
        )),
        "concentrating named seawater must recover computed salt: {events:?}"
    );
}

#[test]
fn evaporating_a_mixture_flags_the_missing_vle() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.0);
    add(&mut bench, &mut stack, v, "ethanol", 1.0);
    let events = step(
        &mut bench,
        &mut stack,
        Operator::Evaporate {
            vessel: v,
            fraction: 0.5,
        },
    );
    assert!(
        events.iter().any(
            |e| matches!(e, Event::NotYetModeled { what, .. } if what.contains("vapour-liquid"))
        ),
        "co-evaporation of ethanol must be honestly flagged, got {events:?}"
    );
}

/// Boiling brine nearly dry is answerable, once the question is posed the
/// way the engine can take it.
///
/// PHREEQC speciates the `SOLUTION` block before it looks at
/// `EQUILIBRIUM_PHASES`, so a beaker whose salt cannot possibly all be
/// dissolved is asked an impossible question first and never reaches the
/// step that would precipitate it. At 99% evaporated the databases were
/// handed 100 mol/kgw nominal and all three refused — though the state
/// being asked about, mostly solid beside a saturated brine, is well
/// inside pitzer's range.
///
/// Posed the other way round — most of the salt in as solid, dissolving to
/// saturation — it is the same equilibrium and it solves. That recasting
/// happens only after a refusal, so the range that already worked is
/// untouched: 50% and 80% stay undersaturated with no crystals, and 90%
/// and 95% crystallise exactly as they did before.
#[test]
fn brine_boiled_almost_dry_still_answers() {
    let mut eq = stack();
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.534_276_991_396_059);
    add(&mut bench, &mut eq, v, "NaCl", 0.1);
    bench
        .step_with(
            Operator::Evaporate {
                vessel: v,
                fraction: 0.99,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("evaporate");

    let vessel = bench.vessel(v).expect("vessel");
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    assert!(
        solid > 0.08,
        "nearly all the salt should be crystals by now, got {solid:.4} mol"
    );
    let solution = vessel
        .solution
        .as_ref()
        .expect("the vessel must still be characterised, not refused");
    assert!(
        (solution.ionic_strength - 6.4).abs() < 1.0,
        "what is left should be saturated brine, not an impossible one: I = {}",
        solution.ionic_strength
    );
}
