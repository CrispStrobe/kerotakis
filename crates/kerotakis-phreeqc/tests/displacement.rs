//! The metallic state: displacement by the activity series, computed over
//! the activities the aqueous engine reports, and checked against the
//! engine's own data where it has any.

use kerotakis_core::displacement::{self, SERIES};
use kerotakis_core::*;
use kerotakis_phreeqc::derived;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    let aqueous = PhreeqcEquilibrator::new().expect("engine");
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(DisplacementEquilibrator::wrapping(Box::new(aqueous))),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

/// Run a sequence of (species, moles) additions into 100 mL of water and
/// hand back the bench and the last step's events.
fn run(additions: &[(&str, f64)]) -> (Bench, Vec<Event>) {
    let mut stack = stack();
    let mut bench = Bench::new();
    add(&mut bench, &mut stack, "water", 5.55);
    let mut last = Vec::new();
    for (key, moles) in additions {
        last = add(&mut bench, &mut stack, key, *moles);
    }
    (bench, last)
}

fn moles_of(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).expect("vessel")
}

/// Magnesium into copper sulfate: copper comes out as the metal, the
/// magnesium goes in as its ion, and the copper sulfate's blue is gone.
/// The amount is the copper that was there, not the magnesium that was
/// offered — the limiting reagent decides, by electron count.
#[test]
fn magnesium_displaces_copper_from_its_sulfate() {
    let (bench, events) = run(&[("CuSO4", 0.01), ("Mg", 0.02)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Plated { species, onto, moles, .. }
                if species.0 == "Cu" && onto.0 == "Mg" && (moles.0 - 0.01).abs() < 1e-9
        )),
        "0.01 mol of copper should plate out onto the magnesium: {events:?}"
    );
    assert!(
        (moles_of(&bench, "Cu", Phase::Solid) - 0.01).abs() < 1e-9,
        "copper metal in the vessel"
    );
    assert!(
        (moles_of(&bench, "Mg", Phase::Solid) - 0.01).abs() < 1e-9,
        "half the magnesium is left, as the metal"
    );
    assert!(
        moles_of(&bench, "Cu+2", Phase::Aqueous) < 1e-9,
        "no copper(II) remains in solution: {:?}",
        vessel(&bench).contents
    );
    assert!(
        (moles_of(&bench, "Mg+2", Phase::Aqueous) - 0.01).abs() < 1e-6,
        "the magnesium that dissolved is in solution as its ion"
    );
    // The announcement this replaces must be gone: a metal is modelled now.
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("Displacement will not happen")
        )),
        "the 17bea5b marker must not fire once displacement computes: {events:?}"
    );
    // And the equation is the one a learner writes.
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::ReactionOccurred { equation, .. } if equation == "Mg + Cu+2 → Mg+2 + Cu"
        )),
        "{events:?}"
    );
}

