//! EXP-31: the four classic gas tests — pop, glowing splint, limewater,
//! damp litmus — each with positive and negative paths, plus open-vessel
//! refusal and mass conservation.

use kerotakis_core::gas_tests::GasTest;
use kerotakis_core::*;

fn add(bench: &mut Bench, key: &str, moles: f64) {
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        })
        .expect("add");
}

fn seal(bench: &mut Bench, volume_ml: f64) {
    bench
        .step(Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(volume_ml / 1000.0),
        })
        .expect("seal");
}

fn test_gas(bench: &mut Bench, test: GasTest) -> Vec<Event> {
    bench
        .step(Operator::TestGas {
            vessel: VesselId(0),
            test,
        })
        .expect("test_gas")
}

fn gas_tested_positive(events: &[Event]) -> bool {
    events
        .iter()
        .find_map(|e| match e {
            Event::GasTested { positive, .. } => Some(*positive),
            _ => None,
        })
        .expect("a GasTested event")
}

fn gas_tested_notes(events: &[Event]) -> String {
    events
        .iter()
        .find_map(|e| match e {
            Event::GasTested { notes, .. } => Some(notes.clone()),
            _ => None,
        })
        .expect("a GasTested event")
}

fn moles_of(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
}

fn mass_g(bench: &Bench) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter_map(|p| species::lookup(&p.species).map(|d| p.moles.0 * d.molar_mass))
        .sum()
}

fn has_not_yet_modeled(events: &[Event], needle: &str) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains(needle)
        )
    })
}

// ── Pop test (H₂) ─────────────────────────────────────────────────

#[test]
fn pop_positive_with_h2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    // Inject H₂ into the sealed headspace — the vessel already
    // has room-air O₂ from the seal.
    let v = &mut bench.vessels[0];
    v.retain_gas(SpeciesId::new("H2"), Moles(0.005));
    let h2_before = bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new("H2"))
        .0;

    let events = test_gas(&mut bench, GasTest::Pop);
    assert!(gas_tested_positive(&events), "pop should be positive");
    let notes = gas_tested_notes(&events);
    assert!(notes.contains("pop"), "notes describe the pop");
    let h2_after = moles_of(&bench, "H2");
    assert!(
        h2_after < h2_before,
        "H₂ consumed by the pop: before={h2_before} after={h2_after}"
    );
}

#[test]
fn pop_negative_without_h2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    // Sealed with room air only — no H₂.
    let events = test_gas(&mut bench, GasTest::Pop);
    assert!(!gas_tested_positive(&events), "pop should be negative");
}

#[test]
fn pop_conserves_mass() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    let v = &mut bench.vessels[0];
    v.retain_gas(SpeciesId::new("H2"), Moles(0.01));
    let before = mass_g(&bench);
    test_gas(&mut bench, GasTest::Pop);
    let after = mass_g(&bench);
    assert!(
        (after - before).abs() < 1e-6,
        "mass conserved: {before} g → {after} g"
    );
}

#[test]
fn pop_limited_by_o2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    // Lots of H₂ but room air has limited O₂.
    let v = &mut bench.vessels[0];
    // 0.02 mol H₂ in 500 mL keeps pressure below the burst threshold
    // (~2 atm vs 4 atm limit) while exceeding 2×O₂ (~0.0086 mol).
    v.retain_gas(SpeciesId::new("H2"), Moles(0.02));
    let h2_before = bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new("H2"))
        .0;

    let events = test_gas(&mut bench, GasTest::Pop);
    assert!(gas_tested_positive(&events));
    // H₂ exceeds 2×O₂, so excess H₂ should remain after combustion.
    let h2_remaining = moles_of(&bench, "H2");
    assert!(
        h2_remaining > 0.001,
        "excess H₂ should remain: h2_before={h2_before:.4} h2_after={h2_remaining:.6}"
    );
}

// ── Glowing splint (O₂) ───────────────────────────────────────────

#[test]
fn splint_positive_with_enriched_o2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    // Add extra O₂ to push mole fraction above the 25% threshold.
    let v = &mut bench.vessels[0];
    v.retain_gas(SpeciesId::new("O2"), Moles(0.05));

    let events = test_gas(&mut bench, GasTest::GlowingSplint);
    assert!(
        gas_tested_positive(&events),
        "splint should relight in enriched O₂"
    );
    let notes = gas_tested_notes(&events);
    assert!(notes.contains("relight"), "notes say relight");
}

