//! Exact original protocol, on the native Rust physics bench; no chemical
//! engine is needed by its water/ethanol separation operators.
use kerotakis_core::*;
fn emit(name: &str, data: &serde_json::Value) {
    if let Some(path) = std::env::var_os("KEROTAKIS_SP003_EVIDENCE_DIR") {
        std::fs::write(
            std::path::PathBuf::from(path).join(name),
            serde_json::to_vec_pretty(data).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn unchanged_original_sp003_requires_both_equal_cut_products() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/sp003-original-20261016.json")).unwrap();
    let mut bench = Bench::new();
    let mut frames = Vec::new();
    for line in original["script"].as_str().unwrap().lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let command = script::parse_command(line, Locale::EN).unwrap();
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let events = match command.operator {
            Some(operator) => bench.step(operator).unwrap(),
            None => {
                assert!(matches!(
                    command.canonical.as_str(),
                    "inspect" | "register lv3"
                ));
                Vec::new()
            }
        };
        frames.push(serde_json::json!({"command":line,"canonical":command.canonical,"before":before,"after":bench.vessels,"events":events}));
    }
    let totals: Vec<_> = ["water", "ethanol"]
        .iter()
        .map(|s| {
            bench
                .vessels
                .iter()
                .map(|v| v.moles_of(&SpeciesId::new(s)).0)
                .sum::<f64>()
        })
        .collect();
    emit(
        "original.json",
        &serde_json::json!({"original":original,"frames":frames,"after":bench.vessels,"totals":totals,"engine_scope":"Native Rust core physics + honesty stack; no aqueous chemical engine"}),
    );
    let amount = |index: usize, s: &str| bench.vessels[index].moles_of(&SpeciesId::new(s)).0;
    let r1 = amount(2, "water") + amount(2, "ethanol");
    let r4 = amount(3, "water") + amount(3, "ethanol");
    assert!(r1>0.0 && r4>0.0,"SP-003-C1 requires two actual nonempty products, got one-stage={r1:.17e}, four-stage={r4:.17e}; a typed refused cut is not qualification");
    assert!(amount(3, "ethanol") / r4 > amount(2, "ethanol") / r1);
    let mm = species::lookup(&SpeciesId::new("ethanol"))
        .unwrap()
        .molar_mass;
    assert!(
        (totals[1] - 10.0 / mm).abs() <= 1e-10,
        "SP-003-C2: no extra ethanol"
    );
}
#[test]
fn independently_supported_small_cut_and_impossible_composition_controls() {
    use kerotakis_thermo::vle::*;
    let small =
        ethanol_water_still_checked(1.0, 1e-14, StillTake::Fraction(0.01), 1, ATMOSPHERE_KPA)
            .unwrap();
    assert!(small.water_over > 0.0 && small.ethanol_over > 0.0);
    assert!(small.water_over < 1.0 && small.ethanol_over < 1e-14);
    for (w, e) in [(1e-300, 1e300), (1e300, 1e-300)] {
        assert!(
            ethanol_water_still_checked(w, e, StillTake::Fraction(0.01), 4, ATMOSPHERE_KPA)
                .is_err()
        );
    }
    assert!(ethanol_water_still_checked(
        1.0,
        0.1,
        StillTake::Fraction(0.1),
        u32::MAX,
        ATMOSPHERE_KPA
    )
    .is_err());
}
