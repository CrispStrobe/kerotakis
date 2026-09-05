//! BRD-041 — the kinetic route may not walk past the thermodynamics.
//!
//! Two solvers answer the same question here by completely different
//! means. NASA-CEA minimises Gibbs energy over the species it has NASA-9
//! polynomials for and reports where the mixture *settles*; the BRD-041
//! mechanism packs integrate elementary rate laws and report where the
//! mixture is *going*. Neither knows about the other.
//!
//! The oracle is the relationship between them, and it is one-sided on
//! purpose. A kinetic network is allowed to be slow — that is what a rate
//! constant is for — but it is never allowed to produce more product than
//! equilibrium permits, and it is never allowed to consume more fuel. A
//! network that overshoots equilibrium has a thermodynamic inconsistency
//! in it (usually an irreversible step written where a reversible one
//! belongs), and this file is what says so.
//!
//! The packs' hydrogen chemistry is written as irreversible elementary
//! steps, so its own endpoint is complete conversion. That is honest only
//! where equilibrium says the same thing, which is why the conditions
//! chosen below sit well under the temperature at which water begins to
//! dissociate measurably. Above roughly 1800 K that agreement stops being
//! true, and the test states the bound rather than hiding it.

use std::collections::BTreeMap;
use std::path::Path;

use kerotakis_cea::{db, equilibrate_tp, Species};
use kerotakis_core::kinetics::mechanism::{parse_yaml, MechanismArena};
use kerotakis_core::kinetics::{advance_network_with_options, IntegrationOptions};
use kerotakis_core::species::Phase;
use kerotakis_core::units::{Kelvin, Liters, Moles};
use kerotakis_core::vessel::{Headspace, Vessel, VesselId};
use kerotakis_core::SpeciesId;

/// The conditions the agreement is claimed at, and only at.
const TEMPERATURE_K: f64 = 1200.0;
const PRESSURE_BAR: f64 = 1.0;
/// Half a percent of a two-mole water yield. Dissociation of water at
/// 1200 K and 1 bar is far below this; the tolerance is set by what
/// "essentially complete" means for a teaching claim, not by how well
/// the minimiser converges.
const COMPLETE_CONVERSION_TOLERANCE: f64 = 1e-2;
/// See `gas_mechanism_packs.rs`: the isothermal integrator has no spark,
/// so the chain is given one. Five parts in a hundred thousand of the
/// fuel, which is why the overshoot check below carries a thousandth of
/// relative slack rather than none.
const SEED_MOLES: f64 = 1e-7;

fn pack_text(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/mechanisms")
        .join(format!("{name}.yaml"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn pool(names: &[&str]) -> Vec<&'static Species> {
    names.iter().filter_map(|name| db().get(name)).collect()
}

fn budget(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), *value))
        .collect()
}

/// Integration controls for a stiff radical chain, and the window this
/// engine can carry one across. The reasoning, and the diagnosis of why
/// the window has to be bounded, is in
/// `crates/kerotakis-core/tests/gas_mechanism_packs.rs`; the short of it
/// is that the extent integrator probes its Jacobian with one scalar
/// finite difference for a state vector spanning nine orders of
/// magnitude, and gives up at about 2.7 µs on this network.
///
/// The one-sided oracle below is unaffected by that. "Kinetics may not
/// walk past equilibrium" is true at every instant, not only at the end,
/// so a bounded window tests it honestly — it just cannot also
/// demonstrate arrival, which is why the endpoint itself is established
/// against CEA rather than by integrating to it.
const STIFF: IntegrationOptions = IntegrationOptions {
    relative_tolerance: 1e-6,
    absolute_tolerance_moles: 1e-14,
    initial_step_seconds: 1e-9,
};

/// One interval of the bounded window, and how many of them.
const STEP_SECONDS: f64 = 1.0e-8;
const STEPS: usize = 50;

fn reactor(volume_litres: f64, temperature_k: f64, feeds: &[(&str, f64)]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "mechanism reactor");
    vessel.temperature = Kelvin(temperature_k);
    vessel.headspace = Headspace::Sealed {
        volume: Liters(volume_litres),
    };
    for (species, moles) in feeds {
        vessel.deposit(SpeciesId::new(species), Moles(*moles), Phase::Gas);
    }
    vessel.refresh_pressure();
    vessel
}

fn moles(vessel: &Vessel, species: &str) -> f64 {
    vessel.moles_of(&SpeciesId::new(species)).0
}

/// Every species the H₂/O₂ pack names is a species CEA can also price.
///
/// If this ever fails, the two solvers are no longer answering about the
/// same chemistry and every comparison below is meaningless.
#[test]
fn cea_carries_every_species_the_hydrogen_pack_names() {
    let mechanism = parse_yaml(&pack_text("h2-o2-skeletal-v1")).expect("the pack parses");
    for species in mechanism.species_names() {
        assert!(
            db().get(species).is_some(),
            "CEA has no thermochemistry for {species}, so the endpoint cannot be checked"
        );
    }
}

