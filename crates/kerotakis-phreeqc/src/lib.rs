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

#[cfg(feature = "engine")]
use std::ffi::{CStr, CString};

pub mod acceptance;
mod aqueous;
pub mod dbindex;
pub mod derived;
pub mod enthalpy;
pub mod pourbaix;
pub use aqueous::{
    CacheData, CacheEntry, PathOutcome, PathResult, PhreeqcEquilibrator, SolveHook, SolveOutput,
};

#[cfg(feature = "engine")]
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
        pub fn GetSelectedOutputString(id: c_int) -> *const c_char;
        pub fn GetUserGraphJson(id: c_int) -> *const c_char;
        pub fn GetSelectedOutputStringLineCount(id: c_int) -> c_int;
        pub fn GetSelectedOutputStringLine(id: c_int, n: c_int) -> *const c_char;
        pub fn GetSpeciesDeltaH(id: c_int, name: *const c_char, delta_h: *mut f64) -> c_int;
        pub fn SetOutputStringOn(id: c_int, value: c_int) -> c_int;
        pub fn GetOutputString(id: c_int) -> *const c_char;
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
    /// Metals, complexation, sorption. PRIVATE on purpose: everything
    /// goes through [`minteq_v4()`], which adds the reviewed lactate
    /// definition. Reading these bytes directly would give a caller a
    /// database the engine is not running.
    const MINTEQ_V4: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/minteq.v4.dat");
    /// Pitzer model — brines, high ionic strength.
    pub const PITZER: &[u8] = include_bytes!("../../../vendor/iphreeqc/database/pitzer.dat");

    /// One reviewed species added to minteq.v4: lactate.
    ///
    /// A lactic fermentation is the commonest acid a kitchen makes, and
    /// none of the three databases this lab loads defines its anion — so
    /// the carboxylic proton of the acid the yoghurt just made was absent
    /// from the pH, and the vessel was refused a characterisation
    /// altogether rather than report a pH missing its only acid.
    ///
    /// The constant is llnl-organics' own — that file writes the
    /// dissociation `C3H6O3 = C3H5O3- + H+` at `log_k -3.8629`, which is
    /// pKa 3.86 and is lactic acid's measured value — sign-flipped into
    /// the association direction minteq.v4 writes its acids in. Alkalinity
    /// and formula weight follow minteq's own `Acetate  Acetate-  1
    /// 59.045` line; lactate is likewise monoprotic, at 89.07 g/mol.
    ///
    /// **No enthalpy, deliberately.** llnl-organics states `-delta_h
    /// +164.070 kcal/mol` for this reaction, which is 686 kJ/mol where
    /// minteq's acetate dissociation is 0.41 — that column is LLNL's
    /// formation-from-basis convention, not a dissociation enthalpy, and
    /// carrying it across would have handed the heat balance a number
    /// three orders of magnitude wrong. A step that moves lactate declines
    /// its heat by name instead, which is the honest answer while nobody
    /// has reviewed one.
    ///
    /// This is the same trade `derived::FOREIGN_POSABLE` already makes for
    /// phases: take the log K, leave the enthalpy, and say so.
    const LACTATE_EXTENSION: &[u8] = b"
SOLUTION_MASTER_SPECIES
    Lactate   Lactate-  1   89.07   89.07
SOLUTION_SPECIES
    Lactate- = Lactate-
        log_k 0
    H+ + Lactate- = H(Lactate)
        log_k 3.8629
";

    /// Byte offset of the final `END` line, which is where a database
    /// stops being read. `None` when the file has none, in which case the
    /// end of the file is the right place after all.
    fn find_last_end(text: &[u8]) -> Option<usize> {
        let mut at = None;
        let mut line_start = 0usize;
        for (i, byte) in text.iter().enumerate() {
            if *byte == b'\n' {
                let line = &text[line_start..i];
                let trimmed: &[u8] = {
                    let s = line
                        .iter()
                        .position(|c| !c.is_ascii_whitespace())
                        .unwrap_or(line.len());
                    let e = line
                        .iter()
                        .rposition(|c| !c.is_ascii_whitespace())
                        .map(|p| p + 1)
                        .unwrap_or(s);
                    &line[s..e]
                };
                if trimmed.eq_ignore_ascii_case(b"END") {
                    at = Some(line_start);
                }
                line_start = i + 1;
            }
        }
        at
    }

    /// minteq.v4 as this lab runs it: the vendored file plus
    /// [`LACTATE_EXTENSION`].
    ///
    /// Everything that loads or PARSES the database goes through here, so
    /// the engine, the derived index, the element bookings and the
    /// provenance string all describe the same database. Reading the
    /// vendored bytes anywhere else would give the ledger an element the
    /// engine has and it does not.
    pub fn minteq_v4() -> &'static [u8] {
        use std::sync::OnceLock;
        static EXTENDED: OnceLock<Vec<u8>> = OnceLock::new();
        EXTENDED.get_or_init(|| {
            // BEFORE the file's trailing `END`, not after it. PHREEQC stops
            // reading a database at `END`, so an appended block is not a
            // block the engine ignores loudly — it is one it never sees.
            // The element went in as a 0.0038 mol total and came back as
            // exactly 0.0, and the acid's mass left the ledger with it.
            let text = MINTEQ_V4;
            let insert_at = find_last_end(text).unwrap_or(text.len());
            let mut bytes = Vec::with_capacity(text.len() + LACTATE_EXTENSION.len());
            bytes.extend_from_slice(&text[..insert_at]);
            bytes.extend_from_slice(LACTATE_EXTENSION);
            bytes.extend_from_slice(&text[insert_at..]);
            bytes
        })
    }
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
    #[error("species {0:?} is not present in the loaded PHREEQC database")]
    UnknownSpecies(String),
    #[error("PHREEQC returned a non-finite enthalpy for species {0:?}")]
    NonFiniteSpeciesDeltaH(String),
}

