//! Declared transparent teaching-wash controls, not retail-paint calibration.
use kerotakis_core::script::{parse_command, parse_legacy_command, parse_op};
use kerotakis_core::{material, observe, Bench, Locale, SpeciesId, VesselId};

fn apply(bench: &mut Bench, line: &str) {
    bench.step(parse_op(line).unwrap().unwrap()).unwrap();
}

fn wash(blue: &str, yellow: &str) -> Bench {
    let mut bench = Bench::new();
    apply(&mut bench, &format!("add v1 {blue} 100g"));
    apply(&mut bench, &format!("add v1 {yellow} 100g"));
    bench
}

fn colour(bench: &Bench) -> [u8; 3] {
    let colour = observe(bench.vessel(VesselId(0)).unwrap()).liquid.unwrap();
    [colour.r, colour.g, colour.b]
}

#[test]
fn current_blue_yellow_washes_make_visible_green_and_old_dense_washes_remain_replayable() {
    let old = colour(&wash("watercolour_blue@1", "watercolour_yellow@1"));
    assert!(
        old.into_iter().max().unwrap() < 40,
        "preserve the reported dense baseline: {old:?}"
    );
    let mixed = colour(&wash("watercolour_blue", "watercolour_yellow"));
    assert!(
        mixed[1] > 80
            && mixed[1] > mixed[0].saturating_add(40)
            && mixed[1] > mixed[2].saturating_add(40),
        "declared green teaching mixture: {mixed:?}"
    );
}

#[test]
fn revised_primaries_keep_blue_and_yellow_hues_at_the_declared_dose() {
    let mut blue = Bench::new();
    apply(&mut blue, "add v1 watercolour_blue 100g");
    let rgb = colour(&blue);
    assert!(
        rgb[2] > rgb[0].saturating_add(40) && rgb[2] > rgb[1].saturating_add(40),
        "blue primary: {rgb:?}"
    );
    let mut yellow = Bench::new();
    apply(&mut yellow, "add v1 watercolour_yellow 100g");
    let rgb = colour(&yellow);
    assert!(
        rgb[0] > 160 && rgb[1] > 120 && rgb[2] < 80,
        "yellow primary: {rgb:?}"
    );
}

#[test]
fn teaching_mix_is_order_independent_and_does_not_change_while_waiting() {
    let expected = colour(&wash("watercolour_blue", "watercolour_yellow"));
    let mut reversed = wash("watercolour_yellow", "watercolour_blue");
    assert_eq!(colour(&reversed), expected);
    apply(&mut reversed, "wait 1s");
    assert_eq!(colour(&reversed), expected);
}

#[test]
fn dilution_and_path_length_still_control_absorbance_without_losing_dye() {
    let mut bench = wash("watercolour_blue", "watercolour_yellow");
    let v = bench.vessel(VesselId(0)).unwrap();
    let full = kerotakis_core::solution_optics::absorbance(v, 4.0);
    let half = kerotakis_core::solution_optics::absorbance(v, 2.0);
    for (a, b) in full.iter().zip(half) {
        assert!((a - 2.0 * b).abs() < 1e-10);
    }
    let dyes: Vec<_> = ["indigo_carmine", "curcumin"]
        .iter()
        .map(|key| v.moles_of(&SpeciesId::new(key)))
        .collect();
    let before = colour(&bench);
    apply(&mut bench, "add v1 water 200mL");
    let v = bench.vessel(VesselId(0)).unwrap();
    let diluted = kerotakis_core::solution_optics::absorbance(v, 4.0);
    assert!(full.iter().any(|a| *a > 1e-8));
    for (a, b) in full.iter().zip(diluted) {
        if *a > 1e-8 {
            assert!(b < *a);
        }
    }
    for (key, amount) in ["indigo_carmine", "curcumin"].iter().zip(dyes) {
        assert_eq!(v.moles_of(&SpeciesId::new(key)), amount);
    }
    assert!(
        colour(&bench).into_iter().map(u16::from).sum::<u16>()
            > before.into_iter().map(u16::from).sum::<u16>()
    );
}

#[test]
fn revision_two_is_an_explicit_tenfold_dilution_with_the_same_total_recipe_mass() {
    for key in ["watercolour_blue", "watercolour_yellow"] {
        let current = material::lookup(key, None).unwrap();
        let old = material::lookup_versioned(&current.id, 1).unwrap();
        assert_eq!(current.version, 2);
        let new = current.expand(100.0, 0).unwrap();
        let previous = old.expand(100.0, 0).unwrap();
        assert!((previous.components[0].amount - 10.0 * new.components[0].amount).abs() < 1e-12);
        assert!(
            (new.components
                .iter()
                .map(|component| component.amount)
                .sum::<f64>()
                - 100.0)
                .abs()
                < 1e-12
        );
        assert_eq!(current.confidence, old.confidence);
    }
}

#[test]
fn old_saved_commands_and_new_input_select_different_reviewed_revisions() {
    let line = "add v1 watercolour_blue 100g";
    let old = parse_legacy_command(line, Locale::EN).unwrap();
    let new = parse_command(line, Locale::EN).unwrap();
    assert_eq!(old.canonical, "add v1 watercolour_blue@1 100g");
    assert_eq!(new.canonical, "add v1 watercolour_blue@2 100g");
    let mut old_bench = Bench::new();
    old_bench.step(old.operator.unwrap()).unwrap();
    let mut new_bench = Bench::new();
    new_bench.step(new.operator.unwrap()).unwrap();
    let blue = SpeciesId::new("indigo_carmine");
    assert!(
        (old_bench.vessels[0].moles_of(&blue).0 - 10.0 * new_bench.vessels[0].moles_of(&blue).0)
            .abs()
            < 1e-12
    );
}

#[test]
fn finite_bottles_remain_separate_across_actual_recipe_revisions() {
    let mut bench = Bench::new();
    for line in [
        "stock watercolour_blue@1 100g",
        "stock watercolour_blue 200g",
        "add v1 watercolour_blue@1 25g",
        "add v1 watercolour_blue 50g",
    ] {
        apply(&mut bench, line);
    }
    assert_eq!(
        bench.stock.remaining("watercolour_blue@1").unwrap().amount,
        75.0
    );
    assert_eq!(
        bench.stock.remaining("watercolour_blue@2").unwrap().amount,
        150.0
    );
    assert_eq!(
        bench.stock.remaining("watercolour_blue").unwrap().amount,
        150.0
    );
}
