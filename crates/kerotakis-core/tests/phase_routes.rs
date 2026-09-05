//! EXP-33: sublimation as a separation, and the hydrate mass ledger.
//!
//! Both of these are taught as arithmetic, so both are tested as arithmetic.
//! The sublimation quest's whole claim is that one component left and the
//! other did not; the crucible lesson's whole claim is that the missing mass
//! is exactly the water. Neither survives a rounding, so neither is allowed
//! one.

use kerotakis_core::phase_route::PhaseRouteEquilibrator;
use kerotakis_core::species::Phase;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn vessel_of(bench: &Bench) -> &vessel::Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    vessel_of(bench)
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, n: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(n),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("add")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn cool(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("cool")
}

/// Total mass of everything the vessel still holds.
fn mass(bench: &Bench) -> f64 {
    vessel_of(bench).mass().0
}

fn molar_mass(key: &str) -> f64 {
    species::lookup(&SpeciesId::new(key)).unwrap().molar_mass
}

// ── sublimation ────────────────────────────────────────────────────

#[test]
fn ammonium_chloride_leaves_and_common_salt_stays() {
    // The separation quest, in six lines. Heat the mixture past 338 °C and
    // one of the two components is simply gone from the crucible.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    add(&mut bench, &mut stack, "NaCl", 0.1);

    // Enough heat to pass ammonium chloride's 338 °C sublimation point and
    // nowhere near sodium chloride's 800.7 °C melting point — the window
    // the separation lives in. 0.2 mol NH4Cl + 0.1 mol NaCl is 21.9 J/K, so
    // 9 kJ lands somewhere near 700 K.
    let before = mass(&bench);
    let events = heat(&mut bench, &mut stack, 9_000.0);
    let t = vessel_of(&bench).temperature.0;
    assert!(
        (611.15..1073.85).contains(&t),
        "the separation window is between the two transitions; landed at {t} K"
    );

    assert!(
        moles(&bench, "NH4Cl", Phase::Solid) < 1e-9,
        "the ammonium chloride should have sublimed away"
    );
    assert!(
        (moles(&bench, "NaCl", Phase::Solid) - 0.1).abs() < 1e-12,
        "the salt does not sublime and must be untouched"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged {
                from: Phase::Solid,
                to: Phase::Gas,
                ..
            }
        )),
        "the phase route must be reported, not silent: {events:#?}"
    );

    // Mass accounting across the open boundary: what left is exactly the
    // ammonium chloride, to the digit.
    let lost = before - mass(&bench);
    assert!(
        (lost - 0.2 * molar_mass("NH4Cl")).abs() < 1e-9,
        "lost {lost} g, expected {} g",
        0.2 * molar_mass("NH4Cl")
    );
}

#[test]
fn a_sealed_vessel_keeps_the_vapour_and_gives_it_back_on_cooling() {
    // The cold-finger half. Sealed, nothing crosses the boundary at all, so
    // the balance never moves — and the solid comes back when it cools.
    //
    // The quantities are chosen so the flask survives: 0.02 mol of vapour in
    // two litres is about 0.6 atm at these temperatures, well inside school
    // glassware's rating. The first draft of this test sealed ten times as
    // much into half the volume and the vessel burst, correctly — that case
    // is now pinned below on purpose rather than by accident.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.02);
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(2.0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("seal");

    // 1.6 kJ, not 0.8: the vessel carries heat capacity of its own beyond
    // the 1.68 J/K of the sample, so the first attempt at this test stopped
    // at 534 K — short of the 611 K it was aiming for. The assertion below
    // is what caught it, which is why it asserts the temperature and not
    // just the phase.
    let sealed_mass = mass(&bench);
    heat(&mut bench, &mut stack, 1_600.0);
    let hot = vessel_of(&bench).temperature.0;
    assert!(
        hot >= 611.15,
        "should be past the sublimation point; {hot} K"
    );
    assert!(
        vessel_of(&bench).pressure.0 < kerotakis_core::senses::GLASS_BURST_PA,
        "the flask must survive: {} Pa",
        vessel_of(&bench).pressure.0
    );
    assert!(
        (moles(&bench, "NH4Cl", Phase::Gas) - 0.02).abs() < 1e-9,
        "a sealed vessel keeps its vapour; gas = {}",
        moles(&bench, "NH4Cl", Phase::Gas)
    );
    assert!(
        (mass(&bench) - sealed_mass).abs() < 1e-9,
        "a sealed vessel conserves mass exactly through a phase change"
    );

    cool(&mut bench, &mut stack, 900.0);
    let cold = vessel_of(&bench).temperature.0;
    assert!(
        cold < 611.15,
        "should be back below the threshold; {cold} K"
    );
    assert!(
        (moles(&bench, "NH4Cl", Phase::Solid) - 0.02).abs() < 1e-9,
        "deposition should return every mole to the solid"
    );
    assert!((mass(&bench) - sealed_mass).abs() < 1e-9);
}

#[test]
fn a_sealed_flask_of_subliming_solid_on_a_hot_plate_bursts() {
    // Not a curiosity: sublimation makes gas out of a solid, and a sealed
    // vessel has a limit. Ten times the sample in half the volume is a bomb,
    // and the engine says so rather than quietly holding the vapour.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(1.0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("seal");
    let events = heat(&mut bench, &mut stack, 60_000.0);
    assert!(
        events.iter().any(|e| matches!(e, Event::Burst { .. })),
        "the flask should burst rather than hold this: {events:#?}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::HazardWarning { rule, .. } if rule == "sealed-vessel-burst"
        )),
        "and it should say why it was dangerous"
    );
}

