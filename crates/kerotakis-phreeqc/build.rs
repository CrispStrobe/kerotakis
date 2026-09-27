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
fn has_tool(name: &str) -> bool {
    std::process::Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

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

    let with_my_basic = std::env::var_os("CARGO_FEATURE_MY_BASIC").is_some();
    let mut cfg = cmake::Config::new(&vendor);
    cfg.define("BUILD_SHARED_LIBS", "OFF")
        .define("IPHREEQC_ENABLE_MODULE", "OFF")
        .define(
            "IPHREEQC_WITH_MY_BASIC",
            if with_my_basic { "ON" } else { "OFF" },
        )
        .define("KEROTAKIS_MY_BASIC_DIR", &my_basic)
        .define("BUILD_TESTING", "OFF")
        .profile("Release");

    // IPhreeqc's thread.h guards its Windows mutex macros on `#if defined(WIN32)`
    // and otherwise includes <pthread.h>, which MSVC does not ship. `WIN32` is
    // not a compiler predefine — MSVC provides `_WIN32` — it arrives through
    // CMake's own `CMAKE_CXX_FLAGS_INIT` of "/DWIN32 /D_WINDOWS". The cmake
    // crate sets `CMAKE_CXX_FLAGS` on the command line, and a cache variable
    // given there is never initialised from *_INIT, so that default is lost and
    // three translation units die on a missing pthread.h. Appended rather than
    // defined, to leave the flags the crate computes alone.
    //
    // `NOMINMAX` is the consequence of fixing that: taking the WIN32 branch
    // pulls in <windows.h>, whose `min`/`max` macros then eat
    // `std::numeric_limits<size_t>::max()` in KeroBasicAdapter.cpp and report
    // it as "C2589: '(': illegal token on right side of '::'".
    // `WIN32_LEAN_AND_MEAN` trims the rest of that header's surface; all
    // IPhreeqc wants from it is InterlockedExchange and Sleep, both of which
    // are in the lean set.
    if target.contains("windows") && target.contains("msvc") {
        for flag in ["/DWIN32", "/DNOMINMAX", "/DWIN32_LEAN_AND_MEAN"] {
            cfg.cflag(flag).cxxflag(flag);
        }
    }

    if has_tool("ninja") {
        cfg.generator("Ninja");
    }
    if has_tool("ccache") {
        cfg.define("CMAKE_C_COMPILER_LAUNCHER", "ccache");
        cfg.define("CMAKE_CXX_COMPILER_LAUNCHER", "ccache");
    }

    let dst = cfg.build();

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
