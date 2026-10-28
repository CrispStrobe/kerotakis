# Does FeOs compile to wasm32? (manifest reading only, not built)

Read from the published manifests of feos 0.10.1, feos-core 0.10.1 and
quantity 0.14.0 (2026-10-27). Nothing was compiled.

With `default-features = false, features = ["pcsaft"]` the dependency tree is
pure Rust: feos-core, nalgebra 0.35, num-dual 0.14, num-traits, quantity 0.14,
indexmap 2, itertools 0.15, serde/serde_json, thiserror. None of these has a
build script that compiles C, and feos's published Cargo.lock contains no
`getrandom` package, so no wasm-specific RNG backend selection is needed.

Things that would break or bloat wasm32-unknown-unknown, all avoided here:
- `rusqlite` (feos-core, optional, `bundled` = compiles SQLite C) — not enabled.
- `rayon` (optional) — threads; not enabled.
- `dft` feature (ndarray 0.17, petgraph, num-dual/ndarray, feos-dft) — ndarray
  itself is wasm-friendly, but the DFT stack is large; not enabled.

Verdict: expected to compile for wasm32-unknown-unknown with the `pcsaft`
feature only — UNVERIFIED. Confirm on the hosted runner with
`cargo build -p kerotakis-feos --features feos --target wasm32-unknown-unknown`.
kerotakis-wasm does not depend on this crate, so the shipped wasm is unchanged.
