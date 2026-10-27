//! Stale check and family tests for the gas-phase reconciliation
//! (src/gas_phases.rs, docs/gas-phase-coverage-20261027.md).
//!
//! Regenerate after an intended change with
//! `KEROTAKIS_BLESS_GAS_COVERAGE=1 cargo test -p kerotakis-phreeqc --test gas_phase_coverage`.

use kerotakis_phreeqc::dbindex::DbIndex;
use kerotakis_phreeqc::gas_phases::{self, CoverageRow, GasRefusal, GasStatus};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gas_phase_coverage.json"
);
const DOC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/gas-phase-coverage-20261027.md"
);

fn close(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => (a - b).abs() < 1e-6,
        _ => false,
    }
}

fn same(a: &CoverageRow, b: &CoverageRow) -> bool {
    a.gas == b.gas
        && a.route == b.route
        && a.status == b.status
        && a.attempts.len() == b.attempts.len()
        && a.attempts.iter().zip(&b.attempts).all(|(x, y)| {
            x.donor == y.donor
                && x.refusal == y.refusal
                && x.links.len() == y.links.len()
                && x.links.iter().zip(&y.links).all(|(l, m)| {
                    l.species == m.species
                        && close(l.log_k_delta, m.log_k_delta)
                        && close(l.delta_h_delta_kj, m.delta_h_delta_kj)
                })
        })
}

#[test]
fn coverage_table_is_not_stale() {
    let rows = gas_phases::coverage();
    if std::env::var_os("KEROTAKIS_BLESS_GAS_COVERAGE").is_some() {
        std::fs::write(FIXTURE, serde_json::to_string_pretty(&rows).unwrap() + "\n").unwrap();
    }
    let pinned: Vec<CoverageRow> =
        serde_json::from_str(&std::fs::read_to_string(FIXTURE).unwrap()).unwrap();
    assert_eq!(rows.len(), pinned.len(), "gas x route cells changed");
    for (now, was) in rows.iter().zip(&pinned) {
        assert!(same(now, was), "stale row:\n now {now:?}\n was {was:?}");
    }
    let doc = std::fs::read_to_string(DOC).unwrap();
    assert!(
        doc.contains(&gas_phases::markdown_table(&rows)),
        "coverage table in the doc is stale"
    );
    assert!(
        doc.contains(&gas_phases::coupling_table(&rows)),
        "coupling table in the doc is stale"
    );
}

#[test]
fn every_cell_is_native_imported_or_a_typed_refusal() {
    for row in gas_phases::coverage() {
        match row.status {
            GasStatus::Native => assert!(row.attempts.is_empty()),
            GasStatus::Imported => assert!(row.attempts.last().unwrap().refusal.is_none()),
            GasStatus::Refused => assert!(row.attempts.iter().all(|a| a.refusal.is_some())),
        }
    }
}

#[test]
fn routed_indexes_agree_with_the_table() {
    let rows = gas_phases::coverage();
    for (route, bytes) in [
        ("wateq4f", kerotakis_phreeqc::databases::wateq4f()),
        ("minteq.v4", kerotakis_phreeqc::databases::minteq_v4()),
        ("pitzer", kerotakis_phreeqc::databases::pitzer()),
    ] {
        let idx = DbIndex::parse(bytes);
        // Native cells are the database's own business; imports must be
        // visible to the index and refusals must not leak in.
        for row in rows
            .iter()
            .filter(|r| r.route == route && r.status != GasStatus::Native)
        {
            assert_eq!(
                idx.has_phase(&row.gas),
                row.status == GasStatus::Imported,
                "{} on {route}",
                row.gas
            );
        }
    }
}

