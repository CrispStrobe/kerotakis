//! L2g through the bench: heating a solid until it decomposes, and
//! burning magnesium — the two demonstrations every school does.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
    v: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, v: VesselId, kj: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: v,
                energy: Joules(kj * 1000.0),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

#[test]
fn species_map_into_the_nasa_data_by_formula() {
    // Nothing lists these pairs; they are matched by composition.
    assert_eq!(kerotakis_cea::cea_name("CaCO3"), Some("CaCO3(cr)"));
    assert_eq!(kerotakis_cea::cea_name("CaO"), Some("CaO(cr)"));
    assert_eq!(kerotakis_cea::cea_name("MgO"), Some("MgO(cr)"));
    assert_eq!(kerotakis_cea::cea_name("Fe2O3"), Some("Fe2O3(cr)"));
    assert_eq!(kerotakis_cea::cea_name("O2"), Some("O2"));
    assert_eq!(kerotakis_cea::cea_name("CO2"), Some("CO2"));
    assert_eq!(
        kerotakis_cea::cea_name("ethanol"),
        Some("C2H5OH(L)"),
        "liquid fuel uses CEA's separate feed-only thermochemistry"
    );
    assert_eq!(
        kerotakis_cea::cea_name("methanol"),
        Some("CH3OH(L)"),
        "methanol has its own feed-only record and must not borrow ethanol's"
    );
    assert_eq!(
        kerotakis_cea::cea_name("isopropanol"),
        None,
        "an isopropanol identity must not borrow another C3H8O isomer's thermochemistry"
    );
}

#[test]
fn steel_wool_ignites_to_named_iron_oxide_in_open_air() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    let wool = script::parse_op("add v1 Stahlwolle 1g")
        .expect("localized steel-wool command")
        .expect("operator");
    bench
        .step_with(wool, &mut stack, &PermissiveScreen)
        .expect("add steel wool");

    let events = bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("ignite steel wool");
    let vessel = bench.vessel(v).expect("vessel");

    assert!(
        vessel.moles_of(&SpeciesId::new("Fe")).0 < 1e-6,
        "the resolved iron fibres should be consumed: {events:?}"
    );
    assert!(
        vessel.moles_of(&SpeciesId::new("Fe2O3")).0 > 0.008,
        "air oxygen should become conserved named hematite: {:?}",
        vessel.contents
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Ignited { energy_j: Some(energy), .. } if *energy > 1_000.0
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Precipitated { species, .. } if species.0 == "Fe2O3"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactionOccurred { equation, .. } if equation.contains("Fe₂O₃")
    )));
}

#[test]
fn a_flame_really_burns_liquid_ethanol() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "ethanol", 0.010);

    let events = bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("ignite ethanol");

    assert!(
        bench
            .vessel(v)
            .unwrap()
            .moles_of(&SpeciesId::new("ethanol"))
            .0
            < 1e-6,
        "the fuel is consumed: {events:?}"
    );
    assert!(events.iter().any(
        |event| matches!(event, Event::GasEvolved { species, moles, .. }
            if species.0 == "CO2" && moles.0 > 0.015)
    ));
    assert!(events.iter().any(
        |event| matches!(event, Event::GasEvolved { species, moles, .. }
            if species.0 == "water" && moles.0 > 0.020)
    ));
    assert!(events.iter().any(
        |event| matches!(event, Event::Ignited { energy_j: Some(energy), .. }
            if *energy > 1_000.0)
    ));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ThermalEquilibrium { provenance, .. }
            if provenance.dataset.contains("CEA")
                && provenance.dataset_sources.iter().any(|source| source.contains("C2H5OH(L)"))
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::ReactionOccurred { equation, .. } if equation.contains("C₂H₅OH")
    )));
}

/// Energy released per mole of fuel by the `Ignited` event, in kJ/mol.
fn released_kj_per_mol(events: &[Event], moles: f64) -> f64 {
    let energy = events
        .iter()
        .find_map(|event| match event {
            Event::Ignited {
                energy_j: Some(energy),
                ..
            } => Some(*energy),
            _ => None,
        })
        .expect("a flame that caught reports its energy");
    energy / moles / 1000.0
}

