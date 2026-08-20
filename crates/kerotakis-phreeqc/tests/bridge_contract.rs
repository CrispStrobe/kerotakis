//! The shape `web/kerotakis.mjs` hands back must keep deserialising.
//!
//! `SolveOutput` crosses a boundary that no Rust test exercises: the
//! browser's solver hook returns JSON written by JavaScript, and the JS
//! knows only `selected` and `report`. Adding a required field to the
//! struct therefore breaks the demo and nothing else — every native test
//! passes, clippy is happy, and the failure appears only in the two
//! CI jobs that need emscripten and a headless browser to run.
//!
//! That is exactly what happened: a `pe_determined: bool` with no default
//! turned the shipped demo into "the solver's JSON did not parse: missing
//! field `pe_determined`" for every state it had not pre-computed.
//!
//! This test is the cheap native guard for that boundary. It costs
//! microseconds and it fails in the same second as the mistake.

use kerotakis_phreeqc::SolveOutput;

/// Byte-for-byte the object `PhreeqcPool.solve` builds — see
/// `web/kerotakis.mjs`, which stringifies exactly `{selected, report}`.
const WHAT_THE_BROWSER_SENDS: &str = r#"{"selected":[["pH","mu"],["7.0","0.001"]],"report":"..."}"#;

#[test]
fn the_browsers_json_still_deserialises() {
    let out: SolveOutput = serde_json::from_str(WHAT_THE_BROWSER_SENDS).expect(
        "the browser bridge sends only `selected` and `report` — every other field \
                 on SolveOutput must carry #[serde(default)] or the shipped demo stops solving",
    );
    assert_eq!(out.selected.len(), 2);
    assert_eq!(out.report, "...");
}

/// And the defaults it gets must be the harmless ones.
///
/// A field that defaults to the unusual case is worse than a field that
/// fails to parse: the demo would keep answering while quietly never
/// reporting a redox potential again, and nothing would say so. Hence
/// `pe_undetermined` rather than `pe_determined` — false has to mean
/// "nothing out of the ordinary".
#[test]
fn the_defaults_the_browser_gets_are_the_ordinary_ones() {
    let out: SolveOutput = serde_json::from_str(WHAT_THE_BROWSER_SENDS).expect("parses");
    assert!(
        !out.pe_undetermined,
        "a browser solve must default to a well-defined potential, not to a withheld one"
    );
}
