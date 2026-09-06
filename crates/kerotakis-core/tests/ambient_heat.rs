//! The room exists.
//!
//! Every one of these is a transcript that used to be wrong. Ten grams of
//! dry ice in an open beaker stayed ten grams of dry ice for ever, because
//! nothing on the bench ever gave it a joule; a beaker of hot water stayed
//! hot for ever for the same reason. `clock::AmbientClock` is Newton's law
//! of cooling against the room, and these tests are its arithmetic — every
//! one states the `h·A` it expects and where the number comes from, so a
//! failure says which half is wrong.
//!
//! The one number under all of them, for the default 250 mL beaker:
//!
//! ```text
//! A  = pi*0.070*0.095 + pi*0.070^2/4 = 0.02474 m^2
//! hA = 7.0 W/(m^2 K) * 0.02474 m^2   = 0.17318 W/K
//! ```

use kerotakis_core::phase_route::PhaseRouteEquilibrator;
use kerotakis_core::species::Phase;
use kerotakis_core::*;

/// `h·A` for the beaker every test below uses, W/K.
const BEAKER_UA: f64 = 0.173_18;
const ROOM_K: f64 = 298.15;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

/// The same stack with the solvent model in it, for the one lesson that is
/// water's own: `phase_route` deliberately does not own ice.
fn solvent_stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
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

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, n: f64) -> Vec<Event> {
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
        .expect("add")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
                source: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn wait(bench: &mut Bench, stack: &mut SolverStack, seconds: f64) -> Vec<Event> {
    bench
        .step_with(Operator::Wait { seconds }, stack, &PermissiveScreen)
        .expect("wait")
}

/// The transcript this whole tranche came from: `add v1 dry_ice 10g`,
/// `wait 10min`, and ten minutes later still 0.2272 mol of dry ice at
/// −78.5 °C, in an OPEN beaker, in a warm room.
///
/// The arithmetic the fix has to satisfy:
///
/// ```text
/// 10 g / 44.009 g/mol            = 0.22723 mol
/// latent  = 0.22723 * 25 200 J/mol   = 5 726 J     (phase_route::SUBLIMATION_ENTHALPIES)
/// driving = 298.15 - 194.65 K        = 103.5 K     (its 1 atm sublimation point)
/// Q       = 0.17318 W/K * 103.5 K    = 17.9 W
/// t       = 5 726 J / 17.9 W         = 320 s
/// ```
///
/// Five and a half minutes, so half an hour is not a close-run thing —
/// which is the point: the old answer was not slightly slow, it was
/// infinitely slow.
#[test]
fn ten_grams_of_dry_ice_do_not_survive_half_an_hour_in_a_warm_room() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let charged = 10.0 / 44.009;
    add(&mut bench, &mut stack, "dry_ice", charged);

    // It arrives at its own sublimation point and the beaker goes with it.
    assert!(
        (vessel_of(&bench).temperature.0 - 194.65).abs() < 0.5,
        "dry ice arrives at {} K",
        vessel_of(&bench).temperature.0
    );
    assert!((moles(&bench, "dry_ice", Phase::Solid) - charged).abs() < 1e-9);

    let events = wait(&mut bench, &mut stack, 1800.0);
    let left = moles(&bench, "dry_ice", Phase::Solid);
    assert!(
        left <= 0.10 * charged,
        "after half an hour {left} mol of {charged} mol is left; the room should have taken \
         5.7 kJ of it at 17.9 W"
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
        "the heat has to be spent as sublimation, not as a temperature: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "CO2")),
        "and the carbon dioxide leaves an open beaker: {events:?}"
    );
    // One announcement per vessel per wait, however many sub-steps it took.
    assert_eq!(
        events
            .iter()
            .filter(
                |e| matches!(e, Event::TemperatureChanged { vessel, .. } if *vessel == VesselId(0))
            )
            .count(),
        1,
        "{events:?}"
    );

    wait(&mut bench, &mut stack, 1800.0);
    let ended = vessel_of(&bench).temperature.0;
    assert!(
        (ended - ROOM_K).abs() <= 10.0,
        "an empty beaker an hour later should be near the room, not at {ended} K"
    );
}

