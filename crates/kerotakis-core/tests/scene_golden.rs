//! GUI-003: numeric scene answers for the five release lessons.
//!
//! The whole-corpus protocol tests intentionally pin shape. This test pins
//! the values that shape carries after every executable lesson line, so a
//! renderer cannot silently inherit a changed temperature, colour, volume,
//! phase, gel fraction, or corrosion extent.

use kerotakis_core::{scene, script, Bench};
use serde_json::{Number, Value};
use std::fs;
use std::path::{Path, PathBuf};

const LESSONS: [&str; 5] = [
    "cabbage-rainbow",
    "boiling-curve",
    "water-filter",
    "slime",
    "rusting",
];

fn round_numbers(value: &mut Value) {
    match value {
        Value::Number(number) if number.is_f64() => {
            let value = number.as_f64().expect("a JSON float");
            let rounded = (value * 1_000_000.0).round() / 1_000_000.0;
            *number = Number::from_f64(if rounded == 0.0 { 0.0 } else { rounded })
                .expect("scene floats are finite");
        }
        Value::Array(values) => values.iter_mut().for_each(round_numbers),
        Value::Object(values) => values.values_mut().for_each(round_numbers),
        _ => {}
    }
}

fn replay(path: &Path) -> Value {
    let source = fs::read_to_string(path).expect("lesson source");
    let mut bench = Bench::default();
    let mut frames = Vec::new();

    for (offset, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(operator) = script::parse_op(line)
            .unwrap_or_else(|error| panic!("{}:{}: {error}", path.display(), offset + 1))
        else {
            continue;
        };
        let outcome = match bench.step(operator) {
            Ok(events) => serde_json::json!({
                "events": events.into_iter().map(|event| {
                    serde_json::to_value(event).expect("serializable event")["event"]
                        .as_str().expect("tagged event").to_owned()
                }).collect::<Vec<_>>()
            }),
            Err(error) => panic!(
                "{}:{} canonical GUI-003 command failed: {error}",
                path.display(),
                offset + 1,
            ),
        };
        let mut picture = serde_json::to_value(scene(&bench)).expect("serializable scene");
        round_numbers(&mut picture);
        frames.push(serde_json::json!({
            "line": offset + 1,
            "command": line,
            "outcome": outcome,
            "scene": picture,
        }));
    }
    Value::Array(frames)
}

#[test]
fn five_release_lessons_match_numeric_scene_golden() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut lessons = serde_json::Map::new();
    for name in LESSONS {
        lessons.insert(
            name.to_owned(),
            replay(&root.join("lessons").join(format!("{name}.lab"))),
        );
    }
    let current = format!("{}\n", serde_json::to_string_pretty(&lessons).unwrap());
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/scene-five.json");

    if std::env::var("KEROTAKIS_BLESS_GOLDEN").as_deref() == Ok("1") {
        fs::write(&golden, &current).expect("write blessed GUI-003 golden");
        return;
    }
    let expected = fs::read_to_string(&golden).unwrap_or_else(|_| {
        panic!(
            "GUI-003 golden is absent; review and run \
             KEROTAKIS_BLESS_GOLDEN=1 cargo test -p kerotakis-core --test scene_golden"
        )
    });
    assert_eq!(
        current.trim_end(),
        expected.trim_end(),
        "GUI-003 scene values changed; bless only after reviewing the chemistry and picture"
    );
}