#[test]
fn splint_negative_in_room_air() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    // Room air ~21% O₂ — below the 25% threshold.
    let events = test_gas(&mut bench, GasTest::GlowingSplint);
    assert!(
        !gas_tested_positive(&events),
        "splint should not relight in normal air"
    );
}

// ── Limewater (CO₂) ──────────────────────────────────────────────

#[test]
fn limewater_positive_with_co2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    let v = &mut bench.vessels[0];
    v.retain_gas(SpeciesId::new("CO2"), Moles(0.01));

    let events = test_gas(&mut bench, GasTest::Limewater);
    assert!(gas_tested_positive(&events), "limewater should turn milky");
    let notes = gas_tested_notes(&events);
    assert!(notes.contains("milky"), "notes say milky");
    assert!(notes.contains("CaCO"), "notes mention CaCO₃");
}

#[test]
fn limewater_consumes_co2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    let v = &mut bench.vessels[0];
    v.retain_gas(SpeciesId::new("CO2"), Moles(0.01));
    let co2_before = moles_of(&bench, "CO2");

    test_gas(&mut bench, GasTest::Limewater);
    let co2_after = moles_of(&bench, "CO2");
    assert!(
        co2_after < co2_before,
        "limewater consumed some CO₂: {co2_before} → {co2_after}"
    );
}

#[test]
fn limewater_negative_without_co2() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    // Room air has ~0.04% CO₂ — we trapped negligible amounts.
    let events = test_gas(&mut bench, GasTest::Limewater);
    assert!(
        !gas_tested_positive(&events),
        "limewater should stay clear without CO₂"
    );
}

// ── Damp litmus (NH₃) ────────────────────────────────────────────

#[test]
fn litmus_positive_with_nh3() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    let v = &mut bench.vessels[0];
    v.retain_gas(SpeciesId::new("NH3"), Moles(0.01));

    let events = test_gas(&mut bench, GasTest::DampLitmus);
    assert!(gas_tested_positive(&events), "litmus should turn blue");
    let notes = gas_tested_notes(&events);
    assert!(notes.contains("blue"), "notes say blue");
}

#[test]
fn litmus_negative_without_nh3() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    let events = test_gas(&mut bench, GasTest::DampLitmus);
    assert!(
        !gas_tested_positive(&events),
        "litmus should stay red without NH₃"
    );
}

// ── Open vessel refusal ──────────────────────────────────────────

#[test]
fn open_vessel_refuses_pop() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    // Vessel is open — no headspace inventory.
    let events = test_gas(&mut bench, GasTest::Pop);
    assert!(
        has_not_yet_modeled(&events, "sealed vessel"),
        "refuses with sealed-vessel guidance"
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::GasTested { .. })),
        "no gas test event on open vessel"
    );
}

#[test]
fn open_vessel_refuses_limewater() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    let events = test_gas(&mut bench, GasTest::Limewater);
    assert!(
        has_not_yet_modeled(&events, "sealed vessel"),
        "refuses with sealed-vessel guidance"
    );
}

// ── Script parsing ───────────────────────────────────────────────

#[test]
fn test_pop_parses() {
    let op = script::parse_op("test v1 pop")
        .expect("grammar")
        .expect("an operator");
    assert!(matches!(
        op,
        Operator::TestGas {
            test: GasTest::Pop,
            ..
        }
    ));
}

#[test]
fn test_splint_parses() {
    assert!(script::parse_op("test v1 splint")
        .expect("grammar")
        .is_some());
}

#[test]
fn test_limewater_parses() {
    assert!(script::parse_op("test v1 limewater")
        .expect("grammar")
        .is_some());
}

#[test]
fn test_litmus_parses() {
    assert!(script::parse_op("test v1 litmus")
        .expect("grammar")
        .is_some());
}

#[test]
fn unknown_test_errors() {
    let err = script::parse_op("test v1 bogus").unwrap_err();
    assert!(err.contains("pop"), "error lists valid options, got: {err}");
}

