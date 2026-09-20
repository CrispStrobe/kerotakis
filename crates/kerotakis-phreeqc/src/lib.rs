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
pub mod aqueous_gases;
pub mod complexation;
pub mod dbindex;
pub mod derived;
pub mod enthalpy;
mod inventory;
mod native_namespace;
mod phase_diagnostics;
pub mod pourbaix;
mod redox_isolation;
pub use aqueous::{
    CacheData, CacheEntry, PathOutcome, PathResult, PhreeqcEquilibrator, SolveHook, SolveOutput,
};

/// The exact native database namespace expected by an external solver hook.
/// Hosts must load this version rather than an unextended upstream file.
pub fn aqueous_database(tag: &str) -> Result<String, String> {
    native_namespace::database(tag).map(|db| String::from_utf8_lossy(&db.database).into_owned())
}

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

/// Thermodynamic databases embedded in the binary: the upstream files carry
/// the USGS User Rights Notice; the shared ligand extension is independently
/// reviewed U.S. Bureau of Mines public-domain data.
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
    /// goes through [`minteq_v4()`], which adds the reviewed lactate,
    /// hypochlorite and thiosulfate definitions and the shared reviewed
    /// ligand slice. Reading
    /// these bytes directly would give a caller a database the engine is not
    /// running.
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

    /// One reviewed couple added to minteq.v4: hypochlorite / hypochlorous
    /// acid, which is what household bleach is.
    ///
    /// The gap this closes was not a gap at all but a wrong claim. The
    /// bench used to tell a learner that "no thermodynamic database defines
    /// a hypochlorite species … and the `ClO-` matches are all
    /// perchlorate". Both halves are false, and the file that falsifies
    /// them is vendored in this repository:
    /// `vendor/iphreeqc/database/llnl.dat` line 107 reads
    /// `Cl(1)     ClO-      0         Cl` — perchlorate is `Cl(7)`, three
    /// lines below it — line 898 writes the formation of `ClO-`, and line
    /// 4493 writes `H+ + ClO- = HClO` at `log_k 7.5692`. So the number was
    /// on the shelf the whole time; nothing had gone and read it.
    ///
    /// That pKa of 7.57 is the entire answer to "does diluted bleach stay
    /// alkaline". Hypochlorite is the conjugate base of a weak acid whose
    /// pKa sits above neutral, so a hypochlorite solution hydrolyses:
    /// 0.01 mol/L of it is about pH 9.8, and the couple buffers it there.
    ///
    /// **A pseudo-element, not chlorine's `Cl(1)` redox state.** llnl.dat
    /// couples `ClO-` to chloride through oxygen —
    /// `Cl- + 0.5 O2 = ClO-`, log K −15.1 — so entering it as a redox state
    /// of Cl would hand the solver an open beaker's atmospheric pe and let
    /// it decide how much of the bleach has already reduced itself to
    /// chloride. That is thermodynamically defensible and pedagogically
    /// useless: a bottle of bleach is a bottle of bleach, and its
    /// hypochlorite is kinetically persistent for months. `Hypochlorite`
    /// is therefore its own element with no redox partner, exactly as
    /// `Lactate` above is its own element rather than a state of carbon.
    /// The identifier it borrows settles the ACID–BASE behaviour of the
    /// couple and NOTHING ELSE: not its oxidising strength, not the rate at
    /// which it bleaches a dye, not its decomposition to chlorate. Every
    /// oxidation this bench does with bleach is still curated, and stays
    /// curated.
    ///
    /// **No enthalpy, deliberately** — the same trade `LACTATE_EXTENSION`
    /// makes, and for a sharper reason: llnl.dat states `-delta_H 0` for
    /// this protonation with the comment "Not possible to calculate
    /// enthalpy of reaction HClO". Zero there means *unknown*, not
    /// *athermal*, and carrying it across as a number would have turned an
    /// absent datum into a claim that protonating hypochlorite releases no
    /// heat.
    ///
    /// Alkalinity 1 and 51.4521 g/mol follow minteq.v4's own convention for
    /// a monoprotic anion (`Acetate  Acetate-  1  59.045`); the mass is
    /// Cl 35.4527 + O 15.9994 from IUPAC/CIAAW 2021, and agrees with
    /// PubChem CID 61739 to the second decimal.
    /// **The spelling is PHREEQC's, not chemistry's.** A master species must
    /// contain its element's name as written — `ClO-` for an element called
    /// `Hypochlorite` is rejected outright ("Master species, ClO- must
    /// contain the element, Hypochlorite"), which is why minteq.v4 writes
    /// acetate as `Acetate-` and `H(Acetate)` rather than `CH3COO-` and
    /// `CH3COOH`. So the couple goes in as `Hypochlorite-` and
    /// `H(Hypochlorite)`, and `derived::PROTONATION_SPLITS` maps those two
    /// names onto the registry's `ClO-` and `HClO`. The log K is llnl.dat's
    /// number unchanged; only the names are PHREEQC's.
    const HYPOCHLORITE_EXTENSION: &[u8] = b"
