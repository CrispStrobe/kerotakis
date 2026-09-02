# Curiosity v1 baseline

`manifest.toml` orders four authored prompt shards. `baseline.toml` records the
observed native route for every prompt as an ID, owning task, outcome, and
stable reason code. It deliberately excludes rendered prose, numerical solver
details, and route timing.

Run the fast cross-family gate with:

```sh
cargo run -p kerotakis-cli -- coverage curiosity --smoke --check
```

Run the complete comparison with:

```sh
cargo run -p kerotakis-cli -- coverage curiosity --check
```

`--check` accepts known failures recorded in the baseline and fails only when
an observation is added, removed, changes disposition, changes reason, or
changes owner. A baseline update therefore needs a scientific explanation in
the commit or PR; do not regenerate it merely to make CI green. To inspect a
candidate after an intentional engine/data change:

```sh
cargo run -p kerotakis-cli -- coverage curiosity --emit-baseline
```

Review the diff prompt by prompt, update the applicable `CAP-*`, `EXP-*`, or
`BRD-*` task, and only then replace the checked-in baseline.

Refreshed 2026-09-02 (four rows), and wired into CI in the same change.

The gate had never run anywhere — not in `ci.yml`, not in `preflight.sh` —
so its baseline drifted on four rows without anyone seeing it, which makes a
regression detector into a file. It now runs in the native test job; 500
prompts through the real solver stack cost about 30 seconds.

All four rows moved in the honest direction, which is why the refresh is a
record rather than a concession:

- `mat-024`, `th-032`, `th-061`: `typed-engine-event` -> `computed-route`.
  The disposition did not change; the REASON got better. A real computed
  solver route now succeeds where the classifier previously fell back to
  "there were typed events, so call it computed" — the fallback is the
  weakest evidence the classifier accepts, and these three no longer need it.
- `mat-032`: `missing`/`not-yet-modeled` -> `qualitative`/`typed-observation`.
  A row that used to stand aside now yields a typed observation.

The species behind them became real in already-merged work — `dough` in
BRD-014 (#237), `methanol` last moved by EXP-33 (#288), `PE` in EXP-12 — so
the engine had genuinely improved and only the record was stale. Note the
smoke set would NOT have caught this: none of the four are in it, which is
why the full check is what runs.

## What "expectation mismatch" actually counts (2026-09-02)

The summary used to print one number — 151 — and one number here cannot be
acted on, because it conflates three populations with different owners and
opposite meanings. A prompt's `expected` is a PREDICTION of what the engine
will do, and predictions age in both directions. The tool now prints the
split, and names the rows in the only column that is a backlog:

| population | count | what it means |
|---|---|---|
| engine gained | 64 | the corpus said `missing`; the engine now answers. The expectation is stale because the engine IMPROVED. Nothing is wrong. |
| engine stood aside | 19 | the corpus claimed an answer; the engine declined (`not-yet-modeled`). **The tail worth working** — but named for what was OBSERVED, because the cause is not established. See below. |
| route differs | 68 | both answer, by different roads. The author predicted one and the engine took another. |

**The 19 need the same suspicion applied to them that they applied to the
64.** Naming this column "capability absent" would assert a cause it has
not established, and early reading says these are not one thing either.
At least four sub-kinds are now known, three of them measured rather than
read:

- **The script never reaches the capability.** `aq-035`, `aq-037` and
  `aq-038` are classic gas tests written as two lines — `add v1 <gas>`
  then `test v1 <reagent>` — with no `seal`. A fresh vessel is open, so
  the gas has left before the test runs, and `gas_tests.rs` implements all
  four with sourced thresholds. Add `seal v1 500mL` and all three go
  positive immediately. The engine is right and the script is incomplete:
  it omits the step a real bench requires, which is collecting the gas
  before testing it. (Found and measured by kerotakis-e9.)
- **The engine answers, confidently, and is wrong.** `aq-036` looks like
  one of the three above and is not. Sealed, it reports "damp red litmus —
  negative", because `NH3` in this registry is AMMONIA SOLUTION (its
  registry name is literally "ammonia solution") and there is no gaseous
  ammonia species at all — so the headspace fraction is always 0.00% and
  the test reports an absence the bench never modelled. It also teaches the
  opposite of the chemistry: damp red litmus DOES identify ammonia, and
  holding it over the bottle is the school demonstration. A silent stand
  aside is bad; a confident negative on a question the bench cannot see is
  worse, because nothing about it looks like a gap. (kerotakis-e9.)
