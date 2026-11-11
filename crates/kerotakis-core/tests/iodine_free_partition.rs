//! SP-011: a distribution coefficient applies to the FREE neutral solute.
//! Iodine bound as the aqueous-only triiodide ion (I2 + I- <=> I3-) stays
//! in the water, so iodide lowers the iodine an organic solvent extracts.
//! Generic over `apparatus::AQUEOUS_ONLY_COMPLEXES`.

use kerotakis_core::apparatus::{
    aqueous_only_complex, extract_free_solute, extract_repeated, AQUEOUS_ONLY_COMPLEXES,
};
use kerotakis_core::iodine_chemistry::{split, TRIIODIDE_K};
use kerotakis_core::{script::parse_op, Bench, Event, SpeciesId, VesselId};

fn bench_after(lines: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut events = Vec::new();
    for line in lines {
        events = bench
            .step(parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    (bench, events)
}

fn iodine_split(events: &[Event]) -> Option<(f64, f64, String)> {
    events.iter().find_map(|event| match event {
        Event::Extracted { solutes, .. } => solutes
            .iter()
            .find(|s| s.species.0 == "I2")
            .map(|s| (s.extracted.0, s.remaining.0, s.model.clone())),
        _ => None,
    })
}

#[test]
fn free_solute_extraction_without_ligand_equals_plain_extraction() {
    for stages in [1usize, 3, 1000] {
        let plain = extract_repeated(1e-4, 0.1, 0.013, 85.0, stages);
        let free = extract_free_solute(1e-4, 0.0, 0.1, 0.013, 85.0, TRIIODIDE_K, stages);
        assert_eq!(plain.organic_moles, free.organic_moles);
        assert_eq!(plain.aqueous_moles, free.aqueous_moles);
    }
}

#[test]
fn only_free_iodine_partitions_and_the_triiodide_stays_aqueous() {
    let (total, iodide, v_aq, v_org, kd) = (1e-4, 1e-3, 0.1, 0.013, 85.0);
    let one = extract_free_solute(total, iodide, v_aq, v_org, kd, TRIIODIDE_K, 1);
    let plain = extract_repeated(total, v_aq, v_org, kd, 1);
    assert!((one.aqueous_moles + one.organic_moles - total).abs() <= 1e-18);
    assert!(one.organic_moles < plain.organic_moles);
    // The organic layer holds K_D times the FREE aqueous I2 concentration
    // that the triiodide split of the remaining water leaves.
    let free = split(one.aqueous_moles, iodide, v_aq).iodine;
    let organic_conc = one.organic_moles / v_org;
    assert!((organic_conc - kd * free / v_aq).abs() <= 1e-9 * organic_conc);
    // Staged: more portions extract more, never more than the total.
    let four = extract_free_solute(total, iodide, v_aq, v_org / 4.0, kd, TRIIODIDE_K, 4);
    assert!(four.organic_moles > one.organic_moles && four.organic_moles <= total);
    let many = extract_free_solute(total, iodide, v_aq, v_org / 1e6, kd, TRIIODIDE_K, 1_000_000);
    assert!(many.organic_moles.is_finite() && many.organic_moles <= total);
}

#[test]
fn every_aqueous_only_complex_row_is_cited_and_findable() {
    for row in AQUEOUS_ONLY_COMPLEXES {
        assert!(row.association_k_l_per_mol > 0.0);
        assert!(row.source.contains("doi:"));
        assert!(aqueous_only_complex(&SpeciesId::new(row.solute)).is_some());
    }
}

#[test]
fn iodide_lowers_the_iodine_hexane_extracts_from_water() {
    let (with_iodide, events) = bench_after(&[
        "add v1 water 100mL",
        "add v1 KI 0.001mol",
        "add v1 I2 0.0001mol",
        "new",
        "extract v1 v2 hexane 0.1mol stages 1",
    ]);
    let (plain, plain_events) = bench_after(&[
        "add v1 water 100mL",
        "add v1 I2 0.0001mol",
        "new",
        "extract v1 v2 hexane 0.1mol stages 1",
    ]);
    let (ext, rem, model) = iodine_split(&events).expect("iodine extracted");
    let (plain_ext, _, plain_model) = iodine_split(&plain_events).expect("iodine extracted");
    assert!(model.contains("I3-"), "{model}");
    assert!(!plain_model.contains("I3-"), "{plain_model}");
    assert!(ext > 0.0 && ext < plain_ext, "{ext} vs {plain_ext}");
    assert!((ext + rem - 1e-4).abs() <= 1e-15);
    let iodine = SpeciesId::new("I2");
    for bench in [&with_iodide, &plain] {
        let total: f64 = bench.vessels.iter().map(|v| v.moles_of(&iodine).0).sum();
        assert!((total - 1e-4).abs() <= 1e-15);
    }
    // Iodide never enters the hexane receiver.
    let receiver = with_iodide.vessel(VesselId(1)).unwrap();
    assert_eq!(receiver.moles_of(&SpeciesId::new("I-")).0, 0.0);
    assert_eq!(receiver.moles_of(&SpeciesId::new("KI")).0, 0.0);
}

#[test]
fn undissolved_iodine_beside_iodide_is_refused_and_atomic() {
    let (mut bench, _) = bench_after(&[
        "add v1 water 2mol",
        "add v1 KI 0.0001mol",
        "add v1 I2 0.01mol",
    ]);
    let events = bench
        .step(
            parse_op("extract v1 v2 hexane 0.5mol stages 1")
                .unwrap()
                .expect("operator"),
        )
        .unwrap();
    assert!(iodine_split(&events).is_none());
    assert!(events.iter().any(|event| matches!(
        event,
        Event::NotYetModeled { what, .. } if what.contains("aqueous I3- complex")
    )));
    // No iodine reached a receiver.
    let iodine = SpeciesId::new("I2");
    let moved: f64 = bench
        .vessels
        .iter()
        .filter(|v| v.id != VesselId(0))
        .map(|v| v.moles_of(&iodine).0)
        .sum();
    assert_eq!(moved, 0.0);
}
