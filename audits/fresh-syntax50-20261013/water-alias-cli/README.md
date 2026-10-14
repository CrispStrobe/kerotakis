# Mixed-spelling CLI acceptance controls

Two source-informed controls compare canonical water inputs with mixed `H2O`
and `water` inputs. Both must debit one finite bottle, retain 0.5 mol after two
accepted additions, refuse the 0.75 mol overdraw without inventory mutation, and
preserve canonical stock/inventory identities through later observations.

[Expectations](expectations.json) and [protocol](protocol.json) were frozen before
execution at PR787 source `e4fb5e33`, bound to its verified own CLI artifact.
Current state: **2/2 executed, 2 failed** on the hosted worker. Both saved
traces show canonical water inventory and the expected shared finite-bottle
exhaustion at 0.5 mol. The final `balance v1` command was invalid, so both
processes exited 1. The original reader also incorrectly expected a stock map
in CLI bench JSON. These are experiment/reader errors, not evidence of a new
kernel alias defect. Preserve the [first outcomes](../water-alias-cli-first-outcomes.json),
original scripts, expectations, raw streams and reader crash.

The [first hosted run](https://github.com/CrispStrobe/kerotakis/actions/runs/38607283700)
completed with failure. The [saved-data reader](analyse.py) verifies stream hashes
and uses the named stock events to assess the original observations. It preserves
both formal failures. Map cardinality is separately covered by native tests.

Two [separately frozen follow-ups](../water-alias-cli-followup/README.md) correct
only the final command and keep the same quantitative expectations. Suspend
local retries while their hosted worker is queued; do not duplicate executions.

The [nine native alias regressions passed on macOS](../water-alias-native-validation.json).
These CLI controls remain separate from the original blind fifty and normalized
forty, and do not establish physics calibration or current integration acceptance.
