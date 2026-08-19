//! Builds the vendored IPhreeqc (public domain, USGS) as a static library.
//!
//! The vendored source is the git submodule at `vendor/iphreeqc`
//! (github.com/phreeqc-dev/iphreeqc). The web target does not go through this
//! build: on wasm the engine is an Emscripten side module (Track B in
//! PLAN.md), produced by `tools/build-iphreeqc-wasm.sh`.

#[cfg(not(feature = "engine"))]
fn main() {
    // Cache-only build: nothing to compile, nothing to link.
}

#[cfg(feature = "engine")]
use std::path::PathBuf;

#[cfg(feature = "engine")]
fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = manifest.join("../../vendor/iphreeqc");
    if !vendor.join("CMakeLists.txt").exists() {
        panic!(
            "vendor/iphreeqc is missing — run `git submodule update --init` \
             to fetch the IPhreeqc source"
        );
    }

    let target = std::env::var("TARGET").unwrap();
    if target.starts_with("wasm32") {
        // Track B: the wasm engine is a separate Emscripten side module, not
        // linked into the Rust wasm binary. Nothing to build here.
        println!("cargo:warning=kerotakis-phreeqc: wasm target uses the Emscripten side module (tools/build-iphreeqc-wasm.sh), skipping native build");
        return;
    }

    let dst = cmake::Config::new(&vendor)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("IPHREEQC_ENABLE_MODULE", "OFF")
        .define("BUILD_TESTING", "OFF")
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=IPhreeqc");

    // IPhreeqc is C++; link the platform C++ runtime.
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("android") {
        println!("cargo:rustc-link-lib=c++_shared");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
