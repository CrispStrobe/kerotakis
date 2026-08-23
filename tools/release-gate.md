# REL-001: Release Gate

A release is blocked unless ALL of the following pass:

## Automated checks

- [ ] `cargo deny check` — all four categories (advisories, bans, licences, sources)
- [ ] `tools/provenance-lint.sh` — all checksums match, no oracle leakage
- [ ] `cargo test --workspace` — full native test suite
- [ ] Wasm build + `test-iphreeqc-wasm.mjs` + `test-iphreeqc-wasm-basic.mjs`
- [ ] `cargo run -p kerotakis-data --bin compile-registry` — pack round-trips
- [ ] `data/sbom.cdx.json` is up to date
- [ ] `data/inventory.json` matches `Cargo.lock`
- [ ] No `FIXME` or `TODO` in committed code without an issue reference

## Manual review

- [ ] NOTICE file lists every shipped third-party component
- [ ] CONTRIBUTING.md PR checklist is current
- [ ] basic-replace.md status is "migration complete"
- [ ] No PBasic symbols in any binary (`strings` check)
- [ ] ROADMAP blockers for this release are resolved
- [ ] Changelog entry written

## Per-platform

- [ ] Linux x86_64 binary builds and passes `cargo test`
- [ ] macOS aarch64 binary builds and passes `cargo test`
- [ ] Wasm module builds (tools/build-iphreeqc-wasm.sh)
- [ ] Android builds (when target is added)
- [ ] iOS builds (when target is added)
