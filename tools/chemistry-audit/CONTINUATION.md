# Continuation: computed coverage and preserved experiments

This report supersedes the iteration-3 status in IMPLEMENTATION.md. All work
remains in the isolated `audit/chemistry-experiments-20260906` worktree. Runtime
chemistry does not dispatch on an experiment identifier or expected answer.

**Current status: moving final verification to GitHub CI with user approval.**
The original 50 and next 36 experiment replays, catalog checks,
566 focused tests, licences and full provenance checks passed. During the
subsequent workspace test execution, this worktree's entire `target/` directory
disappeared. See Build coordination below; this is not an all-green workspace
verification claim. On September 7 the user requested continuation; the complete
rebuild is recorded in `workspace-continuation-3.txt`. A third 36-case fleet is
being designed while that gate rebuilds, with execution deferred until the gate
passes. Extra boundary review identified malformed redox-ledger output and
nonfinite distillation-property handling for repair before source freeze.

## Repairs

### September 7 boundary follow-up

Additional review before the third fleet found three generic input-boundary
defects, not failed outputs in the already completed 86 experiments:

- Redox electron-ledger readback accepted missing columns as zero and parsed
  nonfinite values as balances. It now requires a complete, unambiguous,
  finite, nonnegative molality projection and finite positive solvent mass;
  negative oxidation-weighted sums remain valid.
- The generic still could return `Some` with NaN heat from an invalid inactive
  component or infinite heat from very large finite inventory. All property
  models and resulting heat are now validated. `batch-finite-validation.txt`
  preserves both pre-fix failures and **5/5 passing standalone production-code
  tests**, including valid zero-inventory and large-but-representable controls.
- N/S phase transformation dropped inline semicolon thermodynamic parameters.
  It now preserves those suffixes, with regression checks for equilibrium
  constants, enthalpy units and analytical expressions.

`continuation-source-revision-5.json` records the resulting 97 changed files.
The changes were frozen before the rebuilding workspace reached these crates;
their integrated verification is still pending in `workspace-continuation-3.txt`.
No new external data or dependency was imported for these repairs.

The user subsequently approved committing/pushing the audit branch and opening
a draft PR to run GitHub CI. The third local workspace rebuild was stopped
deliberately during dependency compilation, not recorded as a test pass.
Only validated cargo/compiler/provenance processes in this worktree were stopped;
no build artifacts were removed. The CI native Linux job now replays the first
86 experiments and the corrected catalog after its normal regression gates,
then runs cases 87–122 if those preceding steps pass. Raw audit outputs and check
reports are uploaded even on failure. See THIRD-BATCH.md for predeclared checks.

- Native nitrogen and sulfur oxidation-state pools are now independent
  components. A structural database transformation preserves within-state
  acid/base and complex equilibria, phase identities, equilibrium constants,
  and enthalpies. Native speciation—not a repaired analytical readback—now
  honors the declared slow-redox policy. Direct solves and MIX translate names
  at their boundary, return physical names, and fingerprint the prepared data
  in cache keys. Partial selected state totals are not labelled total elements.
- The browser receives the same prepared database from Rust. Separate native
  instances prevent pinned redox definitions from contaminating ordinary solves.
- A reviewed US Bureau of Mines public-domain slice supplies cumulative Cu/NH3
  and Fe/SCN formation constants at 25 °C and zero ionic strength. Ionic
  activities are evaluated by the chosen database model. KSCN is an aqueous
  analytical feed, not a newly parameterized dissolving solid. The import and
  rejected fields are documented in `provenance/usbm-ic9429-complexes-review.md`.
- Unsupported ionic feeds retain their material but no longer inherit a pure
  water pH. Partial mixed-solution readings disclose the missing components.
  Analytical proton/hydroxide coordinates are excluded from that warning.
- Phase diagnostics distinguish unavailable solids, registered-but-excluded
  solids, gas/reference fugacity, and actual temperature-dependent exclusions.
  A positive saturation index alone does not assert nucleation.
- Thiosulfate kinetics now spends two represented acid equivalents per sulfur
  and balances sodium, sulfur dioxide, and water. Every consumed reactant bounds
  the accepted step. Proton activity depletes with acid at a locally frozen
  activity coefficient; a weak-acid/buffer capability guard withholds unsupported
  coupled kinetics. This is generic reaction-network accounting, not a recipe.
