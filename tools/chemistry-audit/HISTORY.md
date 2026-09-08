# Chemistry audit history

This file is the sole narrative record of completed audit work. Raw scripts,
JSON, NDJSON, stderr, hashes and test logs remain in their evidence directories.
Counts describe the recorded revision and declared model domain, not universal
experimental accuracy.

## 2026-09-08 — first catalog promotion

After PR #542 passed and merged, nine distinct concepts were selected from the
240-case source-informed fleet. Three became guided Experiments (K61–K63), five
became runnable Codex cards, and one became a staged-transfer Mission. The GUI
receives ingredients, apparatus, safety/boundary detail, complete German prose
and explicit learning-progress metadata through those existing surfaces. No
parameter-only duplicate or private source mapping entered the public catalog.

## 2026-09-08 — source-informed continuation begins

PR #542 moved frozen cases 195–218 to an authoritative GitHub runner so the
small deployment host does not compile or execute fleets. Its first two runs
exposed workflow-envelope defects before any experiment executed: this checkout
does not commit `Cargo.lock`, and a revision marker made the deliberately fresh
run directory nonempty. Both failures and logs remain in GitHub; both fixes are
infrastructure-only and the frozen scripts/checks are unchanged.

The next programme is fixed and expanded at 240 original, distinct scripts,
IDs 219–458, across ten 24-case scientific families with 97 frozen relations.
`source_fleets.py` validates and records them; a named-relation analyzer has no
expression evaluator and rejects missing, nonfinite, unregistered and mutated
evidence. Ten bounded GitHub shards and a 240-case aggregate gate are wired.
Source/license mappings remain in private storage, never this repository. No
post-218 case has run yet.

The first authoritative sixth-fleet execution retained 78 files and passed
56/66 checks. Ten unmet checks reduced to two generic gaps: amount parsing split
at the `e` in scientific notation, and pure/nonionic water without PHREEQC
speciation produced no conductivity observation. The parser now accepts signed
scientific exponents. The blank is computed at 25 °C from water autoprotolysis
and the existing H⁺/OH⁻ limiting ionic conductivities, with temperature,
dissolved CO₂ and contamination explicitly out of scope. A repaired replay is
pending CI. The failed evidence archive is
`/mnt/storage/kerotakis-archive/chemistry-audit-sixth-195-218-ef4d11b8.tar.gz`
(SHA-256 `95f3a114b5c4f3d0f4642efda0848e856c0e1d481748e6a2e5b66098ce431975`).

## 2026-09-07 — PR #504 rebase onto `139a18d6`

PR #504 was rebased with merge topology preserved from pre-rebase head
`b5894243` onto `origin/main` at `139a18d6`. Backup branch
`backup/audit-504-before-main-sync-20260907-141633` retains the old head.

Four conflicts were reconciled record-by-record. CI retains `--locked` and
`--no-fail-fast`. Element coverage retains main's larger hydrogen/oxygen set.
The registry now contains the union of main's eight aqueous identities and the
audit's three calcium-phosphate identities, 37 NASA heat-capacity curves and
recipe/provenance revisions: 179 identities, 646 phase-property records and no
duplicate record IDs. The golden registry contains the same 179 species and all
37 curves.

The audit evidence manifest is unchanged at 2,260 files. The Codex snapshot
still has 113 unique experiment IDs, 28 models, 189 defined concepts and 167
used concepts. JSON parsing, duplicate-ID checks, `cargo fmt --check` and diff
checks passed. Full compilation and cross-platform validation remain delegated
to GitHub CI after the rebased branch is pushed.

## 2026-09-07 — PR #504 CI repair

Run 34118803477 tested audit head `cc6bcccf` through merge commit `3825363777`.
Five jobs failed for four causes:

- Full preflight found two German summary keys serialized after provenance rather
  than beside summary. Only key position changed; 113 entries stayed identical.
- Native Linux and macOS found registry number-token formatting drift and two
  assertions tied to obsolete diagnostic prose. Golden values were unchanged.
  The organic refusal now checks typed unavailable reaction capacity, vessel,
  tolerance and unchanged inventory. The apatite check now requires the curated
  temperature threshold and explicit non-rate disclaimer.
- German rendering still localized the vessel but its test searched leaf text
  for an obsolete `v1:` prefix. The selector now reads the vessel-specific feed
  row while retaining German-species and decimal-comma assertions.
- Registry export preserved all fields but rebase serialization reordered three
  calcium-phosphate provenance records and a NASA heat-capacity source. Exporter
  order and serde number spellings were restored record-by-record.

