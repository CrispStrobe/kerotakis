# Remaining final-state safety rule matrix

## Current next step after the parser prerequisite

The parser prerequisite [PR783](https://github.com/CrispStrobe/kerotakis/pull/783)
is integrated as `b246408b`. The nested-rate admission repair PR784 remains
unintegrated and precedes this safety replay. PR782's existing head `24dda97b`
and its old-head results remain unchanged.
[Source carry-forward review](source-carry-forward-review-24dda97b.json)
finds no merge conflicts against accepted main or the prospective PR784 source:
only the original two safety files change, and both original safety blobs are
preserved. This is source feasibility, not a new candidate or test execution.
After PR784 integrates, bind the replay to actual fresh main and validate the
resulting new head with its own CI, native controls, fleet and supported CLI
witnesses. Preserve all original forecasts, freezes, failures and outcomes.

## Original baseline and validation history

Baseline source: `a125c88d5e4e97cfc55eb2f51e0490e9cb0ed21b`, tree
`cbe987d1fba5eaeaecbd7a23232fb2105eee8684` (accepted main after PR781).

The original fifty controls and expectations are frozen in [freeze.json](freeze.json).
The [fixture](fixtures/production_remaining_final_rules.rs) was authored before
execution. These are source-informed engineering contracts using synthetic
inventories and solver proposals, rather than blind chemistry experiments or
empirical accuracy evidence. The independent static forecast of 24 passes and
26 failures is a forecast, not an observed result.

The tranche covers `bleach-weak-acid-chlorine`, `oxidizer-flammable-gas`,
`oxidizer-reducer` and `water-reactive-base`, all already present in the reviewed
incompatibility matrix but absent from the six-rule final-state whitelist.
Each rule has introduction, unchanged/reordered identity, benign, portion cutoff,
older-warning priority, accepted journal and discarded refinement controls.
Additional controls cover distinct partners, existing reporting aliases and the
separate weak-acidity cutoff. Every expected warning checks all metadata fields.
Gas controls explicitly own a sealed finite headspace. The base routing control
retains pre-existing slaking and checks that an additional base warning survives
only an accepted proposal. Neutral NaCl titration and synthetic pH isolate routing.

[hosted_baseline.py](hosted_baseline.py) compares every regular source blob with
Git, verifies recursive submodule commits/status, captures a freshly resolved
actual dependency lock, injects the unchanged frozen fixture and records raw
commands, tool versions, exits and outcomes. It runs all fifty contracts and the
complete affected library suites. Successful collection can contain contract
failures; compilation errors or incomplete test inventories fail collection.
Ignored submodule/cache contents are outside the byte comparison. Ordinary CI,
both native platforms, production CLI routes and empirical chemistry remain
separate acceptance gates.

The dedicated `remaining-final-rules-baseline` dispatch in
[the audit workflow](../../.github/workflows/chemistry-audit.yml) runs this collector
without scheduling the regular fleets. Local execution was deferred because the
resource gate rejected the host load. The separately captured [local original baseline](local-original-baseline.json)
completed **24 pass / 26 fail**, with **604 core + 28 safety library tests passing**.
[Independent review](local-baseline-independent-review.json) verifies source,
inputs, actual lock and exact unfiltered outcomes. All failures concern omitted
final warnings; four accepted-journal tests abort before later inventory checks.
The local pre-existing offline lock and reused ignored native inputs remain
explicitly qualified and cannot certify the separate hosted run.

[Draft PR782](https://github.com/CrispStrobe/kerotakis/pull/782), head `24dda97b`,
adds only the four reviewed whitelist entries and the unchanged fifty controls.
The separate [source-bound candidate run](local-candidate-fifty.json) now passes
**all fifty controls**, including the later accepted-journal inventory assertions.
The first library invocation was rejected by the resource gate at load6.14;
a fresh gate later opened and the complete library run passed604core+28safety.
[Independent fifty-only review](local-candidate-independent-review.json) and
formatting pass. Root verified all632 actual library identities against the
original baseline and rebound candidate source/fixture/lock after execution.
The [prior404 focused controls and targeted lint](local-regressions-review-24dda97b.json)
now pass. Both own [Linux](native-linux-review-24dda97b.json) and
[macOS](native-macos-review-24dda97b.json) CI verify454focused/38fixtures plus
649core/data/safety library tests. Ordinary CI's actual lock remains uncaptured;
the [separate hosted original baseline](hosted-original-baseline-review.json)
captures its own actual lock and confirms24pass/26fail+632librarypass.

WholeCI38253547880 **failed preflight** on archived `unsafe-libyaml` through the
existing YAML parser dependency. The
[separate parser migration prerequisite](../yaml-parser-migration-20261012/README.md)
keeps advisory policy unchanged and freezes original compatibility expectations
before replacing the dependency. Fleet38253547887 and supported CLI witnesses
remain pending. At that checkpoint PR782 was a draft and main was `a125c88d`; its source-specific native
passes cannot automatically certify a future source including the parser change.


Next executable lanes:

1. Finish PR784's own source-bound fleet and guarded integration. Capture actual
   fresh main afterward; do not substitute its prospective head for accepted main.
2. Carry forward PR782's four whitelist additions and unchanged fifty-test
   fixture onto that main. Verify the exact two-file diff and original safety
   blobs; resolve any new conflicts explicitly. Record the resulting new head,
   tree and actual lock. Run the unchanged fifty controls and complete affected
   libraries when the resource gate permits, or use its own hosted checks.
3. Derive the complete focused native fixture inventory from that actual source;
   retain all prior six-rule contracts. Require the candidate's own CI on both
   native platforms and complete fleet before integration. Establish actual
   surviving inventories and supported solver domains before claiming CLI
   witnesses. Unsupported chemistry routes remain explicit model boundaries.
4. Integrate through fresh head/base guarded checks. Update public status and the
   private operating handoff with actual results and remaining scope.

The callback uses the fixed location `vessel`, so this tranche makes no
multi-location identity claim. Weak-acid findings use derived H+ and do not
identify individual weak-acid bottles. Additional reporting aliases,
intermediate-product observation and physical predictions are outside scope.
