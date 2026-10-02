# Component constraints and ordered physical transfers

This continues the checked source and replay through `7d4f256b`.
The original 50 scripts and predictions remain unchanged.

## Changes

- Integer atom rows in a represented CEA pool now receive exact checked
  fraction-free rank analysis. A redundant balance equation is removed from
  the Newton variables only after its relation is verified in the input
  budget. Every original element remains checked on readback at its own
  inventory scale. Fractional or overflowing rank calculations retain the
  legacy equations; this does not drop a chemical species or a real balance.
- A missing carrier for a positive element remains `NoSpecies`, preserving
  the existing HP temperature search's source-data ceiling. Incompatible
  supported-pool budgets refuse. The first new run exposed two HP regressions
  when these cases were conflated; the classification fix and regression are
  retained rather than weakening the heat-balance assertions.
- Decanting, mixing, spill creation/recovery, breakage and discard preserve
  positive inventory instead of using a display cutoff as physical cleanup.
  The prior actual CLI erased 2e-18 mol solid Fe on a zero-fraction decant.
  Phase freezing, melting and boiling withdraw their total once across
  matching portions, preserving duplicated-stock conservation and unrelated
  positive traces. Independent operator tests check resolved and unresolved
  moved stock, residual stock, untouched solids, recoverable spills and waste
  events.
- Delta validation follows ordered bulk, bound and electrode inventory,
  including electrode coating effects. Shadows preserve
  separate portions and follow application order instead of trusting an
  initial amount plus a cancelling cumulative sum. A later deposit cannot
  finance an earlier overdraft. Empty and trace inventory receive no fixed absolute credit; conflicting effect
  declarations and ambiguous duplicate layers refuse atomically. Enrichment
  and complete removal/recreation receive positive controls.

## Evidence

Source checkpoint: `81fc26a1b438c5000650d501acbe558f8e5257b2`.
Core validation passed 697 selected tests, CEA passed 111 and CLI contracts
passed 62: 870 Rust tests in this pass. Browser cross-target compilation passed;
no browser runtime or frontend test run is counted here.

The rebuilt CLI ran 100 original text/JSON commands: 96 succeeded and four
formula-ambiguous sugar commands refused as expected. Four separate
explicit-sucrose commands passed. The physical assessment passed 49 of 50
frozen expectations; zinc kinetics remains the sole known gap. There are zero
unexpected mismatches and no `SolverFailed` events in the 50 assessed cases.
Supplemental combustion, rate-boundary and layer-scope checks also pass.

The actual pre-fix zero-fraction decant erased 2e-18 mol Fe; the same input
through the rebuilt CLI retains all 2e-18 mol with no solver failure. Both raw
runs and their execution receipts are preserved in `components/`.

[Validation receipt](components/validation-summary.json) pins source, binary,
source-file hashes, suite logs, superseded failures and resource policy.
[Assessment](components/assessment.txt) and raw CLI output are preserved.
Frozen prediction SHA-256 remains
`0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.

Builds remain single-job and tests serial. The final core run includes 606 unit tests plus discard, interface,
mixing, phase heat capacity, pressure boiling, ordered transactions, spill,
water-state and trace transfer suites. The existing cumulative-overdraw unit
assertion now checks the current 0.04 mol stock and 0.06 mol request; its
physical refusal requirement is unchanged. Large-build startup now requires
load1 <= 3.5 on this four-CPU host,
at least 3200 MiB available RAM, 600 MiB free swap and 2048 MiB free workspace
disk. Own command groups are stopped if RAM/swap or disk headroom becomes
unsafe. Continuous pressure checks pause only the own process group at load1
above 6, available RAM below 1536 MiB, free swap below 512 MiB or free disk
below 1024 MiB; resume uses the startup thresholds. Hard pressure terminates
only that own group. CLI replay checks headroom before every command, before
its subprocess timeout begins. Logs and large artifacts stay on storage.

Resource gating delayed work during host load above 26 and available RAM
below 2 GiB. Per-command replay waiting is recorded separately: its reported
wall times include waiting and are not a performance benchmark. No unrelated
processes, worktrees, branches or active build files were removed in this pass.

Tested source and CLI artifacts were archived on `/mnt/storage`; six changed
source members and the copied binary were hash-verified. Their
[artifact receipt](components/tested-artifact-hashes.json) accompanies the
[resource events](components/resource-events.json). The final archive completed
with about 5.1 GiB available RAM, 9.5 GiB free swap, load1 3.07 and 5.2 GiB
free workspace disk.

## Remaining limits

This closes static exact component redundancy, not every transient Newton
rank loss or generic gas/condensed phase coexistence problem. Broader finite
multiphase/swept combustion and aqueous/interface bookkeeping remain model
extensions. Zinc/acid dissolution still needs an approved rate model.
