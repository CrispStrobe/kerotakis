# Fresh-agent handover — 2026-10-18

This is the executable continuation plan. Read `AGENTS.md`, `CONTRIBUTING.md`,
`docs/STATUS.md`, `docs/improvement-agenda-20261014.md` and
`docs/PR-WORKFLOW.md` first. User instructions override older dated notes.
Do not repeat completed batches or start new broad implementation while wrapping
up the already pushed repairs. All paths below are repository paths or public
GitHub references. Local operation details belong only in an ignored handoff.

## Start here

1. Fetch actual main and inspect PR824–826, their exact heads, current checks and
   active workflows. A queued check is neither a failure nor an acceptance.
2. Consult the ignored local operating handoff if present before touching
   worktrees, build targets, evidence or running controllers. Check load, available
   memory and disk before large work. Keep Hermes workers frozen. Do not start
   extra agents without a new explicit instruction.
3. Continue one narrow accepted lane from actual main. Never merge the public
   `audit/systematic-chemistry-20261002` branch or isolated validation history.
   Retrieve specific evidence at its public commit instead.
4. Preserve first failures, raw responses and original forecasts. Freeze changed
   follow-ups separately with source, tested merge/tree, binary, actual producer
   lock and script hashes. No success claim may derive from mocks, self-assigned
   values, relaxed thresholds, HTTP200 or exit0 alone.

Evidence not present in main is available under public commit
[`9b7e7099`](https://github.com/CrispStrobe/kerotakis/tree/9b7e70998db9a89ca595e2d7f3719c78230f44f5).
Read paths below at that commit where necessary. It is an evidence source, not
an integration branch. Do not copy credentials or host details into public docs.

## Lane 1 — finish pending delivery, first priority

PR822/823 are now integrated as `bc472fb4`/`d6020643`; their source/run rows
below preserve accepted bindings, not pending merge tasks. PR824/825 remain
open; PR825 reruns only failed preflight attempt2 after a runner communication
loss, while PR824 has a new comment-history correction.

Current source heads and existing runs are listed below. Refresh them; never
launch duplicate validation or accept an older head's checks for a new head.

| PR | Exact reviewed head | Existing CI / own fleet / REST run |
| --- | --- | --- |
| 822 | `e2a4ff7054041c21adaaf90435d1a4363362fac6` | `39001207755` / `39001207751` / not separately required by unchanged HTTP layer |
| 823 | `4730f7e689fee8ec3ed0eaa3794c50f18755c8d1` | `38999106064` / `38999106102` / unchanged HTTP layer |
| 824 | `654cf041dcf794d529bd9049c1316b9602e0c7e1` | `39104378850` / `39104378835` / `39104378816` |
| 825 | `bc8607061d8cec84de4035fa2d8b66d71b7b7892` | `39002639113` / `39002639110` / `39002639181` |

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
  policy tests passed; new-head hosted native acceptance is pending.
- **825:** inspect the shared finite-quantity guard, scaled public quantity
  readers and `finite_script_quantities.rs`. Require NaN/infinity/exponent and
  unit-scaling overflow refusals across numeric families, plus ordinary and
  large-finite JSON round trips. Finite input spelling alone does not prove a
  scaled result is finite. Preserve physical-range semantics.

**Acceptance and completion:** inspect focused logs, all applicable CI and all33
own-fleet jobs; REST checks apply where triggered. Preserve and repair genuine
failures in narrow signed commits. Once fully accepted, mark ready and enable
normal auto-merge with the exact reviewed head. No admin bypass or force push.
Refresh base/main between integrations. If a runtime prerequisite or conflict
changes a candidate, use a normal signed merge/rebase consistent with branch
policy and require the changed head's acceptance. Verify actual merged parents
and reconstructed tree; record a receipt and update STATUS. Do not call these
product repairs deployed until lane2 passes. PR826 is the evidence/documentation
PR and follows the narrower protected-check policy.

## Lane 2 — deliver one accepted frontend/native production snapshot

**Inputs:** `docs/simulation-api-lane.md`, `services/rest_api/README.md`,
`tools/build-web.sh`, the existing deployment tooling and authenticated source
receipts. Last verified production is `c4469205` / binary `670bd3d8`; Pages client
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
catalog `audits/working-experiments-20261016.json`, original manifests and saved
assessments. Current catalog200 / fixed468 selected196; goal401 means205 more.
First120 has64 unassessed,14 reviewed holds and42 selected. All668 baseline
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
