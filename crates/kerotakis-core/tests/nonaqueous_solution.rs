//! SP-011 (2026-11-11): a solute dissolved in a water-free liquid (hexane
//! after an extraction, any other organic solvent) is not "aqueous". The
//! snapshot labels it `nonaqueous_solution` with its solvent, and the
//! iodine inventory behind `inspect` counts I2 held in that liquid as
//! oxidised iodine. Generic over the solvent; water-bearing vessels are
//! unchanged.
//!
//! `nonaqueous_solution_coverage_is_current` pins the label and pool name to
//! `docs/nonaqueous-solution-coverage-20261111.json`
//! (`python3 tools/nonaqueous_solution_coverage.py` regenerates it).

use kerotakis_core::iodine_chemistry as iodine;
use kerotakis_core::vessel::{nonaqueous_solvent, NONAQUEOUS_SOLUTION};
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/nonaqueous-solution-coverage-20261111.json");

fn vessel_with(liquid: &str, liquid_mol: f64, i2: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(3), "beaker");
    v.deposit(SpeciesId::new(liquid), Moles(liquid_mol), Phase::Liquid);
    v.deposit(SpeciesId::new("I2"), Moles(i2), Phase::Aqueous);
    v
}

fn portion<'a>(json: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    json["contents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["species"] == key)
        .unwrap_or_else(|| panic!("{key} missing in {json}"))
}

#[test]
fn nonaqueous_solution_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    assert_eq!(doc["snapshot_label"].as_str().unwrap(), NONAQUEOUS_SOLUTION);
    assert_eq!(doc["iodine_pool"].as_str().unwrap(), "nonaqueous_i2_mol");
}

#[test]
fn solute_in_a_water_free_liquid_is_not_tagged_aqueous() {
    for requested in ["hexane", "cyclohexane", "toluene", "CCl4", "ethanol"] {
        let v = vessel_with(requested, 0.1, 5.8922465400127e-5);
        // The registry may canonicalise the input key; use what it stored.
        let solvent = v.contents[0].species.0.clone();
        assert_eq!(
            nonaqueous_solvent(&v.contents).map(|s| s.0.as_str()),
            Some(solvent.as_str())
        );
        let json = serde_json::to_value(&v).unwrap();
        let i2 = portion(&json, "I2");
        assert_eq!(i2["phase"], NONAQUEOUS_SOLUTION, "{solvent}: {json}");
        assert_eq!(i2["solvent"], solvent.as_str());
        assert_ne!(i2["phase"], "aqueous");
        // The solvent itself stays a liquid.
        assert_eq!(portion(&json, &solvent)["phase"], "liquid");
    }
}

#[test]
fn solute_in_water_stays_aqueous_and_carries_no_solvent_field() {
    let mut v = vessel_with("water", 5.5, 1e-4);
    // Water plus a second liquid is still an aqueous solution here.
    v.deposit(SpeciesId::new("hexane"), Moles(10.0), Phase::Liquid);
    assert!(nonaqueous_solvent(&v.contents).is_none());
    let json = serde_json::to_value(&v).unwrap();
    assert_eq!(portion(&json, "I2")["phase"], "aqueous");
    assert!(portion(&json, "I2").get("solvent").is_none());
}

#[test]
fn a_vessel_without_any_liquid_names_no_solvent() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("I2"), Moles(1e-4), Phase::Solid);
    assert!(nonaqueous_solvent(&v.contents).is_none());
}

#[test]
fn the_nonaqueous_label_reads_back_as_the_same_vessel() {
    let v = vessel_with("hexane", 0.1, 5e-5);
    let text = serde_json::to_string(&v).unwrap();
    assert!(text.contains(NONAQUEOUS_SOLUTION));
    let back: Vessel = serde_json::from_str(&text).unwrap();
    assert_eq!(back.contents, v.contents);
}

#[test]
fn iodine_held_in_hexane_counts_as_oxidised_iodine() {
    let v = vessel_with("hexane", 0.1, 5.8922465400127e-5);
    let inv = iodine::iodine_inventory(&v);
    assert_eq!(inv.nonaqueous_i2_mol, 5.8922465400127e-5);
    assert_eq!(inv.nonaqueous_solvent.as_deref(), Some("hexane"));
    assert_eq!(inv.oxidised_iodine_mol, 5.8922465400127e-5);
    assert!(inv.other_pools.is_empty(), "{inv:?}");
    assert!(inv.closure_residual_mol.abs() <= 1e-18, "{inv:?}");
    assert_eq!(inv.free_i2_mol, 0.0);
    assert_eq!(inv.triiodide_mol, 0.0);
}

#[test]
fn water_and_solid_iodine_do_not_enter_the_nonaqueous_pool() {
    // Aqueous I2: the existing free/I3- split, nothing non-aqueous.
    let w = vessel_with("water", 5.5, 1e-4);
    let inv = iodine::iodine_inventory(&w);
    assert_eq!(inv.nonaqueous_i2_mol, 0.0);
    assert!(inv.nonaqueous_solvent.is_none());
    assert!((inv.oxidised_iodine_mol - 1e-4).abs() <= 1e-15, "{inv:?}");
    // Solid I2 beside hexane is a solid pool, not dissolved iodine.
    let mut s = Vessel::new(VesselId(1), "beaker");
    s.deposit(SpeciesId::new("hexane"), Moles(0.1), Phase::Liquid);
    s.deposit(SpeciesId::new("I2"), Moles(2e-4), Phase::Solid);
    let inv = iodine::iodine_inventory(&s);
    assert_eq!(inv.nonaqueous_i2_mol, 0.0);
    assert_eq!(inv.oxidised_iodine_mol, 0.0);
    assert_eq!(inv.other_pools.len(), 1);
    assert!(inv.closure_residual_mol.abs() <= 1e-18, "{inv:?}");
}

/// The SP-011 script shape end to end: the control receiver's hexane holds
/// the extracted I2 as non-aqueous oxidised iodine, the receiver from the
/// thiosulfate-reduced feed holds none.
#[test]
fn sp011_receivers_report_oxidised_iodine_in_hexane() {
    let mut bench = Bench::new();
    for line in [
        "add v1 water 100mL",
        "add v1 KI 0.001mol",
        "add v1 I2 0.0001mol",
        "add v1 Na2S2O3 0.0002mol",
        "new",
        "add v2 water 100mL",
        "add v2 KI 0.001mol",
        "add v2 I2 0.0001mol",
        "new",
        "new",
        "extract v1 v3 hexane 0.1mol stages 1",
        "extract v2 v4 hexane 0.1mol stages 1",
    ] {
        bench
            .step(script::parse_op(line).unwrap().expect("operator"))
            .unwrap();
    }
    let reduced = iodine::iodine_inventory(bench.vessel(VesselId(2)).unwrap());
    let control_vessel = bench.vessel(VesselId(3)).unwrap();
    let control = iodine::iodine_inventory(control_vessel);
    assert_eq!(reduced.oxidised_iodine_mol, 0.0, "{reduced:?}");
    assert!(control.oxidised_iodine_mol > 0.0, "{control:?}");
    assert_eq!(control.oxidised_iodine_mol, control.nonaqueous_i2_mol);
    assert_eq!(control.nonaqueous_solvent.as_deref(), Some("hexane"));
    assert!(control.other_pools.is_empty(), "{control:?}");
    let json = serde_json::to_value(control_vessel).unwrap();
    assert_eq!(portion(&json, "I2")["phase"], NONAQUEOUS_SOLUTION);
}
