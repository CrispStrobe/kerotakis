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

/// Every shipped pack, as `gas_mechanism_packs.rs` lists them.
const PACKS: &[&str] = &["h2-o2-skeletal-v1", "co-h2-wet-v1", "hydrocarbon-global-v1"];

/// The CEA record for a mechanism species.
///
/// One name needs translating. `thermo.inp` keys its two butanes by
/// their full record names — `C4H10,n-butane` and `C4H10,isobutane` —
/// because the formula alone does not say which one, and the mechanism
/// packs say `C4H10` because Westbrook and Dryer's n-paraffin fit is for
/// the straight chain. Naming the translation here beats a silent
/// lookup miss.
fn cea_record(pack: &str, species: &str) -> &'static Species {
    let key = match species {
        "C4H10" => "C4H10,n-butane",
        other => other,
    };
    db().get(key)
        .unwrap_or_else(|| panic!("{pack}: CEA has no thermochemistry for {species} (as {key})"))
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

/// Every pack in the directory names species CEA can also price.
///
/// If this ever fails, the two solvers are no longer answering about the
/// same chemistry and every comparison below is meaningless.
#[test]
fn cea_carries_every_species_the_packs_name() {
    for pack in PACKS {
        let mechanism = parse_yaml(&pack_text(pack)).expect("the pack parses");
        for species in mechanism.species_names() {
            let _ = cea_record(pack, species);
        }
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

/// Every species either kind of pack can move is a species CEA can price,
/// so the comparison below is between two accounts of one chemistry.
#[test]
fn every_species_a_pack_can_move_is_one_cea_can_price() {
    for pack in PACKS {
        let mechanism = parse_yaml(&pack_text(pack)).expect("the pack parses");
        let arena = MechanismArena::default();
        let network = mechanism.compile_in(&arena);
        for reaction in network.reactions {
            for term in reaction.stoichiometry {
                let _ = cea_record(pack, term.species);
            }
        }
    }
}

/// A global step ends where CEA ends, when equilibrium is complete
/// combustion.
///
/// This is the endpoint oracle the skeletal packs cannot supply, because
/// a three-reaction network with no radical pool integrates all the way
/// to exhaustion. Lean methane at 1600 K is the condition where the
/// claim is fair: with four times the stoichiometric oxygen, equilibrium
/// really is CO₂ and H₂O, so a one-step form that can only make CO₂ and
/// H₂O is allowed to agree with it. Pressure is not a variable here —
/// `CH4 + 2 O2 → CO2 + 2 H2O` has Δn = 0, so the equilibrium composition
/// does not move with it.
#[test]
fn a_global_step_ends_where_cea_ends_under_lean_conditions() {
    let fuel = 1.0e-3;
    let oxygen = 4.0e-3;
    let nitrogen = 1.0e-2;

    let species = pool(&["CH4", "O2", "CO2", "H2O", "CO", "H2", "OH", "O", "H", "N2"]);
    let equilibrium = equilibrate_tp(
        &budget(&[
            ("C", fuel),
            ("H", 4.0 * fuel),
            ("O", 2.0 * oxygen),
            ("N", 2.0 * nitrogen),
        ]),
        &species,
        1600.0,
        PRESSURE_BAR,
    )
    .expect("Gibbs minimisation converges");
    assert!(
        equilibrium.moles_of("CO") < 0.01 * equilibrium.moles_of("CO2"),
        "lean methane at 1600 K leaves almost no CO: {:?}",
        equilibrium.composition
    );

    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    let mut vessel = reactor(
        1.0,
        1600.0,
        &[("CH4", fuel), ("O2", oxygen), ("N2", nitrogen)],
    );
    for _ in 0..20 {
        advance_network_with_options(&mut vessel, 5.0e-3, &network, STIFF)
            .expect("a three-reaction global network integrates");
    }

    for product in ["CO2", "H2O"] {
        let kinetic = moles(&vessel, product);
        let thermodynamic = equilibrium.moles_of(product);
        assert!(
            (kinetic - thermodynamic).abs() / thermodynamic < 0.02,
            "{product}: the global step ends at {kinetic} mol, CEA at {thermodynamic}"
        );
    }
}

/// And where equilibrium is NOT complete combustion, the global step is
/// wrong in the direction its own authors documented.
///
/// Westbrook and Dryer's Table III (p. 37) puts a detailed methane-air
/// mechanism at CO/CO₂ = 0.11 at an equivalence ratio of 1.0 and 0.69 at
/// 1.2, against a one-step form that gives none at all — and an adiabatic
/// flame temperature 100 K too high because of it. CEA agrees that the
/// carbon monoxide is there. The pack cannot make any: it has no CO in
/// it, by construction. This test is the limitation, executable.
#[test]
fn a_global_step_cannot_make_the_carbon_monoxide_cea_finds() {
    // Rich: an equivalence ratio of 1.2, so 2/1.2 moles of O2 per methane.
    let species = pool(&["CH4", "O2", "CO2", "H2O", "CO", "H2", "OH", "O", "H"]);
    let equilibrium = equilibrate_tp(
        &budget(&[("C", 1.0), ("H", 4.0), ("O", 2.0 * 2.0 / 1.2)]),
        &species,
        2000.0,
        PRESSURE_BAR,
    )
    .expect("Gibbs minimisation converges");
    let ratio = equilibrium.moles_of("CO") / equilibrium.moles_of("CO2");
    assert!(
        ratio > 0.05,
        "rich methane at 2000 K really does make carbon monoxide: CO/CO2 = {ratio}"
    );

    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    assert!(
        !mechanism.species_names().any(|name| name == "CO"),
        "the global pack has no carbon monoxide in it at all"
    );
    let note = pack_text("hydrocarbon-global-v1");
    assert!(
        note.contains("does not claim carbon monoxide"),
        "and the pack says so in its own header"
    );
}

/// Every hydrocarbon releases about the same heat per mole of oxygen it
/// burns, and each global step's stoichiometry must too.
///
/// This is the energy check, and it is deliberately not a comparison
/// against three memorised heats of combustion. Roughly 400-410 kJ per
/// mole of O₂ is a property of hydrocarbon oxidation itself, so a step
/// whose stoichiometry drifted — a lost water, a miscounted CO₂ — falls
/// outside the band even though it still balances by element. The
/// enthalpies are CEA's own formation values, priced through the pack's
/// own stoichiometric vector.
#[test]
fn every_global_step_releases_about_the_same_heat_per_mole_of_oxygen() {
    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    for reaction in network.reactions {
        let mut enthalpy = 0.0;
        let mut oxygen = 0.0;
        for term in reaction.stoichiometry {
            enthalpy +=
                term.coefficient * cea_record("hydrocarbon-global-v1", term.species).h_formation;
            if term.species == "O2" {
                oxygen -= term.coefficient;
            }
        }
        assert!(oxygen > 0.0, "{}: consumes no oxygen", reaction.equation);
        let per_oxygen = enthalpy / oxygen;
        assert!(
            (-420_000.0..=-395_000.0).contains(&per_oxygen),
            "{}: {per_oxygen:.0} J per mole of O2 is outside the 400-410 kJ band \
             every hydrocarbon shares",
            reaction.equation
        );
    }
}
