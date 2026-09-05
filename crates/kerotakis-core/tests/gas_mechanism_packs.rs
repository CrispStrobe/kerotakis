//! BRD-041 — the gas-phase mechanism packs, checked as data.
//!
//! BRD-040 found that no audited combustion mechanism may ship as runtime
//! data: not one of them carries a redistribution grant. The packs under
//! `data/mechanisms/` are therefore written here, reaction by reaction,
//! from published rate constants — and a rate constant without a source
//! is exactly the thing this file refuses to let through.
//!
//! Three properties are enforced, and each of them has failed for a real
//! mechanism file at some point in this project's history:
//!
//! 1. **Every reaction names its source, its validity range and its
//!    retrieval date.** The parser does not read `note`, so nothing else
//!    in the engine would notice a reaction that quietly has no
//!    provenance.
//! 2. **The declared units are the units that arrive in the network.**
//!    BRD-040's audit found the activation-energy scale misread by 10³ on
//!    a very common file shape. The check here re-derives every
//!    pre-exponential and every activation energy from the document's own
//!    `units:` block and compares against what the parser produced, for
//!    every reaction in every pack.
//! 3. **A pack that stops balancing stops parsing.** The mutation test at
//!    the bottom breaks one equation and demands a typed refusal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kerotakis_core::kinetics::mechanism::{parse_yaml, MechanismArena, MechanismError};
use kerotakis_core::kinetics::{advance_network_with_options, IntegrationOptions};
use kerotakis_core::species::Phase;
use kerotakis_core::units::{Kelvin, Liters, Moles};
use kerotakis_core::vessel::{Headspace, Vessel, VesselId};
use kerotakis_core::SpeciesId;

/// Every shipped pack. A new file that is not listed here is not tested,
/// so the directory listing is checked against this list as well.
const PACKS: &[&str] = &["h2-o2-skeletal-v1", "co-h2-wet-v1", "hydrocarbon-global-v1"];

/// `units: {length: cm, quantity: mol}` means a concentration written in
/// mol·cm⁻³ is 10³ times the same concentration in mol·L⁻¹.
const CONCENTRATION_SCALE: f64 = 1_000.0;

/// `units: {activation-energy: cal/mol}`, the thermochemical calorie.
const CALORIE_JOULES: f64 = 4.184;

fn pack_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/mechanisms")
}

