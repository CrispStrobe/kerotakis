//! Carbonate chemistry with an open vessel: the fizz is computed. CO2
//! escapes through an equilibrium phase pinned at atmospheric partial
//! pressure — supersaturation bubbles out, an undersaturated solution stays
//! quiet, and the balance sees the mass leave.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn add_with(
    bench: &mut Bench,
    solver: &mut dyn Equilibrator,
    v: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            solver,
            &ReactiveGroupScreen,
        )
        .expect("step")
}

fn evolved_co2(events: &[Event]) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum()
}

#[test]
fn baking_soda_alone_degasses_slowly_not_dramatically() {
    // Chemistry the test author had to learn from the engine: a 0.5 m
    // bicarbonate solution IS supersaturated versus atmospheric pCO2 and
    // does lose CO2 in an open beaker, drifting basic — but only a modest
    // fraction, nothing like an acidified fizz. (Whether it *bubbles* or
    // seeps out over hours is kinetics — L5's job; equilibrium only says
    // where it ends.)
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    let events = add_with(&mut bench, &mut eq, v, "NaHCO3", 0.05);

    let co2 = evolved_co2(&events);
    assert!(
        co2 > 0.005 && co2 < 0.02,
        "open-vessel equilibrium releases a modest fraction (~23%), got {co2} mol"
    );
    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph;
    assert!(
        ph > 8.0 && ph < 9.7,
        "degassed bicarbonate drifts basic, got {ph}"
    );
}

#[test]
fn vinegar_and_baking_soda_fizz() {
    // THE school reaction. Acid protonates bicarbonate, the solution
    // supersaturates in CO2, and it bubbles out until the solution sits at
    // atmospheric pCO2 — all from the database.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "NaHCO3", 0.05);
    let mass_before = bench.vessel(v).unwrap().mass().0;
    let events = add_with(&mut bench, &mut eq, v, "CH3COOH", 0.06);

    let co2 = evolved_co2(&events);
    // Most, not all: at open-vessel equilibrium some CO2 stays dissolved
    // (Henry's law) — the "flat soda still has some" fact.
    assert!(
        co2 > 0.035 && co2 <= 0.05,
        "most of the 0.05 mol carbonate should leave as CO2, got {co2} mol"
    );

    // The balance notices: ~2 g of gas left the open vessel (net of the
    // acid that was added).
    let mass_after = bench.vessel(v).unwrap().mass().0;
    let acid_added = 0.06 * 60.052;
    let lost = mass_before + acid_added - mass_after;
    assert!(
        (lost - co2 * 44.009).abs() < 0.2,
        "mass loss ({lost:.2} g) should equal the escaped CO2 ({:.2} g)",
        co2 * 44.009
    );

    // Excess weak acid leaves the solution mildly acidic.
    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph;
    assert!(
        ph > 3.0 && ph < 6.0,
        "excess acetic acid over spent bicarbonate, got pH {ph}"
    );
}

#[test]
fn strong_acid_on_baking_soda_also_fizzes() {
    // Same fizz through the inorganic (wateq4f) route.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "NaHCO3", 0.02);
    let events = add_with(&mut bench, &mut eq, v, "HCl", 0.03);

    let co2 = evolved_co2(&events);
    assert!(
        co2 > 0.015,
        "strong acid must expel the carbonate as CO2, got {co2} mol"
    );
}

#[test]
fn dissolving_baking_soda_cools_the_water() {
    // ΔH_dis = +16.7 kJ/mol (endothermic): 0.2 mol into 200 mL cools by
    // ~4 K — the classic cold-pack observation.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 11.1);
    add_with(&mut bench, &mut eq, v, "NaHCO3", 0.2);
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        t < 22.0 && t > 18.0,
        "endothermic dissolution should cool ~4 K, got {t:.1} °C"
    );
}
