# Fresh-agent handover — 2026-10-18

Four remaining saved gas candidates GS002/004/006/008 now earn all eight original bounded criteria after [independent phase/carbon/temperature/EOS review](../audits/target468-20261016/saved-gas-remaining-final-review-20261018/README.md). Current complete catalog237 / unique fixed468233 preserves all previous233 entries;168 more are needed for401. Ten first-screen candidates remain: KN012 and SP001/002/004–010. Quantitative N2 dissolution, nonideal-gas corrections and empirical calibration remain unsupported. No unchanged engine/API experiment was rerun.

Current delivery checkpoint: ammonia PR834 integrated normally as `c19b6e94` after25 CI checks, both actual native-platform log reviews and accepted full33 jobs; [actual parents/reconstructed tree](../audits/target468-20261016/accepted-pr-integration/pr834-integration.json) match. Its single production delivery is issued; artifact/API/browser and original GS005 replay remain unaccepted. Prior accepted production remains `d3883757`. Hydrolysis `bf03c11a` now also passes [all33 original logical jobs/142 steps/29 law reports](../audits/target468-20261016/named-alkaline-fleet-acceptance-20261018/README.md), separately bound to accepted2137 core/608 native controls. It predates ammonia. PR836 current `0c37523e` normally merges actual main `c19b6e94`; validate this combined source and complete current platform/protected acceptance before integration/delivery/original OR006 replay. Do not repeat either completed predecessor fleet or unchanged original experiments. Catalog remains237 total /233 fixed468;168 further earned selections are needed for401. Older source-specific pending notes below retain their historical scope.

This is the executable continuation plan. Read `AGENTS.md`, `CONTRIBUTING.md`,
`docs/STATUS.md`, `docs/improvement-agenda-20261014.md` and
`docs/PR-WORKFLOW.md` first. User instructions override older dated notes.
Do not repeat completed batches or start new broad implementation while wrapping
up the already pushed repairs. All paths below are repository paths or public
GitHub references. Local operation details belong only in an ignored handoff.

## Start here

1. Fetch actual main and inspect integrated PR828/829/830, the production delivery and prepared PR832/833/834, their exact heads, current checks and
   active workflows. A queued check is neither a failure nor an acceptance.
2. Consult the ignored local operating handoff if present before touching
   worktrees, build targets, evidence or running controllers. Check load, available
   memory and disk before large work. Legacy Hermes workers are closed. Do not start
   extra agents without a new explicit instruction.
3. Continue one narrow accepted lane from actual main. Never merge the public
   `audit/systematic-chemistry-20261002` branch or isolated validation history.
   Retrieve specific evidence at its public commit instead.
4. Preserve first failures, raw responses and original forecasts. Freeze changed
   follow-ups separately with source, tested merge/tree, binary, actual producer
   lock and script hashes. No success claim may derive from mocks, self-assigned
   values, relaxed thresholds, HTTP200 or exit0 alone.

