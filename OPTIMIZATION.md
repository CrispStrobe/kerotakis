# Kerotakis — Optimization tasks

Findings from a full-workspace performance survey, **2026-08-23**, every
file:line reference verified against the tree that day. Line numbers rot:
**re-verify each reference against the current tree before editing** — the
claim is the anchor, the number is a hint.

The survey's headline: the codebase is structurally healthy (zero
TODO/FIXME markers, 322 tests, proptest conservation checks, differential
oracles, 105-package dependency tree, ~53 s cold `cargo check`). The debt
is not cleanliness; it is a handful of concrete hot-path problems, almost
all of them in `kerotakis-phreeqc/src/aqueous.rs` and
`kerotakis-cea/src/gibbs.rs`, plus the total absence of any way to measure
an improvement. Hence the ordering rule below.

**The ordering rule: no optimization lands without a before/after number.**
OPT-1 exists to make that number obtainable. Until OPT-1 has landed, only
behavior-preserving, test-proven changes (OPT-2, OPT-8) may merge.

**Status check after the R-sprint merge (2026-08-23, same day):** the
survey's hot-path findings survive the sprint — the 34-iteration pe
bisection with its bracket-width-only exit now sits at
`aqueous.rs:298`/`336` in a file grown to 3,077 lines; there are still
no benches, no `[profile.release]`, and no `wasm-opt` pass. Every line
number in the tasks below is pre-merge — re-verify per ground rule
before editing.

