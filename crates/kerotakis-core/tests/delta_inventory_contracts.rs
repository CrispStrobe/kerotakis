//! Independent ordered inventory and rollback contracts for transactional deltas.
use kerotakis_core::compartment::{ElectrodeDeposit, ElectrodeState};
use kerotakis_core::{
    delta::{ElectrodeInventory, StateDelta},
    vessel::{AdsorbedAmount, Portion},
    *,
};

#[derive(Clone, Copy)]
enum Owner {
    Bulk,
    Adsorbed,
    Substrate,
    Deposit,
}
fn electrode() -> ElectrodeState {
    ElectrodeState {
        label: "plate".into(),
        material: "Zn".into(),
        surface_preparation: None,
        substrate_moles: Some(1.0),
        area_m2: 0.001,
        roughness: 1.0,
        double_layer_capacitance_f_per_m2: None,
        interfacial_potential_v: None,
        interfacial_species: vec![],
        diagnostics: None,
        deposits: vec![],
    }
}
fn deposit(species: &str, moles: f64) -> ElectrodeDeposit {
    ElectrodeDeposit {
        species: species.into(),
        moles,
        thickness_m: None,
        coverage_fraction: None,
        effect: None,
        electrical_resistivity_ohm_m: None,
    }
}
fn vessel(owner: Owner, amount: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "inventory");
    match owner {
        Owner::Bulk => v.contents.push(Portion {
            species: SpeciesId::new("Zn"),
            phase: Phase::Solid,
            moles: Moles(amount),
        }),
        Owner::Adsorbed => v.adsorbed.push(AdsorbedAmount {
            sorbent: SpeciesId::new("activated_charcoal"),
            sorbate: SpeciesId::new("Zn"),
            moles: Moles(amount),
        }),
        Owner::Substrate => {
            let mut e = electrode();
            e.substrate_moles = Some(amount);
            v.electrodes.push(e);
        }
        Owner::Deposit => {
            let mut e = electrode();
            e.deposits.push(deposit("Zn", amount));
            v.electrodes.push(e);
        }
    }
    v
}
fn delta(owner: Owner, changes: &[f64]) -> StateDelta {
    let mut d = StateDelta::new("independent-inventory-contract");
    for &n in changes {
        d = match owner {
            Owner::Bulk => d.with_moles(SpeciesId::new("Zn"), Phase::Solid, n),
            Owner::Adsorbed => d.with_adsorbed(
                SpeciesId::new("activated_charcoal"),
                SpeciesId::new("Zn"),
                n,
            ),
            Owner::Substrate => d.with_electrode_moles("plate", ElectrodeInventory::Substrate, n),
            Owner::Deposit => d.with_electrode_moles(
                "plate",
                ElectrodeInventory::Deposit {
                    species: SpeciesId::new("Zn"),
                    growth: None,
                    effect: None,
                },
                n,
            ),
        };
    }
    d
}
fn amount(v: &Vessel, owner: Owner) -> f64 {
    match owner {
        Owner::Bulk => v.moles_of(&SpeciesId::new("Zn")).0,
        Owner::Adsorbed => v.adsorbed.iter().map(|x| x.moles.0).sum(),
        Owner::Substrate => v.electrodes[0].substrate_moles.unwrap(),
        Owner::Deposit => v.electrodes[0]
            .deposits
            .iter()
            .filter(|d| d.species == "Zn")
            .map(|d| d.moles)
            .sum(),
    }
}
fn unchanged_refusal(owner: Owner, initial: f64, changes: &[f64]) {
    let mut v = vessel(owner, initial);
    let before = format!("{v:?}");
    assert!(delta(owner, changes).commit(&mut v).is_err());
    assert_eq!(format!("{v:?}"), before, "all state must survive refusal");
}
macro_rules! owner_contracts {
    ($prefix:ident,$reverse:ident,$overflow:ident,$malformed:ident,$tiny:ident,$exhaustion:ident,$limited:ident,$owner:expr) => {
        #[test]
        fn $prefix() {
            unchanged_refusal($owner, 5.0, &[-6.0, 6.0]);
        }
        #[test]
        fn $reverse() {
            let mut v = vessel($owner, 5.0);
            delta($owner, &[6.0, -6.0]).commit(&mut v).unwrap();
            assert_eq!(amount(&v, $owner), 5.0);
        }
        #[test]
        fn $overflow() {
            unchanged_refusal($owner, f64::MAX, &[f64::MAX]);
        }
        #[test]
        fn $malformed() {
            for n in [f64::NAN, f64::INFINITY, -1.0] {
                unchanged_refusal($owner, n, &[0.25]);
            }
        }
        #[test]
        fn $tiny() {
            unchanged_refusal($owner, 1e-18, &[-2e-18]);
        }
        #[test]
        fn $exhaustion() {
            let mut v = vessel($owner, 1.0);
            delta($owner, &[-1.0, 0.5]).commit(&mut v).unwrap();
            assert_eq!(amount(&v, $owner), 0.5);
        }
        #[test]
        fn $limited() {
            let mut v = vessel($owner, 5.0);
            let limited = delta($owner, &[-6.0, 6.0]).inventory_limited(&v).unwrap();
            assert!((limited.accepted_fraction - 5.0 / 6.0).abs() < 1e-15);
            limited.delta.commit(&mut v).unwrap();
            assert_eq!(amount(&v, $owner), 5.0);
        }
    };
}
owner_contracts!(
    bulk_prefix,
    bulk_reverse,
    bulk_overflow,
    bulk_malformed,
    bulk_tiny_overdraw,
    bulk_exhaustion,
    bulk_limited,
    Owner::Bulk
);
owner_contracts!(
    adsorbed_prefix,
    adsorbed_reverse,
    adsorbed_overflow,
    adsorbed_malformed,
    adsorbed_tiny_overdraw,
    adsorbed_exhaustion,
    adsorbed_limited,
    Owner::Adsorbed
);
owner_contracts!(
    substrate_prefix,
    substrate_reverse,
    substrate_overflow,
    substrate_malformed,
    substrate_tiny_overdraw,
    substrate_exhaustion,
    substrate_limited,
    Owner::Substrate
);
owner_contracts!(
    deposit_prefix,
    deposit_reverse,
    deposit_overflow,
    deposit_malformed,
    deposit_tiny_overdraw,
    deposit_exhaustion,
    deposit_limited,
    Owner::Deposit
);

