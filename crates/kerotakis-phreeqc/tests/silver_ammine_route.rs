//! Native equilibrium admission checks, not experimental yield/rate calibration.
#![cfg(feature = "engine")]
use kerotakis_core::{
    ledger::ConservedLedger, Equilibrator, Event, Kelvin, Moles, Phase, SpeciesId, ThermalMode,
    Vessel, VesselId,
};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn has_ligand_inventory_boundary(events: &[Event]) -> bool {
    events.iter().any(|event| {
        serde_json::to_value(event).unwrap()["reason"]["key"].as_str()
            == Some("not-modeled.silver-ammine-ligand-inventory")
    })
}

fn water() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "finite silver/ammonia control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    vessel
}

fn deposit(vessel: &mut Vessel, key: &str, moles: f64) {
    let data = kerotakis_core::species::lookup_key(key).expect("registered reagent");
    vessel.deposit(SpeciesId::new(key), Moles(moles), data.standard_phase);
}

fn molality(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .solution
        .as_ref()
        .unwrap()
        .species
        .iter()
        .find(|species| species.name == key)
        .map_or(0.0, |species| species.molality)
}

fn ammine_moles(vessel: &Vessel) -> f64 {
    let kgw = vessel.solution.as_ref().unwrap().solvent_kg.unwrap();
    (molality(vessel, "AgNH3+") + molality(vessel, "Ag(NH3)2+")) * kgw
}

fn solid_silver(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "AgCl" && portion.phase == Phase::Solid)
        .map(|portion| portion.moles.0)
        .sum()
}

fn assert_conserved(before: &Vessel, after: &Vessel) {
    let before = ConservedLedger::from_vessel(before);
    let after = ConservedLedger::from_vessel(after);
    assert!((before.charge - after.charge).abs() < 1e-9);
    let elements: std::collections::BTreeSet<_> = before
        .elements
        .keys()
        .chain(after.elements.keys())
        .collect();
    for element in elements {
        let amount = before.elements.get(element).copied().unwrap_or(0.0);
        let actual = after.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            (actual - amount).abs() < 1e-9 + amount.abs() * 1e-6,
            "{element}: {amount} -> {actual}"
        );
    }
}

// Public SpeciesDetail values carry four significant figures. Sums and
// comparisons derived from them need a rounding budget, unlike the analytical
// ledger's separate conservation checks above.
fn assert_rounded_species_close(actual: f64, expected: f64) {
    let tolerance = 1e-12 + 5e-4 * actual.abs().max(expected.abs());
    assert!(
        (actual - expected).abs() <= tolerance,
        "rounded species: {actual} vs {expected}, tolerance {tolerance}"
    );
}

#[test]
fn finite_ammonia_forms_complexes_and_increases_silver_chloride_dissolution() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut baseline = water();
    deposit(&mut baseline, "AgCl", 0.001);
    let baseline_events = solver.equilibrate(&mut baseline).unwrap();
    assert!(!has_ligand_inventory_boundary(&baseline_events));
    assert!(baseline
        .solution
        .as_ref()
        .unwrap()
        .provenance
        .as_ref()
        .unwrap()
        .dataset
        .contains("wateq4f.dat"));
    let baseline_dissolved = 0.001 - solid_silver(&baseline);

    let mut vessel = water();
    deposit(&mut vessel, "AgCl", 0.001);
    deposit(&mut vessel, "NH3", 0.002);
    let before = vessel.clone();
    let events = solver.equilibrate(&mut vessel).unwrap();
    assert!(has_ligand_inventory_boundary(&events));
    let solution = vessel.solution.as_ref().unwrap();
    let provenance = solution.provenance.as_ref().unwrap();
    assert!(provenance.dataset.contains("minteq.v4.dat"));
    assert!(provenance.routing.contains("silver-ammonia complexes"));
    // Match the native NH4-based NIST equations, not a guessed yield or
    // a constant converted to an incompatible NH3 reaction convention.
    // The public distribution omits molality <=1e-9. In this alkaline
    // mixture H+ is below that reporting threshold, so its absence is not
    // zero proton activity. pH defines the authoritative hydrogen activity.
    assert!(solution.ph > 9.0);
    assert!(!solution.species.iter().any(|species| species.name == "H+"));
    let activity = |name: &str| {
        if name == "H+" {
            return 10.0_f64.powf(-solution.ph);
        }
        solution
            .species
            .iter()
            .find(|species| species.name == name)
            .unwrap_or_else(|| panic!("required {name} activity absent at pH {}", solution.ph))
            .activity
    };
    for (complex, ligands, log_k) in [("AgNH3+", 1.0, -5.934), ("Ag(NH3)2+", 2.0, -11.268)] {
        let log_q = activity(complex).log10() + ligands * activity("H+").log10()
            - activity("Ag+").log10()
            - ligands * activity("NH4+").log10();
        // Public SpeciesDetail values are rounded; native equations are exact.
        assert!((log_q - log_k).abs() < 0.003, "{complex}: {log_q}");
    }
    assert!(ammine_moles(&vessel) > baseline_dissolved);
    assert!(0.001 - solid_silver(&vessel) > 2.0 * baseline_dissolved);
    // The finite ligand is conserved, not an infinite NH3 reservoir. No
    // assertion requires all supplied AgCl to dissolve at this ligand dose.
    let kgw = solution.solvent_kg.unwrap();
    let ligand_in_ammines =
        (molality(&vessel, "AgNH3+") + 2.0 * molality(&vessel, "Ag(NH3)2+")) * kgw;
    let recovered_ligand =
        ligand_in_ammines + (molality(&vessel, "NH3") + molality(&vessel, "NH4+")) * kgw;
    assert!(ligand_in_ammines > 0.0 && ligand_in_ammines < 0.002);
    assert_rounded_species_close(recovered_ligand, 0.002);
    let dissolved_silver: f64 = solution
        .species
        .iter()
        .filter(|species| species.name.starts_with("Ag"))
        .map(|species| species.molality * kgw)
        .sum();
    assert_rounded_species_close(dissolved_silver + solid_silver(&vessel), 0.001);
    assert_conserved(&before, &vessel);
    let ph = solution.ph;
    let complexes = ammine_moles(&vessel);
    solver.equilibrate(&mut vessel).unwrap();
    assert!((vessel.solution.as_ref().unwrap().ph - ph).abs() < 1e-4);
    assert_rounded_species_close(ammine_moles(&vessel), complexes);
    assert_conserved(&before, &vessel);
}

