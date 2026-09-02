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
