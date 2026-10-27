//! The embedded parameter subsets must be exactly the upstream FeOs records.
//!
//! Set KEROTAKIS_FEOS_UPSTREAM_DIR to a directory holding the full
//! `esper2023.json` and `rehner2023_binary.json` from feos-org/feos at
//! aabc02bbf6c469cf702206cfa68bbb7b2105a324; otherwise the comparison is
//! skipped (it is printed, not silently passed). Structural checks on the
//! subsets themselves always run.

use serde_json::Value;
use std::path::PathBuf;

fn subset(name: &str) -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn names(r: &Value, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .map(|k| r[*k]["name"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn subsets_carry_source_header_and_expected_records() {
    let p = subset("esper2023.subset.json");
    assert_eq!(
        p["_source"]["upstream_sha256"],
        "87fdcd8733245d90cd6ad9e9d7d0583bd3f7ffb3c4d67a25197190f289e2a679"
    );
    let mut pn: Vec<String> = p["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["identifier"]["name"].as_str().unwrap().to_string())
        .collect();
    pn.sort();
    assert_eq!(pn, ["ethanol", "methanol", "water"]);
    let b = subset("rehner2023_binary.subset.json");
    assert_eq!(
        b["_source"]["upstream_sha256"],
        "8b7d008b2bdf7ec49542eb443e36078fbe46e2737775126589c1718dfa13c335"
    );
    assert_eq!(b["records"].as_array().unwrap().len(), 2);
}

#[test]
fn subsets_equal_upstream_records_when_available() {
    let Some(dir) = std::env::var_os("KEROTAKIS_FEOS_UPSTREAM_DIR") else {
        println!("SKIPPED: KEROTAKIS_FEOS_UPSTREAM_DIR not set");
        return;
    };
    let dir = PathBuf::from(dir);
    let load = |n: &str| -> Vec<Value> {
        serde_json::from_str(&std::fs::read_to_string(dir.join(n)).unwrap()).unwrap()
    };
    let up_pure = load("esper2023.json");
    for r in subset("esper2023.subset.json")["records"]
        .as_array()
        .unwrap()
    {
        let n = &r["identifier"]["name"];
        let u = up_pure
            .iter()
            .find(|u| &u["identifier"]["name"] == n)
            .expect("pure record upstream");
        assert_eq!(r, u, "pure record {n} differs from upstream");
    }
    let up_bin = load("rehner2023_binary.json");
    for r in subset("rehner2023_binary.subset.json")["records"]
        .as_array()
        .unwrap()
    {
        let key = names(r, &["id1", "id2"]);
        let u = up_bin
            .iter()
            .find(|u| names(u, &["id1", "id2"]) == key)
            .expect("binary record upstream");
        assert_eq!(r, u, "binary record {key:?} differs from upstream");
    }
}
