use kerotakis_core::ops::Event;
use kerotakis_core::script::parse_op;
use kerotakis_core::{Bench, Moles, SpeciesId};

fn run(bench: &mut Bench, command: &str) -> Vec<Event> {
    bench
        .step(parse_op(command).expect("valid command").expect("operator"))
        .expect("step succeeds")
}

fn prepared(yeast: Option<&str>) -> Bench {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 table_sugar 10g");
    if let Some(yeast) = yeast {
        run(&mut bench, &format!("add v1 {yeast}"));
    }
    bench
}

#[test]
fn hydrated_yeast_converts_finite_sucrose_with_balanced_stoichiometry() {
    let mut bench = prepared(Some("dry_yeast 1g"));
    let sucrose_before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
    let events = run(&mut bench, "wait 600s");
    let step = events.iter().find_map(|event| match event {
        Event::Fermented {
            sucrose_moles,
            ethanol_moles,
            carbon_dioxide_moles,
            active_yeast_grams,
            ..
        } => Some((
            sucrose_moles.0,
            ethanol_moles.0,
            carbon_dioxide_moles.0,
            *active_yeast_grams,
        )),
        _ => None,
    });
    let (sugar, ethanol, carbon_dioxide, active_yeast) = step.expect("fermentation event");
    assert!(sugar > 1e-4);
    assert!((ethanol - 4.0 * sugar).abs() < 1e-12);
    assert!((carbon_dioxide - 4.0 * sugar).abs() < 1e-12);
    assert!(active_yeast > 0.5 && active_yeast <= 1.0);
    assert!(
        (bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0 - (sucrose_before - sugar)).abs()
            < 1e-12
    );
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0 - ethanol).abs() < 1e-12);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { reaction, species, moles, .. }
            if reaction == "yeast-sucrose-fermentation"
                && species.0 == "CO2"
                && (moles.0 - carbon_dioxide).abs() < 1e-12
    )));
}

#[test]
fn sugar_water_without_yeast_does_not_ferment() {
    let mut bench = prepared(None);
    let before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose"));
    let events = run(&mut bench, "wait 600s");
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("sucrose")),
        before
    );
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("ethanol")),
        Moles(0.0)
    );
}

#[test]
fn already_hydrated_fresh_yeast_starts_faster_than_dry_yeast() {
    let produced = |yeast: &str| {
        let mut bench = prepared(Some(yeast));
        run(&mut bench, "wait 2s")
            .into_iter()
            .find_map(|event| match event {
                Event::Fermented { sucrose_moles, .. } => Some(sucrose_moles.0),
                _ => None,
            })
            .unwrap_or(0.0)
    };
    let dry = produced("dry_yeast 1g");
    let fresh = produced("fresh_yeast 3.333g");
    assert!(fresh > dry * 2.0, "dry={dry}, fresh={fresh}");
}

// ── The three cultures beside the yeast ──────────────────────────────

fn lactic(bench: &Bench) -> f64 {
    bench.vessels[0].moles_of(&SpeciesId::new("lactic_acid")).0
}

fn milk_solids(bench: &Bench) -> f64 {
    bench.vessels[0]
        .unresolved_materials
        .iter()
        .filter(|portion| portion.recipe_id == "household/whole-milk-surrogate")
        .map(|portion| portion.amount)
        .sum()
}

fn yoghurt_at(celsius: Option<&str>) -> Bench {
    let mut bench = Bench::new();
    match celsius {
        Some(at) => run(&mut bench, &format!("add v1 milk 100mL @ {at}")),
        None => run(&mut bench, "add v1 milk 100mL"),
    };
    run(&mut bench, "add v1 yoghurt_culture 1g");
    run(&mut bench, "wait 8h");
    bench
}

