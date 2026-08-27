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

The initial baseline contains seven solver failures. They are regression
records, not supported or missing chemistry:

| Prompt | Owner | Failure route |
|---|---|---|
| `aq-106` | `EXP-2` | CEA thermal convergence for heated bicarbonate |
| `th-022` | `CAP-25` | PHREEQC convergence in extreme sealed-water heating |
| `th-034`, `th-035` | `BRD-041` | CEA gas-combustion convergence |
| `th-057` | `BRD-023` | PHREEQC convergence for an organic redox mixture |
| `th-097`, `th-102` | `BRD-032` | CEA high-temperature equilibrium convergence |

Each owner may remove its failure from the baseline only after the full prompt
executes without `SolverFailed` on both native CI platforms.
