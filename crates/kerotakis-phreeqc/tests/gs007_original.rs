//! One unchanged original; a stopped route or unchanged solid is not credit.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
fn emit(name: &str, data: &serde_json::Value) {
    if let Some(path) = std::env::var_os("KEROTAKIS_GS007_EVIDENCE_DIR") {
        std::fs::write(
            std::path::PathBuf::from(path).join(name),
            serde_json::to_vec_pretty(data).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn unchanged_gs007_requires_dissolution_then_increased_carbonate_solid() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/gs007-original-20261016.json")).unwrap();
    let mut bench = Bench::new();
    let mut stack =
        kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())]);
    let mut frames = Vec::new();
    let mut rich = None;
    let mut solver_failed = false;
    for line in original["script"].as_str().unwrap().lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let command = script::parse_command(line, Locale::EN).unwrap();
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let events = match command.operator {
            Some(op) => bench.step_with(op, &mut stack, &PermissiveScreen).unwrap(),
            None => {
                assert!(matches!(
                    command.canonical.as_str(),
                    "register lv3" | "inspect v1" | "inspect"
                ));
                Vec::new()
            }
        };
        solver_failed |= events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. }));
        if command.canonical == "inspect v1" && rich.is_none() {
            rich = Some(bench.vessels[0].clone());
        }
        frames.push(serde_json::json!({"command":line,"canonical":command.canonical,"before":before,"after":bench.vessels,"events":events}));
    }
    let rich = rich.unwrap();
    let final_v = &bench.vessels[0];
    let solid = |v: &Vessel| {
        v.contents
            .iter()
            .filter(|p| p.species.0 == "CaCO3" && p.phase == Phase::Solid)
            .map(|p| p.moles.0)
            .sum::<f64>()
    };
    let dissolved_before = 0.0005 - solid(&rich);
    emit(
        "original.json",
        &serde_json::json!({"original":original,"frames":frames,"rich":rich,"after":bench.vessels,"rich_solid_mol":solid(&rich),"final_solid_mol":solid(final_v),"dissolved_rich_mol":dissolved_before,"solver_failed":solver_failed}),
    );
    assert!(
        !solver_failed,
        "A native solver failure is not qualification"
    );
    assert!(
        dissolved_before > OBSERVABLE_MOLES,
        "GS-007-C1: CO2-rich stage must actually dissolve a measurable calcium fraction"
    );
    assert!(
        solid(final_v) > solid(&rich),
        "GS-007-C2: swept stage must increase carbonate solid; rich={:.17e}mol final={:.17e}mol",
        solid(&rich),
        solid(final_v)
    );
}
