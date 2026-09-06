# Kerotakis — Optimization tasks

> Finished work is not listed here. What landed, and what it taught us, is in
> [HISTORY.md](HISTORY.md). Task numbers are never renumbered and never reused.

Findings from a full-workspace performance survey, **2026-08-23** (line
numbers rot — **re-verify each reference against the current tree
before editing**; the claim is the anchor, the number is a hint). The
debt was a handful of concrete hot-path problems, almost all in
`kerotakis-phreeqc/src/aqueous.rs` and `kerotakis-cea/src/gibbs.rs`,
plus no way to measure an improvement.

**The ordering rule: no optimization lands without a before/after
number.** OPT-1 made that number obtainable; the same discipline
applies to every open task below. **A completion checkbox requires its
Acceptance evidence in the marking commit — a checkbox without it is a
claim, not a status** (this file was mis-restored twice from stale
replacements that re-bound task numbers to different topics; see
`## Number allocation rule` below for the canonical resolution).

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
3. **Main moves only by PR; the full gate is CI's job.** Enforced by
   branch protection since 2026-08-25 (owner-directed, the day the box
   near-OOMed running stacked local full gates on 7.6 GiB of RAM):
   five required checks (`Full preflight gate`, native tests ×2, wasm
   bench, browser demo), `enforce_admins` on, auto-merge enabled,
   merged branches auto-delete. The flow: branch → push →
   `gh pr create` → `gh pr merge --auto --merge`. Locally,
   `tools/preflight.sh --light` (fmt + clippy + featureless build)
   before pushing a branch is the courtesy gate — cheap enough for
   this box; the FULL local run is no longer required for anything and
   stacking it across sessions is forbidden (it also now takes a
   machine-wide flock). History: the rule was local-full-gate from
   2026-08-24 (an ungated merge had landed an inconsistent iphreeqc
   pin) until protection made CI the enforcer. Never chain
   "check; push" with semicolons — that pushes on red.
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

OPT-3/4/5 touch disjoint files and can run in parallel worktrees. (All
of OPT-1 through OPT-8 are now done — see `## Completed OPT tasks`.)

---

**Second replacement (2026-08-24 audit).** This file was replaced again
on 2026-08-23 (9a88ba7, then built upon through e51d870), re-binding
OPT-6 to database pre-parsing and OPT-7 to the `solve_once`
decomposition. Restored 2026-08-24; the replacement's real work is kept
and recorded under the canonical numbers it actually corresponds to,
each claim verified against the tree. Task numbers are stable
identifiers and are never re-bound.

## OPT-9 — The wasm↔JS solver boundary (measure, then decide — do not build yet)

- [ ] Status: open (investigation) — **verdict already recorded** under
      OPT-11's baseline measurement below: **do not build.** The
      checkbox is left open here because the Acceptance paragraph was
      written into that companion section instead of this one; nothing
      substantive remains unless a future domain multiplies calls.

**Why.** In the browser, every engine call crosses wasm→JS→wasm: input
marshalled as a JS string, the multi-KB PHREEQC report copied out of the
Emscripten heap byte-wise (`web/kerotakis.mjs` ~52-58, working around a
Chrome resizable-ArrayBuffer restriction), then JSON-serialized and
re-parsed by `serde_json` on the Rust side. OPT-7 (done) attacked the
*count* of crossings; this task asks whether the *cost per crossing*
still matters afterwards.

**Verdict (2026-08-24).** Measured post-OPT-6/7 at 139 engine crossings
over the full 27-lesson corpus, 3,328 ms total in the hook (~24 ms/call
— PHREEQC compute, not crossing overhead), 56 KB in / 917 KB out. That
leaves no marshalling case: **do not build.** Candidates if this ever
needs revisiting — bytes+length instead of a NUL-scan, `postcard`
instead of JSON across the bindgen boundary, batching. Re-run
`tools/measure-wasm-calls.mjs` if a future domain multiplies calls
(per-cell transport, fine-grained kinetics).

**Acceptance.** A paragraph here with measurements and a
build/don't-build decision (satisfied above). **Size.** Small
(investigation). **Depends on:** OPT-7 (done, see `HISTORY.md`).

---

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

### OPT-11 baseline (2026-08-24) — and OPT-9's verdict

