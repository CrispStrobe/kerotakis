//! OPT-1: Benchmark suite for the solve path.
//!
//! Run: cargo bench -p kerotakis-core
//! Compare: cargo bench -p kerotakis-core -- --save-baseline before
//!          <make changes>
//!          cargo bench -p kerotakis-core -- --baseline before

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};

fn make_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.temperature = Kelvin(298.15);
    v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
    v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
    v.deposit(SpeciesId::new("Na2S2O3"), Moles(0.1), Phase::Aqueous);
    v.solution = Some(kerotakis_core::vessel::SolutionInfo {
        redox: Vec::new(),
        pe: None,
        ph: 1.7,
        ionic_strength: 0.02,
        species: Vec::new(),
        provenance: None,
    });
    v
}

fn bench_species_lookup(c: &mut Criterion) {
    c.bench_function("species::lookup (75 species)", |b| {
        b.iter(|| {
            for key in &["water", "NaCl", "HCl", "NaOH", "CuSO4", "AgNO3", "Fe", "Zn"] {
                black_box(kerotakis_core::species::lookup_key(key));
            }
        })
    });
}

fn bench_kinetics_advance(c: &mut Criterion) {
    c.bench_function("kinetics::advance (thiosulfate, 0.1s)", |b| {
        b.iter_batched(
            make_vessel,
            |mut v| {
                let _ = black_box(kerotakis_core::kinetics::advance(&mut v, 0.1));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_mixing_equilibrator(c: &mut Criterion) {
    use kerotakis_core::solve::{Equilibrator, MixingEquilibrator};
    c.bench_function("MixingEquilibrator::equilibrate", |b| {
        let mut solver = MixingEquilibrator;
        b.iter_batched(
            make_vessel,
            |mut v| {
                let _ = black_box(solver.equilibrate(&mut v));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_conservation_audit(c: &mut Criterion) {
    use kerotakis_core::ledger::ConservedLedger;
    c.bench_function("ConservedLedger::from_vessel", |b| {
        let v = make_vessel();
        b.iter(|| {
            black_box(ConservedLedger::from_vessel(&v));
        })
    });
}

fn bench_vessel_clone(c: &mut Criterion) {
    c.bench_function("Vessel::clone (3 species)", |b| {
        let v = make_vessel();
        b.iter(|| {
            black_box(v.clone());
        })
    });
}

criterion_group!(
    benches,
    bench_species_lookup,
    bench_kinetics_advance,
    bench_mixing_equilibrator,
    bench_conservation_audit,
    bench_vessel_clone,
);
criterion_main!(benches);