- **A genuine absence.** `mat-096`/`mat-099` want rusting over `wait 1h`,
  `mat-003` wants surface area from `grind` to change a rate, `mat-034`
  wants a polymer to dissolve, `mat-011` wants the conductivity of a solid
  metal from a meter that is Kohlrausch and wants a solution.
- **A negative the engine is right to give and wrong to explain.**
  `aq-089` asks whether a magnet removes copper powder — it does not,
  copper is not ferromagnetic, and `MagnetSeparated` carries
  `attracted`/`remained` and could say so. `aq-087` asks whether dissolved
  salt appears in a neutral-solute chromatogram — it does not, and
  `Chromatographed.outside_method` exists precisely to name what the method
  cannot see. In both the refusal is correct and `not-yet-modeled` is the
  wrong reason for it.

The 19 rows span ten owning tasks — EXP-25 (4), EXP-18 (3),
CAP-16 (3), EXP-15 (2), BRD-023 (2), and one each for CAP-11, CAP-23,
CAP-25, EXP-13, BRD-014 — so they are a distributed backlog rather than one
defect, and each belongs to the task that claimed the capability.

**This answers the queued th-030/th-059 item, and the answer is that there
is nothing to fix in the classifier.** Both rows are in the *engine gained*
column: they expect `missing` and observe `computed`, and both are owned by
BRD-014, which is exactly the tranche that gave those materials real
registry identities after the expectations were written. The reverted
classifier change of 2026-08-31 was trying to make the engine's OBSERVATION
match a stale EXPECTATION — which is backwards, and is why it downgraded 49
honest rows on the way. The expectations are what aged, not the classifier.

Whether a stale `expected` should be rewritten in bulk is a content ruling
and is deliberately not taken here: it turns on whether `expected` is a
prediction of the engine or a requirement on it, and those two readings
want opposite fixes.

The baseline contains no solver failures. The last two left on
2026-08-31 when the aqueous engine gained its validity boundaries —
and both rows got *better* than a refusal, because the crash had been
sitting on top of an answer. `th-022` (extreme sealed-water heating)
no longer asks PHREEQC to converge beyond llnl.dat's 300 °C
parameterisation, and the burst physics that was the question's whole
point classifies as `computed` — matching the prompt's expectation for
the first time. `th-057` (permanganate heated with ethanol) no longer
crashes the aqueous engine on a mixture that was never aqueous, and
the curated oxidation route underneath answers as `curated`; the
speciation the model cannot claim is declined, with the reason spoken.
A solver failure returning here is a regression; record it in this
table with its owning task while it is being fixed.

| Prompt | Owner | Failure route |
|---|---|---|
| _none_ | | |

The five CEA failures the initial baseline carried (`aq-106`, `th-034`,
`th-035`, `th-097`, `th-102`) were removed on 2026-08-29 after the Gibbs
solver's phase handling was repaired: condensed records are admitted only
within their data range, same-name phase records merge their intervals,
each substance's phase family joins the pool, a singular solve re-seeds
crushed carriers instead of failing, and the adiabatic bisection survives
bracket points that cannot converge. `th-097` still runs from a garbage-in
vessel temperature (the latent-heat plateau is BRD-032's open model); what
changed is that the solver now answers consistently instead of dying.

Each owner may remove its failure from the baseline only after the full prompt
executes without `SolverFailed` on both native CI platforms.
