//! EXP-33: sublimation as a separation, and the hydrate mass ledger.
//!
//! Both of these are taught as arithmetic, so both are tested as arithmetic.
//! The sublimation quest's whole claim is that one component left and the
//! other did not; the crucible lesson's whole claim is that the missing mass
//! is exactly the water. Neither survives a rounding, so neither is allowed
//! one.

use kerotakis_core::phase_route::PhaseRouteEquilibrator;
use kerotakis_core::species::Phase;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn vessel_of(bench: &Bench) -> &vessel::Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    vessel_of(bench)
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, n: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(n),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("add");
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn cool(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("cool")
}

/// Total mass of everything the vessel still holds.
fn mass(bench: &Bench) -> f64 {
    vessel_of(bench).mass().0
}

fn molar_mass(key: &str) -> f64 {
    species::lookup(&SpeciesId::new(key)).unwrap().molar_mass
}

// ── sublimation ────────────────────────────────────────────────────

#[test]
fn ammonium_chloride_leaves_and_common_salt_stays() {
    // The separation quest, in six lines. Heat the mixture past 338 °C and
    // one of the two components is simply gone from the crucible.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    add(&mut bench, &mut stack, "NaCl", 0.1);

    let before = mass(&bench);
    let events = heat(&mut bench, &mut stack, 60_000.0);

    assert!(
        moles(&bench, "NH4Cl", Phase::Solid) < 1e-9,
        "the ammonium chloride should have sublimed away"
    );
    assert!(
        (moles(&bench, "NaCl", Phase::Solid) - 0.1).abs() < 1e-12,
        "the salt does not sublime and must be untouched"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged {
                from: Phase::Solid,
                to: Phase::Gas,
                ..
            }
        )),
        "the phase route must be reported, not silent: {events:#?}"
    );

    // Mass accounting across the open boundary: what left is exactly the
    // ammonium chloride, to the digit.
    let lost = before - mass(&bench);
    assert!(
        (lost - 0.2 * molar_mass("NH4Cl")).abs() < 1e-9,
        "lost {lost} g, expected {} g",
        0.2 * molar_mass("NH4Cl")
    );
}

#[test]
fn a_sealed_vessel_keeps_the_vapour_and_gives_it_back_on_cooling() {
    // The cold-finger half. Sealed, nothing crosses the boundary at all, so
    // the balance never moves — and the solid comes back when it cools.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(1.0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("seal");

    let sealed_mass = mass(&bench);
    heat(&mut bench, &mut stack, 60_000.0);
    assert!(
        moles(&bench, "NH4Cl", Phase::Gas) > 0.19,
        "a sealed vessel keeps its vapour"
    );
    assert!(
        (mass(&bench) - sealed_mass).abs() < 1e-9,
        "a sealed vessel conserves mass exactly through a phase change"
    );

    cool(&mut bench, &mut stack, 200_000.0);
    assert!(
        (moles(&bench, "NH4Cl", Phase::Solid) - 0.2).abs() < 1e-9,
        "deposition should return every mole to the solid"
    );
    assert!((mass(&bench) - sealed_mass).abs() < 1e-9);
}

#[test]
fn nothing_sublimes_at_bench_temperature() {
    // The route must not fire on a cold bench, or every ammonium chloride
    // test in the suite would start evaporating.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    assert!((moles(&bench, "NH4Cl", Phase::Solid) - 0.2).abs() < 1e-12);
    assert!(moles(&bench, "NH4Cl", Phase::Gas) < 1e-12);
}

// ── the hydrate ledger ─────────────────────────────────────────────