#[test]
fn a_flame_really_burns_liquid_methanol() {
    let mut bench = Bench::new();
    let mut methanol_stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut methanol_stack, v, "methanol", 0.010);

    let events = bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut methanol_stack,
            &PermissiveScreen,
        )
        .expect("ignite methanol");

    assert!(
        bench
            .vessel(v)
            .unwrap()
            .moles_of(&SpeciesId::new("methanol"))
            .0
            < 1e-6,
        "the fuel is consumed: {events:?}"
    );
    // CH3OH + 1.5 O2 -> CO2 + 2 H2O: one carbon, two waters per mole.
    assert!(
        events.iter().any(
            |event| matches!(event, Event::GasEvolved { species, moles, .. }
            if species.0 == "CO2" && moles.0 > 0.008)
        ),
        "methanol's single carbon must leave as CO2: {events:?}"
    );
    assert!(
        events.iter().any(
            |event| matches!(event, Event::GasEvolved { species, moles, .. }
            if species.0 == "water" && moles.0 > 0.016)
        ),
        "two waters per mole of methanol: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::TemperatureChanged { from, to, .. } if to.0 > from.0
        )),
        "an adiabatic flame must run hotter than the spark that lit it: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::ThermalEquilibrium { provenance, .. }
                if provenance.dataset.contains("CEA")
                    && provenance.dataset_sources.iter().any(|source| source.contains("CH3OH(L)"))
        )),
        "the feed record that carried the energy must be cited: {events:?}"
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::ReactionOccurred { equation, .. } if equation.contains("CH₃OH")
        )),
        "the burn is announced as an equation: {events:?}"
    );

    // How much energy, and why this band.
    //
    // The textbook number is the standard enthalpy of combustion of LIQUID
    // methanol to CO2(g) and LIQUID water:
    //
    //   ΔcH°(CH3OH, l, 298.15 K) = −726.1 kJ/mol
    //
    // Provenance: it is the Hess-law sum of standard formation enthalpies —
    // ΔfH°(CO2, g) = −393.51 and ΔfH°(H2O, l) = −285.83 kJ/mol (CODATA Key
    // Values for Thermodynamics), minus ΔfH°(CH3OH, l) = −238.91 kJ/mol,
    // which is the value carried by the CH3OH(L) record this very solve
    // reads (vendor/nasa-cea/thermo.inp, "MeOH. TRC(12/87) p5000"):
    //   −393.51 + 2(−285.83) − (−238.91) = −726.26 kJ/mol.
    //
    // What the solver reports is deliberately NOT that number, and the band
    // below encodes the difference rather than hiding it. `ignite` is a
    // flame: its water leaves as VAPOUR, and the release is evaluated at the
    // ignition-zone temperature, where the feed record has already handed
    // over to the gas-phase polynomial (thermal.rs::enthalpy_within_record).
    // Condensing the two product waters would add 2 × 44 = 88 kJ/mol, so the
    // honest expectation for a vapour-product flame is
    //   −726 + 88 ≈ −638 kJ/mol from the liquid, and ≈ −676 kJ/mol from
    // vapour methanol (the 38 kJ/mol enthalpy of vaporisation of methanol),
    // shifting by a few tens of kJ/mol between 298 K and the flame zone.
    // A band of 560–760 kJ/mol therefore brackets every defensible reading
    // of "methanol burned" while still failing an answer that is out by a
    // factor — which is what this assertion is for.
    let methanol_kj = released_kj_per_mol(&events, 0.010);
    assert!(
        (560.0..=760.0).contains(&methanol_kj),
        "methanol's release should sit near its −726 kJ/mol standard \
         combustion enthalpy corrected for vapour water, got {methanol_kj:.1} kJ/mol"
    );

    // The alcohol-burn quest's stated comparison: ethanol carries a second
    // carbon and releases about 1367 kJ/mol against methanol's 726, so per
    // MOLE ethanol must win by a clear margin. Both numbers come from the
    // same solver and the same dataset, so the comparison is meaningful.
    let mut ethanol_bench = Bench::new();
    let mut ethanol_stack = stack();
    add(&mut ethanol_bench, &mut ethanol_stack, v, "ethanol", 0.010);
    let ethanol_events = ethanol_bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut ethanol_stack,
            &PermissiveScreen,
        )
        .expect("ignite ethanol");
    let ethanol_kj = released_kj_per_mol(&ethanol_events, 0.010);
    assert!(
        ethanol_kj > methanol_kj * 1.5,
        "per mole, ethanol must clearly out-release methanol \
         (literature ratio 1367/726 ≈ 1.88): ethanol {ethanol_kj:.1} \
         vs methanol {methanol_kj:.1} kJ/mol"
    );
}

