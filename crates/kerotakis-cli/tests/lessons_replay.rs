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