#[test]
fn heating_the_pentahydrate_drives_off_exactly_five_waters() {
    // The crucible lesson: weigh, heat, weigh. The difference is the water,
    // and "exactly" means exactly.
    let mut bench = Bench::new();
    let mut stack = stack();
    let n = 0.04;
    add(&mut bench, &mut stack, "chalcanthite", n);

    let before = mass(&bench);
    let expected_before = n * molar_mass("chalcanthite");
    assert!((before - expected_before).abs() < 1e-12);

    let events = heat(&mut bench, &mut stack, 40_000.0);

    let dehydrated = events
        .iter()
        .find_map(|e| match e {
            Event::Dehydrated { water, hydrate, .. } => Some((water.0, hydrate.0.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no dehydration reported: {events:#?}"));
    assert_eq!(dehydrated.1.as_str(), "chalcanthite");
    assert!(
        (dehydrated.0 - 5.0 * n).abs() < 1e-12,
        "five waters per formula unit, got {}",
        dehydrated.0
    );

    assert!(moles(&bench, "chalcanthite", Phase::Solid) < 1e-12);
    assert!((moles(&bench, "CuSO4", Phase::Solid) - n).abs() < 1e-12);

    // The residue weighs the anhydrous salt and nothing else.
    let after = mass(&bench);
    assert!(
        (after - n * molar_mass("CuSO4")).abs() < 1e-9,
        "residue {after} g, expected {} g",
        n * molar_mass("CuSO4")
    );
    // And the loss is the water, computed independently of the engine.
    let lost = before - after;
    assert!(
        (lost - 5.0 * n * molar_mass("water")).abs() < 1e-9,
        "lost {lost} g, expected {} g of water",
        5.0 * n * molar_mass("water")
    );
}

#[test]
fn the_water_goes_back_in_and_the_ledger_closes_both_ways() {
    // Round trip. A drop of water on the white powder and the blue returns,
    // with the same mass the crucible started with.
    let mut bench = Bench::new();
    let mut stack = stack();
    let n = 0.02;
    add(&mut bench, &mut stack, "chalcanthite", n);
    let start = mass(&bench);

    heat(&mut bench, &mut stack, 40_000.0);
    assert!((moles(&bench, "CuSO4", Phase::Solid) - n).abs() < 1e-12);

    // Cool it back down, then give it back exactly its own water.
    cool(&mut bench, &mut stack, 200_000.0);
    let events = bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(5.0 * n),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add water");

    assert!(
        events.iter().any(|e| matches!(e, Event::Hydrated { .. })),
        "the crystal should take its water back: {events:#?}"
    );
    assert!(
        (moles(&bench, "chalcanthite", Phase::Solid) - n).abs() < 1e-9,
        "every formula unit should be a pentahydrate again"
    );
    assert!(
        (mass(&bench) - start).abs() < 1e-9,
        "the round trip must return to the starting mass: {} vs {start}",
        mass(&bench)
    );
}

#[test]
fn plenty_of_water_dissolves_the_salt_instead_of_hydrating_it() {
    // The stated boundary, pinned: past the crystal's own stoichiometric
    // demand this bench stops calling it a hydrate, because dissolution is
    // what really happens and the aqueous engine owns that.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "chalcanthite", 0.01);
    heat(&mut bench, &mut stack, 40_000.0);
    cool(&mut bench, &mut stack, 200_000.0);

    let events = bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(5.0),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add water");
    assert!(
        !events.iter().any(|e| matches!(e, Event::Hydrated { .. })),
        "a beaker of water is not a hydration: {events:#?}"
    );
}

#[test]
fn epsomite_carries_its_seven_waters_through_the_same_ledger() {
    // The hydrate machinery is general: it reads the water count off the
    // formula, so the salt that was already on the shelf works too.
    let mut bench = Bench::new();
    let mut stack = stack();
    let n = 0.03;
    add(&mut bench, &mut stack, "epsomite", n);
    let before = mass(&bench);
    heat(&mut bench, &mut stack, 60_000.0);

    assert!((moles(&bench, "MgSO4", Phase::Solid) - n).abs() < 1e-12);
    let lost = before - mass(&bench);
    assert!(
        (lost - 7.0 * n * molar_mass("water")).abs() < 1e-9,
        "lost {lost} g, expected seven waters ({} g)",
        7.0 * n * molar_mass("water")
    );
}

#[test]
fn the_hydrate_survives_a_warm_bench_and_only_goes_at_its_own_temperature() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "chalcanthite", 0.02);
    // A few hundred joules is a warm beaker, not a crucible.
    heat(&mut bench, &mut stack, 300.0);
    assert!(
        (moles(&bench, "chalcanthite", Phase::Solid) - 0.02).abs() < 1e-12,
        "the crystal water is not driven off by a warm bench"
    );
}
