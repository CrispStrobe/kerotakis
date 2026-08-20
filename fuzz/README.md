# Fuzzing

Four libFuzzer targets over the parsers and the bench loop (PLAN.md,
"Testing is part of the architecture"). This directory is its own
workspace on purpose: it needs nightly and libFuzzer, and must never
make `cargo test --workspace` need either.

```console
cargo +nightly fuzz run lab_grammar -- -max_total_time=120
cargo +nightly fuzz run stoich      -- -max_total_time=120
cargo +nightly fuzz run dbindex     -- -max_total_time=120
cargo +nightly fuzz run bench_ops   -- -max_total_time=180
```

| Target | What it feeds | First run (2026-08-20) |
|---|---|---|
| `lab_grammar` | `script::parse_op` / `parse_vessel` | 4.4M runs, clean |
| `stoich` | `parse_equation` + `balance` | found a real panic in minutes |
| `dbindex` | `DbIndex::parse` on corrupted database bytes | 2.3M runs, clean |
| `bench_ops` | arbitrary operator sequences through the bench, NaN and infinity included | 3.2M runs, clean |

A crash goes to `artifacts/<target>/`; minimise with
`cargo +nightly fuzz tmin <target> <artifact>` and report it engine-side
with the artifact rather than fixing across the crate boundary.
Continuous fuzzing (OSS-Fuzz) is the open follow-up, per PLAN.
