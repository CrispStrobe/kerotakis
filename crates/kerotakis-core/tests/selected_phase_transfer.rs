//! Phase-selected physical routes must debit only the stock they priced.
use kerotakis_core::*;

fn amount(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1e-8),
        "{actual:e} != {expected:e}"
    );
}

fn run(bench: &mut Bench, op: Operator) -> Vec<Event> {
    bench
        .step_with(op, &mut SolverStack::new(vec![]), &PermissiveScreen)
        .unwrap()
}

fn fixture(stock: &[(&str, f64, Phase)], reverse: bool) -> Bench {
    let mut bench = Bench::new();
    bench.vessels.push(Vessel::new(VesselId(1), "receiver"));
    let mut portions = stock.to_vec();
    if reverse {
        portions.reverse();
    }
    for (key, n, phase) in portions {
        // Preserve separate saved/API portions instead of merging equal keys.
        bench.vessels[0].contents.push(Portion {
            species: SpeciesId::new(key),
            moles: Moles(n),
            phase,
        });
    }
    bench
}

#[test]
fn drain_leaves_nonmobile_same_species_stock_in_its_original_phase() {
    for reverse in [false, true] {
        let mut b = fixture(
            &[
                ("water", 0.4, Phase::Solid),
                ("water", 0.2, Phase::Gas),
                ("water", 0.6, Phase::Liquid),
                ("water", 0.4, Phase::Liquid),
                ("hexane", 0.5, Phase::Liquid),
                ("NaCl", 0.002, Phase::Solid),
                ("NaCl", 0.001, Phase::Aqueous),
            ],
            reverse,
        );
        let events = run(
            &mut b,
            Operator::Drain {
                from: VesselId(0),
                to: VesselId(1),
            },
        );
        assert!(events.iter().any(|e| matches!(e, Event::Drained { .. })));
        close(amount(&b.vessels[0], "water", Phase::Solid), 0.4);
        close(amount(&b.vessels[0], "water", Phase::Gas), 0.2);
        close(amount(&b.vessels[0], "water", Phase::Liquid), 0.0);
        close(amount(&b.vessels[0], "NaCl", Phase::Solid), 0.002);
        close(amount(&b.vessels[0], "NaCl", Phase::Aqueous), 0.0);
        close(amount(&b.vessels[1], "water", Phase::Liquid), 1.0);
        close(amount(&b.vessels[1], "NaCl", Phase::Aqueous), 0.001);
        close(amount(&b.vessels[0], "hexane", Phase::Liquid), 0.5);
    }
}

#[test]
fn ethanol_water_still_preserves_ice_and_gas_and_matches_liquid_only_cut() {
    for reverse in [false, true] {
        let eligible = [
            ("water", 1.0, Phase::Liquid),
            ("ethanol", 0.1, Phase::Liquid),
            ("ethanol", 0.1, Phase::Aqueous),
        ];
        let op = Operator::Distil {
            from: VesselId(0),
            to: VesselId(1),
            fraction: Some(0.3),
            energy: None,
            stages: 1,
        };
        let mut control = fixture(&eligible, reverse);
        assert!(run(&mut control, op.clone())
            .iter()
            .any(|e| matches!(e, Event::Distilled { .. })));
        let mut stock = vec![
            ("water", 0.4, Phase::Solid),
            ("water", 0.2, Phase::Gas),
            ("ethanol", 0.15, Phase::Gas),
            ("ethanol", 0.05, Phase::Solid),
        ];
        stock.extend(eligible);
        let mut b = fixture(&stock, reverse);
        assert!(run(&mut b, op)
            .iter()
            .any(|e| matches!(e, Event::Distilled { .. })));
        for (key, n, phase) in &stock[..4] {
            close(amount(&b.vessels[0], key, *phase), *n);
        }
        for key in ["water", "ethanol"] {
            close(
                amount(&b.vessels[1], key, Phase::Liquid),
                amount(&control.vessels[1], key, Phase::Liquid),
            );
            close(
                b.vessels
                    .iter()
                    .map(|v| v.moles_of(&SpeciesId::new(key)).0)
                    .sum(),
                stock
                    .iter()
                    .filter(|(s, ..)| *s == key)
                    .map(|(_, n, _)| n)
                    .sum(),
            );
        }
    }
}

#[test]
fn extraction_debits_reviewed_aqueous_and_solid_pool_without_consuming_gas() {
    for reverse in [false, true] {
        let eligible = [
            ("water", 2.0, Phase::Liquid),
            ("I2", 0.0001, Phase::Solid),
            ("I2", 0.0001, Phase::Aqueous),
        ];
        let op = Operator::Extract {
            from: VesselId(0),
            to: VesselId(1),
            solvent: SpeciesId::new("hexane"),
            total_solvent: Moles(0.5),
            stages: 2,
        };
        let mut control = fixture(&eligible, reverse);
        assert!(run(&mut control, op.clone())
            .iter()
            .any(|e| matches!(e, Event::Extracted { .. })));
        let mut stock = vec![("I2", 0.0003, Phase::Gas), ("I2", 0.0004, Phase::Liquid)];
        stock.extend(eligible);
        let mut b = fixture(&stock, reverse);
        assert!(run(&mut b, op)
            .iter()
            .any(|e| matches!(e, Event::Extracted { .. })));
        close(amount(&b.vessels[0], "I2", Phase::Gas), 0.0003);
        close(amount(&b.vessels[0], "I2", Phase::Liquid), 0.0004);
        for id in 0..2 {
            close(
                amount(&b.vessels[id], "I2", Phase::Aqueous),
                amount(&control.vessels[id], "I2", Phase::Aqueous),
            );
        }
        close(
            b.vessels
                .iter()
                .map(|v| v.moles_of(&SpeciesId::new("I2")).0)
                .sum(),
            0.0009,
        );
    }
}
