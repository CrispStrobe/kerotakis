//! A dry residue has a heat capacity.
//!
//! Dissolved ions carry no heat capacity on this bench — a solution's is its
//! water's. When the water boils or evaporates away, whatever `dryness`
//! cannot book as a salt stays filed aqueous, and under that convention it
//! weighed nothing thermally: the leftover of a heat dose read as
//! 1.65×10⁷ °C over a sodium acetate residue (C1 E292), 5.2×10⁵ °C over
//! the residue of hot concentrated sulfuric acid on carbon (C1 E148). Such
//! a residue is now charged as the solid it is, by Kopp's rule, so heat
//! lands on real mass; what it still cannot hold below the flame is
//! reported undelivered (#915), never silently clamped.

use kerotakis_core::apparatus::BUNSEN_CEILING_K;
use kerotakis_core::heat_capacity::{kopp_solid_heat_capacity, KOPP_OTHER};
use kerotakis_core::species::Phase;
use kerotakis_core::*;

const ROOM: f64 = 298.15;

fn vessel_with(contents: &[(&str, f64, Phase)]) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v
}

/// Na⁺/K⁺/Cl⁻/NO₃⁻: a reciprocal system, so `dryness` books no salt and
/// every ion stays stranded.
const RECIPROCAL: [&str; 4] = ["Na+", "K+", "Cl-", "NO3-"];

fn kopp_of(key: &str) -> f64 {
    let data = species::lookup(&SpeciesId::new(key)).expect("registry species");
    kopp_solid_heat_capacity(data.formula).expect("Kopp estimate")
}

fn reciprocal_kopp(n: f64) -> f64 {
    RECIPROCAL.iter().map(|k| n * kopp_of(k)).sum()
}

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

fn heat(bench: &mut Bench, joules: f64) -> (Vec<Event>, f64, f64) {
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
    (events, requested, delivered)
}

fn note_keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key.clone()),
            _ => None,
        })
        .collect()
}

const KOPP_NOTE: &str = "not-modeled.dry-residue-kopp-heat-capacity";

#[test]
fn kopp_rule_is_the_sum_of_tabulated_atomic_heats() {
    // Na and Cl both take Kopp's "all other elements" value.
    let nacl = kopp_solid_heat_capacity("NaCl").unwrap();
    assert!((nacl - 2.0 * KOPP_OTHER).abs() < 1e-12);
    // Within a few percent of the measured 50.50 J/(mol·K) (NBS, Wagman
    // et al. 1982): an estimate, and a good one for a salt.
    assert!((nacl - 50.50).abs() / 50.50 < 0.05, "{nacl}");
    // CH3COONa: 2 C + 3 H + 2 O + Na.
    let acetate = kopp_solid_heat_capacity("CH3COONa").unwrap();
    let want = 2.0 * 7.53 + 3.0 * 9.62 + 2.0 * 16.7 + KOPP_OTHER;
    assert!((acetate - want).abs() < 1e-9, "{acetate} vs {want}");
    // Charge is ignored: an ion's atoms are the residue's atoms.
    assert_eq!(
        kopp_solid_heat_capacity("NO3-"),
        kopp_solid_heat_capacity("NO3")
    );
    assert!(kopp_solid_heat_capacity("not a formula ((").is_none());
}

#[test]
fn stranded_ions_without_water_are_charged_as_a_solid() {
    let n = 0.01;
    let v = vessel_with(&RECIPROCAL.map(|k| (k, n, Phase::Aqueous)));
    assert!(v.solvent_gone());
    let cp = v.heat_capacity();
    let want = reciprocal_kopp(n);
    assert!(want > 1.0, "{want}");
    assert!((cp - want).abs() < 1e-9, "{cp} J/K vs Kopp {want}");
    // The ledger integral agrees with the heat capacity it is built from.
    let e = v.energy_between(ROOM, ROOM + 100.0);
    assert!((e - 100.0 * want).abs() < 1e-6, "{e}");
    // And the inversion lands on it: 1 kJ over ~1.5 J/K is hundreds of
    // kelvin, not millions.
    let t = v.temperature_after(1000.0);
    assert!((t - (ROOM + 1000.0 / want)).abs() < 1e-3, "{t}");
}

