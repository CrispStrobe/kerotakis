# Naming aliases: bounded next lane

Status: [PR789](https://github.com/CrispStrobe/kerotakis/pull/789) is a draft
candidate at `7f7d1f37`, prepared while the original forty wait for a runner.
The approved physical assessment/repair order is unchanged. Rust formatting and
diff checks pass; independent source review closed the input-pack collision and
truncated mission-kit grant findings. Eleven native test groups and fifteen app
cases are prepared but have not executed. CI and the applicable own fleet are
queued. Four separately frozen CLI pairs await a verified candidate executable.
See [candidate binding](../audits/fresh-syntax50-20261013/english-naming-candidate-preparation.json)
and [CLI acceptance forecasts](../audits/fresh-syntax50-20261013/english-naming-cli/design/expectations.json).
Do not integrate before source-bound acceptance. Use auto-merge once review-ready.

## Current behavior and benefit

Integrated main `4d3203a3` canonicalizes exact `H2O` to `water` across operator,
stock, vessel, serialization and Story paths. German/French parsing already
rewrites catalog names, including multiword names, while protecting canonical
grammar and rejecting ambiguous translations. English skips that locale rewrite.
The pure-species registry exposes names, but `species::lookup_key` generally
accepts keys rather than those names. Material recipes have their own aliases.

[Static inventory](../audits/fresh-syntax50-20261013/naming-admission-inventory.json)
examines 186 identities and 132 material recipes on the integrated source.
`oxygen` → `O2`, `nitrogen` → `N2`, `carbon dioxide` → `CO2`, and
`sodium chloride` → `NaCl` are candidates for easier entry. Their display names
have no collisions in that registry snapshot. This is source inspection, not
proof of parser acceptance or collision freedom against grammar and loaded packs.

Formula matching must not infer identity. Existing formula groups contain, for
example, gaseous/dry-ice/aqueous carbon dioxide, nitrogen/liquid nitrogen,
glucose/fructose and several carbon entries. Some formulas are teaching
placeholders. Preserve these distinctions and existing canonical key precedence.

## Implementation lane for a fresh agent

1. Read integrated `species.rs`, `script.rs`, `material.rs`, `ops.rs`, `stock.rs`,
   `vessel.rs`, `build.rs`, the registry identity schema and the water-alias tests.
   Refresh the static inventory against actual main before selecting additions.
   Review grammar words, all locale names and loaded-pack admission as well.
2. Define separate contracts for text names and persisted canonical identity.
   Review the existing identity `synonyms` field before adding any new table.
   Keep canonical keys stable; specify precedence and explicit ambiguity refusal.
   A recipe name must not silently become a pure species. Preserve formula case,
   charge, hydrate, phase and concentration distinctions. Do not auto-admit all
   display names/formulas or use fuzzy matching to execute a command.
3. The current candidate starts with the four mappings above. Freeze expected canonical
   operators and amounts before execution. Support English multiword species
   names in the species argument position without rewriting vessel IDs, filenames,
   comments, labels or unrelated grammar. Preserve German/French behavior and
   longest-name handling. Expand further only after the first tranche is accepted.
4. Use one reviewed identity resolution path for every API that claims to accept
   the new names. Normalize before validation, safety and mutation. Stored keys,
   logs and replay remain canonical. If direct/JSON APIs intentionally require
   canonical keys, document and test that boundary instead of implying parity.
5. Test each admitted spelling against its canonical spelling for parsed operator,
   unit conversion, finite shared stock, exhaustion without mutation, transfer,
   save/load/replay, safety and Story availability/charging. Include mixed-spelling
   inventory collisions and compensated amounts; never sum independently restored
   alias and canonical entries silently. Add negative controls for material,
   grammar, phase/formula and pack collisions, and locale parity controls.
6. Open a narrow main-based product PR, signed commits, source-bound focused
   native/app/CLI acceptance and the applicable full fleet. Follow the
   [auto-merge policy](PR-WORKFLOW.md). Preserve first failures and verify the
   actual integrated source. Update naming/help docs and this lane on completion.

Done means the admitted spellings are discoverable and resolve deterministically,
with the same inventory/safety behavior as their canonical forms. Better naming
does not establish chemical support, calibration, or a new fifty-case score.