SOLUTION_MASTER_SPECIES
    Hypochlorite   Hypochlorite-   1   51.4521   51.4521
SOLUTION_SPECIES
    Hypochlorite- = Hypochlorite-
        log_k 0
    H+ + Hypochlorite- = H(Hypochlorite)
        log_k 7.5692
";

    /// One reviewed anion added to minteq.v4: thiosulfate, the salt the
    /// disappearing-cross practical is run with.
    ///
    /// Another refusal that was a claim about the world, and wrong.
    /// `species.rs` said "Sodium thiosulfate is in no PHREEQC database we
    /// ship" and `codex/rates.toml` told a learner the same thing.
    /// `vendor/iphreeqc/database/llnl.dat` is shipped in this repository
    /// and carries, at line 229, `S(+2)     S2O3-2    0         S`, with
    /// the formation at line 739 and `H+ + S2O3-2 = HS2O3-`, `log_k
    /// 1.0139`, at line 4585. Vendored and not routed is a fair thing to
    /// say; "in no database we ship" is not.
    ///
    /// **A pseudo-element, not sulfur's `S(+2)`.** The same trade
    /// `HYPOCHLORITE_EXTENSION` makes, and the reason is sharper here.
    /// llnl enters thiosulfate as a redox STATE of sulfur, one of nine it
    /// defines between `S(-2)` and `S(+8)`; the three datasets this lab
    /// routes define exactly two, `S(-2)` and `S(6)`, coupled through pe by
    /// `SO4-2 + 9 H+ + 8 e- = HS- + 4 H2O`. An open beaker's pe is pinned
    /// from atmospheric oxygen at about 19.6, and sulfur is not in
    /// `aqueous::FAST_REDOX`, so nothing would hold the state: the engine
    /// would oxidise the bottle of hypo to sulfate before the acid arrived
    /// and report it as the contents of the beaker. Thermodynamically
    /// defensible over geological time, useless for a practical that runs
    /// in forty seconds. `Thiosulfate` is therefore its own element with no
    /// redox partner, as `Lactate` and `Hypochlorite` are. What the
    /// borrowed number settles is the ACID-BASE behaviour of the anion and
    /// NOTHING ELSE: not its reducing strength, not the iodine titration,
    /// not the rate at which acid decomposes it to sulfur. Those are
    /// curated and stay curated.
    ///
    /// **Alkalinity 0, and it is llnl's own column.** Hypochlorite takes 1
    /// because its acid is weak with pKa 7.57. Thiosulfuric acid is not:
    /// pKa2 is 1.01 by the constant borrowed here (1.6-1.7 in the
    /// handbooks), so at the alkalinity endpoint the anion accepts no
    /// protons at all, exactly as sulfate does. llnl's own row writes 0 in
    /// that column and this copies it rather than reasoning to it.
    ///
    /// **No enthalpy, deliberately** - llnl states `-delta_H 0` for this
    /// protonation with the comment "Not possible to calculate enthalpy of
    /// reaction HS2O3-". Zero there means unknown, not athermal, and it is
    /// the same trade the two extensions above make.
    ///
    /// 112.1302 g/mol is 2 S 32.066 + 3 O 15.9994 from IUPAC/CIAAW 2021,
    /// and agrees with PubChem CID 1084's 112.13. The spelling is
    /// PHREEQC's, not chemistry's: a master species must contain its
    /// element's name, so the couple goes in as `Thiosulfate-2` and
    /// `H(Thiosulfate)-` and `derived::BOOKING_OVERRIDES` maps the element
    /// onto the registry's `S2O3-2`.
    const THIOSULFATE_EXTENSION: &[u8] = b"
SOLUTION_MASTER_SPECIES
    Thiosulfate   Thiosulfate-2   0   112.1302   112.1302
SOLUTION_SPECIES
    Thiosulfate-2 = Thiosulfate-2
        log_k 0
    H+ + Thiosulfate-2 = H(Thiosulfate)-
        log_k 1.0139
