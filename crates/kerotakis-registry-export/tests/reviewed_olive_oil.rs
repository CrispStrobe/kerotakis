use kerotakis_data::RegistryDocument;
#[test]
fn olive_reference_density_and_bounded_recipe_survive_reexport() {
    let source: RegistryDocument = serde_json::from_str(include_str!(
        "../../../data/registry/registry-source-v1.json"
    ))
    .unwrap();
    let exported = kerotakis_registry_export::export_current_registry().unwrap();
    let expected = source
        .material_recipes
        .iter()
        .find(|r| r.canonical_key == "olive_oil")
        .unwrap();
    let actual = exported
        .material_recipes
        .iter()
        .find(|r| r.canonical_key == "olive_oil")
        .unwrap();
    assert_eq!(actual, expected);
    let source_id = &actual.bulk_density.as_ref().unwrap().source_id;
    assert_eq!(
        exported.sources.iter().find(|s| &s.id == source_id),
        source.sources.iter().find(|s| &s.id == source_id)
    );
    assert_eq!(
        exported
            .sources
            .iter()
            .filter(|s| &s.id == source_id)
            .count(),
        1
    );
}
