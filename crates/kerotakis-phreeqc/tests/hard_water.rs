//! Hard-water chemistry: chalk, limescale, gypsum's waters of
//! crystallisation, and the calcium chloride hot pack.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn add(
    bench: &mut Bench,
    eq: &mut PhreeqcEquilibrator,
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
            eq,
            &PermissiveScreen,
        )
        .expect("step")
}

#[test]
fn chalk_barely_dissolves_in_water() {
    // Calcite solubility is ~1e-4 mol/kgw against atmospheric CO2 — the
    // solid stays, and the trace that dissolves makes the water mildly
    // basic. Why statues survive rain (and why acid rain matters).
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "CaCO3", 0.01);

    let vessel = bench.vessel(v).unwrap();
    let solid = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "CaCO3" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum::<f64>();
    assert!(solid > 0.0095, "chalk mostly stays solid, {solid} mol left");
    let ph = vessel.solution.clone().expect("characterised").ph;
    assert!(
        ph > 7.5 && ph < 10.5,
        "calcite water is mildly basic, got {ph}"
    );
}

#[test]
fn named_epsom_salt_dissolves_and_releases_seven_crystal_waters() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.55);
    let water_before = bench
        .vessel(v)
        .unwrap()
        .moles_of(&SpeciesId::new("water"))
        .0;
    let epsom = script::parse_op("add v1 Bittersalz 2.46471g")
        .expect("localized Epsom-salt command")
        .expect("operator");
    let events = bench
        .step_with(epsom, &mut eq, &PermissiveScreen)
        .expect("dissolve Epsom salt");
    let vessel = bench.vessel(v).unwrap();

    assert!(events.iter().any(|event| matches!(
        event,
        Event::MaterialAdded { recipe_id, .. }
            if recipe_id == "household/epsom-salt-heptahydrate"
    )));
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::Dissolved { species, moles, .. }
                if species.0 == "epsomite" && moles.0 > 0.0099
        )),
        "the USGS Epsomite phase should dissolve: {events:?}"
    );
    assert!(vessel.moles_of(&SpeciesId::new("epsomite")).0 < 1e-8);
    assert!(vessel.moles_of(&SpeciesId::new("Mg+2")).0 > 0.0099);

    let released_water = vessel.moles_of(&SpeciesId::new("water")).0 - water_before;
    assert!(
        (released_water - 0.07).abs() < 2e-4,
        "0.01 mol MgSO4·7H2O should release 0.07 mol crystal water, got {released_water}"
    );
}

#[test]
fn chalk_dissolves_and_fizzes_in_acid() {
    // The statue in acid rain, sped up: acid consumes the carbonate, CO2
    // escapes, the chalk is gone.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "CaCO3", 0.01);
    let events = add(&mut bench, &mut eq, v, "HCl", 0.03);

    let co2: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(co2 > 0.008, "the carbonate leaves as CO2, got {co2} mol");
    let solid = bench
        .vessel(v)
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == "CaCO3" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum::<f64>();
    assert!(
        solid < 1e-6,
        "the chalk dissolves completely, {solid} mol left"
    );
}

#[test]
fn hard_water_deposits_limescale() {
    // Calcium ions meeting carbonate: the kettle's fur, computed.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    add(&mut bench, &mut eq, v, "NaHCO3", 0.02);
    let events = add(&mut bench, &mut eq, v, "CaCl2", 0.01);

    let scale = events
        .iter()
        .find_map(|e| match e {
            Event::Precipitated { species, moles, .. } if species.0 == "CaCO3" => Some(moles.0),
            _ => None,
        })
        .expect("calcite must precipitate from hard water");
    assert!(
        scale > 0.005,
        "most of the calcium scales out, got {scale} mol"
    );
}

#[test]
fn gypsum_precipitation_binds_water_into_the_crystal() {
    // CaSO4·2H2O: the solid takes its two waters out of the liquid — the
    // ledger notices.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 55.51);
    let water_before = bench
        .vessel(v)
        .unwrap()
        .moles_of(&SpeciesId::new("water"))
        .0;
    add(&mut bench, &mut eq, v, "CaCl2", 0.05);
    let events = add(&mut bench, &mut eq, v, "MgSO4", 0.05);

    let gypsum = events
        .iter()
        .find_map(|e| match e {
            Event::Precipitated { species, moles, .. } if species.0 == "gypsum" => Some(moles.0),
            _ => None,
        })
        .expect("gypsum must precipitate above its solubility (~0.015 m)");
    assert!(gypsum > 0.02, "expected ~0.035 mol gypsum, got {gypsum}");

    let water_after = bench
        .vessel(v)
        .unwrap()
        .moles_of(&SpeciesId::new("water"))
        .0;
    let bound = water_before - water_after;
    // 1e-5, not 1e-6: the water is no longer adjusted by hand — PHREEQC
    // moves it into the crystal and the vessel is rebuilt from its
    // `mass_H2O`, so this now measures the engine's own bookkeeping rather
    // than our arithmetic on top of it. What is left is the float residue
    // of that round trip, about 2 µmol. The claim being tested — two waters
    // per formula unit — is unchanged.
    assert!(
        (bound - 2.0 * gypsum).abs() < 1e-5,
        "the crystal binds 2 waters per formula: {gypsum} mol gypsum should bind {} mol, ledger shows {bound}",
        2.0 * gypsum
    );
}

