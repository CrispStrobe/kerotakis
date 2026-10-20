//! Transfers must debit the phase that paid for the planned move.
use kerotakis_core::species::Phase;
use kerotakis_core::*;

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-12,
        "actual {actual:.17e}, expected {expected:.17e}"
    );
}
fn amount(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}
fn put(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}
fn wet() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "sample");
    v.headspace = Headspace::Sealed {
        volume: Liters(0.1),
    };
    put(&mut v, "water", 5.0, Phase::Liquid);
    v
}
fn drain(mut v: Vessel) -> Bench {
    put(&mut v, "hexane", 0.5, Phase::Liquid);
    let mut b = Bench::new();
    b.vessels[0] = v;
    b.step_with(
        Operator::Drain {
            from: VesselId(0),
            to: VesselId(1),
        },
        &mut SolverStack::new(Vec::new()),
        &PermissiveScreen,
    )
    .unwrap();
    b
}
fn capacity(v: &Vessel, key: &str) -> f64 {
    let water = species::lookup_key("water").unwrap();
    let solute = species::lookup_key(key).unwrap();
    let ml = water
        .liters_from_moles(Moles(amount(v, "water", Phase::Liquid)))
        .0
        * 1000.0;
    solute.aqueous_solubility_at(v.temperature.0).unwrap() * ml / 100.0 / solute.molar_mass
}
fn dissolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Dissolved { species, moles, .. } if species.0 == key => Some(moles.0),
            Event::Precipitated { species, moles, .. } if species.0 == key => Some(-moles.0),
            _ => None,
        })
        .sum()
}
#[test]
fn drain_preserves_solid_before_aqueous_of_same_species() {
    for solid_first in [true, false] {
        let mut v = wet();
        let solid = 0.016623393915185407;
        let aqueous = 0.000019972039145196722;
        let portions = if solid_first {
            [(solid, Phase::Solid), (aqueous, Phase::Aqueous)]
        } else {
            [(aqueous, Phase::Aqueous), (solid, Phase::Solid)]
        };
        for (n, phase) in portions {
            put(&mut v, "SiO2", n, phase);
        }
        let b = drain(v);
        let src = b.vessel(VesselId(0)).unwrap();
        let dst = b.vessel(VesselId(1)).unwrap();
        close(amount(src, "SiO2", Phase::Solid), solid);
        close(amount(src, "SiO2", Phase::Aqueous), 0.0);
        close(amount(dst, "SiO2", Phase::Aqueous), aqueous);
        close(amount(dst, "SiO2", Phase::Solid), 0.0);
        close(amount(src, "water", Phase::Liquid), 0.0);
        close(amount(dst, "water", Phase::Liquid), 5.0);
        close(amount(src, "hexane", Phase::Liquid), 0.5);
    }
}
#[test]
fn drain_preserves_gas_before_aqueous_of_same_species() {
    for gas_first in [true, false] {
        let mut v = wet();
        let portions = if gas_first {
            [(0.01, Phase::Gas), (0.002, Phase::Aqueous)]
        } else {
            [(0.002, Phase::Aqueous), (0.01, Phase::Gas)]
        };
        for (n, phase) in portions {
            put(&mut v, "ethanol", n, phase);
        }
        let b = drain(v);
        let src = b.vessel(VesselId(0)).unwrap();
        let dst = b.vessel(VesselId(1)).unwrap();
        close(amount(src, "ethanol", Phase::Gas), 0.01);
        close(amount(dst, "ethanol", Phase::Gas), 0.0);
        assert!(amount(src, "ethanol", Phase::Aqueous) > 0.0);
        assert!(amount(dst, "ethanol", Phase::Aqueous) > 0.0);
        close(
            amount(src, "ethanol", Phase::Aqueous) + amount(dst, "ethanol", Phase::Aqueous),
            0.002,
        );
    }
}
#[test]
fn drain_ordinary_aqueous_first_keeps_both_inventories() {
    let mut v = wet();
    put(&mut v, "SiO2", 0.00001, Phase::Aqueous);
    put(&mut v, "SiO2", 0.01, Phase::Solid);
    let b = drain(v);
    close(
        amount(b.vessel(VesselId(0)).unwrap(), "SiO2", Phase::Solid),
        0.01,
    );
    close(
        amount(b.vessel(VesselId(1)).unwrap(), "SiO2", Phase::Aqueous),
        0.00001,
    );
}
#[test]
fn catalyst_promotion_debits_only_solid_inventory() {
    for solid_first in [false, true] {
        let mut v = wet();
        let portions = if solid_first {
            [(0.002, Phase::Solid), (0.001, Phase::Aqueous)]
        } else {
            [(0.001, Phase::Aqueous), (0.002, Phase::Solid)]
        };
        for (n, phase) in portions {
            put(&mut v, "KI", n, phase);
        }
        let events = MixingEquilibrator.equilibrate(&mut v).unwrap();
        close(dissolved(&events, "KI"), 0.002);
        close(amount(&v, "KI", Phase::Solid), 0.0);
        close(amount(&v, "KI", Phase::Aqueous), 0.003);
    }
}
fn silica() -> Vessel {
    let mut v = wet();
    let cap = capacity(&v, "SiO2");
    put(&mut v, "SiO2", cap / 2.0, Phase::Aqueous);
    put(&mut v, "SiO2", 0.01, Phase::Solid);
    v
}
#[test]
fn saturation_events_match_actual_phase_changes() {
    let mut v = silica();
    let before_solid = amount(&v, "SiO2", Phase::Solid);
    let before_aqueous = amount(&v, "SiO2", Phase::Aqueous);
    let cap = capacity(&v, "SiO2");
    let events = MixingEquilibrator.equilibrate(&mut v).unwrap();
    let extent = dissolved(&events, "SiO2");
    assert!(extent > 0.0);
    close(amount(&v, "SiO2", Phase::Solid), before_solid - extent);
    close(amount(&v, "SiO2", Phase::Aqueous), before_aqueous + extent);
    close(amount(&v, "SiO2", Phase::Aqueous), cap);
}
#[test]
fn saturation_direct_and_delta_agree_on_owned_phases() {
    let mut direct = silica();
    let mut delta_state = direct.clone();
    let (delta, _) = MixingEquilibrator.equilibrate_delta(&delta_state).unwrap();
    let errors = delta.validate(&delta_state);
    assert!(
        errors.is_empty(),
        "invalid planned phase transfer: {errors:?}"
    );
    delta.apply(&mut delta_state);
    MixingEquilibrator.equilibrate(&mut direct).unwrap();
    for phase in [Phase::Solid, Phase::Aqueous] {
        close(
            amount(&direct, "SiO2", phase),
            amount(&delta_state, "SiO2", phase),
        );
    }
}
#[test]
fn iodide_assisted_iodine_debits_solid_not_existing_solution() {
    let mut v = wet();
    // Start at pure-water saturation so the independent delta planner has
    // no ordinary-dissolution headroom. Multi-route overlap is a separate test.
    let cap = capacity(&v, "I2");
    let dose = cap / 4.0;
    put(&mut v, "I2", cap, Phase::Aqueous);
    put(&mut v, "I2", dose, Phase::Solid);
    put(&mut v, "KI", 0.01, Phase::Aqueous);
    let events = MixingEquilibrator.equilibrate(&mut v).unwrap();
    let extent = dissolved(&events, "I2");
    close(extent, dose);
    close(amount(&v, "I2", Phase::Solid), 0.0);
    close(amount(&v, "I2", Phase::Aqueous), cap + dose);
    close(amount(&v, "KI", Phase::Aqueous), 0.01);
}
#[test]
fn aqueous_capacity_uses_liquid_water_even_in_minority_layer() {
    for water_phase in [Phase::Solid, Phase::Gas] {
        let mut v = Vessel::new(VesselId(0), "dry");
        put(&mut v, "hexane", 1.0, Phase::Liquid);
        put(&mut v, "water", 5.0, water_phase);
        put(&mut v, "SiO2", 0.01, Phase::Solid);
        assert!(solve::saturation_moves(&v).is_empty());
    }
    let mut water = wet();
    put(&mut water, "SiO2", 0.01, Phase::Solid);
    let mut minority = water.clone();
    put(&mut minority, "hexane", 10.0, Phase::Liquid);
    for v in [&mut water, &mut minority] {
        let cap = capacity(v, "SiO2");
        MixingEquilibrator.equilibrate(v).unwrap();
        close(amount(v, "SiO2", Phase::Aqueous), cap);
        close(amount(v, "SiO2", Phase::Solid), 0.01 - cap);
    }
    close(
        amount(&water, "SiO2", Phase::Aqueous),
        amount(&minority, "SiO2", Phase::Aqueous),
    );
}
