# Generic chemistry repairs

Worktree: `/mnt/volume1/kero-experiment-audit`, branch
`audit/chemistry-experiments-20260906`, based on `f511d533`.
No other worktree or agent's changes were modified. Implementation remains in
this worktree; these notes are not a claim that every chemical model is complete.

**Historical checkpoint:** the verification and missing-coverage discussion
below describe iteration 3. They are retained to explain the original failures,
not as current status. See [CONTINUATION.md](CONTINUATION.md) for the subsequent
native redox isolation, reviewed complexation data, finite-acid kinetics,
catalog expansion, and new 36-experiment batch.

## Implemented mechanisms

- Reconstruct aqueous H/O inventory from atom conservation, with explicit
  analytical acid/base equivalents and solvent. True free-ion amounts and
  activities remain separate. Apply the same reconstruction to native MIX.
- Retain supplied slow-redox inventories and reject unnameable nontrivial
  species instead of dropping material. Count unresolved recipe mass, prepared
  objects, adsorbates, and typed solid-solution components in their ledgers.
- Preserve native condensed phase identity, including molten combustion
  products. Keep finite-solubility nonionic solids under their dissolution owner.
- Characterize pure water and neutral aqueous mixtures. Preserve thermostat
  temperature; retain small adiabatic temperature changes even below narration
  thresholds. Resolve database aliases for neutral dissolved carbon in both
  speciation and enthalpy accounting.
- Solve explicit reversible organic equilibrium by bounded mass-action root
  finding, including reverse reaction and arbitrary initial product/water.
  The existing ideal-mixture esterification K=4 is not a kinetic yield model.
- Refine pH titration endpoints by reversible trial solves from the same
  pre-dose state. Commit only the accepted fraction, matter, heat, and events.
- Report missing time models on matched passive organic reactions and finite
  gas doses. Correct the explanation of pure-water electrolysis limitations.
- Give output documents their own monotonic sequence number, independently of
  the mutation-log step index, consistently in CLI and MCP.
- Compute observation/instrument absorbance from one molarity-based path. Use
  native free-species concentrations where available, not analytical metal
  totals. Carry the native solvent mass with the species distribution. Report
  unpriced complex spectra and withhold a complete instrument reading for them.
- Prevent dissolved aliases of native gases from being partitioned by a second
  Henry-law owner into a fictitious gas identity.

No production branch selects an audit experiment or its expected output.

## Verification record

Checkpoint source revision: `final-source-revision.json` (57 file hashes, all
rechecked after testing). The final workspace command completed with exit 0:

```sh
cargo test --locked --offline --no-fail-fast --workspace
```

`workspace-final-revision.txt` records **2,036 passed, 0 failed, 2 ignored**
across 254 test-result summaries, including 449 core unit tests. Parameterized
cases inside individual tests are not counted as separate Rust test functions.
Changed Rust files pass formatting checks, and `git diff --check` passes.

The original 50 inputs and original outputs remain untouched. `iteration-1`
is an intermediate replay, not the final implementation's result.

New parameterized regression tests exercise 2,560 native aqueous cases
(8 reagents × 8 volumes × 5 concentrations × 8 temperatures), 1,280 arbitrary
initial esterification mixtures, and 36 titrations (both directions, varied
scale, burette concentration and increment). Their assertions use atom
conservation, mass action, dilute strong-electrolyte pH, and endpoint/stoichiometric
constraints. Passing them is not proof of universal chemical accuracy.

Focused runs have passed these grids, sealed mass checks, solid-solution
conservation, pouring-order reaction heat, food chemistry, and weak-acid tests.
Numbered workspace logs retain intermediate failures and repairs. Iterations 1
and 2 are triage logs with sources changing during the run, not final-revision
gates. `workspace-final.txt` was deliberately interrupted during compilation
after a later correction; use `workspace-final-revision.txt` for the final gate.

The second 50-case replay passes all 37 original checks, with no solver
failures and one missing KSCN input. The extended audit adds explicit execution
and tetraammine-coverage checks and tightens the titration check: **38 pass,
3 remain unmet**. These extra checks prevent a green numerical subset from
being mistaken for complete chemical coverage. Both check reports are retained.

A further 36 mixed-feed routing probes computed 33 states and refused 3 with
byte-equivalent unchanged vessel state. Those refusals expose an unrepresented
S(-2) readback from co-added solid feeds. They are atom-conservation/atomicity
tests, not 36 validated equilibrium predictions. All 12 organic-family tests,
including the acid/carbonate negative control for passive warnings, passed.
The corrected CLI JSON, lesson, and study targets passed in `cli-iteration-4.txt`.

The final-binary replay is `iteration-3`, with its evidence in
`iteration-3-checks.json`: 49/50 executions complete, no solver-failure events,
**38 checks pass and 5 remain unmet**. Two further checks inspect the native
oxidation-state distribution rather than trusting the analytical inventory.
They expose the remaining slow-redox contradiction in cases 41 and 46; the
other three unmet checks are the missing KSCN input and tetraammine coverage.
An exit code of zero is not a claim that an experiment predicts valid chemistry.

Snapshot changes were reviewed: explicit esterification now leaves the
mass-action residual; electrolysis refusal text changed; registry adds six
analytical identities and corrects hydroxide's molar mass to water minus H+.
The balloon lesson now checks PV=nRT and both boundary conditions rather than
pinning two rounded readings from a prior implementation.