Evidence not present in main is available under public commit
[`7c14f396`](https://github.com/CrispStrobe/kerotakis/tree/7c14f3966d40f7c771bc90a31f497a1a72971320).
Read paths below at that commit where necessary. It is an evidence source, not
an integration branch. Do not copy credentials or host details into public docs.

## Lane 1 — finish pending delivery, first priority

Actual main is `d3883757` after runtime PR833; PR828/829/830/831/832 are integrated with verified parents/tree. Delivered `0a5a92f1` / `e01b96e0` now has authenticated artifact,11 API and37 real-browser acceptance. Original TH016 was replayed once and independently passes both criteria and material/phase-energy checks. Catalog237 / fixed468229;168 more earned selections are needed for401.

PR833 exact `e9cc8782` is integrated as `d3883757` after combined-source Clippy,604 PHREEQC/CEA tests,31 core controls, all33 original logical fleet jobs and25 CI/both native platforms. Its single changed-source production dispatch39154455656 has actual artifact/11 API/37 browser acceptance and the separately earned GS010 selection; do not repeat completed delivery. PR834 prior `6dee9254` passes528 native tests for the phase-name repair; current combined `4c5d591a` incorporates accepted main and passes native/core/full33 acceptance. PR832 exact `09aa2b01` is integrated as `dca5782b` after actual packaged-site/25 CI acceptance; its existing main publication and live parity remain pending. See STATUS for evidence. Never duplicate completed jobs or treat a provider COMPLETE/exit0 as physical acceptance.

PR822/823 are now integrated as `bc472fb4`/`d6020643`; their source/run rows
below preserve accepted bindings, not pending merge tasks. PR825 is also
integrated as `050aea7f` after its infrastructure-only retry passed. PR824 has a
comment-history correction integrated as `76fe144f`, with25 CI, REST and all33 original logical fleet jobs satisfied (28 GitHub +5 exact-source local). The transferred GitHub fleet run is cancelled, not a successful workflow. PR827 repairs full
Vercel engine delivery and is integrated as `e63a0824` after25 CI, REST and all33 original fleet logical jobs (11 GitHub +22 same-source local). Both actual merge trees are verified. Production `ed7c452d` now has actual11 API and37 live-browser controls accepted; see the current STATUS and retained release receipt.

Current source heads and existing runs are listed below. Refresh them; never
launch duplicate validation or accept an older head's checks for a new head.

| PR | Exact reviewed head | Existing CI / own fleet / REST run |
| --- | --- | --- |
| 822 | `e2a4ff7054041c21adaaf90435d1a4363362fac6` | `39001207755` / `39001207751` / not separately required by unchanged HTTP layer |
| 823 | `4730f7e689fee8ec3ed0eaa3794c50f18755c8d1` | `38999106064` / `38999106102` / unchanged HTTP layer |
| 824 | `654cf041dcf794d529bd9049c1316b9602e0c7e1` | `39104378850` / `39104378835` / `39104378816` |
| 825 | `bc8607061d8cec84de4035fa2d8b66d71b7b7892` | `39002639113` / `39002639110` / `39002639181` |
| 827 | `5194f961a59b804f18bc4e593e067ab2cf8af0f5` | `39107414016` / `39107555881` / `39107414019` |

- **822:** review `crates/kerotakis-phreeqc/src/aqueous.rs` phase admission,
  `phase_inclusive_totals`, pinning and comparison refusal plus CLI translations.
  Native controls are in `crates/kerotakis-phreeqc/tests/positive_phase_inventory.rs`.
  Retain trace-positive refusal, zero optional candidates and native-only
  coupled/surface exclusions. Corrected diagnostic execution is already complete;
  use `audits/target468-20261016/finite-phase-corrected-replay/` and preserve the
  separate first repaired failure. Do not rerun the unchanged diagnostic.
- **823:** review dissolved-only MIX guard and real native fallback tests in the
  PHREEQC crate. Four new Linux/macOS groups and the ordinary MIX positive control
  are already verified in `audits/target468-20261016/mix-boundary-native-results.json`.
  Preserve no-engine-call/no-mutation refusals for either source or target,
  finite salt/gas fallback and the working solution-only optimization.
- **824:** inspect `crates/kerotakis-core/src/script.rs`,
  `crates/kerotakis-core/tests/terminal_command_grammar.rs` and
  `services/rest_api/{app,test_api}.py`. Preserve all documented verbs, optional
  forms, comments, embedded hashes and `wait 60 seconds`. Check spaced-unit
  conversion and per-command/aggregate API budgets. First broader-head native,
  lesson, Wasm and browser failures are real; do not regenerate goldens to hide
  them. The latest correction preserves full comments in saved command history
  without weakening the existing English-alias assertion. All19 local HTTP
  policy tests passed; both hosted native focused groups and full original fleet acceptance passed. Preserve [integration receipt](../audits/target468-20261016/accepted-pr-integration/pr824-integration.json) and [provider-qualified raw evidence](https://github.com/CrispStrobe/kerotakis/tree/53c3eee5396018927d608a28db35945056ebebfb/audits/target468-20261016/hybrid-own-fleet-20261018/pr824); do not repeat completed cases.
- **825:** inspect the shared finite-quantity guard, scaled public quantity
  readers and `finite_script_quantities.rs`. Require NaN/infinity/exponent and
  unit-scaling overflow refusals across numeric families, plus ordinary and
  large-finite JSON round trips. Finite input spelling alone does not prove a
  scaled result is finite. Preserve physical-range semantics.

- **827:** read `docs/vercel-live-engine-delivery.md`, the Vercel workflow and
  `tools/build-rest-native.sh`. Require the existing native/browser MY-BASIC
  profile, both live engine assets, complete frontend archive and actual producer
  Cargo.lock bytes. Preserve the first404 observations. After normal acceptance
  and integration, deploy actual main and verify the delivered assets plus a real
  uncached browser/native solution; cache-only output is not full delivery.

**Acceptance and completion:** inspect focused logs, all applicable CI and all33
own-fleet jobs; REST checks apply where triggered. Preserve and repair genuine
failures in narrow signed commits. Once fully accepted, mark ready and enable
normal auto-merge with the exact reviewed head. No admin bypass or force push.
Refresh base/main between integrations. If a runtime prerequisite or conflict
changes a candidate, use a normal signed merge/rebase consistent with branch
policy and require the changed head's acceptance. Verify actual merged parents
and reconstructed tree; record a receipt and update STATUS. Do not call these
product repairs deployed until lane2 passes. PR826 is the evidence/documentation
PR and follows the narrower protected-check policy. PR824/827 integration is complete; preserve their receipts and cancelled transferred GitHub workflows truthfully. Start with actual main and the production lane rather than repeating source acceptance.

## Lane 2 — deliver one accepted frontend/native production snapshot

**Inputs:** `docs/simulation-api-lane.md`, `services/rest_api/README.md`,
`tools/build-web.sh`, the existing deployment tooling and authenticated source
receipts. Last verified production is `ed7c452d` / binary `f9358b91` after actual API/browser acceptance; Pages client
publication is `fa5a75fa`. Preview/sandbox changes are not full production delivery.

**Work:** after lane1 integrations, choose one accepted source and authenticate
its complete frontend/Wasm/native build, actual producer lock and runtime-library
hashes. Reuse an appropriate current artifact where available. Preserve existing
Vercel project, authentication, shared quotas, network-denied bounded execution,
CORS and deployment protection. Keep credentials in configuration only.

**Done:** actual served health/version, frontend assets, native identity and Pages
client identity agree with the declared deployment. Run only changed-source
smokes: finite inventory, command grammar/duration caps, auth/quotas/isolation,
CLI/API parity and online/offline browser behavior. Freeze exact requests before
issuing them and preserve first outcomes; no ambiguous retries. A health200 or
successful deploy command alone is insufficient. Record production acceptance
and update STATUS without incrementing scientific counts.

## Lane 3 — assess saved forecasts and reconcile the working catalog

**Inputs:** `audits/target468-20261016/{README.md,scoreboard.json}`, working
catalog `audits/working-experiments-20261018-reviewed-gas-final.json`, original manifests and saved
assessments. Current catalog237 / fixed468 selected233; goal401 means168 more. KN-007 is a separately qualified repaired-source replay; older forecasts and outcomes are unchanged.
The original first120 had64 cases in this review backlog; three now earned final saved-gas selection. The earlier42 selected and14 reviewed-hold bindings remain unchanged. The [saved64 first screen](saved64-review-lanes-20261018.md)
originally recorded44 candidates and20 named holds;14 candidates still need final review and the20 original holds remain historical. KN-007 has a separately earned repaired-source selection. All668 baseline
executions are already complete. The separate curriculum200 is not part of468.

**Work:** pick a small named group of unassessed IDs, read original prospective
criteria and actual saved outputs, and independently check observables plus
relevant material/charge/gas/energy ledgers. Append assessments and separate
selection receipts. Deduplicate original IDs across corrected protocols. Never
turn unsupported chemistry into a pass or overwrite original outcomes.

**Done:** each selected ID has a reproducible physical argument tied to its exact
source and script; held IDs have named missing observations or model gaps.
Counts reconcile with immutable previous selections. Use a new frozen protocol
or changed-source replay only when saved evidence cannot answer a specific
question. Do not run another whole668 merely to increase exit-success counts.

## Lane 4 — finite-phase ownership and nitrate solubility

**Inputs:** `docs/kno3-finite-solubility-lane.md`, PR822 evidence and
`crates/kerotakis-phreeqc/src/{aqueous,derived}.rs`.

**Work:** after822, inventory all remaining positive solids freed analytically
without an admitted mineral/finite-solubility model across direct, comparison,
MIX and fallback routes. Classify unsupported domains explicitly. Test hydrated
phases, alternate-name collisions, reverse paths, repeated solves, transfer and
cache replay with complete relevant ledgers. Do not hide missing phase data as
unrestricted physical dissolution.

KNO3 remains a separate demonstrated failure:20g in50g water dissolves entirely
at the cold point. Active WATEQ/MINTEQ/Pitzer lack the admitted Niter domain;
inactive LLNL data is not automatically compatible. Follow the dedicated lane's
ordered missing-phase, standard-state/temperature-law and nitrate-activity
reviews. A bounded pure-water curve is an alternative only with reviewed data,
license, units, uncertainty and explicit common-ion/mixed-solvent exclusions.

**Done:** source-bound cold/hot, undersaturated/saturated, reverse-path and
conservation controls pass within the declared domain against independent
references. Phase admission alone is not concentrated-solution validation.
A refusal can close an honesty gap without satisfying the original forecast.

## Lane 5 — authoritative free and bound ligand inventory

**Inputs:** model intake `MODEL-002` (rank124),
`crates/kerotakis-phreeqc/src/aqueous.rs` and core
`{bench,volatility,curated}.rs`. PR798 routing is already integrated.

**Work:** map every writer/consumer of analytical ammonia, free NH3/NH4 and
complex-bound ligand. Define full-precision ownership surviving reconstruction,
solution invalidation, transfer, save/replay and cache/native/browser paths.
Public `SolutionInfo.species` values are rounded to four significant figures;
never use them as authoritative reaction or volatilization amounts. Preserve
analytical totalN while Henry transfer and curated extents use available pools.

**Done:** Ag/N/Cl balances, protonation, no double counting, volatilization,
limiting ligand, repeated readback and transfer agree. Include rejected domains
and atomic failure controls. No ad hoc rounded-species patch or blanket warning
counts as model closure. Then reassess original124 separately.

## Lane 6 — remaining stock/protocol and model intake

**Inputs:** `audits/reaction-dataset-200-20261015/repair-progress.json`,
`CHEMISTRY-RESULTS.md`, `model-issues-20261016/index.json` and
`docs/reaction-repair-lane-contract.md`. Original200 have no complete procedures;
92 completed contact probes are not92 scientific passes.

**Work:** address one issue with an explicit supported stock/procedure first.
MODEL-001's bounded acidic endpoint is integrated; retain its uncalibrated heat
and excluded competing Mn-oxide phases rather than reimplementing it. MODEL-003
ranks126/114/115 need complete ferrous/copper/silver stocks and product/solvent
ledgers. MODEL-004 ranks134/138/116 needs physical gas/halogen boundaries.
MODEL-005–009 cover oxygen residence, hydroxide selectivity, reactive transport,
thermal iteration and optical/CEA diagnostics; refresh accepted repairs before
coding, since some prerequisites are integrated. Hydroxide selectivity itself remains
unaccepted: PR813 only exposes trusted readback. For MODEL-006, implement the
explicit simple-hydroxide domain using a fresh complete answer while the solver
is in scope. Preserve sodium/bromide and carbonate exclusions; test deficient or
neutralized base, failed-readback rollback, unknown heat, native and Wasm. Do not
trust stale cached solutions or fall through after a failed readback. See lanes7–8
for observables.

**Done:** each follow-up has an original ID, unchanged first result, frozen
complete Kero script, source binding, expected/actual observations, positive and
negative domain controls and a separate disposition. Aliases require proven
identity, finite-stock behavior and no ion/hydrate/isomer substitution. Missing
models remain explicit until sourced mechanisms/data and acceptance exist.

Also audit sodium-hypochlorite presentation: formula-equivalent `NaOCl` input
must be distinguishable from the separate5% bleach recipe. Inspect registry
identity, density/strength labels, autocomplete and amount narration. Do not
silently reinterpret saved `NaOCl 100mL` as diluted commercial bleach. Deliver
paired canonical/recipe input and replay controls plus clear user-facing labels.

## Lane 7 — transport, thermal and particle follow-through

**Inputs:** integrated PR809/812/813 receipts and saved transport hot/cold
failures under `audits/new-experiments-201-20261016/transport-effluent-followup/`.
Inspect core `bench.rs`, `transport.rs`, `selectivity.rs` and ordinary fallback.

**Work:** distinguish completed parcel/receiver fixes from still-missing hydraulic
or free-ligand domains. Check whether the separately bound repaired hot/cold
replay has been collected before issuing anything. Freeze a new replay only if
that acceptance is actually missing. Follow pour/decant with settled solid,
nontransferred gas, nonempty receivers and fractional/zero flow; verify aliased
roles refuse before mutation. Review unknown reaction heat and thermal iteration
without presenting material conservation as energy closure. Historical-lot
false settling is repaired; broader grain-size/precipitate metadata is separate.

For repeated dissolve/precipitate output, compare complete before/after ledgers
at unchanged conditions. Diagnose numerical reconciliation, phase ownership and
genuine reaction separately; the reported magnesium-hydroxide/settling sequence
is not proved repaired by the ferrous-only PR822 control. Preserve actual events
and first logs rather than hiding oscillation with narration deduplication.

**Done:** current-source observations meet frozen hot/cold/material/charge and
relevant energy checks with rollback and native/browser parity. Publish remaining
hydraulic, particle and thermal domain boundaries separately.

## Lane 8 — optical provenance, rendering and diagnostic correctness

**Inputs:** `docs/colour-and-mixture-followups.md`, integrated PR790–792/810,
MODEL-009 and saved NaBr optical/CEA observations.

**Work:** preserve accepted liquid clipping/transient tint, teaching-color and
warning-deduplication fixes. Distinguish teaching recipes from calibrated
absorbance. Reproduce only a still-open, source-bound rendering defect before
changing geometry/animation. Obtain compatible primary provenance before adding
NaBr hue. For CEA, inspect the actual element-filtered candidate pool and first
failed matrix; distinguish rank loss from missing data. No invented species,
phase deduplication, relaxed convergence or suppressed failure.

**Done:** targeted actual-browser controls preserve liquid-only geometry and
viewer lifecycle; optical claims have sourced domains and independent references.
CEA controls require real convergence or an explicit supported-domain refusal
with rollback, never a fabricated successful prediction.

## Lanes 9–12 — resume the approved agenda after shared defects

9. **Closed gas:** per agenda step5, choose finite rigid U/V or finite-pressure
   H/P first. Freeze oxygen-limited, positive, inert and rejected-boundary controls;
   implement explicit elemental/energy and pressure/volume balances with
   independent references. Keep condensed/multiphase/swept chemistry excluded
   until separately validated. Do not begin with universal combustion.
10. **Boundary explanations:** step6; make CLI/app and English/German help describe
    sealed inventory versus external purge at the changing command. Verify reported
    pressure/volume meanings and unsupported boundaries. Preserve original sweep
    outcomes; use separate frozen controls.
11. **Annotation and historical coverage:** resume
    `docs/ordered-work-roadmap-20261013.md` lanes3–4 and its linked owner plan.
    Freeze proposed annotation controls, retain unknown/duplicate-key policy,
    then port one prerequisite-complete historical behavior group per narrow PR.
    Refresh source presence first; missing files alone do not prove defects.
12. **Safety and empirical accuracy:** roadmap lanes5–6; scope intermediate-product
    observation and multiple locations with supported routes and benign/priority/
    discarded-trial controls. Then choose one authoritative accounting/data domain,
    migrate all writers and validate against independently reviewed licensed
    primary references with uncertainty. Internal conservation is not calibration.

## Completion and handoff discipline

For each lane, deliver reviewed narrow code/data/docs, meaningful exact-source
controls, applicable native/browser/REST/full-fleet acceptance, signed commits,
protected exact-head integration and an actual merge-tree receipt. Update STATUS
and this handover after a material change. Keep pending, implemented, accepted,
integrated and deployed states distinct. Preserve dated evidence rather than
rewriting it. Public prose must contain no host paths, private snapshots,
credentials, hostnames, session inventories or resource inventories.

## Current delivery and repair checklist

1. **Production/phase — completed:** delivered `0a5a92f1`, binary `e01b96e0`, producer lock `66886813`; full artifact,11 API and37 real-browser controls pass. Only original TH016 then ran once on changed source. Independent45-digit energy and atom checks plus both original criteria pass. Preserve the original all-liquid failure, first binding preflight failure and separate corrected binding. Do not repeat TH016 or the other three unchanged diagnostics.
2. **Finite initializer — delivered and selected:** PR833 source `d3883757` has authenticated artifact/11 API/37 browser acceptance and a single original GS010 replay with both criteria plus whole C/H/O/N/Na/Cl/charge closure. Current binary `14f0cafe`, actual lock `66886813`. Preserve the original imbalance and actual temperature/model boundaries. All old gas release/browser/review controllers are complete; do not restart them.
3. **Ammonia — CPU accepted; platform checks pending:** prior `6dee9254` passes Clippy/all528 native tests and independent12-group formula/charge review. Current `4c5d591a` normally incorporates accepted main/initializer. Current CPU acceptance authenticates Clippy/610 native/31 core/all33 original fleet jobs against exact source/tree, compiler/actual lock/binary/runtime and all142 frozen commands; separate12-group atom/charge review also passes. Finish protected/platform CI39154263763 and actual native log review without repeating accepted CPU work. The original queued fleet39154263789 was cancelled with no assigned runner or executed steps, so no case was repeated. Preserve first queue-metadata reader stops and all native failures. Repair real failures without weakening original assertions, integrate normally after all acceptance, then deliver and separately replay original GS005. Minteq finite NH3(g), complex-bound ligand reconciliation and general thermal domains remain explicitly limited.
4. **Pages — separate delivery:** PR832 `09aa2b01` is integrated as `dca5782b` after25 CI, actual final-site MY-BASIC probe and all browser gates, with authenticated packaged artifact. Follow existing main publication39152732716 (subsequent main CI39165177707 is already pending), then complete independent live Pages arithmetic/uncached-solution parity. Published `76fe144` demonstrably refuses USER_PUNCH; Vercel acceptance does not close Pages chemistry.
5. **Hydrolysis — pushed source, acceptance pending:** `03aab10c` implements the bounded conserved reserve and atomic applier. Review the single frozen baseline/fix CPU validation and actual first failures; both original control files must stay byte-identical. Independently check source/tree/worker/actual lock, every result, Na/K/acetate ownership and charge. Preserve legacy literal-NaOH semantics and explicit unknown-reservoir refusals. Repair genuine failures, then open a narrow draft PR, complete applicable full-fleet/platform/protected acceptance and accepted-source OR006 replay. Neither the rejected free-OH shortcut nor a positive-basis shortcut is admitted.
6. **Saved review/catalog:** complete catalog237 includes233 unique fixed468 selections; all previous215 records remain unchanged. TH016 is one separately earned selection;10 other saved candidates need final criterion/domain review. Twenty original holds remain historical. Achieving401 requires168 further earned selections, not automatic promotion from HTTP200 or engineering tests.
7. **Documentation delivery:** PR831 is integrated as `ac1f9993`. Refresh main with the latest maintained summaries in a narrow MD-only PR, pinning evidence to immutable public audit commits when files are absent from main. Never merge the broad audit or isolated browser validation branches. Keep all host paths and private operations in the ignored handoff.

Current evidence: [phase artifact/API](../audits/target468-20261016/phase-release-api-20261018/README.md), [browser/TH016 and selection](../audits/target468-20261016/phase-release-browser-thermal-20261018/README.md), [PR829 accepted fleet](../audits/target468-20261016/hybrid-own-fleet-20261018/pr829/README.md), [PR833 combined acceptance](../audits/target468-20261016/cpu-combined-gas-833-20261018/README.md), [first ammonia failures](../audits/target468-20261016/ammonia-native-first-failure-20261018/README.md), and [actual invalid-phase diagnosis](../audits/target468-20261016/ammonia-native-admission-trace-20261018/README.md).

Current accepted release: [artifact/11 API](../audits/target468-20261016/gas-release-api-20261018/README.md) and [37 browser/original GS010](../audits/target468-20261016/gas-release-browser-replay-20261018/README.md), source `d3883757`, binary `14f0cafe`, actual producer lock `66886813`. The unchanged GS010 replay now passes both original criteria and independent whole-material/charge closure to3.168e-14mol/eq. Its actual Pitzer capability and0.931K arm temperature difference are recorded; no empirical salinity calibration is claimed. Catalog237 / fixed468229 preserves all prior223 entries and needs168 further selections for401. Gas delivery/replay is complete; do not restart completed controllers or repeat KN007/TH016/OR006.

Ammonia `4c5d591a` has [independently authenticated Clippy/610 native/31 core/full33 acceptance](../audits/target468-20261016/cpu-combined-ammonia-834-20261018/README.md), including12 pre-authored chemical groups/24 acid arms. Its existing exact-head integration observer awaits current platform/protected checks and both actual native logs; preserve the accepted CPU work. Live Pages publication and actual served interpreter/parity remain separate pending work.

Named hydrolysis [draft PR836](https://github.com/CrispStrobe/kerotakis/pull/836), head `bf03c11a`, follows an [authenticated first baseline/source execution](../audits/target468-20261016/named-alkaline-first-failure-20261018/README.md). Unchanged runtime fails six core and two native controls. Repair `03aab10c` passes Clippy and all nine new core controls; the full core run passes2136 and fails one composed-locale test because French lacks the new refusal key. It stops before changed-source native execution. The correction completes French/English messages without changing either chemical control file. Corrected source `bf03c11a` now passes [Clippy/2137 core/608 native and independent nine-arm atom/charge/proton/counterion review](../audits/target468-20261016/named-alkaline-native-acceptance-20261018/README.md), with zero ignored. Maximum native material/charge residual6.60842142e-12mol/eq. Its [single original33-job CPU fleet execution](../audits/target468-20261016/named-alkaline-fleet-preparation-20261018/README.md) follows normal cancellation of a completely unassigned/zero-step GitHub producer; no protected context is replaced. Do not repeat completed focused controls. Applicable full33, current prerequisites, platform/protected integration and original OR006 delivery replay remain required. No hydrolysis scientific selection or general weak-buffer capacity is claimed.

Eight saved named-organic protocols OR001–005 and OR010–012 now satisfy all16 original bounded criteria with [independent45-digit atom/charge and actual observation review](../audits/target468-20261016/saved-organic-final-review-20261018/README.md). Maximum reaction ledger residual3.91872206643e-12mol/eq; source/producer-lock qualifications and mixed-solvent warnings remain explicit. The complete catalog is237 / fixed468 selected233, preserving all prior223 entries;168 more are needed for401. Ten first-screen candidates remain for final review. OR006–009 are still excluded pending separate hydrolysis source/native/fleet/delivery evidence. No unchanged experiment was rerun.


Ten saved thermal protocols TH001–006 and TH011–014 satisfy all20 original bounded criteria after [source-bound actual phase/export/energy review](../audits/target468-20261016/saved-thermal-final-review-20261018/README.md). The catalog is237 / fixed468 selected233, preserving previous223 entries;168 more are needed for401. Ten first-screen candidates remain. TH012 four freezing messages and a compensating melting message close the signed net owned solid gain within6e-17mol; the initial unreviewed concern is preserved and no current-source defect is inferred from repetition alone. No current deployment certification or empirical calibration is claimed.


Documentation PR835 `173be8a2` merged normally as `2eab41c4` after all25 applicable checks; [actual parents/tree receipt](../audits/target468-20261016/accepted-pr-integration/pr835-integration.json) verifies the four-file-only scope. Runtime/accepted production remains `d3883757`. Deliver the current summaries through a new narrow MD-only branch from actual main, pinning audit evidence to an immutable public commit. Never merge broad audit history.


The fresh four-file documentation update is [PR837](https://github.com/CrispStrobe/kerotakis/pull/837), exact reviewed head `841593be`, ready with normal auto-merge enabled. It starts from actual main `2eab41c4`, pins61 authenticated public audit links to `e83413d3` and changes no runtime/workflow/scientific contract. Observe protected CI and verify actual merged parents/tree afterward; do not churn its accepted head for optional snapshot edits.

After current hydrolysis delivery/OR006, use [lane K](saved64-review-lanes-20261018.md#k-extend-proved-reserve-to-a-bounded-carbonate-pool-before-sealed-staged-synthesis) for the original sealed MO003 carbonate-reserve blocker. [Four frozen original follow-ups](../audits/target468-20261016/next-shared-repair-followups-20261018/README.md) separately scope MO006/MO011 dose and staged-product checks plus AB031 nitrogen export. These are unissued contracts, not automatic beneficiaries or extra passes.
