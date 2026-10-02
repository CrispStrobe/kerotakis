//! A published characterisation and the persistent next-step inputs must agree.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn run(b: &mut Bench, s: &mut PhreeqcEquilibrator, line: &str) {
    let events = b
        .step_with(
            script::parse_op(line).unwrap().unwrap(),
            s,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{line}: {events:?}"
    );
}
fn scalar_contract(v: &Vessel) {
    let info = v.solution.as_ref().expect("characterised solution");
    let kgw = info.solvent_kg.expect("solver solvent mass");
    for (name, measured) in [("H+", v.free_proton), ("OH-", v.free_hydroxide)] {
        let expected = info
            .species
            .iter()
            .filter(|p| p.name == name)
            .map(|p| p.molality)
            .sum::<f64>()
            * kgw;
        // The report omits species below 1e-9 mol/kgw and rounds included
        // rows. This checks the independent persistent readback, not exact
        // string equality between differently rounded engine outputs.
        assert!(
            (expected - measured).abs() <= 2e-9 * kgw + expected.abs() * 8e-4,
            "{name}: persistent {measured:e}, speciation {expected:e}, pH {}",
            info.ph
        );
    }
    if let Some(h) = info
        .species
        .iter()
        .find(|p| p.name == "H+" && p.activity > 0.0)
    {
        assert!(
            (info.ph + h.activity.log10()).abs() < 8e-4,
            "pH {} vs H+ activity {}",
            info.ph,
            h.activity
        );
    }
    let co2 = info
        .species
        .iter()
        .filter(|p| matches!(p.name.as_str(), "CO2" | "H2CO3"))
        .map(|p| p.molality)
        .sum::<f64>();
    let coefficient = kerotakis_core::properties::henry_lookup("CO2").unwrap();
    let expected = co2 / kerotakis_core::properties::henry_at_t(coefficient, v.temperature.0).value;
    if let Some(measured) = v.co2_partial_pressure_atm {
        assert!(
            (expected - measured).abs() <= 1e-10 + expected.abs() * 8e-4,
            "CO2 driving pressure {measured:e}, current speciation {expected:e}"
        );
    }
}

#[test]
fn canonical_ordering_updates_all_next_step_inputs_with_the_report() {
    for additions in [
        ["add v1 CaCl2 0.01mol", "add v1 laundry_detergent 5g"],
        ["add v1 laundry_detergent 5g", "add v1 CaCl2 0.01mol"],
    ] {
        let mut b = Bench::new();
        let mut s = PhreeqcEquilibrator::new().unwrap();
        run(&mut b, &mut s, "add v1 water 100mL");
        for line in additions {
            run(&mut b, &mut s, line);
            scalar_contract(&b.vessels[0]);
        }
        run(&mut b, &mut s, "heat v1 1J");
        scalar_contract(&b.vessels[0]);
    }
}

#[test]
fn carbon_driving_pressure_matches_speciation_across_finite_and_room_boundaries() {
    for sealed in [false, true] {
        let mut b = Bench::new();
        let mut s = PhreeqcEquilibrator::new().unwrap();
        if sealed {
            b.vessels[0].headspace = Headspace::Sealed {
                volume: Liters(0.1),
            };
        }
        for line in [
            "add v1 water 100mL",
            "add v1 CO2 0.001mol",
            "wait 1s",
            "heat v1 1J",
            "wait 1s",
        ] {
            run(&mut b, &mut s, line);
            if line != "add v1 water 100mL" {
                scalar_contract(&b.vessels[0]);
            }
        }
    }
}

#[test]
fn redox_population_and_scalar_readback_share_the_same_characterisation() {
    let mut b = Bench::new();
    let mut s = PhreeqcEquilibrator::new().unwrap();
    run(&mut b, &mut s, "add v1 water 100mL");
    for line in ["add v1 FeCl3 0.001mol", "heat v1 1J", "wait 1s"] {
        run(&mut b, &mut s, line);
        scalar_contract(&b.vessels[0]);
        let v = &b.vessels[0];
        let info = v.solution.as_ref().unwrap();
        let kgw = info.solvent_kg.unwrap();
        let reported = info
            .redox
            .iter()
            .filter(|s| s.element == "Fe")
            .map(|s| s.molality)
            .sum::<f64>()
            * kgw;
        let total = kerotakis_core::ledger::ConservedLedger::from_vessel(v).elements["Fe"];
        assert!(
            (reported - total).abs() < 1e-9 + total * 8e-4,
            "redox {reported:e}, inventory {total:e}"
        );
    }
}

