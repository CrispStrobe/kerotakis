use kerotakis_core::{
    adsorption::AdsorptionEquilibrator, hmix::MixingEnthalpyEquilibrator,
    volatility::HeadspacePartitionEquilibrator, *,
};

fn vessel() -> Vessel {
    Vessel::new(VesselId(0), "inventory")
}

#[test]
fn withdrawals_preserve_positive_unrelated_and_residual_inventory() {
    for phase_specific in [false, true] {
        let mut v = vessel();
        v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Aqueous);
        for (key, amount) in [("Fe", 1e-15), ("methyl_orange", 5e-16)] {
            v.deposit(SpeciesId::new(key), Moles(amount), Phase::Solid);
        }
        let taken = if phase_specific {
            v.withdraw_phase(&SpeciesId::new("NH3"), Moles(0.001), Phase::Aqueous)
        } else {
            v.withdraw(&SpeciesId::new("NH3"), Moles(0.001))
        };
        assert!((taken.0 - 0.001).abs() < 1e-18);
        assert_eq!(v.moles_of(&SpeciesId::new("Fe")).0, 1e-15);
        assert_eq!(v.moles_of(&SpeciesId::new("methyl_orange")).0, 5e-16);

        // Partial removal of a tiny portion must also keep its remainder.
        let before = v.moles_of(&SpeciesId::new("methyl_orange")).0;
        let removed = if phase_specific {
            v.withdraw_phase(&SpeciesId::new("methyl_orange"), Moles(2e-16), Phase::Solid)
        } else {
            v.withdraw(&SpeciesId::new("methyl_orange"), Moles(2e-16))
        };
        assert!(v.moles_of(&SpeciesId::new("methyl_orange")).0 > 0.0);
        assert_eq!(
            v.moles_of(&SpeciesId::new("methyl_orange")).0 + removed.0,
            before
        );
    }
}

#[test]
fn production_distribution_routes_enforce_their_complete_element_ledgers() {
    for route in ["adsorption", "headspace", "mixing heat"] {
        let mut v = vessel();
        v.deposit(SpeciesId::new("water"), Moles(3.0), Phase::Liquid);
        v.deposit(SpeciesId::new("Fe"), Moles(1e-15), Phase::Solid);
        let solver: Box<dyn Equilibrator> = match route {
            "adsorption" => {
                v.deposit(
                    SpeciesId::new("methyl_orange"),
                    Moles(0.001),
                    Phase::Aqueous,
                );
                v.deposit(
                    SpeciesId::new("activated_charcoal"),
                    Moles(0.1),
                    Phase::Solid,
                );
                Box::new(AdsorptionEquilibrator)
            }
            "headspace" => {
                v.headspace = Headspace::Sealed {
                    volume: Liters(0.5),
                };
                v.deposit(SpeciesId::new("NH3"), Moles(0.01), Phase::Aqueous);
                Box::new(HeadspacePartitionEquilibrator)
            }
            _ => {
                v.deposit(SpeciesId::new("propanone"), Moles(1.0), Phase::Liquid);
                Box::new(MixingEnthalpyEquilibrator)
            }
        };
        assert_eq!(solver.element_conservation_tolerance(), Some(1e-10));
        let before = ConservedLedger::from_vessel(&v);
        let mut stack = SolverStack::new(vec![solver]);
        let events = stack.equilibrate(&mut v).unwrap();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SolverFailed { .. })),
            "{route}: {events:?}"
        );
        assert_eq!(v.moles_of(&SpeciesId::new("Fe")).0, 1e-15, "{route}");
        let after = ConservedLedger::from_vessel(&v);
        assert!(
            before.check_against(&after, 1e-10, 1e-15).is_empty(),
            "{route}"
        );
        match route {
            "adsorption" => {
                assert!(v.adsorbed_moles(&SpeciesId::new("methyl_orange")).0 > 0.0);
                // C14H14N3NaO3S dye plus C charcoal; the bound dye retains
                // its atoms independently of its dissolved/bound split.
                for (element, expected) in [("C", 0.114), ("N", 0.003), ("Na", 0.001), ("S", 0.001)]
                {
                    assert!((after.elements[element] - expected).abs() < 1e-14);
                }
            }
            "headspace" => {
                assert!(v
                    .contents
                    .iter()
                    .any(|p| p.species.0 == "NH3" && p.phase == Phase::Gas));
                assert!((after.elements["N"] - 0.01).abs() < 1e-14);
                assert!((after.elements["H"] - 6.03).abs() < 1e-13);
            }
            _ => {
                assert!(events
                    .iter()
                    .any(|e| matches!(e, Event::HeatOfMixing { .. })));
                // One mole C3H6O plus three moles H2O, independently of
                // the fitted mixing heat or the resulting temperature.
                for (element, expected) in [("C", 3.0), ("H", 12.0), ("O", 4.0)] {
                    assert!((after.elements[element] - expected).abs() < 1e-13);
                }
            }
        }
    }
}

