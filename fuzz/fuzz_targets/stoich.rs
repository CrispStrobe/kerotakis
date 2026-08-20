#![no_main]
use libfuzzer_sys::fuzz_target;

// The formula parser handles Unicode subscripts and superscripts,
// parenthesised groups, hydrate dots, state labels and two charge
// notations — classic fuzz territory. Refusal is fine; panic is not.
fuzz_target!(|data: &str| {
    let _ = kerotakis_core::stoich::parse_equation(data);
    for arrow in ["->", "→", "=", "⇌"] {
        if let Some((l, r)) = data.split_once(arrow) {
            let lhs: Vec<&str> = l.split(" + ").map(str::trim).collect();
            let rhs: Vec<&str> = r.split(" + ").map(str::trim).collect();
            let _ = kerotakis_core::stoich::balance(&lhs, &rhs);
            break;
        }
    }
});
