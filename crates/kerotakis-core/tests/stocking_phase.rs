//! Stocking phase (2026-11-09): a reagent leaves its bottle in the phase
//! its own sourced melting point gives at the temperature it arrives at,
//! and a flame that is taken away does not leave a molten pool behind.
//!
//! Triggers (originals, evidence only): batch3-diverse th-002 (hexadecane
//! stocked liquid at 10 °C, mp 18 °C), th-003 (tert-butanol liquid at
//! 15 °C, mp 25.8 °C), an-001/an-002 (NaCl/KCl molten at 25 °C after a
//! flame test), th-004 (paraffin has no melting data and stays solid).

use kerotakis_core::phase_route::{self, PhaseRouteEquilibrator, StockingBasis};
use kerotakis_core::species::Phase;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/stocking-phase-coverage-20261109.json");

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn step(bench: &mut Bench, stack: &mut SolverStack, op: Operator) -> Vec<Event> {
    bench.step_with(op, stack, &PermissiveScreen).expect("step")
}

fn add_at(bench: &mut Bench, stack: &mut SolverStack, key: &str, n: f64, celsius: Option<f64>) {
    step(
        bench,
        stack,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(n),
            at: celsius.map(Kelvin::from_celsius),
        },
    );
}

fn stocked(key: &str, celsius: Option<f64>) -> (f64, f64) {
    let mut bench = Bench::new();
    let mut stack = stack();
    add_at(&mut bench, &mut stack, key, 0.02, celsius);
    (
        moles(&bench, key, Phase::Solid),
        moles(&bench, key, Phase::Liquid),
    )
}

#[test]
fn stocking_phase_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for row in doc["species"].as_array().unwrap() {
        let key = row["species"].as_str().unwrap();
        seen.insert(key.to_string());
        let data = species::lookup(&SpeciesId::new(key))
            .unwrap_or_else(|| panic!("{key} is in the table but not the registry"));
        assert_eq!(
            phase_route::stocking_basis(data).key(),
            row["basis"].as_str().unwrap(),
            "{key}: runtime stocking basis differs from the generated table"
        );
        for (probe, phase) in row["stocked"].as_object().unwrap() {
            let celsius: f64 = probe.trim_end_matches('C').parse().unwrap();
            let runtime = phase_route::stocked_phase(data, celsius + 273.15);
            let runtime = serde_json::to_value(runtime).unwrap();
            assert_eq!(
                &runtime, phase,
                "{key} at {probe}: runtime stocks {runtime}"
            );
        }
    }
    for data in species::REGISTRY {
        assert!(
            seen.contains(data.key),
            "{} is registered but missing from docs/stocking-phase-coverage-20261109.json",
            data.key
        );
    }
}

/// th-002: cetane in a 10 °C bottle is a solid; dodecane beside it is not.
#[test]
fn hexadecane_below_its_melting_point_is_stocked_solid() {
    let (solid, liquid) = stocked("hexadecane", Some(10.0));
    assert!(
        solid > 0.019 && liquid < 1e-12,
        "solid {solid} liquid {liquid}"
    );
    let (solid, liquid) = stocked("hexadecane", Some(25.0));
    assert!(
        solid < 1e-12 && liquid > 0.019,
        "at 25 °C: solid {solid} liquid {liquid}"
    );
    let (solid, liquid) = stocked("dodecane", Some(10.0));
    assert!(
        solid < 1e-12 && liquid > 0.019,
        "dodecane: solid {solid} liquid {liquid}"
    );
}

/// A cold-stocked solid is paid for when it melts: warming it through its
/// melting point announces the melt and spends the latent heat.
#[test]
fn cold_stocked_hexadecane_melts_when_warmed_past_its_melting_point() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add_at(&mut bench, &mut stack, "hexadecane", 0.02, Some(10.0));
    // 0.02 mol x 53.36 kJ/mol is 1067 J of fusion; 3 kJ covers that and
    // the sensible heat on either side.
    let events = step(
        &mut bench,
        &mut stack,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(3000.0),
            source: None,
        },
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged { from: Phase::Solid, to: Phase::Liquid, species, .. }
                if species.0 == "hexadecane"
        )),
        "no melt announced: {events:?}"
    );
    assert!(moles(&bench, "hexadecane", Phase::Solid) < 1e-9);
    let t = bench.vessel(VesselId(0)).unwrap().temperature.0;
    assert!(t > 291.35, "ended at {t} K, below the melting point");
}

/// th-003: tert-butanol at 15 °C is a solid, isopropanol is not.
#[test]
fn tert_butanol_below_its_melting_point_is_stocked_solid() {
    let (solid, liquid) = stocked("tert_butanol", Some(15.0));
    assert!(
        solid > 0.019 && liquid < 1e-12,
        "solid {solid} liquid {liquid}"
    );
    let (solid, liquid) = stocked("isopropanol", Some(15.0));
    assert!(
        solid < 1e-12 && liquid > 0.019,
        "isopropanol: solid {solid} liquid {liquid}"
    );
}

