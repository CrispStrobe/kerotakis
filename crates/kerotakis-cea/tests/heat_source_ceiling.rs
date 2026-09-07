//! Nothing is heated above the thing heating it.
//!
//! `heat v1 40kJ` on ten grams of chalk used to be arithmetic and nothing
//! else: 40 kJ divided by 8.2 J/K put the crucible at 4913 °C, three times
//! hotter than any laboratory flame, and the Gibbs minimiser — correctly,
//! for that temperature — reported carbon monoxide. The energy was real;
//! the temperature was not, because no source of heat was modelled at all.
//!
//! These tests pin the fix. A burner has a temperature of its own, so
//! energy crosses into the vessel only while the vessel is colder than the
//! flame; beyond that the only route left for the rest of the dose is
//! chemistry that consumes heat at that temperature, and when that is
//! exhausted the remainder is reported as undelivered rather than turned
//! into degrees.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::apparatus::{HeatSource, BUNSEN_CEILING_K};
use kerotakis_core::render::{render_event, Register};
use kerotakis_core::*;

/// Standard enthalpy of the calcination CaCO₃(s) → CaO(s) + CO₂(g) at
/// 298.15 K, J/mol.
///
/// From standard enthalpies of formation: CaCO₃ (calcite) −1207.4 ± 1.3
/// kJ/mol and CaO (lime) −635.1 kJ/mol, both from Robie & Hemingway,
/// *Thermodynamic Properties of Minerals and Related Substances at 298.15 K
/// and 1 Bar Pressure*, U.S. Geological Survey Bulletin 2131 (1995), pp. 25
/// and 182; CO₂(g) −393.51 ± 0.13 kJ/mol, NIST Chemistry WebBook SRD 69
/// (CODATA review value, Cox, Wagman & Medvedev 1984). CaO cross-checks
/// against NIST-JANAF table Ca-027 at −635.089 kJ/mol.
///
///   (−635.1) + (−393.51) − (−1207.4) = +178.8 kJ/mol
///
/// The engine never sees this number: the CEA lane derives the same
/// chemistry from the NASA-9 polynomials in `vendor/nasa-cea/thermo.inp`,
/// which is why it is worth checking the two against each other here.
const CALCINATION_ENTHALPY_J_PER_MOL: f64 = 178_800.0;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        // The phase plateau: CEA declines while there is liquid water, so
        // the boiling beaker is this rung's job, not the minimiser's.
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
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
            stack,
            &PermissiveScreen,
        )
        .expect("add")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, v: VesselId, kj: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: v,
                energy: Joules(kj * 1000.0),
                source: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

struct Ledger {
    requested_j: f64,
    delivered_j: f64,
    sensible_j: f64,
    passes: u32,
    capped: bool,
    ceiling_k: f64,
    source: String,
}

fn ledger(events: &[Event]) -> Ledger {
    events
        .iter()
        .find_map(|event| match event {
            Event::EnergyTransferred {
                heating: true,
                requested_j,
                delivered_j,
                sensible_j,
                passes,
                capped,
                ceiling_k,
                source,
                ..
            } => Some(Ledger {
                requested_j: *requested_j,
                delivered_j: *delivered_j,
                sensible_j: *sensible_j,
                passes: *passes,
                capped: *capped,
                ceiling_k: ceiling_k.unwrap_or(f64::NAN),
                source: source.clone().unwrap_or_default(),
            }),
            _ => None,
        })
        .expect("a heating step reports what it delivered")
}

fn gas(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == key => Some(moles.0),
            _ => None,
        })
        .sum()
}

/// Everything the run said and everything it left behind, so a failing
/// assertion shows the numbers rather than only the verdict.
fn transcript(bench: &Bench, v: VesselId, events: &[Event]) -> String {
    let vessel = bench.vessel(v).expect("vessel");
    let mut out = String::new();
    for event in events {
        out.push_str("    ");
        out.push_str(&render_event(event, Register::LV3));
        out.push('\n');
    }
    out.push_str(&format!(
        "    -- final: {:.2} K ({:.1} °C), Cp {:.3} J/K\n",
        vessel.temperature.0,
        vessel.temperature.to_celsius(),
        vessel.heat_capacity(),
    ));
    for portion in &vessel.contents {
        out.push_str(&format!(
            "    -- holds {:.6} mol {} ({:?})\n",
            portion.moles.0, portion.species.0, portion.phase
        ));
    }
    out
}

// ── The scenario the owner ran ───────────────────────────────────────

