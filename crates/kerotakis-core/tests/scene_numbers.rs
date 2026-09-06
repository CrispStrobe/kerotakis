//! GUI-099: the numbers the stage was reconstructing, put on the wire.
//!
//! The animation audit (`docs/ANIMATION-AUDIT.md`) ends with a list of
//! quantities the engine computes and the wire did not carry, each of which
//! forced the bench to draw something client-side that was honest but
//! weaker than the number already in the solver. Every assertion below is
//! the same shape: change the thing the quantity is supposed to depend on,
//! and require the wire number to move with it. A field that does not move
//! when its driver does is the defect this whole file exists to catch.

use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, op: Operator) -> Vec<Event> {
    bench
        .step_with(op, &mut stack(), &PermissiveScreen)
        .expect("operator")
}

fn add(bench: &mut Bench, key: &str, moles: f64) -> Vec<Event> {
    step(
        bench,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        },
    )
}

fn scene(bench: &Bench) -> kerotakis_core::scene::SceneVessel {
    kerotakis_core::scene::scene_vessel(bench.vessel(VesselId(0)).expect("vessel 0"))
}

// ---------------------------------------------------------------- boiling

#[test]
fn an_open_beaker_of_water_stands_at_the_normal_boiling_point() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    let v = scene(&bench);
    let boiling = v.boiling_point_k.expect("water has a boiling point");
    assert!(
        (boiling - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-9,
        "an open beaker stands at {boiling} K"
    );
    let melting = v.melting_point_k.expect("water has a melting point");
    assert!((melting - kerotakis_core::states::WATER_FREEZING_K).abs() < 1e-9);
}

#[test]
fn a_vessel_under_partial_vacuum_reads_as_boiling_while_it_merely_sits() {
    // The defect this field exists for. A renderer with no standing value
    // fell back to pure water at one atmosphere, so a flask held at 50 kPa
    // and 355 K — genuinely, visibly boiling — drew as still water.
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    let atmospheric = scene(&bench).boiling_point_k.expect("a boiling point");
    step(
        &mut bench,
        Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(50_000.0),
            initial_volume: Liters(1.0),
        },
    );
    let reduced = scene(&bench).boiling_point_k.expect("a boiling point");
    assert!(
        reduced < atmospheric - 15.0,
        "50 kPa moved the plateau from {atmospheric:.2} K only to {reduced:.2} K"
    );
    // The steam tables put water at 50 kPa at 81.3 °C.
    assert!(
        (reduced - 273.15 - 81.3).abs() < 0.5,
        "50 kPa reads {:.2} °C",
        reduced - 273.15
    );
}

#[test]
fn the_standing_plateau_is_the_one_a_boil_reports() {
    // Two numbers that agree by inspection are two numbers. This asserts
    // they are one: the scene's standing value and `state_changed.at` come
    // from the same call.
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    step(
        &mut bench,
        Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(50_000.0),
            initial_volume: Liters(1.0),
        },
    );
    let standing = scene(&bench).boiling_point_k.expect("a boiling point");
    let events = step(
        &mut bench,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(30_000.0),
            source: None,
        },
    );
    let reported = events
        .iter()
        .find_map(|event| match event {
            Event::StateChanged {
                from: Phase::Liquid,
                to: Phase::Gas,
                at,
                ..
            } => Some(at.0),
            _ => None,
        })
        .expect("30 kJ passes the reduced boiling point");
    assert!(
        (standing - reported).abs() < 1e-9,
        "standing {standing} K vs reported {reported} K"
    );
}

#[test]
fn an_empty_vessel_claims_no_plateau_at_all() {
    let bench = Bench::new();
    let v = scene(&bench);
    assert!(v.boiling_point_k.is_none());
    assert!(v.melting_point_k.is_none());
}

// -------------------------------------------------------------- headspace

#[test]
fn only_a_vessel_that_owns_its_gas_reports_a_headspace() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 2.0);
    let open = scene(&bench);
    assert!(
        open.headspace_volume_l.is_none() && open.headspace_moles.is_none(),
        "an open vessel's headspace is the room, not its own"
    );

    step(
        &mut bench,
        Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.4),
        },
    );
    let sealed = scene(&bench);
    let volume = sealed.headspace_volume_l.expect("a sealed headspace");
    assert!(
        (volume - 0.4).abs() < 1e-9,
        "sealed over 0.4 L, reported {volume} L"
    );
    let moles = sealed.headspace_moles.expect("trapped air");
    assert!(
        moles > 0.0,
        "sealing traps the room air that occupied the space"
    );
}