/// While there is dry ice left the beaker does NOT warm up: the heat goes
/// into the phase change, which is the observation the demonstration is
/// for. Ten minutes in, the block is going but the thermometer has barely
/// moved off −78.5 °C.
#[test]
fn the_thermometer_stays_at_the_sublimation_point_while_the_block_lasts() {
    let mut bench = Bench::new();
    let mut stack = stack();
    // Enough that half an hour cannot finish it: 100 g needs 57 kJ and the
    // room delivers 17.9 W, which is nearly an hour.
    add(&mut bench, &mut stack, "dry_ice", 100.0 / 44.009);
    wait(&mut bench, &mut stack, 1800.0);

    let left = moles(&bench, "dry_ice", Phase::Solid);
    assert!(left > 0.0, "the block should not be gone yet");
    let t = vessel_of(&bench).temperature.0;
    assert!(
        (t - 194.65).abs() < 1.0,
        "with solid CO2 still in the beaker the temperature is pinned at its sublimation \
         point, not {t} K"
    );
    // 1800 s * 17.9 W = 32 kJ, which is 1.28 mol of sublimation out of 2.27.
    let gone = 100.0 / 44.009 - left;
    let expected = BEAKER_UA * (ROOM_K - 194.65) * 1800.0 / 25_200.0;
    assert!(
        (gone - expected).abs() < 0.15 * expected,
        "{gone} mol went in half an hour; h·A·ΔT/ΔH_sub says {expected} mol"
    );
}

/// A beaker of hot water cools, and by how much is arithmetic:
///
/// ```text
/// 50 mL water = 50/18.015          = 2.7755 mol
/// C           = 2.7755 * 75.3 J/(mol K) = 209.0 J/K
/// tau         = 209.0 / 0.17318    = 1207 s
/// dT          = 45 K * (1 - e^(-600/1207)) = 45 * 0.3917 = 17.6 K
/// ```
///
/// So 70 °C becomes 52 °C in ten minutes — the 10–30 K band a bench sheet
/// would accept, and nothing like the "still 70 °C" the bench used to say.
#[test]
fn a_beaker_of_seventy_degree_water_loses_about_eighteen_kelvin_in_ten_minutes() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let water_moles = 50.0 / 18.015;
    add(&mut bench, &mut stack, "water", water_moles);

    let capacity = water_moles * 75.3;
    heat(&mut bench, &mut stack, 45.0 * capacity);
    let started = vessel_of(&bench).temperature.0;
    assert!(
        (started - 343.15).abs() < 0.5,
        "the water should be at 70 °C, not {started} K"
    );

    wait(&mut bench, &mut stack, 600.0);
    let ended = vessel_of(&bench).temperature.0;
    let dropped = started - ended;
    let newton = 45.0 * (1.0 - (-600.0 * BEAKER_UA / capacity).exp());
    assert!(
        (dropped - newton).abs() < 0.5,
        "cooled {dropped} K; Newton's law with hA = {BEAKER_UA} W/K and C = {capacity} J/K \
         says {newton} K"
    );
    assert!(
        (10.0..=30.0).contains(&dropped),
        "cooled {dropped} K, which is outside the 10–30 K a bench sheet would accept"
    );
}

/// The room takes heat through the wall of a sealed flask exactly as it
/// does through an open one. What the seal keeps in is MATTER.
#[test]
fn a_sealed_vessel_cools_and_keeps_its_gas() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "CO2", 0.05);
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
    // 200 J into the sealed gas — the dose of CO2 plus the litre of air
    // the seal shut in with it, about 2.3 J/K between them — is a rise of
    // some 87 K. The flask's time constant against the room is then
    // 2.3 / 0.17318 = 13 s, so ten minutes is more than forty of them and
    // it comes back to the room exactly.
    heat(&mut bench, &mut stack, 200.0);
    let hot = vessel_of(&bench).temperature.0;
    assert!(hot > 370.0, "the gas should be hot, not {hot} K");
    // Measured after the seal, not against the 0.05 mol that went in: the
    // litre of room air the seal shut in beside the dose brings its own
    // 400 ppm of carbon dioxide with it, which is 1.6e-5 mol and is the
    // room's, not the dose's. What this test is about is whether any of it
    // LEAVES.
    let sealed_in = moles(&bench, "CO2", Phase::Gas);

    let events = wait(&mut bench, &mut stack, 600.0);
    let cooled = vessel_of(&bench).temperature.0;
    assert!(
        hot - cooled > 70.0,
        "a sealed flask is not a vacuum flask: {hot} K -> {cooled} K"
    );
    assert!(
        (cooled - ROOM_K).abs() < 1.0,
        "forty time constants is the room: {cooled} K"
    );
    assert!(
        (moles(&bench, "CO2", Phase::Gas) - sealed_in).abs() < 1e-9,
        "the seal keeps the gas: {sealed_in} mol became {} mol",
        moles(&bench, "CO2", Phase::Gas)
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::GasEvolved { .. })),
        "nothing may leave a sealed vessel: {events:?}"
    );
}

