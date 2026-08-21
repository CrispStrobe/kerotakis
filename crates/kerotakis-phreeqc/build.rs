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
    let my_basic = manifest.join("../../vendor/my-basic");
    if !vendor.join("CMakeLists.txt").exists() {
        panic!(
            "vendor/iphreeqc is missing — run `git submodule update --init` \
             to fetch the IPhreeqc source"
        );
    }
    // The cmake helper does not make Cargo watch the vendored sources for us.
    // Without this, a C++ edit can leave a stale static archive linked into
    // tests until some unrelated Rust build input changes.
    println!("cargo:rerun-if-changed={}", vendor.display());

    let target = std::env::var("TARGET").unwrap();
    if target.starts_with("wasm32") {
        // Track B: the wasm engine is a separate Emscripten side module, not
        // linked into the Rust wasm binary. Nothing to build here.
        println!("cargo:warning=kerotakis-phreeqc: wasm target uses the Emscripten side module (tools/build-iphreeqc-wasm.sh), skipping native build");
        return;
    }

    let with_basic = std::env::var_os("CARGO_FEATURE_LEGACY_BASIC_ORACLE").is_some();
    let with_my_basic = !with_basic && std::env::var_os("CARGO_FEATURE_MY_BASIC_PREVIEW").is_some();
    let dst = cmake::Config::new(&vendor)
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("IPHREEQC_ENABLE_MODULE", "OFF")
        .define("IPHREEQC_WITH_BASIC", if with_basic { "ON" } else { "OFF" })
        .define(
            "IPHREEQC_WITH_MY_BASIC",
            if with_my_basic { "ON" } else { "OFF" },
        )
        .define("KEROTAKIS_MY_BASIC_DIR", &my_basic)
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
