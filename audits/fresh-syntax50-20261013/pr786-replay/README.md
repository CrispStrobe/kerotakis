# PR786 CLI acceptance controls

Seven source-informed controls are frozen before execution. The first two copy
the original failed magnesium/carbon scripts byte for byte. Open-air magnesium,
inert gas heating, warm methane, finite pressure control and an external sweep
check the supported behavior and explicit remaining boundaries.

Read [the expectations](expectations.json) and [the frozen protocol](protocol.json).
Current state: **0/7 executed**. The own CLI build is pending. Before execution,
verify its actual checkout and artifact identity, then freeze a separate runtime
binding with ZIP, binary and captured lock digests. Existing production/frozen
experiment files remain untouched. Every CLI run requires current resource gates.

Compare each ignition row with its preceding bench snapshot: retain total fuel
identity and each owned gas amount; allow explicit heating and phase changes.
Require the typed refusal and reject a thermal-equilibrium success or emitted
exhaust at unsupported boundaries. Positive open combustion, pressure/temperature
consistency and below-autoignition diagnostics are separate acceptance controls.
Keep unsupported chemistry separate from the original physical predictions.

If the warm-methane heat input misses its stated temperature interval, preserve
that inconclusive design result and freeze a corrected control separately. Do not
change the expectations after execution. These controls qualify the narrow
inventory-preservation repair; they do not validate closed U/V or finite H/P
combustion. Integration also requires native and applicable repository CI gates.

## Run and assess

[run.py](run.py) requires an accepted own-artifact receipt and a separately frozen
[runtime binding](runtime-binding-fields.json), the verified binary and captured
lock, a dedicated output directory and configured fast/storage locations.
It verifies the source, tested merge parents, own run/build job, artifact identity,
protocol and helper digests before execution. Resource gates are journaled before
refusal, cases run sequentially, and a resumed run skips hash-verified completed
cases. Incomplete streams, timeouts and completed batches cannot be overwritten.
Each process uses the relative script path and default Kero configuration.

[analyse.py](analyse.py) consumes only saved streams and their producer records.
It verifies the runtime/artifact association and frozen inputs, rejects malformed
or nonfinite JSON, checks inventory through ignition and later observations, and
writes an exclusive report. All seven must pass for replay acceptance. The
[eleven checker controls](checker-verification.json) include both original real
gas-loss failures and the real inert-gas pass, plus explicitly synthetic adversarial
fixtures. These tests validate the checker; they are not seven app executions or
current-source acceptance.

Example argument names: producer `--binary`, `--lock`, `--binding`,
`--artifact-review`, `--out`, `--fast`, `--storage`; consumer `--raw`, `--out`.
Actual operating locations are recorded privately. Preserve the original
protocol and all first outcomes when changing a producer, consumer or source.
