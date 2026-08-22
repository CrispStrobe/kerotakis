# MY-BASIC Migration Coordination Note

**Date:** 2026-08-22
**Author:** kero-basic agent (worktree-my-basic-completion)

## Status: Migration Complete

All seven stages of the PBasic → MY-BASIC migration are done and merged to
main. The `basic-replace.md` at the repo root is the authoritative record.

### What changed

- `PBasic.cpp` (8,350 lines) and `PBasic.h` (573 lines) deleted from
  `vendor/iphreeqc/src/phreeqcpp/`.
- `IPHREEQC_WITH_BASIC` CMake option removed; only `IPHREEQC_WITH_MY_BASIC`
  and the disabled-BASIC path remain.
- `legacy-basic-oracle` Cargo feature deleted; `my-basic` is the default.
- The iphreeqc submodule pointer updated on the `kerotakis/my-basic-preview`
  branch of `CrispStrobe/iphreeqc`.

### Impact on other work

- **No impact on non-BASIC code.** Equilibrium, speciation, minerals, gases,
  surfaces, exchange, transport, and all aqueous features are unchanged.
- **Cargo workspace:** `crates/kerotakis-phreeqc/Cargo.toml` now has
  `default = ["engine", "my-basic"]` and no `legacy-basic-oracle` feature.
  If you were passing `--features legacy-basic-oracle`, that feature no
  longer exists.
- **Submodule:** If you see the iphreeqc submodule as dirty, run
  `git submodule update --init vendor/iphreeqc` to sync.
- **Disk space:** The shared target directory at `/mnt/volume1/kerotakis/target`
  has multiple build hashes from different feature combinations. Run
  `cargo clean -p kerotakis-phreeqc` if you hit disk pressure.

### Test results

- 114 native tests pass with default features
- 20/20 wasm differential tests pass (MY-BASIC BASIC features)
- Existing wasm P0 gate (AgCl precipitation + native thermochemistry) passes
- No PBasic symbols or strings in wasm binary
