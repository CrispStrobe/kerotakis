//! EXP-13: ascorbic acid + iodine decolorisation assay.

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

fn total_moles(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
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
    let eq = stoich::parse_equation("C6H8O6 + I2 -> C6H6O6 + 2 HI").expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

// ── Reaction fires ──────────────────────────────────────────────────

#[test]
fn fires_when_both_present() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "ascorbic_acid", 0.01);
    let events = add(&mut bench, &mut s, "I2", 0.005);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("I₂"))),
        "iodine decolorisation must fire: {events:?}"
    );
}

#[test]
fn does_not_fire_without_ascorbic_acid() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    let events = add(&mut bench, &mut s, "I2", 0.005);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("I₂"))),
        "should NOT fire without vitamin C: {events:?}"
    );
}

// ── Titration-style endpoint ─────────────────────────────────────────

#[test]
fn iodine_consumed_while_vitamin_c_present() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "ascorbic_acid", 0.01);
    // Add I2 in small aliquots — each should be consumed.
    add(&mut bench, &mut s, "I2", 0.003);
    let i2_remaining = total_moles(&bench, "I2");
    assert!(
        i2_remaining < 1e-9,
        "I2 should be fully consumed while vitamin C remains: {i2_remaining}"
    );
    let vit_c = total_moles(&bench, "ascorbic_acid");
    assert!(vit_c > 0.006, "vitamin C should remain: {vit_c}");
}

#[test]
fn excess_iodine_persists_past_endpoint() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "ascorbic_acid", 0.01);
    // Add excess I2 (0.015 mol > 0.01 mol vitamin C).
    add(&mut bench, &mut s, "I2", 0.015);
    let i2_remaining = total_moles(&bench, "I2");
    assert!(
        i2_remaining > 0.004,
        "excess I2 should persist past endpoint: {i2_remaining}"
    );
    let vit_c = total_moles(&bench, "ascorbic_acid");
    assert!(vit_c < 1e-9, "all vitamin C should be consumed: {vit_c}");
}

// ── Products ────────────────────────────────────────────────────────

#[test]
fn produces_dehydroascorbic_acid() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "ascorbic_acid", 0.01);
    add(&mut bench, &mut s, "I2", 0.005);
    let dhaa = total_moles(&bench, "dehydroascorbic_acid");
    assert!(
        (dhaa - 0.005).abs() < 1e-6,
        "expected 0.005 mol dehydroascorbic acid: {dhaa}"
    );
}

#[test]
fn produces_hi() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "ascorbic_acid", 0.01);
    add(&mut bench, &mut s, "I2", 0.005);
    let hi = total_moles(&bench, "HI");
    assert!(
        (hi - 0.01).abs() < 1e-6,
        "expected 0.01 mol HI (2:1 ratio): {hi}"
    );
}

// ── Conservation ────────────────────────────────────────────────────

#[test]
fn conserves_elements() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "ascorbic_acid", 0.01);
    let c_before = total_element(&bench, "C");
    let h_before = total_element(&bench, "H");
    let o_before = total_element(&bench, "O");
    let i_before = total_element(&bench, "I");
    add(&mut bench, &mut s, "I2", 0.005);
    let c_after = total_element(&bench, "C");
    let h_after = total_element(&bench, "H");
    let o_after = total_element(&bench, "O");
    let i_after = total_element(&bench, "I");
    assert!(
        (c_after - c_before).abs() < 1e-9,
        "C: {c_before} → {c_after}"
    );
    assert!(
        (h_after - h_before).abs() < 1e-9,
        "H: {h_before} → {h_after}"
    );
    assert!(
        (o_after - o_before).abs() < 1e-9,
        "O: {o_before} → {o_after}"
    );
    // I2 adds 0.01 mol I (0.005 * 2).
    assert!(
        (i_after - (i_before + 0.01)).abs() < 1e-9,
        "I: {i_before} + 0.01 ≠ {i_after}"
    );
}

// ── Juice vs water contrast ─────────────────────────────────────────

#[test]
fn juice_vs_water_contrast() {
    // Juice vessel: water + ascorbic acid + I2 → I2 consumed.
    let mut bench_juice = Bench::new();
    let mut s1 = stack();
    add(&mut bench_juice, &mut s1, "water", 55.0);
    add(&mut bench_juice, &mut s1, "ascorbic_acid", 0.01);
    add(&mut bench_juice, &mut s1, "I2", 0.005);
    let i2_juice = total_moles(&bench_juice, "I2");

    // Water vessel: just water + I2 → I2 persists.
    let mut bench_water = Bench::new();
    let mut s2 = stack();
    add(&mut bench_water, &mut s2, "water", 55.0);
    add(&mut bench_water, &mut s2, "I2", 0.005);
    let i2_water = bench_water
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new("I2"))
        .0;

    assert!(
        i2_juice < 1e-9,
        "juice vessel: I2 should be consumed: {i2_juice}"
    );
    assert!(
        i2_water > 0.004,
        "water vessel: I2 should persist: {i2_water}"
    );
}

// ── Rendering ───────────────────────────────────────────────────────

#[test]
fn three_registers_render() {
    let reaction = Event::ReactionOccurred {
        vessel: VesselId(0),
        equation: "C₆H₈O₆ + I₂ → C₆H₆O₆ + 2 HI".into(),
    };
    for level in [1, 2, 3] {
        let text = render::render_event(&reaction, render::Register(level));
        assert!(!text.is_empty(), "level {level} must produce output");
    }
}
