//! Event projection controls, not an original qualification or native replay.
use super::*;
use kerotakis_core::{Pascal, VesselId};

fn projection(headspace: Headspace, kind: ExternalGasKind) -> (Vec<Event>, Vec<Portion>) {
    let mut vessel = Vessel::new(VesselId(0), "carbonate boundary control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("CaCO3"), Moles(0.0005), Phase::Solid);
    vessel.headspace = headspace;
    let mut problem = partition(&vessel).expect("represented carbonate input");
    // Bind the projection to one explicitly selected external phase.
    problem.external_gases = vec![ExternalGas {
        phase: "CO2(g)".into(),
        species: "CO2".into(),
        initial_moles: 0.0,
        kind,
    }];
    PhreeqcEquilibrator::rebuild_contents_and_events(
        &vessel,
        &problem,
        &[],
        problem.kgw,
        &[],
        &[("CO2(g)".into(), 0.0001)],
        &[],
        &[],
        &BTreeMap::new(),
    )
}

fn warning(event: &Event) -> bool {
    matches!(event, Event::NotYetModeled { reason: Some(reason), .. }
        if reason.key.as_str() == "not-modeled.swept-gas-equilibrium")
}

#[test]
fn swept_reservoir_warns_without_changing_carbon_export_or_inventory() {
    let (swept, swept_contents) = projection(
        Headspace::Swept {
            pressure: Pascal(101325.0),
        },
        ExternalGasKind::Reservoir,
    );
    let (open, open_contents) = projection(Headspace::Open, ExternalGasKind::Reservoir);
    assert_eq!(swept.iter().filter(|event| warning(event)).count(), 1);
    assert!(!open.iter().any(warning));
    assert_eq!(swept_contents, open_contents);
    let gas = |events: &[Event]| {
        events
            .iter()
            .filter_map(|event| match event {
                Event::GasEvolved { species, moles, .. } => Some((species.clone(), *moles)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(gas(&swept), vec![(SpeciesId::new("CO2"), Moles(0.0001))]);
    assert_eq!(gas(&swept), gas(&open));
    let event = swept.iter().find(|event| warning(event)).unwrap();
    assert!(matches!(
        event,
        Event::NotYetModeled {
            cause: kerotakis_core::ops::NotModelledCause::ModelBoundary,
            ..
        }
    ));
}

#[test]
fn finite_dose_does_not_acquire_reservoir_warning() {
    let (events, _) = projection(
        Headspace::Swept {
            pressure: Pascal(101325.0),
        },
        ExternalGasKind::Dose,
    );
    assert!(!events.iter().any(warning));
    assert!(events.iter().any(|event| matches!(event,
        Event::NotYetModeled { reason: Some(reason), .. } if reason.key.as_str() == "not-modeled.finite-gas-dose")));
}
