//! The pre-warmed cache: build-time solver results, shipped as data, so
//! guided content never waits for an engine on device (PLAN.md, P2).

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn replay(eq: &mut PhreeqcEquilibrator) -> f64 {
    let mut bench = Bench::new();
    let v = VesselId(0);
    for (key, moles) in [
        ("water", 55.51),
        ("NaHCO3", 0.05),
        ("CH3COOH", 0.06),
        ("NaCl", 0.01),
    ] {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                eq,
                &PermissiveScreen,
            )
            .expect("step");
    }
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph
}

#[test]
fn a_prewarmed_cache_serves_a_cold_engine() {
    // Build time: run the lesson once and export.
    let mut builder = PhreeqcEquilibrator::new().expect("engine");
    let expected = replay(&mut builder);
    let exported = builder.export_cache();
    assert!(!exported.entries.is_empty());

    // Ship it: postcard round-trip, exactly as the binary would embed it.
    let bytes = postcard::to_allocvec(&exported).expect("serialise");
    let loaded: kerotakis_phreeqc::CacheData = postcard::from_bytes(&bytes).expect("deserialise");

    // Device: a fresh engine that has never solved anything.
    let mut device = PhreeqcEquilibrator::new().expect("engine");
    let added = device.import_cache(loaded);
    assert_eq!(added, exported.entries.len(), "all entries loaded");

    let before = device.cache_len();
    let got = replay(&mut device);
    assert_eq!(got, expected, "cached answers are bit-identical");
    // The invariant is that nothing reached the engine, and the way to say
    // that without hard-coding a call count is that the cache never grew: a
    // miss would compute a fresh answer and store it. How many solves a
    // lesson makes is not fixed — solubility and temperature are iterated to
    // a common answer, so a step with a heat effect solves more than once.
    assert_eq!(
        device.cache_len(),
        before,
        "every solver-reaching step was served from the cache"
    );
    assert!(device.cache_hits() > 0);
}

#[test]
fn importing_never_overwrites_freshly_computed_results() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    replay(&mut eq);
    let live = eq.cache_len();
    // Importing the same keys must be a no-op.
    let same = eq.export_cache();
    let added = eq.import_cache(same);
    assert_eq!(added, 0);
    assert_eq!(eq.cache_len(), live);
}
