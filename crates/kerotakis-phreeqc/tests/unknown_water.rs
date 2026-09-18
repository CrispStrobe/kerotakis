//! The unknown-water investigation (INF-002).
//!
//! A hidden sample is prepared on the real bench through the standard
//! solver stack. The learner-facing side holds only serialisable
//! measurements and hypotheses — never the composition — and inference
//! runs through [`kerotakis_core::inference`].
//!
//! # Forward-model structure
//!
//! Each hypothesis is a TABULATED forward model: one shared PHREEQC
//! engine evaluates the bench at every grid point exactly once, up front.
//! Fitting then runs over the table. Rebuilding an engine per grid point
//! would be both wasteful and a correctness trap — an engine failure must
//! fail the table build loudly rather than degrade silently into an empty
//! prediction (which the fitter would read as a perfect fit at chi² 0).
//!
//! Synthetic and real-bench rows below validate the inference software on
//! this bench. They are not independent evidence about the world.

#![cfg(feature = "engine")]

use kerotakis_core::inference::{
    fit_two_parameters, Hypothesis, HypothesisFit, InferenceError, Observation,
};
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

/// The two candidate identities the learner is offered. Both are one
/// solute plus water, so one two-parameter fit (concentration, dilution)
/// covers each — the first slice fits two parameters, not four.
#[allow(dead_code)]
const HYPOTHESES: [&str; 2] = ["NaCl", "sucrose"];

/// One forward evaluation of a candidate composition through the REAL
/// stack, reduced to the two observables the learner's instruments read.
fn forward_observables(
    eq: &mut PhreeqcEquilibrator,
    species: &str,
    concentration_g_per_l: f64,
    dilution: f64,
) -> Vec<(&'static str, f64)> {
    let mut bench = Bench::new();
    let v = VesselId(0);
    for (key, moles) in [
        ("water", 55.51 / dilution.max(1.0)),
        (species, concentration_g_per_l / 58.44),
    ] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                eq,
                &ReactiveGroupScreen,
            )
            .expect("forward-model step");
    }
    let vessel = bench.vessel(v).unwrap();
    let ph = vessel
        .solution
        .as_ref()
        .map(|s| s.ph)
        .expect("forward model characterised");
    let reading = kerotakis_core::instrument::ConductivityMeter
        .measure(&vessel)
        .expect("forward model reads conductivity");
    vec![("pH", ph), ("conductivity_uS_per_cm", reading.value)]
}

/// A hypothesis whose forward model was evaluated once, on a grid, by a
/// shared engine. `predict` reads the nearest grid point, so the fitter's
/// grid must use the same resolution the table was built with.
struct TabulatedHypothesis {
    name: &'static str,
    concentration_g_per_l: (f64, f64),
    dilution: (f64, f64),
    steps: usize,
    table: Vec<Vec<Vec<(&'static str, f64)>>>,
}

impl TabulatedHypothesis {
    fn build(
        eq: &mut PhreeqcEquilibrator,
        name: &'static str,
        concentration_g_per_l: (f64, f64),
        dilution: (f64, f64),
        steps: usize,
    ) -> Self {
        let mut table = Vec::with_capacity(steps);
        for step_a in 0..steps {
            let concentration = concentration_g_per_l.0
                + (concentration_g_per_l.1 - concentration_g_per_l.0) * step_a as f64
                    / (steps - 1) as f64;
            let mut row = Vec::with_capacity(steps);
            for step_b in 0..steps {
                let dilution =
                    dilution.0 + (dilution.1 - dilution.0) * step_b as f64 / (steps - 1) as f64;
                row.push(forward_observables(eq, name, concentration, dilution));
            }
            table.push(row);
        }
        Self {
            name,
            concentration_g_per_l,
            dilution,
            steps,
            table,
        }
    }