- Native MIX now declines coupled fast-redox problems before mutation, allowing
  the ordinary equilibrium owner to solve the merged electron budget. Merely
  suppressing a displayed pe would not have corrected the underlying model.
  The direct root search also retains its interval after a failed native
  evaluation: a failed midpoint has no residual sign. Bounded alternative
  interior probes replace the old invalid interval update.
- Finite reactive-gas uptake is separate from atmospheric reservoirs. HBr uses
  a reviewed dissociative Henry constant and local temperature derivative from
  the CC-BY-4.0 Sander compilation, with explicit concentration-to-molality
  conversion. Gas and already-aqueous analytical feeds have distinct paths;
  neither path creates an infinite atmospheric HBr source.
  Finite headspace inputs use PHREEQC's own gas-constant convention to encode
  supplied moles; using the lab's different physical constant in that conversion
  caused approximately 28 ppm inventory drift. Physical pressure reporting
  retains the lab's gas constant.
- Additional-solvent distillation now plans all supported volatile components
  with a generic multicomponent Rayleigh/ideal-liquid calculation. The existing
  ethanol/water activity-coefficient model remains its own route. Methanol and
  isopropanol use narrowly reviewed public-domain USCG physical properties, not
  quarantined Antoine tables. Unsupported competing volatiles (including
  dissolved ammonia) cause atomic refusal, not silent retention. Structured
  output carries component amounts and the approximation's limits.

## Evidence currently completed

- `iteration-4`: 50/50 original experiments execute, zero solver-failure
  events, **43/43 independent checks pass**. All five unmet iteration-3 checks
  now pass, including native oxidation-state and tetraammine coverage.
- `continuation-aqueous-tests.txt`: 17 targeted Rust tests pass, including
  the 2,560 aqueous variations, 36 mixed feeds (all computed), ligand mass
  action, reaction heat, sealed mass, and titration refinement. Subsequent
  diagnostic/proton-depletion edits require the final workspace rerun.
- `redox-isolation-final.txt`: 30 library and three native proof tests pass;
  all three prepared databases load, original phase headers/constants survive,
  and concentration/pe plus REACTION/MIX tests retain separate redox pools.
- `node --test web/kerotakis-pool.test.mjs`: two prepared-database/pool tests pass.
- `catalog-depletion-1`: 10 executions and 30 independent checks pass.
  Full catalog lint passes **107 entries**, with 234 remaining prose-number
  warnings explicitly not counted as verified claims.
- `next-batch-1`: all **36/36 executions complete**, zero solver failures;
  **118/121 checks pass**. Three genuine failures concern HBr/KOH neutralization
  and methanol/isopropanol distillation. Their repairs above await fresh-binary
  replay; the failed evidence is deliberately retained, not overwritten.
- `check_native_inventory.py`: 457 emitted rows across both batches, **150/150
  native/analytical element comparisons pass**, with no internal namespace leaks.
  This aggregation check complements, rather than replaces, species-state
  and law-of-mass-action checks.

The previous 2,036-test workspace checkpoint predates these changes and is
not the final verification for this revision. The new workspace run and next
36-experiment results will be recorded here after completion.

`next-batch-fixes-tests-1.txt` is a triage run: 562 tests pass and four fail.
Two failures are fixture maintenance (canonical `propanone`, and the additional
reviewed gas phase count); two exposed the real sealed-gas conversion and
redox-root interval defects described above. The original failures remain
preserved. `standalone-batch-2.txt` independently passes both production
distillation-kernel tests; `hbr-native-preflight.txt` passes nine native finite
HBr dose/mass-action checks. These are not substitutes for final integration.

`next-batch-fixes-tests-2.txt` now passes all **566 tests**: 453 core unit,
six distillation integration, 33 native-adapter unit, three ligand-reference,
two redox/MIX boundary, two reactive-gas, two unsupported-ionic, and 65
thermodynamics tests. All four triage failures are repaired without weakening
the physical assertions. The full workspace gate is still separate and pending.

The workspace-rebuilt CLI (SHA256
`99c22ca16ab90c637dc8e5348a48fbb6215196e7936282712cfe4be932566658`)
now replays the original 50 cases in `iteration-5`: **50/50 executions and
43/43 independent checks pass**, with no solver failures. The new 36 cases
in `next-batch-2` likewise give **36/36 executions and 121/121 checks passing**.
All three previously unmet checks now pass against unchanged inputs and
assertions. Its native inventory check passes 62/62 comparisons across 198
rows, with no internal namespace leaks. These results supersede the earlier
replay verdicts, not their preserved historical evidence. The original 50-case
native audit adds 88/88 comparisons across 259 rows: **150/150 comparisons
across 457 rows** in the final two batches.

