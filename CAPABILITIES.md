# Kerotakis — Capability Tasks

Tasks that wire approved crates into the engine. Each task names the
crate, its licence, the capability it serves, and the acceptance
criterion.

## CAP-1 ✓ — Aqueous equilibrium (IPhreeqc)

Already shipped. USGS public domain. The L2 engine.

## CAP-2 — Data parallelism (rayon)

`rayon` (MIT/Apache-2.0). Multi-vessel benchmarks and batch lesson
replay. Acceptance: `cargo bench` shows wall-clock improvement on a
4-vessel benchmark without changing any answer.

## CAP-3 — Charting

Hand-roll first: the renderer-neutral `ChartObject`/`CurveObject` is
already implemented and serves USER_GRAPH, CLI sparklines, and the
Tauri/web chart layer. Adopt a library only when the hand-rolled
version proves insufficient for interactive phase diagrams.

**Explicit ban:** `plotters`' AGPL sibling must never be a dependency.
Only MIT/Apache-2.0/BSD charting libraries are eligible.

## CAP-4 — Phase diagrams (contour)

`contour` (MIT). Contour line generation from grid data for
Txy/Pxy/ternary phase diagrams. Acceptance: a binary VLE system
produces a correct boiling-point envelope.

## CAP-5 ✓ — Organic identity (InChI)

The official IUPAC InChI library (MIT since v1.07.1). Pure-Rust
`chematic-inchi` (MIT/Apache-2.0) is already integrated for the
wasm-compatible path. The native InChI library provides the
IUPAC-standard reference for cross-validation.

## CAP-6 ✓ — Physical constants (physical_constants)

`physical_constants` (MIT). CODATA recommended values. Acceptance:
Avogadro, Boltzmann, Faraday, gas constant, Planck — each matches
the CODATA 2018 adjustment to full published precision.

## CAP-7 ✓ — Exact stoichiometry (num-rational)

`num-rational` (MIT/Apache-2.0). Exact rational arithmetic for
stoichiometric coefficients. Acceptance: balancing a combustion
equation yields integer coefficients without floating-point drift.

## CAP-8 — Statistics (rand_chacha, rand_distr, statrs)

`rand_chacha` (MIT/Apache-2.0), `rand_distr` (MIT/Apache-2.0),
`statrs` (MIT). Reproducible RNG and statistical distributions for
Monte Carlo initial-rates experiments and stochastic kinetics.
Acceptance: a seeded run produces bit-identical results across
platforms.

## CAP-9 ✓ — Optimization (argmin, csv)

`argmin` (MIT/Apache-2.0). Numerical optimization for parameter
fitting (Levenberg–Marquardt, Nelder–Mead). `csv` (MIT/Unlicense)
for exporting selected-output tables. Acceptance: fit a first-order
rate constant from synthetic initial-rates data within 1%.

## CAP-10 ✓ — MY-BASIC interpreter

Already shipped. MIT. The PHREEQC BASIC adapter with 81+ callbacks.

## CAP-11 ✓ — Stiff integration (DiffSol)

Already shipped. MIT. Adaptive implicit BDF with positivity,
events, and equilibrium coupling.

## CAP-12 ✓ — Thermal chemistry (NASA CEA)

Already shipped. Apache-2.0. NASA-9 polynomials for combustion and
decomposition.

## CAP-13 — Vendor the official InChI library

Vendor the IUPAC InChI C library (MIT since v1.07.1) on the IPhreeqc
pattern: vendored source, cmake build, bindgen FFI. Add a CI check
that every registry InChIKey recomputes and matches the vendored
library's output. Acceptance: all 75 species InChIKeys verified.

## CAP-14 ✓ — Licence bar as a lint

Turn the shipping bar (PLAN.md, hardened 2026-08-23) into a
`cargo-deny` allowlist wired into `tools/preflight.sh` and CI.
`cargo-about` generates the store-facing attribution inventory.

**Status:** `deny.toml` exists and passes all four checks. `about.toml`
exists. The lint runs in `tools/provenance-lint.sh`. CAP-14 is flagged
to land early — until the bar is a lint, it is reviewer memory.
