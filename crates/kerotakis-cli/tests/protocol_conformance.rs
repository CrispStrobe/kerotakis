//! GUI-001: the EngineHost protocol conformance suite, corpus half.
//!
//! PROTOCOL.md declares the serialized shapes of the step object, the
//! event stream, and Scene JSON v1 to be API. This suite replays the whole
//! lesson corpus through the CLI's `--json` stream — the same builder the
//! MCP server uses, carrying the same `scene` object the wasm `step()`
//! emits — and pins the structure of every line. A renamed field or a
//! retagged event fails here before it breaks a client.
//!
//! Structural pinning, not numeric goldens: numbers are the business of
//! the lesson-replay and acceptance tests; this file cares that every
//! host sees the same *shape*.

use std::process::Command;

fn lessons() -> Vec<std::path::PathBuf> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lessons");
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
    lessons
}

fn assert_scene_shape(scene: &serde_json::Value, context: &str) {
    assert_eq!(scene["scene"], 1, "{context}: scene version");
    let vessels = scene["vessels"].as_array().unwrap_or_else(|| {
        panic!("{context}: scene.vessels is an array");
    });
    for v in vessels {
        for key in [
            "id",
            "label",
            "liquid",
            "solids",
            "bubbling",
            "boundary",
            "temperature_k",
            "pressure_pa",
            "elapsed_s",
            "words",
            "badges",
        ] {
            assert!(
                !v[key].is_null() || key == "liquid",
                "{context}: vessel.{key} present"
            );
        }
        assert!(
            v["words"].as_str().is_some_and(|w| !w.is_empty()),
            "{context}: words render"
        );
        assert!(
            v["boundary"].is_string(),
            "{context}: boundary is the Headspace tag"
        );
        if let Some(liquid) = v["liquid"].as_object() {
            for key in [
                "volume_l",
                "srgb",
                "colour_word",
                "cloudiness",
                "path_length_cm",
            ] {
                assert!(liquid.contains_key(key), "{context}: liquid.{key} present");
            }
            assert_eq!(liquid["srgb"].as_array().map(Vec::len), Some(3));
        }
        for solid in v["solids"].as_array().into_iter().flatten() {
            for key in [
                "species",
                "name",
                "moles",
                "srgb",
                "colour_word",
                "metallic",
            ] {
                assert!(!solid[key].is_null(), "{context}: solid.{key} present");
            }
        }
        for badge in v["badges"].as_array().into_iter().flatten() {
            for key in ["key", "value", "confidence"] {
                assert!(!badge[key].is_null(), "{context}: badge.{key} present");
            }
        }
    }
}

#[test]
fn every_lesson_step_carries_the_protocol_shapes() {
    for lesson in lessons() {
        let name = lesson.file_name().unwrap().to_string_lossy().into_owned();
        let out = Command::new(env!("CARGO_BIN_EXE_kero"))
            .args(["run", &lesson.to_string_lossy(), "--json"])
            .output()
            .expect("kero runs");
        assert!(
            out.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8(out.stdout).expect("utf8");
        for (lineno, line) in stdout.lines().enumerate() {
            let context = format!("{name}:{}", lineno + 1);
            let step: serde_json::Value =
                serde_json::from_str(line).unwrap_or_else(|e| panic!("{context}: not JSON: {e}"));

            assert!(
                step["operator"]["op"].is_string(),
                "{context}: operator tagged"
            );
            for event in step["events"].as_array().into_iter().flatten() {
                assert!(
                    event["event"].is_string(),
                    "{context}: every event carries its snake_case tag: {event}"
                );
            }
            // Step objects that mutate the bench carry the render model.
            if let Some(scene) = step.get("scene").filter(|s| !s.is_null()) {
                assert_scene_shape(scene, &context);
            }
        }
    }
}
