//! `discard <vessel>` — the verb that empties ONE vessel into the waste.
//!
//! The bench could take a vessel away (`remove`, and only once it was
//! already empty) and it could pour one vessel into another (`decant`,
//! `drain`), but it had no way to say "this is finished with". The web's
//! disposal station therefore offered the only thing the engine could do,
//! which was to clear the entire bench — a reset dressed up as chemistry.
//!
//! What makes this a chemistry operation rather than a delete is that the
//! matter does not leave the model. It moves into the bench's waste
//! compartment, where it is still weighed and still screened, and the
//! tests below are about exactly that: nothing vanishes, a closed vessel
//! is refused rather than silently opened, and the bin knows what is
//! already in it when the next vessel is poured away.

use kerotakis_core::authority::SpillDestination;
use kerotakis_core::solve::{SafetyScreen, SafetyVerdict, Severity};
use kerotakis_core::{
    Bench, BenchError, ConservedLedger, Event, Kelvin, Liters, Moles, Operator, SpeciesId, Vessel,
    VesselId,
};
use std::collections::BTreeMap;

/// Everything the bench is holding, wherever it is holding it.
fn totals(bench: &Bench) -> (BTreeMap<String, f64>, f64) {
    let mut elements = BTreeMap::new();
    let mut mass = 0.0;
    let ledgers = bench
        .vessels
        .iter()
        .map(ConservedLedger::from_vessel)
        .chain(
            bench
                .spills
                .iter()
                .map(|spill| ConservedLedger::from_vessel(&spill.as_vessel_probe())),
        );
    for ledger in ledgers {
        mass += ledger.mass;
        for (element, amount) in ledger.elements {
            *elements.entry(element).or_insert(0.0) += amount;
        }
    }
    (elements, mass)
}

fn charged() -> Bench {
    let mut bench = Bench::new();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(5.0),
            at: Some(Kelvin::from_celsius(60.0)),
        })
        .unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("NaCl"),
            moles: Moles(0.05),
            at: None,
        })
        .unwrap();
    bench
}

fn waste(bench: &Bench) -> &kerotakis_core::SpillCompartment {
    bench
        .spill(&SpillDestination::Waste)
        .expect("the waste compartment exists after a discard")
}

