//! Heat capacity follows the phase, and a sugar raises the boiling point.
//!
//! Two defects found in the same ledger, both of which the bench already
//! held every constant needed to avoid.
//!
//! Cooling 100 mL of water with 60 kJ reported −39.2 °C. The plateau at
//! 0 °C was right — 10.4 kJ to reach it, then 33.3 kJ spent freezing 5.53
//! mol at 6.01 kJ/mol — and the remaining 16.3 kJ was then spent chilling
//! ICE at liquid water's 75.3 J/(mol·K). Ice's own 37.7 puts the same
//! beaker at −78 °C. The plateau being right is why it survived: the
//! observation the heating curve is drawn for was never the wrong one.
//!
//! And 20 g of sugar in 100 mL boiled at exactly 100.00 °C, which is the
//! one temperature a sugar solution does not boil at. The colligative
//! machinery was all there; the particle count was not, because sucrose is
//! a non-electrolyte and no aqueous engine lists it, and that silence was
//! read as "nothing dissolved".

use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("add");
}

fn transcript(bench: &Bench, events: &[Event]) -> String {
    use kerotakis_core::render::{render_event, Register};
    let vessel = bench.vessel(VesselId(0)).expect("vessel");
    let mut out = String::new();
    for event in events {
        out.push_str("    ");
        out.push_str(&render_event(event, Register::LV3));
        out.push('\n');
    }
    out.push_str(&format!(
        "    -- final: {:.2} K ({:.2} °C), Cp {:.3} J/K\n",
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

/// 100 mL of water is 5.5508 mol.
const HUNDRED_ML: f64 = 5.5508;

#[test]
fn the_leftover_energy_chills_ice_at_ices_own_heat_capacity() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", HUNDRED_ML);
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(60_000.0),
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("cool");
    let seen = transcript(&bench, &events);
    let vessel = bench.vessel(VesselId(0)).expect("vessel");

    // 10.4 kJ to 0 °C, 33.4 kJ to freeze it, 16.2 kJ into 209 J/K of ice.
    let celsius = vessel.temperature.to_celsius();
    assert!(
        (celsius + 78.0).abs() < 3.0,
        "60 kJ out of 100 mL of water leaves ice at about −78 °C, not \
         {celsius:.1} °C\n{seen}"
    );

    // The plateau it passed through is still announced, at 0 °C.
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::StateChanged { from: Phase::Liquid, to: Phase::Solid, at, .. }
                if (at.to_celsius()).abs() < 0.05
        )),
        "it still froze at 0.0 °C on the way down\n{seen}"
    );
    assert!(
        vessel
            .contents
            .iter()
            .all(|portion| portion.phase != Phase::Liquid),
        "all of it froze\n{seen}"
    );
}

#[test]
fn ice_and_water_and_steam_each_carry_their_own_heat_capacity() {
    use kerotakis_core::states::heat_capacity_in;
    let water = SpeciesId::new("water");
    assert!((heat_capacity_in(&water, Phase::Solid, 75.3) - 37.7).abs() < 1e-9);
    assert!((heat_capacity_in(&water, Phase::Liquid, 75.3) - 75.3).abs() < 1e-9);
    assert!((heat_capacity_in(&water, Phase::Gas, 75.3) - 33.6).abs() < 1e-9);
    // Everything else keeps the registry figure: this bench models no
    // transition but water's, so a per-phase number for anything else
    // would be data with nothing to check it.
    let chalk = SpeciesId::new("CaCO3");
    assert!((heat_capacity_in(&chalk, Phase::Gas, 81.9) - 81.9).abs() < 1e-9);
}

#[test]
fn sugar_water_boils_at_the_temperature_it_actually_boils_at() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", HUNDRED_ML);
    // 20 g of sucrose is 0.0584 mol in 0.100 kg of water: 0.584 mol/kg,
    // and water's ebullioscopic constant of 0.513 K·kg/mol makes that
    // +0.30 K. Neither number is curated — both fall out of R, the
    // boiling point and the enthalpy of vaporisation.
    add(&mut bench, &mut s, "sucrose", 20.0 / 342.2965);
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(60_000.0),
                source: None,
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("heat");
    let seen = transcript(&bench, &events);
    let vessel = bench.vessel(VesselId(0)).expect("vessel");

    let celsius = vessel.temperature.to_celsius();
    assert!(
        (celsius - 100.30).abs() < 0.1,
        "sugar water holds at its own boiling point, about 100.30 °C, not \
         {celsius:.2} °C\n{seen}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::StateChanged { to: Phase::Gas, shifted_by, .. } if *shifted_by > 0.25
        )),
        "and the event says how far the sugar moved it\n{seen}"
    );

    // 60 kJ − 33.6 kJ of warming, at 40.65 kJ/mol.
    let steam: f64 = events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == "water" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(
        (steam - 0.65).abs() < 0.05,
        "about 0.65 mol of steam expected, got {steam:.4}\n{seen}"
    );
}

#[test]
fn plain_water_is_unmoved_by_the_same_change() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", HUNDRED_ML);
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(60_000.0),
                source: None,
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("heat");
    let seen = transcript(&bench, &events);
    let celsius = bench
        .vessel(VesselId(0))
        .expect("vessel")
        .temperature
        .to_celsius();
    assert!(
        (celsius - 100.0).abs() < 0.05,
        "pure water still boils at 100.00 °C: {celsius:.2}\n{seen}"
    );
}
