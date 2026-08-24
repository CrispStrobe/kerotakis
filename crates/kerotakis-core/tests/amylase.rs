//! EXP-14: amylase-catalysed starch hydrolysis.

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
    let eq = stoich::parse_equation("2 C6H10O5 + H2O -> C12H22O11").expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

// ── Enzyme gate ─────────────────────────────────────────────────────

#[test]
fn fires_with_amylase() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    let events = add(&mut bench, &mut s, "amylase", 1e-6);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("amylase"))),
        "hydrolysis must fire with amylase: {events:?}"
    );
}

#[test]
fn does_not_fire_without_amylase() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    let events = add(&mut bench, &mut s, "starch", 0.1);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("amylase") || equation.contains("maltose"))),
        "hydrolysis must NOT fire without amylase: {events:?}"
    );
}

#[test]
fn amylase_not_consumed() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    add(&mut bench, &mut s, "amylase", 1e-6);
    let amylase = total_moles(&bench, "amylase");
    assert!(
        (amylase - 1e-6).abs() < 1e-12,
        "amylase must not be consumed: {amylase}"
    );
}

// ── Products ────────────────────────────────────────────────────────

#[test]
fn produces_maltose() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    add(&mut bench, &mut s, "amylase", 1e-6);
    let maltose = total_moles(&bench, "maltose");
    assert!(
        (maltose - 0.05).abs() < 1e-6,
        "expected 0.05 mol maltose (2:1 starch:maltose): {maltose}"
    );
}

#[test]
fn starch_consumed() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    add(&mut bench, &mut s, "amylase", 1e-6);
    let starch = total_moles(&bench, "starch");
    assert!(starch < 1e-9, "starch should be fully consumed: {starch}");
}

// ── Conservation ────────────────────────────────────────────────────

#[test]
fn conserves_elements() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    let c_before = total_element(&bench, "C");
    let h_before = total_element(&bench, "H");
    let o_before = total_element(&bench, "O");
    // 1e-9 mol: catalytic trace, well above TRACE (1e-12) but negligible element contribution
    add(&mut bench, &mut s, "amylase", 1e-9);
    let c_after = total_element(&bench, "C");
    let h_after = total_element(&bench, "H");
    let o_after = total_element(&bench, "O");
    assert!(
        (c_after - c_before).abs() < 1e-6,
        "C: {c_before} → {c_after}"
    );
    assert!(
        (h_after - h_before).abs() < 1e-6,
        "H: {h_before} → {h_after}"
    );
    assert!(
        (o_after - o_before).abs() < 1e-6,
        "O: {o_before} → {o_after}"
    );
}

// ── Starch-positive vs starch-negative (Lugol) ──────────────────────

#[test]
fn starch_positive_without_enzyme() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    let starch = total_moles(&bench, "starch");
    assert!(starch > 0.09, "starch must remain without enzyme: {starch}");
}

#[test]
fn starch_negative_after_enzyme() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    add(&mut bench, &mut s, "amylase", 1e-6);
    let starch = total_moles(&bench, "starch");
    assert!(
        starch < 1e-9,
        "starch must be consumed after enzyme: {starch}"
    );
}

// ── Rendering ───────────────────────────────────────────────────────

#[test]
fn three_registers_render() {
    let reaction = Event::ReactionOccurred {
        vessel: VesselId(0),
        equation: "2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁".into(),
    };
    for level in [1, 2, 3] {
        let text = render::render_event(&reaction, render::Register(level));
        assert!(!text.is_empty(), "level {level} must produce output");
    }
}
