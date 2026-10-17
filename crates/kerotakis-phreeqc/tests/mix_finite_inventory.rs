//! Dissolved-only native MIX must decline finite inventories before mutation.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn solution(id: usize) -> Vessel {
    let mut vessel = Vessel::new(VesselId(id), "finite MIX boundary");
    vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("Cl-"), Moles(0.001), Phase::Aqueous);
    vessel
}

#[test]
fn source_and_target_solid_or_gas_decline_without_engine_calls_or_mutation() {
    let mut solver = PhreeqcEquilibrator::new().expect("native engine");
    for (species, phase) in [("NaCl", Phase::Solid), ("O2", Phase::Gas)] {
        for position in 0..3 {
            let mut a = solution(0);
            let mut b = solution(1);
            let mut target = solution(2);
            let location = match position {
                0 => &mut a,
                1 => &mut b,
                _ => &mut target,
            };
            location.deposit(SpeciesId::new(species), Moles(0.01), phase);
            let before = serde_json::to_value(&target).unwrap();
            let calls = solver.engine_calls();
            assert!(
                solver.mix(&mut target, &a, 1.0, &b, 1.0).is_none(),
                "MIX admitted {species} in location {position}"
            );
            assert_eq!(solver.engine_calls(), calls);
            assert_eq!(serde_json::to_value(&target).unwrap(), before);
        }
    }
}

#[test]
fn typed_surface_declines_even_without_a_solid_portion() {
    let mut solver = PhreeqcEquilibrator::new().expect("native engine");
    for position in 0..3 {
        let mut a = solution(0);
        let mut b = solution(1);
        let mut target = solution(2);
        let location = match position {
            0 => &mut a,
            1 => &mut b,
            _ => &mut target,
        };
        location.surfaces.push(SurfaceSites {
            label: "finite HFO".into(),
            model: SurfaceModel::HydrousFerricOxide,
            mass: Grams(0.09),
            specific_area_m2_per_g: 600.0,
            strong_capacity: Moles(5e-6),
            weak_capacity: Moles(2e-4),
            occupancy: Vec::new(),
            water_release: Moles(0.0),
        });
        let before = serde_json::to_value(&target).unwrap();
        let calls = solver.engine_calls();
        assert!(solver.mix(&mut target, &a, 1.0, &b, 1.0).is_none());
        assert_eq!(solver.engine_calls(), calls);
        assert_eq!(serde_json::to_value(&target).unwrap(), before);
    }
}

#[test]
fn exchanger_and_solid_solution_decline_without_mutation() {
    let mut solver = PhreeqcEquilibrator::new().expect("native engine");
    for crystal in [false, true] {
        for position in 0..3 {
            let mut a = solution(0);
            let mut b = solution(1);
            let mut target = solution(2);
            let location = match position {
                0 => &mut a,
                1 => &mut b,
                _ => &mut target,
            };
            if crystal {
                location
                    .solid_solutions
                    .push(SolidSolution::aragonite_strontianite(
                        "finite carbonate crystal",
                        Moles(0.001),
                        Moles(0.001),
                    ));
            } else {
                location.exchanges.push(ExchangeSites {
                    label: "sodium resin".into(),
                    dry_mass: Grams(2.0),
                    capacity: Moles(0.002),
                    occupancy: vec![ExchangeOccupancy {
                        ion: ExchangeIon::Sodium,
                        moles: Moles(0.002),
                    }],
                });
            }
            let before = serde_json::to_value(&target).unwrap();
            let calls = solver.engine_calls();
            assert!(solver.mix(&mut target, &a, 1.0, &b, 1.0).is_none());
            assert_eq!(solver.engine_calls(), calls);
            assert_eq!(serde_json::to_value(&target).unwrap(), before);
        }
    }
}

#[test]
fn ordinary_fallback_retains_finite_salt_and_sealed_inert_gas() {
    let mut solver = PhreeqcEquilibrator::new().expect("native engine");
    let a = solution(0);
    let b = solution(1);
    let mut target = solution(2);
    target.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    target.deposit(SpeciesId::new("NaCl"), Moles(1.0), Phase::Solid);
    target.deposit(SpeciesId::new("O2"), Moles(0.01), Phase::Gas);
    assert!(solver.mix(&mut target, &a, 1.0, &b, 1.0).is_none());
    solver
        .equilibrate(&mut target)
        .expect("ordinary finite inventory solve");
    let sodium: f64 = target
        .contents
        .iter()
        .map(|portion| {
            species::lookup(&portion.species)
                .and_then(|data| stoich::parse_formula(data.formula).ok())
                .map_or(0.0, |formula| {
                    formula.counts.get("Na").copied().unwrap_or(0.0) * portion.moles.0
                })
        })
        .sum();
    assert!(
        (sodium - 1.001).abs() < 1e-8,
        "finite sodium lost: {sodium}"
    );
    assert!(
        target
            .contents
            .iter()
            .any(|portion| portion.species.0 == "NaCl"
                && portion.phase == Phase::Solid
                && portion.moles.0 > 0.1),
        "finite salt remainder missing"
    );
    assert!((target.moles_of(&SpeciesId::new("O2")).0 - 0.01).abs() < 1e-10);
    assert!(target
        .solution
        .as_ref()
        .is_some_and(|solution| solution.ph.is_finite()));
}
