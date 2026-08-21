//! AQ-009: compare time integration without conflating it with speciation.
//!
//! The rate program is project-authored and deliberately first order in the
//! remaining calcite inventory. PHREEQC may speciate the dissolved calcium
//! and carbonate however its approved database requires; the shared state
//! contract is remaining mineral plus analytical Ca/C totals.

#![cfg(all(
    feature = "engine",
    feature = "my-basic-preview",
    not(feature = "legacy-basic-oracle")
))]

use kerotakis_core::kinetics::{
    advance_network, lint_network, KineticReaction, Locality, OrderTerm, RateExpression, RateLaw,
    ReactionNetwork, StoichiometricTerm, Uncertainty, Validity,
};
use kerotakis_core::{Moles, Phase, SpeciesId, Vessel, VesselId};
use kerotakis_phreeqc::{databases, Phreeqc};

const INITIAL_MOLES: f64 = 0.01;
const RATE_CONSTANT_PER_SECOND: f64 = 0.4;
const SAMPLE_TIMES_SECONDS: &[f64] = &[0.0, 0.25, 0.5, 1.0, 2.0];
const CALCITE_RATE: &str = "KeroCalciteDissolution";

const STOICHIOMETRY: &[StoichiometricTerm<'static>] = &[
    StoichiometricTerm {
        species: "CaCO3",
        coefficient: -1.0,
        phase: Phase::Solid,
    },
    // This is an analytical dissolved pool. PHREEQC owns its aqueous
    // speciation; the reaction-network side owns only the conserved formula
    // amount needed to compare the integrators.
    StoichiometricTerm {
        species: "CaCO3",
        coefficient: 1.0,
        phase: Phase::Aqueous,
    },
];

const ORDERS: &[OrderTerm<'static>] = &[OrderTerm {
    species: "CaCO3",
    phase: Some(Phase::Solid),
    order: 1.0,
}];

const REACTION: KineticReaction<'static> = KineticReaction {
    id: "aq009-calcite-dissolution",
    equation: "CaCO3(s) -> CaCO3(aq)",
    stoichiometry: STOICHIOMETRY,
    locality: Locality::Interface {
        from: Phase::Solid,
        to: Phase::Aqueous,
    },
    forward: RateExpression {
        arrhenius: RateLaw {
            pre_exponential: RATE_CONSTANT_PER_SECOND,
            temperature_exponent: 0.0,
            activation_energy: 0.0,
        },
        orders: ORDERS,
    },
    reverse: None,
    equilibrium: None,
    pressure_dependence: None,
    catalysts: &[],
    sites: &[],
    electrons: 0.0,
    validity: Validity {
        temperature_k: None,
        pressure_pa: None,
        note: "project-authored AQ-009 integrator comparison",
    },
    uncertainty: Uncertainty {
        relative: Some(0.0),
        note: "exact comparison parameter, not a measured calcite rate",
    },
    source_ids: &["kerotakis:test:aq-009"],
    provenance: "project-authored numerical comparison; not a physical calcite rate claim",
};

const NETWORK: ReactionNetwork<'static> = ReactionNetwork {
    id: "aq009-mineral-dissolution",
    reactions: &[REACTION],
};

#[derive(Debug, Clone, Copy)]
struct TrajectoryPoint {
    elapsed_seconds: f64,
    remaining_moles: f64,
    dissolved_calcium_moles: f64,
    dissolved_carbon_moles: f64,
}

