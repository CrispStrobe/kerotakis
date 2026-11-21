//! G4: the green "copper carbonate" of a school shelf is malachite.

use kerotakis_core::*;

#[test]
fn cuco3_is_refused_with_malachite_named() {
    let refusal = script::unknown_ingredient("CuCO3");
    assert_eq!(
        refusal.key,
        "error.unknown-ingredient-basic-copper-carbonate"
    );
    assert!(refusal.en.contains("malachite"));
    assert!(refusal.en.contains("two copper atoms"));
}

#[test]
fn malachite_and_its_formula_both_reach_the_registry_solid() {
    for spelling in ["malachite", "Cu2(OH)2CO3"] {
        let op = script::parse_op(&format!("add v1 {spelling} 1g"))
            .unwrap_or_else(|e| panic!("{spelling}: {e:?}"))
            .expect("operator");
        let Operator::Add { species, .. } = op else {
            panic!("{spelling}: not an add");
        };
        assert_eq!(species.0, "Cu2(OH)2CO3", "{spelling}");
    }
}

#[test]
fn the_new_school_solids_are_on_the_shelf() {
    for key in ["ZnO", "Al2O3", "MgCO3", "ZnCO3", "Cu2(OH)2CO3"] {
        let data = species::lookup_key(key).unwrap_or_else(|| panic!("{key} missing"));
        assert_eq!(data.standard_phase, species::Phase::Solid, "{key}");
        assert!(data.density > 0.0, "{key} density");
        // Withheld from the aqueous route until the competing-phase review
        // lifts the gate; the dry thermal route does not read it.
        assert_eq!(data.forms_only_above_k, Some(400.0), "{key}");
    }
}
