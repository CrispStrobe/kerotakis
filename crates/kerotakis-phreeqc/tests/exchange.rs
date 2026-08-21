//! Live finite-capacity batch softening through PHREEQC `EXCHANGE`.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn sodium_resin() -> ExchangeSites {
    ExchangeSites {
        label: "sodium softener resin".to_string(),
        dry_mass: Grams(2.0),
        capacity: Moles(2e-3),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(2e-3),
        }],
    }
}

fn step(
    bench: &mut Bench,
    solver: &mut PhreeqcEquilibrator,
    operator: Operator,
) -> Result<Vec<Event>, BenchError> {
    bench.step_with(operator, solver, &PermissiveScreen)
}

fn add(
    bench: &mut Bench,
    solver: &mut PhreeqcEquilibrator,
    species: &str,
    moles: f64,
) -> Result<Vec<Event>, BenchError> {
    step(
        bench,
        solver,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(species),
            moles: Moles(moles),
            at: None,
        },
    )
}

fn bound(vessel: &Vessel, ion: ExchangeIon) -> f64 {
    vessel
        .exchanges
        .iter()
        .map(|exchange| exchange.bound(ion).0)
        .sum()
}

fn inventory(vessel: &Vessel, ion: ExchangeIon) -> f64 {
    vessel.moles_of(&ion.species()).0 + bound(vessel, ion)
}

fn prepare_batch(with_resin: bool) -> (Bench, PhreeqcEquilibrator) {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    if with_resin {
        bench.vessels[0].exchanges.push(sodium_resin());
    }
    add(&mut bench, &mut solver, "water", 55.51).expect("water");
    add(&mut bench, &mut solver, "CaCl2", 1e-3).expect("calcium hardness");
    add(&mut bench, &mut solver, "MgSO4", 1e-3).expect("magnesium hardness");
    (bench, solver)
}

#[test]
fn sodium_form_resin_softens_a_finite_hard_water_batch() {
    let (control, _) = prepare_batch(false);
    let (softened, _) = prepare_batch(true);
    let control = control.vessel(VesselId(0)).unwrap();
    let softened = softened.vessel(VesselId(0)).unwrap();

    let control_hardness =
        control.moles_of(&SpeciesId::new("Ca+2")).0 + control.moles_of(&SpeciesId::new("Mg+2")).0;
    let softened_hardness =
        softened.moles_of(&SpeciesId::new("Ca+2")).0 + softened.moles_of(&SpeciesId::new("Mg+2")).0;
    assert!(
        softened_hardness < 0.75 * control_hardness,
        "finite sodium resin should measurably soften the batch: control={control_hardness:.12e}, softened={softened_hardness:.12e} mol"
    );
    assert!(bound(softened, ExchangeIon::Calcium) > 0.0);
    assert!(bound(softened, ExchangeIon::Magnesium) > 0.0);

    for (ion, expected) in [
        (ExchangeIon::Sodium, 2e-3),
        (ExchangeIon::Calcium, 1e-3),
        (ExchangeIon::Magnesium, 1e-3),
    ] {
        let actual = inventory(softened, ion);
        assert!(
            (actual - expected).abs() < 2e-8,
            "{ion:?} must be conserved across solution and exchanger: expected={expected:.12e}, actual={actual:.12e} mol"
        );
    }
    assert!(softened.exchanges[0].has_valid_capacity());
}

#[test]
fn repeated_equilibration_preserves_exchange_inventory_and_state() {
    let (mut bench, mut solver) = prepare_batch(true);
    let first = bench.vessel(VesselId(0)).unwrap().clone();

    step(
        &mut bench,
        &mut solver,
        Operator::Stir {
            vessel: VesselId(0),
        },
    )
    .expect("repeat equilibrium");
    let settled = bench.vessel(VesselId(0)).unwrap();

    for ion in [
        ExchangeIon::Sodium,
        ExchangeIon::Calcium,
        ExchangeIon::Magnesium,
    ] {
        assert!((inventory(settled, ion) - inventory(&first, ion)).abs() < 2e-8);
        assert!((bound(settled, ion) - bound(&first, ion)).abs() < 2e-8);
    }
    assert!((settled.mass().0 - first.mass().0).abs() < 2e-5);
    assert!(settled.exchanges[0].has_valid_capacity());
}

#[test]
fn an_untracked_exchangeable_cation_fails_loudly() {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    bench.vessels[0].exchanges.push(sodium_resin());
    add(&mut bench, &mut solver, "water", 55.51).expect("water");

    let error = add(&mut bench, &mut solver, "KCl", 1e-3).expect_err("K can bind X sites");
    let detail = error.to_string();
    assert!(detail.contains("can bind K"), "unexpected error: {detail}");
    assert!(detail.contains("retains only H, Na, Ca and Mg"));
}
