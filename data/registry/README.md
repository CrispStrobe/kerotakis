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

This document is not an app data pack. Every source is deliberately assigned to
the `build_oracle` lane and carries
`LicenseRef-Kerotakis-Legacy-Provenance-Review-Required`. DATA-003 must exclude
all of these records unless a separate provenance review changes their source
lane to `runtime`. A citation that mentions an otherwise acceptable source is
not itself that review.
