//! CAP-23 rung 2: permanganate–ethanol curated redox.

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
        .contents
        .iter()
        .filter(|p| p.species.0 == key)
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
fn organic_equation_balances() {
    let eq = stoich::parse_equation("4 KMnO4 + 3 C2H5OH → 4 MnO2 + 3 CH3COOH + 4 KOH + H2O")
        .expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

#[test]
fn aqueous_equation_balances() {
    let eq = stoich::parse_equation("4 MnO4- + 3 C2H5OH → 4 MnO2 + 3 CH3COOH + 4 OH- + H2O")
        .expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

// ── Organic solvent case: KMnO4 solid + ethanol ─────────────────────

#[test]
fn permanganate_in_ethanol_fires_curated_reaction() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 1.0);
    let events = add(&mut bench, &mut s, "KMnO4", 0.04);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("KMnO4"))),
        "curated reaction must fire: {events:?}"
    );
}

#[test]
fn permanganate_in_ethanol_produces_mno2_solid() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 1.0);
    add(&mut bench, &mut s, "KMnO4", 0.04);
    let vessel = bench.vessel(VesselId(0)).unwrap();
    let mno2 = vessel
        .contents
        .iter()
        .find(|p| p.species.0 == "MnO2" && p.phase == Phase::Solid);
    assert!(mno2.is_some(), "MnO2 solid expected: {:?}", vessel.contents);
    assert!(mno2.unwrap().moles.0 > 0.0);
}

#[test]
fn permanganate_in_ethanol_conserves_mass() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 1.0);
    let mn_before = total_element(&bench, "Mn");
    let k_before = total_element(&bench, "K");
    let c_before = total_element(&bench, "C");
    let o_before = total_element(&bench, "O");
    let h_before = total_element(&bench, "H");
    add(&mut bench, &mut s, "KMnO4", 0.04);
    let mn_after = total_element(&bench, "Mn");
    let k_after = total_element(&bench, "K");
    let c_after = total_element(&bench, "C");
    let o_after = total_element(&bench, "O");
    let h_after = total_element(&bench, "H");
    // Adding 0.04 mol KMnO4 adds 0.04 Mn, 0.04 K, 0.16 O.
    assert!(
        (mn_after - (mn_before + 0.04)).abs() < 1e-9,
        "Mn: {mn_before} + 0.04 ≠ {mn_after}"
    );
    assert!(
        (k_after - (k_before + 0.04)).abs() < 1e-9,
        "K: {k_before} + 0.04 ≠ {k_after}"
    );
    assert!(
        (c_after - c_before).abs() < 1e-9,
        "C changed: {c_before} → {c_after}"
    );
    assert!(
        (o_after - (o_before + 0.16)).abs() < 1e-9,
        "O: {o_before} + 0.16 ≠ {o_after}"
    );
    assert!(
        (h_after - h_before).abs() < 1e-9,
        "H changed: {h_before} → {h_after}"
    );
}

#[test]
fn permanganate_in_ethanol_no_apology() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 1.0);
    let events = add(&mut bench, &mut s, "KMnO4", 0.04);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "permanganate in ethanol is modelled now: {events:?}"
    );
}

// ── Aqueous case: MnO4⁻ ion + ethanol ──────────────────────────────

#[test]
fn permanganate_ion_plus_ethanol_fires_reaction() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "MnO4-", 0.04);
    let events = add(&mut bench, &mut s, "ethanol", 0.10);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("MnO₄"))),
        "aqueous permanganate + ethanol must react: {events:?}"
    );
}

#[test]
fn permanganate_ion_conserves_elements() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "MnO4-", 0.04);
    let mn_before = total_element(&bench, "Mn");
    let o_before = total_element(&bench, "O");
    let h_before = total_element(&bench, "H");
    add(&mut bench, &mut s, "ethanol", 0.10);
    let mn_after = total_element(&bench, "Mn");
    let o_after = total_element(&bench, "O");
    let h_after = total_element(&bench, "H");
    // Adding 0.10 mol ethanol (C2H5OH) adds 0.10 O, 0.60 H, 0.20 C.
    assert!(
        (mn_after - mn_before).abs() < 1e-9,
        "Mn changed: {mn_before} → {mn_after}"
    );
    assert!(
        (o_after - (o_before + 0.10)).abs() < 1e-9,
        "O: {o_before} + 0.10 ≠ {o_after}"
    );
    assert!(
        (h_after - (h_before + 0.60)).abs() < 1e-9,
        "H: {h_before} + 0.60 ≠ {h_after}"
    );
}

// ── Rendering ────────────────────────────────────────────────────────

#[test]
fn three_registers_render() {
    let reaction = Event::ReactionOccurred {
        vessel: VesselId(0),
        equation: "4 KMnO4 + 3 C₂H₅OH → 4 MnO₂↓ + 3 CH₃COOH + 4 KOH + H₂O".into(),
    };
    for level in [1, 2, 3] {
        let text = render::render_event(&reaction, render::Register(level));
        assert!(!text.is_empty(), "level {level} must produce output");
    }
}

// ── Limiting reagent: excess ethanol ─────────────────────────────────

#[test]
fn excess_ethanol_leaves_ethanol_behind() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 1.0);
    add(&mut bench, &mut s, "KMnO4", 0.04);
    // 0.04 KMnO4 needs 0.03 ethanol (4:3 ratio). 1.0 - 0.03 = 0.97 left.
    let ethanol = total_moles(&bench, "ethanol");
    assert!(
        (ethanol - 0.97).abs() < 1e-9,
        "ethanol left: {ethanol}, expected 0.97"
    );
}
