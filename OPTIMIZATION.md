# Kerotakis — Optimization Tasks

Performance work that does not change chemistry or API.

## OPT-1 ✓ — Profile before optimizing

No optimization lands without a measurement showing it matters.
`cargo bench` with criterion, `twiggy` for wasm size attribution,
`heaptrack` or `dhat` for allocation profiles. Numbers go in the
commit message.

**Done (2026-08-23):** criterion benchmark suite (`benches/solve.rs`)
with 5 benchmarks: species lookup, kinetics integration, mixing
equilibrator, conservation audit, vessel clone. Run `cargo bench -p
kerotakis-core` to measure before and after.

## OPT-2 — Allocator selection (feature-gated, ready to measure)

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
- dhat allocation profiler (`tests/allocation_profile.rs`):
  baseline 996 blocks / 37 KB for combined workload (200 lookups +
  kinetics integration + 10 conservation audits). Budget gate: < 5000 blocks.

## OPT-6 ✓ — PHREEQC database pre-parsing

`generate-dbindex` already produces serialized indexes. Wire the
runtime to load the pre-parsed index instead of re-parsing the
raw database text on every `Phreeqc::with_database()` call.
Measured improvement: skip ~50 ms of text parsing per engine
instance creation.

## OPT-7 — One-worker web engine, fewer wasm→JS→wasm crossings

Binding restored 2026-08-24: this number was already bound by
ROADMAP-Webapp.md ("the scoped fixes — fewer calls first, cheaper
crossings only if measurement then says so — are OPT-7 and OPT-9"),
but the definition was missing from this file after the canonical
restore at 70ec6fb.

Scope: run kerotakis-wasm and IPhreeQC together in ONE module Web
Worker so the engine's internal synchronous hook stays synchronous
and per-call full-report JSON marshalling disappears from the hot
path; reduce the *number* of engine calls per vessel equilibration
first. This is also GUI-004 in ROADMAP-GUI.md (the `WorkerHost`
behind the EngineHost protocol) and is a prerequisite for the GUI's
web target. Owner: Fable (per /tmp/kero-sync.md; touches
crates/kerotakis-phreeqc/src/aqueous.rs).

Acceptance: current PWA runs unchanged on the consolidated worker;
before/after measurement of one `add`+equilibrate on the wasm path
recorded in the Baselines table.

**ID-collision note:** branch commit e51d870
(kero1/opt-bench-profiles) reuses "OPT-7" for a move-only
decomposition of `solve_once`. That refactor is welcome but must be
renumbered at merge — task numbers are stable identifiers and this
file plus ROADMAP-Webapp.md hold the binding.

## OPT-9 — Cheaper individual wasm boundary crossings (measure-gated)

Same restored binding as OPT-7. Only in scope if measurement AFTER
OPT-7 shows the remaining crossings still dominate: replace
full-report JSON strings with a narrower serialized delta or shared
buffer. Do not start this on assumption; the roadmap's order is
"fewer calls first, cheaper crossings only if measurement then says
so."

## OPT-8 — ccache + ninja auto-detection for vendored C/C++ builds

Owner: kero1 (commits b77260f/18a0649 on kero1/opt-bench-profiles).
build.rs for kerotakis-phreeqc and sundials-sys adds
CMAKE_*_COMPILER_LAUNCHER=ccache and the Ninja generator ONLY when the
tools are on PATH — a machine or CI runner without them builds exactly
as before. The cache itself is system-wide and shared across projects:
/etc/ccache.conf pins cache_dir=/mnt/volume1/ccache, max_size=5G; no
CCACHE_DIR may be set in env or scripts. Acceptance: `ccache -s`
evidence of a near-100%-hit second clean build of kerotakis-phreeqc,
plus one configure proving the no-ccache path still builds.

## OPT-10 — Hot-path clone reduction in aqueous.rs

Owner: kero1 (commit 87e4608, currently numbered "OPT-3" on its branch —
renumber to this at merge). MERGE HELD: the file is reserved and the
change overlaps OPT-7's call-count work; keromaster reviews the diff
against OPT-7 before it lands.

## Number allocation rule (2026-08-24)

A task number exists when — and only when — it has a section in THIS
file on main. The 2026-08-23 replacement file's numbering (its OPT-7
"redox bisection"/"solve_once", OPT-8 "one parser", OPT-9 "wasm
boundary decide") is void; branches still carrying that variant must
take main's file at merge and re-register any wanted tasks under fresh
numbers (OPT-11+). Allocate here first, then commit work against the
number.