`catalog-depletion-2` passes all 10 executions and 30 independent checks;
its final full catalog lint passes 107 entries, and the German locale lint
passes with all 18 new fields translated. The 234 unaccounted prose-number
warnings remain disclosed. Its optional native-redox inventory audit has
zero applicable comparisons (118 rows), so that particular report is **no
coverage**, not another passing chemical validation.

Final browser-pool tests (two tests), changed-Rust formatting, `git diff --check`,
and `cargo deny --offline check licenses` pass. The initial licence invocation
placed `--offline` after the subcommand and failed argument parsing; the
corrected invocation and successful result are preserved separately in
`continuation-license-final-2.txt`.
The full `tools/provenance-lint.sh` run also passes, including every Cargo-backed
quarantine/PubChem promotion gate (`continuation-provenance-full.txt`); no gate
was skipped in this final run. Source revision 4 still verifies all 97 changed
runtime/catalog/provenance files against the pre-verification snapshot.

## Preservation and licensing

Two new catalog entries and matching executable lessons preserve endpoint
refinement and reverse equilibrium. Seven existing rate entries were corrected
in English/German to teach finite acid, and the cold bicarbonate entry separates
dissolution-only arithmetic from the full energy balance. See NEXT-BATCH.md.
The export snapshot was reviewed by entry; numerical expectations are not used
as runtime chemistry tables.

No new external package was added. The only new Cargo edge is an existing
workspace data-hashing library. New external constants come from explicitly
reviewed USBM/USCG public-domain fields and the attribution-only CC-BY-4.0
Sander compilation; new stoichiometric identities are original CC0.
No GPL, NC, or other restricted third-party source was newly imported. The
repository's pre-existing AGPL licence was not changed; existing prototype
licence exclusions are not a shipping clearance.

## Deliberate remaining model limits

Passing these tests is not proof of arbitrary chemical accuracy. Complex spectra,
formation enthalpies and temperature derivatives are not supplied by the new
25 °C data. Colours/instrument completeness and off-reference predictions remain
qualified. A contradictory HNCS protonation row was rejected, so strongly acidic
thiocyanate is not an exhaustive speciation model. Carbon redox is outside the
new N/S isolation policy. Physical gas-transfer rates and passive esterification
rates still require licensed, condition-appropriate models and inputs. Coupled
buffered proton-consuming kinetics is withheld, not falsely computed from a
fixed free-proton reservoir. The optical storage format still lacks a general
condition-aware wavelength-table model. No absent number was invented to make
an experiment appear complete.

The additional-solvent model is an ideal-liquid, constant-latent-heat
approximation, not quantitative azeotropic/nonideal separation. Its ±40 K
per-component domain is a declared editorial limit, not an accuracy guarantee.
HBr's off-reference van't Hoff continuation is similarly local, and its
enthalpy is derived from a temperature slope rather than an independent
calorimetric measurement. These limitations accompany the computed results.

## Build coordination

The first continuation workspace build was deliberately interrupted after new
batch failures required further implementation. Only validated compiler/cargo
processes belonging to this worktree were stopped, including an orphaned local
compiler left by a failed cache server. Subsequent builds disable that wrapper
and use one job. Shared-host CPU/memory pressure can make the build very slow;
an incomplete build log is never recorded as a passing verification gate.

`workspace-continuation-2.txt` compiled successfully in 39m40s and began running
tests. Its complete 30-test lesson group passed. During the following CLI tests,
the entire `/mnt/volume1/kero-experiment-audit/target/` directory disappeared:
three metamorphic tests failed to launch `kero` with OS error 2, subsequent test
executables were absent, and doctests could not find their dependency libraries.
Cargo exited 101 and reported 239 failed targets. These are infrastructure
failures, not evidence that those unexecuted chemistry assertions failed.

The deletion's origin is not established. Root performed no cleanup during
this build; the catalog agent also confirmed no cleanup. Read-only verification
afterward confirms all 97 source-file hashes unchanged, `git diff --check`
passing, and all three final replay evidence directories intact. The final CLI
itself is gone and must be rebuilt. Concurrent build-cleanup coordination is
needed before spending another complete rebuild on the remaining gate. Earlier
passing focused/replay/provenance results remain valid evidence but do not
substitute for that gate.
