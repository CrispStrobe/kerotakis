# Thermal phase and energy contracts, 2026-10-03

This continues the independently designed fifty experiments from October 2.
Twelve targeted followups were frozen before this turn's CLI execution at
`7efc7a41`; these are diagnosis-driven controls, not another blind batch.
The original forecasts and historical captures are unchanged.

## Problems and repairs

Cooling used to report its entire delivered dose as sensible heat even when
water froze. Heat used final-stock Cp times temperature change, which is not
a complete energy partition when phases or chemical inventory change.
Both operators now reconcile a narrow certificate against committed state:
conserved pure-water fusion, or unchanged supported sensible inventory.
Unestablished chemical/phase partitions explicitly carry
`energy_partition_complete=false`, and narration does not assert an exact
chemical residual. Saved events without the new optional field still load;
their partition remains unspecified rather than becoming certified.
Incomplete partitions retain heater source/ceiling, undelivered energy and
pass-cap disclosures at the relevant narration levels.

Deep brine cooling could accept a full energy withdrawal despite stopping at
an activity/freezing boundary. Heat and Cool now refuse the complete operation
at those supported-domain boundaries and restore the full Bench checkpoint.
The independently disclosed absolute-zero cap remains supported.

Chemistry-first thawing could fail on an almost dry, concentrated liquid before
melting diluted it. A bounded retry funds a liquid seed from superheated ice,
then requires complete chemistry, conserved material, integrated thermal energy
closure, supported phase consistency, and unchanged compartment/apparatus and
operator ownership before committing. Failed chemistry mutations and failed
retry events are discarded. This does not extend the brine model's activity
domain or certify arbitrary reaction enthalpies.

The first hosted validation exposed two failing new targets. A brine fixture
had supplied cached speciation that Bench correctly invalidates before each
step; it now recomputes independently counted salt particles before calling
the real phase model, with a small supported-dose control. The thaw retry
also needed to avoid inverting a signed sub-ulp remainder near the melting
anchor. Partial plateaus now select the exact
reference temperature only when the existing strict energy bound already
certifies it. The final energy, phase, conservation and ownership bounds were
not relaxed. The failed receipt and hash-verified stage logs remain in
[validation-first-attempt](evidence/validation-first-attempt/validation.json).

The second run isolated a broader numerical problem: a full small-stock thaw
missed its strict energy certificate by 3.23e-11 J on a 43.23 J budget. The
NASA integral already factored differences of endpoints, but large alternating
liquid-water terms still lost precision internally. It now retains rounded
low parts through multiplication, division, accumulation and logarithm, then
rounds the final integral back to f64. Published coefficients, thermal domain
limits, endpoint holding, and the strict recovery energy bound are unchanged.
Twelve independent 80-digit antiderivative references cover adjacent floats,
the failing full-thaw temperature, interval crossings and wide nitrogen and
calcite intervals. The reference generator preserves exact binary inputs and
records the registry hash. A small cooling control's exact floating equality
was also corrected to its existing numerical budget comparison. The
[second failed receipt](evidence/validation-second-attempt/validation.json)
and logs are retained.

## Baseline evidence

