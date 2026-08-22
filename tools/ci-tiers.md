# CI-002: Validation Tiers

## Fast (PR gate, < 2 min)

Runs on every PR and push to main.

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test -p kerotakis-core --lib
cargo test -p kerotakis-data
cargo deny check
tools/provenance-lint.sh
```

## Full (merge gate, < 15 min)

Runs before merging to main.

```bash
cargo test --workspace
cargo test -p kerotakis-phreeqc --features my-basic  # full BASIC suite
IPHREEQC_BASIC_MODE=my-basic node tools/test-iphreeqc-wasm.mjs ...
IPHREEQC_BASIC_MODE=my-basic node tools/test-iphreeqc-wasm-basic.mjs ...
cargo run -p kerotakis-data --bin compile-registry -- ...  # pack round-trip
```

## Oracle (on demand, licensed environment)

Runs against stock PHREEQC, Cantera, Reaktoro. Never in CI by default.

```bash
# Uses tools/oracle/ directory
# Results go to tools/oracle/output/
# Approved facts promoted to tests/oracle/expected/
```

## Platform matrix

| Target | Fast | Full | Oracle |
|--------|------|------|--------|
| Linux x86_64 | ✓ | ✓ | ✓ |
| macOS aarch64 | ✓ | ✓ | — |
| wasm32 (Emscripten) | — | ✓ | — |
| Android aarch64 | ✓ | — | — |
| iOS aarch64 | ✓ | — | — |
