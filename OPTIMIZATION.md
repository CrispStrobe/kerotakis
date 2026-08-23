# Kerotakis — Optimization Tasks

Performance work that does not change chemistry or API.

## OPT-1 ✓ — Profile before optimizing

No optimization lands without a measurement showing it matters.
`cargo bench` with criterion, `twiggy` for wasm size attribution,
`heaptrack` or `dhat` for allocation profiles. Numbers go in the
commit message.

**Done (2026-08-23):** criterion benchmark suites across all three
engine crates. Run commands:

```sh
cargo bench -p kerotakis-core
cargo bench -p kerotakis-phreeqc
cargo bench -p kerotakis-cea
```

### Baselines (2026-08-23, Linux x86-64, stable Rust)

#### kerotakis-core (`benches/solve.rs`)

| Benchmark | Median |
|---|---|
| species::lookup (8 keys, HashMap) | 401 ns |
| kinetics::advance (thiosulfate, 0.1 s) | 74 µs |
| MixingEquilibrator::equilibrate | 423 ns |
| ConservedLedger::from_vessel | 9.7 µs |
| Vessel::clone (3 species) | 509 ns |

#### kerotakis-phreeqc (`benches/equilibrate.rs`)

| Benchmark | Median |
|---|---|
| PhreeqcEquilibrator::new (3 databases) | 123 ms |
| equilibrate (NaCl 0.1 mol) | 30 µs |
| equilibrate (HCl 0.01 mol) | 27 µs |
| equilibrate (cache hit) | 55 µs |
| dbindex::parse (wateq4f) | 3.7 ms |

#### kerotakis-cea (`benches/thermo.rs`)

| Benchmark | Median |
|---|---|
| ThermoDb::parse (thermo.inp) | 42 ms |
| nasa9::db lookup (10 species) | 3.0 µs |
| Species::cp (CO2, 10 temperatures) | 216 ns |
| equilibrate_tp (chalk in air, 1500 K) | 1.0 ms |
| equilibrate_hp (chalk adiabatic) | 7.5 ms |

## OPT-2 ✓ — Release profiles + wasm-opt

Cargo features `mimalloc` and `talc-alloc` are wired but not default.
Enable and benchmark before adopting:

```sh
cargo bench -p kerotakis-core --features mimalloc
```

Measured-adoption candidates:

- **`talc`** (MIT) — wasm allocator. The default `dlmalloc` is fine
  for correctness but `talc` is designed for wasm's linear-memory
  model and benchmarks show 10–20% allocation throughput improvement
  on wasm targets. Measure with `twiggy` size delta and `criterion`
  solve-time delta before adopting.
- **`mimalloc`** (MIT) — native allocator. Microsoft's compact
  general-purpose allocator. Measure on the full lesson-replay
  benchmark before adopting. Only for native targets; wasm stays
  on `talc` or default.
- **`twiggy`** (Apache-2.0) — wasm binary size attribution. Dev-only,
  never shipped. Use to identify which PHREEQC/SymEngine/MY-BASIC
  functions dominate the 1.9 MB wasm binary.
- **Warning: `wee_alloc` is unmaintained.** Do not adopt. Last release
  2020, known memory-leak bugs. `talc` is the maintained alternative.

**Done (2026-08-23):**
- `[profile.release]` in workspace Cargo.toml: `lto = "thin"`,
  `codegen-units = 1`, `strip = "debuginfo"`.
- `wasm-opt -Oz` pass added to `tools/build-web.sh` (runs when
  `wasm-opt` is installed; reports size reduction).

## OPT-3 ✓ — Wasm binary size budget

Current: 1.9 MB raw, 572 KB gzipped. Budget: 1 MiB gzipped.
Tools: `wasm-opt -Oz`, `twiggy top`, LTO, `codegen-units = 1`.
Measured in `tools/bundle-budget.sh`.

## OPT-4 ✓ — SpeciesId interning

The current `SpeciesId(String)` allocates a new string for every
species reference. For the DATA-010 refactor (pack-loaded registry),
intern species keys into a global string table.

Follow-up candidates:
- **`lasso`** (MIT/Apache-2.0) — concurrent string interner with
  `ThreadedRodeo`. Benchmarks show 5–10x faster lookups than
  `HashMap<String>` for repeated keys.
- **`string-interner`** (MIT/Apache-2.0) — simpler single-threaded
  interner if concurrency is not needed.

Acceptance: `SpeciesId` becomes a `Copy` type (index into the intern
table), and the `species::lookup()` path is a table index rather
than a linear scan.

## OPT-5 — Hot-path allocation reduction

Profile the solve path with `dhat`. Identify the top 5 allocation
sites. Reduce or eliminate allocations in:
- Rate evaluation inner loop (avoid `Vec::new` per step)
- Stoichiometric matrix reconstruction (cache once)
- Species lookup (intern, see OPT-4)
- Selected-output string splitting (avoid per-row allocation)

**Done (2026-08-23):**
- `species::lookup()` now uses a `OnceLock<HashMap>` for O(1) lookups
  (was O(n) linear scan over 75 entries per call)
- Event-restart loop zero-vector allocation hoisted outside the loop
- `lasso` wired with `multi-threaded` feature for the `intern.rs` module
- `apply_coupled_extents` deltas Vec and proposed-extents Vec hoisted
  outside the event-restart loop; reused via `clear()` across iterations
- Stoichiometric matrix is already static (`&[StoichiometricTerm]`
  slices in the `NETWORK` definition) — no reconstruction to cache
- dhat allocation profiler (`tests/allocation_profile.rs`):
  baseline 996 blocks / 37 KB for combined workload (200 lookups +
  kinetics integration + 10 conservation audits). Budget gate: < 5000 blocks.

**Remaining:** `selected_output` per-cell String allocation is
per-PHREEQC-run (not per-timestep), so it is low priority. The
public API returns `Vec<Vec<String>>` and callers in `aqueous.rs`
depend on the owned strings.

## OPT-6 ✓ — PHREEQC database pre-parsing

`generate-dbindex` already produces serialized indexes. Wire the
runtime to load the pre-parsed index instead of re-parsing the
raw database text on every `Phreeqc::with_database()` call.
Measured improvement: skip ~50 ms of text parsing per engine
instance creation.

## OPT-7 ✓ — Decompose `solve_once` (move-only refactor)

`solve_once` in `aqueous.rs` was ~1,320 lines. Extracted six named
private methods plus a `SolveSetup` struct for the partition/route
locals. Move-only: no logic edits, no reordering, no renaming.

| Method                        | Lines |
|-------------------------------|------:|
| `solve_once` (coordinator)    |   157 |
| `setup_problem`               |   214 |
| `dispatch_solve`              |   100 |
| `readback_raw_values`         |   235 |
| `apply_balance_corrections`   |   256 |
| `rebuild_contents_and_events` |   281 |
| `finalize_solution_info`      |   204 |

All 188 tests pass unchanged (0 failures, 0 tolerance changes).