#[test]
fn nothing_sublimes_at_bench_temperature() {
    // The route must not fire on a cold bench, or every ammonium chloride
    // test in the suite would start evaporating.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    assert!((moles(&bench, "NH4Cl", Phase::Solid) - 0.2).abs() < 1e-12);
    assert!(moles(&bench, "NH4Cl", Phase::Gas) < 1e-12);
}

// ── dry ice: sublimation that costs something ──────────────────────

/// th-026, as arithmetic a kitchen thermometer can check.
///
/// 5 g of dry ice is 0.1136 mol and 2.86 kJ of sublimation enthalpy;
/// 100 g of water is 418 J/K. 2863 / 418 is 6.85 K, and that is the whole
/// claim. The vessel is open, so the carbon dioxide leaves — which is the
/// honest answer to the second half of the row's question and is pinned
/// here rather than glossed: seal the flask AFTERWARDS and you have
/// sealed an empty headspace.
#[test]
fn dry_ice_cools_the_water_it_is_dropped_into() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "water", 100.0 / molar_mass("water"));
    let before = vessel_of(&bench).temperature.0;
    assert!((before - 298.15).abs() < 1e-9);

    let n = 5.0 / molar_mass("dry_ice");
    let events = add(&mut bench, &mut stack, "dry_ice", n);

    let after = vessel_of(&bench).temperature.0;
    let drop = before - after;
    // The latent heat alone: the dry ice arrives at 194.65 K, but its
    // own heat capacity leaves with the gas when the route settles the
    // vessel from the sublimation point, so the water pays 25.2 kJ/mol
    // and nothing for warming the sample — the kitchen-thermometer figure
    // the tranche's own sanity check quotes.
    assert!(
        (drop - 6.85).abs() < 0.15,
        "5 g of dry ice cools 100 g of water by 6.85 K, got {drop:.2} K"
    );
    assert!(
        moles(&bench, "dry_ice", Phase::Solid) < 1e-9,
        "there is far more than enough heat in the water to take all of it"
    );
    // The vapour is carbon dioxide, not "dry ice gas": the route reads the
    // gas partner off the formula.
    let evolved: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(
        (evolved - n).abs() < 1e-9,
        "every mole should leave as carbon dioxide, got {evolved}"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged {
                from: Phase::Solid,
                to: Phase::Gas,
                ..
            }
        )),
        "the sublimation must be reported: {events:#?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TemperatureChanged { .. })),
        "and so must the cooling: {events:#?}"
    );
}