Commit `5338e3e6982015aae2ef619162ba922598a458d7` was pushed with an explicit
lease. Semantic checks retain 113 catalog entries, 179 registry identities and
37 heat-capacity polynomial records. Both platforms' generated snapshots agree;
204 focused web tests, six browser-selector controls, Rust formatting, JavaScript
syntax, workflow lint and diff checks passed locally. No local full build or PR
merge was performed. Rebuilt CI remained pending at this checkpoint.

The failed run itself passed six new React diagnostic regressions and the
1,280-mixture organic-equilibrium grid. It stopped native workflow progress
before later CLI fleets, so it did not validate the sixth fleet.

## 2026-09-07 — safe rebase onto current main

The audit branch was rebased with merge topology preserved from base `4762a550`
onto `9084c03e`. Backup branch
`backup/audit-504-before-main-sync-20260907-1139` retains old head `58416bcf`.

Conflicts in registry source, registry golden and element coverage were resolved
by stable identity and field, never wholesale file selection. Eight audit aqueous
identities and three main calcium-phosphate identities coexist. Hydroxide retains
the corrected 17.007 g/mol mass and the new heat-capacity field. Hydrogen and
oxygen coverage counts incorporate both parents. The reconciled head before the
CI repair was `cc6bcccf`.

Preservation checks proved 113 unique catalog IDs and scripts, all audit tests
and evidence, 28 model records, 189 defined concepts, 179 registry identities
and 37 heat-capacity curves survived. Focused web tests passed 204/204.

## 2026-09-07 — catalog and diagnostic additions

Three original lessons were added with English/German prose, guidance, prediction
diagnosis, labels and snapshots:

- `three-components-one-cut`;
- `equal-charge-different-clocks`;
- `grouping-does-not-change-water-heat`.

A 13-entry selected-catalog replay passed 37 checks. Snapshot review accepted
only the three computed additions and no changes to existing lessons.

Curated organic reaction diagnostics were separated into model error, nonfinite
result, absent forward/reverse capacity and supported zero-extent equilibrium.
A valid equilibrium no-op preserves inventory and reports its model boundary;
it is not mislabeled as missing reactants. No experiment-ID dispatch, numerical
law or tolerance was added.

The finite-acid thiosulfate network now consumes represented acid equivalents,
conserves atoms and distinguishes initial rate from accumulated yield. Titration
commits only an accepted refined trial. Declined operations preserve full state.
Native engine construction fails closed instead of silently substituting a
reduced stack. Output sequence numbers are independent from mutation indices.

## Recorded experiment fleets

| Fleet | Cases | Frozen checks | Result |
| --- | ---: | ---: | --- |
| Initial and second | 1–86 | 164 | Passed on their recorded native CLI |
| Third | 87–122 | 174 | First run exposed mutating refused distillation; clean repaired replay passed 174/174 |
| Fourth | 123–158 | 108 | Passed 108/108; native inventory passed 49/49 |
| Fifth | 159–194 | 119 | Passed 119/119; native inventory passed 213/213 comparisons over 226 rows |

Across cases 1–194, 565 frozen checks passed on their recorded revisions after
the preserved third-fleet failure was repaired and replayed. These are model-law,
conservation and matched-control screens, not empirical certification.

The third-fleet failure showed that unsupported ammonia distillation returned a
refusal and then re-equilibrated the source. A typed unchanged-operation result
fixed the generic runner. An intermediate rebuilt replay suffered four timeouts
under heavy host contention; its missing outputs remain failures. A later clean
36-case replay passed all 174 checks and 69 native-inventory comparisons.

The fourth fleet covered multicomponent distillation, unequal neutralization,
precipitation/reversal, sealed gas, asymmetric ester equilibrium and buffer
controls. All 36 processes exited without solver failure. Expectations and
binary identity were frozen before interpretation.

The fifth fleet covered unequal precipitation, zinc hydroxide/acid reversal,
diprotic titration, sealed electrolysis, three-feed fractional mixing and repeated
organic equilibrium. All 36 processes exited successfully. Twelve organic-rich
aqueous-domain warnings were legitimate. Case 192 exposed the misleading
zero-extent diagnostic subsequently fixed generically.

## Earlier implementation and validation

The audit repaired conserved aqueous H/O reconstruction, analytical acid/base
bookkeeping, native MIX reconstruction, unresolved homogeneous-liquid mass, condensed
phase identity, pure-water characterization, small adiabatic temperature state,
bounded reversible ester equilibrium, optical concentration ownership, dissolved
gas alias ownership, and atomic refusal for unrepresentable solver states.

