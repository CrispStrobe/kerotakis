#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}
#[test]
fn diag36() {
    let keys = ["water", "NaCl", "AgNO3", "Ag+", "Cl-", "Na+", "NO3-"];
    let seed = 36u64;
    let mut bench = Bench::new();
    let mut st = stack();
    for step in 0..6 {
        let mut h = DefaultHasher::new();
        (seed, step).hash(&mut h);
        let r = h.finish();
        let key = keys[(r % keys.len() as u64) as usize];
        let amount = ((r >> 8) % 10_000) as f64 / 1000.0 + 1e-4;
        bench.step_with(Operator::Add { vessel: VesselId(0), species: SpeciesId::new(key), moles: Moles(amount), at: None }, &mut st, &PermissiveScreen).expect("step");
        let ves = bench.vessel(VesselId(0)).unwrap();
        println!("step {step}: add {key} {amount:.4}  -> T={:.3} K", ves.temperature.0);
    }
    let ves = bench.vessel(VesselId(0)).unwrap();
    println!("FINAL T = {:.3} K", ves.temperature.0);
    for p in &ves.contents { println!("    {:<12} {:>12.5} {:?}", p.species.0, p.moles.0, p.phase); }
}