/// The half of th-026 its script cannot reach, and the capability is real.
///
/// Seal the flask FIRST and the carbon dioxide has nowhere to go: it fills
/// the headspace, the pressure rises above two atmospheres, and the water
/// still cools — less, because the cold gas stayed to be warmed.
#[test]
fn dry_ice_in_an_already_sealed_flask_fills_the_headspace() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "water", 100.0 / molar_mass("water"));
    bench
        .step_with(
            Operator::Seal {
                vessel: VesselId(0),
                headspace_volume: Liters(1.0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("seal");
    let sealed_mass = mass(&bench);

    let n = 0.05;
    add(&mut bench, &mut stack, "dry_ice", n);

    assert!(
        (moles(&bench, "CO2", Phase::Gas) - n).abs() < 1e-3,
        "a sealed vessel keeps the vapour: {} mol",
        moles(&bench, "CO2", Phase::Gas)
    );
    let p = vessel_of(&bench).pressure.0;
    assert!(
        (2.0e5..kerotakis_core::senses::GLASS_BURST_PA).contains(&p),
        "0.05 mol of gas on top of a litre of trapped air is a couple of atmospheres, \
         and the flask survives it; got {p} Pa"
    );
    let t = vessel_of(&bench).temperature.0;
    assert!(
        (290.0..297.5).contains(&t),
        "the water still cools, by rather less than it would in the open: {t} K"
    );
    // Mass is conserved exactly through a paired phase change, which is
    // only true because dry ice's molar mass IS carbon dioxide's.
    let expected = sealed_mass + n * molar_mass("dry_ice");
    assert!(
        (mass(&bench) - expected).abs() < 1e-9,
        "sealed mass {} vs expected {expected}",
        mass(&bench)
    );
}

/// The superheat correction, on its own.
///
/// A flask with nothing in it but dry ice has nothing to pay the latent
/// heat with, so nothing sublimes — but it also cannot be at 25 °C, which
/// is where `add` put it. It settles on its own sublimation point and
/// keeps its block, which is what an insulated flask of dry ice does.
#[test]
fn a_lone_block_of_dry_ice_settles_on_its_own_sublimation_point() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "dry_ice", 0.05);

    let t = vessel_of(&bench).temperature.0;
    assert!(
        (t - 194.65).abs() < 1e-6,
        "a flask of dry ice is at -78.5 C, not at room temperature; got {t} K"
    );
    assert!(
        (moles(&bench, "dry_ice", Phase::Solid) - 0.05).abs() < 1e-12,
        "with nothing to draw heat from, the block stays"
    );
}

/// The latent-heat table is deliberately partial, and this is the row
/// that proves it stayed that way.
///
/// Ammonium chloride's crucible separation is a mass ledger that nobody
/// weighed the heat of, and giving it an enthalpy it does not have would
/// move every temperature in the tests above it. A substance with no row
/// sublimes as it always did: all of it, at no cost.
#[test]
fn a_substance_with_no_enthalpy_row_sublimes_exactly_as_before() {
    use kerotakis_core::phase_route::{
        is_condensed_gas, sublimation_enthalpy, sublimation_product,
    };
    assert!(sublimation_enthalpy("NH4Cl").is_none());
    assert!(sublimation_enthalpy("dry_ice").is_some());
    // Ammonium chloride's vapour is ammonium chloride; dry ice's is
    // carbon dioxide, and the pairing is read off the formula.
    assert_eq!(sublimation_product("NH4Cl"), "NH4Cl");
    assert_eq!(sublimation_product("dry_ice"), "CO2");
    assert!(!is_condensed_gas("NH4Cl"));
    assert!(is_condensed_gas("dry_ice"));

    // And the behaviour: heat it past its point and it is simply gone,
    // with no temperature the enthalpy would have taken away.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "NH4Cl", 0.2);
    heat(&mut bench, &mut stack, 9_000.0);
    assert!(moles(&bench, "NH4Cl", Phase::Solid) < 1e-9);
}

// ── the cryogen route ──────────────────────────────────────────────

/// 10 mL of ethanol in 100 mL of liquid nitrogen: 0.171 mol and 2.881 mol.
fn ethanol_moles() -> f64 {
    10.0 * 0.789 / molar_mass("ethanol")
}

fn nitrogen_moles() -> f64 {
    100.0 * 0.807 / molar_mass("liquid_nitrogen")
}