**Claim audit (2026-08-23, evening).** This file is canonical; a same-day
replacement re-bound the OPT numbers to different topics and was
restored — its two real measurements are kept: **the wasm module is
1.9 MB raw / 572 KB gzipped**, and `tools/bundle-budget.sh` exists to
track it (fold both into OPT-2's baseline row). Completion claims from
the day's waves, verified against the tree: **OPT-4 is half done** —
the `OnceLock` + `HashMap` lookup index landed in `species.rs`, while
`REGISTRY` remains `pub const` (the binary-size half is open). **OPT-1,
OPT-2, OPT-3 and OPT-5 remain open despite marks to the contrary**: no
`benches/` exists anywhere, no `[profile.release]` exists, the cache
deep-clone is alive (`hit.clone()`, now near `aqueous.rs:1492`), and
the CEA Newton loop still allocates `vec![vec![…]]` per iteration.
A checkbox without its Acceptance evidence is a claim, not a status —
and the ordering rule stands: perf work keeps landing with no
before/after number because OPT-1 keeps not being done first.

---

## Ground rules — read before starting any task

These are the working protocol for every agent taking a task from this
file. They exist because each was violated once and cost a day.

1. **Work in your own git worktree** (`git worktree add`), never in the
   shared checkout. Consolidate only by merge/fast-forward/cherry-pick or
   rebasing your own unpushed work. No `git reset` on shared branches, no
   force-push, no history rewriting — shared history is append-only.
2. **Build with a private `CARGO_TARGET_DIR`.** The shell environment on
   the primary machine redirects it to a backup volume that is not always
   mounted, and a *shared* target dir can serve another worktree's rmeta
   under an identical fingerprint — observed as a compile error demanding
   an enum variant that existed only in a peer's in-progress sources.
   `export CARGO_TARGET_DIR="$(pwd)/.target-local"` (gitignored) is fine.
3. **`tools/preflight.sh --light` gates every branch push** (fmt +
   clippy + no-engine; <2 min warm). **Main moves only by PR with the
   full CI `preflight-gate` job green.** The full `tools/preflight.sh`
   (without `--light`) runs in CI on every PR and push to main. Never
   chain "check; push" with semicolons — that pushes on red.
4. **Commit *and push* at every checkpoint.** A commit that was never
   pushed is invisible to the other sessions and dies with the window.
5. **Chemistry output is the contract.** The conservation proptests, the
   oracle suites (`differential_oracle.rs`, `basic_dialect_oracle.rs`,
   `bundled_rates_oracle.rs`), and the lesson replays must pass unchanged.
   A task that can legitimately move a number in its last digits says so
   explicitly in its acceptance section; every other task means
   *bit-identical results*.
6. **One task, one branch, one PR-sized diff.** Do not fold two tasks into
   one commit "while you're there" — the exception is where a task's scope
   below says so.
7. **Record your numbers in the Baselines section** at the bottom of this
   file, in the same commit as the change. An optimization commit without
   its measurement is not done.
8. Sign commits with `Signed-off-by:` (see CONTRIBUTING.md).

### Dependency graph

```
OPT-1 (benches)          OPT-2 (profiles/wasm-opt)     OPT-8 (one parser)
   │                          independent                  independent
   ├──────────┬──────────┬──────────┐
   ▼          ▼          ▼          ▼
 OPT-3      OPT-4      OPT-5      OPT-6 (decompose solve_once)
 (aqueous   (registry  (CEA                 │
  mechanics) index)     matrix)             ▼
                                   OPT-7 (redox bisection — the big one)
                                            │
                                            ▼
                                   OPT-9 (wasm boundary — decide, don't build)
```

OPT-3/4/5 touch disjoint files and can run in parallel worktrees.

---

**Second replacement (2026-08-24 audit).** This file was replaced again
on 2026-08-23 (9a88ba7, then built upon through e51d870), re-binding
OPT-6 to database pre-parsing and OPT-7 to the `solve_once`
decomposition. Restored 2026-08-24; the replacement's real work is kept
and recorded below under the canonical numbers it actually corresponds
to, each claim verified against the tree. Task numbers are stable
identifiers and are never re-bound.

## OPT-1 — Criterion benchmarks (enabling; do this first)

- [x] Status: **done 2026-08-23** (kero1, 9a88ba7 + 2c753aa; audited
      2026-08-24: `benches/` exist in all three engine crates —
      `solve.rs`, `equilibrate.rs`, `thermo.rs` — with medians recorded
      in Baselines below).

**Why.** There is no `benches/` directory, no criterion, no `#[bench]`
anywhere in the workspace. Every task below is required to produce a
before/after number, and today none is obtainable.

**Scope.**

- Add `criterion` to `[workspace.dependencies]` and as a dev-dependency of
  `kerotakis-phreeqc`, `kerotakis-core`, `kerotakis-cea`.
- `crates/kerotakis-phreeqc/benches/`: one bench equilibrating a vessel
  that forces the coupled redox path (`solve_coupled`), and one that stays
  on the plain path — both behind the `engine` feature, skipped without it.
  Reuse an existing integration-test setup (e.g. from `tests/redox.rs`) so
  the bench is chemistry that already has a correctness test.
- `crates/kerotakis-core/benches/`: one `displace()` pass over a
  multi-couple vessel; one short `.lab` script replay through
  `bench::apply`. Careful: `kerotakis-core/src/bench.rs` is the
  *laboratory bench* (vessels + operators), not benchmarking — name the
  bench files so nobody confuses the two (`benches/perf_displace.rs`, …).
- `crates/kerotakis-cea/benches/`: one `equilibrate_tp` and one
  temperature-bisecting solve (the `gibbs.rs:455` path).
- Run each bench 3× on a quiet machine; record medians in the Baselines
  section of this file.

**Out of scope.** Any change to `src/`. CI integration (benches are run by
hand; a CI perf job is a separate decision).

**Acceptance.** `cargo bench` runs green in each of the three crates;
baselines recorded below; `tools/preflight.sh` green.

**Size.** Small. **Depends on:** nothing.

---

## OPT-2 — Release profiles, wasm-opt, dependency hygiene

- [x] Status: **done 2026-08-23** (kero1; audited 2026-08-24:
      `[profile.release]` in the workspace root, `wasm-opt -Oz` in
      `tools/build-web.sh` guarded on `command -v wasm-opt` with the
      loud warning, `mimalloc`/`talc-alloc` wired as opt-in features
      awaiting a measured adoption decision — which is what the scope
      asked for).

**Why.** No `[profile.release]` exists anywhere in the workspace: default
release means no LTO and `codegen-units = 16`. `tools/build-web.sh` runs
`wasm-bindgen` with no `wasm-opt` pass. For a numerics workload with a
browser target this is free speed and free bundle size.

**Scope.**

- Workspace root `Cargo.toml`:
  `[profile.release] lto = "fat", codegen-units = 1`. Do **not** set
  `panic = "abort"` workspace-wide without checking what
  `cargo test --release` and the fuzz targets need; if it complicates
  either, apply it only to the wasm build via a dedicated
  `[profile.wasm-release]` (profile inheritance) or `RUSTFLAGS` in
  `tools/build-web.sh`, and say which you chose in the commit message.
- `tools/build-web.sh`: add a `wasm-opt -Oz` pass after `wasm-bindgen`,
  guarded on `command -v wasm-opt` with a loud warning when absent (the
  build must not start failing on machines without binaryen).
- Move `postcard` (currently declared verbatim in 4 places) and `toml`
  (2 places) into `[workspace.dependencies]`; drop the redundant
  `postcard` entry in `kerotakis-cli`'s `[dev-dependencies]` (it is
  already a regular dependency there).
