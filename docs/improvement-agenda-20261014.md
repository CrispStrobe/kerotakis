# Ordered improvement agenda — 2026-10-14

Current executable scope and delivery prerequisites are in the
[fresh-agent handover](next-agent-handoff.md). Steps1–3 below are historical
completed work; do not repeat them. PR822/823/824/825/827 are integrated. Finish the full
production delivery before new model work. The handover scopes remaining step4 work and queued steps5–6. PR830 cancellation repair is integrated as `0fe97e76`. PR828 phase ownership/energy repair is integrated as `0a5a92f1` after111 focused tests,25 CI and all33 original fleet jobs. Its production artifact/API/browser and original TH016 replay now pass. PR829 partial chromatogram coverage is integrated as `b3c18f5c` after11 focused controls,25 CI checks, both native platforms and all33 logical fleet jobs. PR832 is integrated as `dca5782b` after actual final-site/25 CI acceptance; main publication and live Pages parity remain pending. PR833 addresses finite gas initialization; its first source passes524 PHREEQC native tests, and combined source `e9cc8782` passes Clippy,604 PHREEQC/CEA tests,31 core controls and all33 original logical fleet jobs; it is integrated as `d3883757` after25 CI/both native platforms. Changed-source delivery/replay remains. Source preparation does not replace remaining acceptance or actual changed-source deployment.

The user authorized all six steps below in this order. Steps 1–3 are complete;
step 4 is active and steps 5–6 are queued. This agenda orders the current fresh-experiment gap work;
the annotation and other queued lanes in the [earlier roadmap](ordered-work-roadmap-20261013.md)
resume afterward. Preserve all original predictions, first outcomes, failed CI
logs and versioned source associations. Host-specific operating details belong
in the private handoff. Heavy local work and delegation require resource gates.

1. **Validate and integrate the thermal inventory repair — complete.** PR786
   [integrated as `8a6aa68e`](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/pr786-integration.json)
   after all 25 CI checks and 33 own-fleet jobs passed. The actual main tree
   equals the validated candidate tree. Exact-source native controls verify
   retained inventory below the burst threshold and accounted gas venting when
   mechanical rupture occurs, plus explicit unsupported thermal boundaries and
   existing open/inert/kinetic behavior. Coverage has zero unmet requirements,
   runtime failures or drift. Closed combustion remains an unimplemented domain.
   Preserve the [original replay failures](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/pr786-replay-first-outcomes.json),
   [matched low-pressure passes](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/pr786-low-pressure-outcomes.json),
   [qualified support claims](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/pr786-coverage-correction-preparation.json)
   and [native acceptance](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/pr786-current-native-validation.json).
   Do not repeat this completed lane on unchanged source.