#[test]
fn a_lactic_culture_turns_milk_sugar_into_acid_and_makes_no_gas() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 milk 100mL");
    let solids_before = milk_solids(&bench);
    run(&mut bench, "add v1 yoghurt_culture 1g");
    // FINDING, not asserted here because it is older than this culture:
    // `Vessel::mass` counts unresolved material only for HomogeneousLiquid
    // recipes, so a gram of granular culture — like a gram of dry yeast,
    // or of flour — weighs nothing to the bench. Milk is a liquid, so the
    // conservation this test does check is real.
    let mass_with_culture = bench.vessels[0].mass().0;
    let events = run(&mut bench, "wait 8h");
    assert!(lactic(&bench) > 0.0, "no lactic acid: {events:?}");
    // The lactose came out of conserved unresolved milk solids, and the
    // vessel's mass did not move: acid mass in, solids and water out.
    assert!(milk_solids(&bench) < solids_before);
    assert!(
        (bench.vessels[0].mass().0 - mass_with_culture).abs() < 1e-9,
        "mass moved: {} -> {}",
        mass_with_culture,
        bench.vessels[0].mass().0
    );
    // Homolactic means no gas. A yoghurt pot does not rise.
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("CO2")),
        Moles(0.0)
    );
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("ethanol")),
        Moles(0.0)
    );
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
}

#[test]
fn refrigeration_slows_the_yoghurt_culture_down() {
    let warm = lactic(&yoghurt_at(None));
    let cold = lactic(&yoghurt_at(Some("5C")));
    assert!(warm > 0.0);
    assert!(cold >= 0.0);
    assert!(
        cold * 10.0 < warm,
        "a refrigerator must slow it by much more than a tenth: warm={warm}, cold={cold}"
    );
}

#[test]
fn acetic_bacteria_oxidise_ethanol_to_vinegar_and_stop_without_air() {
    let vinegar = |with_oxygen: bool| {
        let mut bench = Bench::new();
        run(&mut bench, "add v1 water 100mL");
        run(&mut bench, "add v1 ethanol 0.1mol");
        run(&mut bench, "add v1 acetobacter 1g");
        if with_oxygen {
            run(&mut bench, "add v1 O2 0.1mol");
        }
        let before = bench.vessels[0].mass().0;
        run(&mut bench, "wait 24h");
        let acid = bench.vessels[0].moles_of(&SpeciesId::new("CH3COOH")).0;
        (acid, (bench.vessels[0].mass().0 - before).abs())
    };
    let (aerated, mass_drift) = vinegar(true);
    let (sealed_off, _) = vinegar(false);
    assert!(aerated > 0.0, "no vinegar was made");
    assert!(mass_drift < 1e-9, "mass moved by {mass_drift}");
    assert_eq!(
        sealed_off, 0.0,
        "an oxidation with no oxygen in the vessel must do nothing"
    );
}

#[test]
fn sourdough_makes_acid_and_gas_from_the_same_sugar() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 sourdough_starter 50g");
    run(&mut bench, "add v1 flour 50g");
    run(&mut bench, "add v1 water 50mL");
    let before = bench.vessels[0].mass().0;
    let events = run(&mut bench, "wait 8h");
    assert!(lactic(&bench) > 0.0, "no acid: {events:?}");
    assert!(
        bench.vessels[0].moles_of(&SpeciesId::new("CO2")).0 > 0.0,
        "no gas: {events:?}"
    );
    assert!(bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0 > 0.0);
    // The heterolactic equation balances exactly on paper; the residue
    // here is the registry's own sucrose molar mass, 342.2965 against the
    // 342.2970 the same atomic weights add up to. The alcoholic route has
    // carried that 0.0005 g/mol since it was written, so the tolerance
    // names the rounding rather than hiding it.
    let drift = (bench.vessels[0].mass().0 - before).abs();
    assert!(drift < 1e-5, "mass drifted by {drift} g");
    // The gas is announced rather than deposited silently.
    assert!(events.iter().any(|event| matches!(
        event,
        Event::GasProduced { species, .. } if species.0 == "CO2"
    )));
    // Acid and gas in the ratio the balanced heterolactic equation gives.
    let acid = lactic(&bench);
    let gas = bench.vessels[0].moles_of(&SpeciesId::new("CO2")).0;
    assert!((acid - gas).abs() < 1e-12, "acid={acid}, gas={gas}");
}

