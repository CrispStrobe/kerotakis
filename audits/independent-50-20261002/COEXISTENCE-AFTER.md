# Mixed-gas equilibrium and transport ownership

This pass continues `5b940f89`. The original fifty scripts and predictions
remain unchanged; separate [predictions](coexistence/predictions.json) cover
the newly investigated contracts.

## Changes and independent evidence

- Repeated physical cell IDs now refuse before mutation, both at the bench
  boundary and in the public cell-chain constructor/revalidation path.
  The prior actual CLI accepted a repeated cell and doubled chloride from
  1 mmol to 2 mmol despite a chloride-free inlet. Tests compare complete
  state before/after refusal and retain distinct-cell positive controls.
- The transport receiver preserves cumulative analytical charge, aqueous
  versus liquid phase, and actual outlet temperatures. The previous CLI
  delivered a 346.10 K source outlet at approximately 298.15 K. Independent
  hot-outlet and straddling-temperature tests check physical energy balance.
  The receiver now aggregates compensated amounts by species/phase, incoming
  sensible enthalpy and charge, plus outlet temperature bounds. Memory grows
  with represented species/phases rather than the number of steps. A 1,000-step
  mass/energy regression supplements the tight two-parcel energy check;
  results are compared within conditioned floating-point roundoff, not claimed
  to be bitwise identical to parcel-by-parcel integration. Invalid nonfinite
  aggregates refuse before transport writeback.
- Prepared-object osmosis consumes only liquid bath water, summing all
  matching portions. The recorded pre-fix regression reduced 0.1 mol ice
  to 0.06207276647028655 mol. New controls retain ice and gas, consume the
  entire available duplicated liquid stock once, preserve mass and perform
  no uptake when no liquid bath exists. Osmosis parameters are unchanged.
- A 1e-20 mol independent nitrogen inventory in a represented
  carbonate/lime/CO2/N2 pool previously refused with `NotConverged(144)`.
  Ordinary finite-nitrogen TP/HP coexistence already passed the independent
  baseline and remains a positive control. The new singularity fallback
  derives a feasible inventory line from stoichiometry for exactly two gases,
  two condensed phases and one affine reaction degree. It minimizes full
  ideal-mixture Gibbs energy, then independently verifies occupied/inactive
  dual stability, every original atom budget at its own scale, source interval
  coverage and finite readback. It neither floors the inert stock nor accepts
  a failed Newton iterate. Pressure, inventory scaling, pool order, interior
  and endpoint checks supplement explicit refusal domains.

At 1100 K and 1 bar with 0.2 mol N2, the independent mass-action calculation
gives 0.13083847753953914 mol CO2 and 0.03083847753953914 mol CaO. HP receives
the separately assembled NASA enthalpy and must recover the same temperature
and partition. The pure-CO2 endpoint shortcut is not used for this mixture.

## Validation and reproducibility

Source checkpoint: `570b3022ac1c1ac37210bbfa533c2aa6f8066bf2`, pushed to
`audit/systematic-chemistry-20261002`. The
[validation receipt](coexistence/validation-summary.json) records source and
log hashes, commands' results and the resource policy. Final selected suites
passed **908 Rust tests**: core 725, CEA 119, PHREEQC transport 2 and CLI 62.
The WASM target check passed; this is a compile check, not browser validation.
The earlier 694-test core run predates bounded-memory aggregation and is not
included in the final count.

The actual CLI ran both new experiments in text and JSON modes. Duplicate
cells now refuse in both modes. The hot-outlet receiver reaches
346.0976414736614 K against the source's 346.09764773028746 K, retaining
1 mmol chloride. The [CLI assessment](coexistence/cli-assessment.json) and
raw before/after outputs preserve these observations.

The unchanged original fifty predictions again yielded **49 met, one known
zinc rate gap, zero unexpected failures and zero SolverFailed events**.
All 100 original CLI starts and four explicit-sucrose follow-ups are retained
in [replay evidence](coexistence/replay/). Four original exits are the expected
ambiguous-sugar refusals; all four explicit-sucrose follow-ups succeeded.
The original predictions SHA-256 remains
`0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.
These runs establish behavior, not performance benchmarks.

Two agents handled bounded source work while root ran builds serially with
one Cargo job and one test thread. Builds required load1 at most 3.5,
available RAM at least 3,200 MiB, free swap at least 600 MiB and local disk
at least 2,048 MiB. Continuous monitoring could pause owned processes under
pressure. One baseline build waited about eleven minutes while external load
peaked at 11.89; the agents were idle and no build began until headroom
returned. [Resource events](coexistence/resource-events.json) retain the
gates. No worktrees or unrelated processes were removed in this pass.

Source and executable archives reside under
`/mnt/storage/kerotakis-maintenance-20261002`; their sizes and verified hashes
are in the [archive receipt](coexistence/coexistence-artifact-hashes.json).
The archived executable hash matches the new CLI and original replay runs.
At archive completion (2026-10-02 17:05:07 UTC), load1 was 2.36, available
memory 5.27 GiB, free swap 9.38 GiB and local disk free 4.13 GiB.

## Remaining scope

The new recovery covers a bounded four-species, one-reaction slice. Larger
mixed-gas pools, arbitrary condensed assemblages, absent-gas tangent boundaries
and unsupported source intervals remain outside its proof domain. Finite
multiphase/swept combustion and broader aqueous/interface reconciliation remain
model extensions; zinc/acid kinetics still lacks an approved rate model.
