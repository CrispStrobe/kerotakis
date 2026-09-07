//! The explicit equilibrium request must solve arbitrary initial inventories,
//! including excess water and reverse hydrolysis, without prescribed yields.
use kerotakis_core::*;

#[test]
fn ester_equilibrium_obeys_mass_action_and_conservation_across_1280_mixtures() {
    let mut checked = 0;
    for scale in [1e-6, 1e-3, 1.0, 10.0, 1000.0] {
        for acid in [0.0, 0.1, 1.0, 10.0] {
            for alcohol in [0.0, 0.1, 1.0, 10.0] {
                for ester in [0.0, 0.1, 1.0, 10.0] {
                    for water in [0.0, 0.1, 1.0, 10.0] {
                        let mut bench = Bench::new();
                        let v = &mut bench.vessels[0];
                        for (key, n) in [
                            ("CH3COOH", acid),
                            ("ethanol", alcohol),
                            ("ethyl_acetate", ester),
                            ("water", water),
                        ] {
                            v.deposit(
                                SpeciesId::new(key),
                                Moles(n * scale),
                                species::lookup_key(key).unwrap().standard_phase,
                            );
                        }
                        let before = ledger::ConservedLedger::from_vessel(v);
                        // No solver stack: isolate the requested equilibrium from
                        // evaporation and aqueous dissociation (separate laws).
                        let mut solver = SolverStack::new(vec![]);
                        bench
                            .step_with(
                                Operator::React {
                                    vessel: VesselId(0),
                                    reaction: "esterification".into(),
                                },
                                &mut solver,
                                &PermissiveScreen,
                            )
                            .unwrap();
                        let v = &bench.vessels[0];
                        let after = ledger::ConservedLedger::from_vessel(v);
                        for (el, n) in before.elements {
                            assert!(
                                (after.elements.get(&el).copied().unwrap_or(0.0) - n).abs()
                                    < 1e-10 + n.abs() * 1e-10
                            );
                        }
                        let n = |key| v.moles_of(&SpeciesId::new(key)).0;
                        let r = n("CH3COOH") * n("ethanol");
                        let p = n("ethyl_acetate") * n("water");
                        if r > scale * scale * 1e-12 && p > scale * scale * 1e-12 {
                            assert!(
                                (p / r - 4.0).abs() < 1e-4,
                                "initial [{acid}, {alcohol}, {ester}, {water}] × {scale}: Q={}",
                                p / r
                            );
                        }
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 1280);
}
