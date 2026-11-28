//! Hydroxide precipitates redissolving in excess complexing reagent,
//! table-driven (2026-11-27, table form 2026-11-28).
//!
//! A metal hydroxide thrown down by a little base goes back into solution
//! when the excess reagent is itself a complexing ligand: copper(II) and
//! zinc hydroxides in excess ammonia (tetraammines; the copper one deep
//! blue, the zinc one colourless), zinc and aluminium hydroxides in excess
//! lye (zincate Zn(OH)4 2-, aluminate Al(OH)4-, both colourless).
//!
//! Nothing here is a scenario yield and nothing is per-metal code: the
//! routed databases' own mass action decides, and the fresh precipitate is
//! the databases' amorphous hydroxide (`derived::fresh_precipitate_rank`),
//! not an aged crystal. A new metal is one row in [`CASES`]; its complex
//! and hydroxide must already be in a routed database (wateq4f.dat /
//! minteq.v4.dat), never a constant written here.
//!
//! Negative rows: too little reagent (bounded by the complex's own
//! stoichiometry, e.g. at most n_NH3 / 4 of the copper can become
//! Cu(NH3)4+2), and metals that form no ammine (iron(III), aluminium) stay
//! down in excess ammonia.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// What the excess reagent must do to the precipitate.
#[derive(Debug, Clone, Copy)]
enum Outcome {
    /// At most `left_max` mol of the solid survives, and the named complex
    /// reaches at least `min_molality`.
    Dissolves {
        left_max: f64,
        complex: &'static str,
        min_molality: f64,
    },
    /// At least `down_min` mol of the solid survives.
    Stays { down_min: f64 },
}

