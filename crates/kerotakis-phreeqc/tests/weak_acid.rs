//! Weak-acid chemistry from the database's own equilibria (H+ + Acetate- =
//! HAc, pKa ≈ 4.76 in minteq.v4), buffers, and the content-addressed cache.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn add_with(
    bench: &mut Bench,
    solver: &mut dyn Equilibrator,
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
            solver,
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
fn acetic_acid_is_weakly_acidic() {
    // 0.1 m acetic acid: pH ≈ 2.9 — far above the 1.0 a strong acid would
    // give at the same concentration. The difference IS the weak-acid lesson.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "CH3COOH", 0.1);
    let ph = ph(&bench, v);
    assert!(
        (ph - 2.9).abs() < 0.2,
        "0.1 m acetic acid should be pH ~2.9, got {ph}"
    );
}

#[test]
fn equimolar_buffer_sits_at_pka() {
    // Henderson–Hasselbalch without ever writing it down: 1:1
    // acid/conjugate-base sits at pKa ≈ 4.76 (activity effects shift it a
    // touch).
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "CH3COOH", 0.1);
    add_with(&mut bench, &mut eq, v, "NaOAc", 0.1);
    let ph = ph(&bench, v);
    assert!(
        (ph - 4.76).abs() < 0.25,
        "equimolar acetate buffer should sit near pKa 4.76, got {ph}"
    );
}

#[test]
fn buffer_resists_acid_where_water_does_not() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");

    // Buffered vessel.
    let mut buffered = Bench::new();
    let v = VesselId(0);
    add_with(&mut buffered, &mut eq, v, "water", 55.51);
    add_with(&mut buffered, &mut eq, v, "CH3COOH", 0.1);
    add_with(&mut buffered, &mut eq, v, "NaOAc", 0.1);
    let before_buffered = ph(&buffered, v);
    add_with(&mut buffered, &mut eq, v, "HCl", 0.01);
    let shift_buffered = (ph(&buffered, v) - before_buffered).abs();

    // Plain water, same acid.
    let mut plain = Bench::new();
    add_with(&mut plain, &mut eq, v, "water", 55.51);
    add_with(&mut plain, &mut eq, v, "HCl", 0.01);
    let shift_plain = (ph(&plain, v) - 7.0).abs();

    assert!(
        shift_buffered < 0.15,
        "the buffer should barely move, shifted {shift_buffered}"
    );
    assert!(
        shift_plain > 4.0,
        "plain water should crash to ~pH 2, shifted {shift_plain}"
    );
}

#[test]
fn half_neutralised_weak_acid_reads_pka() {
    // The classic titration midpoint: half the acid converted → pH = pKa.
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add_with(&mut bench, &mut eq, v, "water", 55.51);
    add_with(&mut bench, &mut eq, v, "CH3COOH", 0.02);
    add_with(&mut bench, &mut eq, v, "NaOH", 0.01);
    let ph = ph(&bench, v);
    assert!(
        (ph - 4.76).abs() < 0.25,
        "half-neutralised acetic acid reads pKa, got {ph}"
    );
}

#[test]
fn identical_states_hit_the_cache() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");

    let run = |eq: &mut PhreeqcEquilibrator| {
        let mut bench = Bench::new();
        let v = VesselId(0);
        add_with(&mut bench, eq, v, "water", 55.51);
        add_with(&mut bench, eq, v, "NaCl", 0.05);
        add_with(&mut bench, eq, v, "CH3COOH", 0.1);
        ph(&bench, v)
    };

    let first = run(&mut eq);
    assert_eq!(eq.cache_hits(), 0, "first run is all engine calls");
    let second = run(&mut eq);
    // Two solver-reaching steps per run (plain water never reaches the
    // engine); the replay is served entirely from the cache.
    assert_eq!(eq.cache_hits(), 2, "identical replay must hit the cache");
    assert_eq!(first, second, "cached answers are bit-identical");
}
