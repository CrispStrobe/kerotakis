//! Gibbs minimisation against textbook chemistry: combustion, flame
//! temperature, and thermal decomposition.

use std::collections::BTreeMap;

use kerotakis_cea::{db, equilibrate_hp, equilibrate_tp};

fn budget(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn pool(names: &[&str]) -> Vec<&'static kerotakis_cea::Species> {
    names.iter().filter_map(|n| db().get(n)).collect()
}

#[test]
fn methane_burns_to_carbon_dioxide_and_water() {
    // CH4 + 2 O2 → CO2 + 2 H2O, at a temperature where equilibrium is
    // essentially complete combustion.
    let species = pool(&["CH4", "O2", "CO2", "H2O", "CO", "H2", "OH", "O", "H"]);
    let eq = equilibrate_tp(
        &budget(&[("C", 1.0), ("H", 4.0), ("O", 4.0)]),
        &species,
        1200.0,
        1.0,
    )
    .expect("equilibrium");

    assert!(
        (eq.moles_of("CO2") - 1.0).abs() < 0.02,
        "essentially all carbon ends as CO2: {:?}",
        eq.composition
    );
    assert!(
        (eq.moles_of("H2O") - 2.0).abs() < 0.05,
        "and all hydrogen as water: {:?}",
        eq.composition
    );
    assert!(eq.moles_of("CH4") < 1e-6, "no methane survives");
    assert!(
        !eq.sources.is_empty(),
        "the result cites its species' sources"
    );
}

#[test]
fn elements_are_conserved_exactly() {
    let species = pool(&["CH4", "O2", "CO2", "H2O", "CO", "H2", "OH", "O", "H"]);
    let eq = equilibrate_tp(
        &budget(&[("C", 1.0), ("H", 4.0), ("O", 4.0)]),
        &species,
        2000.0,
        1.0,
    )
    .expect("equilibrium");

    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for (name, moles) in &eq.composition {
        let s = db().get(name).unwrap();
        for (el, count) in &s.composition {
            *totals.entry(el.clone()).or_insert(0.0) += count * moles;
        }
    }
    for (el, want) in [("C", 1.0), ("H", 4.0), ("O", 4.0)] {
        let got = totals.get(el).copied().unwrap_or(0.0);
        assert!(
            (got - want).abs() < 1e-6,
            "element {el}: expected {want}, got {got}"
        );
    }
}

#[test]
fn dissociation_appears_at_flame_temperatures() {
    // The pedagogical point: at 1200 K combustion looks complete, but at
    // 3000 K the products are visibly dissociated into CO, OH, H, O.
    let species = pool(&["CH4", "O2", "CO2", "H2O", "CO", "H2", "OH", "O", "H"]);
    let b = budget(&[("C", 1.0), ("H", 4.0), ("O", 4.0)]);
    let cool = equilibrate_tp(&b, &species, 1200.0, 1.0).expect("cool");
    let hot = equilibrate_tp(&b, &species, 3000.0, 1.0).expect("hot");

    assert!(cool.moles_of("CO") < 1e-3, "no CO when cool");
    assert!(
        hot.moles_of("CO") > 0.1,
        "CO2 dissociates to CO at 3000 K: {:?}",
        hot.composition
    );
    assert!(
        hot.moles_of("OH") > 0.01,
        "radicals appear: {:?}",
        hot.composition
    );
}

#[test]
fn methane_in_air_has_a_textbook_flame_temperature() {
    // Stoichiometric CH4/air: Tad ≈ 2225 K. The enthalpy of the cold
    // reactants sets the adiabatic condition.
    let species = pool(&[
        "CH4", "O2", "N2", "CO2", "H2O", "CO", "H2", "OH", "O", "H", "NO",
    ]);
    let reactants = [("CH4", 1.0), ("O2", 2.0), ("N2", 7.52)];
    let h_cold: f64 = reactants
        .iter()
        .map(|(n, m)| db().get(n).unwrap().h(298.15).unwrap() * m)
        .sum();

    let b = budget(&[("C", 1.0), ("H", 4.0), ("O", 4.0), ("N", 15.04)]);
    let flame = equilibrate_hp(&b, &species, h_cold, 1.0).expect("flame");
    assert!(
        (flame.temperature - 2225.0).abs() < 120.0,
        "adiabatic flame temperature ≈ 2225 K, got {:.0} K",
        flame.temperature
    );
}

#[test]
fn chalk_decomposes_when_hot_enough() {
    // CaCO3 → CaO + CO2. Below the decomposition temperature the carbonate
    // is stable; above it, the kiln works.
    //
    // Modelled with the air that is actually in the vessel: with CO2 as
    // the only possible gas the problem has no gas phase at all below the
    // decomposition point, which is physically true and numerically
    // degenerate. A real beaker has an atmosphere, and including it is both
    // more honest and better conditioned.
    let species = pool(&["CO2", "CaCO3(cr)", "CaO(cr)", "CO", "O2", "N2"]);
    assert!(
        species.len() >= 5,
        "need the condensed calcium phases, found {:?}",
        species.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    let air = |ca: f64| budget(&[("Ca", ca), ("C", ca), ("O", 3.0 * ca + 2.0), ("N", 8.0)]);

    let cold = equilibrate_tp(&air(1.0), &species, 800.0, 1.0).expect("cold");
    let hot = equilibrate_tp(&air(1.0), &species, 1400.0, 1.0).expect("hot");

    assert!(
        cold.moles_of("CaCO3(cr)") > 0.9,
        "chalk is stable at 800 K: {:?}",
        cold.composition
    );
    assert!(
        cold.moles_of("CaO(cr)") < 0.1,
        "and has not turned to quicklime yet: {:?}",
        cold.composition
    );
    assert!(
        hot.moles_of("CaO(cr)") > 0.9 && hot.moles_of("CO2") > 0.9,
        "and decomposes by 1400 K: {:?}",
        hot.composition
    );
}

#[test]
fn the_decomposition_temperature_is_computed_not_assumed() {
    // Walk the temperature up and find where the carbonate gives way. The
    // textbook figure for CaCO3 in air is around 1100-1200 K.
    let species = pool(&["CO2", "CaCO3(cr)", "CaO(cr)", "CO", "O2", "N2"]);
    let b = budget(&[("Ca", 1.0), ("C", 1.0), ("O", 5.0), ("N", 8.0)]);
    let mut switch = None;
    let mut t = 700.0;
    while t <= 1500.0 {
        let eq = equilibrate_tp(&b, &species, t, 1.0).expect("equilibrium");
        if eq.moles_of("CaO(cr)") > 0.5 {
            switch = Some(t);
            break;
        }
        t += 25.0;
    }
    let t_switch = switch.expect("the carbonate must decompose somewhere in range");
    assert!(
        (900.0..=1300.0).contains(&t_switch),
        "decomposition near the textbook ~1170 K, computed {t_switch:.0} K"
    );
}
