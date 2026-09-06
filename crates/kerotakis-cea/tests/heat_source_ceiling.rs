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

    // Energy conservation: what crossed is the warming plus the calcination.
    let warming = vessel.enthalpy().0;
    let chemistry = 0.1 * CALCINATION_ENTHALPY_J_PER_MOL;
    let accounted = warming + chemistry;
    assert!(
        (book.delivered_j - accounted).abs() <= 0.05 * accounted.abs(),
        "delivered {:.1} J should be warming {:.1} J plus calcination \
         {:.1} J = {:.1} J, within 5%\n{seen}",
        book.delivered_j,
        warming,
        chemistry,
        accounted
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

// ── TEMPORARY: read the numbers out of the CI log ───────────────────
//
// Removed once the assertions above carry them.
#[test]

fn print_the_three_scenarios() {
    let mut report = String::new();

    for (label, key, moles, kj) in [
        ("chalk 0.1 mol + 40 kJ", "CaCO3", 0.1, 40.0),
        ("chalk 0.1 mol + 5 kJ", "CaCO3", 0.1, 5.0),
        ("water 5.5508 mol + 50 kJ", "water", 5.5508, 50.0),
    ] {
        let mut bench = Bench::new();
        let mut s = stack();
        let v = VesselId(0);
        add(&mut bench, &mut s, v, key, moles);
        let events = heat(&mut bench, &mut s, v, kj);
        report.push_str(&format!("\n== {label}\n"));
        report.push_str(&transcript(&bench, v, &events));
    }
    panic!("{report}");
}