#[test]
fn ten_grams_of_chalk_and_forty_kilojoules_stop_at_the_flame() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    add(&mut bench, &mut s, v, "CaCO3", 0.1);
    let events = heat(&mut bench, &mut s, v, 40.0);
    let seen = transcript(&bench, v, &events);
    let book = ledger(&events);
    let vessel = bench.vessel(v).expect("vessel");

    assert!(
        vessel.temperature.0 <= BUNSEN_CEILING_K + 1e-6,
        "a burner cannot drive a crucible past its own flame ({BUNSEN_CEILING_K} K), \
         but this one reached {:.2} K\n{seen}",
        vessel.temperature.0
    );
    assert_eq!(book.source, "Bunsen burner", "the default source\n{seen}");
    assert!(
        (book.ceiling_k - BUNSEN_CEILING_K).abs() < 1e-9,
        "the ceiling is reported\n{seen}"
    );

    // Carbon monoxide is what 5000 K looks like. At a burner's temperature
    // carbon dioxide is stable, and the absence of CO is the whole point.
    let co = gas(&events, "CO");
    assert!(
        co < 1e-6,
        "no carbon monoxide from chalk on a burner, but {co:.6} mol appeared\n{seen}"
    );

    // Fully calcined: quicklime and carbon dioxide, one mole each per mole
    // of chalk.
    let chalk_left = vessel.moles_of(&SpeciesId::new("CaCO3")).0;
    let lime = vessel.moles_of(&SpeciesId::new("CaO")).0;
    let co2 = gas(&events, "CO2");
    assert!(
        chalk_left < 1e-3,
        "the chalk should be gone, {chalk_left:.6} mol left\n{seen}"
    );
    assert!(
        (lime - 0.1).abs() < 1e-3,
        "0.1 mol of quicklime expected, got {lime:.6}\n{seen}"
    );
    assert!(
        (co2 - 0.1).abs() < 1e-3,
        "0.1 mol of carbon dioxide expected, got {co2:.6}\n{seen}"
    );

    // The dose could not all be delivered, and the bench says so exactly.
    let undelivered = book.requested_j - book.delivered_j;
    assert!(
        undelivered > 0.0,
        "40 kJ into 8.2 J/K of chalk cannot all be delivered from a flame\n{seen}"
    );
    assert!(
        (book.requested_j - book.delivered_j - undelivered).abs() < 1.0,
        "requested − delivered is the undelivered remainder\n{seen}"
    );
    assert!(!book.capped, "the pass cap should not be reached\n{seen}");
    assert!(
        book.passes > 1,
        "reaching the flame and being pulled back by the chemistry takes \
         more than one pass\n{seen}"
    );

    // lv2's split: what arrived is partly warmth the crucible still holds
    // and partly the price of breaking the carbonate apart.
    assert!(
        book.sensible_j < book.delivered_j,
        "some of the {:.1} J that arrived was spent on chemistry, not \
         warming, but sensible heat is reported as {:.1} J\n{seen}",
        book.delivered_j,
        book.sensible_j
    );

    // Energy: the crucible's own books, and there is nothing left outside
    // them.
    //
    // What the crucible costs is three things, and it took two changes to
    // be able to write all three down:
    //
    //     the sensible heat the lime still holds     7 529 J
    //     0.1 mol x 178.8 kJ/mol                    17 880 J
    //     the sensible heat the CO2 carried out      7 783 J
    //                                               --------
    //                                               33 192 J
    //
    // The burner used to book 13 941 J of that - 58 % - and the hole was
    // one lane away. `ThermalEquilibrator` solved an ADIABATIC charge that
    // admitted eight times the vessel's own moles of air and let that air's
    // sensible heat pay for the decomposition, though room air is at 298 K
    // and `Vessel::heat_capacity()` never held it. Closing that took it to
    // 22 538 J against a TWO-term ledger of 24 075 J, 93.6 %.
    //
    // That 93.6 % was two errors of opposite sign, and both are now named.
    // `Vessel::heat_capacity()` was a room-temperature constant - 82.3
    // J/(mol.K) for calcite, 42.0 for lime - while the NASA-9 polynomials
    // the solver reads rise to about 139 and 53 by 1500 K. The burner was
    // billed at 25 C prices for a crucible at 1500 C, and BOTH sides of the
    // ratio were wrong with it: the ledger charged too little to warm the
    // charge, and `vessel.enthalpy()` under-reported what the charge was
    // holding. Both integrate the same curves now.
    //
    // With that gone, the second error stopped hiding behind it. The carbon
    // dioxide leaves at the temperature it formed at and takes its sensible
    // heat with it - a kiln really does pay that - and the two-line ledger
    // simply never named it. Naming it closes the balance to 99.5 %.
    //
    // The 0.5 % that is left has a sign and a reason: the exhaust term here
    // is charged at the crucible's FINAL temperature, and some of the CO2
    // left on earlier passes when the crucible was cooler. So the accounted
    // figure is a slight over-estimate, and `delivered < accounted` below is
    // an assertion about that direction rather than a formality.
    let warming = vessel.enthalpy().0;
    let chemistry = 0.1 * CALCINATION_ENTHALPY_J_PER_MOL;
    // The gas's own sensible heat, from the same NASA-9 record the
    // minimiser used. `h` is referenced so that h(298.15) is the formation
    // enthalpy, which makes the difference a pure sensible heat.
    let co2 = kerotakis_cea::nasa9::db()
        .species
        .get("CO2")
        .expect("thermo.inp has carbon dioxide");
    let exhaust = 0.1
        * (co2
            .h(vessel.temperature.0)
            .expect("CO2 enthalpy at the ceiling")
            - co2.h(298.15).expect("CO2 enthalpy at 298.15 K"));
    let accounted = warming + chemistry + exhaust;
    assert!(
        exhaust > 7000.0 && exhaust < 8500.0,
        "0.1 mol of CO2 taken from 25 C to 1500 C carries about 7.8 kJ out \
         of the crucible, this says {exhaust:.1} J\n{seen}"
    );
    assert!(
        book.delivered_j < accounted,
        "the burner cannot deliver more than the crucible costs: delivered \
         {:.1} J against warming {warming:.1} J plus calcination \
         {chemistry:.1} J plus exhaust {exhaust:.1} J = {accounted:.1} J\n{seen}",
        book.delivered_j
    );
    assert!(
        book.delivered_j > 0.99 * accounted,
        "the burner should pay for what the crucible cost: {accounted:.1} J of \
         warming, calcination and hot exhaust against {:.1} J booked, which is \
         {:.1} % and leaves a bigger hole than charging the exhaust at the \
         final temperature accounts for\n{seen}",
        book.delivered_j,
        100.0 * book.delivered_j / accounted
    );

    // The split the event reports is exactly the energy it says arrived.
    assert!(
        (book.sensible_j + (book.delivered_j - book.sensible_j) - book.delivered_j).abs() < 1e-6,
        "the lv2 split adds up\n{seen}"
    );
}

