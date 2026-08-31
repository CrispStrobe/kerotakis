//! PHREEQC MIX routing: mixing two solved solutions by fraction.

#![cfg(feature = "engine")]

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
        .expect("add")
}

/// Every reagent poured into one beaker, in order.
fn direct(adds: &[(&str, f64)]) -> Vec<Event> {
    let mut bench = Bench::new();
    let mut stack = stack();
    let mut events = Vec::new();
    for (key, moles) in adds {
        events.extend(add(&mut bench, &mut stack, VesselId(0), key, *moles));
    }
    events
}

/// The same reagents reached the other way: each list made up as its own
/// solution, then the two combined whole into a third vessel.
fn mixed(a: &[(&str, f64)], b: &[(&str, f64)]) -> Vec<Event> {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v1
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    for (key, moles) in a {
        add(&mut bench, &mut stack, VesselId(0), key, *moles);
    }
    for (key, moles) in b {
        add(&mut bench, &mut stack, VesselId(1), key, *moles);
    }
    bench
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 1.0,
                fraction_b: 1.0,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix")
}

fn precipitated(events: &[Event], species: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Precipitated {
                species: s, moles, ..
            } if s.0 == species => Some(moles.0),
            _ => None,
        })
        .sum()
}

/// A4 — the same chemistry must give the same answer whether the beaker was
/// reached directly or by combining two solutions by fraction.
///
/// The candidate phase list is database-blind on purpose, and the direct
/// path reconciles it with the routed database in two ways the MIX input
/// builder used to skip, because it filtered candidates to names the
/// database defines natively:
///
/// * **polymorph translation** — the registry names ferric hydroxide
///   `Fe(OH)3`, which wateq4f spells `Fe(OH)3(a)`;
/// * **foreign-phase injection** — wateq4f spells no ferrous hydroxide at
///   all, and `Fe(OH)2` reaches the input only as the reviewed `PHASES`
///   definition carrying its home database's log K.
///
/// Filtered away, neither solid could form on the MIX solve itself. Note
/// what this test can and cannot see: `Bench` swallows a failed MIX and
/// silently re-equilibrates the target through the direct path, so the
/// precipitate arrived either way and these two assertions passed even
/// before the fix. They pin the parity that must hold at the bench, no more
/// than that. Whether the MIX solve reaches it *itself* is a separate
/// question, and still an open one — see the ignored
/// `mix_solves_in_one_engine_call_without_falling_back` below.
///
/// Ferric iron as the chloride, at the dilution `school_salts` pins: 0.1
/// mol/kgw hydrolyses to pH 1.9, where the amorphous hydroxide is *not*
/// supersaturated. That matters for the comparison rather than for the
/// chemistry — `Mix` moves the liquid and what is dissolved in it, so iron
/// that had already precipitated in its own beaker would never reach the
/// third vessel, and the two paths would be asked different questions.
#[test]
fn mix_translates_a_polymorph_the_way_the_direct_path_does() {
    let direct_moles = precipitated(
        &direct(&[("water", 11.1), ("FeCl3", 0.01), ("NaOH", 0.04)]),
        "Fe(OH)3",
    );
    assert!(
        direct_moles > 0.009,
        "the direct path must give the red-brown hydroxide, got {direct_moles}"
    );
    let mix_moles = precipitated(
        &mixed(
            &[("water", 5.55), ("FeCl3", 0.01)],
            &[("water", 5.55), ("NaOH", 0.04)],
        ),
        "Fe(OH)3",
    );
    assert!(
        (mix_moles - direct_moles).abs() <= 0.05 * direct_moles,
        "MIX must reach the direct path's answer: {mix_moles} vs {direct_moles}"
    );
}

/// The rest of A4, recorded as a failing guard rather than a claim.
///
/// Posing the phase was necessary and is done; it is not sufficient, and this
/// test is the standing proof of what is still missing. It fails today. Do not
/// un-ignore it until the MIX builder closes the two gaps below — at which
/// point it becomes the regression guard the phase fix alone cannot be.
///
/// A parity assertion on the precipitate cannot see any of this. `Bench`
/// treats a failed MIX as advisory — "MIX failed; fall through to normal
/// equilibrate" — drops the error and re-solves the target through the direct
/// path, which reaches the right chemistry. Right answer, wrong route, no
/// event to show for it. The route is only observable in what it costs: `mix`
/// runs the engine once (`run_raw`, the only such call on that path), so a
/// second call means the solve died and the fallback rescued it.
///
/// Compared against the direct input for the same merged solution, the MIX
/// input still lacks two things (dump either with `KERO_DUMP_INPUT=all`):
///
/// * **the FAST_REDOX pin.** `build_input_at` emits `Fe+2 = Fe+3 + 1 e-`
///   at `log_k 50` *because* a phase is posed — PHREEQC redistributes an
///   uncoupled element across its states against pe on any reaction step.
///   `build_mix_input` emits no such block and does not carry the second
///   state (`Fe(2)`) in `-totals`, so the iron that moves there vanishes
///   from the readback's mass balance. Note this gap is *newly reached* by
///   the phase fix: before it, the MIX path posed no phase at all, so the
///   redistribution never fired.
/// * **phases whose elements only meet in the mixture.** `merged_phases` is
///   the union of the two solutions' own candidate lists, and neither an
///   iron chloride nor a lye solution proposes `Halite` on its own — only
///   the merged element set does, which is why the direct input poses it
///   and the MIX input does not.
#[test]
#[ignore = "A4 remainder: MIX still lacks the FAST_REDOX pin and merged-element \
            candidates, so the solve dies and Bench silently falls back"]