/// Equilibrium has no memory: metal first or salt first, same final state.
///
/// This is the metamorphic test the handover asked for before trusting
/// any displacement number, and it is the one that would have caught a
/// wrapper that disturbed the aqueous solver's temperature fixed point.
/// Tolerances, from measurement (2026-08-20, 100 mL, both orders of
/// Zn/CuSO₄ and Mg/CuSO₄): temperature agreed to 1.7e-7 K, pH to 7e-10,
/// solute amounts to 4e-10 mol. Asserted with headroom at 1e-8 mol and
/// 1e-5 pH as `metamorphic.rs` does, and at 0.05 K for temperature —
/// *not* tighter, because 0.05 K is the aqueous solver's own fixed-point
/// convergence criterion and a tighter bound would be a claim about luck.
/// Still five times under the 0.19 K Hess violation of c1d493c, so that
/// fault could not hide here.
#[test]
fn displacement_does_not_depend_on_addition_order() {
    for (salt, metal, amount_salt, amount_metal) in [
        ("CuSO4", "Mg", 0.01, 0.02),
        ("CuSO4", "Zn", 0.05, 0.05),
        ("AgNO3", "Cu", 0.02, 0.05),
        ("FeSO4", "Zn", 0.02, 0.01),
    ] {
        let (a, _) = run(&[(salt, amount_salt), (metal, amount_metal)]);
        let (b, _) = run(&[(metal, amount_metal), (salt, amount_salt)]);
        let (va, vb) = (vessel(&a), vessel(&b));
        for p in &va.contents {
            let other = moles_of(&b, &p.species.0, p.phase);
            // Water is rebuilt from the solver's equilibrated mass each
            // solve; compare it relatively, as 5.55 mol is not 0.01 mol.
            let tolerance = if p.species.0 == "water" {
                1e-7 * p.moles.0
            } else {
                1e-8
            };
            assert!(
                (p.moles.0 - other).abs() < tolerance,
                "{metal}/{salt}: {} ({:?}) is {} one way and {other} the other",
                p.species,
                p.phase,
                p.moles.0
            );
        }
        let (sa, sb) = (
            va.solution.as_ref().expect("solution"),
            vb.solution.as_ref().expect("solution"),
        );
        assert!(
            (sa.ph - sb.ph).abs() < 1e-5,
            "{metal}/{salt}: pH {} vs {}",
            sa.ph,
            sb.ph
        );
        assert!(
            (va.temperature.0 - vb.temperature.0).abs() < 0.05,
            "{metal}/{salt}: final temperature {} K vs {} K — enthalpy has stopped being a state function",
            va.temperature.0,
            vb.temperature.0
        );
    }
}

/// Zinc into copper sulfate is the calorimetry practical: the heat is the
/// difference of two formation enthalpies, −218.7 kJ/mol, and 0.05 mol in
/// 100 mL of water warms it by q / Cp(water) to within the heat capacity
/// the metals carry.
#[test]
fn the_heat_of_displacement_is_computed_from_formation_enthalpies() {
    let (bench, events) = run(&[("CuSO4", 0.05), ("Zn", 0.05)]);
    let (from, to) = events
        .iter()
        .filter_map(|e| match e {
            Event::TemperatureChanged { from, to, .. } => Some((from.0, to.0)),
            _ => None,
        })
        .next_back()
        .expect("the step announces its temperature change");
    let rise = to - from;
    assert!(
        (to - vessel(&bench).temperature.0).abs() < 1e-9,
        "the announced end temperature is the vessel's"
    );
    // q = 218.7 kJ/mol × 0.05 mol = 10.93 kJ; Cp ≈ 5.55 mol × 75.3 J/K
    // (plus ~25 J/K of metal) ≈ 443 J/K → 24.7 K. Asserted as a band
    // rather than a figure, because the exact Cp depends on which portions
    // the registry gives a heat capacity.
    assert!(
        (22.0..28.0).contains(&rise),
        "zinc into copper sulfate should warm 100 mL by ~25 K, got {rise:.2} K ({from:.2} → {to:.2})"
    );
    // One step tells one reaction-heat story, not one per solve. (The
    // mixing pass may announce its own small change first — cold zinc
    // into a warm beaker — and that is a different, real event.)
    let big = events
        .iter()
        .filter(|e| matches!(e, Event::TemperatureChanged { from, to, .. } if (to.0 - from.0).abs() > 1.0))
        .count();
    assert_eq!(big, 1, "{events:?}");
}