- Check whether `kerotakis-thermo`'s declared `thiserror` is actually
  used; drop it if not.
- Measure: `kerotakis-wasm` module size before/after (and the Emscripten
  IPhreeqc module if the flags reach it); native bench deltas once OPT-1
  exists. `twiggy` (dev-only) attributes where the wasm bytes actually
  go before and after.
- Optional, each adopted only if its measured delta earns it: `talc`
  (MIT) as the wasm allocator — the old default suggestion `wee_alloc`
  is unmaintained, do not use it; `mimalloc` (MIT) as the native CLI's
  global allocator (the aqueous hot path is allocation-heavy until
  OPT-3/OPT-7 land, and remains String-heavy after).

**Out of scope.** Compressing the shipped PHREEQC `.dat` databases
(worthwhile, but it touches the service worker and load path — file it
separately if the numbers say it matters).

**Acceptance.** `tools/preflight.sh` green; the wasm demo still loads and
runs a lesson; sizes recorded in Baselines. Bit-identical chemistry is
expected (LTO must not change results; if a golden test moves, stop and
report rather than adjusting the tolerance).

**Size.** Small. **Depends on:** nothing (numbers richer after OPT-1).

---

## OPT-3 — Hot-path mechanics in `aqueous.rs`

- [ ] Status: open. **Audit 2026-08-24:** an earlier "Mark OPT-3/4
      complete" over-claimed this half — verified against the tree:
      `hit.clone()` still at aqueous.rs:614 and :1893, the
      `#[allow(clippy::type_complexity)]` tuple still at :122/:1866,
      and no `OnceLock` env-flag hoists exist in the file.

**Why.** Three independently small costs sit on the path of *every*
engine call, and one more on every cache hit.

**Scope** (all in `crates/kerotakis-phreeqc/src/aqueous.rs`):

- **Cache values are deep-cloned on every hit and every insert.** The
  cache value is an untyped 5-tuple `(Vec<Vec<String>>, Vec<SpeciesDetail>,
  Vec<(String,f64)>, bool, bool)` — `hit.clone()` at ~line 1246 and the
  triple `rows.clone()/speciation.clone()/saturation.clone()` on insert
  (~1305). Give the tuple a named struct (this also retires the
  `#[allow(clippy::type_complexity)]` at ~line 115) and wrap it in `Arc`
  so hits and inserts are refcount bumps.
- **O(n²) dedup in the species-distribution parser**: ~line 2500,
  `!result.iter().any(|r| r.name == tokens[0])` — a linear scan with
  string compare per parsed line, on every cache miss. Replace with a
  `HashSet<String>` (or `&str` into an arena) seen-set.
- **`std::env::var` in loop bodies**: `KERO_DUMP_INPUT` (~224, inside
  `run_raw`, so every engine call; and ~1257), `KERO_REDOX` (~311, inside
  the 34-iteration bisection), `KERO_READBACK` (~1398). Hoist each into a
  `OnceLock` read once per process. Note this freezes the flags at first
  read — acceptable for debug flags; say so in a comment.
- **Leave the eviction policy alone** (`len() >= 10_000 → clear()`,
  ~1301). Its own comment says "refine when profiling says so", and
  profiling has not said so yet.

**Out of scope.** Anything that changes *which* engine calls happen —
that is OPT-7. This task must be behavior-preserving.

**Acceptance.** Bit-identical test results across the whole workspace;
`tools/preflight.sh` green; phreeqc bench delta recorded.

**Size.** Small. **Depends on:** OPT-1 (for the number).

---

## OPT-4 — Species registry: index the lookups, un-inline the table

