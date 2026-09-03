//! Every lesson replays in CI (PLAN.md, "Testing is part of the
//! architecture"). A lesson that stops computing — because a solver
//! changed, a species moved, or the grammar drifted — fails the build.
//!
//! Lessons are also the pre-warmed cache's source, so this doubles as a
//! guarantee that the shipped cache covers what the lessons need.

use std::process::Command;

fn lessons_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lessons")
}

fn run(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(args)
        .output()
        .expect("kero runs");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

#[test]
fn every_lesson_replays_and_computes_chemistry() {
    let dir = lessons_dir();
    let mut lessons: Vec<_> = std::fs::read_dir(&dir)
        .expect("lessons directory")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "lab"))
        .collect();
    lessons.sort();
    assert!(
        lessons.len() >= 9,
        "expected the lesson set, found {lessons:?}"
    );

    for lesson in lessons {
        let path = lesson.to_string_lossy().into_owned();
        let name = lesson.file_name().unwrap().to_string_lossy().into_owned();
        let (stdout, stderr, ok) = run(&["run", &path]);
        assert!(ok, "{name} failed to replay:\n{stderr}");
        assert!(
            !stdout.trim().is_empty(),
            "{name} produced no output — a lesson that says nothing teaches nothing"
        );
        // No lesson may end in a solver failure: honest failure is a valid
        // runtime state, but a curated lesson must not rely on one.
        assert!(
            !stdout.contains("solver '"),
            "{name} hit a solver failure:\n{stdout}"
        );
    }
}

#[test]
fn lessons_are_json_clean_at_every_register() {
    // The --json contract must hold for real lesson content, and every
    // register must render every event without panicking.
    let dir = lessons_dir();
    for name in ["fizz.lab", "silver-and-salt.lab", "never-mix.lab"] {
        let path = dir.join(name);
        let (stdout, stderr, ok) = run(&["run", &path.to_string_lossy(), "--json"]);
        assert!(ok, "{name} --json failed:\n{stderr}");
        for line in stdout.lines() {
            let v: serde_json::Value =
                serde_json::from_str(line).unwrap_or_else(|e| panic!("{name}: bad JSON: {e}"));
            assert!(v["operator"]["op"].is_string());
            assert!(v["events"].is_array());
        }
    }
}

#[test]
fn the_prewarmed_cache_covers_the_lessons() {
    // Build the cache from the lessons, then confirm it is non-trivial and
    // round-trips — the shipped-data path (PLAN.md, P2).
    let dir = lessons_dir();
    let out = std::env::temp_dir().join(format!("kero-lessons-{}.postcard", std::process::id()));
    let mut args: Vec<String> = vec!["prewarm".to_string()];
    let mut lessons: Vec<_> = std::fs::read_dir(&dir)
        .expect("lessons directory")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "lab"))
        .collect();
    lessons.sort();
    for l in &lessons {
        args.push(l.to_string_lossy().into_owned());
    }
    args.push("-o".into());
    args.push(out.to_string_lossy().into_owned());

    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (stdout, stderr, ok) = run(&refs);
    assert!(ok, "prewarm failed:\n{stderr}");
    assert!(stdout.contains("pre-warmed"), "{stdout}");

    let bytes = std::fs::read(&out).expect("cache written");
    assert!(
        bytes.len() > 1000,
        "cache suspiciously small: {} bytes",
        bytes.len()
    );
    let data: kerotakis_phreeqc::CacheData =
        postcard::from_bytes(&bytes).expect("cache deserialises");
    assert!(
        data.entries.len() >= 20,
        "expected the lessons' solver results, got {}",
        data.entries.len()
    );
    std::fs::remove_file(&out).ok();
}

/// KID-2: a lesson must demonstrate the claim it makes in its own prose.
///
/// `milk-curds.lab` promises that ten times the vinegar takes the milk past
/// its curdling onset while the tiny dose leaves it a dispersed colloid.
/// With the aqueous solver linked it did neither: `curdling::observe`
/// summed vessel contents whose species id was `CH3COOH`, and the solver
/// had already speciated that into `CH3COO-`, so the dose read zero and the
/// event never fired. The core's own curdling test passed the whole time,
/// because it drives `Bench::step` with no solver behind it — which is why
/// this test lives here, on the binary a reader actually runs.
#[test]
fn the_milk_lesson_curdles_through_the_full_solver_stack() {
    let lesson = lessons_dir().join("milk-curds.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");
    let curds = out.matches("curds").count();
    assert_eq!(
        curds, 1,
        "exactly the high-dose vessel separates — the control must stay a \
         colloid, or the lesson's fair test is not a fair test:\n{out}"
    );
    // The dose that curdles is the second vessel's, so the event has to come
    // after the control's report rather than before it.
    let control = out.find("v1 (beaker)").expect("the control reports");
    assert!(
        out.find("curds").expect("the curd line") > control,
        "the control vessel curdled, which is the bug in reverse:\n{out}"
    );
}

