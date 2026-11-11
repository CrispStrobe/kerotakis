//! th-004 (2026-11-11, round 4): a solid compared with its OWN melt uses a
//! sourced solid/liquid density pair (`phase_density`), so solid paraffin
//! sinks in molten paraffin; and a range material's freezing/melting event
//! is reported inside its declared range, not at wherever the vessel ended.

use kerotakis_core::script::parse_op;
use kerotakis_core::species::Phase;
use kerotakis_core::*;
use kerotakis_core::{buoyancy, melting_range, phase_density};

fn vessel_with(contents: &[(&str, f64, Phase)], t_k: f64) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v.temperature = Kelvin(t_k);
    v
}

fn run(commands: &[&str]) -> Vec<Vec<Event>> {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    commands
        .iter()
        .map(|command| {
            let op = parse_op(command)
                .unwrap_or_else(|e| panic!("parse {command}: {e}"))
                .expect("operator");
            bench
                .step_with(op, &mut stack, &PermissiveScreen)
                .unwrap_or_else(|e| panic!("{command}: {e}"))
        })
        .collect()
}

fn has_clause(v: &vessel::Vessel, key: &str) -> bool {
    appearance::observe(v).clauses.iter().any(|c| c.key == key)
}

#[test]
fn paraffin_pair_is_sourced_and_denser_as_a_solid() {
    let row = phase_density::pair("paraffin").expect("paraffin pair");
    assert!(row.source.contains("doi:10.3390/app6040112"));
    assert_eq!(row.solid_g_per_ml, 0.88);
    assert_eq!(row.liquid_g_per_ml, 0.77);
    assert!(row.solid_g_per_ml > row.liquid_g_per_ml);
}

#[test]
fn molten_paraffin_has_its_sourced_liquid_density() {
    let row = melting_range::range("paraffin").unwrap();
    let v = vessel_with(&[("paraffin", 0.0141767, Phase::Liquid)], row.upper_k + 5.0);
    let d = buoyancy::liquid_density_g_per_ml(&v).expect("a liquid");
    assert!((d - 0.77).abs() < 1e-9, "{d}");
}

#[test]
fn solid_paraffin_sinks_in_its_own_melt() {
    // th-004's split after 500 J: 0.0106913 mol solid, 0.0034854 liquid.
    let v = vessel_with(
        &[
            ("paraffin", 0.0106913, Phase::Solid),
            ("paraffin", 0.0034854, Phase::Liquid),
        ],
        321.54,
    );
    let d = buoyancy::liquid_density_g_per_ml(&v).expect("a liquid");
    assert_eq!(
        buoyancy::floats_in(&SpeciesId::new("paraffin"), d),
        Some(false)
    );
    assert!(!has_clause(&v, "look.floats-on-top"));
}

#[test]
fn solid_paraffin_still_floats_on_water() {
    // Negative control: the pair moves paraffin's melt, not water.
    let v = vessel_with(
        &[
            ("water", 5.5, Phase::Liquid),
            ("paraffin", 0.01, Phase::Solid),
        ],
        293.15,
    );
    let d = buoyancy::liquid_density_g_per_ml(&v).expect("a liquid");
    assert_eq!(
        buoyancy::floats_in(&SpeciesId::new("paraffin"), d),
        Some(true)
    );
    assert!(has_clause(&v, "look.floats-on-top"));
}

#[test]
fn a_species_without_a_pair_keeps_its_registry_density() {
    for key in ["PP", "hexadecane", "water"] {
        assert!(phase_density::pair(key).is_none(), "{key}");
        let registry = species::lookup(&SpeciesId::new(key)).unwrap().density;
        for phase in [Phase::Solid, Phase::Liquid] {
            assert_eq!(
                phase_density::density_in_phase(&SpeciesId::new(key), phase),
                Some(registry),
                "{key} {phase:?}"
            );
        }
    }
    assert_eq!(
        phase_density::density_in_phase(&SpeciesId::new("not-a-species"), Phase::Solid),
        None
    );
}

fn transitions(events: &[Event]) -> Vec<(Phase, f64)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::StateChanged {
                species, to, at, ..
            } if species.0 == "paraffin" => Some((*to, at.0)),
            _ => None,
        })
        .collect()
}

#[test]
fn th004_freezing_on_cooling_is_reported_inside_the_declared_range() {
    let row = melting_range::range("paraffin").unwrap();
    let low = row.lower_k.unwrap();
    let steps = run(&["add v1 paraffin 5g @ 20C", "heat v1 0.5kJ", "cool v1 0.5kJ"]);
    let melted = transitions(&steps[1]);
    assert!(
        melted.iter().any(|(to, _)| *to == Phase::Liquid),
        "{:?}",
        steps[1]
    );
    for (_, at) in &melted {
        assert!(*at >= low && *at <= row.upper_k, "melting at {at} K");
    }
    let frozen = transitions(&steps[2]);
    assert!(
        frozen.iter().any(|(to, _)| *to == Phase::Solid),
        "{:?}",
        steps[2]
    );
    for (_, at) in &frozen {
        assert!(*at >= low && *at <= row.upper_k, "freezing at {at} K");
        // Cooled to 20 °C: the last of it froze at the bottom of the range.
        assert!((*at - low).abs() < 1e-9, "freezing at {at} K");
    }
}

#[test]
fn melting_inside_the_range_is_reported_at_the_vessel_temperature() {
    // Negative control for the clamp: a melt that ends inside the range
    // reports the temperature it ended at.
    let row = melting_range::range("paraffin").unwrap();
    let n = 5.0
        / species::lookup(&SpeciesId::new("paraffin"))
            .unwrap()
            .molar_mass;
    let mut v = vessel_with(&[("paraffin", n, Phase::Solid)], row.upper_k - 5.0);
    let events = PhaseRouteEquilibrator.equilibrate(&mut v).expect("settle");
    let melted = transitions(&events);
    assert!(!melted.is_empty(), "{events:?}");
    for (_, at) in melted {
        assert!(
            (at - v.temperature.0).abs() < 1e-9,
            "{at} vs {}",
            v.temperature.0
        );
    }
}
