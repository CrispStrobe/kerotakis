//! What "nothing happened" is made of.
//!
//! `Event::DidNotIgnite` carried a vessel id and nothing else, which made
//! it the one row of the animation audit that could not be closed from
//! the client: with no quantity on the wire there is nothing for a visual
//! to be a function of, and anything drawn would have been a picture of
//! the word. Its sibling `FlameStarved` carries the fuel, what burned and
//! the oxygen fraction, and is drawn from those.
//!
//! Four absences, and a learner is doing a different experiment in each.
//! Each one is pinned here, against a solver stack that is a fixture
//! rather than a chemistry engine — see below for why that is the honest
//! way to test this and not a shortcut.

use kerotakis_core::ops::DidNotIgniteReason;
use kerotakis_core::vessel::Headspace;
use kerotakis_core::*;

/// A chemistry engine that looks at the vessel and finds no reaction.
///
/// This is a fixture and it is the RIGHT fixture, because the thing under
/// test is a distinction the bench draws with exactly this question.
/// `DidNotIgnite` may only be said when a chemistry solver examined the
/// vessel and found nothing; when no solver claims the state the bench
/// must say `NotYetModeled` instead, because "it does not burn" is a
/// claim about the world and "nobody looked" is a claim about us. A real
/// engine that declines is therefore not a substitute — it produces the
/// other event. What is needed is a solver that looks and reports
/// nothing, which is what this is.
///
/// (CI taught this the direct way: the first version of the `NoFuel` case
/// held a beaker of water in a flame and expected `NoFuel`. Water goes
/// past the aqueous model's 300 °C ceiling, nothing claimed it, and the
/// bench correctly answered `NotYetModeled`.)
struct LooksAndFindsNothing;

impl Equilibrator for LooksAndFindsNothing {
    fn name(&self) -> &'static str {
        "looks-and-finds-nothing"
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        true
    }

    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(Vec::new())
    }
}

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(LooksAndFindsNothing),
    ])
}

/// The absence, and everything it says about itself.
struct Absence {
    reason: DidNotIgniteReason,
    fuel: Option<String>,
    moles: Option<f64>,
    oxygen: Option<f64>,
    gap: Option<f64>,
}