/// Nothing to exchange, nothing to say. The bench must not narrate a
/// beaker that is already the temperature of the room it stands in.
#[test]
fn a_room_temperature_beaker_is_silent_for_thirty_seconds() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "water", 50.0 / 18.015);
    let events = wait(&mut bench, &mut stack, 30.0);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::TemperatureChanged { .. })),
        "{events:?}"
    );
}

/// Ice melts at 0 °C on the room's heat, and it melts on the SAME
/// arithmetic dry ice sublimes on.
///
/// The two routes belong to different engines on purpose — `phase_route`
/// owns the cryogens and `solve::StateEquilibrator` owns the solvent — and
/// the ambient clock drives both rather than carrying a table of its own.
/// So the numbers here are the same shape as the dry ice ones:
///
/// ```text
/// 50 g ice = 2.7755 mol
/// C(ice)   = 2.7755 * 37.7 J/(mol K)  = 104.6 J/K   (ICE's Cp, not water's)
/// latent   = 2.7755 * 6 010 J/mol     = 16 681 J
/// driving  = 298.15 - 273.15          = 25 K
/// Q        = 0.17318 W/K * 25 K       = 4.33 W
/// t        = 16 681 / 4.33            = 3 852 s = 64 min
/// ```
///
/// So an hour is not enough to finish it, and that is the observation: the
/// glass of ice water sits at 0 °C for as long as there is ice in it.
#[test]
fn a_beaker_of_ice_melts_on_the_rooms_heat_and_sits_at_zero_while_it_does() {
    let mut bench = Bench::new();
    let mut stack = solvent_stack();
    let water_moles = 50.0 / 18.015;
    add(&mut bench, &mut stack, "water", water_moles);

    // Enough to reach the freezing point, freeze it all, and chill the ice
    // a little below: 5.2 kJ sensible + 16.7 kJ latent, plus a margin.
    bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(23_000.0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("cool");
    let ice_before = moles(&bench, "water", Phase::Solid);
    assert!(
        ice_before > 0.5 * water_moles,
        "the beaker should be mostly ice: {ice_before} mol of {water_moles} mol"
    );

    wait(&mut bench, &mut stack, 3600.0);

    let ice_after = moles(&bench, "water", Phase::Solid);
    let melted = ice_before - ice_after;
    assert!(
        melted > 0.0,
        "the room has to melt some of it: {ice_before} mol -> {ice_after} mol"
    );
    assert!(
        ice_after > 0.0,
        "and not all of it in an hour: h·A·ΔT is 4.33 W and the block asks 16.7 kJ"
    );
    let t = vessel_of(&bench).temperature.0;
    assert!(
        (t - 273.15).abs() < 1.0,
        "with ice still in the beaker the thermometer reads the melting point, not {t} K"
    );
    // The energy the room delivered, read back off what melted, against
    // h·A·ΔT·t. The ice first has to be warmed to the melting point, so
    // the melting gets slightly less than the whole hour.
    let spent = melted * 6_010.0;
    let offered = BEAKER_UA * (ROOM_K - 273.15) * 3600.0;
    assert!(
        spent < offered && spent > 0.7 * offered,
        "{spent} J went into melting; the room offered {offered} J over the hour"
    );
}
