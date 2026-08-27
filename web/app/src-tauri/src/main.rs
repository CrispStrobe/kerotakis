//! The desktop binary: a shim over the shell library.
//!
//! Everything lives in `lib.rs` because iOS links a staticlib rather than
//! a binary (see `run`). Keeping the desktop entry point this thin is what
//! guarantees the two platforms start the same way.

// No console window behind the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app::run()
}