// ── A dissolved gas the bench cannot let out ──────────────────────
//
// `NH3` in this registry is *ammonia solution* — standard phase Liquid,
// formula NH3(aq) — and there is no gaseous ammonia species at all. The
// tests above reach a positive litmus only through `retain_gas`, which puts
// NH₃ in the headspace directly; no operator a learner can type does that.
// So the path a learner actually walks — pour ammonia into a sealed vessel,
// hold litmus over it — read "litmus stays red", a confident negative about
// a gas the bench had never let out of the liquid, and one that teaches the
// opposite of the chemistry it is demonstrating.

#[test]
fn dissolved_ammonia_is_not_reported_as_absent() {
    let mut bench = Bench::new();
    seal(&mut bench, 500.0);
    add(&mut bench, "NH3", 0.01);

    let events = test_gas(&mut bench, GasTest::DampLitmus);
    // Not `!gas_tested_positive(..)` — that helper unwraps the verdict and
    // so asserts one exists. The claim here is that NO verdict is given: a
    // negative would be as wrong as a positive.
    assert!(
        !events.iter().any(|e| matches!(e, Event::GasTested { .. })),
        "no verdict either way, because the bench cannot see: {events:?}"
    );
    assert!(
        has_not_yet_modeled(&events, "no path from"),
        "it says why it cannot answer: {events:?}"
    );
}

/// The refusal is narrow: a vessel with no ammonia at all still reads
/// negative, because that is a true statement about the world rather than a
/// gap in the model.
#[test]
fn an_empty_vessel_still_reads_a_true_negative() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);

    let events = test_gas(&mut bench, GasTest::DampLitmus);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::GasTested {
                positive: false,
                ..
            }
        )),
        "no ammonia anywhere is an honest negative: {events:?}"
    );
}

/// And a gas that IS in the headspace is still read, whatever the registry
/// calls its standard phase — so a gaseous ammonia added later opens the
/// positive path by existing, without this guard needing an edit.
#[test]
fn ammonia_in_the_headspace_is_still_detected() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 1.0);
    seal(&mut bench, 500.0);
    bench.vessels[0].retain_gas(SpeciesId::new("NH3"), Moles(0.01));

    let events = test_gas(&mut bench, GasTest::DampLitmus);
    assert!(
        gas_tested_positive(&events),
        "gas in the headspace is gas the test can read: {events:?}"
    );
}

/// The three gases the bench does carry are untouched by the guard — they
/// are `Phase::Gas` in the registry, so it never fires for them.
#[test]
fn the_carried_gases_are_unaffected() {
    for (key, test) in [
        ("H2", GasTest::Pop),
        ("O2", GasTest::GlowingSplint),
        ("CO2", GasTest::Limewater),
    ] {
        let mut bench = Bench::new();
        seal(&mut bench, 500.0);
        add(&mut bench, key, 0.05);
        let events = test_gas(&mut bench, test);
        assert!(
            gas_tested_positive(&events),
            "{key} is carried as a gas and {test} should read it: {events:?}"
        );
    }
}

/// The divergence the refusal names, pinned so a fix closes both sides.
///
/// `senses::waft` walks `vessel.contents` directly and treats a dissolved
/// odorous species as reaching the nose, so `smell` reports ammonia off the
/// solution. The gas tests read the headspace inventory, and nothing moves
/// dissolved NH₃ into it. One vessel, one physical fact, two answers.
///
/// This asserts the CURRENT state, not the desired one. When a path from
/// solution to headspace exists — a gaseous ammonia species, or NH₃ joining
/// the approved gas/liquid exchange — this test should fail, and the right
/// response is to delete it and let `litmus_positive_with_nh3` cover the
/// case through the route a learner can actually walk.
#[test]
fn smell_and_gas_test_disagree_about_dissolved_ammonia() {
    let mut bench = Bench::new();
    seal(&mut bench, 500.0);
    add(&mut bench, "NH3", 0.01);

    let smelled = bench
        .step(Operator::Smell {
            vessel: VesselId(0),
        })
        .expect("smell");
    assert!(
        format!("{smelled:?}").to_lowercase().contains("ammonia"),
        "the nose finds it in the liquid: {smelled:?}"
    );

    let tested = test_gas(&mut bench, GasTest::DampLitmus);
    assert!(
        has_not_yet_modeled(&tested, "no path from"),
        "the headspace cannot: {tested:?}"
    );
}
