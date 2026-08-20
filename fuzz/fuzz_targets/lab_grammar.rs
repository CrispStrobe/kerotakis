#![no_main]
use libfuzzer_sys::fuzz_target;

// Every lesson line, REPL line and MCP call comes through this parser.
// It must reject, never panic.
fuzz_target!(|data: &str| {
    let _ = kerotakis_core::script::parse_op(data);
    let _ = kerotakis_core::script::parse_vessel(data);
});
