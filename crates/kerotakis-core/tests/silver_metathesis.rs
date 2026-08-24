//! CAP-23 rung 2b: silver metathesis in ethanol.

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
fn nacl_equation_balances() {
    let eq = stoich::parse_equation("AgNO3 + NaCl → AgCl + NaNO3").expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

#[test]
fn kcl_equation_balances() {
    let eq = stoich::parse_equation("AgNO3 + KCl → AgCl + KNO3").expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

// ── Ethanol bench: AgNO3 + NaCl ──────────────────────────────────────

#[test]
fn silver_nacl_fires_in_ethanol() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 5.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    let events = add(&mut bench, &mut s, "NaCl", 0.1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("AgCl"))),
        "silver metathesis must fire in ethanol: {events:?}"
    );
}

#[test]
fn silver_kcl_fires_in_ethanol() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 5.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    let events = add(&mut bench, &mut s, "KCl", 0.1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("AgCl"))),
        "silver/KCl metathesis must fire in ethanol: {events:?}"
    );
}

#[test]
fn agcl_precipitates_as_solid() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 5.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    add(&mut bench, &mut s, "NaCl", 0.1);
    let agcl = moles_in_phase(&bench, "AgCl", species::Phase::Solid);
    assert!(agcl > 0.0, "AgCl solid expected");
}

#[test]
fn dissolved_fractions_only() {
    let mut bench = Bench::new();
    let mut s = stack();
    // ~5 mol ethanol ≈ 290 mL; AgNO3 solubility 2.1 g/100 mL;
    // NaCl solubility 0.065 g/100 mL → ~0.003 mmol dissolved NaCl.
    // The dissolved NaCl is the limiting reagent, not the 1 mol solid.
    add(&mut bench, &mut s, "ethanol", 5.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    add(&mut bench, &mut s, "NaCl", 1.0);

    let nacl_solid = moles_in_phase(&bench, "NaCl", species::Phase::Solid);
    let agcl = moles_in_phase(&bench, "AgCl", species::Phase::Solid);

    // Most NaCl must remain undissolved: the reaction consumed only
    // the dissolved fraction, not the solid on the bottom.
    assert!(
        nacl_solid > 0.99,
        "undissolved NaCl should stay: {nacl_solid}"
    );
    // AgCl formed should be tiny (limited by NaCl solubility in ethanol).
    assert!(
        agcl > 0.0 && agcl < 0.01,
        "AgCl should be small (dissolved fraction only): {agcl}"
    );
}

#[test]
fn conserves_elements() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 5.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    let ag_before = total_element(&bench, "Ag");
    let na_before = total_element(&bench, "Na");
    let cl_before = total_element(&bench, "Cl");
    let n_before = total_element(&bench, "N");
    let o_before = total_element(&bench, "O");
    add(&mut bench, &mut s, "NaCl", 0.1);
    // Adding 0.1 mol NaCl adds 0.1 Na, 0.1 Cl.
    let ag_after = total_element(&bench, "Ag");
    let na_after = total_element(&bench, "Na");
    let cl_after = total_element(&bench, "Cl");
    let n_after = total_element(&bench, "N");
    let o_after = total_element(&bench, "O");
    assert!(
        (ag_after - ag_before).abs() < 1e-9,
        "Ag changed: {ag_before} → {ag_after}"
    );
    assert!(
        (na_after - (na_before + 0.1)).abs() < 1e-9,
        "Na: {na_before} + 0.1 ≠ {na_after}"
    );
    assert!(
        (cl_after - (cl_before + 0.1)).abs() < 1e-9,
        "Cl: {cl_before} + 0.1 ≠ {cl_after}"
    );
    assert!(
        (n_after - n_before).abs() < 1e-9,
        "N changed: {n_before} → {n_after}"
    );
    assert!(
        (o_after - o_before).abs() < 1e-9,
        "O changed: {o_before} → {o_after}"
    );
}

// ── Water suppression: PHREEQC handles aqueous ──────────────────────

#[test]
fn does_not_fire_in_water() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    let events = add(&mut bench, &mut s, "NaCl", 0.01);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("AgCl"))),
        "silver metathesis must NOT fire in water (PHREEQC handles it): {events:?}"
    );
}

// ── No apology ──────────────────────────────────────────────────────

#[test]
fn no_apology_for_covered_species() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "ethanol", 5.0);
    add(&mut bench, &mut s, "AgNO3", 0.01);
    let events = add(&mut bench, &mut s, "NaCl", 0.1);
    let apologies: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::NotYetModeled { .. }))
        .collect();
    assert!(
        apologies.is_empty(),
        "no apology expected for covered species: {apologies:?}"
    );
}

// ── Rendering ────────────────────────────────────────────────────────

#[test]
fn three_registers_render() {
    let reaction = Event::ReactionOccurred {
        vessel: VesselId(0),
        equation: "AgNO₃ + NaCl → AgCl↓ + NaNO₃".into(),
    };
    for level in [1, 2, 3] {
        let text = render::render_event(&reaction, render::Register(level));
        assert!(!text.is_empty(), "level {level} must produce output");
    }
}