#[test]
fn characterization_cannot_assume_a_different_finite_surface_inventory() {
    let mut b = Bench::new();
    let mut s = PhreeqcEquilibrator::new().unwrap();
    b.vessels[0].surfaces.push(SurfaceSites {
        label: "oxide".into(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: vec![],
        water_release: Moles(0.0),
    });
    run(&mut b, &mut s, "add v1 water 1000mL");
    for line in ["add v1 ZnSO4 0.0001mol", "heat v1 1J", "wait 1s"] {
        run(&mut b, &mut s, line);
        scalar_contract(&b.vessels[0]);
        let v = &b.vessels[0];
        let info = v.solution.as_ref().unwrap();
        let dissolved = info
            .species
            .iter()
            .filter_map(|p| {
                stoich::parse_formula(&p.name)
                    .ok()
                    .map(|f| f.counts.get("Zn").copied().unwrap_or(0.0) * p.molality)
            })
            .sum::<f64>()
            * info.solvent_kg.unwrap();
        let bound = v
            .surfaces
            .iter()
            .map(|p| p.bound(SurfaceSorbate::Zinc).0)
            .sum::<f64>();
        assert!(
            (dissolved + bound - 0.0001).abs() < 1e-8 + 0.0001 * 8e-4,
            "dissolved {dissolved:e}, committed interface {bound:e}"
        );
    }
}

#[test]
fn small_finite_carbon_dose_is_not_limited_by_four_digit_report_rounding() {
    let mut b = Bench::new();
    let mut s = PhreeqcEquilibrator::new().unwrap();
    run(&mut b, &mut s, "add v1 water 50mL");
    run(&mut b, &mut s, "add v1 CO2 0.00001mol");
    let v = &b.vessels[0];
    let info = v.solution.as_ref().unwrap();
    let represented = info
        .species
        .iter()
        .filter(|p| matches!(p.name.as_str(), "CO2" | "H2CO3" | "HCO3-" | "CO3-2"))
        .map(|p| p.molality)
        .sum::<f64>()
        * info.solvent_kg.unwrap();
    assert!(
        (represented - 0.00001).abs() < 1e-10,
        "precise speciation {represented:.15e}"
    );
    scalar_contract(v);
}

#[test]
fn empty_finite_surface_cells_converge_without_importing_metal_or_sulfate() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    // The first mass is the minimized downstream transport cell. Neighboring
    // masses and a different scale exercise the numerical domain, not one
    // floating-point spelling of that cell.
    for kgw in [0.099999463, 0.099999464, 0.099999465, 0.1, 1.0] {
        let mut vessel = Vessel::new(VesselId(0), "empty oxide cell");
        vessel.deposit(
            SpeciesId::new("water"),
            Moles(kgw * 1000.0 / 18.015),
            Phase::Liquid,
        );
        vessel.surfaces.push(SurfaceSites {
            label: "finite oxide".into(),
            model: SurfaceModel::HydrousFerricOxide,
            mass: Grams(0.09),
            specific_area_m2_per_g: 600.0,
            strong_capacity: Moles(5e-6),
            weak_capacity: Moles(2e-4),
            occupancy: vec![],
            water_release: Moles(0.0),
        });
        let before = kerotakis_core::ledger::ConservedLedger::from_vessel(&vessel);
        solver
            .equilibrate(&mut vessel)
            .unwrap_or_else(|err| panic!("empty finite surface at {kgw} kg: {err:?}"));
        scalar_contract(&vessel);
        let after = kerotakis_core::ledger::ConservedLedger::from_vessel(&vessel);
        for element in ["Zn", "S"] {
            assert_eq!(after.elements.get(element).copied().unwrap_or(0.0), 0.0);
        }
        for element in ["H", "O"] {
            let initial = before.elements[element];
            let final_amount = after.elements[element];
            assert!(
                (initial - final_amount).abs() <= initial * 1e-9,
                "{kgw} kg, {element}: {initial:e} -> {final_amount:e}"
            );
        }
        let surface = &vessel.surfaces[0];
        assert_eq!(surface.strong_capacity.0, 5e-6);
        assert_eq!(surface.weak_capacity.0, 2e-4);
        assert_eq!(surface.bound(SurfaceSorbate::Zinc).0, 0.0);
        assert_eq!(surface.bound(SurfaceSorbate::Sulfate).0, 0.0);
    }
}