#[test]
fn aqueous_rubbing_alcohol_keeps_an_explicit_combustion_boundary() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let add = kerotakis_core::script::parse_op("add v1 Isopropanol_70% 10mL")
        .expect("valid rubbing-alcohol command")
        .expect("operator");
    bench
        .step_with(add, &mut stack, &PermissiveScreen)
        .expect("add rubbing alcohol");
    let before = bench.vessels[0].moles_of(&SpeciesId::new("isopropanol"));

    let events = bench
        .step_with(
            Operator::Ignite {
                vessel: VesselId(0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("evaluate ignition boundary");

    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("isopropanol")),
        before,
        "an unmodelled flame must not consume fuel"
    );
    assert!(events.iter().any(|event| matches!(event,
        Event::NotYetModeled { what, .. } if what.contains("combustion")
    )));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Ignited { .. } | Event::DidNotIgnite { .. })));
}

#[test]
fn chalk_in_a_cold_crucible_just_sits_there() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "CaCO3", 0.1);

    let vessel = bench.vessel(v).unwrap();
    assert!(
        (vessel.moles_of(&SpeciesId::new("CaCO3")).0 - 0.1).abs() < 1e-3,
        "chalk is stable at room temperature: {:?}",
        vessel.contents
    );
}

#[test]
fn heating_chalk_hard_enough_calcines_it() {
    // The lime kiln, computed: heat until the carbonate gives way, and
    // quicklime is left with the CO2 gone.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "CaCO3", 0.1);
    // ~0.1 mol of chalk needs a few hundred kJ to reach kiln temperature.
    let events = heat(&mut bench, &mut stack, v, 40.0);

    let vessel = bench.vessel(v).unwrap();
    assert!(
        vessel.temperature.0 > 1100.0,
        "the crucible got hot: {:.0} K",
        vessel.temperature.0
    );
    assert!(
        vessel.moles_of(&SpeciesId::new("CaO")).0 > 0.05,
        "quicklime formed: {:?}",
        vessel.contents
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "CO2")),
        "and the carbon dioxide left: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ThermalEquilibrium { .. })),
        "the thermal solver reported itself"
    );
}

#[test]
fn burning_magnesium_makes_white_oxide_and_a_great_deal_of_heat() {
    // Mg + ½O₂ → MgO, ΔH ≈ −601 kJ/mol: the ribbon that burns white in every
    // school lab. Oxygen comes from the vessel's atmosphere, not from the
    // inventory.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "Mg", 0.05);
    let before = bench.vessel(v).unwrap().temperature.0;
    // A spark: enough to start it.
    let events = heat(&mut bench, &mut stack, v, 1.0);

    let vessel = bench.vessel(v).unwrap();
    assert!(
        vessel.moles_of(&SpeciesId::new("MgO")).0 > 0.04,
        "the ribbon becomes magnesium oxide: {:?}",
        vessel.contents
    );
    assert!(
        vessel.moles_of(&SpeciesId::new("Mg")).0 < 0.01,
        "and little metal survives"
    );
    assert!(
        vessel.temperature.0 > before + 200.0,
        "burning magnesium releases a great deal of heat: {before:.0} K → {:.0} K",
        vessel.temperature.0
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Consumed { species, .. } if species.0 == "Mg")),
        "the metal is reported consumed: {events:?}"
    );
}