#[test]
fn calcium_chloride_is_a_hot_pack() {
    // ΔH_dis = −82.8 kJ/mol: 0.1 mol into 100 mL → +19.8 K. The classic
    // road-salt / hot-pack exotherm.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.55);
    add(&mut bench, &mut eq, v, "CaCl2", 0.1);
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        (t - 44.8).abs() < 3.0,
        "expected ~45 °C after the exotherm, got {t:.1} °C"
    );
}

#[test]
fn potassium_chloride_cools() {
    // The endothermic mirror: KCl at +17.2 kJ/mol.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.55);
    add(&mut bench, &mut eq, v, "KCl", 0.1);
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(t < 22.0 && t > 18.0, "KCl cools ~4 K, got {t:.1} °C");
}

// ── MIX tests (CAP-10) ────────────────────────────────────────────────

fn step(bench: &mut Bench, eq: &mut PhreeqcEquilibrator, op: Operator) -> Vec<Event> {
    bench.step_with(op, eq, &PermissiveScreen).expect("step")
}

fn element_total(bench: &Bench, vessels: &[VesselId], species_key: &str) -> f64 {
    vessels
        .iter()
        .map(|v| {
            bench
                .vessel(*v)
                .unwrap()
                .moles_of(&SpeciesId::new(species_key))
                .0
        })
        .sum()
}

fn prepare_hard_and_soft(eq: &mut PhreeqcEquilibrator) -> Bench {
    let mut bench = Bench::new();
    let v1 = VesselId(0);
    add(&mut bench, eq, v1, "water", 55.51);
    add(&mut bench, eq, v1, "CaCl2", 5e-3);
    add(&mut bench, eq, v1, "MgSO4", 3e-3);

    step(&mut bench, eq, Operator::NewVessel { kind: None });
    let v2 = VesselId(1);
    add(&mut bench, eq, v2, "water", 55.51);
    add(&mut bench, eq, v2, "NaCl", 0.01);

    step(&mut bench, eq, Operator::NewVessel { kind: None });
    bench
}

#[test]
fn mix_emits_mixed_event() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = prepare_hard_and_soft(&mut eq);
    let events = step(
        &mut bench,
        &mut eq,
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.5,
            fraction_b: 0.5,
        },
    );
    assert!(
        events.iter().any(|e| matches!(e, Event::Mixed { .. })),
        "Operator::Mix must produce Event::Mixed, got {events:#?}"
    );
}

#[test]
fn mix_conserves_elements() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = prepare_hard_and_soft(&mut eq);
    let all = [VesselId(0), VesselId(1), VesselId(2)];

    let before: Vec<(String, f64)> = ["water", "Ca+2", "Mg+2", "Na+", "Cl-"]
        .iter()
        .map(|s| (s.to_string(), element_total(&bench, &all, s)))
        .collect();

    step(
        &mut bench,
        &mut eq,
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 1.0,
            fraction_b: 1.0,
        },
    );

    for (species, total_before) in &before {
        let total_after = element_total(&bench, &all, species);
        let delta = (total_after - total_before).abs();
        assert!(
            delta < 1e-6,
            "{species} not conserved: before={total_before:.12e}, after={total_after:.12e}, delta={delta:.12e}"
        );
    }
}

#[test]
fn mix_produces_speciated_result() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = prepare_hard_and_soft(&mut eq);

    let events = step(
        &mut bench,
        &mut eq,
        Operator::Mix {
            a: VesselId(0),
            b: VesselId(1),
            into: VesselId(2),
            fraction_a: 0.5,
            fraction_b: 0.5,
        },
    );

    let failed: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::SolverFailed { .. }))
        .collect();
    assert!(
        failed.is_empty(),
        "mix should not produce solver failures: {failed:#?}"
    );

    let mixed = bench.vessel(VesselId(2)).unwrap();
    let soln = mixed
        .solution
        .as_ref()
        .expect("mixed vessel must be speciated");
    assert!(
        soln.ph > 4.0 && soln.ph < 10.0,
        "mixed solution pH should be reasonable, got {}",
        soln.ph
    );
    assert!(
        soln.ionic_strength > 0.0,
        "mixed solution must have positive ionic strength"
    );
    let ca = mixed.moles_of(&SpeciesId::new("Ca+2")).0;
    assert!(
        ca > 0.0,
        "calcium from hard water must be present in the mix"
    );
    let na = mixed.moles_of(&SpeciesId::new("Na+")).0;
    assert!(
        na > 0.0,
        "sodium from soft water must be present in the mix"
    );
}
