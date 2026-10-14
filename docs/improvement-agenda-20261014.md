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
   is based on actual integrated main `8a6aa68e`; source `e4fb5e33` implements
   the reviewed input boundaries and nine regression tests. Review the
   [preparation receipt](../audits/fresh-syntax50-20261013/water-alias-preparation.json)
   and exact-source CI `38598370658` / own fleet `38598370252`; preserve failures
   before fixes. [All nine regressions passed on macOS](../audits/fresh-syntax50-20261013/water-alias-native-validation.json),
   along with workspace tests, CODEX and coverage (zero unmet requirements,
   runtime failures or drift). The [new CLI archive is verified](../audits/fresh-syntax50-20261013/water-alias-artifact-review.json).
   The [full core CI passed](../audits/fresh-syntax50-20261013/water-alias-core-ci-validation.json);
   its own fleet has 24 passes and eight queued jobs at this checkpoint.
   The first two [mixed-spelling CLI controls](../audits/fresh-syntax50-20261013/water-alias-cli/README.md)
   failed because their final command was invalid; preserve those first outcomes.
   Their saved prefixes confirm canonical inventory/shared exhaustion. Two separately
   frozen [corrected controls](../audits/fresh-syntax50-20261013/water-alias-cli-followup/README.md)
   passed both cases with unchanged quantitative forecasts; archived stream hashes
   and the worker report were verified, then reproduced by the saved-data reader.
   The full result ZIP remains unarchived locally; selected member checks are scoped
   in the [receipt](../audits/fresh-syntax50-20261013/water-alias-cli-followup-outcomes.json).
   A [staged app follow-up](../audits/fresh-syntax50-20261013/water-alias-story-followup.json)
   closes Story access/accounting bypasses and handles actual titration dose events;
   seven new app tests remain unexecuted. Do not merge the known-incomplete core-only
   candidate. Let its current fleet finish, refresh head/base/main, then fast-forward
   PR787 to reviewed staged source `98051f7b` and require fresh exact-source CI,
   including app tests. Complete these acceptances before guarded integration.
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