/// At 1200 K equilibrium IS complete combustion — which is the only
/// condition under which an irreversible skeletal set tells the truth
/// about where it ends.
#[test]
fn equilibrium_at_the_claimed_conditions_is_complete_combustion() {
    let species = pool(&["H2", "O2", "H2O", "H", "O", "OH", "HO2", "H2O2", "N2"]);
    let equilibrium = equilibrate_tp(
        // 2 H2 + O2, plus nitrogen as a diluent.
        &budget(&[("H", 4.0), ("O", 2.0), ("N", 8.0)]),
        &species,
        TEMPERATURE_K,
        PRESSURE_BAR,
    )
    .expect("Gibbs minimisation converges");

    assert!(
        (equilibrium.moles_of("H2O") - 2.0).abs() < COMPLETE_CONVERSION_TOLERANCE,
        "all hydrogen ends as water at {TEMPERATURE_K} K: {:?}",
        equilibrium.composition
    );
    assert!(
        equilibrium.moles_of("H2") < COMPLETE_CONVERSION_TOLERANCE,
        "no hydrogen survives: {:?}",
        equilibrium.composition
    );
    assert!(
        equilibrium.moles_of("O2") < COMPLETE_CONVERSION_TOLERANCE,
        "and no oxygen: {:?}",
        equilibrium.composition
    );
}

/// The kinetic route moves towards that endpoint and does not pass it.
#[test]
fn the_kinetic_route_approaches_equilibrium_without_overshooting_it() {
    let species = pool(&["H2", "O2", "H2O", "H", "O", "OH", "HO2", "H2O2", "N2"]);
    // The same charge, in moles, that the reactor below is given.
    let hydrogen = 2.0e-3;
    let oxygen = 1.0e-3;
    let nitrogen = 4.0e-3;
    let equilibrium = equilibrate_tp(
        &budget(&[
            ("H", 2.0 * hydrogen),
            ("O", 2.0 * oxygen),
            ("N", 2.0 * nitrogen),
        ]),
        &species,
        TEMPERATURE_K,
        PRESSURE_BAR,
    )
    .expect("Gibbs minimisation converges");

    let mechanism = parse_yaml(&pack_text("h2-o2-skeletal-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    let mut feeds = vec![("H2", hydrogen), ("O2", oxygen), ("N2", nitrogen)];
    for radical in ["H", "O", "OH", "HO2", "H2O2", "H2O"] {
        feeds.push((radical, SEED_MOLES));
    }
    let mut vessel = reactor(1.0, TEMPERATURE_K, &feeds);

    for _ in 0..STEPS {
        advance_network_with_options(&mut vessel, STEP_SECONDS, &network, STIFF)
            .expect("a skeletal hydrogen network integrates");
    }

    let kinetic_water = moles(&vessel, "H2O");
    let equilibrium_water = equilibrium.moles_of("H2O");
    assert!(
        kinetic_water > 0.0,
        "the kinetic route makes some water in a millisecond"
    );
    // The slack covers the nanomole radical seed, which the element
    // budget handed to CEA deliberately does not contain: a real
    // overshoot would be tens of per cent, not a tenth of one.
    assert!(
        kinetic_water <= equilibrium_water * (1.0 + 1e-3),
        "kinetics produced {kinetic_water} mol of water; equilibrium permits {equilibrium_water}"
    );
    assert!(
        moles(&vessel, "H2") >= equilibrium.moles_of("H2") - 1e-9,
        "kinetics consumed more hydrogen than equilibrium allows"
    );
    assert!(
        moles(&vessel, "O2") >= equilibrium.moles_of("O2") - 1e-9,
        "kinetics consumed more oxygen than equilibrium allows"
    );
}

/// The overall reaction the pack's stoichiometry can perform is the
/// reaction CEA settles on: nothing in the network can make a species the
/// equilibrium calculation is not also offered.
#[test]
fn the_networks_only_overall_change_is_the_one_cea_finds() {
    for pack in ["h2-o2-skeletal-v1", "co-h2-wet-v1"] {
        let mechanism = parse_yaml(&pack_text(pack)).expect("the pack parses");
        let arena = MechanismArena::default();
        let network = mechanism.compile_in(&arena);
        for reaction in network.reactions {
            for term in reaction.stoichiometry {
                assert!(
                    db().get(term.species).is_some(),
                    "{pack}: {} moves {}, which CEA cannot price",
                    reaction.equation,
                    term.species
                );
            }
        }
    }
}