/// Magnesium in dilute acid: hydrogen, not copper, and it leaves.
#[test]
fn a_reactive_metal_dissolves_in_acid_with_hydrogen() {
    let (bench, events) = run(&[("HCl", 0.1), ("Mg", 0.02)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::GasEvolved { species, moles, .. }
                if species.0 == "H2" && (moles.0 - 0.02).abs() < 1e-9
        )),
        "0.02 mol of hydrogen per 0.02 mol of magnesium: {events:?}"
    );
    assert!(
        moles_of(&bench, "Mg", Phase::Solid) < 1e-12,
        "all the magnesium dissolved"
    );
    let ph = vessel(&bench).solution.as_ref().expect("solution").ph;
    // 0.06 mol of acid left in 0.1 kg of water: 0.6 mol/kgw, pH ≈ 0.2.
    assert!(
        (0.0..0.5).contains(&ph),
        "the acid the magnesium consumed is gone and the rest is still there: pH {ph}"
    );
    // The acid the metal consumed must not be booked again as a
    // neutralisation: that would add 5.7 kJ that no reaction released.
    // ΔH = −466.85 kJ/mol × 0.02 = 9.34 kJ over ~418 J/K ≈ 22 K.
    let rise = vessel(&bench).temperature.0 - 298.15;
    assert!(
        (19.0..25.0).contains(&rise),
        "heat of Mg + 2H+ → Mg2+ + H2 alone, got {rise:.2} K"
    );
}

/// Copper in dilute acid does nothing — and the bench says that as a
/// result about copper, with the reason, not as a gap in the lab.
#[test]
fn a_noble_metal_in_acid_is_reported_inert_not_unmodelled() {
    let (bench, events) = run(&[("HCl", 0.1), ("Cu", 0.02)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Inert { species, why, .. }
                if species.0 == "Cu" && why.contains("above hydrogen")
        )),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "a computed negative is not a modelling gap: {events:?}"
    );
    assert!((moles_of(&bench, "Cu", Phase::Solid) - 0.02).abs() < 1e-12);
}

/// The series grid has negative cells too: silver does not displace
/// copper.
#[test]
fn the_less_reactive_metal_does_not_displace_the_more_reactive_ion() {
    let (bench, events) = run(&[("CuSO4", 0.01), ("Ag", 0.02)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Inert { species, why, .. } if species.0 == "Ag" && why.contains("above copper")
        )),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::Plated { .. })),
        "nothing plates: {events:?}"
    );
    assert!((moles_of(&bench, "Cu+2", Phase::Aqueous) - 0.01).abs() < 1e-6);
}

/// Magnesium in brine has nothing to displace, and what it would do
/// instead — react slowly with the water — is a rate this lab does not
/// compute. That is a gap, and it is named as one.
#[test]
fn a_metal_with_nothing_to_displace_names_the_unmodelled_rate() {
    let (bench, events) = run(&[("NaCl", 0.1), ("Mg", 0.02)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("reaction with water itself")
        )),
        "{events:?}"
    );
    assert!((moles_of(&bench, "Mg", Phase::Solid) - 0.02).abs() < 1e-12);
    assert!(
        moles_of(&bench, "Mg+2", Phase::Aqueous) < 1e-12,
        "the metal must not be booked as its cation any more: {:?}",
        vessel(&bench).contents
    );
}

/// The strongest reductant takes the most noble ion first: zinc into a
/// mixed silver/copper solution plates silver before copper.
#[test]
fn the_series_orders_competing_oxidants() {
    let (bench, events) = run(&[("AgNO3", 0.01), ("CuSO4", 0.01), ("Zn", 0.005)]);
    // 0.005 mol Zn gives 0.01 mol of electrons: exactly the silver.
    assert!(
        (moles_of(&bench, "Ag", Phase::Solid) - 0.01).abs() < 1e-6,
        "all the silver, {events:?}"
    );
    assert!(
        moles_of(&bench, "Cu", Phase::Solid) < 1e-6,
        "and none of the copper yet: {:?}",
        vessel(&bench).contents
    );
}

/// A metal in contact with its own ion is an electrode, and the potential
/// the bench reports is that electrode's, not the air's.
#[test]
fn a_metal_in_its_own_solution_pins_the_potential() {
    let (bench, _) = run(&[("CuSO4", 0.01), ("Cu", 0.01)]);
    let v = vessel(&bench);
    let info = v.solution.as_ref().expect("solution");
    let eh = info.eh_volts(v.temperature.0).expect("pe is reported");
    // E = 0.342 + 0.0296·log a(Cu²⁺); a ≈ 0.03 at 0.1 mol/kgw with γ ≈ 0.3.
    assert!(
        (0.28..0.33).contains(&eh),
        "the Cu/Cu²⁺ electrode should sit near +0.30 V, got {eh:.3} V"
    );
    let routing = &info.provenance.as_ref().expect("provenance").routing;
    assert!(
        routing.contains("Nernst"),
        "the provenance says where the potential came from: {routing}"
    );
}