#[test]
fn discard_conserves_every_element_and_the_mass() {
    let mut bench = charged();
    let (elements_before, mass_before) = totals(&bench);
    assert!(mass_before > 0.0, "the fixture put nothing on the bench");

    let warm = bench.vessel(VesselId(0)).unwrap().temperature;
    let events = bench
        .step(Operator::Discard {
            vessel: VesselId(0),
        })
        .unwrap();

    let (elements_after, mass_after) = totals(&bench);
    assert!(
        (mass_after - mass_before).abs() < 1e-9,
        "discard lost {} g: {mass_before} → {mass_after}",
        mass_before - mass_after
    );
    for (element, before) in &elements_before {
        let after = elements_after.get(element).copied().unwrap_or(0.0);
        assert!(
            (after - before).abs() < 1e-9,
            "element {element}: {before} → {after}"
        );
    }

    // The vessel is empty and STILL THERE, still at the temperature the
    // glass had. A disposal is not a tidy-up: nobody took the beaker away
    // and nobody ran it under the cold tap.
    let vessel = bench.vessel(VesselId(0)).unwrap();
    assert!(vessel.is_empty(), "the vessel kept something back");
    assert!(
        (vessel.temperature.0 - warm.0).abs() < 1e-9,
        "the glass forgot how warm it was: {} K → {} K",
        warm.0,
        vessel.temperature.0
    );

    // And the ledger says what it swallowed.
    let discarded = events
        .iter()
        .find_map(|event| match event {
            Event::Discarded {
                vessel,
                into,
                moles_total,
                grams_total,
                species,
                ..
            } => Some((
                *vessel,
                into.clone(),
                *moles_total,
                *grams_total,
                species.clone(),
            )),
            _ => None,
        })
        .expect("a Discarded event");
    assert_eq!(discarded.0, VesselId(0));
    assert_eq!(discarded.1, SpillDestination::Waste);
    assert!(discarded.3 > 0.0, "nothing weighed anything");
    let summed: f64 = discarded.4.iter().map(|portion| portion.moles.0).sum();
    assert!(
        (summed - discarded.2 .0).abs() < 1e-9,
        "the per-species lines do not add up to the total: {summed} vs {}",
        discarded.2 .0
    );
    // Largest first — the same order on every host, so the rendered lines
    // cannot differ between the desktop and the browser.
    let mut sorted = discarded.4.clone();
    sorted.sort_by(|a, b| b.moles.0.partial_cmp(&a.moles.0).unwrap());
    assert_eq!(
        discarded
            .4
            .iter()
            .map(|p| p.species.0.clone())
            .collect::<Vec<_>>(),
        sorted
            .iter()
            .map(|p| p.species.0.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn two_discards_share_one_bin() {
    let mut bench = charged();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(1.0),
            at: None,
        })
        .unwrap();
    bench
        .step(Operator::Discard {
            vessel: VesselId(0),
        })
        .unwrap();
    let first: f64 = waste(&bench).contents.iter().map(|p| p.moles.0).sum();
    bench
        .step(Operator::Discard {
            vessel: VesselId(1),
        })
        .unwrap();
    let second: f64 = waste(&bench).contents.iter().map(|p| p.moles.0).sum();

    assert_eq!(bench.spills.len(), 1, "each discard opened its own bin");
    assert!(
        second > first + 0.5,
        "the second discard did not reach the same bin: {first} → {second}"
    );
    assert_eq!(waste(&bench).sources, vec![VesselId(0), VesselId(1)]);
}

#[test]
fn a_sealed_vessel_is_refused_and_untouched() {
    let mut bench = charged();
    bench
        .step(Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(0.5),
        })
        .unwrap();

    let refusal = bench.step(Operator::Discard {
        vessel: VesselId(0),
    });
    match refusal {
        Err(BenchError::VesselSealed(vessel)) => {
            assert_eq!(vessel, VesselId(0));
            let reason = BenchError::VesselSealed(vessel).to_string();
            assert!(
                reason.contains("open v1 first"),
                "the refusal has to say what to do about it: {reason}"
            );
        }
        other => panic!("expected a typed sealed refusal, got {other:?}"),
    }
    assert!(
        !bench.vessel(VesselId(0)).unwrap().is_empty(),
        "a refused discard emptied the vessel anyway"
    );
    assert!(bench.spills.is_empty(), "a refused discard opened a bin");

    // Doing what the refusal says makes it work.
    bench
        .step(Operator::Open {
            vessel: VesselId(0),
        })
        .unwrap();
    bench
        .step(Operator::Discard {
            vessel: VesselId(0),
        })
        .expect("an opened vessel can be discarded");
    assert!(bench.vessel(VesselId(0)).unwrap().is_empty());
}

/// A screen that refuses any bin holding both of two species at once.
/// The real incompatibility matrix warns rather than vetoes, so this
/// stands in for the veto BOUNDARY — what a discard does when the screen
/// says no, which is nothing at all.
struct NoAcidIntoBleach;

impl SafetyScreen for NoAcidIntoBleach {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        let has = |key: &str| {
            vessel
                .contents
                .iter()
                .any(|portion| portion.species.0 == key && portion.moles.0 > 1e-12)
        };
        if has("NaOCl") && has("HCl") {
            return SafetyVerdict::Veto {
                reason: "chlorine gas".into(),
            };
        }
        SafetyVerdict::Allow
    }
}