## Licence and scientific-data boundaries

No external dependency or thermodynamic/optical constant was added. The new
aqueous identities carry original CC0 stoichiometric derivation records.
Registry re-export preserves those records instead of inventing legacy-import
provenance. `Cargo.lock` is now included; external copyleft is not globally
allowed by cargo-deny. Existing AGPL allowances are restricted to named
workspace packages. The repository itself was already AGPL; it was not relicensed.
Existing prototype exclusions in cargo-deny remain and are not evidence that
those excluded packages have passed a shipping licence review.

`cargo deny --offline check licenses` passed (unused-allowance warnings).
The full provenance lint also passed the existing quarantine-promotion checks;
its final status is recorded in `provenance-check.txt`.

The following chemistry is **not yet quantitatively completed**:

- Copper–ammonia requires a reviewed complex ladder and spectra. The
  [CDC-hosted Miller report](https://stacks.cdc.gov/view/cdc/234547) explicitly
  carries a public-domain label. Its printed p. 12 defines concentration-based
  cumulative constants; their temperature/ionic-medium applicability has not
  been established sufficiently to transplant them into the runtime activity
  model. No numbers from that report were imported.
- Iron–thiocyanate still lacks an integrated identity/speciation/optics slice.
  A bottle-only addition would not implement the requested phenomenon.
- Physical gas-transfer times and uncatalyzed esterification rates are not
  supplied by an equilibrium solver. These require independently sourced rate
  laws and geometry/condition inputs; elapsed time alone is insufficient.
- Co-added mineral feeds need a stronger native treatment of kinetically
  isolated redox states. The adapter now rejects unrepresented sulfur states
  atomically; it does not claim those refused equilibria were computed.
  More seriously, in case 41 the native solution reports S(-2) and N(0), while
  the analytical ledger restores sulfate and ammonia and routing claims the
  slow states were held fixed. Case 46 similarly reports some N(+3).
  Those native pH/species predictions are not validated by the conserved
  ledger. This requires native state isolation, not another readback rename.
- Supersaturation diagnostics still classify all unoffered, non-temperature-
  withheld phases as missing precipitates. Gas phases and registered phases
  excluded for other model reasons need distinct explanations.

NC-restricted data, GPL/LGPL software, guessed spectra, and fitted audit outputs
are not substitutes for those missing inputs.

## Remaining repair order

1. Isolate slow oxidation states inside the native thermodynamic problem,
   including material entering through mineral feeds and MIX. Check native
   species, analytical inventory, and electron/atom balances together. Do not
   treat restored inventory labels as a correction to native equilibrium.
2. Review the copper/ammine and iron/thiocyanate data as complete slices:
   identities, complex ladders, activity conventions, applicable temperature
   and ionic medium, and spectra with explicit reuse rights. Cross-check
   database routes against independent reference calculations before promoting
   those audit cases to teaching examples.
3. Separate missing-phase, withheld-phase, and gas-boundary diagnostics.
4. Add physical rate laws only with the required geometry and condition inputs;
   preserve the explicit lack-of-kinetics boundary until then.

## Preservation assessment

Preserve the entire original audit as reproducible evidence, but not as 50
hard-coded production scenarios. The strongest permanent regression families
are conservation across dissolution and sealed thermal cycles; order/scale
invariance; titration with coarse increments; forward/reverse equilibrium;
native versus analytical optical concentration; and gas-boundary ownership.
These now have law-based tests or are covered by existing suites.

Keep the following small set as the durable index into the audit. Case numbers
refer to the preserved inputs; the regression tests, not a case-number switch,
implement the checks across varying inputs.

| Audit cases | What is worth preserving | Permanent check / existing lesson |
| --- | --- | --- |
| 04–05 | Neutralization order must not change the final heat or material | `reaction_heat.rs` (the separate `neutral-moves` lesson teaches temperature-dependent neutral pH) |
| 06–07 | A buffer needs a matched water control | `lessons/buffer.lab`; aqueous regression suites |
| 19–21, 48 | Open, sealed, and pressure-controlled boundaries have different mass/gas outcomes | `sealed_mass.rs`; CLI gas-law and escaped-mass tests; `lessons/sealed-mass-conservation.lab` |
| 34–35 | Passive waiting is not an equilibrium command; initial products and water change equilibrium yield | `organic_equilibrium_grid.rs` (1,280 mixtures); organic-family boundary tests |
| 42 | A coarse burette increment should not set endpoint accuracy | `titration_refinement.rs` (36 variations); `lessons/titration.lab` |
| 43–45, 49–50 | Analytical bookkeeping must preserve atoms and characterize neutral solutions | `analytical_inventory.rs` (2,560 cases); mass-ledger tests |
| 30, 41, 46 | Analytical metal totals are not free chromophore concentrations | `solution_optics.rs` unit tests; 41/46 remain failing chemical-coverage probes |

For teaching, the best candidates are the paired neutralization orders, buffer
versus water control, sealed versus open fizz, coarse versus refined titration,
and product/water effects on reversible esterification. Preserve controls and
model-boundary explanations with them. Existing matching lessons should be
strengthened rather than duplicated. Copper–ammonia and Fe/SCN should remain
coverage probes until their missing data are genuinely implemented and checked.
