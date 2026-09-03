//! BRD-001: the particle census is an operator, so a SCRIPT can ask what a
//! learner can ask in the REPL.
//!
//! `particles` answered "what dissolved ions are present?" perfectly and no
//! corpus prompt could pose it, because the lint refuses a session command:
//! *"script line 3 is a session command, not an operator"*. The engine could
//! answer and the script surface could not ask.

use kerotakis_core::*;

#[test]
fn a_script_can_ask_for_the_census() {
    assert_eq!(
        script::parse_op("particles v1").unwrap(),
        Some(Operator::Particles {
            vessel: VesselId(0)
        })
    );
    // `zoom` is the long-standing alias and stays one.
    assert_eq!(
        script::parse_op("zoom v2").unwrap(),
        Some(Operator::Particles {
            vessel: VesselId(1)
        })
    );
}

/// The corpus lint rejects any script line that is not an operator, which is
/// exactly what kept aq-049 from asking its own question. Parsing to
/// `Some(op)` rather than `None` is the whole difference.
#[test]
fn the_census_is_no_longer_a_session_command() {
    assert!(
        script::parse_op("particles v1").unwrap().is_some(),
        "a session command parses to None, and a corpus script may not carry one"
    );
    // The ones that really are shell questions stay shell questions.
    for shell in ["register lv2", "species", "help", "inspect"] {
        assert!(
            script::parse_op(shell).unwrap().is_none(),
            "{shell} asks the shell, not the bench"
        );
    }
}

#[test]
fn the_census_names_what_is_dissolved() {
    let mut bench = Bench::new();
    for (key, moles) in [("water", 5.55), ("NaCl", 0.01)] {
        bench
            .step(Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            })
            .expect("add");
    }
    let events = bench
        .step(Operator::Particles {
            vessel: VesselId(0),
        })
        .expect("particles");

    let census = events
        .iter()
        .find_map(|e| match e {
            Event::ParticlesCounted { census, .. } => Some(census.clone()),
            _ => None,
        })
        .expect("the census is an event now, not a printed string");
    assert!(
        census
            .populations
            .iter()
            .any(|p| p.label.contains("Na") || p.label.contains("Cl")),
        "the salt's own particles are drawn: {:?}",
        census.populations
    );
}

/// Reading a vessel must not change it. The census is an observation, and
/// `Smell` is the neighbour it was modelled on.
#[test]
fn drawing_the_census_changes_nothing() {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(5.55),
            at: None,
        })
        .expect("add");
    let before = bench.vessel(VesselId(0)).unwrap().clone();
    bench
        .step(Operator::Particles {
            vessel: VesselId(0),
        })
        .expect("particles");
    let after = bench.vessel(VesselId(0)).unwrap();
    assert_eq!(before.contents, after.contents, "contents untouched");
    assert_eq!(before.temperature.0, after.temperature.0, "no heat moved");
}
