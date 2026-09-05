use kerotakis_core::material;

#[test]
fn prepared_materials_are_distinct_and_do_not_leak_to_sources() {
    for key in ["naked_egg", "cut_apple", "fatty_soap"] {
        assert_eq!(material::lookup(key, None).unwrap().canonical_key, key);
    }
    assert_ne!(
        material::lookup("apple", None).unwrap().id,
        material::lookup("cut_apple", None).unwrap().id
    );
    assert_ne!(
        material::lookup("egg_white", None).unwrap().id,
        material::lookup("naked_egg", None).unwrap().id
    );
    assert_ne!(
        material::lookup("dish_soap", None).unwrap().id,
        material::lookup("fatty_soap", None).unwrap().id
    );
}
