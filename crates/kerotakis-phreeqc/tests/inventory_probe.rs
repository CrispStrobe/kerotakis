#![cfg(feature = "engine")]
//! TEMPORARY diagnostic (one CI round): what the acid/base basis is built
//! from, on both `aq-023` orderings. Fails on purpose so the numbers print.

use kerotakis_core::*;
use kerotakis_phreeqc::inventory::{ProbeRecord, PROBE};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn run(second: &str, third: &str) -> (Vec<ProbeRecord>, f64, f64) {
    PROBE.lock().unwrap().clear();
    let mut bench = Bench::new();
    let mut solvers = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    for line in ["add v1 water 100mL", second, third] {
        let op = kerotakis_core::script::parse_op(line)
            .expect("parses")
            .expect("known verb");
        bench
            .step_with(op, &mut solvers, &PermissiveScreen)
            .expect("the step runs");
    }
    let vessel = bench.vessel(VesselId(0)).expect("the beaker");
    let moles = |key: &str, phase: Phase| -> f64 {
        vessel
            .contents
            .iter()
            .filter(|p| p.species.0 == key && p.phase == phase)
            .map(|p| p.moles.0)
            .sum()
    };
    let records = PROBE.lock().unwrap().clone();
    (
        records,
        moles("water", Phase::Liquid),
        moles(species::BASE_EQUIVALENTS, Phase::Aqueous),
    )
}

fn show(tag: &str, r: &ProbeRecord) -> String {
    let old_base = (-r.excess_difference).max(0.0);
    let old_water = r.target_o - r.booked_o - old_base;
    format!(
        "{tag}\n  target   H={:.17e} O={:.17e} q={:.17e}\n  booked   H={:.17e} O={:.17e} q={:.17e}\n  \
         core     H={:.17e} O={:.17e}\n  basis_excess={:.17e} gas_excess={:.17e}\n  \
         excess: difference={:.17e} reassociated={:.17e} charge={:.17e}\n  \
         new water={:.17e} acid={:.17e} base={:.17e}\n  old water={:.17e} base={:.17e}\n",
        r.target_h,
        r.target_o,
        r.target_charge,
        r.booked_h,
        r.booked_o,
        r.booked_charge,
        r.core_h,
        r.core_o,
        r.basis_excess,
        r.gas_excess,
        r.excess_difference,
        r.excess_reassociated,
        r.excess_charge,
        r.water,
        r.acid,
        r.base,
        old_water,
        old_base,
    )
}

#[test]
fn probe() {
    let (salt, salt_water, salt_base) = run("add v1 CaCl2 0.01mol", "add v1 laundry_detergent 5g");
    let (powder, powder_water, powder_base) =
        run("add v1 laundry_detergent 5g", "add v1 CaCl2 0.01mol");

    let mut report = format!(
        "\nsalt first   contents[water]={salt_water:.17e} base_equivalents={salt_base:.17e}\n\
         powder first contents[water]={powder_water:.17e} base_equivalents={powder_base:.17e}\n\
         salt records={} powder records={}\n",
        salt.len(),
        powder.len()
    );
    for (tag, set) in [("SALT", &salt), ("POWDER", &powder)] {
        for (i, r) in set.iter().enumerate().rev().take(6).rev() {
            report.push_str(&show(&format!("{tag}[{i}]"), r));
        }
    }
    let (a, b) = (
        salt.last().copied().unwrap(),
        powder.last().copied().unwrap(),
    );
    report.push_str(&format!(
        "last-record spread:\n  difference route  {:.6e}\n  reassociated route {:.6e}\n  \
         charge route      {:.6e}\n  within salt:   reassoc-difference {:.6e}  charge-difference {:.6e}\n  \
         within powder: reassoc-difference {:.6e}  charge-difference {:.6e}\n",
        (a.excess_difference - b.excess_difference).abs(),
        (a.excess_reassociated - b.excess_reassociated).abs(),
        (a.excess_charge - b.excess_charge).abs(),
        (a.excess_reassociated - a.excess_difference).abs(),
        (a.excess_charge - a.excess_difference).abs(),
        (b.excess_reassociated - b.excess_difference).abs(),
        (b.excess_charge - b.excess_difference).abs(),
    ));
    panic!("{report}");
}