#[test]
fn ammonia_and_hydrogen_are_imported_verbatim_onto_minteq() {
    let wateq = DbIndex::parse(kerotakis_phreeqc::databases::wateq4f());
    let minteq = DbIndex::parse(kerotakis_phreeqc::databases::minteq_v4());
    for gas in ["NH3(g)", "H2(g)"] {
        let (w, m) = (&wateq.phases[gas], &minteq.phases[gas]);
        assert_eq!(w.log_k, m.log_k, "{gas}");
        assert!(
            (w.delta_h_kj.unwrap() - m.delta_h_kj.unwrap()).abs() < 1e-12,
            "{gas}"
        );
        assert_eq!(w.products, m.products, "{gas}");
    }
    // Same constants the former one-off extension carried (AB-017).
    assert_eq!(minteq.phases["NH3(g)"].log_k, Some(1.77));
    assert!((minteq.phases["NH3(g)"].delta_h_kj.unwrap() - (-8.17 * 4.184)).abs() < 1e-9);
}

#[test]
fn negatives_stay_refused_with_their_reason() {
    let rows = gas_phases::coverage();
    let row = |gas: &str, route: &str| {
        rows.iter()
            .find(|r| r.gas == gas && r.route == route)
            .unwrap()
            .clone()
    };
    // Redox-coupled methane: minteq's CH4(g) dissolves to carbonate + e-,
    // pitzer speciates only C(4).
    let ch4 = row("CH4(g)", "pitzer");
    assert_eq!(ch4.status, GasStatus::Refused);
    assert!(ch4
        .attempts
        .iter()
        .any(|a| a.donor == "minteq.v4"
            && a.refusal == Some(GasRefusal::RedoxStateNotModelledOnRoute)));
    // H2Se: same stoichiometry, log K 8.4 apart.
    let h2se = row("H2Se(g)", "wateq4f");
    assert_eq!(
        h2se.attempts[0].refusal,
        Some(GasRefusal::LinkingLogKOutsideTolerance)
    );
    // Pitzer has no reduced nitrogen at all.
    assert_eq!(row("NH3(g)", "pitzer").status, GasStatus::Refused);
    // Bench gases no vendored database defines.
    for gas in ["Cl2(g)", "SO2(g)", "CO(g)", "He(g)"] {
        for route in gas_phases::ROUTES {
            let r = row(gas, route);
            assert_eq!(
                r.attempts[0].refusal,
                Some(GasRefusal::NoReviewedDonorDefinition),
                "{gas}"
            );
        }
    }
}

const DONOR: &[u8] = b"
SOLUTION_MASTER_SPECIES
H H+ -1 H 1
N(-3) NH4+ 0 N 14
N(5) NO3- 0 N 14
SOLUTION_SPECIES
H+ = H+
    log_k 0
NH4+ = NH4+
    log_k 0
NO3- = NO3-
    log_k 0
NH4+ = NH3 + H+
    log_k -9.25
    delta_h 52 kJ
PHASES
NH3(g)
    NH3 = NH3
    log_k 1.77
    delta_h -8.17 kcal
END
";

fn target(log_k: f64, delta_h: &str) -> Vec<u8> {
    format!(
        "SOLUTION_MASTER_SPECIES\nH H+ -1 H 1\nN(-3) NH4+ 0 N 14\nN(5) NO3- 0 N 14\nSOLUTION_SPECIES\nH+ = H+\n    log_k 0\nNH4+ = NH4+\n    log_k 0\nNO3- = NO3-\n    log_k 0\nNH4+ = NH3 + H+\n    log_k {log_k}\n{delta_h}\nEND\n"
    )
    .into_bytes()
}

#[test]
fn log_k_tolerance_boundary() {
    let inside = -9.25 + gas_phases::LOG_K_TOLERANCE * 0.95;
    let outside = -9.25 + gas_phases::LOG_K_TOLERANCE * 1.05;
    assert_eq!(
        gas_phases::check_import(DONOR, &target(inside, "    delta_h 52 kJ"), "NH3(g)").0,
        None
    );
    assert_eq!(
        gas_phases::check_import(DONOR, &target(outside, "    delta_h 52 kJ"), "NH3(g)").0,
        Some(GasRefusal::LinkingLogKOutsideTolerance)
    );
}