#[test]
fn a_sourdough_starter_in_milk_ferments_nothing_rather_than_making_silent_gas() {
    let mut bench = Bench::new();
    run(&mut bench, "add v1 milk 100mL");
    run(&mut bench, "add v1 sourdough_starter 20g");
    run(&mut bench, "wait 8h");
    // The heterolactic route deliberately reads dissolved sucrose only,
    // because the gas it makes is announced through the sucrose count and
    // the milk's lactose has no such count. The starter carries a little
    // sugar of its own, so that much ferments and the milk's does not.
    assert_eq!(milk_solids(&bench), {
        let mut control = Bench::new();
        run(&mut control, "add v1 milk 100mL");
        milk_solids(&control)
    });
}

#[test]
fn baker_s_yeast_still_runs_the_route_it_always_did() {
    let mut bench = prepared(Some("dry_yeast 1g"));
    let events = run(&mut bench, "wait 600s");
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Fermented { .. })));
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("lactic_acid")),
        Moles(0.0)
    );
}

// ── Extensivity: twice the experiment is twice the yoghurt ───────────

/// THE PROPERTY THAT WAS BROKEN, and the reason `fermentation.rs` divides
/// by a liquid volume.
///
/// Doubling every quantity in a script is not a new experiment: it is the
/// same experiment in a bigger beaker, so every amount must double and
/// nothing else may move. Until 2026-09-16 this file's rate multiplied the
/// declared constant by the GRAMS of culture, so doubling a batch doubled
/// the rate as well as the substrate and the product came out four times
/// larger. The corpus perturbation suite measured it on `bio-070`: 2.716e-5
/// mol of total lactic acid became 1.086e-4, a factor of 3.999, where the
/// suite wanted 2.0.
///
/// The tolerance is 1e-12 relative and that is not optimism. The rate is
/// `reference * grams * 1 L / litres`, the grams and the litres both double
/// exactly, and the extent that comes out is bit-for-bit the same number —
/// so the only thing left to differ is the substrate, which doubles. A
/// loose tolerance here would pass a rate that was first order in the
/// square root of the batch.
#[test]
fn doubling_every_quantity_doubles_the_alcoholic_product() {
    let brew = |scale: f64| {
        let mut bench = Bench::new();
        run(&mut bench, &format!("add v1 water {}mL", 100.0 * scale));
        run(&mut bench, &format!("add v1 table_sugar {}g", 10.0 * scale));
        run(&mut bench, &format!("add v1 dry_yeast {}g", 1.0 * scale));
        run(&mut bench, "wait 600s");
        (
            bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0,
            bench.vessels[0].moles_of(&SpeciesId::new("CO2")).0,
        )
    };
    let (ethanol, gas) = brew(1.0);
    let (ethanol_doubled, gas_doubled) = brew(2.0);
    assert!(ethanol > 0.0 && gas > 0.0, "nothing fermented at all");
    for (single, doubled, what) in [
        (ethanol, ethanol_doubled, "ethanol"),
        (gas, gas_doubled, "carbon dioxide"),
    ] {
        let ratio = doubled / single;
        assert!(
            (ratio - 2.0).abs() < 1e-12,
            "{what} went {single} -> {doubled}, a factor of {ratio} where a \
             doubled experiment must give exactly 2"
        );
    }
}

/// The same property on the lactic route, which is where the defect was
/// found. `bio-070` is this script at 5 C.
#[test]
fn doubling_every_quantity_doubles_the_lactic_product() {
    let ferment = |scale: f64| {
        let mut bench = Bench::new();
        run(&mut bench, &format!("add v1 milk {}mL @ 5C", 100.0 * scale));
        run(
            &mut bench,
            &format!("add v1 yoghurt_culture {}g", 1.0 * scale),
        );
        run(&mut bench, "wait 8h");
        lactic(&bench)
    };
    let single = ferment(1.0);
    let doubled = ferment(2.0);
    assert!(single > 0.0, "the refrigerated culture made no acid at all");
    let ratio = doubled / single;
    assert!(
        (ratio - 2.0).abs() < 1e-12,
        "lactic acid went {single} -> {doubled}, a factor of {ratio}"
    );
}

