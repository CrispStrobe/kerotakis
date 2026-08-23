use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::Equilibrator;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn make_nacl_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.temperature = Kelvin(298.15);
    v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
    v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
    v
}

fn make_acid_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.temperature = Kelvin(298.15);
    v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
    v.deposit(SpeciesId::new("HCl"), Moles(0.01), Phase::Aqueous);
    v
}

fn bench_engine_init(c: &mut Criterion) {
    c.bench_function("PhreeqcEquilibrator::new (3 databases)", |b| {
        b.iter(|| {
            black_box(PhreeqcEquilibrator::new().unwrap());
        })
    });
}

fn bench_equilibrate_nacl(c: &mut Criterion) {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    c.bench_function("equilibrate (NaCl 0.1 mol)", |b| {
        b.iter_batched(
            make_nacl_vessel,
            |mut v| {
                let _ = black_box(eq.equilibrate(&mut v));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_equilibrate_acid(c: &mut Criterion) {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    c.bench_function("equilibrate (HCl 0.01 mol)", |b| {
        b.iter_batched(
            make_acid_vessel,
            |mut v| {
                let _ = black_box(eq.equilibrate(&mut v));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_cache_hit(c: &mut Criterion) {
    let mut eq = PhreeqcEquilibrator::new().unwrap();
    let mut v = make_nacl_vessel();
    let _ = eq.equilibrate(&mut v);
    c.bench_function("equilibrate (cache hit)", |b| {
        b.iter_batched(
            make_nacl_vessel,
            |mut v| {
                let _ = black_box(eq.equilibrate(&mut v));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_database_parse(c: &mut Criterion) {
    c.bench_function("dbindex::parse (wateq4f)", |b| {
        let raw = kerotakis_phreeqc::databases::WATEQ4F;
        b.iter(|| {
            black_box(kerotakis_phreeqc::dbindex::DbIndex::parse(raw));
        })
    });
}

criterion_group!(
    benches,
    bench_engine_init,
    bench_equilibrate_nacl,
    bench_equilibrate_acid,
    bench_cache_hit,
    bench_database_parse,
);
criterion_main!(benches);
