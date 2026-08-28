# Generated source registry

`registry-source-v1.json` is the reviewable DATA-002 export of the handwritten
seed declarations in `kerotakis_core::species::REGISTRY`. Do not edit it by
hand. Regenerate it with:

```sh
cargo run -p kerotakis-registry-export -- data/registry/registry-source-v1.json
```

CI performs both a typed, every-field comparison against the Rust declarations
and a byte-for-byte regeneration check.

The external-source growth path is scoped in
[BREADTH.md](../../BREADTH.md):
`BRD-003` owns the common fetch/quarantine/promotion adapter; `BRD-010` and
`BRD-011` own PubChem/ChEBI candidates; `BRD-013` owns USDA-derived material
recipe candidates; and `BRD-060` owns reviewed COD crystal records. None of
those adapters may write promoted records directly. Their output remains in a
quarantine/build-oracle lane until per-field licence, identity, provenance and
scientific review explicitly promotes it.

The shared `kerotakis_data` adapter contract now enforces the first promotion
boundary: a raw snapshot has a revision and SHA-256 manifest; quarantine JSON
is deterministic; every candidate field retains its exact source path and
licence; a reviewed field/licence allowlist produces a report rather than a
registry mutation; and multiple records sharing one identity key produce an
explicit conflict report. Source-specific adapters must use this contract and
commit reviewable fixtures. Fetching remains outside builds and runtime.

Pinned adapter fixtures use this layout (the synthetic contract fixture lives
under `crates/kerotakis-data/tests/fixtures/quarantine/synthetic-v1`):

```text
<adapter-id>/
├── manifest.json
├── raw/<snapshot artifact>
├── candidates-old.json
├── candidates-new.json
└── policy.json
```

Review them offline with:

```sh
cargo run -p kerotakis-data --bin quarantine-review -- verify manifest.json raw/snapshot.json
cargo run -p kerotakis-data --bin quarantine-review -- canonicalize candidates-new.json
cargo run -p kerotakis-data --bin quarantine-review -- review candidates-new.json policy.json
cargo run -p kerotakis-data --bin quarantine-review -- diff candidates-old.json candidates-new.json
```

All output goes to stdout for explicit inspection/check-in. The tool has no
promotion or registry-write command.

This document is not an app data pack. Every source is deliberately assigned to
the `build_oracle` lane and carries
`LicenseRef-Kerotakis-Legacy-Provenance-Review-Required`. DATA-003 must exclude
all of these records unless a separate provenance review changes their source
lane to `runtime`. A citation that mentions an otherwise acceptable source is
not itself that review.