/// The other half of the same claim: a fixed dose in HALF the liquid is
/// twice as concentrated and must run twice as fast. Without this, a rate
/// that ignored the volume entirely — the defect's predecessor — would
/// still pass the two extensivity tests above, because a constant rate is
/// extensive too.
#[test]
fn halving_the_liquid_at_a_fixed_dose_doubles_the_rate() {
    let extent = |millilitres: f64| {
        let mut bench = Bench::new();
        run(&mut bench, &format!("add v1 water {millilitres}mL"));
        run(&mut bench, "add v1 table_sugar 10g");
        run(&mut bench, "add v1 dry_yeast 1g");
        let before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
        run(&mut bench, "wait 60s");
        let after = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
        // The first-order extent, read back out of what it consumed.
        1.0 - after / before
    };
    let wide = extent(200.0);
    let narrow = extent(100.0);
    assert!(wide > 0.0, "nothing fermented in the larger beaker");
    // Sixty seconds is short enough that 1 - exp(-kt) is still nearly kt,
    // so the ratio of extents is close to the ratio of rates. It is NOT
    // exactly 2, and the bound says by how much rather than hiding it.
    let ratio = narrow / wide;
    assert!(
        (1.9..=2.0).contains(&ratio),
        "half the water should roughly double the extent: {wide} -> {narrow} \
         is a factor of {ratio}"
    );
}

// ── The calibration anchors: the two points the rates were fitted to ──