/// th-123, and the answer is the one the question hopes for.
///
/// The nitrogen boils at 77.36 K taking the ethanol's heat with it; the
/// ethanol reaches 159.01 K, freezes, and the heat THAT releases boils
/// still more nitrogen rather than warming anything. There is nitrogen
/// left over, which is why a cold bath works.
#[test]
fn liquid_nitrogen_freezes_ethanol_solid() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "ethanol", ethanol_moles());
    let events = add(&mut bench, &mut stack, "liquid_nitrogen", nitrogen_moles());

    let t = vessel_of(&bench).temperature.0;
    assert!(
        (t - 77.36).abs() < 0.01,
        "the flask sits at the nitrogen's boiling point while any is left; got {t} K"
    );
    assert!(
        (moles(&bench, "ethanol", Phase::Solid) - ethanol_moles()).abs() < 1e-9,
        "every mole of ethanol should be a solid: {} mol",
        moles(&bench, "ethanol", Phase::Solid)
    );
    assert!(
        moles(&bench, "ethanol", Phase::Liquid) < 1e-9,
        "and none of it liquid"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged {
                from: Phase::Liquid,
                to: Phase::Solid,
                ..
            }
        )),
        "the freezing must be reported: {events:#?}"
    );
    // The nitrogen boils off AS NITROGEN, not as "liquid nitrogen gas":
    // the vapour is read off the formula, exactly as dry ice's is.
    let evolved: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "N2" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(evolved > 0.9 && evolved < 0.93, "{evolved} mol boiled off");
    assert!(
        moles(&bench, "liquid_nitrogen", Phase::Liquid) > 1.9,
        "and most of the dewar is still liquid: {} mol",
        moles(&bench, "liquid_nitrogen", Phase::Liquid)
    );
}

/// The coupling, as an energy identity computed outside the engine.
///
/// This is the assertion that would fail if freezing warmed the flask
/// instead of boiling nitrogen. The heat the ethanol gave up is its
/// sensible heat from room temperature down to 77.36 K plus its enthalpy
/// of fusion, and every joule of it has to come back out as boiled
/// nitrogen — nothing else in the vessel can hold it.
#[test]
fn the_heat_of_freezing_boils_nitrogen_rather_than_warming_the_flask() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "ethanol", ethanol_moles());
    add(&mut bench, &mut stack, "liquid_nitrogen", nitrogen_moles());

    let boiled = nitrogen_moles() - moles(&bench, "liquid_nitrogen", Phase::Liquid);
    let cp = ethanol_moles()
        * species::lookup(&SpeciesId::new("ethanol"))
            .unwrap()
            .heat_capacity;
    let sensible = cp * (298.15 - 77.36);
    let fusion = ethanol_moles() * 4930.0;
    let expected = (sensible + fusion) / 5570.0;
    assert!(
        (boiled - expected).abs() < 1e-6,
        "{boiled} mol boiled, but the ethanol only gave up {sensible:.0} + {fusion:.0} J, \
         which is {expected} mol of nitrogen"
    );
}

/// A dewar with nothing in it but the cryogen sits at its own boiling
/// point and keeps its contents — the same correction the lone block of
/// dry ice gets, in the other phase.
#[test]
fn a_lone_dewar_of_liquid_nitrogen_settles_on_its_boiling_point() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "liquid_nitrogen", 1.0);

    let t = vessel_of(&bench).temperature.0;
    assert!(
        (t - 77.36).abs() < 1e-6,
        "a dewar of liquid nitrogen is at -196 C, not at room temperature; got {t} K"
    );
    assert!(
        (moles(&bench, "liquid_nitrogen", Phase::Liquid) - 1.0).abs() < 1e-12,
        "with nothing to draw heat from, none of it boils"
    );
}

/// The tables are short on purpose, and the two absences are different.
///
/// Water must never appear in either: `solve::StateEquilibrator` owns the
/// solvent's freezing and boiling with the colligative shifts on top, and
/// two solvers moving the same ice would be a bug. Ethanol must never
/// appear in the vaporisation table: it would install a general boiling
/// route through the back door of a cryogen tranche, and this bench boils
/// nothing but water and the one cryogen.
#[test]
fn the_cryogen_tables_are_deliberately_short() {
    use kerotakis_core::phase_route::{fusion_enthalpy, is_condensed_gas, vaporisation_enthalpy};
    assert!(
        fusion_enthalpy("water").is_none(),
        "the solvent's fusion belongs to states.rs and must not be duplicated here"
    );
    assert!(vaporisation_enthalpy("water").is_none());
    assert!(
        vaporisation_enthalpy("ethanol").is_none(),
        "this bench has no general boiling route and must not pretend to"
    );
    assert!(fusion_enthalpy("ethanol").is_some());
    assert!(vaporisation_enthalpy("liquid_nitrogen").is_some());
    // And the superheat correction is gated on being a condensed gas,
    // which is why frozen ethanol can still melt.
    assert!(is_condensed_gas("liquid_nitrogen"));
    assert!(!is_condensed_gas("ethanol"));
}

