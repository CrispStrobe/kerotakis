use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Joules, VesselId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench.step(parse_op(command).unwrap().unwrap()).unwrap()
}

#[test]
fn only_a_dry_lemon_mark_browns_on_heating() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 paper 2g");
    let marked = run(&mut bench, "add v1 lemon_juice 2mL");
    assert!(marked
        .iter()
        .any(|e| matches!(e, Event::LemonPaperMarked { .. })));

    let wet_heat = bench
        .step(Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(2000.0),
        })
        .unwrap();
    assert!(!wet_heat
        .iter()
        .any(|e| matches!(e, Event::LemonPaperBrowned { .. })));

    run(&mut bench, "evaporate v1 1");
    let events = bench
        .step(Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(2000.0),
        })
        .unwrap();
    assert!(events.iter().any(|e| matches!(e, Event::LemonPaperBrowned { browned_fraction, .. } if *browned_fraction > 0.0)));
}

#[test]
fn plain_paper_and_lemon_without_paper_do_not_gain_a_mark() {
    let mut paper = Bench::new();
    run(&mut paper, "add v1 paper 2g");
    assert!(paper.vessels[0].lemon_paper_mark.is_none());
    let mut lemon = Bench::new();
    run(&mut lemon, "add v1 lemon_juice 2mL");
    assert!(lemon.vessels[0].lemon_paper_mark.is_none());
}

#[test]
fn mark_state_round_trips_and_old_saves_default_cleanly() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 paper 2g");
    run(&mut bench, "add v1 lemon_juice 2mL");
    let json = serde_json::to_string(&bench).unwrap();
    let restored: Bench = serde_json::from_str(&json).unwrap();
    assert_eq!(
        restored.vessels[0].lemon_paper_mark,
        bench.vessels[0].lemon_paper_mark
    );

    let mut old = serde_json::to_value(&bench).unwrap();
    old["vessels"][0]
        .as_object_mut()
        .unwrap()
        .remove("lemon_paper_mark");
    let restored: Bench = serde_json::from_value(old).unwrap();
    assert!(restored.vessels[0].lemon_paper_mark.is_none());
}