fn named_finite_oxide() -> SurfaceSites {
    SurfaceSites {
        label: "receiving oxide bed".into(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: vec![],
        water_release: Moles(0.0),
    }
}

#[test]
fn native_mix_declines_every_finite_interface_without_mutating_any_vessel() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for owner in 0..3 {
        for interface in 0..3 {
            let mut vessels: Vec<_> = (0..3)
                .map(|i| {
                    let mut v = Vessel::new(VesselId(i), "aqueous salt");
                    v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
                    v.deposit(SpeciesId::new("Na+"), Moles(1e-4), Phase::Aqueous);
                    v.deposit(SpeciesId::new("Cl-"), Moles(1e-4), Phase::Aqueous);
                    v
                })
                .collect();
            match interface {
                0 => vessels[owner].surfaces.push(named_finite_oxide()),
                1 => vessels[owner].exchanges.push(ExchangeSites {
                    label: "named finite resin".into(),
                    dry_mass: Grams(0.1),
                    capacity: Moles(1e-4),
                    occupancy: vec![ExchangeOccupancy {
                        ion: ExchangeIon::Sodium,
                        moles: Moles(1e-4),
                    }],
                }),
                _ => vessels[owner]
                    .solid_solutions
                    .push(SolidSolution::aragonite_strontianite(
                        "named mixed crystal",
                        Moles(1e-4),
                        Moles(1e-5),
                    )),
            }
            let mut target = vessels.remove(0);
            let a = &vessels[0];
            let b = &vessels[1];
            let before = format!("{target:?}{a:?}{b:?}");
            assert!(solver.mix(&mut target, a, 0.5, b, 0.25).is_none());
            assert_eq!(format!("{target:?}{a:?}{b:?}"), before);
        }
    }
}

#[test]
fn production_pour_retains_named_oxide_and_conserves_the_fractional_zinc_inlet() {
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for line in [
        "add v1 water 100mL",
        "add v1 ZnSO4 0.0001mol",
        "add v2 water 100mL",
        "add v2 ZnSO4 0.0001mol",
    ] {
        run(&mut bench, &mut solver, line);
    }
    bench.vessels[2].surfaces.push(named_finite_oxide());
    let zinc = |v: &Vessel| {
        kerotakis_core::ledger::ConservedLedger::from_vessel(v)
            .elements
            .get("Zn")
            .copied()
            .unwrap_or(0.0)
    };
    let inlet = zinc(&bench.vessels[0]) * 0.5 + zinc(&bench.vessels[1]) * 0.25;
    let zinc_before: f64 = bench.vessels.iter().map(zinc).sum();
    let mass_before: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    run(&mut bench, &mut solver, "mix v1 0.5 v2 0.25 into v3");
    let target = &bench.vessels[2];
    let surface = &target.surfaces[0];
    assert_eq!(surface.label, "receiving oxide bed");
    assert_eq!(surface.mass.0, 0.09);
    assert_eq!(surface.strong_capacity.0, 5e-6);
    assert_eq!(surface.weak_capacity.0, 2e-4);
    assert!(surface.bound(SurfaceSorbate::Zinc).0 > 1e-8);
    scalar_contract(target);
    assert!((zinc(target) - inlet).abs() < 1e-10);
    let zinc_after: f64 = bench.vessels.iter().map(zinc).sum();
    assert!((zinc_after - zinc_before).abs() < 1e-10);
    let mass_after: f64 = bench.vessels.iter().map(|v| v.mass().0).sum();
    assert!(
        (mass_after - mass_before).abs() < mass_before * 1e-8,
        "bench mass {mass_before:e} -> {mass_after:e}"
    );
}