- [x] Status: **done** (audited true 2026-08-23: `OnceLock` +
      `HashMap` behind `species::lookup`, API unchanged; CAP-21's
      build-time codegen then completed the binary-size half — the
      table is generated, `static`, and `species.rs` shrank 1,563 → 179
      lines).

**Why.** `species::lookup` is a linear scan —
`REGISTRY.iter().find(|s| s.key == id.0)` at
`crates/kerotakis-core/src/species.rs:1522` (and a twin at 1526) over a
74-entry table, with 62 call sites across the workspace, several inside
loops (per-portion in `aqueous.rs::partition`, inside the candidate-phase
loop, per-couple in `displacement.rs`). Separately, `REGISTRY` is
`pub const` (species.rs:146): a `const` table is inlined per use site
across 5 dependent crates — a codegen and binary-size liability that the
wasm bundle pays for.

**Scope.**

- Change `pub const REGISTRY` to `pub static REGISTRY`. Check the few
  places that may rely on `const` promotion; the fix is mechanical.
- Build a `OnceLock<HashMap<&'static str, &'static SpeciesData>>` keyed
  on `key`, populated from `REGISTRY` on first use; route both `find`
  call sites through it. The public API of `lookup` does not change, so
  the 62 call sites need no edits.

**Out of scope.** Interning `SpeciesId` (it holds a `String` and is
cloned wholesale in the vessel fixed-point loop — real, but it spreads
into every crate; file separately if OPT-7's numbers say vessel clones
still matter afterwards — `lasso` or `string-interner`, both
MIT/Apache-2.0, are the ready-made answer when that day comes).

**Acceptance.** Bit-identical tests; preflight green; wasm module size
delta recorded (this is where the `const`→`static` change shows up).

**Size.** Small. **Depends on:** OPT-1 (for the number).

---

## OPT-5 — CEA Gibbs solver: stop allocating inside Newton

- [x] Status: **done 2026-08-24** (Opus). Flat row-major matrix
      allocated once outside the Newton loop; `pi`, `d_ln`, `gas_ni`
      hoisted; `solve_flat` replaces Vec-of-Vec Gauss-Jordan; per-element
      gas sums precomputed once per iteration. Summation order preserved.
      Criterion bench delta: `equilibrate_tp` 642 µs → 456 µs (−29%),
      `equilibrate_hp` 12.4 ms → 10.7 ms (−14%). All CEA tests,
      golden fixtures, and conservation proptests unchanged.
      **Audit 2026-08-24:** gibbs.rs:228 still allocates
      `vec![vec![0.0; dim + 1]; dim]` per Newton iteration. (kero1's
      afbd549 hoisted `deltas`/proposed-extents Vecs out of the
      event-restart loop in `kerotakis-core` kinetics — real and kept,
      but an adjacent site, not this task.)

**Why.** `crates/kerotakis-cea/src/gibbs.rs:226-228`: every iteration of
a 400-iteration Newton loop allocates a fresh `Vec<Vec<f64>>` — `dim + 1`
separate heap allocations plus pointer-chasing through the Gauss-Jordan
solve — and that loop sits inside a 60-iteration temperature bisection
(gibbs.rs:455). Up to ~24,000 iterations per solve, each allocating. The
inner element×element assembly (~241-243) also recomputes a
`gas.iter().map(...).sum()` for every `(j,k)` pair — O(elements² × gas
species) per iteration, recomputable in one pass.

**Scope.**

- Replace the matrix with a single flat row-major `Vec<f64>` allocated
  once outside both loops and zeroed per iteration; adapt the
  Gauss-Jordan elimination (~471-500) to flat indexing.
- Precompute the per-element gas sums once per iteration instead of per
  `(j,k)` pair.
- **Preserve the arithmetic order of every summation.** Reordering
  floating-point sums changes bits, and the golden tests define the
  contract. If a reordering is genuinely needed, stop and report the
  observed drift instead of widening a tolerance.

**Out of scope.** Changing the Newton or bisection iteration counts,
damping, or convergence criteria — this task is allocation and indexing
only.

**Acceptance.** CEA unit tests, golden fixtures and the Cantera-side
oracle comparisons pass unchanged; preflight green; CEA bench delta
recorded.

**Size.** Small-medium. **Depends on:** OPT-1.

---

