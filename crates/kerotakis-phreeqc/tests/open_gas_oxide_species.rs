//! Three reaction-sweep gaps (2026-11-21):
//!
//! - H6: a gas a curated rate law makes in an OPEN vessel goes through the
//!   open boundary. Thiosulfate + HCl used to leave 2.9 mmol of SO2 held as
//!   a Gas portion inside an open beaker; it now dissolves below its bubble
//!   threshold (SO2 is very soluble); a sealed vessel keeps it.
//! - H7: a basic oxide or hydroxide dissolving in acid is told as a
//!   reaction with its derived net ionic equation.
//! - H3: potassium dichromate, potassium chromate and sodium nitrite are on
//!   the shelf.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn run(script: &[&str]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let mut steps = Vec::new();
    for op in script {
        steps.push(
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .unwrap_or_else(|e| panic!("{op}: {e:?}")),
        );
    }
    (bench, steps)
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn phase_moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    vessel(bench)
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

/// Sulfur in every form the vessel can book it.
fn sulfur(bench: &Bench) -> f64 {
    vessel(bench)
        .contents
        .iter()
        .map(|p| {
            let per = match p.species.0.as_str() {
                "S2O3-2" | "Na2S2O3" | "HS2O3-" => 2.0,
                "S" | "SO2" | "HSO3-" | "SO3-2" | "SO4-2" | "HSO4-" | "H2S(aq)" | "HS-" | "S-2"
                | "hydrogen_sulfide" | "NaSO4-" | "NaS2O3-" => 1.0,
                _ => 0.0,
            };
            per * p.moles.0
        })
        .sum()
}

fn evolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == key => Some(moles.0),
            _ => None,
        })
        .sum()
}

const THIOSULFATE: [&str; 4] = [
    "add v1 water 100mL",
    "add v1 Na2S2O3 0.005mol",
    "add v1 HCl 0.01mol",
    "wait 60s",
];

#[test]
fn h6_thiosulfate_sulfur_dioxide_does_not_hang_in_an_open_beaker() {
    let (bench, steps) = run(&THIOSULFATE);
    let made = phase_moles(&bench, "S", Phase::Solid);
    assert!(made > 1e-3, "the reaction ran: {made}\n{:#?}", steps[3]);
    assert!(
        phase_moles(&bench, "SO2", Phase::Gas) < 1e-12,
        "no SO2 is left as a gas portion over an open beaker: {:#?}",
        vessel(&bench).contents
    );
    // 2.9 mmol in 0.1 L is far below the 0.12 mol a litre-atmosphere of
    // SO2 would need, so none of it bubbles out.
    assert!(evolved(&steps[3], "SO2") < 1e-12, "{:#?}", steps[3]);
    assert!(
        (sulfur(&bench) - 0.010).abs() < 1e-9,
        "sulfur is conserved: {}\n{:#?}",
        sulfur(&bench),
        vessel(&bench).contents
    );
}

#[test]
fn h6_a_sealed_vessel_keeps_the_sulfur_dioxide_it_makes() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 Na2S2O3 0.005mol",
        "seal v1 500mL",
        "add v1 HCl 0.01mol",
        "wait 60s",
    ]);
    assert!(evolved(&steps[4], "SO2") < 1e-12, "{:#?}", steps[4]);
    assert!((sulfur(&bench) - 0.010).abs() < 1e-9, "{}", sulfur(&bench));
}

#[test]
fn h6_peroxide_oxygen_is_outside_this_change() {
    // O2's open-boundary handling is a separate gap (see
    // `volatility::OPEN_BOUNDARY_UPTAKE`); this pins that the SO2 change
    // did not move it.
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 H2O2 0.05mol",
        "add v1 KI 0.001mol",
        "wait 60s",
    ]);
    assert!(evolved(&steps[3], "O2") < 1e-12, "{:#?}", steps[3]);
    assert!(phase_moles(&bench, "O2", Phase::Gas) > 1e-5);
}

