# kerotakis-coolprop — provenance

| Field | Value |
|---|---|
| Upstream | CoolProp — https://github.com/CoolProp/CoolProp |
| Licence | MIT (`LICENSE` in the upstream repository root). Compatible with Kerotakis' AGPL-3.0-or-later; the MIT notice must travel with any binary that links it. Released apps do not: the `engine` feature is off by default, so the repository `NOTICE` (which lists what released binaries contain) deliberately omits CoolProp. A build that enables `engine` must add `CoolProp — MIT, Copyright (c) 2010-2024 Ian H. Bell, Jorrit Wronski, Sylvain Quoilin, Vincent Lemort and the CoolProp contributors` to its own notice. |
| Pinned version | tag `v6.8.0`, commit `442e8bd29867b11b9a5d8b6bbba7b7dcf35d2a03` (resolved with `git ls-remote --tags` on 2026-10-27). build.rs refuses a source tree whose `HEAD` is a different commit unless `COOLPROP_ALLOW_UNPINNED=1`. |
| Interface | CoolProp's stable C API (`include/CoolPropLib.h`): `PropsSI`, `get_global_param_string`. Declared by hand in `src/ffi.rs`; no bindgen. |
| Equation of state used | `HEOS::Water` = IAPWS-95 (W. Wagner, A. Pruß, *J. Phys. Chem. Ref. Data* 31, 387–535 (2002), doi:10.1063/1.1461829; IAPWS R6-95(2018)). |
| Vendored bytes | **None.** No CoolProp source, data or binary is committed to this repository. |

## How the library is supplied (build.rs, `engine` feature only)

1. **Prebuilt library** — set `COOLPROP_LIB_DIR` to a directory containing
   `libCoolProp.{so,dylib,a}` / `CoolProp.lib` built from the pinned tag
   (e.g. the upstream release shared library). Set `COOLPROP_LINK=static` if
   that directory holds a static archive (default `dylib`).
2. **Pinned-source build** — set `COOLPROP_SOURCE_DIR` to a recursive clone of
   the pinned tag (`git clone --recursive --branch v6.8.0 --depth 1
   https://github.com/CoolProp/CoolProp`). build.rs runs CMake with
   `COOLPROP_STATIC_LIBRARY=ON` and links the static archive plus the C++
   runtime.

Neither variable set with `engine` on is a build error, never a silent stub.

## Wasm (out of scope for this PR)

The Rust wasm build never enables `engine`. Upstream publishes an official
Emscripten build (`wrappers/Javascript`, producing `coolprop.js` +
`coolprop.wasm` that exposes `PropsSI`). A later PR could load that module as a
separate side asset — the same shape as the IPhreeqc Emscripten side module
(Track B in PLAN.md) — and call it from the web client with the same domain
gate as `src/domain.rs`, which is pure Rust and already wasm-safe.
