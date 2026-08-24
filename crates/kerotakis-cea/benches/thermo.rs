use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kerotakis_cea::nasa9::{self, ThermoDb};
use std::collections::BTreeMap;

fn bench_db_parse(c: &mut Criterion) {
    let text = include_str!("../../../vendor/nasa-cea/thermo.inp");
    c.bench_function("ThermoDb::parse (thermo.inp)", |b| {
        b.iter(|| {
            black_box(ThermoDb::parse(text));
        })
    });
}

fn bench_species_lookup(c: &mut Criterion) {
    let db = nasa9::db();
    c.bench_function("nasa9::db lookup (10 species)", |b| {
        b.iter(|| {
            for name in &[
                "CO2",
                "H2O",
                "O2",
                "N2",
                "CH4",
                "CaCO3(cr)",
                "CaO(cr)",
                "Fe2O3(cr)",
                "Al2O3(a)",
                "SiO2(a)",
            ] {
                black_box(db.get(name));
            }
        })
    });
}

fn bench_cp_evaluation(c: &mut Criterion) {
    let db = nasa9::db();
    let co2 = db.get("CO2").unwrap();
    c.bench_function("Species::cp (CO2, 10 temperatures)", |b| {
        b.iter(|| {
            for t in &[
                300.0, 500.0, 800.0, 1000.0, 1500.0, 2000.0, 2500.0, 3000.0, 4000.0, 5000.0,
            ] {
                black_box(co2.cp(*t));
            }
        })
    });
}

fn budget(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn bench_equilibrate_tp(c: &mut Criterion) {
    let db = nasa9::db();
    let b = budget(&[
        ("Ca", 0.0999),
        ("C", 0.0999),
        ("O", 0.2997 + 0.336),
        ("N", 1.248),
    ]);
    let pool: Vec<&nasa9::Species> = [
        "CO2",
        "CO",
        "O2",
        "N2",
        "NO",
        "CaO(cr)",
        "CaCO3(cr)",
        "Ca(a)",
        "C(gr)",
    ]
    .iter()
    .filter_map(|n| db.get(n))
    .collect();
    c.bench_function("equilibrate_tp (chalk in air, 1500 K)", |b2| {
        b2.iter(|| {
            black_box(kerotakis_cea::equilibrate_tp(&b, &pool, 1500.0, 1.0).unwrap());
        })
    });
}

fn bench_equilibrate_hp(c: &mut Criterion) {
    let db = nasa9::db();
    let b = budget(&[
        ("Ca", 0.0999),
        ("C", 0.0999),
        ("O", 0.2997 + 0.336),
        ("N", 1.248),
    ]);
    let pool: Vec<&nasa9::Species> = [
        "CO2",
        "CO",
        "O2",
        "N2",
        "NO",
        "CaO(cr)",
        "CaCO3(cr)",
        "Ca(a)",
        "C(gr)",
    ]
    .iter()
    .filter_map(|n| db.get(n))
    .collect();
    let warm = kerotakis_cea::equilibrate_tp(&b, &pool, 1000.0, 1.0).unwrap();
    c.bench_function("equilibrate_hp (chalk adiabatic)", |b2| {
        b2.iter(|| {
            black_box(kerotakis_cea::equilibrate_hp(&b, &pool, warm.enthalpy, 1.0).unwrap());
        })
    });
}

criterion_group!(
    benches,
    bench_db_parse,
    bench_species_lookup,
    bench_cp_evaluation,
    bench_equilibrate_tp,
    bench_equilibrate_hp,
);
criterion_main!(benches);
