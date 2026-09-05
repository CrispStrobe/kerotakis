//! The aqueous heat balance, through the whole stack.
//!
//! `enthalpy`'s unit tests price hand-built states; these run real beakers
//! and read the thermometer, which is the only place the three heats that
//! used to be charged separately can be caught disagreeing.
//!
//! Every figure here is checked against something outside this bench —
//! a literature band, or the same beaker filled in a different order —
//! rather than against a number this code produced. A test pinned to our
//! own output cannot fail when our output is wrong, and the enthalpy work
//! this file covers was out by a factor of two at one point while every
//! internal check was green.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

/// Pour a list of reagents into one beaker and report the final
/// temperature. 100 g of water throughout, so a kJ is about 2.4 K.
fn pour(order: &[(&str, f64)]) -> f64 {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    for (key, moles) in order {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(*moles),
                    at: None,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("add");
    }
    bench.vessel(v).unwrap().temperature.0
}

const ROOM: f64 = 298.15;
const WATER: f64 = 5.55;

/// Neutralisation is not a special case in the balance — it is one
/// hydroxide going to water — and it must not depend on which bottle went
/// in first.
///
/// The whole heat is dissolving the alkali (−44.5 kJ/mol, registry) plus
/// neutralising it (−55.81 kJ/mol, the database's own figure for
/// `H₂O = OH⁻ + H⁺` reversed). 0.01 mol of that in 100 g of water is
/// about 2.4 K, and the two orders must agree exactly: nothing about a
/// state function knows the order.
#[test]
fn a_strong_acid_and_a_strong_base_warm_the_same_either_way_round() {
    let acid_first = pour(&[("water", WATER), ("HCl", 0.01), ("NaOH", 0.01)]);
    let base_first = pour(&[("water", WATER), ("NaOH", 0.01), ("HCl", 0.01)]);
    assert!(
        (acid_first - base_first).abs() < 1e-6,
        "pouring order changed the heat: {acid_first} K vs {base_first} K"
    );
    let rise = acid_first - ROOM;
    assert!(
        (2.0..3.0).contains(&rise),
        "0.01 mol of dissolution-plus-neutralisation in 100 g should be about 2.4 K, got {rise}"
    );
}

/// The cold pack, against a number anyone can look up: ammonium nitrate
/// dissolves endothermically at about +25.7 kJ/mol, so 0.1 mol in 100 g
/// of water drops it about 6 K.
#[test]
fn an_instant_cold_pack_gets_cold() {
    let t = pour(&[("water", WATER), ("NH4NO3", 0.1)]);
    let drop = ROOM - t;
    assert!(
        (5.0..7.5).contains(&drop),
        "0.1 mol of ammonium nitrate in 100 g should cool it about 6 K, got {drop}"
    );
}

/// The reaction this work was for.
///
/// Baking soda and vinegar is endothermic — a child can put a hand on the
/// beaker and feel it — and the bench used to report it getting WARMER.
/// Nothing was ever red, because no test asserted a temperature here.
///
/// The band is the literature's (+25 to +30 kJ/mol), not this bench's
/// output, and that is deliberate: the registry's enthalpy of dissolution
/// for NaHCO₃ is itself flagged as not reproducible (+16.7 against a
/// tabulated ~17.5 and a formation-enthalpy ~18.7). Pinned to the band, a
/// correction to that datum moves the answer without breaking this test.
/// Pinned to our own number, the correction would have gone red and been
/// "fixed" by editing the expectation.
#[test]
fn the_volcano_gets_cold() {
    let t = pour(&[("water", WATER), ("NaHCO3", 0.02), ("CH3COOH", 0.02)]);
    let drop = ROOM - t;
    assert!(drop > 0.0, "the volcano warmed by {} K", -drop);
    // 0.02 mol at +25..30 kJ/mol in 100 g of water.
    assert!(
        (1.1..1.6).contains(&drop),
        "expected about 1.3 K of cooling for 0.02 mol, got {drop}"
    );
}

/// **A known gap, pinned rather than asserted as correct.**
///
/// The same three reagents, poured the other way round, barely move the
/// thermometer. It is not the balance that differs: it is which solver
/// does the chemistry.
///
/// Adding the powder to vinegar leaves a `NaHCO3` portion beside the acid,
/// which is exactly what the CURATED row `NaHCO₃ + CH₃COOH` matches — so
/// the curated solver converts them and deposits the products, and it
/// charges no enthalpy. By the time the aqueous tail runs, the reaction
/// has already happened and its before-state is the aftermath. Adding the
/// powder to water first dissolves it, the curated row can no longer see
/// its named reactant, and the tail does the chemistry itself, where the
/// balance prices it correctly.
///
/// So the heat of a step is not the aqueous tail's to own: solvers above
/// it also change what the vessel holds. The complete fix is to take the
/// balance across the whole step rather than across one solver's call —
/// which would price curated and reaction-family products the same way,
/// with no second enthalpy path anywhere. That is a change to where the
/// balance is invoked, not to the balance, and it is deliberately not
/// smuggled in here.
///
/// This asserts what the bench does today, so it turns red when that
/// changes in either direction.
#[test]
fn the_curated_route_is_still_missing_its_heat() {
    let through_the_tail = ROOM - pour(&[("water", WATER), ("NaHCO3", 0.02), ("CH3COOH", 0.02)]);
    let through_curated = ROOM - pour(&[("water", WATER), ("CH3COOH", 0.02), ("NaHCO3", 0.02)]);
    assert!(
        through_the_tail > 1.0,
        "the tail's route should cool properly, got {through_the_tail} K"
    );
    assert!(
        through_curated < 0.1,
        "FIXED: the curated route now carries heat ({through_curated} K) — \
         delete this test and assert the two orders agree"
    );
}