#[test]
fn bulk_withdrawal_preserves_unrelated_trace_and_partial_remainder() {
    for n in [1e-12, 1e-15, 5e-16, 1e-18, f64::from_bits(1)] {
        let mut v = vessel(Owner::Bulk, 1.0);
        v.deposit(SpeciesId::new("Cu"), Moles(n), Phase::Solid);
        delta(Owner::Bulk, &[-0.5]).commit(&mut v).unwrap();
        assert_eq!(v.moles_of(&SpeciesId::new("Cu")).0, n);
    }
    let mut v = vessel(Owner::Bulk, 5e-16);
    delta(Owner::Bulk, &[-2e-16]).commit(&mut v).unwrap();
    assert_eq!(amount(&v, Owner::Bulk), 5e-16 - 2e-16);
}
#[test]
fn electrode_changes_preserve_unrelated_trace_layers() {
    for owner in [Owner::Substrate, Owner::Deposit] {
        for n in [1e-12, 1e-15, 5e-16, 1e-18, f64::from_bits(1)] {
            let mut v = vessel(owner, 1.0);
            v.electrodes[0].deposits.push(deposit("Cu", n));
            delta(owner, &[-0.5]).commit(&mut v).unwrap();
            assert_eq!(
                v.electrodes[0]
                    .deposits
                    .iter()
                    .find(|d| d.species == "Cu")
                    .map(|d| d.moles),
                Some(n)
            );
        }
    }
}
#[test]
fn electrode_zero_change_preserves_positive_trace_layers() {
    let mut v = vessel(Owner::Substrate, 1.0);
    v.electrodes[0].deposits.push(deposit("Cu", 5e-16));
    let before = format!("{v:?}");
    delta(Owner::Substrate, &[0.0]).commit(&mut v).unwrap();
    assert_eq!(format!("{v:?}"), before);
}
#[test]
fn electrode_partial_trace_layer_is_not_deleted() {
    let mut v = vessel(Owner::Deposit, 5e-16);
    delta(Owner::Deposit, &[-2e-16]).commit(&mut v).unwrap();
    assert_eq!(amount(&v, Owner::Deposit), 5e-16 - 2e-16);
}
#[test]
fn separate_bulk_trace_portion_survives_exhaustion_of_first() {
    let mut v = vessel(Owner::Bulk, 1.0);
    v.contents.push(Portion {
        species: SpeciesId::new("Zn"),
        phase: Phase::Solid,
        moles: Moles(1e-18),
    });
    delta(Owner::Bulk, &[-1.0]).commit(&mut v).unwrap();
    assert_eq!(amount(&v, Owner::Bulk), 1e-18);
}
#[test]
fn ambiguous_touched_electrode_deposits_refuse_without_mutation() {
    let mut v = vessel(Owner::Deposit, 1.0);
    v.electrodes[0].deposits.push(deposit("Zn", 1.0));
    let before = format!("{v:?}");
    assert!(delta(Owner::Deposit, &[-1.5]).commit(&mut v).is_err());
    assert_eq!(format!("{v:?}"), before);
}

#[test]
fn one_ulp_exhaustion_roundoff_is_bounded_and_nonnegative() {
    for owner in [
        Owner::Bulk,
        Owner::Adsorbed,
        Owner::Substrate,
        Owner::Deposit,
    ] {
        let mut v = vessel(owner, 1.0);
        delta(owner, &[-(1.0 + f64::EPSILON)])
            .commit(&mut v)
            .unwrap();
        assert_eq!(amount(&v, owner), 0.0);
    }
}
#[test]
fn sixty_four_ulp_overdraw_is_outside_the_arithmetic_budget() {
    for owner in [
        Owner::Bulk,
        Owner::Adsorbed,
        Owner::Substrate,
        Owner::Deposit,
    ] {
        unchanged_refusal(owner, 1.0, &[-(1.0 + 64.0 * f64::EPSILON)]);
    }
}
#[test]
fn nonfinite_requests_refuse_for_every_owner() {
    for owner in [
        Owner::Bulk,
        Owner::Adsorbed,
        Owner::Substrate,
        Owner::Deposit,
    ] {
        for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            unchanged_refusal(owner, 1.0, &[n]);
        }
    }
}