#[test]
fn acid_protonation_suppresses_the_finite_ammonia_complexes() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut basic = water();
    deposit(&mut basic, "AgCl", 0.001);
    deposit(&mut basic, "NH3", 0.002);
    solver.equilibrate(&mut basic).unwrap();
    let mut acidic = water();
    deposit(&mut acidic, "AgCl", 0.001);
    deposit(&mut acidic, "NH3", 0.002);
    deposit(&mut acidic, "HCl", 0.005);
    let before = acidic.clone();
    solver.equilibrate(&mut acidic).unwrap();
    assert!(acidic.solution.as_ref().unwrap().ph < 3.0);
    assert!(molality(&acidic, "NH4+") > molality(&acidic, "NH3"));
    assert!(ammine_moles(&acidic) < ammine_moles(&basic) * 0.01);
    assert!(acidic
        .solution
        .as_ref()
        .unwrap()
        .provenance
        .as_ref()
        .unwrap()
        .dataset
        .contains("minteq.v4.dat"));
    assert_conserved(&before, &acidic);
}

#[test]
fn nitrate_is_not_an_ammonia_ligand_but_ammonium_can_supply_one() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut nitrate = water();
    deposit(&mut nitrate, "AgNO3", 0.001);
    let nitrate_events = solver.equilibrate(&mut nitrate).unwrap();
    assert!(!has_ligand_inventory_boundary(&nitrate_events));
    assert!(nitrate
        .solution
        .as_ref()
        .unwrap()
        .provenance
        .as_ref()
        .unwrap()
        .dataset
        .contains("wateq4f.dat"));
    assert_eq!(ammine_moles(&nitrate), 0.0);
    let mut ammonium = water();
    deposit(&mut ammonium, "AgCl", 0.001);
    deposit(&mut ammonium, "NH4Cl", 0.002);
    solver.equilibrate(&mut ammonium).unwrap();
    assert!(ammonium
        .solution
        .as_ref()
        .unwrap()
        .provenance
        .as_ref()
        .unwrap()
        .dataset
        .contains("minteq.v4.dat"));
}

#[test]
fn mixing_separate_silver_and_ammonia_uses_target_capability_routing() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut silver = water();
    deposit(&mut silver, "AgCl", 0.001);
    solver.equilibrate(&mut silver).unwrap();
    let mut ammonia = water();
    deposit(&mut ammonia, "NH3", 0.002);
    solver.equilibrate(&mut ammonia).unwrap();
    for source in [&silver, &ammonia] {
        assert!(source
            .solution
            .as_ref()
            .unwrap()
            .provenance
            .as_ref()
            .unwrap()
            .dataset
            .contains("wateq4f.dat"));
    }
    let silver_before = serde_json::to_value(&silver).unwrap();
    let ammonia_before = serde_json::to_value(&ammonia).unwrap();
    let mut target = silver.clone();
    target.contents.extend(ammonia.contents.clone());
    target.solution = None;
    target.resolved.invalidate();
    let before = target.clone();
    assert!(solver
        .mix(&mut target, &silver, 1.0, &ammonia, 1.0)
        .is_none());
    // The caller's ordinary fallback must solve the combined composition,
    // rather than the native MIX helper silently keeping wateq4f coverage.
    let target_events = solver.equilibrate(&mut target).unwrap();
    assert!(has_ligand_inventory_boundary(&target_events));
    assert!(target
        .solution
        .as_ref()
        .unwrap()
        .provenance
        .as_ref()
        .unwrap()
        .routing
        .contains("silver-ammonia complexes"));
    assert!(ammine_moles(&target) > 0.0);
    assert_conserved(&before, &target);
    assert_eq!(serde_json::to_value(&silver).unwrap(), silver_before);
    assert_eq!(serde_json::to_value(&ammonia).unwrap(), ammonia_before);

    let mut direct = water();
    direct.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    deposit(&mut direct, "AgCl", 0.001);
    deposit(&mut direct, "NH3", 0.002);
    solver.equilibrate(&mut direct).unwrap();
    assert!(
        (target.solution.as_ref().unwrap().ph - direct.solution.as_ref().unwrap().ph).abs() < 1e-4
    );
    assert_rounded_species_close(ammine_moles(&target), ammine_moles(&direct));
    assert!((solid_silver(&target) - solid_silver(&direct)).abs() < 2e-8);
}
