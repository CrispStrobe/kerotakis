//! S6 (2026-11-28 sweep): KNO3 crystallises on cooling through the whole
//! production stack, a dilute solution does not, and `cool` stops at the
//! partial-freezing boundary it announces instead of running past it.

#![cfg(feature = "engine")]

use kerotakis_core::crystallisation;
use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const KNO3_G_PER_MOL: f64 = 101.1032;

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )))])
}

fn run(bench: &mut Bench, solvers: &mut SolverStack, line: &str) -> Vec<Event> {
    let op = parse_op(line).unwrap().expect(line);
    bench
        .step_with(op, solvers, &PermissiveScreen)
        .unwrap_or_else(|e| panic!("{line}: {e:?}"))
}

fn cool(bench: &mut Bench, solvers: &mut SolverStack, joules: f64) -> Vec<Event> {
    let vessel = bench.vessels[0].id;
    bench
        .step_with(
            Operator::Cool {
                vessel,
                energy: Joules(joules),
            },
            solvers,
            &PermissiveScreen,
        )
        .expect("cool")
}

fn moles(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn water_g(v: &Vessel) -> f64 {
    moles(v, "water", Phase::Liquid) * constants::WATER_MOLAR_MASS_G_PER_MOL
}

fn reasons(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key.clone()),
            _ => None,
        })
        .collect()
}

/// `salt_g` of KNO3 dissolved, booked on its ions, in `water_g` of liquid
/// water at `t_k`. Built directly so the start temperature is the one the
/// test names, not what a dissolution heat on `add` left behind.
fn solution(water_g: f64, salt_g: f64, t_k: f64) -> (Bench, SolverStack) {
    let mut bench = Bench::new();
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(
        SpeciesId::new("water"),
        Moles(water_g / constants::WATER_MOLAR_MASS_G_PER_MOL),
        Phase::Liquid,
    );
    let n = salt_g / KNO3_G_PER_MOL;
    v.deposit(SpeciesId::new("K+"), Moles(n), Phase::Aqueous);
    v.deposit(SpeciesId::new("NO3-"), Moles(n), Phase::Aqueous);
    v.temperature = Kelvin(t_k);
    bench.vessels.push(v);
    (bench, stack())
}

/// The dose that takes `v` to `target_k` when the curve salt crystallises
/// on the way, priced at the routed dissolution enthalpy.
fn dose_to(v: &Vessel, target_k: f64) -> f64 {
    let curve = crystallisation::curve_for("KNO3").unwrap();
    let dissolved = moles(v, "K+", Phase::Aqueous);
    let capacity = curve.capacity_mol(water_g(v), target_k).unwrap();
    let n = (dissolved - capacity).max(0.0);
    let mut end = v.clone();
    crystallisation::apply(&mut end, curve, n);
    let dh = crystallisation::dissolution_enthalpy_kj("KNO3").unwrap();
    end.energy_between(target_k, v.temperature.0) + n * dh * 1000.0
}

/// Berkeley 1904 at 20 C, read independently of the code: linear between
/// the 14.90 C (25.78 g) and 30.80 C (47.52 g) points.
fn berkeley_20c_g_per_100g() -> f64 {
    25.78 + (47.52 - 25.78) * (20.0 - 14.90) / (30.80 - 14.90)
}

#[test]
fn hot_saturated_kno3_cooled_to_20c_deposits_crystals_on_the_curve() {
    // 40 g in 50 mL saturates near 50 C; start at 70 C, unsaturated.
    let (mut bench, mut solvers) = solution(50.0, 40.0, 343.15);
    let dose = dose_to(&bench.vessels[0], 293.15);
    let water_before = water_g(&bench.vessels[0]);
    let events = cool(&mut bench, &mut solvers, dose);
    let v = &bench.vessels[0];
    assert!(
        (v.temperature.0 - 293.15).abs() < 0.5,
        "{} K; {:?}",
        v.temperature.0,
        reasons(&events)
    );
    let solid = moles(v, "KNO3", Phase::Solid);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Precipitated { species, .. } if species.0 == "KNO3"
        )),
        "{events:?}"
    );
    // Roughly the curve: 40 g less what 50 g of water holds at 20 C.
    let expected_g = 40.0 - berkeley_20c_g_per_100g() * water_before / 100.0;
    let deposited_g = solid * KNO3_G_PER_MOL;
    assert!(expected_g > 20.0);
    assert!(
        (deposited_g - expected_g).abs() < 0.03 * expected_g,
        "{deposited_g} g vs {expected_g} g"
    );
    // Potassium closes: crystals plus what stayed dissolved is the feed.
    let k = solid + moles(v, "K+", Phase::Aqueous) + moles(v, "KNO3", Phase::Aqueous);
    let fed = 40.0 / KNO3_G_PER_MOL;
    assert!((k - fed).abs() < 1e-3 * fed, "{k} vs {fed}");
    assert!(!reasons(&events)
        .iter()
        .any(|r| r.starts_with("not-modeled.crystallisation")));
}

