#![no_main]
use libfuzzer_sys::fuzz_target;

// BRD-041: the mechanism front end is the only parser in the engine that
// turns an untrusted file into executable rate laws, and BRD-040 relaxed
// two of its guards so that real literature could be written down —
// negative activation energies, and explicit reaction orders for the
// global steps. Both widen what a hostile document may say.
//
// Refusal is fine; panic is not, and neither is a document that parses
// into a network the evaluator cannot even price.
fuzz_target!(|data: &str| {
    let Ok(mechanism) = kerotakis_core::kinetics::mechanism::parse_yaml(data) else {
        return;
    };
    let summary = mechanism.summary();
    let arena = kerotakis_core::kinetics::mechanism::MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    assert_eq!(
        network.reactions.len(),
        summary.reactions,
        "compiling must not add or drop a reaction"
    );
    for reaction in network.reactions {
        for temperature in [200.0, 1000.0, 3000.0] {
            let k = reaction.forward.arrhenius.rate_constant(temperature);
            assert!(!k.is_nan(), "a parsed rate law produced a NaN rate constant");
        }
    }
});
