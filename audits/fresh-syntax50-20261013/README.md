# Fresh syntax-only fifty — frozen, execution pending

Lane 2 follows the completed performance lane. A fresh agent with no inherited
history read only [the syntax reference](syntax-only.md), then independently
invented exactly fifty distinct scripts and physical expectations. No registry,
implementation, tests, prior corpus, web references or previous outcomes were
consulted by the designer. [The independence statement](design/README.md) records
assumptions and observation conventions.

All fifty are frozen before execution in [the design manifest](design/freeze.json).
[Expectations](design/expectations.json) include questions, rationale, confidence,
observation points and pass/fail/unsupported/inconclusive criteria. Predictions
are independent hypotheses, not empirical calibration. Unsupported ingredient
keys are coverage results; they do not disprove the physical prediction.

The [execution binding](execution-binding.json) pins integrated main `30fd09eb`,
reviewed tree `25d1dd49`, ordinary release artifact `11811008023`, binary and lock
hashes, every design member, and the runner. The artifact was built on the
reviewed candidate merge tree; actual main has the same tree. This is source-tree
equivalence, not a claim of a new postmerge build.

Current result: **0/50 executed; no physical verdicts yet.** Load and memory gates
closed after design completed. The verified binary is staged privately. The
completed earlier evidence archives were also preserved durably and verified;
these are separate from the fresh fifty.

## Execution and review task

Run [run-frozen.py](run-frozen.py) with `--design` pointing to `design`, `--binary`
and `--binary-sha256` matching the binding, `--out` identifying a dedicated first
outcome directory, and `--fast` / `--storage` identifying the configured mounts.
Private operational locations are recorded in the private handoff.

The runner checks every frozen hash before execution, journals resource gates,
runs one fresh CLI process per script, and preserves exit codes, stdout, stderr,
digests and timeouts. Resume the same directory only to run untouched cases;
completed streams are hash-checked and skipped. Incomplete executions or timeouts
require review and a separately documented continuation. Never overwrite first
outcomes, rewrite forecasts, or substitute corrected scripts for the blind batch.

Once all first outcomes exist, evaluate each original expectation against its
observation commands, converting reported units and qualifying inferred command
semantics. Record physical failures, unsupported coverage and inconclusive
observations separately. Rank demonstrated failures by user impact and shared
cause. Repair proven app defects through narrow actual-main branches with
regression controls; keep corrected source-informed replays separate from this
original fifty. Finish this lane before starting the annotation, historical,
safety and accounting lanes in [the ordered roadmap](../../docs/ordered-work-roadmap-20261013.md).