#[test]
fn five_kilojoules_is_delivered_whole_because_the_chalk_stays_cold() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    add(&mut bench, &mut s, v, "CaCO3", 0.1);
    let events = heat(&mut bench, &mut s, v, 5.0);
    let seen = transcript(&bench, v, &events);
    let book = ledger(&events);
    let vessel = bench.vessel(v).expect("vessel");

    assert!(
        (book.delivered_j - book.requested_j).abs() < 1.0,
        "5 kJ fits under the flame and is delivered whole\n{seen}"
    );
    assert_eq!(book.passes, 1, "one pass, nothing to chunk\n{seen}");
    assert!(
        vessel.temperature.0 > Kelvin::STANDARD.0,
        "the chalk got warmer\n{seen}"
    );
    assert!(
        vessel.temperature.0 <= BUNSEN_CEILING_K + 1e-6,
        "still under the flame\n{seen}"
    );
    // 5 kJ used to reach 908 K, on a crucible billed 8.19 J/K all the way
    // up. Calcite's own curve takes it from 8.38 J/K at 25 C to 12.1 by
    // 900 K, so the same dose now reaches about 775 K - which is the
    // point of the change, and it is also why less of the chalk goes than
    // it did. The crucible pays for its own calcination out of its own
    // heat: the room it stands in does not chip in
    // (`gibbs::OpenAtmosphere`).
    let chalk_left = vessel.moles_of(&SpeciesId::new("CaCO3")).0;
    assert!(
        chalk_left > 0.09,
        "the chalk survives 5 kJ: {chalk_left:.6} mol left of 0.1\n{seen}"
    );
    assert!(
        vessel.temperature.0 < 1000.0,
        "nowhere near the flame: {:.2} K\n{seen}",
        vessel.temperature.0
    );
}

#[test]
fn a_crucible_stopped_half_way_can_be_heated_again() {
    // The state a small dose leaves: some carbonate, some lime, standing in
    // the same air. Handing THAT back to the Gibbs minimiser failed at
    // every temperature — calcium has no gaseous carrier, so it has to be
    // shared between two solids, and the solve let one of them grow past
    // the whole calcium budget and then dropped both. It was never seen
    // because nothing could reach the state: a burner used to calcine ten
    // grams of chalk in a single pass.
    //
    // It is reachable in one line of script, so it is a test.
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    add(&mut bench, &mut s, v, "CaCO3", 0.1);
    // 7 kJ rather than 5: on its own heat-capacity curve the crucible costs
    // about a quarter more to warm than the room-temperature constant said,
    // and 5 kJ no longer reaches the temperature where the carbonate starts
    // to go. The state this test needs is a half-calcined crucible, and the
    // dose that leaves one is now a bigger dose.
    let first = heat(&mut bench, &mut s, v, 7.0);
    let half = bench.vessel(v).expect("vessel");
    assert!(
        half.moles_of(&SpeciesId::new("CaCO3")).0 > 0.01
            && half.moles_of(&SpeciesId::new("CaO")).0 > 1e-4,
        "5 kJ should leave both phases standing: {:?}\n{}",
        half.contents,
        transcript(&bench, v, &first)
    );

    let again = heat(&mut bench, &mut s, v, 40.0);
    let seen = transcript(&bench, v, &again);
    assert!(
        !again
            .iter()
            .any(|event| matches!(event, Event::SolverFailed { .. })),
        "the minimiser must have an answer for a half-calcined crucible\n{seen}"
    );
    let vessel = bench.vessel(v).expect("vessel");
    assert!(
        vessel.moles_of(&SpeciesId::new("CaCO3")).0 < 1e-3,
        "and the rest of the chalk gives way: {:.6} mol left\n{seen}",
        vessel.moles_of(&SpeciesId::new("CaCO3")).0
    );
    assert!(
        (vessel.moles_of(&SpeciesId::new("CaO")).0 - 0.1).abs() < 1e-3,
        "0.1 mol of quicklime, all told\n{seen}"
    );
}

