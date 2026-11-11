//! AN-004 (2026-11-11): the iodine inventory behind `inspect`'s
//! `iodine_speciation`. Every iodine pool of a vessel is named — free I2,
//! I3⁻, free I⁻, the starch-bound complex and any other iodine-bearing
//! portion — and the pools close against the ledger's iodine atoms. A
//! vessel whose iodine is all starch-bound is still reported.
//!
//! `iodine_inventory_coverage_is_current` pins the runtime constants to
//! `docs/iodine-inventory-coverage-20261111.json`
//! (`python3 tools/iodine_inventory_coverage.py` regenerates it).

use kerotakis_core::iodine_chemistry as iodine;
use kerotakis_core::*;

const TABLE: &str = include_str!("../../../docs/iodine-inventory-coverage-20261111.json");

/// 0.10 g starch as anhydroglucose units (M = 162.14 g/mol, registry).
const STARCH_MOL: f64 = 0.10 / 162.14;

fn water(ml: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(ml / 18.015), Phase::Liquid);
    v
}

fn put(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}

fn assert_closes(inv: &iodine::IodineInventory) {
    let scale = inv.ledger_iodine_atoms_mol.abs().max(1e-30);
    assert!(
        inv.closure_residual_mol.abs() <= 1e-12 * scale,
        "iodine closure: {inv:?}"
    );
    assert!(inv.unresolved_species.is_empty(), "{inv:?}");
}

#[test]
fn iodine_inventory_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(TABLE).unwrap();
    let c = &doc["constants"];
    assert_eq!(c["triiodide_k"].as_f64().unwrap(), iodine::TRIIODIDE_K);
    assert_eq!(
        c["starch_site_fraction"].as_f64().unwrap(),
        iodine::STARCH_SITE_FRACTION
    );
    assert_eq!(
        doc["starch_binding_limitation"].as_str().unwrap(),
        iodine::STARCH_BINDING_LIMITATION
    );
}

/// The AN-004 treatment vessel after the second iodine addition: 4e-5 mol
/// I2(aq), 1.6e-4 mol I⁻ and 0.10 g starch in 40 mL. The starch-bound
/// share used to vanish from inspect (77 % of the oxidised iodine).
#[test]
fn starch_bound_iodine_is_named_and_the_inventory_closes() {
    let mut v = water(40.0);
    put(&mut v, "starch", STARCH_MOL, Phase::Solid);
    put(&mut v, "I2", 4e-5, Phase::Aqueous);
    put(&mut v, "I-", 1.6e-4, Phase::Aqueous);
    put(&mut v, "K+", 1e-4, Phase::Aqueous);
    let inv = iodine::iodine_inventory(&v);
    assert_closes(&inv);
    let bound = STARCH_MOL * iodine::STARCH_SITE_FRACTION;
    assert!(
        (inv.starch_bound_triiodide_mol - bound).abs() < 1e-18,
        "{inv:?}"
    );
    // Every oxidised iodine is accounted: free I2 + I3⁻ + starch-bound.
    assert!((inv.oxidised_iodine_mol - 4e-5).abs() < 1e-18, "{inv:?}");
    assert!((inv.ledger_iodine_atoms_mol - (2.0 * 4e-5 + 1.6e-4)).abs() < 1e-18);
    assert!(inv.other_pools.is_empty(), "{inv:?}");
    assert_eq!(inv.limitations.len(), 1);
    assert_eq!(inv.limitations[0].key, iodine::STARCH_BINDING_LIMITATION);
}

