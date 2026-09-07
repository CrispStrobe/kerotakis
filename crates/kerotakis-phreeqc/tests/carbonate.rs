//! Carbonate chemistry with an open vessel: the fizz is computed.
//!
//! An ACIDIFIED solution is driven far past saturation by the reaction
//! itself, and the CO2 leaves within the step that makes it — that is the
//! fizz, and it is chemistry rather than transport.
//!
//! Carbon LEAVING an open vessel is equilibrium, as it always was: the
//! adapter offers a CO2(g) phase holding zero moles, so a supersaturated
//! solution degasses to the atmospheric value and an undersaturated one
//! does nothing.
//!
//! Carbon ARRIVING is new, and it is a rate. A phase with no moles in it
//! cannot be dissolved from, so room air could previously only ever take
//! carbon away — and the offer was gated on carbon already being present,
//! so a beaker of hydroxide was not offered the room at all. EXP-57 gives
//! that direction to `GasExchangeClock`, which means a vessel has to be
//! WAITED on before it has taken anything up.

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

fn wait_with(bench: &mut Bench, solver: &mut dyn Equilibrator, seconds: f64) -> Vec<Event> {
    bench
        .step_with(Operator::Wait { seconds }, solver, &ReactiveGroupScreen)
        .expect("step")
}

fn ph_of(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph
}

/// Chemistry the test author had to learn from the engine: a 0.05 m
/// bicarbonate solution IS supersaturated versus atmospheric pCO2 and does
/// lose CO2 in an open beaker, drifting basic — but only a modest fraction,
/// nothing like an acidified fizz.
///
/// EXP-57 left this alone on purpose. Whether the loss is a fizz or a seep
/// over days is still the equilibrium answer here; only the INWARD
/// direction became a rate. See `GasExchangeClock::advance`.
#[test]
fn baking_soda_alone_degasses_slowly_not_dramatically() {
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
    let ph = ph_of(&bench, v);
    assert!(
        ph > 8.0 && ph < 9.7,
        "degassed bicarbonate drifts basic, got {ph}"
    );
}

/// The other half of EXP-57, and the one that was simply missing: the
/// atmospheric reservoir used to be offered only when carbon was ALREADY
/// dissolved, so a beaker of sodium hydroxide — which is the classic thing
/// that goes off in room air — could not take up CO2 at all. Air does not
/// check first.
///
/// The hydroxide is invisible in `contents`, because the aqueous tail
/// carries free base as `solute_charge` rather than as an NaOH portion.
/// The clock reads the MEASURED free hydroxide the solver persists, which
/// is the only reading that survives the step boundary.
#[test]
fn a_beaker_of_alkali_carbonates_in_the_room() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "NaOH", 0.01);

    let ph_fresh = ph_of(&bench, v);
    assert!(
        ph_fresh > 11.0,
        "bench-strength alkali starts strong, got {ph_fresh}"
    );

    let mut absorbed = 0.0;
    for _ in 0..30 {
        let events = wait_with(&mut bench, &mut eq, 86_400.0);
        absorbed += events
            .iter()
            .filter_map(|e| match e {
                Event::GasAbsorbed { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
                _ => None,
            })
            .sum::<f64>();
    }
    assert!(
        absorbed > 0.0,
        "an open beaker of hydroxide must take up room CO2; it took {absorbed} mol"
    );
    let ph_stood = ph_of(&bench, v);
    assert!(
        ph_stood < ph_fresh,
        "carbonating an alkali brings its pH down: {ph_fresh} -> {ph_stood} \
         (absorbed {absorbed} mol)"
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

#[test]
fn a_carbonate_solution_never_precipitates_dry_ice() {
    // The engine-side half of `derived::a_condensed_gas_is_never_a_database_mineral`.
    //
    // `dry_ice` is a registry SOLID whose formula is CO2, and the phase
    // matcher pairs registry solids with database phases by composition.
    // A beaker full of carbonate is the case where that would show:
    // there is plenty of carbon and plenty of oxygen, the solution really
    // is supersaturated in CO2 against the atmosphere, and if dry ice had
    // become a candidate phase the solver could have "precipitated" it at
    // 25 °C — 253 kelvin below the temperature it can exist at. It must
    // not, the vessel must never hold a grain of it, and the carbon that
    // leaves must leave as gas.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "NaHCO3", 0.05);
    let events = add_with(&mut bench, &mut eq, v, "CH3COOH", 0.06);

    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::Precipitated { species, .. } if species.0 == "dry_ice"
        )),
        "dry ice is not a mineral and a fizzing beaker must not make one: {events:#?}"
    );
    assert!(
        bench
            .vessel(v)
            .unwrap()
            .contents
            .iter()
            .all(|portion| portion.species.0 != "dry_ice"),
        "no dry ice may end up in a room-temperature beaker: {:#?}",
        bench.vessel(v).unwrap().contents
    );
    assert!(
        evolved_co2(&events) > 0.03,
        "and the carbon that left must still leave as gas"
    );
}
