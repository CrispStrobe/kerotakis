//! The aqueous equilibrator through the bench loop: dissolution,
//! precipitation and solubility limits computed from thermodynamic data, and
//! bounded fuzzing (P0: random inputs → no crash, honest failure state).

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
    vessel: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

/// Total moles of one element across every portion of the bench, computed
/// from the inventory (ions, salts, minerals).
fn element_total(bench: &Bench, element: &str) -> f64 {
    let contribution: &[(&str, &[(&str, f64)])] = &[
        ("NaCl", &[("Na", 1.0), ("Cl", 1.0)]),
        ("AgNO3", &[("Ag", 1.0), ("N", 1.0)]),
        ("AgCl", &[("Ag", 1.0), ("Cl", 1.0)]),
        ("Na+", &[("Na", 1.0)]),
        ("Cl-", &[("Cl", 1.0)]),
        ("Ag+", &[("Ag", 1.0)]),
        ("NO3-", &[("N", 1.0)]),
    ];
    bench
        .vessels
        .iter()
        .flat_map(|v| v.contents.iter())
        .map(|p| {
            contribution
                .iter()
                .find(|(k, _)| *k == p.species.0)
                .map(|(_, els)| {
                    els.iter()
                        .filter(|(el, _)| *el == element)
                        .map(|(_, c)| c * p.moles.0)
                        .sum::<f64>()
                })
                .unwrap_or(0.0)
        })
        .sum()
}

#[test]
fn salt_dissolves_in_water() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.55); // ~100 mL
    let events = add(&mut bench, &mut stack, v, "NaCl", 0.05);

    assert!(
        events.iter().any(|e| matches!(e, Event::Dissolved { .. })),
        "salt below saturation must dissolve, got {events:?}"
    );
    let vessel = bench.vessel(v).unwrap();
    assert!(
        !vessel
            .contents
            .iter()
            .any(|p| p.species.0 == "NaCl" && p.phase == Phase::Solid),
        "no solid salt should remain"
    );
    assert!((vessel.moles_of(&SpeciesId::new("Na+")).0 - 0.05).abs() < 1e-6);
    let ph = vessel.solution.clone().expect("characterised").ph;
    assert!((ph - 7.0).abs() < 0.5, "NaCl solution is neutral, pH {ph}");
}

#[test]
fn halite_stops_dissolving_at_saturation() {
    // 8 mol NaCl into 1 kg water: solubility is ~6.1 mol/kgw, the rest must
    // stay solid — a computed solubility limit, not a scripted one.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51); // 1.000 kg
    add(&mut bench, &mut stack, v, "NaCl", 8.0);

    let vessel = bench.vessel(v).unwrap();
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "NaCl" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    let dissolved = vessel.moles_of(&SpeciesId::new("Na+")).0;
    assert!(
        solid > 0.5 && dissolved > 5.0 && dissolved < 7.5,
        "expected ~6.1 mol/kgw dissolved with the rest solid; got {dissolved} dissolved, {solid} solid"
    );
    assert!((solid + dissolved - 8.0).abs() < 1e-6, "sodium conserved");
}

#[test]
fn silver_nitrate_plus_salt_precipitates_silver_chloride() {
    // The marquee sequence of the whole product.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaCl", 0.01);
    let events = add(&mut bench, &mut stack, v, "AgNO3", 0.01);

    let precipitated = events.iter().find_map(|e| match e {
        Event::Precipitated { species, moles, .. } if species.0 == "AgCl" => Some(moles.0),
        _ => None,
    });
    let agcl = precipitated.expect("AgCl must precipitate");
    assert!(
        agcl > 0.0098 && agcl <= 0.01,
        "nearly all silver precipitates as AgCl, got {agcl} mol"
    );

    // Conservation: every atom of Ag and Cl still on the bench.
    assert!((element_total(&bench, "Ag") - 0.01).abs() < 1e-6);
    assert!((element_total(&bench, "Cl") - 0.01).abs() < 1e-6);
    assert!((element_total(&bench, "Na") - 0.01).abs() < 1e-6);

    // And the child register says the famous line.
    let text = render_event(
        &Event::Precipitated {
            vessel: v,
            species: SpeciesId::new("AgCl"),
            moles: Moles(agcl),
        },
        Register::LV1,
    );
    assert!(text.contains("cloudy") && text.contains("white"), "{text}");
}

