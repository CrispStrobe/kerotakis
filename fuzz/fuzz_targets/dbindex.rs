#![no_main]
use libfuzzer_sys::fuzz_target;

// The parser that reads the embedded thermodynamic databases. A corrupted
// or truncated database must degrade into a smaller index, not crash —
// the shipped .dat files are trusted, but this parser's robustness is
// what that trust quietly leans on.
fuzz_target!(|data: &[u8]| {
    let _ = kerotakis_phreeqc::dbindex::DbIndex::parse(data);
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = kerotakis_phreeqc::dbindex::parse_formula(s);
        let _ = kerotakis_phreeqc::dbindex::split_hydrate(s);
    }
});