## OPT-6 — Decompose `solve_once` (refactor only, no behavior change)

- [x] Status: **done 2026-08-23** (kero1, e51d870 — committed under the
      label "OPT-7" in the replaced file; this is the task it belongs
      to). Six named private methods (`setup_problem`,
      `dispatch_solve`, `readback_raw_values`,
      `apply_balance_corrections`, `rebuild_contents_and_events`,
      `finalize_solution_info`) plus a `SolveSetup` struct; the energy
      section stays inline in the coordinator. **Move-only review
      (Fable, 2026-08-24):** whitespace-normalized multiset diff of the
      aqueous.rs hunks = method signatures, the struct, and
      destructuring plumbing; the 19 residual changed lines are all
      borrow-shape adjustments forced by extraction (`&mut v` →
      `v.iter_mut()`, `&problem` → `problem` where the parameter is
      already a reference); full workspace suite green on the merged
      tree.

**Why.** `solve_once` (`aqueous.rs`, ~line 1115) is 968 lines and is the
function where the caching, routing and read-back all live. In its
current shape it can be neither profiled per-phase nor safely modified —
and OPT-7 has to modify it.

**Scope.**

- Extract named phases as private methods: partition → route → build
  input → solve → parse → read back. Move-only extraction: the diff
  should read as cut-and-paste plus a signature, and a reviewer must be
  able to verify it as such.
- While extracting the solve phase, give `solve_coupled` (~264) its
  bracket `(lo, hi)` and iteration budget as parameters with the current
  values as defaults — no behavior change, but it is the seam OPT-7
  needs.
- Target: no extracted function over ~200 lines; no logic edits, no
  reordering, no "improvements while we're here".

**Acceptance.** Bit-identical test results; preflight green; diff
reviewed as move-only.

**Size.** Medium (mechanically large, intellectually small).
**Depends on:** best after OPT-3 to avoid churn in the same lines, but
not blocked by it.

---

## OPT-7 — Redox bisection: cache trials, warm-start, converge on residual

- [x] Status: **done 2026-08-23** (Fable). All three compounding
      changes: `bisect_pe` extracted with a residual-tolerance break in
      addition to bracket width; every trial routed through a
      `trial_cache` keyed on the exact pe-tagged input text; warm-start
      bracket ±0.75 around the previous converged pe with full-bracket
      `(-10, 17)` fallback, reset on `equilibrate` entry. Measured by
      the per-equilibration engine-call counter the scope demanded:
      **272 → 20 engine calls** on the worst coupled case, repeat
      count 0, all tests unchanged (no tolerance moved).

**Why.** One vessel equilibration runs up to 8 temperature/volume
fixed-point iterations (`equilibrate`, ~1023); each may enter
`solve_coupled` (~264), a **34-iteration pe bisection** (~285); every
bisection trial rebuilds the input text (`build_input_at`, 183 lines of
string assembly) and invokes the PHREEQC engine fresh. Worst case:
**~272 full engine solves per equilibration.** The content-addressed
cache is consulted once per `solve_once` (~1243) on the *uncoupled*
input, so all 34 trials bypass it; nothing is reused across the 8 outer
iterations either, though the pe root barely moves between temperature
guesses; and the bisection breaks only on bracket width
(`hi - lo < 1e-6`, ~323), never on the residual, so it runs ~25+
iterations even when the target was hit on iteration 3.

In the browser this is multiplied roughly tenfold: each engine call is a
wasm→JS→wasm round trip through the Emscripten IPhreeqc module with the
full report copied out of its heap and re-parsed (see
`web/kerotakis.mjs`).

**Scope** (three compounding changes, one branch, separable commits):

1. **Cache the bisection trials.** Route each trial's engine call through
   the content-addressed cache, keyed on the pe-tagged input (the key
   must include everything that varies — pe, temperature guess, the
   coupled-input differences). Repeated equilibrations of similar states
   then reuse inner trials, not just final solves.
2. **Warm-start the bracket.** Carry the previous outer iteration's
   converged pe (and a small bracket around it) into the next
   `solve_coupled` call instead of starting from the full `(-10, 17)`
   every time; fall back to the full bracket if the warm bracket fails to
   straddle. This is what the OPT-6 seam is for.