fn equations(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::ReactionOccurred { equation, .. } => Some(equation.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn h7_copper_oxide_in_sulfuric_acid_is_told_as_a_reaction() {
    let (bench, steps) = run(&[
        "add v1 water 100mL",
        "add v1 CuO 0.005mol",
        "add v1 H2SO4 0.01mol",
    ]);
    let told = equations(&steps[2]);
    assert!(
        told.iter()
            .any(|e| e == "CuO(s) + 2 H⁺(aq) → Cu²⁺(aq) + H₂O(l)"),
        "{told:?}\n{:#?}",
        steps[2]
    );
    assert!(phase_moles(&bench, "CuO", Phase::Solid) < 1e-9);
}

#[test]
fn h7_copper_hydroxide_in_hydrochloric_acid_makes_two_waters() {
    let (_, steps) = run(&[
        "add v1 water 100mL",
        "add v1 Cu(OH)2 0.002mol",
        "add v1 HCl 0.01mol",
    ]);
    let told = equations(&steps[2]);
    assert!(
        told.iter()
            .any(|e| e == "Cu(OH)₂(s) + 2 H⁺(aq) → Cu²⁺(aq) + 2 H₂O(l)"),
        "{told:?}\n{:#?}",
        steps[2]
    );
}

#[test]
fn h7_a_salt_dissolving_in_water_is_not_called_an_acid_reaction() {
    let (_, steps) = run(&["add v1 water 100mL", "add v1 NaCl 0.01mol"]);
    assert!(equations(&steps[1]).is_empty(), "{:#?}", steps[1]);
    let (_, steps) = run(&[
        "add v1 water 100mL",
        "add v1 HCl 0.01mol",
        "add v1 CuSO4 0.002mol",
    ]);
    assert!(equations(&steps[2]).is_empty(), "{:#?}", steps[2]);
}

#[test]
fn h3_the_new_salts_are_on_the_shelf_and_dissolve() {
    for (key, name, mass) in [
        ("K2Cr2O7", "potassium dichromate", 294.1818),
        ("K2CrO4", "potassium chromate", 194.1887),
        ("NaNO2", "sodium nitrite", 68.9948),
    ] {
        let data = species::lookup_key(key).unwrap_or_else(|| panic!("{key} missing"));
        assert_eq!(data.name, name);
        assert_eq!(data.standard_phase, Phase::Solid, "{key}");
        assert!((data.molar_mass - mass).abs() < 1e-3, "{key}");
    }
    let (bench, _) = run(&["add v1 water 100mL", "add v1 NaNO2 0.001mol"]);
    assert!(phase_moles(&bench, "NaNO2", Phase::Solid) < 1e-12);
    assert!(
        (vessel(&bench).moles_of(&SpeciesId::new("NO2-")).0 - 0.001).abs() < 1e-6,
        "{:#?}",
        vessel(&bench).contents
    );
    let (bench, _) = run(&["add v1 water 100mL", "add v1 K2Cr2O7 0.001mol"]);
    assert!(
        phase_moles(&bench, "K2Cr2O7", Phase::Solid) < 1e-12,
        "0.29 g is well inside 13 g/100 mL: {:#?}",
        vessel(&bench).contents
    );
}

#[test]
fn h3_a_dichromate_beyond_its_solubility_stays_partly_solid() {
    // The two-point curve (13.2 g/100 mL at 20 C, 102 at 100 C) gives
    // 13.2 + 5/80 x 88.8 = 18.75 g at 25 C: 0.0637 mol of the 0.1 mol
    // (29.4 g) dissolves and about 0.036 mol stays on the bottom.
    let (bench, _) = run(&["add v1 water 100mL", "add v1 K2Cr2O7 0.1mol"]);
    let solid = phase_moles(&bench, "K2Cr2O7", Phase::Solid);
    assert!(
        (solid - 0.0363).abs() < 0.002,
        "{:#?}",
        vessel(&bench).contents
    );
}