#[test]
fn the_thermal_answer_carries_its_provenance() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "Mg", 0.05);
    let events = heat(&mut bench, &mut stack, v, 1.0);

    let p = events
        .iter()
        .find_map(|e| match e {
            Event::ThermalEquilibrium { provenance, .. } => Some(provenance.clone()),
            _ => None,
        })
        .expect("thermal equilibrium reported");
    assert!(p.dataset.contains("CEA"));
    assert!(p.model.contains("NASA-9"));
    assert!(
        !p.dataset_sources.is_empty(),
        "the species' own citations travel with the answer"
    );
}

#[test]
fn a_solution_is_left_to_the_aqueous_engine() {
    // Liquid water present and cool: the thermal solver must decline, so
    // the honesty pass speaks instead of a wrong answer being computed.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.0);
    let events = add(&mut bench, &mut stack, v, "CaCO3", 0.01);
    // What this test is for is that the bench does not FABRICATE: with no
    // aqueous engine in the stack, nothing may come out that only a
    // speciation could have known. That is still checked, and it is the
    // half worth keeping.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolutionCharacterized { .. })),
        "no aqueous engine in this stack, so no solution may be characterised: {events:?}"
    );
    // The other half has moved on. It used to demand `NotYetModeled`, and
    // chalk now answers instead: its reviewed solubility is 0.0013 g per
    // 100 mL, which is a handbook number and not an aqueous computation, so
    // saying it needs no engine. A stated position is what was being
    // guarded; which KIND of stated position is a fact about how much the
    // bench has been taught, and pinning it to the apology would have made
    // teaching it a test failure.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. } | Event::Inert { .. })),
        "the bench must take a position rather than fall silent: {events:?}"
    );
}

#[test]
fn a_spark_held_to_something_that_will_not_burn_leaves_no_trace() {
    // Salt does not burn. It must not be left hot, and a nanomole of
    // equilibrium chlorine must not be announced as a gas cloud —
    // bookkeeping is exact, observation has a detection limit.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "NaCl", 0.085);
    let before = bench.vessel(v).unwrap().temperature.0;

    let events = bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");

    assert!(
        !events.iter().any(|e| matches!(e, Event::GasEvolved { .. })),
        "a trace is not an observation: {events:?}"
    );
    assert!(
        (bench.vessel(v).unwrap().temperature.0 - before).abs() < 1.0,
        "the spark dissipated"
    );
    assert!(
        (bench.vessel(v).unwrap().moles_of(&SpeciesId::new("NaCl")).0 - 0.085).abs() < 1e-6,
        "and the salt is untouched"
    );
    // But it does colour the flame — the flame test.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::FlameTest { colour, .. } if colour.contains("yellow"))),
        "sodium paints the flame yellow: {events:?}"
    );
}

#[test]
fn igniting_magnesium_gains_mass_from_the_air() {
    // The result that surprises every student: burning makes it heavier,
    // because the oxygen it takes from the air is heavier than nothing.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "Mg", 0.0494);
    let before = bench.vessel(v).unwrap().mass().0;

    let events = bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");

    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Ignited { flame: Some(f), .. } if f.contains("white"))),
        "it burns with a blinding white light: {events:?}"
    );
    let released = events
        .iter()
        .find_map(|event| match event {
            Event::Ignited {
                energy_j: Some(joules),
                ..
            } => Some(*joules),
            _ => None,
        })
        .expect("the computed reaction energy travels on the ignition event");
    assert!(
        released > 10_000.0,
        "burning 0.0494 mol Mg should release a visibly large amount of heat: {released:.1} J"
    );
    let after = bench.vessel(v).unwrap().mass().0;
    assert!(
        after > before * 1.5,
        "burning magnesium gains mass: {before:.2} g → {after:.2} g"
    );
    // Magnesium's flame is genuinely around 3000 K.
    let t = bench.vessel(v).unwrap().temperature.0;
    assert!(
        (2500.0..3600.0).contains(&t),
        "flame temperature in the right range, got {t:.0} K"
    );
}
