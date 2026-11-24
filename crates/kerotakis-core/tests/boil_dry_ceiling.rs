//! A beaker heated until it boils dry must not report millions of degrees.
//!
//! The burner only ever offers what fits below its own flame, but the
//! boiling pass spends that offer on steam and hands the leftover to
//! whatever remains. A trace of residue has almost no heat capacity, and a
//! few kilojoules over it read as 10^7 °C (C1 E292/E313). A flame cannot
//! heat anything above its own temperature: the vessel stops at the
//! ceiling and the rest of the dose is reported undelivered.

use kerotakis_core::apparatus::BUNSEN_CEILING_K;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, op: Operator) -> Vec<Event> {
    bench
        .step_with(op, &mut stack(), &PermissiveScreen)
        .expect("operator")
}

fn add(bench: &mut Bench, species: &str, moles: f64) {
    step(
        bench,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(species),
            moles: Moles(moles),
            at: None,
        },
    );
}

fn heat(bench: &mut Bench, joules: f64) -> (f64, f64) {
    let events = step(
        bench,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(joules),
            source: None,
        },
    );
    let (requested, delivered) = events
        .iter()
        .find_map(|e| match e {
            Event::EnergyTransferred {
                heating: true,
                requested_j,
                delivered_j,
                ..
            } => Some((*requested_j, *delivered_j)),
            _ => None,
        })
        .expect("a heat dose is reported");
    for e in &events {
        if let Event::TemperatureChanged { to, .. } = e {
            assert!(to.0 <= BUNSEN_CEILING_K + 1e-9, "reported {} K", to.0);
        }
    }
    (requested, delivered)
}

#[test]
fn boiled_dry_residue_stops_at_the_flame() {
    let mut bench = Bench::new();
    add(&mut bench, "water", 0.1);
    add(&mut bench, "NaCl", 1e-6);
    let (requested, delivered) = heat(&mut bench, 10_000.0);
    let v = bench.vessel(VesselId(0)).expect("vessel");
    assert!(
        v.temperature.0 <= BUNSEN_CEILING_K + 1e-9,
        "boiled-dry residue at {} K",
        v.temperature.0
    );
    assert!(v
        .contents
        .iter()
        .all(|p| !(p.species.0 == "water" && p.phase == Phase::Liquid)));
    // The steam was paid for; the rest of the dose never left the burner.
    let latent = 0.1 * kerotakis_core::states::WATER_H_VAP;
    assert!(
        delivered >= latent,
        "delivered {delivered} J < latent {latent} J"
    );
    assert!(
        delivered < requested - 1000.0,
        "a dose that could only lift a trace of salt was reported delivered: {delivered} J"
    );
}

#[test]
fn ample_water_still_takes_the_whole_dose() {
    // Negative control: nothing near the ceiling, nothing taken back.
    let mut bench = Bench::new();
    add(&mut bench, "water", 5.55);
    let (requested, delivered) = heat(&mut bench, 10_000.0);
    assert!(
        (requested - delivered).abs() < 1e-6,
        "{delivered} of {requested} J"
    );
    let v = bench.vessel(VesselId(0)).expect("vessel");
    assert!(v.temperature.0 < 373.15 + 1.0);
}
