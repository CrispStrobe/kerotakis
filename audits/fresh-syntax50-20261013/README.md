# Fresh syntax-only fifty — first outcomes preserved

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

Current result: **50/50 executed: 7 pass, 2 fail, 40 unsupported and 1 inconclusive.**
Every case was admitted by its recorded resource gate. All original scripts,
expectations and first streams remain unchanged. [The first-outcome review](first-outcome-review.json)
and [observation verification](observation-verification.json) distinguish physical
observations from coverage and inferred boundary semantics. This is root review,
not a blinded independent second review or empirical calibration.

The forty unsupported cases all rejected the guessed `H2O` key with a `water`
suggestion before the intended comparison. [Forty spelling-normalized controls](normalized40/design/README.md)
are frozen separately, retaining the physical forecasts and changing only the
`add` ingredient token. These are source-informed follow-ups, not replacements
for the original blind outcomes or forty newly invented experiments.

The two physical failures share a serious thermal boundary defect: sealed carbon
combustion drops all gases, ends with zero pressure and gives a negative CO2 test;
sealed magnesium combustion also loses retained oxygen and nitrogen. The open-air
CEA route adds reservoir air and vents products at a finite boundary. Narrow fix
`faaf4baf` preserves inventory and reports `boundary-mismatch` for sealed,
pressure-controlled and swept charges that would enter this open-air route.
It includes six boundary/fuel controls plus open-air and inert-gas controls,
with an English/German diagnostic. Formatting passed; native validation and
integration are pending. Validated closed energy/volume or energy/pressure
chemistry remains a separate extension; refusing an invalid route does not
implement that chemistry.

The sweep case is inconclusive: `sweep` establishes an external nitrogen purge,
not a finite 1 L gas inventory. The reported pressure is 10 kPa. Preserve the
frozen assumption rather than changing it into a passing forecast.

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

After the preserved first outcomes, review each original expectation against its
observation commands, converting reported units and qualifying inferred command
semantics. Record physical failures, unsupported coverage and inconclusive
observations separately. Rank demonstrated failures by user impact and shared
cause. Repair proven app defects through narrow actual-main branches with
regression controls; keep corrected source-informed replays separate from this
original fifty. Finish this lane before starting the annotation, historical,
safety and accounting lanes in [the ordered roadmap](../../docs/ordered-work-roadmap-20261013.md).