A recorded full workspace gate at its pinned source passed 2,036 Rust tests with
two ignored. Parameterized regressions included 2,560 aqueous cases, 1,280 ester
mixtures and 36 titrations. Later edits require their own CI; the old gate is not
evidence for the current head.

The native CLI was built separately from browser and mobile targets. Its first
recorded checkpoint ran 50 original cases, 36 second-fleet cases, ten selected
catalog entries and the third fleet. Source, lock and binary hashes accompany
each recorder output. A CLI binary never proves the status of source edited after
its build.

Fast-redox probes showed the CLI displacement wrapper did not delegate native
MIX. Direct and mixed scripts therefore reached ordinary equilibrium. The bounded
fix declines native MIX before mutation when coupled redox requires the ordinary
equilibrium owner; direct trait regressions include an uncoupled positive control.

## Snapshot and integration reviews

An earlier combined CI snapshot review found Linux and macOS byte-identical.
Registry drift consisted only of the newly serialized heat-capacity-polynomial
field; codex drift consisted of identified German fields. Both were reconciled
field-by-field against their production sources. No unrelated upstream catalog
file was copied wholesale.

CI run 34133207021 exposed four expectation/integration drifts. The reviewed
element fixture still counted H 179/O 203 after the registry reached H 181/O
206. Two new fixed refusal messages lacked exact German keys. Nine registry
rows lacked the newly serialized empty `heat_capacity_polys` field; Linux and
macOS artifacts agreed field-for-field. Those fixtures and translations were
updated narrowly. The audit's broadened unresolved-solid contribution had also
changed 25 GUI003 Scene mass values; restoring main's established Scene mass
boundary made the numeric five-lesson golden pass without editing either GUI003
golden. The untracked `lessons.actual.json` and temporary Scene diagnostics were
excluded from the repair.

### Run 34139824033 diagnosis

Run 34139824033 tested head `f9ada6f3`. At the last permitted status poll, full
preflight was still running; two jobs had already failed:

- Native macOS passed formatting, Clippy, the Rust test suite and codex coverage,
  then failed `coverage curiosity --check`. Its report contained 497 completed
  prompt results plus three solver failures: 336 computed, 43 curated, 54
  qualitative, 60 boundary and four missing; 82 expectations mismatched and 69
  baseline rows drifted. A local run of the same command reproduced those counts.
- The browser built and ran successfully, then its semantic DOM comparison
  failed. The retained differences are small cabbage-indicator temperature,
  volume and one-colour-channel changes, plus slime becoming colourless/clear.
  Current main's audited GUI003 goldens remain unchanged.

The three solver failures are `aq-097`, `th-002` and `th-003`. Each cools pure
water through freezing; PHREEQC is attempted first and fails at the solution
phase boundary before the independent water phase fallback emits its valid phase
answer. Coverage retains the earlier `SolverFailed`, so the later fallback cannot
make the prompt pass. The narrow production repair is to let an independent-water
phase transition run before aqueous chemistry while retaining explicit chemistry
failure for mixtures and unresolved inventories.

The 69 baseline changes split into 35 computed typed-engine-event to computed
computed-route, 26 qualitative typed-observation to computed computed-route,
three qualitative qualitative-route to computed computed-route, one computed
typed-engine-event to qualitative qualitative-route, the three water solver
failures above and `mat-086` from computed to missing. The common 64 upgrades to
computed are consistent with route attribution being contaminated by the newly
enabled pure-water setup solve; they are not evidence that those questions gained
a computed answer. They require a generic route-attribution repair followed by a
row-wise rerun, not baseline blessing. `bio-062` and every surviving drift must be
reviewed separately after that repair.

Three expected-computed prompts stand aside. `aq-053` has no reviewed aqueous
hypochlorite model; `aq-085` does not execute the repeated-versus-single
extraction comparison its question claims. Their expectations are unsupported.
`mat-086` does compute limewater precipitation, but the coverage missing guard
does not recognize precipitation as an answering event when another
`NotYetModeled` event is present. Its complete event trace must determine whether
that boundary is incidental before changing the classifier.

No baseline or GUI003 golden was regenerated. No production repair was rushed
into this checkpoint. The diagnostic JSON remains in `/tmp` and the pre-existing
untracked `crates/kerotakis-core/tests/golden/lessons.actual.json` remains
unmodified and untracked.

Stale cross-branch catalog counts were reconciled to 113 entries and 167 used
concepts while preserving 189 defined concepts. The audit never modified the
integrator's GUI003, KIDS, readout, discovery or main-synchronization branches.