2. **Canonical water alias — complete.** [PR787](https://github.com/CrispStrobe/kerotakis/pull/787)
   merged as `4d3203a3` using auto-merge with exact-head guards. All 25 CI and 33
   own-fleet jobs passed, including 1,638 app tests and both native platforms.
   The actual integrated tree equals validated candidate `d5c6cecf`.
   [Integration evidence](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/water-alias-integration.json)
   preserves parents/tree/checks; earlier failures remain intact. Exact `H2O`
   shares canonical `water` identity and finite stock through text/direct/JSON,
   save/replay and Story gates. Only actual dispense/positive-volume titration
   consumes Story usage. The visualization guard remains unchanged. Do not repeat
   completed controls on unchanged source. Follow [auto-merge policy](PR-WORKFLOW.md)
   for subsequent PRs.
3. **Execute and assess the forty naming follow-ups — complete.** The
   [forty controls](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/normalized40/design/README.md)
   are dispatched in [hosted run 38699207052](https://github.com/CrispStrobe/kerotakis/actions/runs/38699207052).
   The producer reports all 40 executed (33 zero exits, seven nonzero exits); raw
   stream verification and the first physical assessment are complete: **28 pass,
   1 fail, 9 unsupported, 2 inconclusive**. Read the [case-level assessment and
   follow-up lanes](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/normalized40-first-assessment.md). The [raw-integrity receipt](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/normalized40-raw-integrity.json)
   records artifact/source authentication, checksums and the verified review queue. There was no app
   rebuild. Do not duplicate this execution. The controls
   already freeze `H2O`→`water` substitutions and unchanged physical forecasts
   on original main `30fd09eb`/binary `b290`. That execution is complete; preserve it
   separately from any alias/fix replay.
   For actual integrated source, freeze a new replay binding rather than change
   the original one. Compare physical observations and classify pass, fail,
   unsupported and inconclusive. These controls do not supersede the forty
   original unsupported results or count as forty new blind designs.
4. **Repair demonstrated shared causes — active.** The KNO3 cold/hot failure is
   reproduced against pinned merged main in
   [run 38739278904](https://github.com/CrispStrobe/kerotakis/actions/runs/38739278904),
   with a separate source/binary/lock binding and unchanged forecasts: all 20 g
   dissolves at 20.004 °C with no solid. See the [authenticated result](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/audits/fresh-syntax50-20261013/kno3-current-main/assessment.json)
   and [finite-phase/activity-model repair lanes](kno3-finite-solubility-lane.md).
   The missing active mineral phase currently implies unrestricted dissolution;
   nitrate activity parameters are also absent from the concentrated route.
   Rank the step-3 findings by user
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

The reported transient vessel tint and black watercolor are a user-facing repair
priority alongside step 3. Follow [these scoped lanes](colour-and-mixture-followups.md):
validate PR790 first; then reproduce optical density and preserve recipe replay
before changing teaching washes; then investigate duplicate diagnostics and
equilibrium/settling state. This does not change the frozen forty forecasts or
permit scientific verdicts from process exits.

### Current executable checkpoint

Production `0a5a92f1` / `e01b96e0` passes artifact,11 API and37 real-browser acceptance. Original TH016 replay now passes both criteria, whole zinc and independent phase-energy accounting; KN007 retains its earlier separate repaired selection. Catalog215 / fixed468211 needs190 further earned selections for401. Previous205 entries and all first failures remain unchanged.

PR828/829/830/831 are integrated. PR833 `e9cc8782` is integrated as `d3883757`; follow its single changed-source delivery before original GS010 replay. Preserve its combined Clippy,604 PHREEQC/CEA tests,31 core controls and33 original fleet jobs already pass. PR834 prior `6dee9254` passes528 native tests for corrected ammonia gas naming; combined `4c5d591a` now includes the accepted initializer and passes native/core/full33 validation. It repairs invalid native ammonia gas naming exposed by preserved failed tests. Earlier-source528 acceptance is distinct from current combined-source/full-fleet and protected acceptance; original GS005 replay follows accepted delivery. PR832 is integrated with actual final-site profile acceptance; its existing main publication and independent live parity remain pending. Strong-base hydrolysis capacity/proton/counterion proof remains separate. Complete final32 saved criterion reviews without automatically promoting any candidate. See STATUS and the current handoff for exact evidence and doable scopes.

Current accepted release: [artifact/11 API](../audits/target468-20261016/gas-release-api-20261018/README.md) and [37 browser/original GS010](../audits/target468-20261016/gas-release-browser-replay-20261018/README.md), source `d3883757`, binary `14f0cafe`, actual producer lock `66886813`. The unchanged GS010 replay now passes both original criteria and independent whole-material/charge closure to3.172e-14mol/eq. Its actual Pitzer capability and0.931K arm temperature difference are recorded; no empirical salinity calibration is claimed. Catalog215 / fixed468211 preserves all prior214 entries and needs190 further selections for401. Gas delivery/replay is complete; do not restart completed controllers or repeat KN007/TH016/OR006.

Ammonia `4c5d591a` has [independently authenticated Clippy/610 native/31 core/full33 acceptance](../audits/target468-20261016/cpu-combined-ammonia-834-20261018/README.md), including12 pre-authored chemical groups/24 acid arms. Its existing exact-head integration observer awaits current platform/protected checks and both actual native logs; preserve the accepted CPU work. Live Pages publication and actual served interpreter/parity remain separate pending work.

Named hydrolysis `03aab10c` now has a [pushed bounded source repair and frozen baseline/fix validation plan](../audits/target468-20261016/named-alkaline-preparation-20261018/README.md). The test-only baseline contains unchanged production runtime; both control files are byte-identical in the repair. Review the single existing CPU job, actual first failures, changed-source core/native results and independent counterion/proton/atom balances before claiming acceptance. Full-fleet/platform/protected integration and original OR006 delivery replay remain required. No hydrolysis scientific selection or general weak-buffer capacity is claimed.
