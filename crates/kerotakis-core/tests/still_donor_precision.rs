//! A still cannot credit condensate when scalar subtraction leaves its donor unchanged.
//! Changing scalar stocks retain the existing requested-cut and latent contracts;
//! these tests do not claim compensated or transfer-relative donor accounting.
use kerotakis_core::*;

#[derive(Default)]
struct CountingMutator {
    calls: Vec<VesselId>,
}

impl Equilibrator for CountingMutator {
    fn name(&self) -> &'static str {
        "still-donor-counting-mutator"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls.push(vessel.id);
        vessel.temperature.0 += 1.0;
        vessel.free_proton += 0.001;
        Ok(Vec::new())
    }
}

fn prepared(species: &str) -> Bench {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new(species), Moles(1.0), Phase::Liquid);
    bench.vessels.push(Vessel::new(VesselId(1), "receiver"));
    bench
}

fn physical(bench: &Bench) -> serde_json::Value {
    let mut value = serde_json::to_value(bench).unwrap();
    value.as_object_mut().unwrap().remove("log");
    value
}

fn requests(fraction: f64, latent_kj: f64) -> [Operator; 2] {
    [
        Operator::Distil {
            from: VesselId(0),
            to: VesselId(1),
            fraction: Some(fraction),
            energy: None,
            stages: 1,
        },
        Operator::Distil {
            from: VesselId(0),
            to: VesselId(1),
            fraction: None,
            energy: Some(Joules(fraction * latent_kj * 1000.0)),
            stages: 4,
        },
    ]
}

fn zero_debit_refuses(species: &str, latent_kj: f64) {
    for operation in requests(1e-20, latent_kj) {
        let mut bench = prepared(species);
        let before = physical(&bench);
        let mut solver = CountingMutator::default();
        let events = bench
            .step_with(operation, &mut solver, &PermissiveScreen)
            .unwrap();
        assert_eq!(physical(&bench), before);
        assert!(solver.calls.is_empty());
        assert_eq!(bench.log.len(), 1);
        assert_eq!(events.len(), 1);
        let Event::NotYetModeled {
            cause,
            what,
            reason,
            ..
        } = &events[0]
        else {
            panic!("expected an atomic donor refusal, got {:?}", events[0]);
        };
        assert_eq!(*cause, ops::NotModelledCause::ModelBoundary);
        assert_eq!(
            reason.as_ref().unwrap().key,
            "not-modeled.unrepresentable-still-donor"
        );
        assert!(what.contains("No cut was transferred"));
        assert_eq!(
            serde_json::to_value(&bench.log[0].events).unwrap(),
            serde_json::to_value(&events).unwrap()
        );
    }
}

#[test]
fn pure_water_zero_debit_fraction_and_energy_are_atomic_refusals() {
    zero_debit_refuses("water", 40.657);
}

#[test]
fn pure_ethanol_zero_debit_fraction_and_energy_are_atomic_refusals() {
    zero_debit_refuses("ethanol", 38.58);
}

#[test]
fn pure_methanol_zero_debit_fraction_and_energy_are_atomic_refusals() {
    zero_debit_refuses("methanol", 35.244);
}

#[test]
fn changing_microscopic_donors_preserve_requested_cut_and_latent_accounts() {
    for (species, latent_kj) in [("water", 40.657), ("ethanol", 38.58), ("methanol", 35.244)] {
        for operation in requests(1e-14, latent_kj) {
            let mut bench = prepared(species);
            let mut solver = CountingMutator::default();
            let events = bench
                .step_with(operation, &mut solver, &PermissiveScreen)
                .unwrap();
            assert_eq!(solver.calls, [VesselId(0), VesselId(1)]);
            assert!(!events
                .iter()
                .any(|event| matches!(event, Event::NotYetModeled { .. })));
            let id = SpeciesId::new(species);
            let remaining = bench.vessels[0].moles_of(&id).0;
            let received = bench.vessels[1].moles_of(&id).0;
            assert!(remaining > 0.0 && remaining < 1.0);
            assert!((received / 1e-14 - 1.0).abs() < 1e-10);
            assert!((remaining + received - 1.0).abs() < 4.0 * f64::EPSILON);
            let cuts: Vec<_> = events
                .iter()
                .filter_map(|event| match event {
                    Event::Distilled {
                        components,
                        energy_kj,
                        ..
                    } => Some((components, energy_kj)),
                    _ => None,
                })
                .collect();
            assert_eq!(cuts.len(), 1);
            let (components, energy_kj) = cuts[0];
            let reported: f64 = components
                .iter()
                .filter(|(component, _)| *component == id)
                .map(|(_, amount)| amount.0)
                .sum();
            assert!((reported / received - 1.0).abs() < 1e-12);
            assert!((*energy_kj / (received * latent_kj) - 1.0).abs() < 1e-12);
        }
    }
}
