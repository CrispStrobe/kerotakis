# Checkpoint and continuation — 2026-10-24

This is the current continuation entry. Earlier dated pending statements preserve history and do not override the observed results below. Read `docs/PR-WORKFLOW.md` before delivery. No runtime changes or new scientific qualifications were made in this wrap-up.

## What is complete

Candidate [PR859](https://github.com/CrispStrobe/kerotakis/pull/859) remains draft at source `5407ab541517310b155f76efdab1bd835ba9b5a1`, tree `c1077f69c5f6a46136360b2d3078100f520f2a77`. It repairs solvent-input round trips and refreshes measured free-ion coordinates from numerical native rows. The completed hosted focus observes **110 passes and two failures out of112 unique controls**. Formatting, strict affected all-target Clippy and wasm32 compilation pass. All three frozen CLI protocols exit0 without SolverFailed. The1536-state sweep has eight explicit refusal steps and **zero invariant violations**. These are candidate-specific results, not full delivery acceptance.

Actual generated lock SHA-256 is `41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb`. Before/after source pins match. Independent saved review authenticates517 closed members including the manifest,502 non-gzip bodies and14 executable archives' metadata. Compressed-body verification is still required where applicable; only the captured diagnostic CLI body has separately been opened and verified this checkpoint.

Qualification stays **338 unique originals out of468** (342 catalog entries).130 originals remain;63 more earned originals are needed to exceed400. Production acceptance remains the previously delivered release; PR859 has not merged or been deployed.

## Remaining failures and interpretation

1. `aqueous::finite_gas_temperature_serialization_tests::warm_fractional_primary_and_mix_inputs_round_trip_supplied_solutes` panics looking for a missing `Na ` primary-SOLUTION line. Its fixture deposits solid NaCl then calls partition directly; the current partition correctly represents a Halite phase rather than dissolved Na/Cl totals. The water assertion passes; primary solute/MIX/Cl assertions are NOT_REACHED. This is a fixture admission error, not proof that all serialization checks pass.
2. `native_aqueous_overlap::unrelated_salt_and_typed_zero_time_repeats_do_not_replay_the_ionic_row` fails at repeat0 with scalar drift `5.3971844845806275e−9mol/eq`, above the unchanged `1e−10` bound. Earlier no-failure/material/current-native assertions pass; replay and later repeats are NOT_REACHED.

A separately forecast positive `0.000001s` captured-CLI diagnostic identifies the moving coordinate: solute charge rises `5.397189781902734e−9eq`, native CO2 evolution is `1.010299747846e−8mol`, and water/bicarbonate/carbonate/basis stock changes. Free H changes only `−3.12735e−20mol` and free OH `−3.76979e−15mol`. Formal physical charge and gas-adjusted elemental ledgers close within approximately `1e−14`. This is further native aqueous/gas/basis equilibration, not the separate clock's physical charge leak. A positive microsecond wait is a diagnostic, never a substitute for the original typed-zero test.

## Ordered, independently doable lanes

### 1. Freeze the first failures and repair the fixture

Start from actual current main and inspect PR859's exact candidate before changing source. Preserve its112-control outcomes and the independently frozen nearzero forecast/results. The public audit branch contains `audits/aqueous112-checkpoint-20261024/` and `audits/nearzero-coordinate-diagnostic-20261024/`; use their immutable published commit links from the audit PR/body or private evidence index if not present in a main checkout. Do not merge broad audit history.

In `crates/kerotakis-phreeqc/src/aqueous.rs`, correct the named serialization fixture to deposit an explicit complete aqueous Na+/Cl− pair at the same original moles. Retain all water, primary and MIX assertions, names and physical budgets. Do not invent absent solution fields, default missing values to zero or weaken partition behavior. Done: actual native successor reaches and passes both vessels' primary and MIX checks with exact source/lock bindings.

### 2. Settle canonical physical state inside the thermal transaction

Inspect `PhreeqcEquilibrator::equilibrate`, `solve_once` and `recharacterise_canonically` in `aqueous.rs`, `StepStart` in `crates/kerotakis-core/src/vessel.rs`, and the PHREEQC enthalpy helper. Current thermal iteration applies a physical solve and later canonical recharacterization refreshes diagnostics alone. Source review indicates that ignored canonical physical correction becomes visible on the next wait. Confirm this mechanism against the preserved trace before implementing.

Prospective design: a bounded inner physical aqueous/gas/basis iteration at each existing thermal guess, limited to explicitly admitted component ownership. Commit complete native physical answers and diagnostics together. Freeze the new cap and exact convergence predicates before executing a successor. Preserve existing thermal64/phase32 limits, `1e−10mol/eq`, `1e−6K`, `0.001J`, gas/phase floors and native settings. Do not skip repeated operators, pin scalar coordinates, snap temperatures or suppress actual gas to satisfy tests.

Keep the original immutable StepStart as heat origin. Carry cumulative signed external gas from accepted inner passes in a private context; retained finite-headspace gas is internal. A fully priced whole-step heat quote replaces its predecessor rather than being summed. Known dissolution fallback heat needs cumulative physical extents and explicit unpriced coverage; summing whole-step quotes or using only the last pass is wrong. Certify both local transitions and the entire chemistry-owner call against its own actual gas boundary. Upstream gas belongs in the broader heat origin, not counted again as this owner's inlet/outlet.

Converge individual species/phase stocks, water, charge basis and free ions, plus explicitly frozen current-diagnostic and heat criteria. On exhaustion/error, restore contents, pending gas, solution, temperature and caller StepStart atomically. Publish final net physical gas/material events and final diagnostics once; do not publish numerical iterations as repeated reactions. Preserve first-dose fizz, cooling and heat. This is a proposed repair, not an implemented or validated result.

Done: all original112 controls plus prospective controls pass, including the unchanged unrelated-salt/typed-zero repeats; cache/fresh parity; open/sealed and later-dose ownership; whole-call gas/material/charge conservation; fallback-heat ownership; and deliberate exhausted/nonconvergent rollback. Retain the three exact CLI protocols, full1536 sweep and compiler gates. Execute heavy work on admitted hosted resources, not this VPS. Preserve each new failure before changing source again.

### 3. Repair the separate clock CO2 boundary

Use `docs/clock-co2-boundary-lane-20261024.md` and its preserved captured trace. The old-source positive-wait trace has O error `−2.1110684351e−6mol` and formal charge error `+4.2221368710e−6eq` despite exit0. It is distinct from lane2's closing physical ledger.

Carry actual applied clock CO2 as a typed virtual inlet through inner H/O/charge certification and heat ownership. Spend it once, clear the accepted parcel before canonical reentry and emit the actual crossing at the outer owner once. Do not require a preexisting native ambient gas phase or fabricate a finite dose. Cover positive waits, below-floor accumulation, negative overdraw/clamp, primary/cache/MIX/phase wrappers, earned parcels across sealing and failed-trial rollback. Preserve current rate and `1e−6mol` spending floor. Done: exact original ledger failure closes and all boundary controls pass without altered physical budgets or unearned empirical-rate claims.

### 4. Finish delivery before claiming production improvement

Keep PR859 draft until named source review and non-protected acceptance finish. Then obtain exact-current full own-fleet, both native platforms, protected checks and independent artifact review. Follow normal exact-head auto-merge without bypass; verify merged parents/tree. Changed-release product acceptance retains11API/82browser baseline plus four candidate behaviors, **15API/86browser total**, with actual source/native/frontend/lock/asset bindings. Focused tests do not replace deployed behavior. Do not mutate an active acceptance head for optional documentation.

### 5. Raise original experiment coverage

After accepted delivery, use `docs/remaining-original-followups-20261022.md` to select small disjoint groups among the130 remaining originals. Prioritize demonstrated beneficiaries of the generic fixes, then MIX thermal ownership and SP003 precision from their maintained lanes. Retain original forecasts, full protocols, first failures and source/binary/lock bindings. Done per original: its actual named scientific transformation and unchanged ledger/quantitative contract pass on the accepted release, with independently reviewed catalog addition. Unsupported chemistry/refusal is a coverage limit. Do not count duplicate catalog rows or extra executions as new working originals.

### 6. Complete runtime localization

Continue `docs/peroxide-medium-and-i18n-20261019.md`, lane6. Replace app-owned English diagnostic prose with stable structured keys and typed numeric/species/component slots rendered through existing Phrase/catalog paths. Retain raw third-party diagnostics separately. Require actual German/French transcripts for solver errors, capability boundaries, unknown labels and changed causes; preserve numeric events and chemistry admission. Do not hide limitations to pass language checks.

## Operating checkpoint

No new build, hosted experiment, agent or source repair was started during wrap-up. Before continuing, check load, available memory and disk against the ignored operating handoff. One old exact-warm remote diagnostic exhausted its bounded collector deadline without collected output; its last observed status was RUNNING. Preserve its identity, do not duplicate it and do not infer a chemistry verdict. Private evidence locations, controller ownership and resource inventories belong only in the ignored handoff.
