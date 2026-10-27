//! Links CoolProp (MIT) for the off-by-default `engine` feature.
//!
//! Nothing is vendored. The library comes from one of two places, chosen by
//! environment (see PROVENANCE.md):
//!
//! * `COOLPROP_LIB_DIR` — a prebuilt CoolProp library from the pinned tag.
//!   `COOLPROP_LINK=static` selects a static archive (default: dylib).
//! * `COOLPROP_SOURCE_DIR` — a recursive clone of the pinned tag, built here
//!   with CMake as a static library.
//!
//! With `engine` on and neither set, the build fails loudly: a silently
//! stubbed engine would turn every property query into a refusal that looks
//! like a domain answer.

#[cfg(not(feature = "engine"))]
fn main() {
    // Pure-Rust build: domain checks and typed refusals only.
}

#[cfg(feature = "engine")]
const PINNED_COMMIT: &str = "442e8bd29867b11b9a5d8b6bbba7b7dcf35d2a03"; // v6.8.0

#[cfg(feature = "engine")]
fn main() {
    use std::path::PathBuf;

    println!("cargo:rerun-if-env-changed=COOLPROP_LIB_DIR");
    println!("cargo:rerun-if-env-changed=COOLPROP_LINK");
    println!("cargo:rerun-if-env-changed=COOLPROP_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=COOLPROP_ALLOW_UNPINNED");

    let target = std::env::var("TARGET").unwrap();
    if target.starts_with("wasm32") {
        panic!(
            "kerotakis-coolprop: the `engine` feature is native-only; wasm would use \
             CoolProp's official Emscripten module as a side asset (see PROVENANCE.md)"
        );
    }

    let link_cxx_runtime = |target: &str| {
        if target.contains("apple") {
            println!("cargo:rustc-link-lib=c++");
        } else if target.contains("android") {
            println!("cargo:rustc-link-lib=c++_shared");
        } else if target.contains("linux") {
            println!("cargo:rustc-link-lib=stdc++");
        }
    };

    if let Some(dir) = std::env::var_os("COOLPROP_LIB_DIR") {
        let dir = PathBuf::from(dir);
        println!("cargo:rustc-link-search=native={}", dir.display());
        let kind = std::env::var("COOLPROP_LINK").unwrap_or_else(|_| "dylib".into());
        match kind.as_str() {
            "static" => {
                println!("cargo:rustc-link-lib=static=CoolProp");
                link_cxx_runtime(&target);
            }
            "dylib" => println!("cargo:rustc-link-lib=dylib=CoolProp"),
            other => panic!("COOLPROP_LINK must be `static` or `dylib`, got `{other}`"),
        }
        return;
    }

    let Some(src) = std::env::var_os("COOLPROP_SOURCE_DIR") else {
        panic!(
            "kerotakis-coolprop `engine` feature needs COOLPROP_LIB_DIR (prebuilt) or \
             COOLPROP_SOURCE_DIR (clone of tag v6.8.0); see crates/kerotakis-coolprop/PROVENANCE.md"
        );
    };
    let src = PathBuf::from(src);
    if !src.join("CMakeLists.txt").exists() {
        panic!(
            "COOLPROP_SOURCE_DIR={} has no CMakeLists.txt",
            src.display()
        );
    }

    // Refuse an unpinned tree: the reference values in the tests are only
    // claimed for the recorded upstream commit.
    let head = std::process::Command::new("git")
        .arg("-C")
        .arg(&src)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let allow_unpinned = std::env::var_os("COOLPROP_ALLOW_UNPINNED").is_some();
    match head.as_deref() {
        Some(PINNED_COMMIT) => {}
        _ if allow_unpinned => println!(
            "cargo:warning=kerotakis-coolprop: building unpinned CoolProp source ({head:?})"
        ),
        other => panic!(
            "COOLPROP_SOURCE_DIR HEAD is {other:?}, expected pinned {PINNED_COMMIT} (v6.8.0); \
             set COOLPROP_ALLOW_UNPINNED=1 to override deliberately"
        ),
    }

    let mut cfg = cmake::Config::new(&src);
    cfg.define("COOLPROP_STATIC_LIBRARY", "ON")
        .define("COOLPROP_SHARED_LIBRARY", "OFF")
        .define("COOLPROP_OBJECT_LIBRARY", "OFF")
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .build_target("CoolProp")
        .profile("Release");
    if target.contains("windows") && target.contains("msvc") {
        cfg.cxxflag("/DNOMINMAX");
    }
    let dst = cfg.build();

    // CoolProp's static target is not installed; the archive stays in the
    // build tree (single-config generators put it at the root, multi-config
    // ones under the profile directory).
    let build = dst.join("build");
    for dir in [build.clone(), build.join("Release")] {
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
    println!("cargo:rustc-link-lib=static=CoolProp");
    link_cxx_runtime(&target);
}
