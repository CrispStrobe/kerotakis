# Kerotakis — Optimization Tasks

Performance work that does not change chemistry or API.

## OPT-1 — Profile before optimizing

No optimization lands without a measurement showing it matters.
`cargo bench` with criterion, `twiggy` for wasm size attribution,
`heaptrack` or `dhat` for allocation profiles. Numbers go in the
commit message.

## OPT-2 — Allocator selection

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

## OPT-3 — Wasm binary size budget

Current: 1.9 MB raw, 572 KB gzipped. Budget: 1 MiB gzipped.
Tools: `wasm-opt -Oz`, `twiggy top`, LTO, `codegen-units = 1`.
Measured in `tools/bundle-budget.sh`.

## OPT-4 — SpeciesId interning

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

## OPT-6 — PHREEQC database pre-parsing

`generate-dbindex` already produces serialized indexes. Wire the
runtime to load the pre-parsed index instead of re-parsing the
raw database text on every `Phreeqc::with_database()` call.
Measured improvement: skip ~50 ms of text parsing per engine
instance creation.