/// The AN-004 starch control (v1): 2e-5 mol I2 is entirely starch-bound,
/// so the optical split is empty — and the vessel used to be dropped from
/// `iodine_speciation`. The inventory still names it.
#[test]
fn all_starch_bound_control_vessel_is_still_reported() {
    let mut v = water(40.0);
    put(&mut v, "starch", STARCH_MOL, Phase::Solid);
    put(&mut v, "I2", 2e-5, Phase::Aqueous);
    put(&mut v, "I-", 1e-4, Phase::Aqueous);
    put(&mut v, "K+", 1e-4, Phase::Aqueous);
    assert!(iodine::optical_split(&v).is_none());
    let inv = iodine::iodine_inventory(&v);
    assert!(iodine::holds_iodine(&inv));
    assert_closes(&inv);
    assert!(
        (inv.starch_bound_triiodide_mol - 2e-5).abs() < 1e-18,
        "{inv:?}"
    );
    assert!((inv.oxidised_iodine_mol - 2e-5).abs() < 1e-18, "{inv:?}");
    assert_eq!(inv.free_i2_mol, 0.0);
    assert_eq!(inv.triiodide_mol, 0.0);
}

/// Negative: no starch, no starch pool and no starch limitation; the
/// triiodide split alone closes the ledger.
#[test]
fn without_starch_there_is_no_bound_pool_or_limitation() {
    let mut v = water(40.0);
    put(&mut v, "I2", 4e-5, Phase::Aqueous);
    put(&mut v, "KI", 1.6e-4, Phase::Aqueous);
    let inv = iodine::iodine_inventory(&v);
    assert_closes(&inv);
    assert_eq!(inv.starch_bound_triiodide_mol, 0.0);
    assert!(inv.limitations.is_empty(), "{inv:?}");
    assert!(inv.triiodide_mol > 0.0);
    assert!((inv.free_i2_mol + inv.triiodide_mol - 4e-5).abs() < 1e-18);
    assert!((inv.free_iodide_mol + inv.triiodide_mol - 1.6e-4).abs() < 1e-18);
}

/// Iodine outside the aqueous split — undissolved I2, a silver iodide
/// precipitate, iodate — is its own named pool, counted by formula atoms.
#[test]
fn other_iodine_pools_are_listed_and_counted_by_atoms() {
    let mut v = water(40.0);
    put(&mut v, "I2", 1e-5, Phase::Aqueous);
    put(&mut v, "I-", 2e-5, Phase::Aqueous);
    put(&mut v, "I2", 3e-4, Phase::Solid);
    put(&mut v, "AgI", 5e-5, Phase::Solid);
    put(&mut v, "KIO3", 7e-6, Phase::Aqueous);
    let inv = iodine::iodine_inventory(&v);
    assert_closes(&inv);
    let atoms = |key: &str, phase: Phase| {
        inv.other_pools
            .iter()
            .find(|p| p.species == key && p.phase == phase)
            .map(|p| p.iodine_atoms_mol)
    };
    assert_eq!(atoms("I2", Phase::Solid), Some(6e-4));
    assert_eq!(atoms("AgI", Phase::Solid), Some(5e-5));
    assert_eq!(atoms("KIO3", Phase::Aqueous), Some(7e-6));
    assert_eq!(inv.other_pools.len(), 3, "{inv:?}");
}

/// Negative: a blank holds no iodine, so it is an all-zero inventory that
/// does not by itself switch the inspect block on.
#[test]
fn a_vessel_without_iodine_is_an_empty_inventory() {
    let mut v = water(40.0);
    put(&mut v, "starch", STARCH_MOL, Phase::Solid);
    put(&mut v, "NaCl", 1e-3, Phase::Aqueous);
    let inv = iodine::iodine_inventory(&v);
    assert!(!iodine::holds_iodine(&inv));
    assert_eq!(inv.ledger_iodine_atoms_mol, 0.0);
    assert_eq!(inv.pooled_iodine_atoms_mol, 0.0);
    assert!(inv.limitations.is_empty());
}

/// Without liquid water nothing is dissolved: dry iodine and iodide are
/// reported as other pools, never as a triiodide split.
#[test]
fn dry_iodine_is_an_other_pool_not_a_split() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "I2", 1e-4, Phase::Solid);
    put(&mut v, "KI", 2e-4, Phase::Solid);
    let inv = iodine::iodine_inventory(&v);
    assert_closes(&inv);
    assert_eq!(inv.oxidised_iodine_mol, 0.0);
    assert_eq!(inv.other_pools.len(), 2, "{inv:?}");
}
