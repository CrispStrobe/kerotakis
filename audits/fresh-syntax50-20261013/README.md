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

The two original physical failures share a serious thermal boundary defect:
sealed carbon and magnesium enter an open-air CEA route and silently lose owned
gases. [Merged PR786](https://github.com/CrispStrobe/kerotakis/pull/786), current
source `fdd951a7` on main `30fd09eb`, refuses unsupported thermal chemistry while
preserving the charged inventory. Mechanical pressure rupture remains explicit.

First CI failed on autoignition controls and a hydrogen dose comparison;
[the compatibility correction](pr786-compatibility-correction.json) preserves
those failures and the initial five-file repair. Native workspace tests and
CODEX checks then passed, but coverage found exactly two stale finite-carbon
support claims. [The scoped follow-up](pr786-coverage-correction-preparation.json)
preserves those original computed claims, qualifies current support and adds four
CLI controls. Original questions and scripts remain unchanged. The previous own
fleet completed successfully with all 33 jobs. [The descendant follow-up](pr786-followup-application.json)
brought the scope to ten files. [Integration is complete](pr786-integration.json):
current CI `38552626507` and own fleet `38552626557` passed, and main `8a6aa68e`
has exactly the validated candidate tree.

[Current-source macOS native validation](pr786-current-native-validation.json)
has passed, including the four CLI variants, direct boundary controls and
open/inert/kinetic controls; coverage has zero unmet requirements and zero drift.
The [new own-source artifact](pr786-current-artifact-review.json) is verified
against its actual build checkout, parents/tree and archived bytes. Its executable
is byte-identical to the earlier verified CLI; its captured lock digest is
recorded separately. [All 25 CI jobs now passed](pr786-current-ci-completion.json),
including Ubuntu and browser validation. All 33 own-fleet jobs also passed; the repair is merged.

[Draft PR787](https://github.com/CrispStrobe/kerotakis/pull/787) now contains
the core and app follow-up at `98051f7b` on integrated main `8a6aa68e`.
[The application receipt](water-alias-app-followup-application.json) records its
eleven-file scope and fresh CI `38627469049` / own fleet `38627469041`.
[Current macOS native validation passed](water-alias-app-candidate-native-validation.json)
with all nine regressions and zero coverage gaps/drift. The
[new own build's source association is verified](water-alias-app-candidate-own-build-metadata.json);
archive/binary bytes, other CI and full fleet are pending. The
[first app suite failed](water-alias-app-first-failure.json) with 1,636 passes and
one visualization-guard failure from scanning stock accounting as drawing.
The seven new regressions were included. A staged correction at `d5c6cecf`
separates that predicate into stock accounting and adds a direct test, preserving
the drawing guard. It remains unapplied/unexecuted; PR787 is not integrated.

Previous core source `e4fb5e33` passed [full CI](water-alias-core-ci-validation.json)
and [all 33 own-fleet jobs](water-alias-core-fleet-validation.json), including
nine native alias regressions. The first [two CLI controls failed](water-alias-cli-first-outcomes.json)
on an invalid final command; two separately frozen [corrected controls passed](water-alias-cli-followup-outcomes.json)
with unchanged quantitative forecasts and verified saved stream/report hashes.
Those source bindings remain intact. Core/CLI/WASM is unchanged in the new app
candidate; the old receipts do not establish app acceptance. The
[ordered agenda](../../docs/improvement-agenda-20261014.md) specifies the remaining
validation and guarded integration. Original forecasts and first outcomes remain unchanged.

[The original seven-control replay](pr786-replay-first-outcomes.json) at
`10c963e1` produced five passes and two failed intact-seal predictions. Both
original beakers reported bursts at 440.15/507.59 kPa above the modeled 405.3 kPa
rating, with danger warnings and fully accounted vented gases. Those original
forecasts and failures remain unchanged. [The two separate low-pressure controls](pr786-low-pressure-outcomes.json)
passed at that same source: sealed fuel and gas inventories remain intact through
later observations, with explicit thermal refusals. The complete result archive
and associated stream/report/receipt/binary/lock digests were verified; saved
reanalysis reproduced the report exactly. No earlier-source result is relabeled
as current-source acceptance. Validated closed U/V or finite H/P chemistry remains
a separate extension; refusing the invalid route does not implement it.

Follow the [new ordered improvement agenda](../../docs/improvement-agenda-20261014.md)
before resuming the older queued lanes.

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
