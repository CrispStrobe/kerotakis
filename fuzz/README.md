# Fuzzing

Five libFuzzer targets over the parsers and the bench loop (PLAN.md,
"Testing is part of the architecture"). This directory is its own
workspace on purpose: it needs nightly and libFuzzer, and must never
make `cargo test --workspace` need either.

```console
cargo +nightly fuzz run lab_grammar -- -max_total_time=120
cargo +nightly fuzz run stoich      -- -max_total_time=120
cargo +nightly fuzz run dbindex     -- -max_total_time=120
cargo +nightly fuzz run bench_ops   -- -max_total_time=180
cargo +nightly fuzz run --no-default-features quarantine -- -max_total_time=600
```

`quarantine` uses neither engine crate, so `--no-default-features` drops
their numeric dependency tree from its build: minutes instead of an hour,
and it fits in memory — the full-tree ASAN build was killed by the OOM
reaper on a loaded machine. **The flag must precede the target name**;
cargo-fuzz 0.13 accepts it after and then ignores it.

| Target | What it feeds | First run |
|---|---|---|
| `lab_grammar` | `script::parse_op` / `parse_vessel` | 2026-08-20: 4.4M runs, clean |
| `stoich` | `parse_equation` + `balance` | 2026-08-20: found a real panic in minutes |
| `dbindex` | `DbIndex::parse` on corrupted database bytes | 2026-08-20: 2.3M runs, clean |
| `bench_ops` | arbitrary operator sequences through the bench, NaN and infinity included | 2026-08-20: 3.2M runs, clean |
| `quarantine` | BRD-003's external-bytes surface: snapshot manifests, candidate fixtures, promotion policies and unit spellings | 2026-08-29: 1.57M runs in 901 s, clean |

`quarantine`'s first run started from a 14-file corpus seeded with the
checked-in quarantine and units fixtures and grew it to 2664 inputs
(3702 edges, 1743 exec/s, 446 MB peak). No crash, and neither of its two
assertions fired: canonical quarantine bytes survive a re-parse
unchanged, and an unpinned snapshot is always refused.

A crash goes to `artifacts/<target>/`; minimise with
`cargo +nightly fuzz tmin <target> <artifact>` and report it engine-side
with the artifact rather than fixing across the crate boundary.
Continuous fuzzing (OSS-Fuzz) is the open follow-up, per PLAN.