Hosted run [37092946015](https://github.com/CrispStrobe/kerotakis/actions/runs/37092946015)
ran all twelve scripts in text and JSON modes using the previously validated
`04a82d548fd1474968ded25acb9923014f2ae6bc` executable. Its 91 evidence/provenance
checks pass; this establishes capture integrity, not physical correctness.

- T02 cooled 10 mL water by 2 kJ and froze 0.1590233912 mol. It incorrectly
  reported all 2 kJ as sensible heat, although 955.7305812 J funded freezing.
- T09 accepted unsupported deep cooling, then refused the return heating.
- T10 crossed a partial-freezing model boundary while claiming a successful
  50 kJ cooling step. Its reciprocal cycle ended 0.82008 K below its temperature
  after salt addition (23.7147 °C, or 1.2853 K below 25 °C).
- Supported pure-water cycles returned to their starting temperature; their
  erroneous cooling partition still required repair.

Raw captures and the machine-readable baseline are in
[evidence/baseline](evidence/baseline/verification.json).

## Validated result

Source `59c75fcd7ca40f64c93c8112a1912afd6764bb78` passed hosted
[37095659202](https://github.com/CrispStrobe/kerotakis/actions/runs/37095659202):
1,156 Rust tests, the WASM check, the original fifty's CLI replay and its
assessment. Seventeen new Rust regressions cover thermal budgets, transactional
thaw recovery, high-precision integrals and narration/saved-event compatibility;
three existing energy-transfer tests are also newly selected for validation.
The [final receipt](evidence/validation-final/validation.json), stage logs,
source hashes and executable hash are verified. The executable remains on NAS.

Replay [37096304026](https://github.com/CrispStrobe/kerotakis/actions/runs/37096304026)
used that exact executable for all twelve controls in both modes. All
[426 independent checks](evidence/post-repair/verification.json) pass:

| Controls | Observed repaired behavior |
| --- | --- |
| T01–T03, T07–T08 | Pure-water cycles conserve water and return to their starting temperature. Both dose partitions are established and committed liquid/ice changes reconcile with the existing 6010 J/mol fusion datum. |
| T04–T06 | Supported sealed-water cycles complete within the frozen 0.2 K tolerance (largest drift 0.003265 K). Their coupled chemical energy partitions remain explicitly incomplete. |
| T09–T10 | Both modes refuse cooling before publishing a thermal dose or altered state. Complete internal rollback is independently checked by the unit regressions; the stopped CLI alone cannot prove internal rollback. |
| T11–T12 | Modest salt-solution cycles complete, conserve atoms/charge and disclose incomplete energy partitions. |

For T02 the corrected cooling sensible term is 1044.269419 J; freezing accounts
for the remaining 955.730581 J of its 2000 J dose. The same committed phase
account reconciles the return heating. Typed incomplete coverage suppresses
exact chemical-residual claims while retaining heater limits and input budgets.

The hosted `thermal-contracts` selector now runs the independent verifier and
fails on contract regressions, in addition to checking binary provenance and
capturing outputs. Its defaults point to the successfully validated source.
Later evidence/workflow commits do not change the validated engine.

## Remaining model scope

The bounded thaw recovery is tested across eighteen stock/gas/partial/full
combinations with a chemistry fixture whose initial failure mutates state.
Real native chemistry is exercised separately by the sealed CLI controls;
this is not evidence that arbitrary previously frozen brines now have a fully
priced thermodynamic cycle. The brine activity/eutectic domain and missing
chemical reaction heats remain explicit limitations. No new physical parameters
were fitted to the observed results. Compensated NASA evaluation costs more
arithmetic; backend runtime, especially WASM's FMA implementation, has not been
benchmarked here and no speed improvement is claimed for that kernel.

## Resource handling

No local build or CLI was executed for this continuation. Local execution
requires load1 <= 3.5, available memory >= 3200 MiB, free swap >= 600 MiB,
workspace free space >= 2048 MiB and system free space >= 512 MiB. Resource
headroom allowed two agents to work on disjoint source files initially;
heavy validation used hosted runners, and large artifacts went to
`/mnt/storage/kerotakis-maintenance-20261003/`.

Twenty-nine inactive test executable cache files were hash-verified into a
recoverable NAS archive before removal from `target/debug/deps`, recovering
about 1.4 GiB. Active executables, hardlinks and shared libraries were excluded;
the archive stopped when the CPU gate closed. No worktrees were removed.
The [archive manifest](evidence/resources/archived-test-executables.jsonl)
records original paths, archive paths, sizes and hashes. Other work consumed
some reclaimed space; local execution gates were kept unchanged.
