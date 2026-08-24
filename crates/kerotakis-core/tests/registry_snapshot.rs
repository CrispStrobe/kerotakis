//! CAP-21's equivalence proof: the registry, serialized field-for-field
//! with every curated spectrum evaluated to its sixteen bands, pinned to
//! a golden. Captured from the hand-written table before the switch to
//! pack-driven codegen; the generated table must reproduce it exactly —
//! same entries, same order, same numbers, same spectra — or the
//! migration lost chemistry and this test says so.

use kerotakis_core::species::REGISTRY;
use std::fs;
use std::path::Path;

#[test]
fn registry_matches_the_golden_snapshot() {
    let mut doc = Vec::new();
    for s in REGISTRY {
        let mut v = serde_json::to_value(s).unwrap();
        // `spectrum` is #[serde(skip)] (a function pointer); evaluate it
        // so the golden pins the actual bands, not just the fields.
        v["spectrum_bands"] = match s.spectrum {
            Some(f) => serde_json::to_value(f().to_vec()).unwrap(),
            None => serde_json::Value::Null,
        };
        doc.push(v);
    }
    let current = serde_json::to_string_pretty(&doc).unwrap();

    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/registry.json");
    if !golden.exists() {
        fs::create_dir_all(golden.parent().unwrap()).unwrap();
        fs::write(&golden, &current).unwrap();
        eprintln!("golden created at {}", golden.display());
        return;
    }
    let expected = fs::read_to_string(&golden).unwrap();
    if current != expected {
        let actual = golden.with_extension("actual.json");
        fs::write(&actual, &current).unwrap();
        panic!(
            "registry drifted from the golden snapshot — diff {} against {}",
            golden.display(),
            actual.display()
        );
    }
}