#[test]
fn a_vetoed_discard_refuses_and_leaves_the_vessel_full() {
    let mut solver = kerotakis_core::solve::SolverStack::new(vec![]);
    let mut bench = Bench::new();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    for (vessel, species) in [(VesselId(0), "NaOCl"), (VesselId(1), "HCl")] {
        bench
            .step_with(
                Operator::Add {
                    vessel,
                    species: SpeciesId::new(species),
                    moles: Moles(0.02),
                    at: None,
                },
                &mut solver,
                &NoAcidIntoBleach,
            )
            .unwrap();
    }

    // The bleach goes in first, alone, and nothing objects.
    let clean = bench
        .step_with(
            Operator::Discard {
                vessel: VesselId(0),
            },
            &mut solver,
            &NoAcidIntoBleach,
        )
        .unwrap();
    assert!(clean
        .iter()
        .any(|event| matches!(event, Event::Discarded { .. })));

    // The acid meets what is already in the bin, and the screen sees the
    // MIXTURE rather than the vessel — which is the whole reason disposal
    // goes through a compartment instead of a delete.
    let refused = bench
        .step_with(
            Operator::Discard {
                vessel: VesselId(1),
            },
            &mut solver,
            &NoAcidIntoBleach,
        )
        .unwrap();
    assert!(
        refused
            .iter()
            .any(|event| matches!(event, Event::SafetyVeto { .. })),
        "the combined waste was not screened: {refused:?}"
    );
    assert!(
        !refused
            .iter()
            .any(|event| matches!(event, Event::Discarded { .. })),
        "a vetoed discard still claimed to have happened"
    );
    assert!(
        !bench.vessel(VesselId(1)).unwrap().is_empty(),
        "a vetoed discard emptied the vessel anyway"
    );
}

/// A screen that always warns, to prove the warning reaches the reader
/// with the bin named on it rather than a vessel.
struct AlwaysWarn;

impl SafetyScreen for AlwaysWarn {
    fn assess(&self, _vessel: &Vessel) -> SafetyVerdict {
        SafetyVerdict::Warn {
            severity: Severity::Caution,
            rule: "test-rule".into(),
            hazard: "the bin is getting interesting".into(),
            real_world: "Label the container.".into(),
        }
    }
}

#[test]
fn a_warned_discard_names_the_bin_and_still_happens() {
    let mut solver = kerotakis_core::solve::SolverStack::new(vec![]);
    let mut bench = Bench::new();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
                at: None,
            },
            &mut solver,
            &AlwaysWarn,
        )
        .unwrap();
    let events = bench
        .step_with(
            Operator::Discard {
                vessel: VesselId(0),
            },
            &mut solver,
            &AlwaysWarn,
        )
        .unwrap();

    let hazard = events
        .iter()
        .find_map(|event| match event {
            Event::SpillHazard { destination, .. } => Some(destination.clone()),
            _ => None,
        })
        .expect("the warning names where the hazard is");
    assert_eq!(hazard, SpillDestination::Waste);
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::Discarded { .. })));
    assert!(bench.vessel(VesselId(0)).unwrap().is_empty());
}

#[test]
fn the_waste_destination_survives_a_save() {
    let mut bench = charged();
    bench
        .step(Operator::Discard {
            vessel: VesselId(0),
        })
        .unwrap();
    let json = serde_json::to_string(&bench).unwrap();
    assert!(
        json.contains("\"surface\":\"waste\""),
        "the waste bin is not on the wire: {}",
        &json[..json.len().min(400)]
    );
    let restored: Bench = serde_json::from_str(&json).unwrap();
    assert_eq!(
        restored
            .spill(&SpillDestination::Waste)
            .map(|s| s.contents.len()),
        bench
            .spill(&SpillDestination::Waste)
            .map(|s| s.contents.len())
    );
}

#[test]
fn a_discard_can_be_recovered_because_nothing_was_destroyed() {
    let mut bench = charged();
    let (_, mass_before) = totals(&bench);
    bench
        .step(Operator::Discard {
            vessel: VesselId(0),
        })
        .unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::RecoverSpill {
            destination: SpillDestination::Waste,
            to: VesselId(1),
            fraction: 1.0,
        })
        .expect("the waste ledger holds real matter, so it can be poured back");
    let (_, mass_after) = totals(&bench);
    assert!((mass_after - mass_before).abs() < 1e-9);
    assert!(!bench.vessel(VesselId(1)).unwrap().is_empty());
}