#[test]
fn ions_in_water_still_carry_no_heat_capacity_of_their_own() {
    // Negative control: the aqueous convention is unchanged while the
    // solvent is there.
    let water = vessel_with(&[("water", 1.0, Phase::Liquid)]);
    let mut salty = RECIPROCAL.map(|k| (k, 0.01, Phase::Aqueous)).to_vec();
    salty.push(("water", 1.0, Phase::Liquid));
    let salty = vessel_with(&salty);
    assert!(!salty.solvent_gone());
    assert_eq!(salty.heat_capacity(), water.heat_capacity());
    assert!(dryness::residue_heat_capacity_event(&salty).is_none());
}

#[test]
fn a_solute_with_a_registry_heat_capacity_keeps_it() {
    // Negative control: Kopp only fills a gap, never overrides data.
    for key in ["sucrose", "Na+"] {
        let data = species::lookup(&SpeciesId::new(key)).unwrap();
        let estimate = vessel::residue_molar(data, Phase::Aqueous, true);
        assert_eq!(estimate.is_some(), data.heat_capacity <= 0.0, "{key}");
        // A solid portion is never estimated, dry or not.
        assert!(vessel::residue_molar(data, Phase::Solid, true).is_none());
        assert!(vessel::residue_molar(data, Phase::Aqueous, false).is_none());
    }
}

#[test]
fn heating_a_dry_residue_stops_at_the_flame_on_real_mass() {
    // E292's second heat: nothing boils this step, so #915's boil-dry hold
    // does not fire. The burner offers only what fits below its flame, and
    // with a real heat capacity that is a finite, booked amount.
    let n = 0.01;
    let mut bench = Bench::new();
    for k in RECIPROCAL {
        bench.vessels[0].deposit(SpeciesId::new(k), Moles(n), Phase::Aqueous);
    }
    let room = bench.vessels[0].energy_between(ROOM, BUNSEN_CEILING_K);
    assert!(room > 1000.0, "{room}");
    let (_, requested, delivered) = heat(&mut bench, 3000.0);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (v.temperature.0 - BUNSEN_CEILING_K).abs() < 1e-6,
        "{} K",
        v.temperature.0
    );
    assert!(
        (delivered - room).abs() / room < 1e-6,
        "{delivered} J delivered vs {room} J the residue holds"
    );
    assert!(requested - delivered > 500.0, "the rest is undelivered");
}

#[test]
fn boiling_dry_puts_the_leftover_on_the_residue_and_says_so() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(0.1), Phase::Liquid);
    for k in RECIPROCAL {
        bench.vessels[0].deposit(SpeciesId::new(k), Moles(0.005), Phase::Aqueous);
    }
    let (events, requested, delivered) = heat(&mut bench, 10_000.0);
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(v.solvent_gone(), "{:#?}", v.contents);
    let keys = note_keys(&events);
    assert!(keys.iter().any(|k| k == KOPP_NOTE), "{keys:?}");
    let latent = 0.1 * states::WATER_H_VAP;
    assert!(delivered >= latent, "{delivered} < {latent}");
    assert!(delivered < requested, "{delivered} of {requested}");
    // The residue at its final temperature holds real, booked energy.
    let held = v.energy_between(ROOM, v.temperature.0);
    assert!(
        held > 0.5 * reciprocal_kopp(0.005) * (v.temperature.0 - ROOM),
        "{held} J"
    );
}

#[test]
fn ample_water_says_nothing_about_a_residue() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    for k in RECIPROCAL {
        bench.vessels[0].deposit(SpeciesId::new(k), Moles(0.005), Phase::Aqueous);
    }
    let (events, requested, delivered) = heat(&mut bench, 10_000.0);
    assert!(!note_keys(&events).iter().any(|k| k == KOPP_NOTE));
    assert!((requested - delivered).abs() < 1e-6);
}
