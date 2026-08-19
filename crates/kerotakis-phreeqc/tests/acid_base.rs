//! Acid–base chemistry through charge balance, titration to equivalence,
//! reaction heat in the energy balance, and the L0 veto in the full loop.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

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
            stack,
            &ReactiveGroupScreen,
        )
        .expect("step")
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph
}

#[test]
fn hydrochloric_acid_is_acidic() {
    // 0.001 mol HCl in 1 kg water → pH ≈ 3.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "HCl", 0.001);
    let ph = ph(&bench, v);
    assert!(
        (ph - 3.0).abs() < 0.1,
        "0.001 m HCl should be pH ~3, got {ph}"
    );
}

#[test]
fn sodium_hydroxide_is_basic() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaOH", 0.001);
    let ph = ph(&bench, v);
    assert!(
        (ph - 11.0).abs() < 0.2,
        "0.001 m NaOH should be pH ~11, got {ph}"
    );
}

#[test]
fn titration_walks_the_curve_to_equivalence() {
    // Strong acid titrated with strong base: acidic → equivalence (≈7) →
    // basic, all from charge balance against the same database.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "HCl", 0.01);
    let start = ph(&bench, v);
    assert!(start < 2.3, "0.01 m strong acid, got pH {start}");

    add(&mut bench, &mut stack, v, "NaOH", 0.005);
    let halfway = ph(&bench, v);
    assert!(
        halfway > start && halfway < 3.0,
        "half-neutralised strong acid stays acidic, got pH {halfway}"
    );

    add(&mut bench, &mut stack, v, "NaOH", 0.005);
    let equivalence = ph(&bench, v);
    assert!(
        (equivalence - 7.0).abs() < 0.3,
        "equivalence point of strong-strong titration is ~7, got {equivalence}"
    );

    add(&mut bench, &mut stack, v, "NaOH", 0.001);
    let excess = ph(&bench, v);
    assert!(excess > 10.5, "excess base swings basic, got pH {excess}");
}

#[test]
fn dissolving_sodium_hydroxide_warms_the_water() {
    // 0.1 mol NaOH (ΔH_dis = −44.5 kJ/mol) into 100 mL water:
    // Q = 4.45 kJ into Cp ≈ 418 J/K → ΔT ≈ +10.6 K.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.55);
    let events = add(&mut bench, &mut stack, v, "NaOH", 0.1);

    assert!(
        events.iter().any(
            |e| matches!(e, Event::TemperatureChanged { to, from, .. } if to.0 > from.0 + 5.0)
        ),
        "dissolving NaOH must warm the vessel, got {events:?}"
    );
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        (t - 35.6).abs() < 2.0,
        "expected ~35.6 °C after the exotherm, got {t:.1} °C"
    );
}

#[test]
fn endothermic_salt_cools_slightly() {
    // NaCl ΔH_dis = +3.88 kJ/mol: 1 mol into 1 L water → ΔT ≈ −0.9 K.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaCl", 1.0);
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        t < 25.0 && t > 23.5,
        "dissolving NaCl cools slightly, got {t:.2} °C"
    );
}

#[test]
fn bleach_and_ammonia_warns_then_shows_the_chloramine() {
    // Pedagogy over prohibition: the warning always comes first, and then
    // the virtual lab shows precisely what would happen.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.55);
    add(&mut bench, &mut stack, v, "NaOCl", 0.1);
    let events = add(&mut bench, &mut stack, v, "NH3", 0.1);

    let warn_pos = events
        .iter()
        .position(
            |e| matches!(e, Event::HazardWarning { hazard, .. } if hazard.contains("chloramine")),
        )
        .expect("hazard warning must be present");
    let gas_pos = events
        .iter()
        .position(
            |e| matches!(e, Event::GasEvolved { species, moles, .. } if species.0 == "NH2Cl" && (moles.0 - 0.1).abs() < 1e-9),
        )
        .expect("chloramine gas must actually evolve");
    assert!(warn_pos < gas_pos, "the warning precedes the chemistry");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::ReactionOccurred { equation, .. } if equation.contains("NH2Cl"))
        ),
        "the equation is shown"
    );

    // The reactants are consumed; the NaOH byproduct makes it basic.
    let vessel = bench.vessel(v).unwrap();
    assert!((vessel.moles_of(&SpeciesId::new("NaOCl")).0).abs() < 1e-9);
    assert!((vessel.moles_of(&SpeciesId::new("NH3")).0).abs() < 1e-9);
    let ph = vessel.solution.clone().expect("characterised").ph;
    assert!(
        ph > 12.0,
        "0.1 mol NaOH byproduct in 100 mL is strongly basic, got pH {ph}"
    );
}

#[test]
fn decanting_bleach_into_ammonia_warns_first() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel).unwrap();
    let (a, b) = (VesselId(0), VesselId(1));
    add(&mut bench, &mut stack, a, "NaOCl", 0.1);
    add(&mut bench, &mut stack, b, "NH3", 0.1);
    let events = bench
        .step_with(
            Operator::Decant {
                from: a,
                to: b,
                fraction: 0.5,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("step");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::HazardWarning { .. })),
        "pouring bleach into ammonia must warn, got {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "NH2Cl")),
        "and the gas forms in the target vessel, got {events:?}"
    );
}
