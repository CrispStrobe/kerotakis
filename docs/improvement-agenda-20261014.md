# Ordered improvement agenda — 2026-10-14

The user authorized all six steps below in this order. Step 1 is complete;
step 2 is active and steps 3–6 are queued. This agenda orders the current fresh-experiment gap work;
the annotation and other queued lanes in the [earlier roadmap](ordered-work-roadmap-20261013.md)
resume afterward. Preserve all original predictions, first outcomes, failed CI
logs and versioned source associations. Host-specific operating details belong
in the private handoff. Heavy local work and delegation require resource gates.

1. **Validate and integrate the thermal inventory repair — complete.** PR786
   [integrated as `8a6aa68e`](../audits/fresh-syntax50-20261013/pr786-integration.json)
   after all 25 CI checks and 33 own-fleet jobs passed. The actual main tree
   equals the validated candidate tree. Exact-source native controls verify
   retained inventory below the burst threshold and accounted gas venting when
   mechanical rupture occurs, plus explicit unsupported thermal boundaries and
   existing open/inert/kinetic behavior. Coverage has zero unmet requirements,
   runtime failures or drift. Closed combustion remains an unimplemented domain.
   Preserve the [original replay failures](../audits/fresh-syntax50-20261013/pr786-replay-first-outcomes.json),
   [matched low-pressure passes](../audits/fresh-syntax50-20261013/pr786-low-pressure-outcomes.json),
   [qualified support claims](../audits/fresh-syntax50-20261013/pr786-coverage-correction-preparation.json)
   and [native acceptance](../audits/fresh-syntax50-20261013/pr786-current-native-validation.json).
   Do not repeat this completed lane on unchanged source.
2. **Canonical water alias — active.** [Draft PR787](https://github.com/CrispStrobe/kerotakis/pull/787)
   now contains the corrected app accounting predicate at `d5c6cecf` on integrated
   main `8a6aa68e`: seven core files and four app files. The
   [guarded application](../audits/fresh-syntax50-20261013/water-alias-accounting-followup-application.json)
   records the normal fast-forward after the previous runs completed.
   Fresh CI `38673902939` and own fleet `38673902811` are active on this head.
   Require actual execution of eight new app regressions, all-app Vitest,
   type/browser/native checks and the complete candidate fleet before integration.
   Preserve any first failure and do not cancel or duplicate active runs.

   Previous source `98051f7b` [completed its own fleet with 33 passes](../audits/fresh-syntax50-20261013/water-alias-app-candidate-completion.json).
   CI had 23 passes, two failures and one skip. Both failures were the same
   visualization guard: it classified non-visual stock comparisons in Session as
   drawing handlers. The [first app suite](../audits/fresh-syntax50-20261013/water-alias-app-first-failure.json)
   had 1,636 passes and one failure; its seven new regressions were included.
   The applied correction moves the predicate into `storyStock.ts`, preserves
   the guard and scene classifications, and adds a direct accounting test.
   That eighth regression and corrected-source acceptance are still pending.

   The previous source's [native regressions](../audits/fresh-syntax50-20261013/water-alias-app-candidate-native-validation.json)
   and workspace/CODEX/coverage passed, with zero gaps or drift. Its
   [verified own CLI archive](../audits/fresh-syntax50-20261013/water-alias-app-candidate-artifact-review.json)
   has exactly the earlier executable/lock bytes. The first two
   [CLI controls remain failed](../audits/fresh-syntax50-20261013/water-alias-cli/README.md)
   due to an invalid final command; two separately frozen
   [corrected controls passed](../audits/fresh-syntax50-20261013/water-alias-cli-followup-outcomes.json)
   with unchanged forecasts. Core/CLI/WASM is unchanged in the app descendant;
   retain those source bindings and do not duplicate unchanged CLI controls.
   These receipts do not establish corrected app acceptance.

   Follow the [implementation and acceptance lane](water-alias-task-20261014.md).
   Accept exact `H2O` as input for canonical `water`, persist canonical identities,
   and share one finite stock balance across text/direct/JSON inputs, inventory,
   transfers, extracts and save/replay. Story access and dispense gates must use
   engine-parsed identities. Charge only actual additions or positive-volume
   titration, preserving refused and zero-dose requests. Avoid broadening formula
   aliasing without a separate reviewed contract.
3. **Execute and assess the forty naming follow-ups.** The
   [forty controls](../audits/fresh-syntax50-20261013/normalized40/design/README.md)
   already freeze `H2O`→`water` substitutions and unchanged physical forecasts
   on original main `30fd09eb`/binary `b290`. Run that source-bound control set
   when resources permit, preserving it separately from any alias/fix replay.
   For actual integrated source, freeze a new replay binding rather than change
   the original one. Compare physical observations and classify pass, fail,
   unsupported and inconclusive. These controls do not supersede the forty
   original unsupported results or count as forty new blind designs.
4. **Repair demonstrated shared causes.** Rank the step-3 findings by user
   impact and recurrence. Inventory all affected solver routes and writers.
   Add targeted conservation, retained gas, pressure, energy and observable
   consistency controls, including rejected/unsupported paths. Reported products
   must be available to subsequent measurements. Use narrow source-bound repairs;
   preserve first failures and qualify unresolved domains.
5. **Extend validated closed chemistry.** Start with a bounded supported domain:
   finite rigid gas U/V or finite pressure-controlled H/P, with explicit elemental
   and energy balances and pressure/volume consistency. Review existing
   development implementations by intent and prerequisite before porting them.
   Freeze positive, oxygen-limited, inert and rejected-boundary controls and
   independent reference comparisons. Keep condensed, multiphase and swept
   domains explicitly unsupported until their own balances are validated.
   Internal conservation alone is not empirical calibration.
6. **Explain boundary semantics.** Make help, diagnostics and teaching-facing
   descriptions distinguish finite sealed inventory from an external purge.
   Describe supported boundaries and reported pressure/volume meaning at the
   command that changes them. Cover English/German and CLI/app consistency.
   Preserve the original inconclusive sweep prediction; create separate controls
   for the documented contract. Finish the six-step agenda with evidence links,
   scoped limitations and current integration status.
