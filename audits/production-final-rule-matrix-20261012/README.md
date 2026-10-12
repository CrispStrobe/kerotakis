# Remaining final-state safety rule matrix

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
A separate source-bound candidate run has started; candidate outcomes and
native/CI/fleet/integration acceptance are pending.

Next executable lanes:

1. Dispatch the collector on its committed audit revision. Preserve the uploaded
   artifact, verify every manifest hash, the source inventory, actual lock and all
   fifty exact identities. Read first-failure output before proposing a repair.
2. If failures demonstrate missing final-state routing, create a narrow branch
   from the pinned main. Extend only the reviewed rule whitelist, port the frozen
   fixture, and rerun the identical fifty controls and complete affected libraries.
   Preserve the original freeze and baseline outcomes unchanged.
3. Derive the complete focused native fixture inventory from the candidate source;
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