3. **Break on residual.** Add an early exit when the electron-balance
   residual is within the tolerance that the existing tests demand, in
   addition to the bracket-width criterion.

Instrument first: add a per-equilibration engine-call counter (debug
assertion or bench-only) so the claim "272 → N" is measured, not
estimated.

**Numerics honesty.** (2) and (3) can legitimately move the converged pe
in its last digits, which can propagate into reported speciation. The
acceptance bar is: every existing integration test, conservation
proptest, oracle suite and lesson replay passes **unchanged** — if any
test needs its tolerance widened, that is a finding to report and
discuss, not a change to make. Record any observed drift (max |Δpe|,
max relative species delta on the test corpus) in the commit message
even when tests pass.

**Acceptance.** Tests pass unchanged as above; engine-calls-per-
equilibration before/after and the phreeqc bench delta recorded in
Baselines; preflight green.

**Size.** Medium. **Depends on:** OPT-1, OPT-6 (and OPT-3's cache-value
`Arc`, or trial-caching will multiply deep clones).

---

## OPT-8 — One formula parser (correctness task wearing a cleanup coat)

- [x] Status: **done 2026-08-23** (Fable, 11fe338). The differential
      ran first: both parsers over all 641 formulas in the shipped
      databases — zero numeric disagreements, one structural class
      (PHREEQC pseudo-element symbols), captured as
      `FormulaDialect::{Textbook, PhreeqcMaster}` in stoich.rs. The
      65-line dbindex parser became a 10-line adapter. The unification
      then paid extra: the stoich parser reads nested parentheses and
      decimal solid-solution occupancies the deleted parser could not,
      so eleven real minerals joined the index (cobalt ammines, the
      jarosite family, the autunites) — coverage pins moved 672 → 683
      and the PLAN.md prose followed. Fuzz target ran clean.

**Why.** Two independent chemical-formula parsers exist:
`crates/kerotakis-core/src/stoich.rs:191` (`Result<Formula, ParseError>`,
handles parenthesized groups, charge, state suffixes) and
`crates/kerotakis-phreeqc/src/dbindex.rs:328`
(`Option<BTreeMap<String, f64>>`). Two parsers is a place for the same
formula to mean two things. Smaller twins: `gcd` (displacement.rs:366
u64 vs stoich.rs:546 i64) and `leading_number` (stoich.rs:276 vs
codex/prose.rs:250).

**Scope.**

- Make `stoich.rs`'s parser the only one; give `dbindex` a thin adapter
  producing its `BTreeMap` form. First **diff the two parsers' outputs
  over every formula in the shipped PHREEQC `.dat` databases** — any
  disagreement is a bug in one of them and must be understood before the
  weaker parser is deleted, not after.
- Unify the `gcd` and `leading_number` twins where crate boundaries
  allow it without inventing a new shared crate; if they don't, say so
  here and close that part as won't-fix.
- Run the existing `stoich` fuzz target (`fuzz/`) for a meaningful spell
  against the unified parser.

**Acceptance.** The pre-unification differential diff is empty or every
disagreement is resolved with a test; workspace tests + preflight green;
fuzz run clean.

**Size.** Small-medium. **Depends on:** nothing.

---

## OPT-9 — The wasm↔JS solver boundary (measure, then decide — do not build yet)

- [ ] Status: open (investigation)

**Why.** In the browser, every engine call crosses wasm→JS→wasm: input
marshalled as a JS string, the multi-KB PHREEQC report copied out of the
Emscripten heap byte-wise (`web/kerotakis.mjs` ~52-58, working around a
Chrome resizable-ArrayBuffer restriction), then JSON-serialized and
re-parsed by `serde_json` on the Rust side. OPT-7 attacks the *count* of
crossings; this task asks whether the *cost per crossing* still matters
afterwards.

