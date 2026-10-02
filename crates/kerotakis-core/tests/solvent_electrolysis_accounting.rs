//! Independently balanced undivided-cell half reactions and production stock.
use std::collections::BTreeMap;

use kerotakis_core::{displacement, species, stoich, *};

fn electrolyte(anion: &str, copper: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("Na+"), Moles(0.1), Phase::Aqueous);
    vessel.deposit(
        SpeciesId::new(anion),
        Moles(if anion == "Cl-" { 0.1 } else { 0.05 }),
        Phase::Aqueous,
    );
    if copper > 0.0 {
        vessel.deposit(SpeciesId::new("Cu+2"), Moles(copper), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("SO4-2"), Moles(copper), Phase::Aqueous);
    }
    vessel
}

fn inventory(vessel: &Vessel) -> (BTreeMap<String, f64>, f64) {
    let mut atoms = BTreeMap::new();
    let mut charge = 0.0;
    for portion in &vessel.contents {
        let record =
            species::lookup(&portion.species).expect("every tested stock has a registry record");
        let formula =
            stoich::parse_formula(record.formula).expect("every tested stock has a formula");
        for (element, count) in formula.counts {
            *atoms.entry(element).or_default() += count * portion.moles.0;
        }
        charge += formula.charge * portion.moles.0;
    }
    (atoms, charge)
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "actual {actual:.16e}, expected {expected:.16e}"
    );
}

fn conserved(before: &Vessel, after: &Vessel) {
    let (before_atoms, before_charge) = inventory(before);
    let (after_atoms, after_charge) = inventory(after);
    for element in before_atoms.keys().chain(after_atoms.keys()) {
        close(
            after_atoms.get(element).copied().unwrap_or(0.0),
            before_atoms.get(element).copied().unwrap_or(0.0),
        );
    }
    close(after_charge, before_charge);
}

fn apply_run(vessel: &mut Vessel, run: &displacement::SolventElectrolysis) {
    vessel.withdraw(&SpeciesId::new("water"), Moles(run.water_spent));
    if let Some((ion, amount)) = &run.cathode_ion {
        vessel.withdraw(ion, Moles(*amount));
    }
    vessel.withdraw(&SpeciesId::new("Cl-"), Moles(run.chloride_spent));
    vessel.deposit(
        run.cathode.clone(),
        Moles(run.cathode_moles),
        if run.cathode_plates {
            Phase::Solid
        } else {
            Phase::Gas
        },
    );
    vessel.deposit(
        SpeciesId::new("H2"),
        Moles(run.cathode_hydrogen_moles),
        Phase::Gas,
    );
    vessel.deposit(run.anode.clone(), Moles(run.anode_moles), Phase::Gas);
    vessel.deposit(
        SpeciesId::new("H+"),
        Moles(run.protons_made),
        Phase::Aqueous,
    );
    vessel.deposit(
        SpeciesId::new("OH-"),
        Moles(run.hydroxide_made),
        Phase::Aqueous,
    );
}

fn half_reactions(anion: &str) {
    let electrons = 60.0 / 96_485.332_12;
    for copper in [0.0, 0.0001, 0.001] {
        let before = electrolyte(anion, copper);
        let run = displacement::electrolyse_solvent(&before, 1.0, 60.0).unwrap();
        let metal = copper.min(electrons / 2.0);
        let hydrogen = (electrons - 2.0 * metal) / 2.0;
        let actual_hydrogen = if run.cathode_plates {
            run.cathode_hydrogen_moles
        } else {
            run.cathode_moles
        };
        close(run.electrons, electrons);
        close(actual_hydrogen, hydrogen);
        if run.cathode_plates {
            assert_eq!(run.cathode.0, "Cu");
            close(run.cathode_moles, metal);
            close(run.electrons_to_hydrogen, 2.0 * hydrogen);
        } else {
            assert_eq!(run.cathode.0, "H2");
            // The co-evolution fields are additional to the named product;
            // primary H2 must not be counted a second time as overflow.
            close(run.cathode_hydrogen_moles, 0.0);
        }
        close(2.0 * metal + 2.0 * actual_hydrogen, electrons);
        if anion == "Cl-" {
            // 2Cl- -> Cl2+2e; 2H2O+2e -> H2+2OH-.
            assert_eq!(run.anode.0, "Cl2");
            close(2.0 * run.anode_moles, electrons);
            close(run.chloride_spent, electrons);
            close(run.water_spent, 2.0 * hydrogen);
            close(run.hydroxide_made, 2.0 * hydrogen);
            close(run.protons_made, 0.0);
        } else {
            // Undivided oxygen evolution cancels the H2 cathode's OH-.
            // Net H2O loss=e/2; only the metal-plating charge leaves acid.
            assert_eq!(run.anode.0, "O2");
            close(4.0 * run.anode_moles, electrons);
            close(run.water_spent, electrons / 2.0);
            close(run.protons_made, 2.0 * metal);
            close(run.hydroxide_made, 0.0);
        }
        let mut after = before.clone();
        apply_run(&mut after, &run);
        conserved(&before, &after);
    }
}

#[test]
fn sulfate_half_reactions_close_for_primary_hydrogen_and_plating_overflow() {
    half_reactions("SO4-2");
}

#[test]
fn chloralkali_half_reactions_close_for_primary_hydrogen_and_plating_overflow() {
    half_reactions("Cl-");
}

#[test]
fn production_closed_cells_preserve_atoms_and_charge_for_both_anodes() {
    for anion in ["Cl-", "SO4-2"] {
        for copper in [0.0, 0.0001] {
            let mut bench = Bench::new();
            bench.vessels[0] = electrolyte(anion, copper);
            bench
                .step(Operator::Seal {
                    vessel: VesselId(0),
                    headspace_volume: Liters(0.1),
                })
                .unwrap();
            let before = bench.vessel(VesselId(0)).unwrap().clone();
            let events = bench
                .step(Operator::Electrolyse {
                    vessel: VesselId(0),
                    amps: 1.0,
                    seconds: 60.0,
                })
                .unwrap();
            let after = bench.vessel(VesselId(0)).unwrap();
            assert!(events.iter().any(|e| matches!(e, Event::Electrolysed { coulombs, .. } if (*coulombs-60.0).abs()<1e-12)));
            assert!(
                !events.iter().any(|e| matches!(e, Event::GasEvolved { .. })),
                "closed products must remain in stock"
            );
            conserved(&before, after);
        }
    }
}

#[test]
fn pure_water_remains_outside_the_supporting_electrolyte_model() {
    let mut water = Vessel::new(VesselId(0), "beaker");
    water.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    let before = format!("{water:?}");
    assert!(displacement::electrolyse_solvent(&water, 1.0, 60.0).is_none());
    assert_eq!(format!("{water:?}"), before);
    let salt = electrolyte("SO4-2", 0.0);
    assert!(displacement::electrolyse_solvent(&salt, 1.0, 60.0).is_some());
}