fn pack_text(name: &str) -> String {
    let path = pack_dir().join(format!("{name}.yaml"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn document(name: &str) -> serde_yaml_ng::Value {
    serde_yaml_ng::from_str(&pack_text(name)).expect("a shipped pack is valid YAML")
}

/// The `note` of every reaction, in document order.
fn reaction_notes(document: &serde_yaml_ng::Value) -> Vec<String> {
    document["reactions"]
        .as_sequence()
        .expect("a pack has a reactions sequence")
        .iter()
        .map(|reaction| {
            reaction["note"]
                .as_str()
                .expect("every reaction carries a note")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// Split `key=value; key=value` into pairs, trimming whitespace.
fn note_fields(note: &str) -> Vec<(String, String)> {
    note.split(';')
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect()
}

fn note_field(note: &str, key: &str) -> Option<String> {
    note_fields(note)
        .into_iter()
        .find(|(name, _)| name.as_str() == key)
        .map(|(_, value)| value)
}

#[test]
fn the_listed_packs_are_the_shipped_packs() {
    let mut found: Vec<String> = std::fs::read_dir(pack_dir())
        .expect("data/mechanisms exists")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("yaml"))
        .map(|path| {
            path.file_stem()
                .expect("a yaml file has a stem")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    found.sort();
    let mut listed: Vec<String> = PACKS.iter().map(|name| (*name).to_string()).collect();
    listed.sort();
    assert_eq!(
        found, listed,
        "every pack in data/mechanisms must be listed in PACKS and tested"
    );
}

#[test]
fn every_pack_parses_compiles_and_keeps_its_reactions() {
    for name in PACKS {
        let mechanism = parse_yaml(&pack_text(name))
            .unwrap_or_else(|error| panic!("{name} must parse: {error}"));
        let summary = mechanism.summary();
        assert!(
            summary.reactions >= 3,
            "{name}: a pack with fewer than three reactions is a fixture, not a mechanism"
        );
        assert!(
            summary.species >= 3,
            "{name}: too few species to be a network"
        );

        let arena = MechanismArena::default();
        let network = mechanism.compile_in(&arena);
        assert_eq!(
            network.reactions.len(),
            summary.reactions,
            "{name}: compiling must not drop a reaction"
        );
        assert_eq!(
            network.id, summary.name,
            "{name}: the compiled network keeps the document's identity"
        );
    }
}

#[test]
fn every_pack_states_its_licence_and_its_limits() {
    for name in PACKS {
        let doc = document(name);
        let note = doc["note"]
            .as_str()
            .unwrap_or_else(|| panic!("{name}: the document needs a top-level note"));
        for required in ["CC BY 4.0", "Kerotakis", "does not claim"] {
            assert!(
                note.contains(required),
                "{name}: the pack header must contain {required:?}"
            );
        }
    }
}

#[test]
fn every_reaction_records_source_validity_and_retrieval() {
    let mut seen_ids = BTreeSet::new();
    for name in PACKS {
        for (index, note) in reaction_notes(&document(name)).iter().enumerate() {
            let where_ = format!("{name} reaction {}", index + 1);

            let id =
                note_field(note, "id").unwrap_or_else(|| panic!("{where_}: note has no id= field"));
            assert!(
                !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
                "{where_}: id {id:?} must be a non-empty kebab identifier"
            );
            assert!(
                seen_ids.insert(id.clone()),
                "{where_}: reaction id {id:?} is used twice"
            );

            let source = note_field(note, "source")
                .unwrap_or_else(|| panic!("{where_}: note has no source= field"));
            assert!(
                source.len() > 20,
                "{where_}: source {source:?} is too short to be a citation"
            );

            let validity = note_field(note, "validity")
                .unwrap_or_else(|| panic!("{where_}: note has no validity= field"));
            let (low, high) = validity_range(&validity)
                .unwrap_or_else(|| panic!("{where_}: validity {validity:?} is not 'LOW-HIGH K'"));
            assert!(
                low >= 80.0 && high <= 8000.0 && low < high,
                "{where_}: validity {low}-{high} K is not a sane increasing range"
            );

            let retrieved = note_field(note, "retrieved")
                .unwrap_or_else(|| panic!("{where_}: note has no retrieved= field"));
            assert!(
                is_iso_date(&retrieved),
                "{where_}: retrieved {retrieved:?} is not an ISO date"
            );
        }
    }
}

/// Parse the `LOW-HIGH K` shape the packs use for a validity range.
fn validity_range(text: &str) -> Option<(f64, f64)> {
    let numbers = text.strip_suffix(" K")?;
    let (low, high) = numbers.split_once('-')?;
    Some((low.trim().parse().ok()?, high.trim().parse().ok()?))
}

fn is_iso_date(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()))
}

/// The document's `units:` block is honoured exactly, for every reaction.
///
/// This is the check BRD-040's audit says has to exist: the earlier parser
/// read `units: {quantity: mol}` as kilomoles and mis-scaled every
/// activation energy by 10³, silently. Here the expected value is
/// re-derived from the literature numbers written in the file rather than
/// taken from the parser, so the two have to agree independently.
#[test]
fn declared_units_survive_into_the_compiled_network() {
    for name in PACKS {
        let doc = document(name);
        let units = &doc["units"];
        assert_eq!(
            units["length"].as_str(),
            Some("cm"),
            "{name}: the packs are written in the literature's cm/mol/cal units"
        );
        assert_eq!(units["quantity"].as_str(), Some("mol"), "{name}: quantity");
        assert_eq!(
            units["activation-energy"].as_str(),
            Some("cal/mol"),
            "{name}: activation energy"
        );

        let reactions = doc["reactions"].as_sequence().expect("reactions sequence");
        let summary = parse_yaml(&pack_text(name))
            .expect("a shipped pack parses")
            .summary();

        for (entry, detail) in reactions.iter().zip(&summary.reaction_details) {
            let where_ = format!("{name}: {}", detail.equation);
            // A three-body rate constant carries one more concentration
            // unit than its reactant order, because M is a reactant the
            // equation does not spend.
            let extra = if detail.rate_model == "three_body" {
                1.0
            } else {
                0.0
            };
            let (high_key, low_key) =
                if detail.rate_model == "elementary" || detail.rate_model == "three_body" {
                    ("rate-constant", None)
                } else {
                    ("high-P-rate-constant", Some("low-P-rate-constant"))
                };

            let declared_a = entry[high_key]["A"]
                .as_f64()
                .unwrap_or_else(|| panic!("{where_}: no numeric A under {high_key}"));
            let declared_ea = entry[high_key]["Ea"]
                .as_f64()
                .unwrap_or_else(|| panic!("{where_}: no numeric Ea under {high_key}"));
            let order = detail.total_order + extra;
            let expected_a = declared_a * CONCENTRATION_SCALE.powf(1.0 - order);
            assert!(
                relative_error(detail.pre_exponential, expected_a) < 1e-9,
                "{where_}: A {} does not match {declared_a} cm/mol/s at order {order}",
                detail.pre_exponential
            );
            assert!(
                relative_error(
                    detail.activation_energy_j_per_mol,
                    declared_ea * CALORIE_JOULES
                ) < 1e-9,
                "{where_}: Ea {} J/mol does not match {declared_ea} cal/mol",
                detail.activation_energy_j_per_mol
            );

            if let Some(low_key) = low_key {
                let declared_low = entry[low_key]["A"]
                    .as_f64()
                    .unwrap_or_else(|| panic!("{where_}: no numeric A under {low_key}"));
                let expected_low =
                    declared_low * CONCENTRATION_SCALE.powf(1.0 - (detail.total_order + 1.0));
                let actual_low = detail
                    .low_pressure_pre_exponential
                    .unwrap_or_else(|| panic!("{where_}: falloff without a low-pressure limit"));
                assert!(
                    relative_error(actual_low, expected_low) < 1e-9,
                    "{where_}: low-pressure A {actual_low} does not match {declared_low}"
                );
            }
        }
    }
}

fn relative_error(actual: f64, expected: f64) -> f64 {
    if expected == 0.0 {
        actual.abs()
    } else {
        (actual - expected).abs() / expected.abs()
    }
}

/// Every rate constant is a usable number across the range it claims.
///
/// A modified-Arrhenius expression can overflow to infinity or underflow
/// to zero long before it becomes physically wrong, and a mechanism whose
/// k is infinite at 2500 K is not a mechanism.
#[test]
fn every_rate_constant_is_finite_and_positive_across_its_declared_range() {
    for name in PACKS {
        let notes = reaction_notes(&document(name));
        let mechanism = parse_yaml(&pack_text(name)).expect("a shipped pack parses");
        let arena = MechanismArena::default();
        let network = mechanism.compile_in(&arena);

        for (reaction, note) in network.reactions.iter().zip(&notes) {
            let (low, high) = validity_range(&note_field(note, "validity").expect("validity"))
                .expect("a checked validity range");
            for temperature in [low, 0.5 * (low + high), high] {
                let k = reaction.forward.arrhenius.rate_constant(temperature);
                assert!(
                    k.is_finite() && k > 0.0,
                    "{name}: {} has k = {k} at {temperature} K",
                    reaction.equation
                );
            }
            // A barrier means faster when hotter. Not every step has one:
            // a radical-radical reaction with no barrier can have a
            // NEGATIVE activation energy, and CO + OH -> CO2 + H is the
            // textbook case. Those are real and are written as such, so
            // the monotonicity claim is made only where it is a claim.
            let law = reaction.forward.arrhenius;
            if law.activation_energy >= 0.0 && law.temperature_exponent >= 0.0 {
                let cold = law.rate_constant(low);
                let hot = law.rate_constant(high);
                assert!(
                    hot >= cold,
                    "{name}: {} has a barrier but is slower at {high} K than at {low} K",
                    reaction.equation
                );
            }
        }
    }
}

/// A pack that stops balancing stops parsing.
#[test]
fn an_unbalanced_edit_is_refused() {
    let broken = pack_text("h2-o2-skeletal-v1").replacen("=> O + OH", "=> O + O + OH", 1);
    assert_ne!(
        broken,
        pack_text("h2-o2-skeletal-v1"),
        "the mutation must actually change the document"
    );
    match parse_yaml(&broken) {
        Err(MechanismError::ElementImbalance { element, .. }) => {
            assert_eq!(element, "O", "the imbalance names the element it found");
        }
        other => panic!("an unbalanced equation must be refused, got {other:?}"),
    }
}

/// Integration controls for a stiff radical chain, and the window this
/// engine can actually carry one across.
///
/// The defaults are tuned for the curated aqueous kinetics and start
/// with a millisecond trial step, which is ten times the interval these
/// tests advance and about a hundred thousand times the timescale of the
/// branching chain inside it. A nanosecond first step and a matching
/// absolute tolerance — extents here are nanomoles, not millimoles — let
/// the solver find its own scale.
///
/// Until 2026-09-05 that was not enough to reach ignition, and the reason
/// is kept here because the fix is the reason's mirror image.
/// `kinetics_integrator.rs` gives diffsol a matrix-free Jacobian whose
/// finite-difference probe was ONE SCALAR for the whole extent vector,
/// sized from `(1 + ||x||_inf)` — an absolute 1.5e-8 mol at the zero
/// extents every interval starts from. In a radical chain the species span
/// nine orders of magnitude at once, so a probe of that size linearised the
/// nanomole radicals across their entire range; the Newton iteration then
/// failed, and on this network it exhausted its failure budget at about
/// 2.7 µs. The probe is now sized by the species it moves — no amount the
/// probe touches moves by more than √ε of itself — and the same network
/// integrates through ignition to exhaustion in one call
/// (`hydrogen_burns_through_ignition_in_one_call` below).
///
/// The bounded-window tests stay: they pin the early chain, where the
/// packs were already doing real chemistry, and they are the regression
/// the fix must not break.
const STIFF: IntegrationOptions = IntegrationOptions {
    relative_tolerance: 1e-6,
    absolute_tolerance_moles: 1e-14,
    initial_step_seconds: 1e-9,
};

/// One interval of the bounded window, and how many of them.
const STEP_SECONDS: f64 = 1.0e-8;
const STEPS: usize = 50;

/// A gas vessel holding exactly what the caller asked for.
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

/// The radical seed.
///
/// The integrator is isothermal and has no spark: a vessel of cold H₂ and
/// O₂ with no radicals in it has, correctly, a rate of exactly zero for
/// every chain-carrying step, and the network would sit there forever.
/// A tenth of a micromole of each intermediate stands in for the ignition
/// source. That is five parts in a hundred thousand of the fuel, so it
/// changes the products by nothing anyone could measure, and it buys two
/// things the solver needs: no species starts on the depletion-event
/// path, which exists to stop an amount going negative and is not a place
/// a radical pool should live, and the chain has two decades less
/// exponential growth to climb through, which is two decades less of the
/// sharpest part of the ignition front.
const SEED_MOLES: f64 = 1e-7;

#[test]
fn hydrogen_moves_towards_water_in_bounded_time() {
    let mechanism = parse_yaml(&pack_text("h2-o2-skeletal-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    let mut feeds = vec![("H2", 2.0e-3), ("O2", 1.0e-3), ("N2", 4.0e-3)];
    for radical in ["H", "O", "OH", "HO2", "H2O2", "H2O"] {
        feeds.push((radical, SEED_MOLES));
    }
    let mut vessel = reactor(1.0, 1200.0, &feeds);

    let hydrogen_atoms_before = 2.0 * moles(&vessel, "H2")
        + moles(&vessel, "H")
        + moles(&vessel, "OH")
        + 2.0 * moles(&vessel, "H2O")
        + moles(&vessel, "HO2")
        + 2.0 * moles(&vessel, "H2O2");
    let water_before = moles(&vessel, "H2O");
    let fuel_before = moles(&vessel, "H2");

    let mut steps = 0usize;
    for _ in 0..STEPS {
        let report = advance_network_with_options(&mut vessel, STEP_SECONDS, &network, STIFF)
            .expect("a skeletal hydrogen network integrates");
        steps += report.statistics.accepted_steps + report.statistics.rejected_steps;
    }

    assert!(
        moles(&vessel, "H2") < fuel_before,
        "hydrogen is consumed: {} -> {}",
        fuel_before,
        moles(&vessel, "H2")
    );
    assert!(
        moles(&vessel, "H2O") > water_before,
        "and water is made: {} -> {}",
        water_before,
        moles(&vessel, "H2O")
    );
    assert!(
        relative_error(moles(&vessel, "N2"), 4.0e-3) < 1e-9,
        "nitrogen is a diluent here and takes no part"
    );

    let hydrogen_atoms_after = 2.0 * moles(&vessel, "H2")
        + moles(&vessel, "H")
        + moles(&vessel, "OH")
        + 2.0 * moles(&vessel, "H2O")
        + moles(&vessel, "HO2")
        + 2.0 * moles(&vessel, "H2O2");
    assert!(
        relative_error(hydrogen_atoms_after, hydrogen_atoms_before) < 1e-6,
        "hydrogen atoms are conserved: {hydrogen_atoms_before} -> {hydrogen_atoms_after}"
    );

    assert!(
        steps < 200_000,
        "a millisecond of skeletal hydrogen chemistry cost {steps} solver steps"
    );
}

/// Through ignition and out the other side, in one call.
///
/// The bounded-window test above stops at half a microsecond because
/// that was all the old probe could carry. With the species-scaled probe
/// the same seeded stoichiometric mixture at 1200 K runs its induction
/// period, its branching explosion and its recombination tail inside a
/// single ten-millisecond interval, and comes out with the hydrogen
/// essentially gone and every atom accounted for. The endpoint is what the
/// CEA oracle in `kerotakis-cea/tests/gas_mechanism_endpoint.rs` already
/// says it should be; this test is the integrator getting there itself.
#[test]
fn hydrogen_burns_through_ignition_in_one_call() {
    let mechanism = parse_yaml(&pack_text("h2-o2-skeletal-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    let mut feeds = vec![("H2", 2.0e-3), ("O2", 1.0e-3), ("N2", 4.0e-3)];
    for radical in ["H", "O", "OH", "HO2", "H2O2", "H2O"] {
        feeds.push((radical, SEED_MOLES));
    }
    let mut vessel = reactor(1.0, 1200.0, &feeds);
    let h_atoms = |v: &Vessel| {
        2.0 * moles(v, "H2")
            + moles(v, "H")
            + moles(v, "OH")
            + 2.0 * moles(v, "H2O")
            + moles(v, "HO2")
            + 2.0 * moles(v, "H2O2")
    };
    let o_atoms = |v: &Vessel| {
        2.0 * moles(v, "O2")
            + moles(v, "O")
            + moles(v, "OH")
            + moles(v, "H2O")
            + 2.0 * moles(v, "HO2")
            + 2.0 * moles(v, "H2O2")
    };
    let (h_before, o_before) = (h_atoms(&vessel), o_atoms(&vessel));
    let fuel_before = moles(&vessel, "H2");

    let report = advance_network_with_options(&mut vessel, 1.0e-2, &network, STIFF)
        .expect("the skeletal hydrogen network integrates through ignition");

    let fuel_after = moles(&vessel, "H2");
    assert!(
        fuel_after < 0.1 * fuel_before,
        "ten milliseconds at 1200 K should burn most of the hydrogen: {fuel_before} -> {fuel_after}"
    );
    assert!(
        moles(&vessel, "H2O") > 0.9 * fuel_before,
        "and make it into water: {} mol",
        moles(&vessel, "H2O")
    );
    assert!(
        relative_error(h_atoms(&vessel), h_before) < 1e-6,
        "hydrogen atoms conserved"
    );
    assert!(
        relative_error(o_atoms(&vessel), o_before) < 1e-6,
        "oxygen atoms conserved"
    );
    assert!(relative_error(moles(&vessel, "N2"), 4.0e-3) < 1e-9);
    let steps = report.statistics.accepted_steps + report.statistics.rejected_steps;
    assert!(
        steps < 500_000,
        "ignition should not cost the earth: {steps} solver steps, {} nonlinear failures",
        report.statistics.nonlinear_failures
    );
}

/// More oxygen burns more fuel: the lean/rich metamorphic case.
#[test]
fn more_oxygen_consumes_more_hydrogen() {
    let mechanism = parse_yaml(&pack_text("h2-o2-skeletal-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    let mut consumed = Vec::new();
    for oxygen in [2.5e-4, 1.0e-3] {
        let mut feeds = vec![("H2", 2.0e-3), ("O2", oxygen), ("N2", 4.0e-3)];
        for radical in ["H", "O", "OH", "HO2", "H2O2", "H2O"] {
            feeds.push((radical, SEED_MOLES));
        }
        let mut vessel = reactor(1.0, 1200.0, &feeds);
        for _ in 0..STEPS {
            advance_network_with_options(&mut vessel, STEP_SECONDS, &network, STIFF)
                .expect("a skeletal hydrogen network integrates");
        }
        consumed.push(2.0e-3 - moles(&vessel, "H2"));
    }

    assert!(
        consumed[1] > consumed[0],
        "four times the oxygen must burn more hydrogen, not less: {consumed:?}"
    );
}

/// Wet CO oxidation is the hydrogen mechanism with a carbon sink bolted
/// on, and that is the whole teaching point of the pack.
///
/// Carbon monoxide is famously hard to light dry and easy to light damp.
/// The reason is one reaction — `CO + OH => CO2 + H` — and in this pack
/// it is the ONLY route from CO to CO2 at all. So the demonstration is
/// direct: two identical charges of carbon monoxide and oxygen, one of
/// them holding a trace of water and the radicals a damp gas carries,
/// and only one of them makes any carbon dioxide.
///
/// Note what this test does NOT show, because the bounded window above
/// cannot reach it: the *catalytic* part, where the H atom released by
/// `CO + OH` goes back into `H + O2` and regenerates the OH, so that a
/// trace of water turns over the whole charge without being consumed.
/// That is a chain running past its induction time, and this integrator
/// does not carry one. What is checked here is the structure the
/// catalysis is built out of.
#[test]
fn carbon_monoxide_burns_only_through_a_hydrogen_bearing_radical() {
    let mechanism = parse_yaml(&pack_text("co-h2-wet-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    // The dry charge holds no hydrogen atom in any form.
    let dry: Vec<(&str, f64)> = vec![
        ("CO", 2.0e-3),
        ("O2", 1.0e-3),
        ("N2", 4.0e-3),
        ("O", SEED_MOLES),
        ("CO2", SEED_MOLES),
    ];
    let mut damp = dry.clone();
    damp.push(("H2O", 2.0e-4));
    for radical in ["H", "OH", "HO2", "H2O2", "H2"] {
        damp.push((radical, SEED_MOLES));
    }

    let mut burned = Vec::new();
    for feeds in [&dry, &damp] {
        let mut vessel = reactor(1.0, 1200.0, feeds);
        let carbon_before = moles(&vessel, "CO") + moles(&vessel, "CO2");
        for _ in 0..STEPS {
            advance_network_with_options(&mut vessel, STEP_SECONDS, &network, STIFF)
                .expect("a wet CO network integrates");
        }
        let carbon_after = moles(&vessel, "CO") + moles(&vessel, "CO2");
        assert!(
            relative_error(carbon_after, carbon_before) < 1e-9,
            "carbon is conserved: {carbon_before} -> {carbon_after}"
        );
        burned.push(moles(&vessel, "CO2") - SEED_MOLES);
    }

    assert!(
        relative_error(burned[0], 0.0) < 1e-15,
        "no carbon dioxide without hydrogen: {} mol",
        burned[0]
    );
    assert!(
        burned[1] > 0.0,
        "and carbon dioxide with it: {} mol",
        burned[1]
    );

    // The declared boundary is in the file, not only in this test. The
    // 2005 evaluation recommends nothing for `CO + O2`, `CO + HO2` or
    // `CO + O (+M)`, so the pack carries none of them and the zero above
    // is a property of the pack rather than a claim that dry carbon
    // monoxide cannot burn.
    let note = document("co-h2-wet-v1")["note"]
        .as_str()
        .expect("the pack has a header note")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for missing in ["CO + O2 -> CO2 + O", "CO + HO2 -> CO2 + OH"] {
        assert!(
            note.contains(missing),
            "the header must name the dry route it does not carry: {missing}"
        );
    }
}

// ---------------------------------------------------------------------------
// The global steps
// ---------------------------------------------------------------------------

/// A global step's orders are measured, not read off its equation.
///
/// This is the whole reason the mechanism front end had to learn
/// `orders:`. `CH4 + 2 O2 => CO2 + 2 H2O` looks third order and is not:
/// Westbrook and Dryer fitted it at −0.3 in the fuel and 1.3 in the
/// oxygen, which is FIRST order overall, and the pre-exponential they
/// tabulate only means what they say it means at that order. Writing the
/// equation's exponents instead would be a different rate law wearing a
/// citation that does not belong to it.
#[test]
fn the_global_steps_carry_the_orders_westbrook_and_dryer_fitted() {
    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    let summary = mechanism.summary();
    let expected = [
        ("CH4", 1.0, -0.3, 1.3),
        ("C3H8", 1.75, 0.1, 1.65),
        ("C4H10", 1.75, 0.15, 1.6),
    ];
    assert_eq!(summary.reactions, expected.len(), "one step per fuel");

    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    for (index, (fuel, total, fuel_order, oxygen_order)) in expected.iter().enumerate() {
        let reaction = &network.reactions[index];
        let detail = &summary.reaction_details[index];
        assert!(
            (detail.total_order - *total).abs() < 1e-12,
            "{fuel}: total order {} is not the fitted {total}",
            detail.total_order
        );
        let order_of = |species: &str| {
            reaction
                .forward
                .orders
                .iter()
                .find(|term| term.species == species)
                .unwrap_or_else(|| panic!("{fuel}: no order term for {species}"))
                .order
        };
        assert!(
            (order_of(fuel) - *fuel_order).abs() < 1e-12,
            "{fuel}: fuel order {} is not {fuel_order}",
            order_of(fuel)
        );
        assert!(
            (order_of("O2") - *oxygen_order).abs() < 1e-12,
            "{fuel}: oxygen order {} is not {oxygen_order}",
            order_of("O2")
        );
    }
}

/// Methane inhibits its own combustion, and the arithmetic says so.
///
/// The negative fuel order is not a curiosity. It is what lets a
/// one-step form reproduce the RICH flammability limit at all — the
/// paper's own point is that "the often-employed choice of simultaneous
/// first order fuel and oxidizer dependence ... cannot correctly predict
/// the rich flammability limit". Doubling the methane at fixed oxygen
/// makes the reaction SLOWER, by exactly 2^−0.3.
#[test]
fn methane_inhibits_its_own_combustion() {
    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    let methane = &network.reactions[0];
    assert!(
        methane.equation.starts_with("CH4"),
        "the first step is methane's: {}",
        methane.equation
    );

    let rate_at = |fuel: f64| {
        let vessel = reactor(
            1.0,
            1600.0,
            &[("CH4", fuel), ("O2", 4.0e-3), ("N2", 1.0e-2)],
        );
        methane.rates_now(&vessel).net
    };
    let ratio = rate_at(2.0e-3) / rate_at(1.0e-3);
    assert!(
        (ratio - 2f64.powf(-0.3)).abs() < 1e-9,
        "twice the methane must burn 2^-0.3 as fast, not {ratio} as fast"
    );
    assert!(ratio < 1.0, "and that is slower, not faster");
}

/// Oxygen drives each fitted step at its own fitted power.
#[test]
fn oxygen_drives_each_global_step_at_its_fitted_power() {
    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    for (fuel, oxygen_order) in [("CH4", 1.3), ("C3H8", 1.65), ("C4H10", 1.6)] {
        let reaction = network
            .reactions
            .iter()
            .find(|candidate| candidate.equation.starts_with(fuel))
            .unwrap_or_else(|| panic!("no step for {fuel}"));
        let rate_at = |oxygen: f64| {
            let vessel = reactor(
                1.0,
                1600.0,
                &[(fuel, 1.0e-3), ("O2", oxygen), ("N2", 1.0e-2)],
            );
            reaction.rates_now(&vessel).net
        };
        let ratio = rate_at(4.0e-3) / rate_at(2.0e-3);
        assert!(
            (ratio - 2f64.powf(oxygen_order)).abs() / 2f64.powf(oxygen_order) < 1e-9,
            "{fuel}: doubling oxygen multiplied the rate by {ratio}, not 2^{oxygen_order}"
        );
    }
}

/// A one-step fuel burns out, and takes its atoms with it.
///
/// Unlike the skeletal packs this is a three-reaction network with no
/// radical pool, so the integrator carries it all the way to exhaustion —
/// which is exactly what a global step is for and the only thing it is
/// for.
#[test]
fn lean_methane_burns_out_under_a_global_step() {
    let mechanism = parse_yaml(&pack_text("hydrocarbon-global-v1")).expect("the pack parses");
    let arena = MechanismArena::default();
    let network = mechanism.compile_in(&arena);

    let fuel = 1.0e-3;
    let mut vessel = reactor(
        1.0,
        1600.0,
        &[("CH4", fuel), ("O2", 4.0e-3), ("N2", 1.0e-2)],
    );
    let carbon_before = moles(&vessel, "CH4") + moles(&vessel, "CO2");
    for _ in 0..20 {
        advance_network_with_options(&mut vessel, 5.0e-3, &network, STIFF)
            .expect("a three-reaction global network integrates");
    }

    assert!(
        moles(&vessel, "CH4") < 0.01 * fuel,
        "the fuel is gone: {} mol left of {fuel}",
        moles(&vessel, "CH4")
    );
    assert!(
        relative_error(moles(&vessel, "CO2"), fuel) < 0.01,
        "one CO2 per methane: {}",
        moles(&vessel, "CO2")
    );
    assert!(
        relative_error(moles(&vessel, "H2O"), 2.0 * fuel) < 0.01,
        "two waters per methane: {}",
        moles(&vessel, "H2O")
    );
    let carbon_after = moles(&vessel, "CH4") + moles(&vessel, "CO2");
    assert!(
        relative_error(carbon_after, carbon_before) < 1e-9,
        "carbon is conserved: {carbon_before} -> {carbon_after}"
    );
    assert!(
        relative_error(moles(&vessel, "N2"), 1.0e-2) < 1e-12,
        "nitrogen takes no part"
    );
}
