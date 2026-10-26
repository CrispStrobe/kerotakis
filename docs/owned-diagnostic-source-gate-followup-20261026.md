# Owned diagnostic source gate: next-agent scope

This is a follow-up after the active diagnostic delivery. Keep its reviewed head stable. The current199 owned reason keys pass explicit EN/DE/FR catalog/placeholder contracts and actual refusal transcripts; this scope does not allege a currently missing translation or a failed chemistry result.

## Reproduce the observed parser gaps

Use the exact small Rust source strings in [OBSERVATIONS.json](../audits/owned-diagnostic-source-parser-observations-20261026/OBSERVATIONS.json) with `tools/engine-locale-lint.py`'s current helper and patterns. A `#[cfg(test)]` enum variant makes `without_test_modules` find the next unrelated opening brace and drop the following production implementation. A helper returning `Phrase::bare(key, en)` from literal match-arm keys is visible to `MENTIONED` but not the ordinary Phrase-call inventory. Mentioning a key prevents an orphan warning; it does not independently enforce a missing catalog row. The current fixed199 fixture covers these concrete keys, but a newly added helper key must not silently escape the gate.

## Lane1: remove test-only items without consuming production code

Inspect both `tools/engine-locale-lint.py` and `crates/kerotakis-core/tests/composed_locale.rs`; they share the same simple brace-matching strategy. Implement consistent source-item handling, preferably reusing an existing repository lexical scanner if suitable. Respect comments, ordinary/raw strings, nested delimiters and attributes. A test-only enum variant, match arm, function, import or external module declaration must not eat the next production item. Preserve production following inline test modules. Ambiguous supported syntax must fail visibly rather than report a complete inventory.

Acceptance fixtures cover the actual enum-variant reproduction, nested test modules, semicolon-only modules/imports, test functions, cfg match arms, multiple attributes and braces inside strings/comments. Test-only fixture keys stay excluded and adjacent production keys stay included. These are parser/source controls, not mock chemistry. Keep both Rust and Python gate behavior consistent and verify the current source inventory without translating raw vendor bodies.

## Lane2: close the owned helper-key inventory automatically

Derive the finite production `solver-reason.*` key set from source-owned typed definitions or a compiler-owned registry, including match-arm/context helpers. Do not parse English error strings, whitelist the whole namespace or count keys only because they already exist in a catalog. Compare the derived set against actual shipped non-English catalogs and the placeholder contract inventory, so a new typed reason cannot leave a missing translation invisible. Check all shipped locales rather than hardcoding only two languages. Preserve orphan detection and English fallback/placeholder parity. Keep raw/native identifiers as technical labels when no registered term exists.

Acceptance: injecting a new production helper key without a catalog row must fail; adding an orphan row must fail; adding a test-only key must not inflate production coverage; replacing a placeholder or adding a language without its owned-key rows must fail. A fully localized typed helper must pass. Show the exact current derived key set and its equality with the current199 contracts, with no hidden dropped production range or weakened denominator.

## Delivery and boundaries

No chemistry, routing, thresholds, message severity, old JSON shape, raw detail or unknown-label behavior changes. Run the relevant source/locale/parser controls and required repository acceptance. Preserve the original observations and all first failures. Use a narrow branch from actual main, normal reviewed-head auto-merge and updated maintained status/handoff. This future gate hardening earns no original experiment qualification and is not grounds to restart the current valid acceptance head for an optional change.
