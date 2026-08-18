//! # kerotakis-phreeqc
//!
//! Safe Rust interface to IPhreeqc (USGS, public domain) — the L2
//! aqueous-equilibrium engine: speciation, mineral saturation, gas
//! partitioning, redox and ionic strength, solved simultaneously from
//! thermodynamic databases embedded in this binary.
//!
//! The entire interaction is string-in / value-out and never touches the
//! filesystem (`LoadDatabaseString`, `RunString`, selected-output strings) —
//! the property that makes the engine portable to every target (PLAN.md,
//! "PHREEQC runs on a phone").

use std::ffi::{CStr, CString};

mod aqueous;
pub use aqueous::PhreeqcEquilibrator;

mod ffi {
    use std::os::raw::{c_char, c_int};

    // IPhreeqc's flat C API (src/IPhreeqc.h in the vendored source). Only the
    // string-based surface is declared — the VAR union API is deliberately
    // avoided.
    extern "C" {
        pub fn CreateIPhreeqc() -> c_int;
        pub fn DestroyIPhreeqc(id: c_int) -> c_int;
        pub fn LoadDatabaseString(id: c_int, input: *const c_char) -> c_int;
        pub fn RunString(id: c_int, input: *const c_char) -> c_int;
        pub fn GetErrorString(id: c_int) -> *const c_char;
        pub fn SetOutputFileOn(id: c_int, value: c_int) -> c_int;
        pub fn SetErrorFileOn(id: c_int, value: c_int) -> c_int;
        pub fn SetLogFileOn(id: c_int, value: c_int) -> c_int;
        pub fn SetDumpFileOn(id: c_int, value: c_int) -> c_int;
        pub fn SetSelectedOutputFileOn(id: c_int, value: c_int) -> c_int;
        pub fn SetSelectedOutputStringOn(id: c_int, value: c_int) -> c_int;
        pub fn GetSelectedOutputStringLineCount(id: c_int) -> c_int;
        pub fn GetSelectedOutputStringLine(id: c_int, n: c_int) -> *const c_char;
    }
}

/// Thermodynamic databases embedded in the binary (all USGS User Rights
/// Notice, distributed with IPhreeqc).
///
/// Stored as bytes: some upstream files carry Latin-1 characters in comments
/// (e.g. the degree sign in pitzer.dat), so they are not valid UTF-8. PHREEQC
/// itself is encoding-agnostic.
pub mod databases {
    /// Core aqueous set — most teaching chemistry.
    pub const PHREEQC: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/phreeqc.dat");
    /// Extended natural-water species (incl. Ag, trace metals).
    pub const WATEQ4F: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/wateq4f.dat");
    /// Metals, complexation, sorption.
    pub const MINTEQ_V4: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/minteq.v4.dat");
    /// Pitzer model — brines, high ionic strength.
    pub const PITZER: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/pitzer.dat");
}

#[derive(Debug, thiserror::Error)]
pub enum PhreeqcError {
    #[error("could not create IPhreeqc instance")]
    Create,
    #[error("input contained an interior NUL byte")]
    Nul,
    /// PHREEQC refused or failed — honest, first-class, never a crash
    /// (PLAN.md: "Solver failure is a first-class result").
    #[error("PHREEQC: {0}")]
    Engine(String),
}

/// One IPhreeqc instance with a loaded database. All file output is disabled
/// at construction; results are read from the selected-output string.
pub struct Phreeqc {
    id: i32,
}

// IPhreeqc instances are independent; the id is only used from the owning
// wrapper.
unsafe impl Send for Phreeqc {}

impl Phreeqc {
    /// Create an instance and load a thermodynamic database from memory
    /// (see [`databases`]).
    pub fn with_database(database: impl AsRef<[u8]>) -> Result<Self, PhreeqcError> {
        let id = unsafe { ffi::CreateIPhreeqc() };
        if id < 0 {
            return Err(PhreeqcError::Create);
        }
        let this = Phreeqc { id };
        unsafe {
            ffi::SetOutputFileOn(id, 0);
            ffi::SetErrorFileOn(id, 0);
            ffi::SetLogFileOn(id, 0);
            ffi::SetDumpFileOn(id, 0);
            ffi::SetSelectedOutputFileOn(id, 0);
        }
        let db = CString::new(database.as_ref()).map_err(|_| PhreeqcError::Nul)?;
        let errors = unsafe { ffi::LoadDatabaseString(id, db.as_ptr()) };
        if errors != 0 {
            return Err(PhreeqcError::Engine(this.error_string()));
        }
        // Loading a database resets the selected-output string flag
        // (IPhreeqc.cpp clears SelectedOutputStringOn in its load path), so
        // it must be enabled after the load — and again before every run for
        // robustness.
        unsafe {
            ffi::SetSelectedOutputStringOn(id, 1);
        }
        Ok(this)
    }

    /// Run a PHREEQC input block. On success, selected output (if the input
    /// requested any) is available via [`Self::selected_output`].
    pub fn run(&mut self, input: &str) -> Result<(), PhreeqcError> {
        let input = CString::new(input).map_err(|_| PhreeqcError::Nul)?;
        let errors = unsafe {
            ffi::SetSelectedOutputStringOn(self.id, 1);
            ffi::RunString(self.id, input.as_ptr())
        };
        if errors != 0 {
            return Err(PhreeqcError::Engine(self.error_string()));
        }
        Ok(())
    }

    /// The selected-output block of the last run as rows of tab-separated
    /// columns: first row is the headings.
    pub fn selected_output(&self) -> Vec<Vec<String>> {
        let count = unsafe { ffi::GetSelectedOutputStringLineCount(self.id) };
        (0..count)
            .filter_map(|n| {
                let ptr = unsafe { ffi::GetSelectedOutputStringLine(self.id, n) };
                if ptr.is_null() {
                    return None;
                }
                let line = unsafe { CStr::from_ptr(ptr) }.to_string_lossy();
                Some(
                    line.split('\t')
                        .map(|cell| cell.trim().to_string())
                        .collect(),
                )
            })
            .collect()
    }

    /// Value of a named selected-output column in the last data row.
    pub fn last_value(&self, column: &str) -> Option<f64> {
        let rows = self.selected_output();
        let idx = rows.first()?.iter().position(|h| h == column)?;
        rows.last()?.get(idx)?.parse().ok()
    }

    fn error_string(&self) -> String {
        let ptr = unsafe { ffi::GetErrorString(self.id) };
        if ptr.is_null() {
            return "unknown error".to_string();
        }
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}

impl Drop for Phreeqc {
    fn drop(&mut self) {
        unsafe {
            ffi::DestroyIPhreeqc(self.id);
        }
    }
}