#[test]
fn enthalpy_tolerance_and_unstated_boundary() {
    assert_eq!(
        gas_phases::check_import(DONOR, &target(-9.25, "    delta_h 55.9 kJ"), "NH3(g)").0,
        None
    );
    assert_eq!(
        gas_phases::check_import(DONOR, &target(-9.25, "    delta_h 56.1 kJ"), "NH3(g)").0,
        Some(GasRefusal::LinkingEnthalpyOutsideTolerance)
    );
    assert_eq!(
        gas_phases::check_import(DONOR, &target(-9.25, ""), "NH3(g)").0,
        Some(GasRefusal::LinkingEnthalpyUnstated)
    );
}

#[test]
fn convention_and_absence_are_refused() {
    // Same species, defined from NO3- instead of NH4+: different master basis.
    let other = b"SOLUTION_MASTER_SPECIES\nH H+ -1 H 1\nN(5) NO3- 0 N 14\nSOLUTION_SPECIES\nH+ = H+\n    log_k 0\nNO3- = NO3-\n    log_k 0\nNO3- + 9 H+ + 8 e- = NH3 + 3 H2O\n    log_k 109.8\n    delta_h -1 kJ\nEND\n";
    assert_eq!(
        gas_phases::check_import(DONOR, other, "NH3(g)").0,
        Some(GasRefusal::LinkingSpeciesConventionDiffers)
    );
    let empty =
        b"SOLUTION_MASTER_SPECIES\nH H+ -1 H 1\nSOLUTION_SPECIES\nH+ = H+\n    log_k 0\nEND\n";
    assert_eq!(
        gas_phases::check_import(DONOR, empty, "NH3(g)").0,
        Some(GasRefusal::LinkingSpeciesAbsent)
    );
}

#[test]
fn constant_k_donor_is_refused() {
    let donor = String::from_utf8(DONOR.to_vec())
        .unwrap()
        .replace("    delta_h -8.17 kcal\n", "");
    assert_eq!(
        gas_phases::check_import(
            donor.as_bytes(),
            &target(-9.25, "    delta_h 52 kJ"),
            "NH3(g)"
        )
        .0,
        Some(GasRefusal::DonorPhaseWithoutTemperatureDependence)
    );
}

#[test]
fn only_reviewed_exchange_gases_are_coupled_to_the_solve() {
    for gas in ["NH3(g)", "CO2(g)", "HBr(g)"] {
        assert!(gas_phases::headspace_coupled(gas), "{gas}");
    }
    // #834 siblings: carried as inert headspace inventory, not equilibrated.
    for gas in ["H2S(g)", "SO2(g)", "HCl(g)", "Cl2(g)"] {
        assert!(!gas_phases::headspace_coupled(gas), "{gas}");
    }
}

#[test]
fn last_phase_before_a_keyword_is_native_not_reimported() {
    // Regression (hosted run of d67042d2): the parser dropped the final
    // phase of each PHASES block, so wateq4f's own NH3(g) was "imported"
    // from phreeqc.dat with Peng-Robinson constants and every finite
    // ammonia headspace failed on an ideal/PR gas mixture.
    let rows = gas_phases::coverage();
    for (gas, route) in [
        ("NH3(g)", "wateq4f"),
        ("H2Sg(g)", "pitzer"),
        ("HgI2(g)", "minteq.v4"),
    ] {
        let row = rows
            .iter()
            .find(|r| r.gas == gas && r.route == route)
            .unwrap();
        assert_eq!(row.status, GasStatus::Native, "{gas} on {route}");
    }
    let nh3 = rows
        .iter()
        .find(|r| r.gas == "NH3(g)" && r.route == "minteq.v4")
        .unwrap();
    assert_eq!(nh3.attempts.last().unwrap().donor, "wateq4f");
}

#[test]
fn ideal_gas_is_not_imported_onto_a_peng_robinson_route() {
    let target = format!(
        "{}PHASES\nCO2(g)\n    CO2 = CO2\n    log_k -1.468\n    delta_h -4.776 kcal\n    -T_c 304.2\n    -P_c 72.8\nEND\n",
        String::from_utf8(target(-9.25, "    delta_h 52 kJ")).unwrap().trim_end_matches("END\n")
    );
    assert_eq!(
        gas_phases::check_import(DONOR, target.as_bytes(), "NH3(g)").0,
        Some(GasRefusal::GasEquationOfStateDiffers)
    );
}