/// The engine's own data agrees with the curated series.
///
/// wateq4f defines AgMetal and ZnMetal as phases, with log K for
/// `M = Mⁿ⁺ + n e⁻`. That is −nE°/(RT ln10/F) at 25 °C, so the database
/// is an independent check on the E° table this module carries: the two
/// routes to the same number must agree, and they do to within a
/// millivolt. (CuMetal is written on the Cu⁺ couple and cannot be compared
/// directly; minteq.v4 and pitzer define no metal phases at all, and no
/// shipped dataset has iron or magnesium — which is why the series is a
/// module of its own rather than an EQUILIBRIUM_PHASES line.)
#[test]
fn the_series_agrees_with_the_database_where_the_database_has_an_opinion() {
    let idx = derived::index_for("wateq4f");
    const SLOPE: f64 = 0.059_16;
    for (phase, metal) in [("AgMetal", "Ag"), ("ZnMetal", "Zn")] {
        let log_k = idx
            .phases
            .get(phase)
            .and_then(|p| p.log_k)
            .unwrap_or_else(|| panic!("wateq4f should define {phase}"));
        let couple = displacement::couple_of_metal(metal).expect("in the series");
        let e0_from_database = -log_k * SLOPE / couple.electrons;
        assert!(
            (e0_from_database - couple.e0_volts).abs() < 1e-3,
            "{metal}: wateq4f's {phase} gives E° {e0_from_database:+.4} V, the series carries {:+.4} V",
            couple.e0_volts
        );
    }
    // And those phases must never be offered to the solver: the electrons
    // are this module's to account for.
    for phase in ["AgMetal", "CuMetal", "ZnMetal"] {
        assert!(
            derived::phase_by_name(phase).is_none(),
            "{phase} must not be a candidate phase"
        );
    }
    for metal in SERIES.iter().filter(|c| c.reduced_phase == Phase::Solid) {
        assert!(
            derived::role(metal.reduced).is_none(),
            "{} must not book as its cation",
            metal.reduced
        );
    }
}