fn ignite(vessel: Vessel) -> Absence {
    let mut bench = Bench::new();
    bench.vessels[0] = vessel;
    let mut stack = stack();
    let events = bench
        .step_with(
            Operator::Ignite {
                vessel: VesselId(0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("ignite");
    events
        .iter()
        .find_map(|event| match event {
            Event::DidNotIgnite {
                reason,
                fuel,
                fuel_moles,
                oxygen_fraction,
                gap_k,
                ..
            } => Some(Absence {
                reason: *reason,
                fuel: fuel.as_ref().map(|f| f.0.clone()),
                moles: fuel_moles.as_ref().map(|m| m.0),
                oxygen: *oxygen_fraction,
                gap: *gap_k,
            }),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no DidNotIgnite among {events:?}"))
}

fn sealed(contents: &[(&str, f64, Phase)], temperature_k: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "flask");
    v.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    v.temperature = Kelvin(temperature_k);
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v.refresh_pressure();
    v
}

/// A beaker of water, held in a flame: nothing in there is a fuel.
///
/// This is the case the old sentence — "not everything burns" — was
/// actually true of, and it is now the only one it is said about. The
/// client draws nothing for it, which is the point: there is no fuel to
/// scale a wisp by, so the event says so rather than offering a zero.
#[test]
fn a_beaker_of_water_has_no_fuel_in_it() {
    let mut beaker = Vessel::new(VesselId(0), "beaker");
    beaker.deposit(SpeciesId::new("water"), Moles(0.5), Phase::Liquid);
    let absence = ignite(beaker);

    assert_eq!(absence.reason, DidNotIgniteReason::NoFuel);
    assert_eq!(absence.fuel, None, "there is no candidate to name");
    assert_eq!(absence.moles, None);
    assert_eq!(absence.gap, None, "and so no gap to any temperature");
    // The oxygen fraction is still a true reading, and an open beaker
    // stands in the room's air rather than in an empty ledger.
    let oxygen = absence.oxygen.expect("an open beaker reports the air");
    assert!(
        (oxygen - kerotakis_core::combustion::ROOM_OXYGEN_FRACTION).abs() < 1e-9,
        "{oxygen}"
    );
}

/// A candle sealed in a flask that has no air: the wax is still there and
/// the oxygen is not.
///
/// "Not everything burns" is a lie about candle wax, and this is the
/// vessel it used to be told over. The fuel is named and measured, so a
/// client can say what did not burn instead of shrugging at the vessel.
#[test]
fn a_sealed_flask_with_no_air_names_the_fuel_it_could_not_light() {
    let absence = ignite(sealed(&[("paraffin", 0.02, Phase::Solid)], 298.15));

    assert_eq!(absence.reason, DidNotIgniteReason::NoOxygen);
    assert_eq!(absence.fuel.as_deref(), Some("paraffin"));
    assert!((absence.moles.expect("how much wax") - 0.02).abs() < 1e-12);
    let oxygen = absence.oxygen.expect("a sealed vessel owns its gas");
    assert!(oxygen < 1e-9, "there is no air in there at all: {oxygen}");
    assert_eq!(
        absence.gap, None,
        "the temperature is not the reason, so no gap is claimed"
    );
}

/// Wax and oxygen, and a vessel too cool for either to matter.
///
/// This is the one drawable absence: the fuel is there, the air is there,
/// and the only thing missing is the temperature — so the gap IS the
/// answer, and a client scales a wisp by the fuel and thins it by the
/// gap. Paraffin vapour lights at 473 K and the flask is at 298.15 K.
#[test]
fn wax_in_air_and_too_cool_carries_the_gap() {
    let absence = ignite(sealed(
        &[("paraffin", 0.02, Phase::Solid), ("O2", 0.005, Phase::Gas)],
        298.15,
    ));

    assert_eq!(absence.reason, DidNotIgniteReason::BelowAutoignition);
    assert_eq!(absence.fuel.as_deref(), Some("paraffin"));
    let gap = absence.gap.expect("the gap is the answer");
    assert!((gap - (473.0 - 298.15)).abs() < 1e-9, "{gap}");
    let oxygen = absence.oxygen.expect("a sealed vessel owns its gas");
    assert!(oxygen > 0.99, "the flask is pure oxygen: {oxygen}");
}

/// Fuel, air, hot enough — and a solver looked and burned none of it.
///
/// The bench does not know why, and `NotModelled` is it saying so rather
/// than inventing a fourth reason out of the three readings it has. It is
/// also the default for an event from before any of these fields existed,
/// which knew exactly this much.
#[test]
fn fuel_in_air_above_its_point_and_still_unburnt_is_not_a_claim() {
    // 600 K is well above paraffin's 473 K, and the spark is reverted to
    // this temperature rather than to room temperature.
    let absence = ignite(sealed(
        &[("paraffin", 0.02, Phase::Solid), ("O2", 0.005, Phase::Gas)],
        600.0,
    ));

    assert_eq!(absence.reason, DidNotIgniteReason::NotModelled);
    assert_eq!(
        absence.gap, None,
        "there is no gap: it is past the temperature, not short of it"
    );
    // The reading that is true is still carried; only the claim is not.
    assert_eq!(absence.fuel.as_deref(), Some("paraffin"));
}

/// The fuel that would have caught first is the one named.
///
/// Both tables are read — the vapours `GAS_AUTOIGNITION` gates and the
/// curated `FUELS` NASA CEA cannot name — and the lowest autoignition
/// temperature wins, because that is the one a rising flame reaches
/// first. Reading only the gas table would have called a candle
/// unburnable in exactly the vessels the curated table exists for.
#[test]
fn the_candidate_is_the_most_flammable_thing_present() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.temperature = Kelvin(300.0);
    vessel.deposit(SpeciesId::new("methane"), Moles(0.01), Phase::Gas);
    let gas = kerotakis_core::combustion::ignition_candidate(&vessel).expect("methane is a fuel");
    assert_eq!(gas.fuel.0, "methane");

    // Paraffin lights at 473 K, well below methane's 810 K, so the wax is
    // what the flame is offered.
    vessel.deposit(SpeciesId::new("paraffin"), Moles(0.02), Phase::Solid);
    let wax = kerotakis_core::combustion::ignition_candidate(&vessel).expect("wax is a fuel");
    assert_eq!(wax.fuel.0, "paraffin");
    assert!((wax.autoignition_k - 473.0).abs() < 1e-9);
    assert!((wax.moles.0 - 0.02).abs() < 1e-12);

    // Nothing on either table is no candidate at all, rather than a zero.
    let mut water = Vessel::new(VesselId(1), "beaker");
    water.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    assert!(kerotakis_core::combustion::ignition_candidate(&water).is_none());
}
