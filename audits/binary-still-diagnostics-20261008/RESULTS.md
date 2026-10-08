# Prepared binary still diagnostic integration

This bounded follow-up depends on thermodynamic kernel
[PR #758](https://github.com/CrispStrobe/kerotakis/pull/758), using source
`45c44a4a84a521b909d49840f86d183f4d78fe06`. Before production edits,
commit `8aa5ae09` froze independently declared operator forecasts in
[freeze.json](freeze.json). That forecast commit has not been executed; its
expected failures are not observed baseline evidence.

The caller now uses `ethanol_water_still_checked` and distinguishes all nine
current refusal categories through literal Phrase keys. A fallback handles
future non-exhaustive error variants without pretending their cause is known.
German and French catalogues include all ten keys. Phase-evaluation refusal
reports unavailable supported phase calculation without asserting that every
failure lies outside an Antoine fit. No Event/Operator/save schema changes are
introduced. English remains embedded in translatable Phrase templates.

Every checked binary refusal marks the operation unchanged and returns before
solver processing. The frozen contracts require six independently declared
refusal fixtures to preserve complete serialized bench state except the attempt
log, retain exact category identity and call no downstream solver. A mutating
solver makes accidental processing observable. A positive pure-water cut
requires transferred condensate and normal solver calls for both vessels.

[Prepared source bindings](prepared-repair.json) preserve the frozen test hash
and identify the repair files. Rustfmt, whitespace checks, TOML parsing and
completeness of the ten German/French keys pass. At initial preparation, no Cargo or application run, hosted dispatch, push or
PR had been performed for this follow-up. Compilation,
executed failing baseline and repaired acceptance remain pending.

Successful-cut scalar withdrawal, residual cutoffs, owner conservation and
rollback remain outside this change. Additional-solvent `ideal_still` still
returns Option and needs a separate checked diagnostic API. The additional-solvent API and successful-cut ownership gaps are not closed
by these binary refusal controls.

## Empty-donor follow-up and hosted delivery

Supplementary forecast commit `c5efb416` adds one independent empty-donor
control before its production fix. It preserves the original six-case refusal
and positive-success control bytes in `original-refusal-contracts.rs.txt`;
`freeze.json` continues to bind that original snapshot and forecast revision.
[The supplementary freeze](empty-donor-freeze.json) binds the expanded test file.
The empty-donor branch now marks the operation unchanged, retaining its existing
`NotParameterised` category and `distil-without-water-or-ethanol` Phrase key.
It must preserve full bench state except the attempted-operation log and skip
downstream solvers. Acceptance remains pending.

The change is stacked [PR #759](https://github.com/CrispStrobe/kerotakis/pull/759)
on [PR #758](https://github.com/CrispStrobe/kerotakis/pull/758). Latest unchanged-
production baseline harness `09643a71` carries the exact expanded test bytes;
[run 37731589674](https://github.com/CrispStrobe/kerotakis/actions/runs/37731589674)
is dispatched with execution pending. Superseded dispatch `37731242434` was
cancelled before execution because its workflow attempted to hash untracked
`Cargo.lock` before dependency resolution. Corrected dispatch `37731334165` was
cancelled while queued to include the new frozen empty-donor expectation. Neither
supplies observed behavior evidence. The corrected workflow binds source
`Cargo.toml`, then retains generated `Cargo.lock` and its hash after testing,
including failed tests. No local Cargo or application execution occurred.

[Supplementary preparation bindings](empty-donor-prepared-repair.json) identify
the new source/test hashes separately; original receipts remain immutable.
Merge the thermo prerequisite first, retarget #759 and require fresh main-based
gates. The expected-failure baseline branch must not merge.
