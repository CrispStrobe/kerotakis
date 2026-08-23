# Generated source registry

`registry-source-v1.json` is the reviewable DATA-002 export of the handwritten
seed declarations in `kerotakis_core::species::REGISTRY`. Do not edit it by
hand. Regenerate it with:

```sh
cargo run -p kerotakis-registry-export -- data/registry/registry-source-v1.json
```

CI performs both a typed, every-field comparison against the Rust declarations
and a byte-for-byte regeneration check.

This document is not an app data pack. Every source is deliberately assigned to
the `build_oracle` lane and carries
`LicenseRef-Kerotakis-Legacy-Provenance-Review-Required`. DATA-003 must exclude
all of these records unless a separate provenance review changes their source
lane to `runtime`. A citation that mentions an otherwise acceptable source is
not itself that review.
