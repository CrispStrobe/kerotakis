//! The real catalogue, loaded both ways, must come out identical.
//!
//! `codex/*.toml` still carries its `_de` keys inline, and
//! `codex/i18n/de.toml` carries the same 1095 translations as a sidecar.
//! Until this passes, nothing may be deleted from the source — a
//! round-trip on a three-field fixture proves the mechanism, not the
//! migration.
//!
//! When the inline keys are removed this test loses its point and should
//! go with them; what replaces it is the export snapshot, which will then
//! be produced through the sidecar.

use std::path::{Path, PathBuf};

use kerotakis_codex::Codex;

fn codex_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("codex")
}

/// The catalogue as it is on disk: English and German in the same files.
fn inline() -> Codex {
    let dir = codex_dir();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("codex/ is readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();

    let mut all = Codex::default();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("readable");
        let mut c = Codex::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        all.reactions.append(&mut c.reactions);
        all.models.append(&mut c.models);
    }
    all
}

#[test]
fn the_sidecar_reproduces_the_real_catalogue() {
    let a = serde_json::to_value(inline()).expect("inline serialises");
    let b = serde_json::to_value(Codex::load_dir(&codex_dir()).expect("load_dir works"))
        .expect("sidecar serialises");

    if a != b {
        // Report WHERE, not just that they differ: 1095 keys is too many
        // to eyeball, and "not equal" would send the next reader hunting.
        let sa = serde_json::to_string_pretty(&a).unwrap();
        let sb = serde_json::to_string_pretty(&b).unwrap();
        let first = sa
            .lines()
            .zip(sb.lines())
            .enumerate()
            .find(|(_, (x, y))| x != y);
        match first {
            Some((n, (x, y))) => {
                panic!("the two loads differ at line {n}:\n  inline : {x}\n  sidecar: {y}")
            }
            None => panic!(
                "same prefix, different length: {} vs {} lines",
                sa.lines().count(),
                sb.lines().count()
            ),
        }
    }
}

#[test]
fn every_sidecar_key_matches_something() {
    // A key naming a path that no entry has is stale: the English moved
    // and the translation was left describing what is gone. load_dir
    // tolerates a key belonging to a SIBLING file, so this is the check
    // that nothing is orphaned across the whole directory.
    let dir = codex_dir();
    let sidecar = dir.join("i18n/de.toml");
    let text = std::fs::read_to_string(&sidecar).expect("the German sidecar exists");
    let table: toml::Value = toml::from_str(&text).expect("it parses");
    let keys = table.as_table().expect("a flat table").len();

    let loaded = Codex::load_dir(&dir).expect("load_dir works");
    let json = serde_json::to_string(&loaded).expect("serialises");
    let applied = json.matches("_de\":").count();

    assert_eq!(
        applied, keys,
        "{keys} keys in the sidecar but {applied} reached the catalogue — \
         the difference is translations of English that has moved"
    );
}