#[test]
fn sulfate_surface_water_transfer_counts_each_atom_and_charge_once() {
    let mut before = vessel();
    before.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    before.deposit(SpeciesId::new("SO4-2"), Moles(0.001), Phase::Aqueous);
    before.surfaces.push(SurfaceSites {
        label: "neutral HFO reference".into(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(0.002),
        weak_capacity: Moles(0.002),
        occupancy: vec![],
        water_release: Moles(0.0),
    });
    let mut after = before.clone();
    after.withdraw(&SpeciesId::new("SO4-2"), Moles(0.001));
    after.deposit(SpeciesId::new("water"), Moles(0.001), Phase::Liquid);
    after.surfaces[0].water_release = Moles(0.001);
    after.surfaces[0].occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Weak,
        sorbate: SurfaceSorbate::Sulfate,
        moles: Moles(0.001),
    });
    let a = ConservedLedger::from_vessel(&before);
    let b = ConservedLedger::from_vessel(&after);
    for ledger in [&a, &b] {
        for (element, expected) in [("H", 10.0), ("O", 5.004), ("S", 0.001)] {
            assert!((ledger.elements[element] - expected).abs() < 1e-13);
        }
        assert!((ledger.charge + 0.002).abs() < 1e-15);
    }
    assert!(a.check_against(&b, 1e-10, 1e-15).is_empty());
}

#[test]
fn exchanging_two_sodiums_for_calcium_preserves_formal_charge_and_atoms() {
    let mut before = vessel();
    before.deposit(SpeciesId::new("Ca+2"), Moles(0.001), Phase::Aqueous);
    before.exchanges.push(ExchangeSites {
        label: "resin".into(),
        dry_mass: Grams(2.0),
        capacity: Moles(0.002),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(0.002),
        }],
    });
    let mut after = before.clone();
    after.withdraw(&SpeciesId::new("Ca+2"), Moles(0.001));
    after.deposit(SpeciesId::new("Na+"), Moles(0.002), Phase::Aqueous);
    after.exchanges[0].occupancy = vec![ExchangeOccupancy {
        ion: ExchangeIon::Calcium,
        moles: Moles(0.001),
    }];
    let a = ConservedLedger::from_vessel(&before);
    let b = ConservedLedger::from_vessel(&after);
    for ledger in [&a, &b] {
        assert_eq!(ledger.elements["Na"], 0.002);
        assert_eq!(ledger.elements["Ca"], 0.001);
        // Fixed resin charge is a unchanged reference; represented mobile
        // and bound cations carry four millimoles of positive equivalents.
        assert_eq!(ledger.charge, 0.004);
    }
    assert!(a.check_against(&b, 1e-10, 1e-15).is_empty());
}

#[test]
fn very_large_withdrawal_requests_report_the_actual_finite_transfer() {
    for phase_specific in [false, true] {
        let mut v = vessel();
        v.deposit(SpeciesId::new("NH3"), Moles(0.009), Phase::Aqueous);
        v.deposit(SpeciesId::new("Fe"), Moles(1e-16), Phase::Solid);
        let before = format!("{v:?}");
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let removed = if phase_specific {
                v.withdraw_phase(&SpeciesId::new("NH3"), Moles(invalid), Phase::Aqueous)
            } else {
                v.withdraw(&SpeciesId::new("NH3"), Moles(invalid))
            };
            assert!(removed.0.is_nan());
            assert_eq!(format!("{v:?}"), before);
        }
        let removed = if phase_specific {
            v.withdraw_phase(&SpeciesId::new("NH3"), Moles(1e100), Phase::Aqueous)
        } else {
            v.withdraw(&SpeciesId::new("NH3"), Moles(1e100))
        };
        assert_eq!(removed.0, 0.009);
        assert_eq!(v.moles_of(&SpeciesId::new("NH3")).0, 0.0);
        assert_eq!(v.moles_of(&SpeciesId::new("Fe")).0, 1e-16);
    }
}