";

    /// One reviewed anion added to minteq.v4: sulfite, and unlike the three
    /// above it MOVES THE pH OF EVERY VESSEL THAT HAS EVER HELD THE SALT.
    ///
    /// `vendor/iphreeqc/database/llnl.dat` line 231 is
    /// `S(+4)     SO3-2     0         S` and line 4591 is
    /// `SO3-2 + H+ = HSO3-`, `log_k 7.2054`. Same borrow as lactate (#448),
    /// hypochlorite (#530) and thiosulfate (#536); same reason it is a
    /// borrow rather than a refusal.
    ///
    /// **A pseudo-element, not sulfur's `S(+4)`, and here the argument is
    /// strongest of the four.** llnl enters sulfite as a redox STATE of
    /// sulfur; the three datasets this lab routes define two, `S(-2)` and
    /// `S(6)`, coupled through pe. An open beaker's pe is pinned near 19.6
    /// by atmospheric oxygen and sulfur is not in `aqueous::FAST_REDOX`, so
    /// nothing would hold the state. For thiosulfate that was already fatal.
    /// For sulfite it is worse, because the oxidation is not a modelling
    /// artefact but the substance's actual headline chemistry: llnl's own
    /// line 1348 is `SO4-2 = SO3-2 + 0.5 O2`, and
    /// `kerotakis-safety` classes `Na2SO3` as a `ReducingAgent`. Entered as
    /// `S(+4)` the engine would air-oxidise the bottle to sulfate the
    /// instant it dissolved and report SULFATE as the contents of the
    /// beaker - thermodynamically defensible on a long enough view, and a
    /// lie about what is in front of the learner.
    ///
    /// **What that choice costs, stated rather than hidden.** As its own
    /// element sulfite has no redox partner, so this bench cannot compute
    /// sulfite reducing anything, cannot compute it slowly going off in air,
    /// and cannot compute the iodine or permanganate titrations from
    /// thermodynamics. What the borrowed number settles is the ACID-BASE
    /// behaviour of the anion and nothing else. Every one of those other
    /// claims is curated today and stays curated; none of them silently
    /// becomes wrong, because none of them was ever routed through the
    /// equilibrium solver.
    ///
    /// **Alkalinity 1, and this is the one number here that is NOT a
    /// transcription.** llnl writes 0 in that column. That is inconsistent
    /// with the very constant this block borrows: alkalinity counts the
    /// protons a species accepts down to the CO2 endpoint near pH 4.5, and
    /// at pH 4.5 sulfite is protonated, because its pKa is 7.20. Thiosulfate
    /// took llnl's 0 and the reasoning agreed with it, pKa2 being 1.01;
    /// hypochlorite takes 1 at pKa 7.57 and minteq's own `Acetate` takes 1
    /// at 4.76. Sulfite at 7.20 is the hypochlorite case, not the
    /// thiosulfate one. Copying the 0 would import llnl's inconsistency and
    /// make sulfite the only weak-acid anion on this bench that contributes
    /// nothing to alkalinity. If that judgement is wrong, this is the line
    /// to change.
    ///
    /// **The enthalpy IS carried, and that is new.** The other three
    /// extensions dropped `-delta_H` because llnl said "Not possible to
    /// calculate" and zero there means unknown. Line 4593 says
    /// `-delta_H 9.33032 kJ/mol # Calculated enthalpy of reaction HSO3-`,
    /// which is a datum rather than a hole, so it comes across and the
    /// couple is temperature-dependent by van't Hoff. Dropping it would
    /// have been the claim that protonating sulfite is athermal.
    ///
    /// **llnl's `-analytic` is deliberately NOT taken.** Line 4595 carries a
    /// five-term fit which evaluates to 7.2316 at 25 C against the stated
    /// `log_k 7.2054` - llnl's row disagrees with itself by 0.026 log units.
    /// PHREEQC prefers `-analytic` when present, so including it would
    /// silently borrow a different constant from the one this change is
    /// reviewed on. The stated log K is what goes in.
    ///
    /// **The second protonation is NOT borrowed.** llnl line 4343 has
    /// `2 H+ + SO3-2 = H2SO3`, `log_k 9.2132`, implying pKa1 2.01 for
    /// sulfurous acid. It is left out of this change and the consequence is
    /// real and bounded: below about pH 3 this bench will report HSO3- where
    /// some of the sulfur is really H2SO3 (and, physically, dissolved SO2).
    /// That is one review's worth of work on its own, it needs the SO2
    /// degassing question answered with it, and it does not affect the
    /// near-neutral vessels this change is about.
    ///
    /// 80.0642 g/mol is S 32.066 + 3 O 15.9994 (IUPAC/CIAAW 2021). The
    /// spelling is PHREEQC's: a master species must contain its element's
    /// name, so the couple goes in as `Sulfite-2` and `H(Sulfite)-`, and
    /// `derived::BOOKING_OVERRIDES` and `derived::PROTONATION_SPLITS` map
    /// them onto the registry's `SO3-2` and `HSO3-`.
    ///
    /// **A protonation split is REQUIRED here, where thiosulfate needed
    /// none, and the reason is NOT the one you would guess.** The tempting
    /// argument is that pKa 7.20 sits in the bench's range so a sulfite
    /// beaker is half anion and half acid. That is wrong, and the harvest
    /// in `sulfite_speciation.rs` says so: a salt of a weak acid
    /// HYDROLYSES, so 0.05 M sodium sulfite settles at pH 9.70 - two and a
    /// half units ABOVE the pKa - and is 99.86 % `SO3-2`. On its own that
    /// would argue FOR the thiosulfate treatment of a single booking ion.
    ///
    /// The real reason is that this bench stocks TWO bottles and they sit
    /// on OPPOSITE SIDES of the constant. Sodium sulfite reads pH 9.70 and
    /// is essentially all dianion; sodium bisulfite, `NaHSO3`, reads pH
    /// 4.17 and is 99.83 % `HSO3-`. One booking ion cannot serve both:
    /// whichever were chosen, the other bottle would be booked as a
    /// species it is almost none of. And the moment a sulfite solution is
    /// acidified the split is real in a single vessel too - 0.002 mol of
    /// HCl into that beaker gives pH 7.04 and a genuine 60/40 mixture.
    /// Booking either bottle under one name would be the acetate bug that
    /// `curated_reactants_survive_a_solve` exists to catch.
    const SULFITE_EXTENSION: &[u8] = b"