#[test]
fn a_dilute_kno3_solution_cools_without_depositing_anything() {
    // 2 g in 50 mL saturates well below 0 C.
    let (mut bench, mut solvers) = solution(50.0, 2.0, 343.15);
    let dose = dose_to(&bench.vessels[0], 293.15);
    let events = cool(&mut bench, &mut solvers, dose);
    let v = &bench.vessels[0];
    assert!(
        (v.temperature.0 - 293.15).abs() < 0.5,
        "{}",
        v.temperature.0
    );
    assert_eq!(moles(v, "KNO3", Phase::Solid), 0.0);
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::Precipitated { .. })));
    assert!(!reasons(&events)
        .iter()
        .any(|r| r.starts_with("not-modeled.crystallisation")));
}

/// The sweep's t29, verbatim: 25 g KNO3 in 20 mL water at 80 C, then a
/// 5 kJ cool. It used to end at -67.9 C, liquid and clear.
#[test]
fn the_sweep_t29_case_crystallises_and_never_passes_the_freezing_boundary() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    for line in [
        "add v1 water 20mL @ 80C",
        "add v1 KNO3 25g @ 80C",
        "stir v1",
    ] {
        run(&mut bench, &mut solvers, line);
    }
    let events = run(&mut bench, &mut solvers, "cool v1 5kJ");
    let v = &bench.vessels[0];
    assert!(
        v.temperature.0 >= states::BRINE_MODEL_MIN_K - 1e-9,
        "{} K; {:?}",
        v.temperature.0,
        reasons(&events)
    );
    assert!(
        moles(v, "KNO3", Phase::Solid) > 0.0,
        "crystals: {:?}",
        reasons(&events)
    );
}

/// A solution whose freezing the bench cannot place above the boundary has
/// nothing to absorb a dose there: `cool` stops AT the announced boundary,
/// says so, and a further `cool` does not move it.
#[test]
fn cool_stops_at_the_partial_freezing_boundary_it_announces() {
    // Supersaturated far below the curve's floor, then a dose far beyond
    // what sensible cooling to the boundary needs.
    let (mut bench, mut solvers) = solution(20.0, 25.0, 263.0);
    let events = run(&mut bench, &mut solvers, "cool v1 50kJ");
    let boundary = states::BRINE_MODEL_MIN_K;
    let v = &bench.vessels[0];
    assert!(
        v.temperature.0 >= boundary - 1e-9,
        "{} K; {:?}",
        v.temperature.0,
        reasons(&events)
    );
    assert!(
        (v.temperature.0 - boundary).abs() < 1e-9,
        "{}",
        v.temperature.0
    );
    assert!(
        reasons(&events)
            .iter()
            .any(|r| r == "not-modeled.cooling-stopped-at-freezing-boundary"),
        "{:?}",
        reasons(&events)
    );
    let delivered = events.iter().find_map(|e| match e {
        Event::EnergyTransferred { delivered_j, .. } => Some(*delivered_j),
        _ => None,
    });
    assert!(delivered.is_some_and(|d| d < 50_000.0), "{delivered:?}");
    let again = run(&mut bench, &mut solvers, "cool v1 5kJ");
    let t = bench.vessels[0].temperature.0;
    assert!(t >= boundary - 1e-9, "{t} K; {:?}", reasons(&again));
}
