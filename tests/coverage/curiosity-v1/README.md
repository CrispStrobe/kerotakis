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
