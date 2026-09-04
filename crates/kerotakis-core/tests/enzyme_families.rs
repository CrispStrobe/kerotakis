use kerotakis_core::enzyme::{self, EnzymeFamily};
use kerotakis_core::script::parse_op;
use kerotakis_core::{species, Bench, SpeciesId};

#[test]
fn four_enzyme_families_are_declared_with_distinct_substrates() {
    assert_eq!(enzyme::FAMILIES.len(), 4);
    assert_eq!(
        enzyme::profile("lactase").map(|profile| profile.family),
        Some(EnzymeFamily::Lactase)
    );
    assert_eq!(
        enzyme::profile("protease").map(|profile| profile.acts_on),
        Some("protein peptide bonds")
    );
    assert_eq!(
        enzyme::profile("lipase").map(|profile| profile.acts_on),
        Some("triglycerides")
    );
    assert_eq!(
        enzyme::profile("catalase").map(|profile| profile.acts_on),
        Some("hydrogen peroxide")
    );
}

#[test]
fn newly_exposed_enzyme_catalysts_can_be_added_to_the_bench() {
    for key in ["lactase", "protease", "lipase", "catalase"] {
        assert!(species::lookup(&SpeciesId::new(key)).is_some(), "{key}");
        let mut bench = Bench::new();
        let op = parse_op(&format!("add v1 {key} 0.000001mol"))
            .expect("parse")
            .expect("operator");
        bench.step(op).expect("known enzyme");
        assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new(key)).0, 1e-6);
    }
}
