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
   now contains the reviewed core and app follow-up at `98051f7b` on integrated
   main `8a6aa68e`: seven core files and four app files. The [guarded application
   receipt](../audits/fresh-syntax50-20261013/water-alias-app-followup-application.json)
   records the normal fast-forward after the previous source's
   [full CI](../audits/fresh-syntax50-20261013/water-alias-core-ci-validation.json)
   and [all 33 own-fleet jobs](../audits/fresh-syntax50-20261013/water-alias-core-fleet-validation.json)
   passed. Its nine native regressions, workspace tests, CODEX and coverage passed;
   those receipts retain their original `e4fb5e33` source binding.
   The first two [CLI controls](../audits/fresh-syntax50-20261013/water-alias-cli/README.md)
   remain failed due to the invalid final command. Two separately frozen
   [corrected controls passed](../audits/fresh-syntax50-20261013/water-alias-cli-followup-outcomes.json)
   with unchanged quantitative forecasts and verified saved stream/report hashes.
   The new candidate's core/CLI/WASM source is unchanged, but those CLI passes
   do not establish app acceptance.
   Fresh CI `38627469049` and own fleet `38627469041` are queued on `98051f7b`.
   Require actual execution of seven new app tests, all-app Vitest, type/browser
   checks and the complete candidate fleet before guarded integration. Preserve
   any first new-source failure before fixing it. Do not duplicate the queued runs.
   Follow the [entry-point review and implementation task](water-alias-task-20261014.md).
   Accept `H2O` as an input alias for `water`, resolve at a
   shared boundary, and persist canonical identities. Inventory additions,
   stock, transfers, extracts, formula lookup and save/replay must not create
   duplicate water species or separate stock balances. Inventory every input
   writer before implementation; preserve canonical `water` behavior and unknown
   key refusals. Verify equivalent quantities, snapshots and saved identities
   across accepted forms. Avoid an unreviewed general formula-alias expansion.
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