/// Build two half-cells on one bench: (metal, salt, moles of salt) each in
/// 100 mL, and return the bench.
fn half_cells(left: (&str, &str, f64), right: (&str, &str, f64)) -> (Bench, SolverStack) {
    let mut stack = stack();
    let mut bench = Bench::new();
    bench
        .step_with(Operator::NewVessel, &mut stack, &PermissiveScreen)
        .expect("v2");
    for (v, (metal, salt, moles)) in [(VesselId(0), left), (VesselId(1), right)] {
        for (key, n) in [("water", 5.55), (salt, moles), (metal, 0.05)] {
            bench
                .step_with(
                    Operator::Add {
                        vessel: v,
                        species: SpeciesId::new(key),
                        moles: Moles(n),
                        at: None,
                    },
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step");
        }
    }
    (bench, stack)
}

fn wire(bench: &mut Bench, stack: &mut SolverStack) -> Vec<Event> {
    bench
        .step_with(
            Operator::Cell {
                a: VesselId(0),
                b: VesselId(1),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("cell")
}

/// The Daniell cell. Zinc in zinc sulfate, copper in copper sulfate, a
/// voltmeter across them: +1.10 V, zinc the anode, whichever side it was
/// wired to — and nothing in either beaker changes, because no current
/// flows.
#[test]
fn the_daniell_cell_reads_about_one_point_one_volts() {
    let (mut bench, mut stack) = half_cells(("Zn", "ZnSO4", 0.1), ("Cu", "CuSO4", 0.1));
    let before = serde_json::to_string(&bench.vessels).unwrap();
    let events = wire(&mut bench, &mut stack);
    let (anode, volts, standard, notation) = events
        .iter()
        .find_map(|e| match e {
            Event::CellVoltage {
                anode,
                volts,
                standard_volts,
                notation,
                ..
            } => Some((*anode, *volts, *standard_volts, notation.clone())),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{events:?}"));
    assert_eq!(anode, VesselId(0), "zinc is the anode");
    assert_eq!(notation, "Zn | Zn+2 ‖ Cu+2 | Cu");
    assert!(
        (standard - 1.1037).abs() < 1e-4,
        "E° from the series: {standard}"
    );
    // Equal molalities: the Nernst terms nearly cancel, leaving the γ
    // ratio of two 2+ sulfates at 1 mol/kgw — a few millivolts.
    assert!(
        (1.07..1.13).contains(&volts),
        "Daniell should read ~1.10 V, got {volts:.4}"
    );
    assert_eq!(
        before,
        serde_json::to_string(&bench.vessels).unwrap(),
        "reading a voltmeter must not change either beaker"
    );

    // Wired the other way round, the same cell with the same anode.
    let events = bench
        .step_with(
            Operator::Cell {
                a: VesselId(1),
                b: VesselId(0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("cell");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::CellVoltage { anode, volts: v, .. } if *anode == VesselId(0) && (v - volts).abs() < 1e-12
        )),
        "{events:?}"
    );
}

/// The activities reach the equation: make the copper side ten times
/// more dilute and the cell drops. Ideal solutions would drop by
/// (RT ln10 / 2F) · log 10 = 29.6 mV; the bench drops 17.6 mV, because
/// the speciation knows what a concentration does not — at 1 mol/kgw
/// most of the copper is paired with sulfate, and diluting it frees a
/// larger share, so the *activity* ratio is nearer 4 than 10. If the cell
/// read 1.10 V regardless, Nernst would be decoration; if it dropped
/// exactly 29.6 mV, it would be Nernst over concentrations.
#[test]
fn the_cell_voltage_moves_with_activity_not_concentration() {
    let (mut a, mut sa) = half_cells(("Zn", "ZnSO4", 0.1), ("Cu", "CuSO4", 0.1));
    let (mut b, mut sb) = half_cells(("Zn", "ZnSO4", 0.1), ("Cu", "CuSO4", 0.01));
    let volts = |events: &[Event]| {
        events
            .iter()
            .find_map(|e| match e {
                Event::CellVoltage { volts, .. } => Some(*volts),
                _ => None,
            })
            .expect("a cell")
    };
    let (ea, eb) = (volts(&wire(&mut a, &mut sa)), volts(&wire(&mut b, &mut sb)));
    let drop = ea - eb;
    assert!(
        drop > 0.005,
        "dilution must lower the cell: {ea:.4} → {eb:.4}"
    );
    assert!(
        drop < 0.0296 * 0.9,
        "a drop of the full ideal 29.6 mV would mean concentrations reached the equation, not activities: {:.1} mV",
        drop * 1000.0
    );
    // And the drop is exactly what the two reported activities dictate.
    let ca = displacement::electrode(a.vessel(VesselId(1)).unwrap()).unwrap();
    let cb = displacement::electrode(b.vessel(VesselId(1)).unwrap()).unwrap();
    assert!(
        (drop - (ca.volts - cb.volts)).abs() < 1e-9,
        "the cell moved by something other than its cathode: {drop} vs {}",
        ca.volts - cb.volts
    );
    assert!(
        ca.activity / cb.activity < 6.0,
        "tenfold in molality, less in activity — got a ratio of {:.2}",
        ca.activity / cb.activity
    );
}

/// Silver against copper: the other school cell, +0.46 V by the series.
#[test]
fn copper_against_silver_reads_the_series_difference() {
    let (mut bench, mut stack) = half_cells(("Cu", "CuSO4", 0.1), ("Ag", "AgNO3", 0.1));
    let events = wire(&mut bench, &mut stack);
    let (anode, volts, standard) = events
        .iter()
        .find_map(|e| match e {
            Event::CellVoltage {
                anode,
                volts,
                standard_volts,
                ..
            } => Some((*anode, *volts, *standard_volts)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{events:?}"));
    assert_eq!(anode, VesselId(0), "copper is the anode against silver");
    assert!((standard - 0.4577).abs() < 1e-4, "{standard}");
    // Ag⁺ enters at n = 1 and Cu²⁺ at n = 2, so the Nernst terms do not
    // cancel at equal molality, and they do not cancel the way an ideal
    // calculation says either: free Cu²⁺ sits far below its molality in
    // 1 mol/kgw sulfate, which *raises* the cell above E°. The number is
    // whatever the two electrodes say — checked as the identity
    // E = E(cathode) − E(anode) over the reported activities, each vessel
    // at its own temperature — and bounded only loosely.
    let anode_e = displacement::electrode(bench.vessel(VesselId(0)).unwrap()).unwrap();
    let cathode_e = displacement::electrode(bench.vessel(VesselId(1)).unwrap()).unwrap();
    assert!(
        (volts - (cathode_e.volts - anode_e.volts)).abs() < 1e-9,
        "{volts} vs {} − {}",
        cathode_e.volts,
        anode_e.volts
    );
    assert!(
        (0.40..0.52).contains(&volts),
        "copper–silver should read within ~50 mV of its E° 0.458 V, got {volts:.4}"
    );
}

/// A beaker without a half-cell is said to be one, with the reason; and
/// that is a statement about the beaker, not a gap.
#[test]
fn a_vessel_without_an_electrode_is_not_a_half_cell() {
    // Copper in brine: a metal, no ion of its own.
    let (mut bench, mut stack) = half_cells(("Cu", "NaCl", 0.1), ("Zn", "ZnSO4", 0.1));
    let events = wire(&mut bench, &mut stack);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NoCell { a, why, .. } if *a == VesselId(0) && why.contains("none of its ion")
        )),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. } | Event::CellVoltage { .. })),
        "{events:?}"
    );
}

/// The accessor the electrolysis layer builds on: what electrode is in
/// this vessel, at what potential.
#[test]
fn a_vessel_exposes_its_electrode() {
    let (bench, _) = run(&[("CuSO4", 0.1), ("Cu", 0.05)]);
    let e = displacement::electrode(vessel(&bench)).expect("a copper electrode");
    assert_eq!(e.couple.reduced, "Cu");
    assert_eq!(e.label(), "Cu | Cu+2");
    assert!(
        e.activity > 0.0 && e.activity < 1.0,
        "γ < 1 at 1 mol/kgw: {}",
        e.activity
    );
    assert!((0.28..0.34).contains(&e.volts), "{}", e.volts);
    let (bench, _) = run(&[("NaCl", 0.1), ("Cu", 0.05)]);
    assert!(displacement::electrode(vessel(&bench)).is_none());
}

/// Two copper electrodes, two strengths of copper sulfate: E° cancels
/// exactly and what is left is the Nernst term alone — a few tens of
/// millivolts, with the dilute side as the anode, because copper there
/// would rather dissolve to raise its ion than the strong side would. The
/// purest demonstration that the voltage is made of activities.
#[test]
fn a_concentration_cell_is_the_nernst_term_alone() {
    let (mut bench, mut stack) = half_cells(("Cu", "CuSO4", 0.01), ("Cu", "CuSO4", 0.1));
    let events = wire(&mut bench, &mut stack);
    let (anode, volts, standard) = events
        .iter()
        .find_map(|e| match e {
            Event::CellVoltage {
                anode,
                volts,
                standard_volts,
                ..
            } => Some((*anode, *volts, *standard_volts)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{events:?}"));
    assert_eq!(standard, 0.0, "same couple both sides: E° cancels");
    assert_eq!(anode, VesselId(0), "the dilute side is the anode");
    assert!(
        (0.005..0.040).contains(&volts),
        "a tenfold concentration cell is tens of millivolts, got {:.1} mV",
        volts * 1000.0
    );
}
