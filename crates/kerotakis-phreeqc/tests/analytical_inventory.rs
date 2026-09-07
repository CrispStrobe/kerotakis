//! Parameter-grid checks of the actual aqueous adapter, not recorded scenarios.
#![cfg(feature = "engine")]
use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn aqueous_basis_conserves_atoms_across_doses_volumes_and_temperatures() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut checked = 0;
    for reagent in [
        "HCl", "NaOH", "CH3COOH", "NaHCO3", "H3PO4", "Na2SO4", "CaCl2", "NH3",
    ] {
        for litres in [0.001, 0.002, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0] {
            for molarity in [1e-6, 1e-4, 0.001, 0.01, 0.1] {
                for temperature in [280.0, 285.0, 290.0, 298.15, 305.0, 310.0, 330.0, 350.0] {
                    let mut v = Vessel::new(VesselId(0), "beaker");
                    v.temperature = Kelvin(temperature);
                    v.thermal_mode = ThermalMode::Thermostatted(Kelvin(temperature));
                    let water = species::lookup_key("water").unwrap();
                    v.deposit(
                        SpeciesId::new("water"),
                        water.moles_from_liters(Liters(litres)),
                        Phase::Liquid,
                    );
                    let data = species::lookup_key(reagent).unwrap();
                    v.deposit(
                        SpeciesId::new(reagent),
                        Moles(litres * molarity),
                        data.standard_phase,
                    );
                    let mut before = ConservedLedger::from_vessel(&v);
                    let events = solver.equilibrate(&mut v).unwrap_or_else(|e| {
                        panic!("{reagent}, {litres} L, {molarity} M, {temperature} K: {e}")
                    });
                    for event in events {
                        let (sid, n) = match event {
                            Event::GasEvolved { species, moles, .. } => (species, -moles.0),
                            Event::GasAbsorbed { species, moles, .. } => (species, moles.0),
                            _ => continue,
                        };
                        let formula =
                            stoich::parse_formula(species::lookup(&sid).unwrap().formula).unwrap();
                        for (element, coefficient) in formula.counts {
                            *before.elements.entry(element).or_default() += coefficient * n;
                        }
                    }
                    let after = ConservedLedger::from_vessel(&v);
                    for (element, expected) in before.elements {
                        let actual = after.elements.get(&element).copied().unwrap_or_default();
                        assert!((actual - expected).abs() < 1e-9 + expected.abs() * 2e-6,
                            "{reagent}, {litres} L, {molarity} M, {temperature} K: {element}: {expected} -> {actual}");
                    }
                    assert!(v
                        .contents
                        .iter()
                        .all(|p| p.moles.0.is_finite() && p.moles.0 >= 0.0));
                    assert!(v.solution.as_ref().unwrap().ph.is_finite());
                    if temperature == 298.15 && matches!(reagent, "HCl" | "NaOH") {
                        // Independent dilute strong-electrolyte limit. Activity
                        // corrections are allowed; orders-of-magnitude errors are not.
                        let ideal = if reagent == "HCl" {
                            -molarity.log10()
                        } else {
                            14.0 + molarity.log10()
                        };
                        assert!((v.solution.as_ref().unwrap().ph - ideal).abs() < 0.15);
                    }
                    assert_eq!(
                        v.temperature.0, temperature,
                        "thermostatted temperature must not drift during basis changes"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 2560);
}

#[test]
fn trace_ligand_routing_never_drops_nitrogen() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut computed = 0;
    let mut refused = 0;
    for litres in [0.01, 0.1, 1.0] {
        for ammonia_ratio in [2.0, 20.0, 200.0] {
            for acetate_molarity in [0.0, 1e-12, 1e-9, 1e-6] {
                let mut v = Vessel::new(VesselId(0), "beaker");
                v.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
                let nitrogen = litres * 0.001 * ammonia_ratio;
                for (key, amount) in [
                    (
                        "water",
                        species::lookup_key("water")
                            .unwrap()
                            .moles_from_liters(Liters(litres))
                            .0,
                    ),
                    ("CuSO4", litres * 0.001),
                    ("NH3", nitrogen),
                    ("NaOAc", litres * acetate_molarity),
                ] {
                    v.deposit(
                        SpeciesId::new(key),
                        Moles(amount),
                        species::lookup_key(key).unwrap().standard_phase,
                    );
                }
                let initial = serde_json::to_value(&v).unwrap();
                let events = match solver.equilibrate(&mut v) {
                    Ok(events) => {
                        computed += 1;
                        events
                    }
                    Err(error) => {
                        // A refusal must still be atomic, but is no longer
                        // accepted as model coverage: native component
                        // isolation is required to compute all these feeds.
                        assert!(
                            error.to_string().contains("unrepresented elemental states"),
                            "{error}"
                        );
                        assert_eq!(serde_json::to_value(&v).unwrap(), initial);
                        refused += 1;
                        continue;
                    }
                };
                let escaped: f64 = events
                    .iter()
                    .filter_map(|event| match event {
                        Event::GasEvolved {
                            species: sid,
                            moles,
                            ..
                        } => Some((sid, moles.0)),
                        Event::GasAbsorbed {
                            species: sid,
                            moles,
                            ..
                        } => Some((sid, -moles.0)),
                        _ => None,
                    })
                    .map(|(sid, moles)| {
                        let f =
                            stoich::parse_formula(species::lookup(sid).unwrap().formula).unwrap();
                        moles * f.counts.get("N").copied().unwrap_or(0.0)
                    })
                    .sum();
                let retained = ConservedLedger::from_vessel(&v)
                    .elements
                    .get("N")
                    .copied()
                    .unwrap_or(0.0);
                assert!((retained + escaped - nitrogen).abs() < 1e-9 + nitrogen * 2e-6,
                    "{litres} L, NH3/Cu={ammonia_ratio}, acetate={acetate_molarity} M: N {nitrogen} -> {retained} + {escaped}");
                let solution = v.solution.as_ref().unwrap();
                assert!(!solution.redox.is_empty());
                for state in &solution.redox {
                    let wrong = match state.element.as_str() {
                        "N" => state.oxidation != -3,
                        "S" => state.oxidation != 6,
                        _ => false,
                    };
                    assert!(
                        !wrong || state.molality * solution.solvent_kg.unwrap() < 1e-10,
                        "native state contradicts isolated feed: {state:?}"
                    );
                }
            }
        }
    }
    assert_eq!(refused, 0, "all represented mixed feeds must compute");
    assert_eq!(computed, 36);
    assert_eq!(computed + refused, 36);
    eprintln!("ligand routing: {computed} computed, {refused} explicitly refused with unchanged inventory");
}
