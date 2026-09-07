//! What changes, and what must not, when the heat ledger stops multiplying
//! by a constant and starts integrating a curve.
//!
//! The curves themselves are checked in `heat_capacity_curves`. These tests
//! are about the ledger that spends them: that a beaker of water still costs
//! what every textbook says it costs, that heat put in and taken out again
//! comes back to where it started, and that a species with no curve behaves
//! exactly as it did before curves existed.

use kerotakis_core::script::parse_op;
use kerotakis_core::species::{Phase, REGISTRY};
use kerotakis_core::*;

fn bench_with(script: &[&str]) -> Bench {
    let mut bench = Bench::new();
    for line in script {
        bench
            .step(parse_op(line).unwrap().unwrap())
            .unwrap_or_else(|e| panic!("{line}: {e:?}"));
    }
    bench
}

fn vessel(bench: &Bench) -> &vessel::Vessel {
    bench.vessel(VesselId(0)).expect("v1")
}

#[test]
fn a_hundred_millilitres_of_water_still_costs_thirty_one_kilojoules() {
    // The number on every worksheet. If this moves, the change is wrong
    // however good the crucible looks.
    let bench = bench_with(&["add v1 water 100mL"]);
    let v = vessel(&bench);
    let joules = v.energy_between(298.15, 373.15);
    assert!(
        (joules / 1000.0 - 31.4).abs() / 31.4 < 0.01,
        "25 to 100 C for 100 mL of water should still be about 31.4 kJ, \
         the ledger makes it {:.3} kJ",
        joules / 1000.0
    );
}

#[test]
fn enthalpy_is_the_integral_the_ledger_spends() {
    // `enthalpy()` and `energy_between` are the same claim written twice;
    // if they disagree, one lane of the bench is charging a different price
    // from another and every balance built across them is wrong.
    let bench = bench_with(&["add v1 water 100mL", "heat v1 20kJ"]);
    let v = vessel(&bench);
    let by_hand = v.energy_between(units::Kelvin::STANDARD.0, v.temperature.0);
    assert!(
        (v.enthalpy().0 - by_hand).abs() < 1e-6,
        "enthalpy {} against energy_between {}",
        v.enthalpy().0,
        by_hand
    );
}

#[test]
fn heat_put_in_and_taken_out_again_comes_back() {
    // With `T + j/Cp` in one direction and `T - j/Cp` in the other this was
    // exact by construction. With a curve it is exact only if both
    // directions integrate the same function, which is the property worth
    // pinning.
    let mut bench = bench_with(&["add v1 Cu 200g"]);
    let started = vessel(&bench).temperature.0;
    bench
        .step(parse_op("heat v1 20kJ").unwrap().unwrap())
        .unwrap();
    let hot = vessel(&bench).temperature.0;
    assert!(
        hot > started + 100.0,
        "200 g of copper given 20 kJ should get properly hot, reached {hot} K"
    );
    let spent = vessel(&bench).energy_between(started, hot);
    bench
        .step(Operator::Cool {
            vessel: VesselId(0),
            energy: units::Joules(spent),
        })
        .unwrap();
    let back = vessel(&bench).temperature.0;
    assert!(
        (back - started).abs() < 0.05,
        "taking back the {spent:.1} J that warmed it from {started:.2} K to \
         {hot:.2} K should return it, landed at {back:.2} K"
    );
}

#[test]
fn a_burner_delivers_what_the_charge_actually_costs() {
    // The #488 defect in miniature. A vessel driven to a source's ceiling
    // must be charged the area under its own heat capacity between here and
    // there — not its bench-temperature capacity times the span, which for
    // anything fired is a third too little.
    let bench = bench_with(&["add v1 CaO 5.6g"]);
    let v = vessel(&bench);
    let integrated = v.energy_between(v.temperature.0, 1773.15);
    let flat = v.heat_capacity() * (1773.15 - v.temperature.0);
    assert!(
        integrated > flat * 1.15,
        "0.1 mol of lime taken to 1500 C costs {integrated:.0} J integrated \
         against {flat:.0} J flat; if those agreed there would be nothing here \
         to fix"
    );
    // And the inverse agrees with the forward direction.
    let landed = v.temperature_after(integrated);
    assert!(
        (landed - 1773.15).abs() < 0.01,
        "putting exactly that in should land at 1773.15 K, landed at {landed}"
    );
}

#[test]
fn a_species_with_no_curve_is_charged_exactly_what_it_always_was() {
    // The guarantee that makes this change reviewable: nothing without a
    // curve moves at all.
    let sugar = REGISTRY
        .iter()
        .find(|s| s.key == "sucrose")
        .expect("sucrose");
    assert!(sugar.heat_capacity_polys.is_empty());
    let bench = bench_with(&["add v1 sucrose 34.2g"]);
    let v = vessel(&bench);
    let moles = 34.2 / sugar.molar_mass;
    let integrated = v.energy_between(298.15, 398.15);
    let flat = moles * sugar.heat_capacity * 100.0;
    assert!(
        (integrated - flat).abs() < 1e-6,
        "with no curve the ledger is still Cp times delta T: {integrated} \
         against {flat}"
    );
}

#[test]
fn the_vessel_and_the_species_tables_agree() {
    // `Vessel::heat_capacity_at` sums what `states::heat_capacity_at`
    // returns per portion. Two tables that could disagree is how a bench
    // ends up cooling ice at liquid water's price, so pin them together.
    let bench = bench_with(&["add v1 water 100mL"]);
    let v = vessel(&bench);
    let water = REGISTRY.iter().find(|s| s.key == "water").expect("water");
    let moles = 100.0 * water.density / water.molar_mass;
    for t in [280.0, 298.15, 350.0, 372.0] {
        let vessel_side = v.heat_capacity_at(t);
        let species_side = moles * states::heat_capacity_at(water, Phase::Liquid, t);
        assert!(
            (vessel_side - species_side).abs() < 1e-6,
            "at {t} K the vessel says {vessel_side} J/K and the species table \
             says {species_side}"
        );
    }
}

#[test]
fn cooling_past_what_a_vessel_holds_is_still_refused() {
    // The floor is now measured against the integral rather than a
    // rectangle, and it must still be a floor.
    let bench = bench_with(&["add v1 water 100mL", "cool v1 1000kJ"]);
    let v = vessel(&bench);
    assert!(
        v.temperature.0 >= 0.0,
        "a vessel cannot be cooled below absolute zero, reached {}",
        v.temperature.0
    );
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 water 100mL").unwrap().unwrap())
        .unwrap();
    let events = bench
        .step(parse_op("cool v1 1000kJ").unwrap().unwrap())
        .unwrap();
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::EnergyTransferred {
                heating: false,
                requested_j,
                delivered_j,
                ..
            } if *delivered_j < *requested_j
        )),
        "the bench must say it could not remove what was asked"
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::NotYetModeled { .. })),
        "and it must say why"
    );
}
