//! OPT-5: Allocation profiling for the solve hot path.
//!
//! Run with: cargo test -p kerotakis-core --test allocation_profile -- --nocapture
//!
//! Reports allocation counts for key operations. Use dhat-viewer for
//! detailed call-site attribution.

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};

fn make_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.temperature = Kelvin(298.15);
    v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
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

#[test]
fn allocation_budget() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // 1. Species lookups (should be cheap after OPT-4 HashMap)
    for _ in 0..100 {
        let _ = kerotakis_core::species::lookup_key("water");
        let _ = kerotakis_core::species::lookup_key("NaCl");
    }

    // 2. Kinetics integration (the main hot path)
    let mut v = make_vessel();
    let _ = kerotakis_core::kinetics::advance(&mut v, 0.1);

    // 3. Conservation audit
    for _ in 0..10 {
        let _ = kerotakis_core::ledger::ConservedLedger::from_vessel(&v);
    }

    let stats = dhat::HeapStats::get();
    eprintln!("\n=== OPT-5: Combined allocation profile ===");
    eprintln!("  Total allocations:  {}", stats.total_blocks);
    eprintln!("  Total bytes:        {}", stats.total_bytes);
    eprintln!("  Peak blocks:        {}", stats.max_blocks);
    eprintln!("  Peak bytes:         {}", stats.max_bytes);

    // Budget gate: this number should decrease as OPT-5 progresses.
    // Baseline 2026-08-23: ~500 blocks for the combined workload.
    assert!(
        stats.total_blocks < 5_000,
        "combined workload allocated {} blocks — investigate hot spots",
        stats.total_blocks
    );
}