/// THE LACTIC ANCHOR, and one of only two tests in this file that pin a
/// magnitude rather than an ordering.
///
/// The rate constant is not a measurement and cannot be checked against
/// one: nothing measures an activity per gram of a culture that names no
/// strain. What CAN be checked, and is checked here, is the published
/// fermentation the constant was fitted to reproduce.
///
/// THE FIT. Kim, Oh and Imm 2018 (Korean J Food Sci Anim Resour 38:273-281,
/// doi:10.5851/kosfa.2018.38.2.273) held milk carrying a commercial starter
/// in a 42 degC water bath and their control yoghurt reached pH 4.5 in
/// EIGHT HOURS. Jankowska et al. 2026 (Foods 15(2):314,
/// doi:10.3390/foods15020314) fermented milk at 43 degC to pH 4.6, and
/// their Table 1 gives cow milk 6.06% lactose against 5.69% in the yogurt
/// made from it - 6.106% of the lactose converted. 43 degC is this
/// culture's declared optimum, where the shipped temperature envelope is
/// exactly 1, and Kim's 42 degC sits 0.4% below it on that envelope.
///
/// WHY THE SCRIPT BELOW IS AT ROOM TEMPERATURE AND NOT AT 43 degC, which
/// is the interesting part. Both cited experiments were THERMOSTATTED - a
/// water bath and an incubator - and this bench has no thermostat. Pour
/// milk at 43 degC into a beaker and `wait 8h` and the room takes it back
/// down; the eight hours are spent on a falling temperature, and an
/// isothermal constant cannot be read off a non-isothermal run. So the
/// anchor is checked where the bench IS isothermal, which is at ambient,
/// and the cited extent is carried there through the model's own envelope.
/// That carry is exact and needs no volume, no molar mass and no rounding:
/// `1 - extent = exp(-k*t)`, and the envelope multiplies `k`, so
/// `(1 - extent_ambient) = (1 - extent_optimum)^envelope`. THE GEOMETRY IS
/// OTHERWISE IDENTICAL, because the fit's declared dose is this script's -
/// one gram of culture in 100 mL of milk.
///
/// WHAT FAILS THIS. The rate constant, the declared optimum, the envelope
/// width, the way `active` grams are reduced to a concentration, and the
/// liquid volume 100 mL of milk expands to. Every one of those would move
/// the shipped answer while leaving the citation in place and still reading
/// true, which is why this exists. The milk's lactose SHARE is deliberately
/// not one of them: it sets how much acid comes out, not how fast, and an
/// extent is blind to it.
///
/// NOT A pH TEST, DELIBERATELY. Milk's casein and colloidal calcium
/// phosphate are about 60% of its buffer capacity and are modelled by
/// nothing, so a computed yoghurt pH is a lower bound at any acid dose. A
/// rate fitted to make a pH come out would have cancelled a rate error
/// against a buffer error. This test reads an EXTENT, which is what a rate
/// is measured by. See `docs/milk-buffer-and-the-fermentation-rate.md`.
#[test]
fn the_lactic_rate_reproduces_the_fermentation_it_was_fitted_to() {
    // Jankowska et al. 2026, Table 1, cow milk: 6.06% lactose in, 5.69% in
    // the yogurt. Kim, Oh and Imm 2018: eight hours. Both at the declared
    // optimum, to within the 0.4% of envelope that separates 42 from 43 C.
    const CITED_EXTENT_AT_OPTIMUM: f64 = 1.0 - 5.69 / 6.06;
    // The envelope, written out rather than read from the registry: a test
    // that takes its expectation from the thing it is testing agrees with
    // that thing however wrong it is.
    const OPTIMUM_K: f64 = 316.15;
    const WIDTH_K: f64 = 15.0;
    const AMBIENT_K: f64 = 298.15;

    let envelope = (-((AMBIENT_K - OPTIMUM_K) / WIDTH_K).powi(2)).exp();
    let expected = 1.0 - (1.0 - CITED_EXTENT_AT_OPTIMUM).powf(envelope);

    // `bio-069` exactly. No `@`, so the milk is poured at room temperature
    // and the eight hours are genuinely isothermal.
    let mut bench = Bench::new();
    run(&mut bench, "add v1 milk 100mL");
    let solids_before = milk_solids(&bench);
    run(&mut bench, "add v1 yoghurt_culture 1g");
    run(&mut bench, "wait 8h");
    let solids_after = milk_solids(&bench);

    // THE EXTENT IS THE LACTOSE'S, NOT THE SOLIDS'. What leaves the vessel
    // is a mass of lactose, and it is withdrawn from the milk's conserved
    // solids AS A WHOLE — fat, casein and lactose together — because those
    // solids are one undifferentiated portion. So the fraction of the
    // PORTION that left is the fraction of the LACTOSE that fermented times
    // the lactose's share of the portion, and dividing that share back out
    // is what turns a mass loss into an extent. (A consequence worth
    // noticing while it is in view: the model keeps the share fixed as the
    // portion shrinks, so it believes slightly more lactose is left than
    // really is. It is a fraction of a per cent at these extents and it is
    // not what this test is about.)
    let share =
        kerotakis_core::enzyme_activity::unresolved_lactose_share("household/whole-milk-surrogate")
            .expect("the milk recipe declares a lactose share");
    assert!(solids_before > 0.0 && share > 0.0);
    let converted = (solids_before - solids_after) / (solids_before * share);

    // HALF A PER CENT, AND WHAT LIVES INSIDE IT. The shipped constant is
    // quoted to three figures, which the cited inputs do not beat, and it
    // was solved against 0.08961 L — the water 100 mL of milk carries, by
    // mass — where the bench's own liquid volume is a little larger because
    // the serum ions occupy space too. Those two together are about 0.2%.
    // Anything that actually moved the model would move this by far more:
    // the constant this replaced was fifty-one times out.
    assert!(
        (converted / expected - 1.0).abs() < 5e-3,
        "eight counter-top hours should convert {:.4}% of the milk's lactose - \
         which is Jankowska et al. 2026's 6.106% at the optimum, carried to \
         25 C through the shipped envelope - and it converted {:.4}%. The \
         constant, the declared optimum, the envelope width, the reduction of \
         a dose to a concentration, or the liquid volume 100 mL of milk \
         expands to has moved.",
        expected * 100.0,
        converted * 100.0
    );
}

