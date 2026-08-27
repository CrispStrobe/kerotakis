use kerotakis_codex::{Codex, CodexExport, Vocabulary};
use std::fs;
use std::path::Path;

fn load_codex(dir: &Path) -> Codex {
    // load_dir, not a walk of its own: it finds codex/i18n/*.toml, and a
    // loader that does not would export the catalogue in English while
    // every test still passed.
    Codex::load_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
}

fn load_vocabulary(dir: &Path) -> Vocabulary {
    let path = dir.join("concepts.toml");
    match fs::read_to_string(&path) {
        Ok(text) => Vocabulary::parse(&text).unwrap(),
        Err(_) => Vocabulary::default(),
    }
}

#[test]
fn codex_export_matches_golden_snapshot() {
    let codex_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("codex");
    let codex = load_codex(&codex_dir);
    let vocabulary = load_vocabulary(&codex_dir);
    let export = CodexExport::build(&codex, &vocabulary);
    let current = serde_json::to_string_pretty(&export).unwrap();

    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let golden = golden_dir.join("codex-export.json");
    if !golden.exists() {
        fs::create_dir_all(&golden_dir).unwrap();
        fs::write(&golden, &current).unwrap();
        eprintln!("golden created at {}", golden.display());
        return;
    }
    let expected = fs::read_to_string(&golden).unwrap();
    if current != expected {
        let actual = golden.with_extension("actual.json");
        fs::write(&actual, &current).unwrap();
        panic!(
            "codex export drifted from the golden snapshot — diff {} against {}",
            golden.display(),
            actual.display()
        );
    }
}