fn phase_moles(vessel: &Vessel, species: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == species && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

fn kerotakis_trajectory() -> Vec<TrajectoryPoint> {
    let mut vessel = Vessel::new(VesselId(0), "AQ-009 calcite dissolution");
    // Exactly one kilogram on the registry's molar-mass convention. The
    // first-order extent is independent of this volume, but an explicit
    // solvent keeps the locality and dimensional contract real.
    vessel.deposit(
        SpeciesId::new("water"),
        Moles(1_000.0 / 18.015_28),
        Phase::Liquid,
    );
    vessel.deposit(SpeciesId::new("CaCO3"), Moles(INITIAL_MOLES), Phase::Solid);

    let mut previous_time = 0.0;
    SAMPLE_TIMES_SECONDS
        .iter()
        .map(|&elapsed_seconds| {
            advance_network(&mut vessel, elapsed_seconds - previous_time, &NETWORK).unwrap();
            previous_time = elapsed_seconds;
            let remaining_moles = phase_moles(&vessel, "CaCO3", Phase::Solid);
            let dissolved = phase_moles(&vessel, "CaCO3", Phase::Aqueous);
            TrajectoryPoint {
                elapsed_seconds,
                remaining_moles,
                dissolved_calcium_moles: dissolved,
                dissolved_carbon_moles: dissolved,
            }
        })
        .collect()
}

fn phreeqc_point(elapsed_seconds: f64) -> TrajectoryPoint {
    if elapsed_seconds == 0.0 {
        return TrajectoryPoint {
            elapsed_seconds,
            remaining_moles: INITIAL_MOLES,
            dissolved_calcium_moles: 0.0,
            dissolved_carbon_moles: 0.0,
        };
    }

    let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
    let input = format!(
        "RATES\n\
             {CALCITE_RATE}\n\
             -start\n\
             10 dissolved = PARM(1) * M * TIME\n\
             20 IF dissolved > M THEN dissolved = M\n\
             30 SAVE dissolved\n\
             -end\n\
         SOLUTION 1\n\
             temp 25\n\
             units mol/kgw\n\
             pH 7\n\
         KINETICS 1\n\
             {CALCITE_RATE}\n\
                 -formula CaCO3 1\n\
                 -m {INITIAL_MOLES}\n\
                 -m0 {INITIAL_MOLES}\n\
                 -parms {RATE_CONSTANT_PER_SECOND}\n\
                 -tol 1e-12\n\
                 -steps {elapsed_seconds} seconds\n\
         SELECTED_OUTPUT\n\
             -reset false\n\
             -high_precision true\n\
         USER_PUNCH\n\
             -headings aq009_elapsed_s aq009_remaining_moles aq009_calcium_molality aq009_carbon_molality\n\
             10 PUNCH TOTAL_TIME, KIN(\"{CALCITE_RATE}\"), TOT(\"Ca\"), TOT(\"C\")\n\
         END\n"
    );
    engine.run(&input).unwrap_or_else(|error| {
        panic!("AQ-009 PHREEQC point at {elapsed_seconds}s failed:\n{error}")
    });

    let value = |heading| {
        engine
            .last_value(heading)
            .unwrap_or_else(|| panic!("missing AQ-009 selected-output column {heading}"))
    };
    TrajectoryPoint {
        elapsed_seconds: value("aq009_elapsed_s"),
        remaining_moles: value("aq009_remaining_moles"),
        // PHREEQC's default solution contains one kilogram of water, so
        // molality is numerically the analytical amount in this closed case.
        dissolved_calcium_moles: value("aq009_calcium_molality"),
        dissolved_carbon_moles: value("aq009_carbon_molality"),
    }
}

fn relative_error(actual: f64, expected: f64) -> f64 {
    (actual - expected).abs() / expected.abs().max(1e-15)
}

#[test]
fn phreeqc_and_kerotakis_follow_the_same_mineral_dissolution_trajectory() {
    lint_network(&NETWORK).unwrap();
    let kerotakis = kerotakis_trajectory();
    let phreeqc = SAMPLE_TIMES_SECONDS
        .iter()
        .map(|&time| phreeqc_point(time))
        .collect::<Vec<_>>();

    let mut max_cross_engine_relative_error: f64 = 0.0;
    for ((ours, theirs), &requested_time) in
        kerotakis.iter().zip(&phreeqc).zip(SAMPLE_TIMES_SECONDS)
    {
        let analytic_remaining = INITIAL_MOLES * (-RATE_CONSTANT_PER_SECOND * requested_time).exp();
        let analytic_dissolved = INITIAL_MOLES - analytic_remaining;

        assert!((ours.elapsed_seconds - requested_time).abs() < 1e-12);
        assert!((theirs.elapsed_seconds - requested_time).abs() < 1e-10);
        assert!(
            relative_error(ours.remaining_moles, analytic_remaining) < 2e-6,
            "Kerotakis at {requested_time}s: {ours:?}, analytic={analytic_remaining}"
        );
        assert!(
            relative_error(theirs.remaining_moles, analytic_remaining) < 5e-5,
            "PHREEQC at {requested_time}s: {theirs:?}, analytic={analytic_remaining}"
        );
        assert!(
            (ours.dissolved_calcium_moles - analytic_dissolved).abs() < 2e-8,
            "Kerotakis Ca ledger at {requested_time}s: {ours:?}"
        );
        assert!(
            (ours.dissolved_carbon_moles - analytic_dissolved).abs() < 2e-8,
            "Kerotakis C ledger at {requested_time}s: {ours:?}"
        );
        assert!(
            (theirs.dissolved_calcium_moles - analytic_dissolved).abs() < 5e-7,
            "PHREEQC Ca ledger at {requested_time}s: {theirs:?}"
        );
        assert!(
            (theirs.dissolved_carbon_moles - analytic_dissolved).abs() < 5e-7,
            "PHREEQC C ledger at {requested_time}s: {theirs:?}"
        );
        max_cross_engine_relative_error = max_cross_engine_relative_error
            .max(relative_error(ours.remaining_moles, theirs.remaining_moles));
    }

    assert!(
        max_cross_engine_relative_error < 5e-5,
        "maximum remaining-mineral trajectory error was {max_cross_engine_relative_error:.3e}"
    );
}