/// KID-5 / EXP-34: rust forms only where air and water are both present, and
/// salt makes the same reaction faster.
///
/// The audit in `KIDS.md` put steel wool in brine under oxygen, waited a day,
/// and got back unchanged iron and the words "this part of the lab isn't
/// awake yet" — a silent miss on one of the first chemical changes a child
/// ever watches. This drives the four-arm lesson through the full solver
/// stack and holds every arm, because three of the four are controls and a
/// control that quietly rusts is worse than no control.
#[test]
fn rusting_needs_air_and_water_and_salt_makes_it_faster() {
    let lesson = lessons_dir().join("rusting.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");

    // Each `inspect` block starts with its vessel header; split on those so
    // an arm's ledger cannot be read off another arm's.
    let arm = |vessel: &str| -> String {
        let mut lines = out.lines().skip_while(|line| {
            !line
                .trim_start()
                .starts_with(&format!("{vessel} (beaker) —"))
        });
        let header = lines
            .next()
            .unwrap_or_else(|| panic!("{vessel} reports:\n{out}"));
        // The ledger runs to the next vessel header, which is the next line
        // that is not indented under this one.
        let body: Vec<&str> = lines
            .take_while(|line| !line.contains(" (beaker) —"))
            .collect();
        format!("{header}\n{}", body.join("\n"))
    };
    let rust_in = |vessel: &str| -> f64 {
        arm(vessel)
            .lines()
            .find(|line| line.contains("iron(III) oxide"))
            .and_then(|line| line.split_whitespace().next()?.parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    // The dry arm has air and no water; the swept arm has water and no air.
    assert_eq!(rust_in("v1"), 0.0, "dry iron rusted:\n{}", arm("v1"));
    assert_eq!(
        rust_in("v2"),
        0.0,
        "iron rusted with the oxygen swept out:\n{}",
        arm("v2")
    );

    let plain = rust_in("v3");
    let salty = rust_in("v4");
    assert!(
        plain > 0.0,
        "nothing rusted in water and air:\n{}",
        arm("v3")
    );
    assert!(
        salty > plain * 1.5,
        "salt water must rust visibly faster than plain: {salty} vs {plain}"
    );

    // The oxygen is consumed, which is why a sealed tin does not rust from
    // the inside — and it is what makes the water rise in the real tube.
    assert!(
        arm("v4").contains("0.0001 mol  oxygen"),
        "the salt arm should run its trapped oxygen down:\n{}",
        arm("v4")
    );
}

/// KID-6: heat is not temperature, and the bench has to be able to say so.
///
/// Boiling announced the transition, left the water liquid, and let the
/// temperature run wherever the energy put it — heating juice on paper
/// reached 670 °C with liquid water still in the ledger. Freezing and
/// melting had paid latent heat since they were written; this holds the
/// third plateau to the same standard, on the binary a reader runs.
#[test]
fn the_boiling_plateau_holds_while_the_water_leaves() {
    let lesson = lessons_dir().join("boiling-curve.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");

    let reading = |vessel: &str| -> f64 {
        out.lines()
            .find(|line| line.contains(&format!("{vessel} thermometer:")))
            .and_then(|line| {
                line.split("thermometer:")
                    .nth(1)?
                    .split_whitespace()
                    .next()?
                    .parse::<f64>()
                    .ok()
            })
            .unwrap_or_else(|| panic!("{vessel} reports a temperature:\n{out}"))
    };
    let water_in = |vessel: &str| -> f64 {
        let header = out
            .find(&format!("{vessel} (beaker) —"))
            .unwrap_or_else(|| panic!("{vessel} reports:\n{out}"));
        out[header..]
            .lines()
            .take_while(|line| !line.contains("mol  chloride"))
            .find(|line| line.contains("mol  water"))
            .and_then(|line| line.split_whitespace().next()?.parse::<f64>().ok())
            .unwrap_or_else(|| panic!("{vessel} reports its water:\n{out}"))
    };

    // Below the plateau the thermometer moves; on it, it does not.
    assert!(reading("v1") < 99.0, "30 kJ must not reach boiling");
    assert!(
        (reading("v2") - 100.0).abs() < 0.01 && (reading("v3") - 100.0).abs() < 0.01,
        "60 kJ and 240 kJ must read the same 100 °C: {} and {}",
        reading("v2"),
        reading("v3")
    );
    // Four times the energy past the plateau buys no degrees and much less
    // water. That contrast is the whole lesson.
    assert!(
        water_in("v3") < water_in("v2") / 4.0,
        "240 kJ must leave far less water than 60 kJ: {} vs {}",
        water_in("v3"),
        water_in("v2")
    );
    // And a dissolved solute raises the plateau by the colligative amount.
    assert!(
        reading("v4") > 103.0 && reading("v4") < 104.0,
        "salt water must boil above 100 °C: {}",
        reading("v4")
    );
}

/// KID-8: five colours out of one pigment, and one of them is in no table.
///
/// The audit's K12 died on `red_cabbage_indicator`, which did not exist —
/// there was no anthocyanin in the registry and the indicator table held
/// only two-form weak acids, which cannot produce five colours. This drives
/// the ladder through the full stack and checks the sequence, including the
/// green that appears where a blue form and a yellow form overlap.
#[test]
fn the_cabbage_rainbow_computes_five_colours_from_one_pigment() {
    let lesson = lessons_dir().join("cabbage-rainbow.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");
    let colour = |vessel: &str| -> String {
        out.lines()
            .find(|line| line.contains(&format!("You look closely at {vessel}.")))
            .and_then(|line| line.split("The liquid is ").nth(1))
            .and_then(|rest| rest.split(" and ").next())
            .unwrap_or_else(|| panic!("{vessel} is looked at:\n{out}"))
            .to_string()
    };
    assert_eq!(colour("v1"), "red", "vinegar holds the flavylium form");
    assert_eq!(
        colour("v2"),
        "deep purple",
        "mildly acidic is the middle form"
    );
    assert_eq!(colour("v3"), "blue", "baking soda takes the next proton");
    assert_eq!(
        colour("v5"),
        "yellow",
        "strong alkali is the top of the ladder"
    );
    // The one nobody tabulated: a blue form and a yellow form present
    // together absorb at both ends and leave a window in the middle.
    assert!(
        colour("v4").contains("green"),
        "washing soda must land in the green between blue and yellow, not on \
         either of them: {}",
        colour("v4")
    );
    // Five jars, five different answers — the point of computing the colour.
    let all: Vec<String> = ["v1", "v2", "v3", "v4", "v5"]
        .iter()
        .map(|v| colour(v))
        .collect();
    let mut unique = all.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        5,
        "five jars must not share a colour: {all:?}"
    );
}

/// KID-9 / EXP-8: a black ink is a mixture, and the strip proves it.
///
/// The audit's K26 and K48 both got "nothing dissolved here has a curated
/// UNIFAC decomposition, so the column's method is silent". The refusal was
/// honest and the gap was real: a food dye is a large glycoside, and a
/// UNIFAC decomposition of one would be a fiction dressed as a calculation.
/// This holds the separation and — the part that matters — that the paper
/// strip and the column agree about the order, because both read one
/// partition coefficient.
#[test]
fn the_ink_strip_and_the_column_cannot_disagree() {
    let lesson = lessons_dir().join("ink-chromatography.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");

    // The mixture separates into three named dyes.
    let mixture = out
        .lines()
        .find(|line| line.contains("The mixture from v1 separates"))
        .unwrap_or_else(|| panic!("v1 is separated:\n{out}"));
    for dye in ["indigo carmine", "betanin", "curcumin"] {
        assert!(
            mixture.contains(dye),
            "the strip must show {dye}: {mixture}"
        );
    }

    // Column order and strip order are the same separation read two ways:
    // the dye the column holds longest is the one furthest up the paper.
    let lv3 = out
        .lines()
        .find(|line| line.contains("Rf=") && line.contains("tR="))
        .unwrap_or_else(|| panic!("the lv3 table reports Rf:\n{out}"));
    let rows: Vec<(f64, f64)> = lv3
        .split(';')
        .filter_map(|part| {
            let tr = part.split("tR=").nth(1)?.split('s').next()?.parse().ok()?;
            let rf = part
                .split("Rf=")
                .nth(1)?
                .split_whitespace()
                .next()?
                .parse()
                .ok()?;
            Some((tr, rf))
        })
        .collect();
    assert_eq!(rows.len(), 3, "three peaks in the lv3 table: {lv3}");
    for pair in rows.windows(2) {
        assert!(
            pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1,
            "retention time and Rf must rank the same way: {rows:?}"
        );
    }

    // A single dye run on its own lands where it landed in the mixture,
    // which is what makes a strip able to identify an unknown.
    assert!(
        out.contains("curcumin (turmeric yellow) 85 mm up"),
        "the pure yellow must run to the same height as it did in the ink:\n{out}"
    );
}

/// KID-7: hot water holds more sugar, and a cooled syrup waits to be asked.
///
/// The audit's K20 found sucrose's saturation limit modelled but
/// temperature-independent — identical at 20, 60 and 90 °C — so the one
/// mechanism rock candy exists to show could not happen. This drives all
/// three states through the full stack: more dissolves hot, nothing comes
/// out on cooling, and a seed brings it down to exactly the limit.
#[test]
fn rock_candy_needs_the_heat_and_then_the_seed() {
    let lesson = lessons_dir().join("rock-candy.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");

    let sucrose = |vessel: &str, phase: &str, nth: usize| -> f64 {
        out.lines()
            .skip_while(|line| !line.contains(&format!("{vessel} (beaker) —")))
            .filter(|line| line.contains("mol  sucrose") && line.contains(phase))
            .nth(nth)
            .and_then(|line| line.split_whitespace().next()?.parse().ok())
            .unwrap_or(0.0)
    };

    // Cold water leaves sugar on the bottom; hot water does not.
    assert!(
        sucrose("v1", "Solid", 0) > 0.2,
        "cold water must leave undissolved sugar:\n{out}"
    );
    assert!(
        sucrose("v2", "Aqueous", 0) > sucrose("v1", "Aqueous", 0) * 1.3,
        "hot water must hold substantially more: {} vs {}",
        sucrose("v2", "Aqueous", 0),
        sucrose("v1", "Aqueous", 0)
    );

    // Cooling reports the state rather than precipitating it. A syrup that
    // crystallises on its own is not the experiment.
    let supersaturated = out
        .lines()
        .position(|line| line.contains("more sucrose in the water in v2 than it can really hold"))
        .unwrap_or_else(|| panic!("cooling must report supersaturation:\n{out}"));
    let seeded = out
        .lines()
        .position(|line| line.contains("A white solid appears at the bottom"))
        .unwrap_or_else(|| panic!("the seed must bring it down:\n{out}"));
    assert!(
        supersaturated < seeded,
        "the syrup must be reported supersaturated before anything seeds it"
    );

    // And it lands on the limit, not somewhere near it: what is left in
    // solution after seeding is exactly what this much water holds.
    let after_solid = sucrose("v2", "Solid", 0);
    assert!(
        after_solid > 0.2,
        "the seed must grow into a real yield: {after_solid}"
    );
}

/// KID-14: slime, and the thing about it that is not like other reactions.
///
/// K21 was the last of the thirty children's scripts that could not run at
/// all: neither poly(vinyl alcohol) nor a borate was on the shelf. Both are
/// now, and the observable is a dose response rather than a reaction —
/// which matters, because the crosslinker is not consumed and the ledger
/// has to show that.
#[test]
fn slime_is_a_dose_response_and_the_borax_survives_it() {
    let lesson = lessons_dir().join("slime.lab");
    let (out, err, ok) = run(&["run", lesson.to_str().expect("utf-8 path")]);
    assert!(ok, "lesson replays: {err}");

    // Too little borax makes no slime; the classroom dose and the excess
    // both do. Three vessels, one threshold crossed between the first two.
    let gelled: Vec<usize> = ["v1", "v2", "v3"]
        .iter()
        .enumerate()
        .filter(|(_, vessel)| {
            out.lines().any(|line| {
                line.contains("turned into slime")
                    && out
                        .lines()
                        .position(|l| l == line)
                        .zip(
                            out.lines()
                                .position(|l| l.contains(&format!("{vessel} (beaker)"))),
                        )
                        .is_some_and(|(gel, header)| gel < header)
            })
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        out.matches("turned into slime").count(),
        2,
        "the under-dosed vessel must not gel and the other two must:\n{out}"
    );
    assert!(!gelled.is_empty());

    // The point a reaction would miss: every gram of borax is still there.
    // A crosslinker links; it is not a reagent.
    let borax_lines: Vec<&str> = out
        .lines()
        .filter(|line| line.contains("mol  sodium tetraborate"))
        .collect();
    assert_eq!(
        borax_lines.len(),
        3,
        "each vessel reports its borax:\n{out}"
    );
    for line in borax_lines {
        let moles: f64 = line
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0.0);
        assert!(moles > 0.0, "the crosslinker is not consumed: {line}");
    }
}