/// Negatives: nothing that was already right moves.
#[test]
fn ordinary_reagents_keep_the_phase_they_had() {
    let (solid, liquid) = stocked("water", None);
    assert!(
        solid < 1e-12 && liquid > 0.019,
        "water at 25 °C: {solid} / {liquid}"
    );
    let (solid, liquid) = stocked("ethanol", None);
    assert!(
        solid < 1e-12 && liquid > 0.019,
        "ethanol at 25 °C: {solid} / {liquid}"
    );
    let (solid, liquid) = stocked("NaCl", None);
    assert!(
        solid > 0.019 && liquid < 1e-12,
        "NaCl at 25 °C: {solid} / {liquid}"
    );
}

/// Ice below 0 °C is ice from the bottle, not supercooled water.
#[test]
fn water_below_zero_is_stocked_as_ice() {
    let data = species::lookup(&SpeciesId::new("water")).unwrap();
    assert_eq!(phase_route::stocked_phase(data, 268.15), Phase::Solid);
    assert_eq!(phase_route::stocked_phase(data, 298.15), Phase::Liquid);
    let (solid, liquid) = stocked("water", Some(-5.0));
    assert!(
        solid > 0.019 && liquid < 1e-12,
        "water at -5 °C: {solid} / {liquid}"
    );
}

/// th-004: paraffin has no sourced melting point. It is NOT given a guessed
/// one: it keeps its standard phase and the table lists it as a gap.
#[test]
fn a_reagent_without_a_melting_point_falls_back_to_its_standard_phase() {
    let data = species::lookup(&SpeciesId::new("paraffin")).unwrap();
    assert_eq!(
        phase_route::stocking_basis(data),
        StockingBasis::NoMeltingPoint
    );
    assert_eq!(phase_route::stocked_phase(data, 346.0), data.standard_phase);
    // And a melting point without a latent heat is not acted on either:
    // dodecane at -20 °C would otherwise be a solid nothing could melt.
    let dodecane = species::lookup(&SpeciesId::new("dodecane")).unwrap();
    assert_eq!(
        phase_route::stocking_basis(dodecane),
        StockingBasis::MeltingPointWithoutLatentHeat
    );
    assert_eq!(phase_route::stocked_phase(dodecane, 253.15), Phase::Liquid);
}

/// Gases and condensed gases are never rephased by stocking.
#[test]
fn gases_and_cryogens_are_not_restocked() {
    for key in ["liquid_nitrogen", "dry_ice"] {
        let data = species::lookup(&SpeciesId::new(key)).unwrap();
        assert_eq!(
            phase_route::stocking_basis(data),
            StockingBasis::NotCondensed,
            "{key}"
        );
    }
}

/// an-001/an-002: a flame test melts the edge of the salt, and when the
/// flame is taken away the melt freezes again. No molten pool at 25 °C.
#[test]
fn a_flame_test_leaves_no_molten_salt_behind() {
    for key in ["NaCl", "KCl"] {
        let mut bench = Bench::new();
        let mut stack = stack();
        let n = 0.0017;
        add_at(&mut bench, &mut stack, key, n, None);
        let before = bench.vessel(VesselId(0)).unwrap().temperature.0;
        let events = step(
            &mut bench,
            &mut stack,
            Operator::Ignite {
                vessel: VesselId(0),
            },
        );
        assert!(
            moles(&bench, key, Phase::Liquid) < 1e-12,
            "{key}: {} mol molten after the flame was withdrawn: {events:?}",
            moles(&bench, key, Phase::Liquid)
        );
        assert!(
            (moles(&bench, key, Phase::Solid) - n).abs() < 1e-12,
            "{key}: inventory"
        );
        let after = bench.vessel(VesselId(0)).unwrap().temperature.0;
        assert!((after - before).abs() < 1e-9, "{key}: {before} -> {after}");
        let melted: f64 = events
            .iter()
            .filter_map(|e| match e {
                Event::StateChanged {
                    from: Phase::Solid,
                    to: Phase::Liquid,
                    moles: Some(m),
                    ..
                } => Some(m.0),
                _ => None,
            })
            .sum();
        let refrozen: f64 = events
            .iter()
            .filter_map(|e| match e {
                Event::StateChanged {
                    from: Phase::Liquid,
                    to: Phase::Solid,
                    moles: Some(m),
                    ..
                } => Some(m.0),
                _ => None,
            })
            .sum();
        assert!(
            (melted - refrozen).abs() < 1e-12,
            "{key}: melted {melted} but refroze {refrozen}"
        );
    }
}

/// The ordinary cooling route still refreezes a melt with its latent heat
/// paid: molten lead cooled below 327 °C solidifies.
#[test]
fn a_cooled_melt_refreezes_through_the_fusion_ledger() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add_at(&mut bench, &mut stack, "Pb", 0.05, Some(400.0));
    assert!(
        moles(&bench, "Pb", Phase::Liquid) > 0.049,
        "lead at 400 °C is stocked molten"
    );
    let events = step(
        &mut bench,
        &mut stack,
        Operator::Cool {
            vessel: VesselId(0),
            energy: Joules(500.0),
        },
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged { from: Phase::Liquid, to: Phase::Solid, species, .. }
                if species.0 == "Pb"
        )),
        "no freeze announced: {events:?}"
    );
    assert!(moles(&bench, "Pb", Phase::Liquid) < 1e-9);
}