SOLUTION_MASTER_SPECIES
    Sulfite   Sulfite-2   1   80.0642   80.0642
SOLUTION_SPECIES
    Sulfite-2 = Sulfite-2
        log_k 0
    H+ + Sulfite-2 = H(Sulfite)-
        log_k 7.2054
        -delta_H 9.33032 kJ/mol
";

    /// Byte offset of the final `END` line, which is where a database
    /// stops being read. `None` when the file has none, in which case the
    /// end of the file is the right place after all.
    pub(super) fn find_last_end(text: &[u8]) -> Option<usize> {
        let mut at = None;
        let mut line_start = 0usize;
        // Include the final unterminated line; appending after a bare trailing
        // END would otherwise silently discard every extension definition.
        for line in text.split_inclusive(|b| *b == b'\n') {
            let start = line
                .iter()
                .position(|c| !c.is_ascii_whitespace())
                .unwrap_or(line.len());
            let end = line
                .iter()
                .rposition(|c| !c.is_ascii_whitespace())
                .map(|p| p + 1)
                .unwrap_or(start);
            if line[start..end].eq_ignore_ascii_case(b"END") {
                at = Some(line_start);
            }
            line_start += line.len();
        }
        at
    }

    /// minteq.v4 as this lab runs it: the vendored file plus
    /// [`LACTATE_EXTENSION`], [`HYPOCHLORITE_EXTENSION`],
    /// [`THIOSULFATE_EXTENSION`], [`SULFITE_EXTENSION`] and the reviewed
    /// reference-temperature ligand slice.
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
            let mut bytes = Vec::with_capacity(
                text.len()
                    + LACTATE_EXTENSION.len()
                    + HYPOCHLORITE_EXTENSION.len()
                    + THIOSULFATE_EXTENSION.len()
                    + SULFITE_EXTENSION.len(),
            );
            bytes.extend_from_slice(&text[..insert_at]);
            bytes.extend_from_slice(LACTATE_EXTENSION);
            bytes.extend_from_slice(HYPOCHLORITE_EXTENSION);
            bytes.extend_from_slice(THIOSULFATE_EXTENSION);
            bytes.extend_from_slice(SULFITE_EXTENSION);
            bytes.extend_from_slice(&text[insert_at..]);
            super::aqueous_gases::append_to(&super::complexation::append_to(&bytes))
        })
    }

    /// WATEQ4F plus the reviewed 25 C copper/thiocyanate ligand slice.
    /// The unmodified `WATEQ4F` bytes remain available for upstream-oracle tests.
    pub fn wateq4f() -> &'static [u8] {
        use std::sync::OnceLock;
        static EXTENDED: OnceLock<Vec<u8>> = OnceLock::new();
        EXTENDED.get_or_init(|| {
            super::aqueous_gases::append_to(&super::complexation::append_to(WATEQ4F))
        })
    }

    /// Pitzer plus reviewed gas uptake; no unsupported ligand extension.
    pub fn pitzer() -> &'static [u8] {
        use std::sync::OnceLock;
        static EXTENDED: OnceLock<Vec<u8>> = OnceLock::new();
        EXTENDED.get_or_init(|| super::aqueous_gases::append_to(PITZER))
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