/// Frozen ethanol is not a one-way street: warm it past 159 K with the
/// nitrogen gone and it melts again.
#[test]
fn frozen_ethanol_melts_when_the_nitrogen_is_gone() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "ethanol", 0.1);
    // Just enough nitrogen to freeze it and no more: 0.1 mol of ethanol
    // gives up 0.1 x 112.3 x 220.79 J of sensible heat plus 493 J of
    // fusion, which is 0.53 mol of nitrogen.
    add(&mut bench, &mut stack, "liquid_nitrogen", 0.53);
    assert!(
        moles(&bench, "ethanol", Phase::Solid) > 0.09,
        "the ethanol should be frozen first: {} mol solid",
        moles(&bench, "ethanol", Phase::Solid)
    );
    assert!(
        moles(&bench, "liquid_nitrogen", Phase::Liquid) < 0.02,
        "and the nitrogen essentially gone: {} mol",
        moles(&bench, "liquid_nitrogen", Phase::Liquid)
    );

    heat(&mut bench, &mut stack, 3_000.0);
    assert!(
        moles(&bench, "ethanol", Phase::Liquid) > 0.09,
        "warmed past its melting point it is a liquid again: {} mol liquid, {} mol solid",
        moles(&bench, "ethanol", Phase::Liquid),
        moles(&bench, "ethanol", Phase::Solid)
    );
}

// ── the hydrate ledger ─────────────────────────────────────────────