#[cfg(feature = "engine")]
/// One IPhreeqc instance with a loaded database. All file output is disabled
/// at construction; results are read from the selected-output string.
pub struct Phreeqc {
    id: i32,
}

/// Renderer-neutral data produced by PHREEQC `USER_GRAPH` blocks.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserGraphData {
    pub charts: Vec<UserGraphChart>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserGraphChart {
    pub user_number: i32,
    pub title: String,
    pub axis_titles: Vec<String>,
    pub series: Vec<UserGraphSeries>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct UserGraphSeries {
    pub id: String,
    pub color: String,
    pub symbol: String,
    pub line_width: f64,
    pub symbol_size: f64,
    pub y_axis: i32,
    pub points: Vec<[f64; 2]>,
}

// IPhreeqc instances are independent; the id is only used from the owning
// wrapper.
#[cfg(feature = "engine")]
unsafe impl Send for Phreeqc {}

#[cfg(feature = "engine")]
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
            // Full run output to memory: the "Distribution of species"
            // block is the expert register's raw material.
            ffi::SetOutputStringOn(id, 1);
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

    /// The selected-output stream exactly as emitted by IPhreeqc.
    ///
    /// Unlike [`Self::selected_output`], this preserves tabs, leading spaces,
    /// and embedded newlines. It is useful for PHREEQC programs that generate
    /// another PHREEQC input through `USER_PUNCH`.
    pub fn selected_output_string(&self) -> String {
        let ptr = unsafe { ffi::GetSelectedOutputString(self.id) };
        if ptr.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Renderer-neutral chart metadata and points emitted by `USER_GRAPH`.
    /// A CLI can serialize this directly; a Tauri frontend can render it with
    /// any native or web plotting library without coupling PHREEQC to a GUI.
    pub fn user_graph_data(&self) -> Result<UserGraphData, PhreeqcError> {
        let ptr = unsafe { ffi::GetUserGraphJson(self.id) };
        if ptr.is_null() {
            return Ok(UserGraphData::default());
        }
        let json = unsafe { CStr::from_ptr(ptr) }.to_string_lossy();
        serde_json::from_str(&json).map_err(|error| {
            PhreeqcError::Engine(format!("invalid USER_GRAPH JSON from IPhreeqc: {error}"))
        })
    }

    /// Value of a named selected-output column in the last data row.
    pub fn last_value(&self, column: &str) -> Option<f64> {
        let rows = self.selected_output();
        let idx = rows.first()?.iter().position(|h| h == column)?;
        rows.last()?.get(idx)?.parse().ok()
    }

    /// Reaction enthalpy for an aqueous species, in kJ/mol, evaluated by
    /// PHREEQC's native thermodynamic implementation at its current
    /// temperature and pressure state.
    ///
    /// Run a `SOLUTION` first when a state other than PHREEQC's initial
    /// 25 °C, 1 atm defaults is required.
    pub fn species_delta_h(&mut self, species: &str) -> Result<f64, PhreeqcError> {
        let name = CString::new(species).map_err(|_| PhreeqcError::Nul)?;
        let mut value = f64::NAN;
        let status = unsafe { ffi::GetSpeciesDeltaH(self.id, name.as_ptr(), &mut value) };
        if status == -3 {
            return Err(PhreeqcError::UnknownSpecies(species.to_string()));
        }
        if status != 0 {
            return Err(PhreeqcError::Engine(self.error_string()));
        }
        if !value.is_finite() {
            return Err(PhreeqcError::NonFiniteSpeciesDeltaH(species.to_string()));
        }
        Ok(value)
    }

    /// The complete PHREEQC output of the last run (the report a desktop
    /// PHREEQC user would read), from memory.
    pub fn output_string(&self) -> String {
        let ptr = unsafe { ffi::GetOutputString(self.id) };
        if ptr.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
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

#[cfg(feature = "engine")]
impl Drop for Phreeqc {
    fn drop(&mut self) {
        unsafe {
            ffi::DestroyIPhreeqc(self.id);
        }
    }
}
