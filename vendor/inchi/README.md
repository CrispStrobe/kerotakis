# Vendored IUPAC InChI library (CAP-13)

## Source

- **Library**: IUPAC InChI, version 1.07.1+
- **Licence**: MIT (since v1.07.1)
- **URL**: https://github.com/nickvdg/InChI
- **Upstream wasm build**: proven by InChI project

## Setup

Download the InChI source:

```sh
cd vendor/inchi
curl -sL https://github.com/nickvdg/InChI/archive/refs/tags/v1.07.1.tar.gz | tar xz
mv InChI-1.07.1 src
```

## Build

The build follows the IPhreeqc pattern:
- `build.rs` compiles the InChI C library via cmake
- `bindgen` generates FFI bindings
- Rust wrappers provide safe `inchi_from_smiles()` and `inchikey_from_inchi()`

## CI check (CAP-13)

Every registry InChIKey must recompute and match the vendored library's output:

```sh
cargo test -p kerotakis-org --test inchi_parity
```