/// THE ALCOHOLIC ANCHOR, on the same footing and with the same caveats.
///
/// Pagliardini et al. 2013 (Microb Cell Fact 12:29,
/// doi:10.1186/1475-2859-12-29) give the wild-type CEN.PK113-7D maximum
/// specific ethanol production rate as 1.5 g ethanol per gram of dry cell
/// weight per hour, anaerobic, 30 degC. One gram of dry yeast is taken as
/// one gram of dry cell weight - an upper bound, as is the cited rate
/// itself, so this bench still ferments faster than a kitchen does.
///
/// The declared anchor scenario is one gram of yeast in ONE LITRE of the
/// classroom lesson's own 100 g/L sucrose at 30 degC. This test runs the
/// lesson's own beaker instead, at ambient, and compares RATE CONSTANTS
/// rather than extents, because the two differ in volume as well as in
/// temperature and a constant is what was fitted. The vessel's own liquid
/// volume and the grams of culture actually conserved are read off the
/// bench: they are INPUTS to the rate, not the thing under test, and the
/// extensivity tests above are what hold them.
#[test]
fn the_alcoholic_rate_reproduces_the_specific_rate_it_was_fitted_to() {
    const CITED_ETHANOL_G_PER_G_DCW_HOUR: f64 = 1.5;
    const ANCHOR_SUCROSE_G_PER_LITRE: f64 = 100.0;
    const ANCHOR_TEMPERATURE_K: f64 = 303.15;
    const OPTIMUM_K: f64 = 308.15;
    const WIDTH_K: f64 = 18.0;
    const SECONDS: f64 = 3600.0;

    let molar_mass = |key: &str| {
        kerotakis_core::species::lookup_key(key)
            .expect("registry species")
            .molar_mass
    };

    // The cited rate, turned into the anchor scenario's extent: 1.5 g of
    // ethanol is four sucroses' worth on the balanced route, out of the
    // sucrose one litre of 100 g/L holds.
    let sucrose_consumed = (CITED_ETHANOL_G_PER_G_DCW_HOUR / molar_mass("ethanol")) / 4.0;
    let sucrose_present = ANCHOR_SUCROSE_G_PER_LITRE / molar_mass("sucrose");
    let anchor_extent = sucrose_consumed / sucrose_present;
    // ... and back into the shipped constant, per gram per litre at the
    // declared optimum.
    let envelope = |t: f64| (-((t - OPTIMUM_K) / WIDTH_K).powi(2)).exp();
    let cited_constant = -(1.0 - anchor_extent).ln() / SECONDS / envelope(ANCHOR_TEMPERATURE_K);

    // The lesson's own beaker, at room temperature so the hour is
    // isothermal. Fresh yeast rather than dry: it needs no hydration, so
    // this measures a rate and not a wetting.
    let mut bench = Bench::new();
    run(&mut bench, "add v1 water 100mL");
    run(&mut bench, "add v1 table_sugar 10g");
    let sucrose_before = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
    run(&mut bench, "add v1 fresh_yeast 1g");
    // The grams the engine will actually charge the rate with. A gram of
    // fresh yeast is 70% water and 30% conserved solids, and only the
    // solids stay unresolved, so this reads 0.3 and the rate is scaled by
    // dry solids without the recipe having to say so.
    let culture_grams: f64 = bench.vessels[0]
        .unresolved_materials
        .iter()
        .filter(|portion| portion.recipe_id == "household/fresh-compressed-yeast-surrogate")
        .map(|portion| portion.amount)
        .sum();
    let litres = bench.vessels[0].liquid_volume().0;
    assert!(culture_grams > 0.0 && litres > 0.0);

    run(&mut bench, "wait 3600s");
    let sucrose_after = bench.vessels[0].moles_of(&SpeciesId::new("sucrose")).0;
    let measured_constant = -(sucrose_after / sucrose_before).ln() / SECONDS / culture_grams
        * litres
        / envelope(298.15);

    assert!(
        (measured_constant / cited_constant - 1.0).abs() < 5e-3,
        "the shipped rate constant should be {cited_constant} per second per gram \
         per litre - one gram of yeast in one litre of 100 g/L sucrose giving up \
         1.5 g of ethanol in the first hour at 30 degC, Pagliardini et al. 2013 - \
         and the bench ran at {measured_constant}"
    );
}
