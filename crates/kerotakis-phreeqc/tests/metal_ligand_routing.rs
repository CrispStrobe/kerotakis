//! Metal-ligand route selection from database-derived ligand coverage.
//!
//! The coverage table is generated from the routed databases; these tests
//! pin it (stale check), its reviewed known gaps, the pure selection rule,
//! and, with the native engine, finite positive and negative controls.
//! No constant is asserted that the databases do not themselves state.
use kerotakis_phreeqc::ligand_routing::{
    self, coverage, present_pairs, routes_defining, select, LigandRoute,
    KNOWN_FAMILIES_WITHOUT_ROUTED_DATA,
};

const DOC: &str = "../../docs/metal-ligand-routing-coverage-20261027.md";
const FIXTURE: &str = "tests/fixtures/metal_ligand_coverage.json";

fn els(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn pair(metal: &str, ligand: &str) -> (String, String) {
    (metal.to_string(), ligand.to_string())
}

#[test]
fn checked_in_coverage_table_is_current() {
    let root = env!("CARGO_MANIFEST_DIR");
    let doc_path = format!("{root}/{DOC}");
    let fixture_path = format!("{root}/{FIXTURE}");
    let markdown = ligand_routing::coverage_markdown();
    let json = ligand_routing::coverage_json();
    if std::env::var("KEROTAKIS_BLESS_GOLDEN").as_deref() == Ok("1") {
        std::fs::write(&doc_path, &markdown).unwrap();
        std::fs::write(&fixture_path, &json).unwrap();
        return;
    }
    let stale = |path: &str, generated: &str| {
        let on_disk = std::fs::read_to_string(path).unwrap_or_default();
        if on_disk != generated {
            eprintln!("--- regenerated {path} ---\n{generated}");
            panic!(
                "{path} is stale; regenerate with KEROTAKIS_BLESS_GOLDEN=1 cargo test -p kerotakis-phreeqc --test metal_ligand_routing"
            );
        }
    };
    stale(&doc_path, &markdown);
    stale(&fixture_path, &json);
}

#[test]
fn known_gaps_are_still_undefined_on_every_routed_database() {
    // When a database gains one of these families this fails, and the
    // entry must leave the reviewed list so the family routes instead.
    for known in KNOWN_FAMILIES_WITHOUT_ROUTED_DATA {
        assert!(
            routes_defining(known.metal, known.ligand).is_empty(),
            "{}/{} is now defined; remove it from the known gaps",
            known.metal,
            known.ligand
        );
        assert!(!known.source.is_empty());
    }
}

#[test]
fn enumerated_families_match_the_database_text() {
    // Silver ammines exist only in minteq.v4 (NIST46.4 rows); the copper
    // ammines are on both ligand routes through the shared reviewed slice.
    assert_eq!(routes_defining("Ag", "N(-3)"), vec!["minteq.v4"]);
    let cu = routes_defining("Cu", "N(-3)");
    assert!(cu.contains(&"wateq4f") && cu.contains(&"minteq.v4"));
    assert_eq!(routes_defining("Ni", "N(-3)"), vec!["minteq.v4"]);
    // A ternary ferrocyanide with an ammonium counter-ion is not an iron
    // ammine, and must not be counted as one.
    assert!(routes_defining("Fe", "N(-3)").is_empty());
    // Hydrolysis is enumerated as its own background family.
    assert!(!routes_defining("Fe", ligand_routing::HYDROXIDE).is_empty());
    assert!(coverage()
        .keys()
        .all(|(metal, _)| ligand_routing::is_metal(metal)));
}

#[test]
fn ammines_route_to_the_database_that_defines_them() {
    for metal in ["Ni", "Co", "Hg", "Ag"] {
        assert_eq!(
            select(&els(&[metal, "N(-3)", "Cl"])),
            LigandRoute::Route {
                db: "minteq.v4",
                pairs: vec![pair(metal, "N(-3)")]
            },
            "{metal}"
        );
    }
    // Copper ammines stay on the default route that already defines them.
    assert_eq!(
        select(&els(&["Cu(2)", "N(-3)", "S(6)"])),
        LigandRoute::Route {
            db: "wateq4f",
            pairs: vec![pair("Cu", "N(-3)")]
        }
    );
    // ...until another element needs minteq.v4 as well.
    assert!(matches!(
        select(&els(&["Cu(2)", "N(-3)", "Acetate"])),
        LigandRoute::Route {
            db: "minteq.v4",
            ..
        }
    ));
}

#[test]
fn known_families_without_data_are_refused_not_ignored() {
    for (metal, ligand) in [("Zn", "N(-3)"), ("Ag", "Thiosulfate"), ("Cd", "N(-3)")] {
        assert_eq!(
            select(&els(&[metal, ligand, "Na"])),
            LigandRoute::Undefined {
                pairs: vec![pair(metal, ligand)]
            }
        );
    }
    // Boundary: silver ammines are defined, silver thiosulfate is not; the
    // refusal names only the undefined family.
    assert_eq!(
        select(&els(&["Ag", "N(-3)", "Thiosulfate", "Na"])),
        LigandRoute::Undefined {
            pairs: vec![pair("Ag", "Thiosulfate")]
        }
    );
}

#[test]
fn background_pairs_and_other_redox_states_do_not_route() {
    // Nitrate is not ammonia; sulfate and chloride are background ligands.
    for elements in [
        &["Ag", "N(5)"][..],
        &["Zn", "N(5)", "S(6)"],
        &["Cu(2)", "S(6)", "Cl"],
        // s-block ammine ion pairs are reported, not route-deciding.
        &["Ca", "N(-3)", "Cl"],
        // No iron ammine is defined and none is a reviewed known gap.
        &["Fe(2)", "N(-3)", "S(6)"],
        // Ammonia alone, metal alone.
        &["N(-3)", "Cl"],
        &["Ni", "Cl"],
    ] {
        assert_eq!(
            select(&els(elements)),
            LigandRoute::Unconstrained,
            "{elements:?}"
        );
        assert!(present_pairs(&els(elements)).is_empty());
    }
    assert!(!LigandRoute::Unconstrained.blocks("wateq4f"));
    assert!(LigandRoute::Undefined { pairs: vec![] }.blocks("minteq.v4"));
}

#[cfg(feature = "engine")]
mod native {
    use kerotakis_core::{
        ledger::ConservedLedger, Equilibrator, Event, Kelvin, Moles, SpeciesId, ThermalMode,
        Vessel, VesselId,
    };
    use kerotakis_phreeqc::PhreeqcEquilibrator;

    fn water() -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "finite metal/ligand control");
        vessel.deposit(
            SpeciesId::new("water"),
            Moles(5.55),
            kerotakis_core::Phase::Liquid,
        );
        vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        vessel
    }

    fn deposit(vessel: &mut Vessel, key: &str, moles: f64) {
        let data = kerotakis_core::species::lookup_key(key).expect("registered reagent");
        vessel.deposit(SpeciesId::new(key), Moles(moles), data.standard_phase);
    }

    fn reason_keys(events: &[Event]) -> Vec<String> {
        events
            .iter()
            .filter_map(|event| {
                serde_json::to_value(event).unwrap()["reason"]["key"]
                    .as_str()
                    .map(str::to_string)
            })
            .collect()
    }

    fn provenance(vessel: &Vessel) -> (String, String) {
        let p = vessel
            .solution
            .as_ref()
            .unwrap()
            .provenance
            .as_ref()
            .unwrap();
        (p.dataset.clone(), p.routing.clone())
    }

    fn molality(vessel: &Vessel, name: &str) -> f64 {
        vessel
            .solution
            .as_ref()
            .unwrap()
            .species
            .iter()
            .find(|species| species.name == name)
            .map_or(0.0, |species| species.molality)
    }

    fn assert_conserved(before: &Vessel, after: &Vessel) {
        let before = ConservedLedger::from_vessel(before);
        let after = ConservedLedger::from_vessel(after);
        assert!((before.charge - after.charge).abs() < 1e-9);
        for (element, amount) in &before.elements {
            let actual = after.elements.get(element).copied().unwrap_or(0.0);
            assert!(
                (actual - amount).abs() < 1e-9 + amount.abs() * 1e-6,
                "{element}: {amount} -> {actual}"
            );
        }
    }

    #[test]
    fn copper_ammines_keep_the_default_route_and_gain_the_ledger_note() {
        let mut solver = PhreeqcEquilibrator::new().unwrap();
        let mut vessel = water();
        deposit(&mut vessel, "CuSO4", 0.001);
        deposit(&mut vessel, "NH3", 0.006);
        let before = vessel.clone();
        let events = solver.equilibrate(&mut vessel).unwrap();
        let (dataset, routing) = provenance(&vessel);
        assert!(dataset.contains("wateq4f.dat"), "{dataset}");
        assert!(!routing.contains("metal-ligand complexes"), "{routing}");
        assert!(molality(&vessel, "Cu(NH3)4+2") > 0.0);
        assert!(reason_keys(&events)
            .iter()
            .any(|k| k == "not-modeled.metal-ammine-ligand-inventory"));
        assert_conserved(&before, &vessel);
    }

    #[test]
    fn silver_thiosulfate_is_refused_with_a_typed_reason_before_mutation() {
        let mut solver = PhreeqcEquilibrator::new().unwrap();
        let mut vessel = water();
        deposit(&mut vessel, "AgNO3", 0.001);
        deposit(&mut vessel, "Na2S2O3", 0.002);
        let before = serde_json::to_value(&vessel).unwrap();
        let error = solver.equilibrate(&mut vessel).unwrap_err();
        assert_eq!(
            error.owned_reason().map(|p| p.key.as_str()),
            Some("solver-reason.aqueous.metal-ligand-complexes-undefined"),
            "{error}"
        );
        assert!(error.to_string().contains("Ag/Thiosulfate"), "{error}");
        assert_eq!(before, serde_json::to_value(&vessel).unwrap());
    }

    #[test]
    fn zinc_ammine_is_refused_whether_ammonia_or_ammonium_supplies_it() {
        let mut solver = PhreeqcEquilibrator::new().unwrap();
        for ligand in ["NH3", "NH4Cl"] {
            let mut vessel = water();
            deposit(&mut vessel, "ZnSO4", 0.001);
            deposit(&mut vessel, ligand, 0.004);
            let error = solver.equilibrate(&mut vessel).unwrap_err();
            assert_eq!(
                error.owned_reason().map(|p| p.key.as_str()),
                Some("solver-reason.aqueous.metal-ligand-complexes-undefined"),
                "{ligand}: {error}"
            );
        }
    }

    #[test]
    fn metals_without_a_route_deciding_ligand_are_unchanged() {
        let mut solver = PhreeqcEquilibrator::new().unwrap();
        for (metal, other) in [("ZnSO4", "NH4NO3"), ("FeSO4", "NH4Cl"), ("AgNO3", "AgNO3")] {
            let mut vessel = water();
            deposit(&mut vessel, metal, 0.001);
            if metal != other {
                deposit(&mut vessel, other, 0.001);
            }
            let result = solver.equilibrate(&mut vessel);
            if metal == "ZnSO4" {
                // Ammonium nitrate still carries N(-3): zinc ammines are a
                // known family, so this is the refusal boundary, not a pass.
                assert!(result.is_err());
                continue;
            }
            result.unwrap();
            let (dataset, routing) = provenance(&vessel);
            assert!(dataset.contains("wateq4f.dat"), "{metal}: {dataset}");
            assert!(!routing.contains("metal-ligand complexes"), "{routing}");
        }
    }

    #[test]
    fn ferric_oxalate_stays_an_unspeciated_acid_boundary_not_a_route() {
        // No routed database has an oxalate master species, so oxalic acid
        // never becomes a ligand element: the existing unspeciated-acid
        // boundary owns this vessel and no ligand route is claimed.
        let mut solver = PhreeqcEquilibrator::new().unwrap();
        let mut vessel = water();
        deposit(&mut vessel, "FeCl3", 0.001);
        deposit(&mut vessel, "H2C2O4", 0.003);
        if solver.equilibrate(&mut vessel).is_ok() {
            if let Some(solution) = vessel.solution.as_ref() {
                if let Some(p) = solution.provenance.as_ref() {
                    assert!(!p.routing.contains("metal-ligand complexes"));
                }
            }
        }
    }
}
