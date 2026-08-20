#![no_main]
use arbitrary::Arbitrary;
use kerotakis_core::*;
use libfuzzer_sys::fuzz_target;

// Arbitrary operator sequences through the bench loop with the pure-core
// solver stack (no PHREEQC, no CEA — those are fuzzed at their own
// boundaries; keeping them out keeps throughput high). Floats arrive raw,
// NaN and infinity included: `1e999mol` parses to infinity in the real
// grammar, so non-finite amounts are reachable states, not fantasy.
#[derive(Arbitrary, Debug)]
enum FuzzOp {
    New,
    Add {
        vessel: u8,
        species: u8,
        custom: Option<String>,
        moles: f64,
        at: Option<f64>,
    },
    Heat { vessel: u8, energy: f64 },
    Cool { vessel: u8, energy: f64 },
    Stir { vessel: u8 },
    Wait { seconds: f64 },
    Ignite { vessel: u8 },
    Filter { from: u8, to: u8 },
    Decant { from: u8, to: u8, fraction: f64 },
    Evaporate { vessel: u8, fraction: f64 },
    Measure { vessel: u8, which: u8 },
}

const SPECIES: [&str; 12] = [
    "water", "NaCl", "HCl", "NaOH", "NaHCO3", "CH3COOH", "KMnO4", "FeSO4", "Mg", "CaCO3",
    "CuSO4", "H2SO4",
];

fn vid(v: u8) -> VesselId {
    VesselId((v % 4) as usize)
}

fuzz_target!(|ops: Vec<FuzzOp>| {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    for op in ops.into_iter().take(24) {
        let op = match op {
            FuzzOp::New => Operator::NewVessel,
            FuzzOp::Add {
                vessel,
                species,
                custom,
                moles,
                at,
            } => Operator::Add {
                vessel: vid(vessel),
                species: SpeciesId::new(
                    custom
                        .as_deref()
                        .unwrap_or(SPECIES[(species as usize) % SPECIES.len()]),
                ),
                moles: Moles(moles),
                at: at.map(Kelvin),
            },
            FuzzOp::Heat { vessel, energy } => Operator::Heat {
                vessel: vid(vessel),
                energy: Joules(energy),
            },
            FuzzOp::Cool { vessel, energy } => Operator::Cool {
                vessel: vid(vessel),
                energy: Joules(energy),
            },
            FuzzOp::Stir { vessel } => Operator::Stir { vessel: vid(vessel) },
            FuzzOp::Wait { seconds } => Operator::Wait { seconds },
            FuzzOp::Ignite { vessel } => Operator::Ignite { vessel: vid(vessel) },
            FuzzOp::Filter { from, to } => Operator::Filter {
                from: vid(from),
                to: vid(to),
            },
            FuzzOp::Decant { from, to, fraction } => Operator::Decant {
                from: vid(from),
                to: vid(to),
                fraction,
            },
            FuzzOp::Evaporate { vessel, fraction } => Operator::Evaporate {
                vessel: vid(vessel),
                fraction,
            },
            FuzzOp::Measure { vessel, which } => Operator::Measure {
                vessel: vid(vessel),
                instrument: match which % 4 {
                    0 => Instrument::Thermometer,
                    1 => Instrument::Balance,
                    2 => Instrument::PhMeter,
                    _ => Instrument::Eyes,
                },
            },
        };
        let _ = bench.step_with(op, &mut stack, &PermissiveScreen);
    }
});
