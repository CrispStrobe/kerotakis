#![no_main]
use libfuzzer_sys::fuzz_target;

// Every lesson line, REPL line and MCP call comes through this parser.
// It must reject, never panic.
fuzz_target!(|data: &str| {
    let _ = kerotakis_core::script::parse_op(data);
    let _ = kerotakis_core::script::parse_vessel(data);

    // EXP-39: the same bytes again as a titration *endpoint*. Random
    // input almost never spells `titrate v1 KMnO4 0.02M 0.1mL until …`
    // by itself, so the three-way endpoint grammar behind that prefix
    // would be reached by the loop above roughly never. Fixing the
    // prefix and fuzzing only the tail puts every byte the fuzzer
    // generates straight into the branch that is new.
    let endpoint = format!("titrate v1 KMnO4 0.02M 0.1mL until {data}");
    if let Ok(Some(op)) = kerotakis_core::script::parse_op(&endpoint) {
        // Anything the grammar accepts must survive the operator log,
        // which is the save file: an endpoint that parses but cannot be
        // written down is a bench that cannot replay itself.
        let json = serde_json::to_string(&op).expect("an accepted operator serialises");
        let back: kerotakis_core::Operator =
            serde_json::from_str(&json).expect("and deserialises");
        assert_eq!(back, op, "operator round-trip: {endpoint}");
    }
});
