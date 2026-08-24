//! EXP-2: thermal decomposition of NaHCO3.

use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
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
        .expect("step")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

fn moles_in_phase(bench: &Bench, key: &str, phase: species::Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn total_element(bench: &Bench, element: &str) -> f64 {
    let vessel = bench.vessel(VesselId(0)).unwrap();
    let mut total = 0.0;
    for p in &vessel.contents {
        if let Some(data) = species::lookup(&p.species) {
            if let Ok(f) = stoich::parse_formula(data.formula) {
                total += p.moles.0 * f.counts.get(element).copied().unwrap_or(0.0);
            }
        }
    }
    total
}

// ── Stoichiometric balance ───────────────────────────────────────────

#[test]
fn equation_balances() {
    let eq = stoich::parse_equation("2 NaHCO3 -> Na2CO3 + H2O + CO2").expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

// ── Temperature gating ──────────────────────────────────────────────

#[test]
fn does_not_fire_at_room_temperature() {
    let mut bench = Bench::new();
    let mut s = stack();
    let events = add(&mut bench, &mut s, "NaHCO3", 1.0);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { .. })),
        "decomposition must NOT fire at room temperature: {events:?}"
    );
}

#[test]
fn fires_when_heated() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    let events = heat(&mut bench, &mut s, 10_000.0);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("NaHCO"))),
        "decomposition must fire when heated above threshold: {events:?}"
    );
}

#[test]
fn does_not_fire_below_threshold() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    // Heat just a little — not enough to reach 353 K.
    // Cp of NaHCO3 ≈ 87.6 J/(mol·K); need ΔT > 54.85 K → Q > 4805 J.
    // Add 2000 J → ΔT ≈ 22.8 K → T ≈ 321 K (48 °C), still below 353 K.
    let events = heat(&mut bench, &mut s, 2000.0);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { .. })),
        "decomposition must NOT fire below threshold: {events:?}"
    );
}

// ── Products ────────────────────────────────────────────────────────

#[test]
fn co2_evolves_as_gas() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    let events = heat(&mut bench, &mut s, 10_000.0);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. }
                if species.0 == "CO2")),
        "CO2 must evolve as gas: {events:?}"
    );
}

#[test]
fn co2_contained_in_sealed_headspace() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(1.0),
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("seal");
    let events = heat(&mut bench, &mut s, 10_000.0);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasContained { species, .. }
                if species.0 == "CO2")),
        "CO2 must be contained in sealed headspace: {events:?}"
    );
}

#[test]
fn na2co3_produced_as_solid() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    heat(&mut bench, &mut s, 10_000.0);
    let na2co3 = moles_in_phase(&bench, "Na2CO3", species::Phase::Solid);
    assert!(na2co3 > 0.0, "Na2CO3 solid expected, got {na2co3}");
}

#[test]
fn stoichiometry_correct() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    heat(&mut bench, &mut s, 10_000.0);
    let na2co3 = moles_in_phase(&bench, "Na2CO3", species::Phase::Solid);
    // 2 NaHCO3 → 1 Na2CO3: 1.0 mol NaHCO3 → 0.5 mol Na2CO3
    assert!(
        (na2co3 - 0.5).abs() < 1e-6,
        "expected 0.5 mol Na2CO3, got {na2co3}"
    );
}

// ── Conservation ────────────────────────────────────────────────────

#[test]
fn conserves_elements() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    // Seal the vessel so CO2 stays in the headspace and all
    // elements remain countable inside the vessel.
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(1.0),
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("seal");
    let na_before = total_element(&bench, "Na");
    let c_before = total_element(&bench, "C");
    let o_before = total_element(&bench, "O");
    let h_before = total_element(&bench, "H");
    heat(&mut bench, &mut s, 10_000.0);
    let na_after = total_element(&bench, "Na");
    let c_after = total_element(&bench, "C");
    let o_after = total_element(&bench, "O");
    let h_after = total_element(&bench, "H");
    assert!(
        (na_after - na_before).abs() < 1e-9,
        "Na: {na_before} → {na_after}"
    );
    assert!(
        (c_after - c_before).abs() < 1e-9,
        "C: {c_before} → {c_after}"
    );
    assert!(
        (o_after - o_before).abs() < 1e-9,
        "O: {o_before} → {o_after}"
    );
    assert!(
        (h_after - h_before).abs() < 1e-9,
        "H: {h_before} → {h_after}"
    );
}

// ── Honesty ─────────────────────────────────────────────────────────

#[test]
fn no_apology_for_products() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaHCO3", 1.0);
    let events = heat(&mut bench, &mut s, 10_000.0);
    let apologies: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::NotYetModeled { .. }))
        .collect();
    assert!(apologies.is_empty(), "no apology expected: {apologies:?}");
}

// ── Rendering ───────────────────────────────────────────────────────

#[test]
fn three_registers_render() {
    let reaction = Event::ReactionOccurred {
        vessel: VesselId(0),
        equation: "2 NaHCO₃ →Δ Na₂CO₃ + H₂O + CO₂↑".into(),
    };
    for level in [1, 2, 3] {
        let text = render::render_event(&reaction, render::Register(level));
        assert!(!text.is_empty(), "level {level} must produce output");
    }
}