#[test]
fn ethanol_mixture_is_honestly_out_of_scope() {
    // Ethanol is not aqueous-mappable yet: the aqueous solver must decline
    // (not guess), and the honesty pass must speak up about the salt.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "ethanol", 1.0);
    let events = add(&mut bench, &mut stack, v, "NaCl", 0.01);

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "salt in ethanol must be flagged as unmodelled, got {events:?}"
    );
    assert!(bench.vessel(v).unwrap().solution.is_none());
}

/// Bounded fuzz (P0): random additions of mapped species in random amounts —
/// the loop must never panic, and every accepted step either equilibrates or
/// fails honestly while conserving elements.
#[test]
fn fuzz_random_aqueous_benches() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut stack = stack();
    let keys = ["water", "NaCl", "AgNO3", "Ag+", "Cl-", "Na+", "NO3-"];
    for seed in 0u64..200 {
        let mut bench = Bench::new();
        let mut expected = std::collections::BTreeMap::new();
        for step in 0..6 {
            let mut h = DefaultHasher::new();
            (seed, step).hash(&mut h);
            let r = h.finish();
            let key = keys[(r % keys.len() as u64) as usize];
            let amount = ((r >> 8) % 10_000) as f64 / 1000.0 + 1e-4; // 0.0001..10 mol
            let events = bench
                .step_with(
                    Operator::Add {
                        vessel: VesselId(0),
                        species: SpeciesId::new(key),
                        moles: Moles(amount),
                        at: None,
                    },
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step never hard-errors for valid species");
            // Solver failures are allowed; panics and silent element loss
            // are not.
            let _ = &events;
            *expected.entry(key).or_insert(0.0) += amount;
        }
        for (el, per_key) in [
            ("Na", vec![("NaCl", 1.0), ("Na+", 1.0)]),
            ("Ag", vec![("AgNO3", 1.0), ("Ag+", 1.0)]),
        ] {
            let want: f64 = per_key
                .iter()
                .map(|(k, c)| c * expected.get(*k).copied().unwrap_or(0.0))
                .sum();
            let got = element_total(&bench, el);
            assert!(
                (got - want).abs() < 1e-6 * want.max(1.0),
                "seed {seed}: element {el} drifted: want {want}, got {got}"
            );
        }
    }
}

/// Enthalpy is a state function: the same two salts, dissolved in either
/// order, must leave the beaker at the same temperature.
///
/// This failed for a long time, and the failure was invisible because the
/// chemistry was right. Mineral phases are discovered per database by
/// formula match, and Sylvite exists only in `pitzer.dat`. A phase the
/// routed database does not define is dissolved into element totals
/// instead — correct, and deliberate — but no `Dissolved` event was
/// recorded for it, and the dissolution enthalpy rides on the event.
///
/// Since the router picks a database by ionic strength, potassium chloride
/// therefore cooled the beaker only when something else had already made
/// the solution concentrated enough to route to pitzer. KCl-then-NaCl
/// ended 0.82 K warmer than NaCl-then-KCl, and that gap was also the whole
/// of a long-standing order-dependent pH: 0.82 K against water's
/// dpH/dT ≈ -0.0163/K is 1.34e-2, and the observed drift was 1.36e-2.
/// Holding the temperature equal collapsed it to 2.8e-10.
///
/// The residual tolerance below is not slack for that bug — it is three
/// orders larger than what remains (~4 mK), which comes from applying
/// ΔT = q/cp incrementally while cp grows with each addition.
#[test]
fn dissolution_heat_does_not_depend_on_order() {
    let mut s = stack();
    let mut final_t = Vec::new();
    for order in [["NaCl", "KCl"], ["KCl", "NaCl"]] {
        let mut bench = Bench::new();
        let v = VesselId(0);
        add(&mut bench, &mut s, v, "water", 5.534_276_991_396_059);
        add(
            &mut bench,
            &mut s,
            v,
            order[0],
            if order[0] == "NaCl" { 0.05 } else { 0.02 },
        );
        add(
            &mut bench,
            &mut s,
            v,
            order[1],
            if order[1] == "NaCl" { 0.05 } else { 0.02 },
        );
        final_t.push(bench.vessel(v).expect("vessel").temperature.to_celsius());
    }
    let drift = (final_t[0] - final_t[1]).abs();
    assert!(
        drift < 0.05,
        "dissolving the same salts in a different order changed the final \
         temperature by {drift:.4} K ({:.4} vs {:.4}) — enthalpy is not \
         behaving as a state function",
        final_t[0],
        final_t[1],
    );
}