#[test]
fn the_pistons_volume_follows_the_pressure_it_is_held_at() {
    // Squeeze the gas harder and the number the piston is drawn from has to
    // fall; a headspace that does not move with the pressure is exactly the
    // fixed `y=16` the audit found.
    let volume_at = |pressure_pa: f64| {
        let mut bench = Bench::new();
        step(
            &mut bench,
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(2.0),
                at: None,
            },
        );
        step(
            &mut bench,
            Operator::Regulate {
                vessel: VesselId(0),
                pressure: Pascal(pressure_pa),
                initial_volume: Liters(0.5),
            },
        );
        scene(&bench)
            .headspace_volume_l
            .expect("a pressure-controlled headspace")
    };
    let loose = volume_at(101_325.0);
    let squeezed = volume_at(303_975.0);
    assert!(
        squeezed < loose,
        "three atmospheres held {squeezed} L against {loose} L at one"
    );
}

// ------------------------------------------------------------ molar volume

#[test]
fn molar_volume_separates_a_fluffy_solid_from_a_dense_one() {
    let volume = |key: &str| {
        kerotakis_core::species::lookup(&SpeciesId::new(key))
            .expect("a shipped species")
            .molar_volume_l_per_mol()
            .expect("a shipped density")
    };
    // Calcite ~0.037 L/mol against halite ~0.027: the reason a mole of one
    // draws a bigger pile than a mole of the other.
    let calcite = volume("CaCO3");
    let halite = volume("NaCl");
    assert!(
        calcite > halite,
        "calcite {calcite} L/mol is not above halite {halite} L/mol"
    );
    assert!((0.02..0.06).contains(&calcite), "calcite {calcite} L/mol");
}

// --------------------------------------------------------- phase transition

#[test]
fn a_sublimation_is_not_a_boil_on_the_wire() {
    use kerotakis_core::ops::PhaseTransition;
    assert_eq!(
        PhaseTransition::between(Phase::Solid, Phase::Gas),
        Some(PhaseTransition::Sublimation)
    );
    assert_eq!(
        PhaseTransition::between(Phase::Liquid, Phase::Gas),
        Some(PhaseTransition::Boiling)
    );
    assert_eq!(
        PhaseTransition::between(Phase::Gas, Phase::Solid),
        Some(PhaseTransition::Deposition)
    );
    // A dissolved solute is in the liquid it is dissolved in.
    assert_eq!(
        PhaseTransition::between(Phase::Aqueous, Phase::Gas),
        Some(PhaseTransition::Boiling)
    );
    assert_eq!(PhaseTransition::between(Phase::Gas, Phase::Gas), None);
}

#[test]
fn a_phase_transition_says_how_much_moved() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    let events = step(
        &mut bench,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(60_000.0),
            source: None,
        },
    );
    let (kind, moles) = events
        .iter()
        .find_map(|event| match event {
            Event::StateChanged {
                from: Phase::Liquid,
                to: Phase::Gas,
                kind,
                moles,
                ..
            } => Some((*kind, *moles)),
            _ => None,
        })
        .expect("60 kJ boils 100 mL of water");
    assert_eq!(kind, Some(kerotakis_core::ops::PhaseTransition::Boiling));
    let moles = moles.expect("the amount that changed phase");
    assert!(
        moles.0 > 0.0,
        "a boil that moved no matter is not a boil: {moles:?}"
    );
}

// ---------------------------------------------------------------- corrosion

#[test]
fn corroded_extent_grows_with_the_rust_and_not_otherwise() {
    use kerotakis_core::corrosion::corroded_extent;
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    add(&mut bench, "Fe", 0.02);
    let clean = corroded_extent(bench.vessel(VesselId(0)).expect("vessel"), "Fe")
        .expect("iron has a gated corrosion reaction");
    assert!(clean.1 < 1e-9, "a fresh nail reads {:.4} corroded", clean.1);

    // Half the iron's worth of oxide: 4 Fe → 2 Fe2O3 is two irons per
    // formula unit, so 0.005 mol of oxide is 0.01 mol of iron.
    add(&mut bench, "Fe2O3", 0.005);
    let rusted = corroded_extent(bench.vessel(VesselId(0)).expect("vessel"), "Fe")
        .expect("iron has a gated corrosion reaction");
    assert!(
        (rusted.0 .0 - 0.01).abs() < 1e-6,
        "0.005 mol of Fe2O3 locks {:?}, not 0.01 mol of iron",
        rusted.0
    );
    assert!(
        (rusted.1 - 1.0 / 3.0).abs() < 1e-6,
        "0.01 mol locked beside 0.02 mol left is {:.4}, not a third",
        rusted.1
    );
}

#[test]
fn a_metal_with_no_gated_reaction_claims_no_extent() {
    use kerotakis_core::corrosion::corroded_extent;
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    add(&mut bench, "Cu", 0.02);
    assert!(
        corroded_extent(bench.vessel(VesselId(0)).expect("vessel"), "Cu").is_none(),
        "nothing here models what copper does in water, so nothing may claim an extent"
    );
}
