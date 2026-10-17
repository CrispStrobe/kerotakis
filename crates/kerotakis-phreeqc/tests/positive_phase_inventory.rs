#![cfg(feature = "engine")]
//! Regressions for reviewed finite-solid admission, not empirical calibration.
use kerotakis_core::{species, Equilibrator, Moles, Phase, SpeciesId, Vessel, VesselId};
use kerotakis_phreeqc::{PathOutcome, PhreeqcEquilibrator};

fn solid_iron_hydroxide(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "Fe(OH)2" && portion.phase == Phase::Solid)
        .map(|portion| portion.moles.0)
        .sum()
}

fn total_iron(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .map(|portion| {
            species::lookup(&portion.species)
                .and_then(|data| kerotakis_core::stoich::parse_formula(data.formula).ok())
                .map_or(0.0, |formula| {
                    formula.counts.get("Fe").copied().unwrap_or(0.0) * portion.moles.0
                })
        })
        .sum()
}

#[test]
fn reviewed_positive_foreign_solid_does_not_toggle_between_equilibrations() {
    let mut solver = PhreeqcEquilibrator::new().expect("native aqueous engine");
    let mut vessel = Vessel::new(VesselId(0), "finite hydroxide");
    vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("Fe(OH)2"), Moles(0.01), Phase::Solid);
    solver
        .equilibrate(&mut vessel)
        .expect("first finite equilibrium");
    // A macroscopic remainder is the regression's scope, not a measured Ksp.
    let baseline_solid = solid_iron_hydroxide(&vessel);
    assert!(
        baseline_solid > 0.005,
        "the supplied solid was treated as fully dissolved"
    );
    let baseline_ph = vessel.solution.as_ref().expect("aqueous readback").ph;
    let baseline_temperature = vessel.temperature.0;
    let baseline_enthalpy = vessel.enthalpy().0;
    let baseline_charge = vessel.solute_charge;
    let baseline_water = vessel.moles_of(&SpeciesId::new("water")).0;
    assert!((total_iron(&vessel) - 0.01).abs() < 1e-10);
    for _ in 0..4 {
        solver
            .equilibrate(&mut vessel)
            .expect("unchanged finite equilibrium");
        assert!(
            (total_iron(&vessel) - 0.01).abs() < 1e-10,
            "iron inventory changed"
        );
        assert!(
            (solid_iron_hydroxide(&vessel) - baseline_solid).abs() < 1e-8,
            "unchanged equilibrium erased or recreated the supplied solid"
        );
        assert!(
            (vessel.solution.as_ref().unwrap().ph - baseline_ph).abs() < 1e-5,
            "pH depends on whether the same solid entered as a positive or zero phase"
        );
        assert!((vessel.temperature.0 - baseline_temperature).abs() < 1e-6);
        assert!(
            (vessel.enthalpy().0 - baseline_enthalpy).abs() < 1e-4,
            "unchanged equilibrium changed the represented enthalpy"
        );
        assert!(
            (vessel.solute_charge - baseline_charge).abs() < 1e-10,
            "unchanged equilibrium changed the analytical charge baseline"
        );
        assert!(
            (vessel.moles_of(&SpeciesId::new("water")).0 - baseline_water).abs() < 1e-8,
            "unchanged equilibrium changed the canonical solvent inventory"
        );
    }
    let compared = solver.compare_paths(&vessel);
    let wateq = compared
        .iter()
        .find(|path| path.dataset.starts_with("wateq4f.dat"))
        .expect("the default dataset supplies a comparison result");
    match &wateq.outcome {
        PathOutcome::Solved { ph, phases, .. } => {
            assert!(
                (*ph - baseline_ph).abs() < 1e-5,
                "same-dataset comparison disagrees with the live solution"
            );
            let retained: f64 = phases
                .iter()
                .filter(|(name, _)| {
                    kerotakis_phreeqc::derived::phase_by_name(name)
                        .is_some_and(|phase| phase.species == "Fe(OH)2")
                })
                .map(|(_, amount)| *amount)
                .sum();
            assert!(
                (retained - baseline_solid).abs() < 1e-8,
                "comparison omitted finite hydroxide inventory"
            );
        }
        other => panic!("reviewed hydroxide comparison refused unexpectedly: {other:?}"),
    }
}
