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
completeness of the ten German/French keys pass. No Cargo or application run,
hosted dispatch, push or PR has been performed for this follow-up. Compilation,
executed failing baseline and repaired acceptance remain pending.

Successful-cut scalar withdrawal, residual cutoffs, owner conservation and
rollback remain outside this change. Additional-solvent `ideal_still` still
returns Option and needs a separate checked diagnostic API. The no-binary-liquid
branch also needs its own unchanged-operation contract. Neither gap is closed
by these binary checked-refusal controls.
