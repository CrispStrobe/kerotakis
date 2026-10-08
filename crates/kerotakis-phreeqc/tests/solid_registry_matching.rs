//! Public mineral identity contracts, independent of the lookup implementation.
use kerotakis_phreeqc::{dbindex::parse_formula, derived::registry_solid_for};

#[test]
fn mineral_identity_preserves_composition_hydration_and_absence() {
    let carbonate = parse_formula("CaCO3").unwrap();
    assert_eq!(registry_solid_for(&carbonate, 0.0), Some("CaCO3"));
    assert_eq!(registry_solid_for(&carbonate, 2.0), None);
    let sulfate = parse_formula("CaSO4").unwrap();
    assert_eq!(registry_solid_for(&sulfate, 2.0), Some("gypsum"));
    assert_eq!(registry_solid_for(&sulfate, 7.0), None);
    let chloride = parse_formula("NaCl").unwrap();
    assert_eq!(registry_solid_for(&chloride, 0.0), Some("NaCl"));
    assert_eq!(registry_solid_for(&chloride, f64::NAN), None);
    assert_eq!(registry_solid_for(&chloride, f64::INFINITY), None);
    assert_eq!(
        registry_solid_for(&parse_formula("XeF6").unwrap(), 0.0),
        None
    );
}

#[test]
fn parallel_lookups_preserve_mineral_identity() {
    // Parallel readers must share identities without changing matching rules.
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                let formula = parse_formula("CaSO4").unwrap();
                for _ in 0..32 {
                    assert_eq!(registry_solid_for(&formula, 2.0), Some("gypsum"));
                    assert_eq!(registry_solid_for(&formula, 3.0), None);
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}