#[test]
fn a_candle_is_a_lower_ceiling_than_a_burner() {
    let mut hot = Bench::new();
    let mut cool = Bench::new();
    let mut s = stack();
    let mut t = stack();
    let v = VesselId(0);
    add(&mut hot, &mut s, v, "CaCO3", 0.1);
    add(&mut cool, &mut t, v, "CaCO3", 0.1);

    let on_a_burner = hot
        .step_with(
            Operator::Heat {
                vessel: v,
                energy: Joules(40_000.0),
                source: Some(HeatSource::bunsen_burner()),
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("burner");
    let on_a_candle = cool
        .step_with(
            Operator::Heat {
                vessel: v,
                energy: Joules(40_000.0),
                source: Some(HeatSource::candle()),
            },
            &mut t,
            &PermissiveScreen,
        )
        .expect("candle");

    let burner = ledger(&on_a_burner);
    let candle = ledger(&on_a_candle);
    assert_eq!(candle.source, "candle");
    assert!(
        candle.ceiling_k < burner.ceiling_k,
        "a candle is cooler than a burner: {} vs {}",
        candle.ceiling_k,
        burner.ceiling_k
    );
    assert!(
        cool.vessel(v).expect("vessel").temperature.0
            <= kerotakis_core::apparatus::CANDLE_CEILING_K + 1e-6,
        "nothing in a candle flame ends up hotter than the candle: {:.2} K\n{}",
        cool.vessel(v).expect("vessel").temperature.0,
        transcript(&cool, v, &on_a_candle)
    );
}

// ── The beaker on the burner ────────────────────────────────────────

#[test]
fn a_hundred_millilitres_of_water_and_fifty_kilojoules_boils_rather_than_glows() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    // 100 mL ≈ 5.551 mol.
    add(&mut bench, &mut s, v, "water", 5.5508);
    let events = heat(&mut bench, &mut s, v, 50.0);
    let seen = transcript(&bench, v, &events);
    let book = ledger(&events);
    let vessel = bench.vessel(v).expect("vessel");

    assert!(
        (book.delivered_j - book.requested_j).abs() < 1.0,
        "a beaker of water is nowhere near the flame's temperature, so all \
         50 kJ cross\n{seen}"
    );
    assert!(
        vessel.temperature.0 <= 374.0,
        "the plateau holds the water at its boiling point, not above it: \
         {:.2} K\n{seen}",
        vessel.temperature.0
    );
    // 50 kJ − 5.5508 mol × 75.3 J/(mol·K) × 75 K of warming ≈ 18.6 kJ into
    // vapour, at 40.65 kJ/mol (CRC, `states::WATER_H_VAP`) ≈ 0.46 mol.
    let steam = gas(&events, "water");
    assert!(
        (steam - 0.46).abs() < 0.046,
        "about 0.46 mol of steam expected, got {steam:.4}\n{seen}"
    );
    assert_eq!(book.passes, 1, "no chunking needed below the flame\n{seen}");
}

// ── Cooling is untouched ────────────────────────────────────────────

#[test]
fn cooling_still_bounds_on_the_vessels_own_heat_and_names_no_source() {
    let mut bench = Bench::new();
    let mut s = stack();
    let v = VesselId(0);
    add(&mut bench, &mut s, v, "water", 5.5508);
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: v,
                energy: Joules(1_000_000.0),
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("cool");
    assert!(events.iter().any(|event| matches!(
        event,
        Event::EnergyTransferred {
            heating: false,
            source: None,
            ceiling_k: None,
            requested_j,
            delivered_j,
            ..
        } if delivered_j < requested_j
    )));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::NotYetModeled { .. })),
        "the absolute-zero bound still explains itself: {events:?}"
    );
}