    fn index_of(&self, value: f64, (lo, hi): (f64, f64)) -> usize {
        let fraction = (value - lo) / (hi - lo);
        ((fraction * (self.steps - 1) as f64).round() as usize).min(self.steps - 1)
    }
}

impl Hypothesis for TabulatedHypothesis {
    fn name(&self) -> &str {
        self.name
    }
    fn bounds(&self) -> ((f64, f64), (f64, f64)) {
        (self.concentration_g_per_l, self.dilution)
    }
    fn predict(&self, a: f64, b: f64) -> Vec<(&'static str, f64)> {
        let i = self.index_of(a, self.concentration_g_per_l);
        let j = self.index_of(b, self.dilution);
        self.table[i][j].clone()
    }
}

/// What the learner's notebook holds: two calibrated observations and
/// nothing about the composition.
fn learner_observations(bench: &Bench, v: VesselId) -> Vec<Observation> {
    let vessel = bench.vessel(v).unwrap();
    let mut observations = Vec::new();
    if let Some(solution) = &vessel.solution {
        observations.push(Observation::new("pH", solution.ph, 0.05).expect("finite pH"));
    }
    if let Some(reading) = kerotakis_core::instrument::ConductivityMeter.measure(&vessel) {
        observations.push(
            Observation::new(
                "conductivity_uS_per_cm",
                reading.value,
                (reading.value * 0.02).max(1.0),
            )
            .expect("finite conductivity"),
        );
    }
    observations
}

fn prepare_hidden_bench(species: &str, grams_per_litre: f64) -> (Bench, VesselId) {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    for (key, moles) in [("water", 55.51), (species, grams_per_litre / 58.44)] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut eq,
                &ReactiveGroupScreen,
            )
            .expect("hidden-sample step");
    }
    (bench, v)
}

#[test]
fn the_real_bench_distinguishes_salt_from_sucrose() {
    // The hidden sample: 10 g/L NaCl in water. The learner sees only the
    // observations below — never the species name.
    let (bench, v) = prepare_hidden_bench("NaCl", 10.0);
    let observations = learner_observations(&bench, v);
    assert_eq!(observations.len(), 2, "pH and conductivity both read");

    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let salt = TabulatedHypothesis::build(&mut eq, "NaCl", (0.01, 50.0), (1.0, 10.0), 9);
    let sugar = TabulatedHypothesis::build(&mut eq, "sucrose", (0.01, 200.0), (1.0, 10.0), 9);
    let salt_fit = fit_two_parameters(&salt, &observations, 9).expect("salt fit");
    let sugar_fit = fit_two_parameters(&sugar, &observations, 9).expect("sugar fit");
    // The real electrolyte fits dramatically better than the non-electrolyte.
    assert!(
        salt_fit.chi_squared < sugar_fit.chi_squared,
        "salt {salt_fit:?} vs sugar {sugar_fit:?}"
    );
}

#[test]
fn ambiguous_wide_sigma_data_do_not_pretend_to_discriminate() {
    // Sigma as wide as the signal: neither identity is identifiable, and
    // both fits must say so rather than a confident wrong answer.
    let observations = vec![
        Observation::new("pH", 6.8, 0.5).unwrap(),
        Observation::new("conductivity_uS_per_cm", 2000.0, 2000.0).unwrap(),
    ];
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let salt = TabulatedHypothesis::build(&mut eq, "NaCl", (0.01, 50.0), (1.0, 10.0), 5);
    let sugar = TabulatedHypothesis::build(&mut eq, "sucrose", (0.01, 200.0), (1.0, 10.0), 5);
    let salt_fit = fit_two_parameters(&salt, &observations, 5).unwrap();
    let sugar_fit = fit_two_parameters(&sugar, &observations, 5).unwrap();
    assert!(!salt_fit.identifiable, "{salt_fit:?}");
    assert!(!sugar_fit.identifiable, "{sugar_fit:?}");
}

#[test]
fn a_hypothesis_that_cannot_predict_an_observation_is_refused() {
    struct Impossible;
    impl Hypothesis for Impossible {
        fn name(&self) -> &str {
            "impossible"
        }
        fn bounds(&self) -> ((f64, f64), (f64, f64)) {
            ((0.0, 1.0), (0.0, 1.0))
        }
        fn predict(&self, _a: f64, _b: f64) -> Vec<(&'static str, f64)> {
            vec![("something-else", 1.0)]
        }
    }
    let observations = vec![Observation::new("pH", 7.0, 0.1).unwrap()];
    assert!(matches!(
        fit_two_parameters(&Impossible, &observations, 5),
        Err(InferenceError::UnknownObservable { index: 0, .. })
    ));
}

#[allow(dead_code)]
fn unused_type_witness(_fit: &HypothesisFit) {}