`tools/measure-wasm-calls.mjs` (two-wasm pairing in node, counting hook
wrapper) over the full 27-lesson corpus, post-OPT-6/7: 139 engine
crossings total (median step 1–2 per lesson), 3,328 ms in the hook
(~24 ms/call, solver compute not crossing overhead), 56 KB in / 917 KB
out (~6.6 KB/call, the full-report JSON). Outliers: counting-in-fives
21 (15 on one cold KMnO4 add — redox bisection converging),
spannungsreihe 13, titration 12 (one per increment, inherent). **OPT-9's
decision: do not build** — 139 crossings × ~7 KB against 3.3 s of
solver compute leaves no marshalling case. Full per-lesson record: see
`HISTORY.md`. OPT-11's remaining scope is at most the cold-redox trial
count (already warm-started), not worth surgery ahead of feature work.

## Number allocation rule (2026-08-24)

A task number exists when — and only when — it has a section in THIS
file on origin/main. Allocate here first, then commit work against the
number. Recorded incident: two different "canonical restores" (70ec6fb
and 5f649ab) restored two DIFFERENT lineages of this file with
conflicting OPT-3/6/7/8/9 meanings; this version (the detailed lineage,
which all of 2026-08-24's landed work references) is authoritative, and
the 90-line lineage's bindings are void. Cross-references in
ROADMAP-Webapp.md/ROADMAP-GUI.md/PROTOCOL.md updated to match.

---

## Completed OPT tasks

- **OPT-1** — added criterion benches to the core/phreeqc/cea crates
  and recorded baseline medians. Done 2026-08-23. `9a88ba7` + `2c753aa`
  (kero1). See `HISTORY.md`.
- **OPT-2** — added a workspace `[profile.release]`, a `wasm-opt -Oz`
  pass, and moved shared deps (`postcard`, `toml`) into
  `[workspace.dependencies]`. Done 2026-08-23. (kero1) See
  `HISTORY.md`.
- **OPT-3** — replaced the anonymous cache 5-tuple with a named
  `Rc`-wrapped struct and hoisted four `env::var` reads into
  `OnceLock`s. Done 2026-08-24. `87e4608` (kero1; reviewed by Fable).
  See `HISTORY.md`.
- **OPT-4** — indexed the species-registry lookup with a
  `OnceLock<HashMap>` replacing a linear scan; `REGISTRY` later made
  `static` by CAP-21's codegen. Done. See `HISTORY.md`.
- **OPT-5** — replaced the CEA Newton loop's per-iteration
  `Vec<Vec<f64>>` allocation with one flat row-major matrix allocated
  once; `equilibrate_tp` −29%, `equilibrate_hp` −14%. Done 2026-08-24.
  (Opus) See `HISTORY.md`.
- **OPT-6** — decomposed the 968-line `solve_once` into six named
  move-only private methods plus a `SolveSetup` struct. Done
  2026-08-23. `e51d870` (kero1). See `HISTORY.md`.
- **OPT-7** — cached redox-bisection trials, warm-started the pe
  bracket from the previous converged value, and added a
  residual-tolerance break; 272 → 20 engine calls on the worst coupled
  case, all tests unchanged. Done 2026-08-23. (Fable) See `HISTORY.md`.
- **OPT-8** — unified the two chemical-formula parsers behind
  `stoich.rs`, differentially checked against all 641 shipped-database
  formulas first; eleven real minerals gained coverage as a side
  effect. Done 2026-08-23. `11fe338` (Fable). See `HISTORY.md`.

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
**OPT-2:** wasm module 1.9 MB raw / 572 KB gzipped (`tools/bundle-budget.sh`).
**OPT-5:** `equilibrate_tp` 642 µs → 456 µs (−29%), `equilibrate_hp`
12.4 ms → 10.7 ms (−14%).
**OPT-7:** engine calls per worst-case coupled equilibration 272 → 20
(repeat count 0).
**OPT-9/OPT-11:** 139 engine crossings over the 27-lesson corpus,
3,328 ms in the hook, 56 KB in / 917 KB out.

(The former per-bench baseline/after table is now fully populated only
in `HISTORY.md`'s underlying commits — every column corresponds to a
task above marked done.)