fn mix_solves_in_one_engine_call_without_falling_back() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    for (vessel, key, moles) in [
        (VesselId(0), "water", 5.55),
        (VesselId(0), "FeCl3", 0.01),
        (VesselId(1), "water", 5.55),
        (VesselId(1), "NaOH", 0.04),
    ] {
        bench
            .step_with(
                Operator::Add {
                    vessel,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut eq,
                &PermissiveScreen,
            )
            .expect("add");
    }

    let before = eq.engine_calls();
    let events = bench
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 1.0,
                fraction_b: 1.0,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("mix");
    let spent = eq.engine_calls() - before;
    eprintln!("MIX spent {spent} engine call(s)");

    assert!(
        precipitated(&events, "Fe(OH)3") > 0.009,
        "the MIX path must grow the red-brown hydroxide"
    );
    assert_eq!(
        spent, 1,
        "MIX must solve in one engine call; {spent} means the MIX solve failed \
         and Bench silently re-solved the vessel through the direct path"
    );
}

#[test]
fn mix_injects_the_reviewed_foreign_phase_the_way_the_direct_path_does() {
    let direct_moles = precipitated(
        &direct(&[("water", 5.0), ("FeSO4", 0.01), ("NaOH", 0.03)]),
        "Fe(OH)2",
    );
    assert!(
        direct_moles > 0.009,
        "the direct path must give the green hydroxide, got {direct_moles}"
    );
    let mix_moles = precipitated(
        &mixed(
            &[("water", 2.5), ("FeSO4", 0.01)],
            &[("water", 2.5), ("NaOH", 0.03)],
        ),
        "Fe(OH)2",
    );
    assert!(
        (mix_moles - direct_moles).abs() <= 0.05 * direct_moles,
        "MIX must reach the direct path's answer: {mix_moles} vs {direct_moles}"
    );
}

#[test]
fn mix_two_salt_solutions_conserves_mass() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    add(&mut bench, &mut stack, VesselId(0), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(0), "NaCl", 0.1);

    add(&mut bench, &mut stack, VesselId(1), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(1), "KCl", 0.1);

    let mass_before: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();

    let events = bench
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 0.5,
                fraction_b: 0.5,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix");

    let mass_after: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    assert!(
        (mass_before - mass_after).abs() / mass_before < 1e-3,
        "mass must be conserved: {mass_before} vs {mass_after}"
    );

    assert!(
        events.iter().any(|e| matches!(e, Event::Mixed { .. })),
        "must emit a Mixed event"
    );

    // The mixed solution should be characterized.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::SolutionCharacterized { .. })),
        "mixed solution should be characterized"
    );
}

#[test]
fn mix_acid_and_base_produces_neutral_ph() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3

    add(&mut bench, &mut stack, VesselId(0), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(0), "HCl", 0.01);

    add(&mut bench, &mut stack, VesselId(1), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(1), "NaOH", 0.01);

    bench
        .step_with(
            Operator::Mix {
                a: VesselId(0),
                b: VesselId(1),
                into: VesselId(2),
                fraction_a: 1.0,
                fraction_b: 1.0,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix");

    let v3 = bench.vessel(VesselId(2)).unwrap();
    let ph = v3.solution.as_ref().expect("solution").ph;
    assert!(
        (ph - 7.0).abs() < 0.5,
        "equimolar HCl + NaOH → near-neutral pH, got {ph:.2}"
    );
}

#[test]
fn hard_water_lesson_replays() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v3
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v4
    bench.step(Operator::NewVessel { kind: None }).unwrap(); // v5

    // Hard water with dissolved calcium and magnesium.
    add(&mut bench, &mut stack, VesselId(2), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(2), "CaCl2", 5e-3);
    add(&mut bench, &mut stack, VesselId(2), "MgSO4", 3e-3);

    // Soft water (just sodium chloride).
    add(&mut bench, &mut stack, VesselId(3), "water", 27.75);
    add(&mut bench, &mut stack, VesselId(3), "NaCl", 10e-3);

    // Mix them.
    let events = bench
        .step_with(
            Operator::Mix {
                a: VesselId(2),
                b: VesselId(3),
                into: VesselId(4),
                fraction_a: 0.5,
                fraction_b: 0.5,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("mix");

    assert!(
        events.iter().any(|e| matches!(e, Event::Mixed { .. })),
        "must emit Mixed event"
    );

    let v5 = bench.vessel(VesselId(4)).unwrap();
    assert!(
        v5.solution.is_some(),
        "mixed solution should be characterized"
    );
}
