# Ordered improvement agenda — 2026-10-14

The user authorized all six steps below in this order. Steps 1–2 are complete;
step 3 is active and steps 4–6 are queued. This agenda orders the current fresh-experiment gap work;
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
2. **Canonical water alias — complete.** [PR787](https://github.com/CrispStrobe/kerotakis/pull/787)
   merged as `4d3203a3` using auto-merge with exact-head guards. All 25 CI and 33
   own-fleet jobs passed, including 1,638 app tests and both native platforms.
   The actual integrated tree equals validated candidate `d5c6cecf`.
   [Integration evidence](https://github.com/CrispStrobe/kerotakis/blob/audit/systematic-chemistry-20261002/audits/fresh-syntax50-20261013/water-alias-integration.json)
   preserves parents/tree/checks; earlier failures remain intact. Exact `H2O`
   shares canonical `water` identity and finite stock through text/direct/JSON,
   save/replay and Story gates. Only actual dispense/positive-volume titration
   consumes Story usage. The visualization guard remains unchanged. Do not repeat
   completed controls on unchanged source. Follow [auto-merge policy](PR-WORKFLOW.md)
   for subsequent PRs.
3. **Execute and assess the forty naming follow-ups — active.** The
   [forty controls](../audits/fresh-syntax50-20261013/normalized40/design/README.md)
   are dispatched in [hosted run 38697577370](https://github.com/CrispStrobe/kerotakis/actions/runs/38697577370).
   Only one replay job is selected; there is no app rebuild. Completion and physical
   verdicts remain pending. Do not duplicate this execution. The controls
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
