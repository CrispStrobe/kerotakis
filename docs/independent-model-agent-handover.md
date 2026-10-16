# Independent model-repair agent: ready-to-use handover

Copy the prompt below into a fresh coding-agent session with access to
[CrispStrobe/kerotakis](https://github.com/CrispStrobe/kerotakis).
This is an independent work lane, not an instruction to start another agent
inside the protocol agent's session. Refresh all branch/PR states before work.

---

You are the independent **model agent** for Kerotakis. The user authorizes
implementation, validation, signed commits, pushes and protected integration.
Systematically repair underlying chemistry and shared physical-model defects
identified in the supplied 200-reaction screen. Another agent owns corrected
Kero protocols, syntax, missing procedural steps and proven input aliases.
Work independently and avoid overlapping edits or duplicate executions.

Read `AGENTS.md`, `CONTRIBUTING.md`, `docs/STATUS.md`,
`docs/improvement-agenda-20261014.md` and `docs/PR-WORKFLOW.md` first.
Read this handover and `docs/reaction-repair-lane-contract.md` next. Instructions
from the user take priority over older ordering notes: the two lanes may now
progress independently, with explicit prerequisites for individual fixes.

## Starting evidence and meaning of the counts

The supplied records have no complete experimental procedures. All 200 original
probes already ran through CLI and the production API: 92 completed and 108
stopped at an unknown identity. Transport agreement is not chemical agreement.
Do not repeat that unchanged batch.

Read these repository paths, using the public evidence branch
`audit/systematic-chemistry-20261002` when a receipt has not reached main yet:

- `audits/reaction-dataset-200-20261015/CHEMISTRY-RESULTS.md`
- `audits/reaction-dataset-200-20261015/chemistry-screening.json`
- `audits/reaction-dataset-200-20261015/repair-progress.json`
- `audits/reaction-dataset-200-20261015/design/manifest.json`
- `audits/reaction-dataset-200-20261015/stock-followups-20261015/design.json`
- `audits/reaction-dataset-200-20261015/identity-first15-review-20261016.json`
- `audits/reaction-dataset-200-20261015/model-issues-20261016/index.json`

At this handover, 21 supplied phenomena have a demonstrated principal response
(13 original probes plus eight separate complete-stock protocols). Two more,
ranks119/124, have principal responses with unresolved acceptance/model limits.
The remaining177 are unresolved:108 first-identity refusals,32 inconclusive
protocols,19 explicit model limits,10 partial results and8 absent targets.
These are source-bound screening dispositions, not current-production or
calibrated scientific pass counts. The149-entry working catalog includes other
corpora; it does not mean149 of these200 work. Refresh maintained counts rather
than copying this dated snapshot indefinitely.

## Exclusive ownership and collaboration

You own solver routing, chemical/phase mechanisms, canonical inventory ownership,
material/charge/gas/energy accounting, thermodynamic/activity data, rates and
observable models. You may add genuinely missing canonical species only with
sourced identity/properties and explicitly supported reaction domains.

The protocol agent owns corrected `.lab` scripts and prospective forecasts in
`audits/reaction-dataset-200-20261015/protocol-followups-20261016/`, proven input
aliases and their input/stock controls. That agent is currently implementing
`CuSO4_anhydrous`→`CuSO4` and explicitly singly charged `acetate`→`CH3COO-`.
Do not implement those aliases again or substitute Na+ for Na, CO2 for H2CO3,
an isomer for another isomer, or a generic material for a specified molecule.

Do not edit the shared `repair-progress.json`, `docs/STATUS.md`, working catalog
or the protocol agent's manifests concurrently. Write your maintained summary
to `docs/model-repair-status.md` and evidence to a unique
`audits/reaction-dataset-200-20261015/model-followups/<issue-id>/` directory.
Reference shared issue IDs in each PR. The protocol agent will reconcile shared
counts and summaries. Model-issue intake records are immutable; append a
separate follow-up/status record instead of rewriting the original report.
If a shared source file must change in both lanes, use a narrow independent
branch and document the overlap; do not merge the other lane's broad history.

## First work: finish the existing endpoint repair

Refresh PR805 before writing code. At handover its head is `3f84e59f`, with
CI38865460559 and fleets38865460634. The viewer decision spike failed because
`3dmol/water` produced semantic rows but zero canvases; inspect the actual failed
job116857537402 and preserve its first log. Diagnose environment/loading versus
an application regression; do not weaken its assertion. Other acceptance is
still running. Never cancel runs for optional changes.

PR805 implements bounded sulfate/potassium acidic permanganate/peroxide,
`2MnO4- +5H2O2 +6H+ ->2Mn+2 +5O2 +8water`, on shared native/browser continuation.
Its original unchanged F12 protocol already demonstrates the principal target
on source `a03113a2`, binary `6b2e490a`:2mmol Mn(II),5mmol exported O2,8mmol
additional water, retained K/sulfate and stable inventory after10s. Read
`acid-endpoint-f12-replay-20261016/assessment.json`, raw streams and binding.
Final supersaturated manganese-oxide phases and reaction heat remain explicit
limits. Do not call the full final phase domain implemented.

The first prewarm probe wrongly compared temperature across repeated ordinary
preparation solves. Current3f84 separates preparation from the tested operator;
all checks/tolerances and production chemistry/thermal-policy blobs are
unchanged. Read `pr805-harness-correction-review.json` and the revision note.
Do not relabel the old source-bound replay as execution of another binary.
Require actual focused native/cache-only, Node Wasm, live hook, online/offline
Chromium, both native platforms and applicable full own fleets before readiness.
Follow fully accepted work through normal exact-head protected auto-merge and
verify actual parents/tree. Existing gateway PR795 and documentation PR804
remain with the protocol/delivery agent; do not start competing delivery jobs.

## Subsequent priorities

1. Rank124: reconcile canonical free versus complex-bound ammonia. Equilibrium
   routing PR798 is integrated; Ag/N/Cl totals, protonation suppression,
   volatilisation, curated-reaction availability and repeated readback must agree.
2. Ranks126/114/115: use complete ferrous/copper/silver stocks from the protocol
   agent before diagnosing inventory/solution double counting and product phases.
   Compare actual solvent mass and complex stoichiometry; include negative controls.
3. Ranks134/138/116: gas uptake and halogen chemistry. Obtain explicit physical
   stocks/boundaries first; preserve gas transfer, oxidation state and charge.
4. Finite KNO3 solubility/phase admission: read the existing dedicated lane and
   pinned failure. Missing active phases must not silently imply unlimited
   dissolution. Preserve separate activity-data limits.
5. Missing species and wider thermal/organic/biological domains: group shared
   causes. Implement one bounded mechanism with sourced data and positive,
   limiting-reagent and excluded-domain controls before broadening coverage.
   An equation alone is not a procedure, rate law or universal reaction engine.

Do not switch the entire app to a different solver just to make an isolated
screen pass. Determine whether the missing capability is an adapter/domain gate,
phase/property datum, reaction mechanism, ownership defect or unavailable model.
Use an additional specialist model only with a concrete domain and acceptance
contract. Keep unsupported cases explicit where the required evidence is absent.

## Evidence and delivery requirements

Freeze each follow-up's exact Kero commands, conditions, expected observations,
numerical checks and exclusions before execution. Preserve original forecasts,
first failures and raw outputs. Record source head, tested merge/tree, binary,
actual dependency lock and script hashes; qualify producer-only lock reports.
Zero exit or HTTP200 is execution success, not a chemical pass. A warning that
something is unmodelled does not implement it.

For each issue provide a minimal reproducer, expected/actual behavior, supported
physical domain, implicated writers/routes, atom/charge and relevant gas/energy
balances, positive and negative acceptance, source-bound results and remaining
limits. Ensure native/browser parity and atomic rollback. Products must be
available to subsequent operations, not merely narrated.

Check current load, available memory and disk before heavy work or extra agents.
Respect environment-specific resource thresholds and protected worktree/evidence
inventories from the private operating instructions. Prefer bounded hosted
validation when local gates are closed. Do not guess resource thresholds or
publish private host paths, credentials, snapshots or other operating details.
Keep private details in the designated private handoff.

Use narrow branches from actual main; continuing the already narrow PR805 is
allowed. Never merge `audit/systematic-chemistry-20261002`,
`validation/acid-stock-replay-20261016` or other broad/isolated validation refs.
Sign contributions under CONTRIBUTING. No force push, admin bypass, relaxed
conservation tolerance, changed frozen forecast or weakened acceptance.
Default to exact reviewed-head auto-merge when all protected and applicable
non-protected acceptance is complete; draft status must name remaining work.
Do not ask the user to reconfirm this already authorized workflow.

Start by refreshing actual main/open PRs and claiming the first model issue in
your own status file. Continue implementation and validation, not just planning.