## Preservation decisions recorded on 2026-09-07

The review retained every raw fleet as regression evidence. Durable production
tests cover
neutralization order, buffer versus blank, open/sealed/regulated gas boundaries,
refined titration, forward/reverse equilibrium, analytical inventory, native
versus optical concentration, fractional transfer and atomic refusal.

Catalog cards were judged most useful when conceptually distinct. The three
entries listed above were retained; repeated equilibrium and minor parameter
variations were classified primarily as regression material.

## Evidence locations

The principal immutable evidence directories are `iteration-3`,
`native-cli-20260907`, `native-cli-refusal-fixed-20260907`,
`native-cli-validation-20260907`, `fourth-batch-1`,
`fifth-batch-1` and `catalog-preserved-1`. Their machine-readable summaries and
hash records supersede the removed per-directory narrative Markdown.

## Research, safety and licensing

External source research was moved to separate private storage. Exact URLs, OCR,
manual text, source mappings and source-specific gaps were never added to this
public branch. Source availability was not treated as reuse permission.

No new GPL, LGPL, non-commercial dependency or restricted data was introduced.
No source procedure, branded name, figure or table was vendored. Original virtual
experiments use existing reviewed runtime inputs and standard-library analysis.
Hazardous source concepts remain non-operational gaps.

## First 219–458 execution review (2026-09-08)

The first GitHub execution preserved its raw evidence before any expectation was
changed. Four families passed outright. The failures separated into contract
mistakes and reusable engine gaps: pressure readings were checked as though kPa
were Pa; two gas relations named a nonexistent final-state field; open
electrolysis was incorrectly required to retain vented H/O; a phase change was
mistaken for KCl consumption; the acetate alias was not accepted; inert sealed
gas was routed to open CEA exhaust; a glucose-water blank lacked an explicit pH
reading; and acetic acid was omitted from mixed-solvent routing.

The next revision corrects those contracts from physical boundaries rather than
observed values. Generic production repairs retain sealed inert gas during
heating, admit only an explicitly listed neutral unspeciated solute to the ideal
25 °C pH blank, and count acetic acid in the mixed-solvent denominator while
still reporting its unmodelled acid chemistry. Regression tests accompany each
engine repair. The analyzer now turns missing events and malformed component
inventories into explicit failed checks. The remaining silver feed-order
discrepancy stays open pending a deterministic equilibrium repair; its frozen
tolerance has not been widened.

The order audit subsequently traced that silver discrepancy to PHREEQC problem
vectors accumulated and emitted in learner feed order. Equivalent equilibrium
inventories are now canonicalized before their input deck is built, and a
regression requires reversed silver/chloride feeds to produce byte-identical
decks. The original 821-file run is archived outside the repository as
`chemistry-audit-source-219-458-36eb425c.tar.gz` (SHA-256
`98cbfe7e1c82c57edd3f5868840df40da1d6e77eecf53a9d27bcfc1912a73808`). The
successful 78-file sixth-fleet subset is independently archived as
`chemistry-audit-sixth-195-218-36eb425c.tar.gz` (SHA-256
`5f19afcf35ffdf8047a43eaaf19429ec2c35fc24baaf315cbe37a6855f177459`).

The second GitHub execution passed eight source families and the repaired
sixth fleet. It exposed two remaining contract errors rather than new hidden
engine failures. At 400 J the sealed gas exceeds the vessel's independently
defined pressure rating and correctly bursts, so that case now requires the
burst boundary while the three sub-rating cases retain the monotonic pressure
law. Reversed silver/chloride additions conserve exactly the same system
ledger but differ at trace scale in phase partition because their sequential
thermal paths differ; the relation now asserts Ag/Cl retention rather than the
false claim that those paths are thermodynamically identical.

The final fleet execution at merge revision `957cfce3` passed all 404 checks:
66 for cases 195–218 and 338 execution/relation checks for all 240 cases
219–458. Its complete 822-file artifact is retained as
`/mnt/storage/kerotakis-archive/chemistry-audit-source-final-195-458-957cfce3.tar.gz`
with SHA-256
`970f6f5e6ea59448ce247f2b0c748bd323eecbdc21fdc71af73521baa0f629f5`.

Known limitations retained throughout the audit include mixed-solvent ionic
activity, physical gas-transfer and uncatalyzed reaction rates, nucleation,
surface and geometry effects, calibrated detection thresholds, complete copper-
ammine and iron-thiocyanate spectra/thermodynamics, and exact household-mixture
composition. Passing equilibrium or inventory checks does not close those gaps.