#[test]
fn heating_the_pentahydrate_drives_off_exactly_five_waters() {
    // The crucible lesson: weigh, heat, weigh. The difference is the water,
    // and "exactly" means exactly.
    let mut bench = Bench::new();
    let mut stack = stack();
    let n = 0.04;
    add(&mut bench, &mut stack, "chalcanthite", n);

    let before = mass(&bench);
    let expected_before = n * molar_mass("chalcanthite");
    assert!((before - expected_before).abs() < 1e-12);

    let events = heat(&mut bench, &mut stack, 40_000.0);

    let dehydrated = events
        .iter()
        .find_map(|e| match e {
            Event::Dehydrated { water, hydrate, .. } => Some((water.0, hydrate.0.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no dehydration reported: {events:#?}"));
    assert_eq!(dehydrated.1.as_str(), "chalcanthite");
    assert!(
        (dehydrated.0 - 5.0 * n).abs() < 1e-12,
        "five waters per formula unit, got {}",
        dehydrated.0
    );

    assert!(moles(&bench, "chalcanthite", Phase::Solid) < 1e-12);
    assert!((moles(&bench, "CuSO4", Phase::Solid) - n).abs() < 1e-12);

    // The residue weighs the anhydrous salt and nothing else.
    let after = mass(&bench);
    assert!(
        (after - n * molar_mass("CuSO4")).abs() < 1e-9,
        "residue {after} g, expected {} g",
        n * molar_mass("CuSO4")
    );
    // And the loss is the water, computed independently of the engine.
    let lost = before - after;
    assert!(
        (lost - 5.0 * n * molar_mass("water")).abs() < 1e-9,
        "lost {lost} g, expected {} g of water",
        5.0 * n * molar_mass("water")
    );
}

#[test]
fn the_water_goes_back_in_and_the_ledger_closes_both_ways() {
    // Round trip. A drop of water on the white powder and the blue returns,
    // with the same mass the crucible started with.
    let mut bench = Bench::new();
    let mut stack = stack();
    let n = 0.02;
    add(&mut bench, &mut stack, "chalcanthite", n);
    let start = mass(&bench);

    heat(&mut bench, &mut stack, 40_000.0);
    assert!((moles(&bench, "CuSO4", Phase::Solid) - n).abs() < 1e-12);

    // Cool it back down, then give it back exactly its own water.
    cool(&mut bench, &mut stack, 200_000.0);
    let events = bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(5.0 * n),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add water");

    assert!(
        events.iter().any(|e| matches!(e, Event::Hydrated { .. })),
        "the crystal should take its water back: {events:#?}"
    );
    assert!(
        (moles(&bench, "chalcanthite", Phase::Solid) - n).abs() < 1e-9,
        "every formula unit should be a pentahydrate again"
    );
    assert!(
        (mass(&bench) - start).abs() < 1e-9,
        "the round trip must return to the starting mass: {} vs {start}",
        mass(&bench)
    );
}

#[test]
fn plenty_of_water_dissolves_the_salt_instead_of_hydrating_it() {
    // The stated boundary, pinned: past the crystal's own stoichiometric
    // demand this bench stops calling it a hydrate, because dissolution is
    // what really happens and the aqueous engine owns that.
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "chalcanthite", 0.01);
    heat(&mut bench, &mut stack, 40_000.0);
    cool(&mut bench, &mut stack, 200_000.0);

    let events = bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(5.0),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add water");
    assert!(
        !events.iter().any(|e| matches!(e, Event::Hydrated { .. })),
        "a beaker of water is not a hydration: {events:#?}"
    );
}

#[test]
fn epsomite_carries_its_seven_waters_through_the_same_ledger() {
    // The hydrate machinery is general: it reads the water count off the
    // formula, so the salt that was already on the shelf works too.
    let mut bench = Bench::new();
    let mut stack = stack();
    let n = 0.03;
    add(&mut bench, &mut stack, "epsomite", n);
    let before = mass(&bench);
    heat(&mut bench, &mut stack, 60_000.0);

    assert!((moles(&bench, "MgSO4", Phase::Solid) - n).abs() < 1e-12);
    let lost = before - mass(&bench);
    assert!(
        (lost - 7.0 * n * molar_mass("water")).abs() < 1e-9,
        "lost {lost} g, expected seven waters ({} g)",
        7.0 * n * molar_mass("water")
    );
}

#[test]
fn the_hydrate_survives_a_warm_bench_and_only_goes_at_its_own_temperature() {
    let mut bench = Bench::new();
    let mut stack = stack();
    add(&mut bench, &mut stack, "chalcanthite", 0.02);
    // A few hundred joules is a warm beaker, not a crucible.
    heat(&mut bench, &mut stack, 300.0);
    assert!(
        (moles(&bench, "chalcanthite", Phase::Solid) - 0.02).abs() < 1e-12,
        "the crystal water is not driven off by a warm bench"
    );
}

/// The honesty pass must not apologise for a solid the cold has made: frozen
/// ethanol beside liquid nitrogen is a phase route's answer, not a gap.
#[test]
fn the_honesty_pass_does_not_call_frozen_ethanol_unmodelled() {
    use kerotakis_core::*;
    let mut v = vessel::Vessel::new(VesselId(0), "flask");
    v.temperature = Kelvin(150.0);
    v.deposit(
        SpeciesId::new("ethanol"),
        Moles(0.17),
        species::Phase::Solid,
    );
    v.deposit(
        SpeciesId::new("liquid_nitrogen"),
        Moles(3.0),
        species::Phase::Liquid,
    );
    let events = HonestyEquilibrator
        .equilibrate(&mut v)
        .expect("the pass runs");
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("ethanol")
        ) || matches!(e, Event::Inert { species, .. } if species.0 == "ethanol")),
        "{events:?}"
    );
}

/// Pouring a cryogen cools the flask on the pour, not by a correction
/// afterwards: `add` deposits a condensed gas at its own transition
/// temperature and the adiabatic mix does the rest.
#[test]
fn a_cryogen_arrives_cold_and_a_salt_arrives_at_room_temperature() {
    use kerotakis_core::*;
    let mut bench = Bench::new();
    let v = VesselId(0);
    let add = |bench: &mut Bench, key: &str, moles: f64| {
        bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            })
            .expect("add")
    };
    add(&mut bench, "ethanol", 0.17);
    assert!((bench.vessel(v).unwrap().temperature.0 - Kelvin::STANDARD.0).abs() < 1e-9);
    let events = add(&mut bench, "liquid_nitrogen", 2.9);
    let after = bench.vessel(v).unwrap().temperature.0;
    assert!(after < 200.0, "the pour cools the flask: {after} K");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::TemperatureChanged { from, to, .. } if from.0 > to.0
        )),
        "{events:?}"
    );
    assert!(kerotakis_core::phase_route::arrives_at_k("liquid_nitrogen")
        .is_some_and(|k| (k - 77.36).abs() < 0.5));
    assert!(kerotakis_core::phase_route::arrives_at_k("dry_ice")
        .is_some_and(|k| (k - 194.65).abs() < 0.5));
    assert!(kerotakis_core::phase_route::arrives_at_k("NaCl").is_none());
    assert!(kerotakis_core::phase_route::arrives_at_k("ethanol").is_none());
}