#[derive(Debug, Clone, Copy)]
struct Case {
    name: &'static str,
    water: &'static str,
    /// The metal salt and the base that throws the hydroxide down.
    salt: (&'static str, f64),
    precipitant: (&'static str, f64),
    /// The second, "excess" (or deliberately short) addition.
    reagent: (&'static str, f64),
    solid: &'static str,
    metal: &'static str,
    metal_mol: f64,
    /// Precipitate-first check after salt + precipitant, if asserted.
    first_min: Option<f64>,
    outcome: Outcome,
    /// A word the final appearance must contain.
    colour: Option<&'static str>,
}

const CASES: &[Case] = &[
    // --- redissolves --------------------------------------------------
    Case {
        name: "Cu(OH)2 in excess NH3 -> deep blue tetraammine",
        water: "10mL",
        salt: ("CuSO4", 0.0001),
        precipitant: ("NaOH", 0.0002),
        reagent: ("NH3", 0.03),
        solid: "Cu(OH)2",
        metal: "Cu",
        metal_mol: 0.0001,
        first_min: Some(0.9e-4),
        // ~0.01 mol/kgw of copper, most of it the tetraammine.
        outcome: Outcome::Dissolves {
            left_max: 1e-6,
            complex: "Cu(NH3)4+2",
            min_molality: 0.005,
        },
        colour: Some("blue"),
    },
    Case {
        name: "Zn(OH)2 in excess NH3 -> colourless tetraammine",
        water: "10mL",
        salt: ("ZnSO4", 0.0001),
        precipitant: ("NaOH", 0.0002),
        reagent: ("NH3", 0.02),
        solid: "Zn(OH)2",
        metal: "Zn",
        metal_mol: 0.0001,
        first_min: None,
        // Most of the 100 umol redissolves (the activity model gives ~95%).
        outcome: Outcome::Dissolves {
            left_max: 1e-5,
            complex: "Zn(NH3)4+2",
            min_molality: 0.003,
        },
        colour: Some("colourless"),
    },
    Case {
        name: "Zn(OH)2 in excess NaOH -> colourless zincate",
        water: "7mL",
        salt: ("ZnSO4", 0.0002),
        precipitant: ("NaOH", 0.0004),
        reagent: ("NaOH", 0.005),
        solid: "Zn(OH)2",
        metal: "Zn",
        metal_mol: 0.0002,
        first_min: Some(1.5e-4),
        // Most of the 200 umol goes to zincate in ~0.7 M NaOH (~80%).
        outcome: Outcome::Dissolves {
            left_max: 1e-4,
            complex: "Zn(OH)4-2",
            min_molality: 0.01,
        },
        colour: Some("colourless"),
    },
    Case {
        name: "Al(OH)3 in excess NaOH -> colourless aluminate",
        water: "7mL",
        salt: ("Al2(SO4)3", 0.0001),
        precipitant: ("NaOH", 0.0006),
        reagent: ("NaOH", 0.005),
        solid: "Al(OH)3",
        metal: "Al",
        metal_mol: 0.0002,
        first_min: Some(1.5e-4),
        // Aluminate ceiling ~126 x [OH-] (ni_al_hydroxide.rs docs).
        outcome: Outcome::Dissolves {
            left_max: 1e-6,
            complex: "Al(OH)4-",
            min_molality: 0.02,
        },
        colour: Some("colourless"),
    },
    // --- too little reagent ------------------------------------------
    Case {
        // 0.0001 mol NH3 can carry at most 25 umol of Cu as Cu(NH3)4+2.
        name: "Cu(OH)2 with too little NH3 stays",
        water: "10mL",
        salt: ("CuSO4", 0.0001),
        precipitant: ("NaOH", 0.0002),
        reagent: ("NH3", 0.0001),
        solid: "Cu(OH)2",
        metal: "Cu",
        metal_mol: 0.0001,
        first_min: Some(0.9e-4),
        outcome: Outcome::Stays { down_min: 0.6e-4 },
        colour: None,
    },
    Case {
        // 0.00005 mol OH- can carry at most 25 umol as Zn(OH)4-2.
        name: "Zn(OH)2 with too little extra NaOH stays",
        water: "7mL",
        salt: ("ZnSO4", 0.0002),
        precipitant: ("NaOH", 0.0004),
        reagent: ("NaOH", 0.00005),
        solid: "Zn(OH)2",
        metal: "Zn",
        metal_mol: 0.0002,
        first_min: Some(1.5e-4),
        outcome: Outcome::Stays { down_min: 1.2e-4 },
        colour: None,
    },
    Case {
        // 0.00005 mol OH- can carry at most 50 umol as Al(OH)4-.
        name: "Al(OH)3 with too little extra NaOH stays",
        water: "7mL",
        salt: ("Al2(SO4)3", 0.0001),
        precipitant: ("NaOH", 0.0006),
        reagent: ("NaOH", 0.00005),
        solid: "Al(OH)3",
        metal: "Al",
        metal_mol: 0.0002,
        first_min: Some(1.5e-4),
        outcome: Outcome::Stays { down_min: 1.0e-4 },
        colour: None,
    },
    // --- non-complexing metal -----------------------------------------
    Case {
        name: "Fe(OH)3 forms no ammine and stays in excess NH3",
        water: "10mL",
        salt: ("FeCl3", 0.0001),
        precipitant: ("NH3", 0.0003),
        reagent: ("NH3", 0.03),
        solid: "Fe(OH)3",
        metal: "Fe",
        metal_mol: 0.0001,
        first_min: None,
        outcome: Outcome::Stays { down_min: 0.99e-4 },
        colour: None,
    },
];

fn run(script: &[String]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
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
                .unwrap_or_else(|e| panic!("{op}: {e}")),
        );
    }
    (bench, steps)
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn solid(bench: &Bench, key: &str) -> f64 {
    vessel(bench)
        .contents
        .iter()
        .filter(|p| p.species == SpeciesId::new(key) && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

fn molality(bench: &Bench, name: &str) -> f64 {
    vessel(bench)
        .solution
        .as_ref()
        .map(|s| {
            s.species
                .iter()
                .filter(|x| x.name == name)
                .map(|x| x.molality)
                .sum()
        })
        .unwrap_or(0.0)
}

fn element(bench: &Bench, el: &str) -> f64 {
    kerotakis_core::ledger::ConservedLedger::from_vessel(vessel(bench))
        .elements
        .get(el)
        .copied()
        .unwrap_or(0.0)
}

/// Every failed expectation of one case, empty when it passes.
fn check(c: &Case) -> Vec<String> {
    let mut bad = Vec::new();
    let prefix = vec![
        format!("add v1 water {}", c.water),
        format!("add v1 {} {}mol", c.salt.0, c.salt.1),
        format!("add v1 {} {}mol", c.precipitant.0, c.precipitant.1),
    ];
    if let Some(min) = c.first_min {
        let (before, _) = run(&prefix);
        let first = solid(&before, c.solid);
        if first <= min {
            bad.push(format!("precipitate first: {} {first} <= {min}", c.solid));
        }
    }
    let mut full = prefix;
    full.push(format!("add v1 {} {}mol", c.reagent.0, c.reagent.1));
    let (bench, steps) = run(&full);
    let left = solid(&bench, c.solid);
    match c.outcome {
        Outcome::Dissolves {
            left_max,
            complex,
            min_molality,
        } => {
            if left >= left_max {
                bad.push(format!("{} left {left} >= {left_max}", c.solid));
            }
            let m = molality(&bench, complex);
            if m <= min_molality {
                bad.push(format!("{complex} {m} <= {min_molality}"));
            }
        }
        Outcome::Stays { down_min } => {
            if left <= down_min {
                bad.push(format!("{} down {left} <= {down_min}", c.solid));
            }
        }
    }
    let total = element(&bench, c.metal);
    if (total - c.metal_mol).abs() >= 1e-9 {
        bad.push(format!("{} ledger {total} != {}", c.metal, c.metal_mol));
    }
    if let Some(word) = c.colour {
        let look = kerotakis_core::appearance::observe(vessel(&bench));
        if !look.words.contains(word) {
            bad.push(format!("appearance {:?} lacks {word:?}", look.words));
        }
    }
    if !bad.is_empty() {
        bad.push(format!("steps: {steps:#?}"));
    }
    bad
}

#[test]
fn hydroxide_redissolution_table() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|c| {
            let bad = check(c);
            (!bad.is_empty()).then(|| format!("{}:\n  {}", c.name, bad.join("\n  ")))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Every table row that expects redissolution names a complex that a routed
/// database or a reviewed shipped extension (`data/thermo`, appended by
/// `complexation`) actually forms — the table cannot paper over missing data.
#[test]
fn every_dissolving_row_names_a_shipped_complex() {
    const SOURCES: &[&str] = &[
        "vendor/iphreeqc/database/wateq4f.dat",
        "vendor/iphreeqc/database/minteq.v4.dat",
        "data/thermo/usbm-ic9429-complexes.dat",
        "data/thermo/shipped-complex-borrows.dat",
        "data/thermo/shipped-complex-borrows-minteq.dat",
    ];
    let root = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let text: String = SOURCES
        .iter()
        .map(|s| {
            std::fs::read_to_string(format!("{root}/{s}")).unwrap_or_else(|e| panic!("{s}: {e}"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    for c in CASES {
        if let Outcome::Dissolves { complex, .. } = c.outcome {
            let found = text.lines().any(|l| {
                l.split_once('=')
                    .is_some_and(|(_, rhs)| rhs.split_whitespace().next() == Some(complex))
            });
            assert!(found, "{}: no shipped reaction forms {complex}", c.name);
        }
    }
}
