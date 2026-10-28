//! Lane 4 (#846/#765 generalised): writers name the phase they debit.
//!
//! `Vessel::withdraw` drains a species from whichever portions it meets
//! first, in any phase. For a species that can exist in more than one phase
//! (water/ice/vapour, CO2 gas/aqueous, ethanol liquid/aqueous, ...) that
//! silently debits the wrong compartment while every element total still
//! closes. New writers must use `withdraw_phase` / `withdraw_phases`.
//!
//! This lint walks every crate's `src/` (non-test part: text before the first
//! `#[cfg(test)]`) and compares the per-file count of phase-less
//! `.withdraw(` calls with the reviewed allowlist below. Any difference fails:
//! an increase is a new phase-less writer; a decrease means a site was
//! converted and the allowlist must be tightened.
use std::collections::BTreeMap;
use std::path::Path;

/// (file relative to `crates/`, count, why it is still phase-less).
const ALLOWED: &[(&str, usize, &str)] = &[
    (
        "kerotakis-core/src/adsorption.rs",
        1,
        "sorbate key is data-driven; dissolved vs liquid ownership needs the sorbent contract",
    ),
    (
        "kerotakis-core/src/apparatus.rs",
        1,
        "distillate species are variable; needs a per-component phase from the still model",
    ),
    (
        "kerotakis-core/src/bench.rs",
        3,
        "extraction split (variable key) and the named-React forward/reverse writers (per-row phases needed)",
    ),
    (
        "kerotakis-core/src/combustion.rs",
        1,
        "fuel is deliberately burnt from any condensed phase (documented at the call site)",
    ),
    (
        "kerotakis-core/src/curated.rs",
        1,
        "generic curated row writer; carries the ascorbic/I2 row owned by the halogen lane (#868)",
    ),
    (
        "kerotakis-core/src/displacement.rs",
        4,
        "basis H+, ledger acids counted in liquid+aqueous by design, metal/H2 reduced and oxidised couples (variable)",
    ),
    (
        "kerotakis-core/src/family.rs",
        2,
        "shared family applier reactants (variable) and its virtual water deficit",
    ),
    (
        "kerotakis-core/src/nonaqueous.rs",
        1,
        "solid->liquid relabel of an organic solute; should debit Solid (follow-up, behaviour change)",
    ),
    (
        "kerotakis-core/src/selectivity.rs",
        1,
        "selectivity entry reactant (variable)",
    ),
    (
        "kerotakis-phreeqc/src/continuation_acceptance.rs",
        1,
        "candidate-clone H2O2 debit (liquid/aqueous)",
    ),
];

fn walk(dir: &Path, crates: &Path, out: &mut BTreeMap<String, usize>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            walk(&path, crates, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let rel = path
                .strip_prefix(crates)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if !rel.contains("/src/") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let body = text.split("#[cfg(test)]").next().unwrap_or("");
            let n = body.matches(".withdraw(").count();
            if n > 0 {
                out.insert(rel, n);
            }
        }
    }
}

#[test]
fn phase_less_withdraw_sites_match_the_reviewed_allowlist() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut found = BTreeMap::new();
    walk(crates, crates, &mut found);
    let allowed: BTreeMap<String, usize> = ALLOWED
        .iter()
        .map(|(f, n, _)| (f.to_string(), *n))
        .collect();
    assert_eq!(
        found, allowed,
        "phase-less `.withdraw(` sites changed: use `withdraw_phase`/`withdraw_phases` \
         for a new writer, or tighten ALLOWED after converting one"
    );
}