**Scope.** After OPT-7 lands: measure a full lesson replay in the
browser (the wasm-lab CI job's harness is a starting point) and apportion
time between engine compute and boundary marshalling. Only if
marshalling still dominates, propose (do not yet build) the cheapest fix
— candidates: pass the report as bytes + length instead of a NUL-scan,
`postcard` instead of JSON across the bindgen boundary, or batching. Write
the numbers and the decision into this section.

**Acceptance.** A paragraph here with measurements and a
build/don't-build decision.

**Size.** Small (investigation). **Depends on:** OPT-7.

---

## Housekeeping (not tasks, just so they're written down)

- Kept from the replaced file's unnumbered work: `DbIndex` JSON
  load/save pre-parsing (f4fc2f3, skips ~50 ms of database text parsing
  per `Phreeqc::with_database()`), and the kinetics Vec hoisting
  (afbd549).

- A 6.5 MB `full.json` sits untracked at the repo root, referenced by no
  crate. Ask before deleting; it may be someone's working fixture.
- The primary machine's shell exports `CARGO_TARGET_DIR` pointing at a
  backup volume (named for a different project); any plain `cargo build`
  fails when the volume is unmounted. Ground rule 2 is the workaround.
- `.claude/worktrees/` holds near-full tree copies; any `find`/`grep`
  that doesn't exclude it triples its results.

## Baselines

Filled in by OPT-1 and updated by each task in the same commit as its
change. Machine, date and command line accompany every number.

**OPT-1 medians (2026-08-23, Linux x86-64, stable Rust):**
core — species::lookup 8 keys 401 ns, kinetics::advance 74 µs,
MixingEquilibrator::equilibrate 423 ns, ConservedLedger::from_vessel
9.7 µs, Vessel::clone 509 ns. phreeqc — Equilibrator::new (3 dbs)
123 ms, equilibrate NaCl 30 µs / HCl 27 µs / cache hit 55 µs,
dbindex::parse (wateq4f) 3.7 ms. cea — ThermoDb::parse 42 ms,
equilibrate_tp 1.0 ms, equilibrate_hp 7.5 ms.
**OPT-7:** engine calls per worst-case coupled equilibration 272 → 20
(repeat count 0).

| Bench | Baseline | After OPT-2 | OPT-3 | OPT-4 | OPT-5 | OPT-7 |
|---|---|---|---|---|---|---|
| phreeqc: coupled redox equilibration | — | | | | | |
| phreeqc: plain equilibration | — | | | | | |
| phreeqc: engine calls per equilibration | — | | | | | |
| core: `displace()` pass | — | | | | | |
| core: `.lab` replay | — | | | | | |
| cea: `equilibrate_tp` | — | | | | | |
| cea: temperature-bisecting solve | — | | | | | |
| wasm: `kerotakis-wasm` module size | — | | | | | |

## OPT-10 — ccache + ninja auto-detection for vendored C/C++ builds

Owner: kero1 (commits b77260f/18a0649 on its branch, there labelled
"OPT-8" — renumber to this at merge; this file's OPT-8 is the formula
parser, already landed). build.rs for kerotakis-phreeqc and sundials-sys
adds CMAKE_*_COMPILER_LAUNCHER=ccache and the Ninja generator ONLY when
the tools are on PATH; a machine without them builds exactly as before.
The cache is system-wide and shared across projects: /etc/ccache.conf
pins cache_dir=/mnt/volume1/ccache, max_size=5G; never set CCACHE_DIR in
env or scripts. Acceptance: `ccache -s` evidence of a near-100%-hit
second clean build of kerotakis-phreeqc, plus one configure proving the
no-ccache path still builds.

## OPT-11 — One-worker web engine (GUI-004's engine half)

Owner: keromaster. Run kerotakis-wasm and IPhreeqc in ONE module Web
Worker and reduce the number of engine calls per vessel equilibration
(ROADMAP-Webapp.md's "fewer calls first"; the per-crossing cost question
stays OPT-9's measure-then-decide). Client half landed 2026-08-24 (the
worker attaches IPhreeqc in-process via the shared bridge; see
ROADMAP-GUI.md GUI-004). Open: the engine-side call-count reduction and
the before/after measurement on the wasm path.

## Number allocation rule (2026-08-24)

A task number exists when — and only when — it has a section in THIS
file on origin/main. Allocate here first, then commit work against the
number. Recorded incident: two different "canonical restores" (70ec6fb
and 5f649ab) restored two DIFFERENT lineages of this file with
conflicting OPT-3/6/7/8/9 meanings; this version (the detailed lineage,
which all of 2026-08-24's landed work references) is authoritative, and
the 90-line lineage's bindings are void. Cross-references in
ROADMAP-Webapp.md/ROADMAP-GUI.md/PROTOCOL.md updated to match.
