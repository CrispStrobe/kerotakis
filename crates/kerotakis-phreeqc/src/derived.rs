//! The derivation layer: how registry species participate in aqueous
//! problems, computed from their chemical formulas and the embedded
//! databases — not maintained by hand.
//!
//! What remains curated here, deliberately and documented:
//! - `OXYANION_GROUPS`: ~six rows of real chemistry (a sulfate group is
//!   S(6)) that give formulas their valence mapping;
//! - `BOOKING_OVERRIDES`: the protonation state dissolved element totals
//!   are booked as at teaching pH (bicarbonate rather than the database's
//!   carbonate master, dihydrogen phosphate rather than PO4-3);
//! - `ATMOSPHERIC`: physical constants — log10 partial pressures and registry
//!   identities for gases exchanged across an external boundary.
//!
//! Everything else (which compounds map to which elements, which mineral
//! phases exist where, their stoichiometry, hydrate waters, polymorph
//! choice) is derived.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use kerotakis_core::species::{self, Phase};

use crate::databases;
use crate::dbindex::{parse_formula, split_hydrate, DbIndex};

#[derive(Debug, Clone)]
pub enum DerivedRole {
    Solvent,
    /// Element contributions when dissolved (freely soluble or ionic).
    Dissolves(Vec<(String, f64)>),
    /// Enters as an amount-limited mineral phase.
    Mineral {
        phase: String,
        elements: Vec<(String, f64)>,
    },
}

#[derive(Debug, Clone)]
pub struct DerivedPhase {
    pub name: String,
    /// Registry key of the solid this phase is booked as.
    pub species: &'static str,
    /// Waters of crystallisation moved between liquid and crystal.
    pub waters: f64,
    /// Elements the dissolution reaction exchanges with solution.
    pub elements: Vec<(String, f64)>,
    /// log K of the dissolution reaction, from the database that defines
    /// this phase. Carried so a reviewed phase can be posed outside its
    /// home database (see `foreign_phase_definition`).
    pub log_k: Option<f64>,
}

/// Gas phases an open vessel exchanges, with registry identity and log10
/// atmospheric partial pressure. Physical constants, not chemistry the
/// databases know.
pub const ATMOSPHERIC: &[(&str, &str, f64)] = &[("CO2(g)", "CO2", -3.408)];

/// Reviewed gas/liquid equilibrium capabilities, independent of an ambient
/// reservoir. A finite supplied gas is not an infinite atmospheric source.
/// HBr's phase data and applicability are recorded with the gas data slice.
pub const EQUILIBRIUM_GASES: &[(&str, &str)] = &[("CO2(g)", "CO2"), ("HBr(g)", "HBr")];

/// Oxyanion groups: the valence-carrying units formulas decompose into.
/// (Element-count signatures; order matters — longest/most specific first.)
fn oxyanion_groups() -> &'static [(&'static str, &'static str)] {
    &[
        // Ammonium is a valence-carrying unit exactly as an oxyanion is:
        // the nitrogen in NH4Cl is N(-III), and entering it as bare "N"
        // would both hand PHREEQC the nitrate master species and leave the
        // element coupled to pe — an open beaker's air would then oxidise
        // a school salt to nitrate before anything was added to it.
        ("NH4", "N(-3)"),
        // Ammonia is the same valence-carrying unit one proton lighter,
        // and both shipped databases that carry nitrogen speciate it:
        // `NH4+ = NH3 + H+`, with N(-3) mastered by NH4+. Without this row
        // the nitrogen fell through to the residue rules, where bare N is
        // not allowed, and a bottle of household ammonia could not compute
        // its own pH — the one thing a solution of a weak base is for.
        //
        // It must be tried AFTER NH4, and that ordering is chemistry
        // rather than tidiness: ammonium chloride is N,H4,Cl, and NH3 taken
        // first would extract the base and leave a stray proton, booking a
        // school salt as ammonia plus hydrochloric acid. Checked by
        // simulating both tables over all 141 registry compositions on
        // 2026-09-03: exactly one formula changes, and it is this one.
        ("NH3", "N(-3)"),
        // Citrate before acetate, because "longest/most specific first" is
        // load-bearing here rather than cosmetic: citric acid's C6H8O7
        // admits two whole acetate units by pure arithmetic, and taking
        // them would book the tribasic acid as something it is not.
        // minteq.v4 is the only shipped database that defines Citrate, and
        // it carries all three protonation constants (log K 6.396, 11.157,
        // 14.285 — pKa 3.13, 4.76, 6.40), so this row is what lets a
        // citric-acid solution compute its own pH instead of refusing.
        ("C6H5O7", "Citrate"),
        // Lactate before acetate, same "most specific first" rule: lactic
        // acid is C3H6O3 and an acetate unit is C2H3O2, which fits inside
        // it by arithmetic and leaves CH3O — not a residue any database
        // can place, so the extraction would refuse rather than mis-book.
        // The order makes that a non-question instead of a near miss.
        ("C3H5O3", "Lactate"),
        ("C2H3O2", "Acetate"), // CH3COO
        // Hypochlorite is NOT here, deliberately — see
        // `extract_hypochlorite` below for why greedy extraction is the
        // wrong shape for it.
        ("HCO3", "C"),
        ("CO3", "C"),
        ("NO3", "N(5)"),
        ("SO4", "S(6)"),
        ("PO4", "P"),
        ("MnO4", "Mn(7)"),
    ]
}

/// Elements allowed as bare residue after group extraction: those whose
/// aqueous identity is unambiguous (simple cations and halides). Anything
/// else left over (N, S, C, P outside a group; unbalanced O) means the
/// compound's aqueous chemistry is not derivable — honestly unmappable.
const RESIDUE_OK: &[&str] = &[
    "Na", "K", "Ca", "Mg", "Ag", "Li", "Sr", "Ba", "Cl", "Br", "F", "Cu", "Mn", "Fe", "Zn", "Pb",
];

/// The subset of `RESIDUE_OK` that forms simple cations.
///
/// Oxygen left over after the hydrogen is accounted for is *oxide* oxygen
/// when the only thing beside it is a metal: CuO dissolves as Cu²⁺ + H₂O,
/// the oxide leaving as water exactly as hydroxide does, and PHREEQC's
/// `pH charge` recovers the protons it consumed. With a halogen residue the
/// same arithmetic would be wrong — the oxygen in hypochlorite is bound to
/// chlorine and is not available as water — which is what the original
/// oxygen guard was protecting against, rather more broadly than it needed
/// to: it rejected every simple metal oxide in the databases.
const CATION_RESIDUE: &[&str] = &[
    "Na", "K", "Ca", "Mg", "Ag", "Li", "Sr", "Ba", "Cu", "Mn", "Fe", "Zn", "Pb",
];

/// The subset of `RESIDUE_OK` that carries exactly one negative charge in
/// solution, so a metal halide's stoichiometry fixes the metal's state.
const HALIDE_RESIDUE: &[&str] = &["Cl", "Br", "F"];

/// Acids this lab can name but no shipped database can speciate.
///
/// A carboxylic acid that dissolves and leaves the solution at pH 7.00 is
/// the exact failure this table exists to prevent. Where a database
/// carries the anion, nothing is needed here — citrate is in minteq.v4
/// and computes its own pH through `oxyanion_groups`. Where no database
/// carries it, the acid still dissolves (it genuinely does dissolve) but
/// its protons are not in the speciation, and the solver must say so
/// instead of returning a neutral answer that looks like a result.
///
/// Malate is in none of the roughly forty PHREEQC databases vendored with
/// iphreeqc, let alone the four this lab ships — checked by name against
/// every `.dat` in `vendor/iphreeqc/database` on 2026-08-29. minteq.v4
/// carries tartrate and citrate and a dozen other organic ligands, and
/// simply does not carry this one.
///
/// Each row is (registry key, what is missing). The message the solver
/// builds from it names the substance, so a reader is told which bottle
/// on the bench the caveat is about.
pub const UNSPECIATED_ACIDS: &[(&str, &str)] = &[
    (
        "malic_acid",
        "no shipped database defines a malate species, so its two carboxylic protons are not in this pH",
    ),
];

/// Substances no shipped database can speciate AT ALL.
///
/// `UNSPECIATED_ACIDS` above is about a missing acidity: the substance
/// dissolves and its protons are absent from the pH. This is the total
/// case — nothing gives the thing an aqueous role, so it sits in the water
/// and the solver has nothing whatever to say about it.
///
/// Until this table existed it said nothing, and that was the defect. A
/// beaker of bleach and water produced, in full:
///
/// ```text
/// v1: +27.6714 mol water
/// v1: +0.0050 mol bleach (sodium hypochlorite)
/// v1: the pH meter reads nothing — no aqueous solution has been
///     characterised in this vessel
/// ```
///
/// Every line true, and between them they never say the bleach is why. A
/// learner is told an instrument failed.
///
/// **The table is empty, and the way its one row left is the point.** That
/// row was bleach, and the sentence it carried said that "no thermodynamic
/// database defines a hypochlorite species … and the `ClO-` matches are
/// all perchlorate". Both halves were false, and the file that falsifies
/// them was already vendored here:
/// `vendor/iphreeqc/database/llnl.dat` line 107 is
/// `Cl(1)     ClO-      0         Cl`, with perchlorate three lines below
/// it as `Cl(7)`, and line 4493 is `H+ + ClO- = HClO`, `log_k 7.5692`. The
/// search that sentence claimed to have run cannot have reached llnl.dat,
/// and its second half — that the `ClO-` matches "are all perchlorate" —
/// is not a search result at all but a guess about what a search would
/// have found. It shipped to learners for three days. A refusal is a claim
/// about the world and can be wrong in exactly the way a computed number
/// can be wrong, only worse: nobody re-derives it.
/// `databases::minteq_v4()` now borrows the couple, and the beaker of
/// bleach reads pH 9.8.
///
/// The machinery is kept rather than deleted because the case it exists
/// for is real and recurs; the empty slice is the honest current state of
/// it. Each row carries its whole sentence rather than a fragment, because
/// the reason differs in kind between rows and a shared wrapper would
/// flatten them. Membership is a strong claim — it must be checked against
/// every `.dat` vendored with iphreeqc, and `grep -n` in that directory is
/// the check, not memory of an earlier search — and it is what earns the
/// `NotInAnyDatabase` cause: not in our gift, and not in anybody's.
pub const UNSPECIATED_SOLUTES: &[(&str, &str)] = &[];

/// How dissolved element totals are booked back into the vessel inventory:
/// the database's master species, unless overridden by the documented
/// protonation-state choice.
const BOOKING_OVERRIDES: &[(&str, &str)] = &[
    ("C", "HCO3-"),
    ("P", "H2PO4-"),
    ("Acetate", "CH3COO-"),
    // Lactate's master species is `Lactate-`, which is not a registry key;
    // the registry pairs the ion with its acid as `lactate` / `lactic_acid`,
    // word keys rather than formulae, which is how that pair was already
    // named. Frozen with kerotakis-59 before the registry row was written,
    // because `PROTONATION_SPLITS` below hard-codes the string and a later
    // rename would be a silent mis-book rather than a compile error.
    ("Lactate", "lactate"),
    // minteq.v4's citrate master species is `Citrate-3`, which is not a
    // registry key; the registry books the fully deprotonated ion under
    // its formula, exactly as acetate books as CH3COO-. Without this the
    // rebuild would find no booking ion for the Citrate element and
    // panic rather than return the citrate mass to the vessel.
    ("Citrate", "C6H5O7-3"),
    // Hypochlorite's master species in the borrowed block is
    // `Hypochlorite-`, which PHREEQC insists on and no chemist writes; the
    // registry's word for the same ion is `ClO-`, added with this change so
    // that a solved bleach solution has somewhere honest to book. Required
    // rather than optional: the master-species fallback would book matter
    // into `Hypochlorite-`, which nothing downstream can resolve.
    ("Hypochlorite", "ClO-"),
    // Thiosulfate's master species in the borrowed block is
    // `Thiosulfate-2`, which PHREEQC insists on because a master species
    // must contain its element's name; the registry - and every chemist -
    // writes `S2O3-2`. There is no protonation split beside it, and that
    // is the citrate precedent rather than an omission: the couple's pKa
    // is 1.01, so above pH 3 the anion is the whole of the element and
    // naming the total after it loses nothing. In the practical's beaker
    // a small fraction is `H(Thiosulfate)-`, which the pH accounts for
    // and the ledger books under the anion, exactly as minteq's three
    // citrate constants all book back as `C6H5O7-3`.
    ("Thiosulfate", "S2O3-2"),
    // Sulfite's master species in the borrowed block is `Sulfite-2` for
    // the same PHREEQC reason; the registry writes `SO3-2`. UNLIKE
    // thiosulfate this row does NOT stand alone - there is a
    // `PROTONATION_SPLITS` entry beside it, because the bench stocks two
    // sulfite bottles that sit on opposite sides of pKa 7.20: sodium
    // sulfite hydrolyses to pH 9.70 and is almost all `SO3-2`, sodium
    // bisulfite reads pH 4.17 and is almost all `HSO3-`. One booking ion
    // would book one of them as a species it is barely any of, which is
    // the acetate bug, not the citrate case.
    ("Sulfite", "SO3-2"),
    // Same shape, for the reviewed ligand slice: PHREEQC's component is
    // `Thiocyanate`, the registry's word for the ion is `SCN-`.
    ("Thiocyanate", "SCN-"),
    ("Mn(7)", "MnO4-"),
    // Bare manganese books as the reduced ion, which is what the databases
    // treat as the master species and what dissolved manganese actually is
    // at bench conditions. Needed once MnO2 entered the registry: it is the
    // first substance to put Mn in solution as anything but permanganate.
    ("Mn", "Mn+2"),
    // Iron books as the reduced ion, which is the databases' master
    // species and what dissolving an iron(II) salt actually gives.
    ("Fe", "Fe+2"),
    ("Cu", "Cu+2"),
];

pub struct Derived {
    pub wateq4f: DbIndex,
    pub minteq: DbIndex,
    pub pitzer: DbIndex,
    roles: BTreeMap<&'static str, DerivedRole>,
    phases: Vec<DerivedPhase>,
    /// Every matched polymorph across every database, resolvable by
    /// name — readback must recognise Fe(OH)3(a) even though the
    /// deduped candidate list carries Ferrihydrite.
    all_phases: Vec<DerivedPhase>,
    /// (species, db_tag) → the stablest polymorph name IN that
    /// database. The dedupe above is database-blind by design (one
    /// candidate per solid); this map is how an input written for a
    /// specific database poses that database's own polymorph.
    db_best: BTreeMap<(String, &'static str), String>,
    bookings: BTreeMap<String, &'static str>,
}

static DERIVED: OnceLock<Derived> = OnceLock::new();

pub fn derived() -> &'static Derived {
    DERIVED.get_or_init(Derived::build)
}

pub fn role(key: &str) -> Option<&'static DerivedRole> {
    derived().roles.get(key)
}

pub fn candidate_phases() -> &'static [DerivedPhase] {
    &derived().phases
}

pub fn phase_by_name(name: &str) -> Option<&'static DerivedPhase> {
    let d = derived();
    d.phases
        .iter()
        .find(|p| p.name == name)
        .or_else(|| d.all_phases.iter().find(|p| p.name == name))
}

/// The phase name to pose in `db_tag` for whatever solid `name` names:
/// the name itself if that database defines it, else the database's own
/// stablest polymorph of the same registry solid, else None — and a
/// None is exactly the case where the honesty pass's supersaturation
/// note is the right answer.
pub fn phase_in_db(name: &str, db_tag: &str) -> Option<&'static str> {
    let d = derived();
    let tag: &'static str = DB_TAGS.iter().find(|t| **t == db_tag).copied()?;
    if index_for(tag).has_phase(name) {
        return d
            .all_phases
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.name.as_str());
    }
    let species = phase_by_name(name)?.species;
    d.db_best
        .get(&(species.to_string(), tag))
        .map(String::as_str)
}

/// Phases a routed database may pose even though it does not define them,
/// reviewed one by one. An injected definition carries the home database's
/// log K and no enthalpy — a phase whose home entry has a real delta_h
/// would silently lose its temperature dependence, so membership requires
/// checking the source entry. Fe(OH)2 (minteq.v4, `delta_h -0 kJ`) loses
/// nothing, and without it ferrous iron plus lye stays silent on the
/// default route: wateq4f spells no ferrous hydroxide at all.
const FOREIGN_POSABLE: &[&str] = &["Fe(OH)2"];

/// A `PHASES` input block defining `name` for a database that lacks it —
/// `None` unless the phase is reviewed foreign-posable, absent from the
/// routed database under every polymorph name, and of the shape the
/// synthesis covers: an anhydrous single-metal hydroxide M(OH)n, whose
/// dissolution `M(OH)n + n H+ = M^n+ + n H2O` can be written in the routed
/// database's own master-species spelling.
pub fn foreign_phase_definition(name: &str, db_tag: &str) -> Option<String> {
    if !FOREIGN_POSABLE.contains(&name) {
        return None;
    }
    let tag: &'static str = DB_TAGS.iter().find(|t| **t == db_tag).copied()?;
    let idx = index_for(tag);
    if idx.has_phase(name) {
        return None; // native — nothing to inject
    }
    let d = derived();
    let p = d.all_phases.iter().find(|p| p.name == name)?;
    if d.db_best.contains_key(&(p.species.to_string(), tag)) {
        return None; // the database has its own polymorph — pose that
    }
    if p.waters != 0.0 || p.elements.len() != 1 {
        return None;
    }
    let (el_key, count) = &p.elements[0];
    if *count != 1.0 {
        return None;
    }
    let (base, rest) = el_key.split_once('(')?;
    let state: i32 = rest.trim_end_matches(')').parse().ok()?;
    if state <= 0 {
        return None;
    }
    let master = &idx.masters.get(el_key)?.species;
    let log_k = p.log_k?;
    Some(format!(
        "PHASES\n    {name}\n        {base}(OH){state} + {state} H+ = {master} + {state} H2O\n        log_k {log_k}\n"
    ))
}

/// Element states whose registry name is a question about protonation
/// rather than about oxidation.
///
/// The readback books an element total as one ion. That is the right shape
/// for a state whose identity is settled — dissolved chloride is Cl⁻ at
/// every pH this bench reaches — and the wrong shape for reduced nitrogen,
/// which is ammonia above pKa 9.25 and ammonium below it. Both are registry
/// species, both are what the beaker actually holds, and which one it is
/// decides whether the solution smells: `senses::waft` walks the vessel
/// looking for NH3 by key. Booking a bottle of household ammonia as
/// ammonium would have stopped it smelling of ammonia the moment its pH
/// was measured — the model would have been right about the number and
/// wrong about the thing.
///
/// Each row is (element state, &[(database species, registry key)]). The
/// species are asked for by name in `SELECTED_OUTPUT` and the state's total
/// is divided between the registry keys in proportion to the molalities
/// that come back, so the split is the solve's own and not a curated guess
/// about teaching pH. A state listed here still needs its booking ion:
/// that stays the answer whenever the columns are absent — pitzer carries
/// no nitrogen at all.
/// Note the asymmetry in each row: the database's spelling on the left,
/// the registry's on the right, and they are not the same word. minteq.v4
/// writes undissociated acetic acid `H(Acetate)`; this lab writes
/// `CH3COOH`. Asking PHREEQC for the registry key, or booking the database
/// name, would each fail silently in its own direction.
pub const PROTONATION_SPLITS: &[(&str, &[(&str, &str)])] = &[
    (
        "C",
        &[
            ("CO2", "CO2(aq)"),
            // MINTEQ's hydrated neutral-carbon basis. Solvent completion
            // supplies the H2O difference; water has zero basis enthalpy.
            ("H2CO3", "CO2(aq)"),
            ("HCO3-", "HCO3-"),
            ("CO3-2", "CO3-2"),
        ],
    ),
    (
        "P",
        &[
            ("H3PO4", "H3PO4"),
            ("H2PO4-", "H2PO4-"),
            ("HPO4-2", "HPO4-2"),
            ("PO4-3", "PO4-3"),
        ],
    ),
    ("N(-3)", &[("NH3", "NH3"), ("NH4+", "NH4+")]),
    // Acetate. The consequence is larger than the row: booking the whole
    // element total as `CH3COO-` left undissociated acetic acid out of the
    // ledger entirely, and two curated reactions name it as a reactant —
    // `NaHCO₃ + CH₃COOH` and `CaCO₃ + 2 CH₃COOH`, vinegar and baking soda
    // and vinegar on an eggshell. Both were unable to fire in a beaker with
    // water in it, because by the time the second reagent arrived the acid
    // in their reactant list was no longer there. See
    // `tests/curated_reactants_survive_a_solve.rs`, which now enforces that
    // a curated reaction can find its own reactants after a solve.
    (
        "Acetate",
        &[("H(Acetate)", "CH3COOH"), ("Acetate-", "CH3COO-")],
    ),
    // Lactate, for the same reason acetate is here: booking the whole
    // element total as the anion would drop the undissociated acid out of
    // the ledger, and a lactic fermentation makes nothing else. The
    // database spelling is on the left and the registry's on the right,
    // and as with acetate they are not the same word.
    (
        "Lactate",
        &[("H(Lactate)", "lactic_acid"), ("Lactate-", "lactate")],
    ),
    // Hypochlorite, and here the split is not a convenience: pKa 7.57 sits
    // in the middle of the range a bench works in, so which member the
    // beaker holds is a real question at every step rather than a formality
    // at the extremes. Diluted bleach at pH 9.8 is 99% the anion; the same
    // bleach with a descaler poured into it is almost entirely the
    // undissociated acid, which is the form that then makes chlorine. A
    // single booking ion would have named both of those `ClO-` and lost the
    // distinction the hazard turns on.
    //
    // The asymmetry is at its widest in this row, and it is PHREEQC's
    // doing rather than a choice: a master species must contain its
    // element's name, so the borrowed block spells the couple
    // `Hypochlorite-` and `H(Hypochlorite)` where the registry — and every
    // chemist — writes `ClO-` and `HClO`.
    (
        "Hypochlorite",
        &[("H(Hypochlorite)", "HClO"), ("Hypochlorite-", "ClO-")],
    ),
    // Sulfite, and this is the row the whole change turns on. Thiosulfate
    // took the citrate precedent and had NO split: at pKa 1.01 the anion is
    // the whole of the element anywhere a bench works, so naming the total
    // after it lost nothing.
    //
    // Sulfite needs the split for a reason that is NOT "pKa 7.20 is in the
    // working range so the beaker is half and half". Measured, a sulfite
    // beaker is not half and half: the salt hydrolyses to pH 9.70 and is
    // 99.86 % `SO3-2`. The reason is that the bench stocks BOTH bottles,
    // and they straddle the constant - `NaHSO3` reads pH 4.17 and is
    // 99.83 % `HSO3-`. A single booking ion is wrong for one of the two
    // whichever one is picked. Acidify the sulfite and both forms are real
    // in one vessel as well: 0.002 mol of HCl gives pH 7.04, 60/40.
    (
        "Sulfite",
        &[("H(Sulfite)-", "HSO3-"), ("Sulfite-2", "SO3-2")],
    ),
    // Boron, and this split is the borax buffer itself. `H3BO3 = H2BO3- +
    // H+` is log K −9.236 in minteq.v4 (line 4912) and −9.24 in wateq4f
    // (line 335), so pKa 9.24 — and a borax solution sits AT that pKa by
    // construction rather than near it: dissolving Na₂B₄O₇ books four
    // borons against two sodiums, the charge balance asks for two moles of
    // anion out of four moles of boron, and half-neutralised is the
    // definition of a buffer at its pKa. Booking the element total under a
    // single ion would have named a 50:50 mixture after one of its halves.
    //
    // Unlike hypochlorite, no spelling is borrowed here: `H3BO3` and
    // `H2BO3-` are the names both routed databases that carry the couple
    // already use, and they are the registry's keys unchanged. pitzer
    // spells the same element `B(OH)3`/`B(OH)4-`, so neither column exists
    // on the brine route — which the all-or-nothing rule in the readback
    // handles by falling back to the single booking ion, exactly as it
    // does for pitzer's missing nitrogen.
    ("B", &[("H3BO3", "H3BO3"), ("H2BO3-", "H2BO3-")]),
];

/// The protonation split for an element state, if it has one.
pub fn protonation_split(element: &str) -> Option<&'static [(&'static str, &'static str)]> {
    PROTONATION_SPLITS
        .iter()
        .find(|(el, _)| *el == element)
        .map(|(_, species)| *species)
}

/// Registry key an element total is booked as. `None` means the element has
/// no registry ion to book into (a registry gap, surfaced by tests).
pub fn booking_ion(element: &str) -> Option<&'static str> {
    derived().bookings.get(element).copied()
}

/// The datasets the router chooses between, in the order `explain` reports
/// them.
pub const DB_TAGS: [&str; 3] = ["wateq4f", "minteq.v4", "pitzer"];

/// How much of the mineral world each dataset admits, and how much of it
/// they agree on.
///
/// Asking the same question of every dataset is one of the honest things
/// the bench does, but three answers are not three opinions about one
/// question unless the models are choosing from the same shelf — and they
/// are not. Most phases exist in only one dataset, so part of any
/// disagreement is not a disagreement about thermodynamics at all: it is
/// one dataset being unable to precipitate a solid another one defines.
/// A renderer that shows the three answers should be able to say so.
///
/// Gas phases are excluded. The claim being made is about which *solids* a
/// dataset may form, and `CO2(g)` is not one — it is also one of the 22
/// names all three datasets happen to share, so counting it would inflate
/// exactly the number the sentence leans on.
#[derive(Debug, Clone)]
pub struct PhaseCoverage {
    /// Distinct mineral names across every dataset.
    pub total: usize,
    /// Mineral names present in *every* dataset.
    pub shared: usize,
    /// `(tag, count)` per dataset, in `DB_TAGS` order.
    pub per_database: Vec<(&'static str, usize)>,
}

/// Computed on demand rather than cached: it is cheap, and a `OnceLock`
/// reachable from `derived()` is how we deadlocked this module once before.
pub fn phase_coverage() -> PhaseCoverage {
    let idx: Vec<&DbIndex> = DB_TAGS.iter().map(|t| index_for(t)).collect();
    fn minerals(i: &'static DbIndex) -> Vec<&'static str> {
        i.phases
            .iter()
            .filter(|(_, info)| !info.is_gas)
            .map(|(n, _)| n.as_str())
            .collect()
    }
    let mut names: BTreeSet<&str> = BTreeSet::new();
    for i in &idx {
        names.extend(minerals(i));
    }
    PhaseCoverage {
        shared: names
            .iter()
            .filter(|n| idx.iter().all(|i| i.has_phase(n)))
            .count(),
        total: names.len(),
        per_database: DB_TAGS
            .iter()
            .zip(&idx)
            .map(|(t, i)| (*t, minerals(i).len()))
            .collect(),
    }
}

pub fn index_for(db_tag: &str) -> &'static DbIndex {
    let d = derived();
    match db_tag {
        "minteq.v4" => &d.minteq,
        "pitzer" => &d.pitzer,
        _ => &d.wateq4f,
    }
}

impl Derived {
    fn build() -> Derived {
        let wateq4f = DbIndex::parse(databases::wateq4f());
        let minteq = DbIndex::parse(databases::minteq_v4());
        let pitzer = DbIndex::parse(databases::pitzer());
        let indexes = [&wateq4f, &minteq, &pitzer];

        // --- Candidate phases: database phases whose formula matches a
        // solid registry species exactly (including hydrate waters), deduped
        // per species by stability (lowest log_k = least soluble polymorph).
        let mut phases: Vec<(DerivedPhase, f64)> = Vec::new();
        for idx in indexes {
            for (name, info) in &idx.phases {
                if info.is_gas || phases.iter().any(|(p, _)| p.name == *name) {
                    continue;
                }
                let Some(reg) = registry_solid_matching(&info.composition, info.waters) else {
                    continue;
                };
                // wateq4f defines AgMetal, CuMetal and ZnMetal, and the
                // moment elemental silver entered the registry the matcher
                // paired them — so PHREEQC began plating silver out of
                // silver nitrate with no reductant in the beaker, at
                // whatever electron activity it happened to be holding.
                // Found by the conservation fuzz test (kerotakis-de,
                // 2026-08-20): 6.16 mol of silver became 3.88. The
                // metallic state is `kerotakis_core::displacement`'s, which
                // accounts for the electrons; these phases stay out of the
                // candidate list, and the database's log_k for them serves
                // as the independent check on that module's E° table.
                if kerotakis_core::displacement::is_elemental_metal(reg) {
                    continue;
                }
                // The same exclusion, for the same reason, one class over.
                // `dry_ice` is a registry solid whose formula is CO2, and
                // this matcher pairs registry solids with database phases
                // by composition — so the moment dry ice reached the shelf
                // it became a candidate "mineral" for any phase written
                // with carbon and two oxygens, and a carbonate solution
                // could have precipitated it at 25 °C. A condensed gas is
                // not a mineral: it is the other phase of something this
                // registry also ships as a gas, it exists so a beaker can
                // hold it, and its stability is a temperature threshold in
                // `kerotakis_core::phase_route`, not a solubility product.
                // Pinned by `a_carbonate_solution_never_precipitates_dry_ice`.
                if kerotakis_core::phase_route::is_condensed_gas(reg) {
                    continue;
                }
                // Elements from the formula's own composition through the
                // same group decomposition as registry formulas — robust
                // against per-database equation quirks (e.g. wateq4f's
                // alkalinity conventions).
                let Some(elements) = contribution_from_counts(info.composition.clone(), indexes)
                else {
                    continue;
                };
                phases.push((
                    DerivedPhase {
                        name: name.clone(),
                        species: reg,
                        waters: info.waters,
                        elements,
                        log_k: info.log_k,
                    },
                    info.log_k.unwrap_or(f64::MAX),
                ));
            }
        }
        // Dedupe polymorphs: keep the stablest phase per solid species.
        // Per-database stablest polymorph, and the full resolvable set:
        // collected BEFORE the global dedupe so readback can name any
        // database's polymorph and input-building can pose the routed
        // database's own.
        let mut db_best: BTreeMap<(String, &'static str), String> = BTreeMap::new();
        let mut all_phases: Vec<DerivedPhase> = Vec::new();
        for (tag, idx) in DB_TAGS.iter().zip(indexes.iter()) {
            let mut per_db: Vec<(&DerivedPhase, f64)> = phases
                .iter()
                .filter(|(p, _)| idx.has_phase(&p.name))
                .map(|(p, k)| (p, *k))
                .collect();
            per_db.sort_by(|a, b| a.1.total_cmp(&b.1));
            for (p, _) in per_db {
                db_best
                    .entry((p.species.to_string(), *tag))
                    .or_insert_with(|| p.name.clone());
            }
        }
        for (p, _) in &phases {
            if !all_phases.iter().any(|q| q.name == p.name) {
                all_phases.push(p.clone());
            }
        }

        phases.sort_by(|a, b| a.1.total_cmp(&b.1));
        let mut seen: Vec<&str> = Vec::new();
        let phases: Vec<DerivedPhase> = phases
            .into_iter()
            .filter_map(|(p, _)| {
                if seen.contains(&p.species) {
                    None
                } else {
                    seen.push(p.species);
                    Some(p)
                }
            })
            .collect();

        // --- Bookings: master species per element (must exist as a registry
        // key), with the documented protonation overrides.
        let mut bookings: BTreeMap<String, &'static str> = BTreeMap::new();
        for idx in indexes {
            for (element, master) in &idx.masters {
                if bookings.contains_key(element) {
                    continue;
                }
                let over = BOOKING_OVERRIDES
                    .iter()
                    .find(|(el, _)| el == element)
                    .map(|(_, ion)| *ion);
                let candidate: Option<&'static str> = over.or_else(|| {
                    species::REGISTRY
                        .iter()
                        .find(|s| s.key == master.species)
                        .map(|s| s.key)
                });
                if let Some(ion) = candidate {
                    bookings.insert(element.clone(), ion);
                }
            }
        }

        // --- Roles from formulas.
        let mut roles: BTreeMap<&'static str, DerivedRole> = BTreeMap::new();
        for s in species::REGISTRY {
            if s.key == "water" {
                roles.insert(s.key, DerivedRole::Solvent);
                continue;
            }
            if s.standard_phase == Phase::Gas
                && !EQUILIBRIUM_GASES
                    .iter()
                    .any(|(_, species)| *species == s.key)
            {
                continue;
            }
            // A metal is not its cation. Deriving a role for magnesium
            // ribbon from its formula booked it as Mg²⁺ on contact with
            // water, spending two moles of electrons per mole before any
            // electron balance saw them. The metallic state is modelled
            // by `kerotakis_core::displacement`, which moves the electrons
            // by the activity series and hands this engine the *ions* it
            // produces; the metal itself stays an inventory solid that
            // passes through a solve untouched.
            if kerotakis_core::displacement::is_elemental_metal(s.key) {
                continue;
            }
            // Mineral: a candidate phase books as this species.
            if s.standard_phase == Phase::Solid {
                if let Some(p) = phases.iter().find(|p| p.species == s.key) {
                    roles.insert(
                        s.key,
                        DerivedRole::Mineral {
                            phase: p.name.clone(),
                            elements: p.elements.clone(),
                        },
                    );
                    continue;
                }
            }
            // A species that *is* one of the database's master species
            // carries its oxidation state in its element name: Mn+2 is
            // "Mn(2)", not "Mn". Deriving the element from the formula
            // instead loses that, and then a vessel holding both
            // permanganate and manganese(II) writes `Mn(7)` and `Mn` into
            // the same input — which PHREEQC reads as the same element
            // entered twice and refuses outright.
            let as_master = indexes
                .iter()
                .find_map(|idx| idx.species_element.get(s.key))
                .filter(|el| el.contains('('));
            if let Some(element) = as_master {
                roles.insert(s.key, DerivedRole::Dissolves(vec![(element.clone(), 1.0)]));
                continue;
            }
            // A soluble solid whose dissolution this bench BOUNDS with a
            // curated number keeps that bound, and does not get a
            // `Dissolves` role.
            //
            // `Dissolves` is phase-blind on purpose — `partition` enters
            // its element totals whatever phase the portion is in — so a
            // solid handed this role dissolves ENTIRELY, however much of
            // it is in the beaker. That is right for the salts it was
            // written for, which have no curated limit and are freely
            // soluble at bench doses. It is wrong for a bottle whose whole
            // interest is that most of it sits on the bottom.
            //
            // Where a routed database defines the solid there is no
            // conflict: the `Mineral` branch above claims it first, the
            // portion goes into `EQUILIBRIUM_PHASES`, and the saturation
            // index does the bounding. This guard is for the other case,
            // and boron is what put it here. Every one of the three
            // datasets defines the element (`B  H3BO3`) and two of them
            // carry `H3BO3 = H2BO3- + H+`, so `extract_borate` now derives
            // a contribution for `Na2B4O7` where nothing did before — and
            // NONE of the three spells a sodium borate SOLID. minteq.v4
            // and wateq4f have `Pb(BO2)2`, `Zn(BO2)2`, `Cd(BO2)2` and
            // (minteq only) `Co(BO2)2`, which is the entire boron mineral
            // shelf on the routes that carry the couple. Without this
            // guard, 25 g of borax into 100 mL of water entered the engine
            // as 0.497 mol of boron in 0.1 kg of solvent, the crystals on
            // the bottom of the snowflake beaker stopped existing, and
            // nothing said so: `saturation_moves` had already done the
            // honest arithmetic (2.5 g/100 mL at 20 °C, 27.4 at 100) and
            // the tail undid it on the same step.
            //
            // The curated limit is the registry's own statement that this
            // bench bounds the dissolution itself. It is not evidence
            // against speciating the DISSOLVED part — that is a separate
            // and better change, and it wants `saturation_moves` to be
            // able to see a dissolved amount that has been booked onto
            // ions, which it cannot today.
            let bounded_solid =
                s.standard_phase == Phase::Solid && s.aqueous_solubility_g_per_100_ml.is_some();
            if bounded_solid {
                continue;
            }
            if let Some(contrib) = derive_contribution(s.formula, indexes) {
                roles.insert(s.key, DerivedRole::Dissolves(contrib));
            }
            // else: honestly unmappable — no role, the honesty pass speaks.
        }

        Derived {
            wateq4f,
            minteq,
            pitzer,
            roles,
            phases,
            all_phases,
            db_best,
            bookings,
        }
    }
}

/// A solid registry species whose formula matches this composition and
/// hydrate count exactly.
pub(crate) fn registry_solid_matching(
    composition: &BTreeMap<String, f64>,
    waters: f64,
) -> Option<&'static str> {
    for s in species::REGISTRY {
        if s.standard_phase != Phase::Solid {
            continue;
        }
        let (base, w) = split_hydrate(s.formula);
        if w != waters {
            continue;
        }
        if parse_formula(&base).as_ref() == Some(composition) {
            return Some(s.key);
        }
    }
    None
}

/// Decompose a formula into element contributions: greedy oxyanion-group
/// extraction, then residue rules (drop hydroxide OH pairs and acid
/// protons; anything else unaccounted → unmappable).
fn derive_contribution(formula: &str, indexes: [&DbIndex; 3]) -> Option<Vec<(String, f64)>> {
    let (base, _) = split_hydrate(formula);
    let mut counts = parse_formula(&base)?;
    // Preserve the explicitly written ligand identity, not an empirical
    // C/N/S coincidence (thiourea, for example, is not ammonium thiocyanate).
    // Counterions and multiplicities still go through the generic parser.
    if base.contains("SCN") {
        let ligands = ["S", "C", "N"]
            .iter()
            .map(|el| counts.get(*el).copied().unwrap_or(0.0))
            .fold(f64::INFINITY, f64::min);
        if ligands > 0.0 && ligands.is_finite() {
            for el in ["S", "C", "N"] {
                let left = counts.get(el).copied().unwrap_or(0.0) - ligands;
                if left == 0.0 {
                    counts.remove(el);
                } else {
                    counts.insert(el.into(), left);
                }
            }
            let mut result = if counts.is_empty() {
                Vec::new()
            } else {
                contribution_from_counts(counts, indexes)?
            };
            result.push(("Thiocyanate".into(), ligands));
            return Some(result);
        }
    }
    // CO2 is a complete molecular identity, not an extractable functional
    // group. Greedily subtracting it from arbitrary organic formulas would
    // turn malic acid into acetate plus carbon dioxide.
    if counts.len() == 2 && counts.get("C") == Some(&1.0) && counts.get("O") == Some(&2.0) {
        return Some(vec![("C".into(), 1.0)]);
    }
    contribution_from_counts(counts, indexes)
}

/// The charge this element books into solution as, resolved without
/// touching `derived()` — see the note at the call site.
fn cation_charge(element: &str, indexes: [&DbIndex; 3]) -> Option<f64> {
    let ion = BOOKING_OVERRIDES
        .iter()
        .find(|(el, _)| *el == element)
        .map(|(_, ion)| (*ion).to_string())
        .or_else(|| {
            indexes
                .iter()
                .find_map(|idx| idx.masters.get(element).map(|m| m.species.clone()))
        })?;
    kerotakis_core::stoich::parse_formula(&ion)
        .ok()
        .map(|f| f.charge)
}

/// Hypochlorite, extracted by its own rule rather than by
/// `oxyanion_groups`, because greedy extraction is the wrong shape for it
/// and the wrongness is not hypothetical.
///
/// `ClO` as a greedy group takes one chlorine and one oxygen out of ANY
/// formula that has both. Atacamite is `Cu2ClH3O3` — a hydroxychloride,
/// the green that grows on copper near the sea — and the greedy rule
/// booked its chloride and one of its three hydroxide oxygens as
/// hypochlorite, which cost the phase its registry match and put the
/// `copper-patina` lesson back to "nothing can precipitate out of it
/// here". The shipped databases are full of the same shape:
/// `Zn2(OH)3Cl`, `Zn5(OH)8Cl2`, `CdOHCl`, `Pb2(OH)3Cl`,
/// `Fe(OH)2.7Cl.3`.
///
/// What actually identifies hypochlorite is that its oxygen is bound to
/// the chlorine and there is EXACTLY ONE of each: `O == Cl`, with nothing
/// left over for a hydroxide to claim. Spare oxygen means something else —
/// a chlorate, a perchlorate, or a hydroxide salt — and this returns
/// `false` for all of them, leaving them to the `o > h` guard to refuse.
///
/// Hydrogen decides the last case. `HClO` is the free acid and carries one
/// proton per chlorine; `CdOHCl` has the same three counts plus a metal,
/// and its hydrogen is a hydroxide's. So a formula with a cation in it may
/// carry no hydrogen at all.
fn extract_hypochlorite(counts: &mut BTreeMap<String, f64>) -> Option<f64> {
    let cl = counts.get("Cl").copied().unwrap_or(0.0);
    let o = counts.get("O").copied().unwrap_or(0.0);
    let h = counts.get("H").copied().unwrap_or(0.0);
    if cl < 1.0 || o != cl {
        return None;
    }
    let has_cation = counts
        .keys()
        .any(|el| CATION_RESIDUE.contains(&el.as_str()));
    let protonation_fits = if has_cation {
        h == 0.0
    } else {
        h == 0.0 || h == cl
    };
    if !protonation_fits {
        return None;
    }
    counts.remove("Cl");
    counts.remove("O");
    if h == cl {
        counts.remove("H");
    }
    Some(cl)
}

/// Borate, extracted by its own exact rule rather than by
/// `oxyanion_groups`, for the same reason `extract_hypochlorite` above has
/// one: the greedy machinery cannot spell this group.
///
/// **Boron was never refused by the databases. Nothing asked them.** All
/// three datasets this lab routes define the element outright —
/// `wateq4f.dat` line 16 `B  H3BO3  0  10.81  10.81`, `minteq.v4.dat`
/// line 17 `B  H3BO3  0  B  10.81`, `pitzer.dat` `B  B(OH)3  0  B  10.81`
/// — and both of the first two carry the dissociation that gives borax its
/// pH: `H3BO3 = H2BO3- + H+`, `log_k -9.236` at minteq.v4 line 4912 and
/// `-9.24` at wateq4f line 335, with the polyborates, the alkali and
/// alkaline-earth borate pairs and Hfo sorption beside them. What was
/// missing was three lines in THIS file. `Na2B4O7` decomposes to Na₂B₄O₇,
/// no group in `oxyanion_groups` contains boron, `B` is not in
/// `RESIDUE_OK`, so `derive_contribution` returned `None` and the salt got
/// the `dissolves_without_speciation` fallback — and the registry then
/// wrote down "no shipped database is asked for it", which is a true
/// sentence about the wiring that shipped as a claim about the world. The
/// borosilicate recipe made the stronger claim outright: "no shipped
/// database defines a borate that would let it dissolve or react". That
/// one was simply false.
///
/// **Why a group cannot spell it.** `oxyanion_groups` contributes ONE
/// element unit per matched group, so a group has to contain exactly one
/// boron; but boron's oxygen in a solid is 1.5 per B (the oxide unit is
/// B₂O₃), and "BO1.5" is not a formula. Halving it to `BO2` is not the
/// same rule: on Na₂B₄O₇ it fits three times and leaves `Na2BO`, booking
/// three quarters of the boron and stranding the fourth.
///
/// **What the rule is, exactly.** All boron here is boron(III) and its
/// oxygen is oxide oxygen, so this takes ALL of the boron or none of it,
/// together with exactly 1.5 O per boron, and hands whatever is left to
/// the residue rules unchanged. Those rules then do the charge arithmetic
/// they already do for a metal oxide: Na₂B₄O₇ leaves `Na2O`, whose
/// implied cation charge is (2·1 − 0)/2 = +1 and matches sodium's, so the
/// orphan oxygen leaves as water and PHREEQC's `pH charge` recovers the
/// two protons — which is the borax buffer, 4 mol B against 2 mol Na
/// sitting at pKa 9.24 by construction.
///
/// **What it excludes, by name.** A formula with less than 1.5 O per B is
/// not a borate and is refused whole: `NaBH4` (borohydride, O = 0) and
/// `BF3` and `B2H6` never reach the residue rules at all. A formula
/// carrying any element that is neither boron, oxygen, hydrogen nor a
/// simple cation is refused before any oxygen is taken, which is what
/// keeps this from being greedy in the way #530's first `ClO` attempt
/// was: a borate-sulfate or a boronic acid would otherwise have had 1.5
/// oxygens per boron pulled out from under another group's feet. It runs
/// AFTER `oxyanion_groups` so that a sulfate's or a carbonate's oxygen is
/// already spoken for, and AFTER the free-acid guard because that guard
/// counts fitted group units and this is not a fit: B₂O₃ is two borons
/// because it has two borons, not because two units happened to fit.
fn extract_borate(counts: &mut BTreeMap<String, f64>) -> Option<f64> {
    let b = counts.get("B").copied().unwrap_or(0.0);
    if b <= 0.0 {
        return None;
    }
    let o = counts.get("O").copied().unwrap_or(0.0);
    // 1.5 oxygens per boron: the oxide unit is B2O3, and boron(III) is the
    // only oxidation state any of these datasets speciates.
    let needed = 1.5 * b;
    if o < needed {
        return None;
    }
    // Nothing but boron, its oxygen, hydrogen and simple cations may be
    // present. Anything else means the oxygen is contested and this rule
    // has no business deciding whose it is.
    if counts
        .keys()
        .any(|el| el != "B" && el != "O" && el != "H" && !CATION_RESIDUE.contains(&el.as_str()))
    {
        return None;
    }
    counts.remove("B");
    let rest = o - needed;
    if rest > 0.0 {
        counts.insert("O".to_string(), rest);
    } else {
        counts.remove("O");
    }
    Some(b)
}

/// Thiosulfate, by an exact rule for the reason `extract_hypochlorite`
/// has one: the greedy machinery would reach into another oxyanion's
/// oxygen, and #530 proved that is not hypothetical.
///
/// What identifies thiosulfate is that it is EXACTLY two sulfurs and
/// three oxygens - one sulfate-like centre with one of its oxygens
/// replaced by the second sulfur, which is the whole of why the anion
/// falls apart into sulfur when it is acidified. A greedy `S2O3` group
/// would take those counts out of any formula that could spare them, and
/// the shipped databases are full of formulas that can: `S2O8-2`,
/// `S2O6-2`, `S2O5-2`, `S2O4-2` and `S4O6-2` all carry two or more
/// sulfurs and three or more oxygens and none of them is thiosulfate.
///
/// **What it excludes, by name.** Sulfite and sulfate (one sulfur, so the
/// count is wrong before the oxygen is looked at - and sulfite now has its
/// own exact rule in `extract_sulfite` below, so the two matchers must
/// stay disjoint by that sulfur count and `sulfite_and_thiosulfate_never_
/// claim_each_other` holds them to it), metabisulfite `S2O5`,
/// dithionite `S2O4`, persulfate `S2O8`, tetrathionate `S4O6`, pyrite
/// `FeS2` (no oxygen), and any formula carrying an element that is neither
/// sulfur, oxygen, hydrogen nor a simple cation - which is what stops it
/// claiming the oxygen of a nitrate or a carbonate in the same salt. It
/// runs BEFORE the group loop, like hypochlorite, because the counts it
/// demands are exact: nothing it accepts could have been claimed by `SO4`
/// first, which needs four oxygens per sulfur where this has one and a
/// half.
///
/// Hydrogen decides the last case exactly as it does for hypochlorite:
/// `H2S2O3` is the free acid and carries two protons, while a salt carries
/// a cation instead and no hydrogen of its own.
fn extract_thiosulfate(counts: &mut BTreeMap<String, f64>) -> Option<f64> {
    let sulfur = counts.get("S").copied().unwrap_or(0.0);
    let oxygen = counts.get("O").copied().unwrap_or(0.0);
    let hydrogen = counts.get("H").copied().unwrap_or(0.0);
    if sulfur != 2.0 || oxygen != 3.0 {
        return None;
    }
    if counts
        .keys()
        .any(|el| el != "S" && el != "O" && el != "H" && !CATION_RESIDUE.contains(&el.as_str()))
    {
        return None;
    }
    let has_cation = counts
        .keys()
        .any(|el| CATION_RESIDUE.contains(&el.as_str()));
    let protonation_fits = if has_cation {
        hydrogen == 0.0
    } else {
        hydrogen == 0.0 || hydrogen == 2.0
    };
    if !protonation_fits {
        return None;
    }
    counts.remove("S");
    counts.remove("O");
    if hydrogen == 2.0 {
        counts.remove("H");
    }
    Some(1.0)
}

/// Sulfite, by an exact rule for the same reason thiosulfate and
/// hypochlorite have one, and with one extra case they do not.
///
/// What identifies sulfite is EXACTLY one sulfur and three oxygens. The
/// neighbouring oxyanion is sulfate at one sulfur and FOUR, so the two
/// cannot be confused by a counting rule - which is why the greedy `SO4`
/// group in `oxyanion_groups()` cannot claim a sulfite and why this rule
/// can run before it safely.
///
/// **The extra case is a protonated SALT.** Thiosulfate and hypochlorite
/// could assume that a formula with a cation carries no hydrogen of its
/// own, because sodium hypochlorite and sodium thiosulfate are the only
/// salts of theirs a bench holds. Sulfite has two: `Na2SO3` AND `NaHSO3`,
/// sodium bisulfite, which is `Na1 H1 O3 S1` and is the reagent the
/// Landolt clock is actually run with. So a cation permits zero or one
/// hydrogen here where the older rules permit only zero, and the hydrogen
/// is consumed when it is there.
///
/// **What it excludes, by name.** Sulfate and bisulfate (four oxygens),
/// sulfur dioxide `SO2` (two), metabisulfite `S2O5`, dithionite `S2O4` and
/// thiosulfate itself (two sulfurs, rejected before the oxygen is looked
/// at), and any formula carrying an element that is neither sulfur, oxygen,
/// hydrogen nor a simple cation. That last clause is what stops it claiming
/// the three oxygens of `methyl_orange`, which is `C14 H14 N3 Na1 O3 S1` -
/// exactly one sulfur and three oxygens, and the ONLY registry formula that
/// passes the count while not being a sulfite. It is refused on its carbon
/// and nitrogen, deliberately and not by accident: a greedy `("SO3",
/// "Sulfite")` row in `oxyanion_groups()` would have booked the dye's
/// sulfonate as a bottle of sulfite, which is exactly the atacamite failure
/// #530 was written about.
fn extract_sulfite(counts: &mut BTreeMap<String, f64>) -> Option<f64> {
    let sulfur = counts.get("S").copied().unwrap_or(0.0);
    let oxygen = counts.get("O").copied().unwrap_or(0.0);
    let hydrogen = counts.get("H").copied().unwrap_or(0.0);
    if sulfur != 1.0 || oxygen != 3.0 {
        return None;
    }
    if counts
        .keys()
        .any(|el| el != "S" && el != "O" && el != "H" && !CATION_RESIDUE.contains(&el.as_str()))
    {
        return None;
    }
    let has_cation = counts
        .keys()
        .any(|el| CATION_RESIDUE.contains(&el.as_str()));
    // `Na2SO3` and `NaHSO3` with a cation; `SO3-2`, `HSO3-` and the free
    // acid `H2SO3` without one.
    let protonation_fits = if has_cation {
        hydrogen == 0.0 || hydrogen == 1.0
    } else {
        hydrogen == 0.0 || hydrogen == 1.0 || hydrogen == 2.0
    };
    if !protonation_fits {
        return None;
    }
    counts.remove("S");
    counts.remove("O");
    if hydrogen > 0.0 {
        counts.remove("H");
    }
    Some(1.0)
}

fn contribution_from_counts(
    mut counts: BTreeMap<String, f64>,
    indexes: [&DbIndex; 3],
) -> Option<Vec<(String, f64)>> {
    let mut contrib: Vec<(String, f64)> = Vec::new();

    if let Some(n) = extract_hypochlorite(&mut counts) {
        contrib.push(("Hypochlorite".to_string(), n));
    }

    if let Some(n) = extract_thiosulfate(&mut counts) {
        contrib.push(("Thiosulfate".to_string(), n));
    }

    if let Some(n) = extract_sulfite(&mut counts) {
        contrib.push(("Sulfite".to_string(), n));
    }

    for (group_formula, element) in oxyanion_groups() {
        let sig = parse_formula(group_formula).expect("group formulas parse");
        let fit = sig
            .iter()
            .map(|(el, n)| (counts.get(el).copied().unwrap_or(0.0) / n).floor())
            .fold(f64::INFINITY, f64::min);
        if fit >= 1.0 {
            for (el, n) in &sig {
                let c = counts.get_mut(el).unwrap();
                *c -= n * fit;
            }
            contrib.push((element.to_string(), fit));
        }
    }
    counts.retain(|_, n| *n > 0.0);

    // A free acid carries exactly one of its anion skeleton.
    //
    // Group extraction is pure arithmetic, and on a molecule made only of
    // carbon, hydrogen and oxygen the arithmetic will happily find units
    // that are not there. Glucose is C6H12O6, which is three times
    // C2H3O2 plus three protons — so without this guard the sugar entered
    // solution as three acetates and acidified it, and malic acid entered
    // as two. Both are formula arithmetic dressed as speciation, and both
    // were silent.
    //
    // The chemistry that rules them out: a compound with no cation to
    // balance the charge is the *free acid* of its anion, and a free acid
    // contains one anion skeleton, not several. Where a cation residue
    // does exist the count is real and stays unrestricted — a diacetate
    // salt is two acetates because the metal says so. Only the
    // no-cation case is constrained, and only per group, so a mixed salt
    // like NH4NO3 (one ammonium, one nitrate) is untouched.
    let no_cation_residue = !counts
        .keys()
        .any(|el| el != "H" && el != "O" && RESIDUE_OK.contains(&el.as_str()));
    if no_cation_residue && contrib.iter().any(|(_, fit)| *fit > 1.0) {
        return None;
    }

    // Borate, after the guard above rather than before it: the guard
    // counts fitted group units and refuses a no-cation formula that
    // yielded more than one, which is right for glucose-as-three-acetates
    // and wrong for B2O3-as-two-borons. See `extract_borate` for why this
    // is not an `oxyanion_groups` row and for what it excludes.
    if let Some(n) = extract_borate(&mut counts) {
        if !indexes.iter().any(|idx| idx.has_element("B")) {
            return None;
        }
        contrib.push(("B".to_string(), n));
    }

    // Hydroxide pairs and acid protons drop out: they live in the water /
    // charge-balance domain, and PHREEQC's `pH charge` recovers them.
    let h = counts.remove("H").unwrap_or(0.0);
    let o = counts.remove("O").unwrap_or(0.0);
    // A pure hydroxide (equal O and H, one residue cation) fixes the
    // metal's oxidation state by its own stoichiometry: n·q = h. Booking
    // charge is the right tag for a simple salt, but Fe(OH)3 is iron(III)
    // no matter that this lab books bare iron as iron(II) — tagging it
    // Fe(2) posed a ferric solid against ferrous totals, and PHREEQC
    // obligingly ran the redox conversion the engine had stood down.
    let hydroxide_state: Option<i32> = if o > 0.0 && o == h && counts.len() == 1 {
        counts.iter().next().and_then(|(_, n)| {
            let implied = h / n;
            (implied.fract() == 0.0 && implied > 0.0).then_some(implied as i32)
        })
    } else {
        None
    };
    // A simple halide says the same thing by the same arithmetic: in MXₙ
    // the halide count per metal atom IS the metal's oxidation state.
    // Without this, iron(III) chloride entered as the lab's default
    // iron(II) — three chlorides balanced against a divalent cation, with
    // PHREEQC's `pH charge` quietly inventing the acid that made up the
    // difference. The salt's own stoichiometry is the evidence; nothing is
    // curated per compound.
    let halide_state: Option<i32> = if h == 0.0 && o == 0.0 && counts.len() == 2 {
        let cation = counts
            .iter()
            .find(|(el, _)| CATION_RESIDUE.contains(&el.as_str()))
            .map(|(_, n)| *n);
        let halide = counts
            .iter()
            .find(|(el, _)| HALIDE_RESIDUE.contains(&el.as_str()))
            .map(|(_, n)| *n);
        match (cation, halide) {
            (Some(metal), Some(halide)) if metal > 0.0 => {
                let implied = halide / metal;
                (implied.fract() == 0.0 && implied > 0.0).then_some(implied as i32)
            }
            _ => None,
        }
    } else {
        None
    };
    if o > h {
        // Oxide oxygen may be dropped as water only when doing so conserves
        // electrons. Treating O as −2 and each O–H pair as hydroxide, the
        // metal's oxidation state in the solid is (2·O − H) per metal atom;
        // it must equal the charge on the ion the element books as.
        //
        // CuO booked as Cu²⁺ balances, so tenorite dissolves as itself.
        // MnO2 booked as Mn²⁺ does not — that is Mn(IV) + 2e⁻ with no
        // oxidant anywhere, and the orphaned oxygen leaves as alkalinity:
        // 5 g of manganese dioxide dissolving completely to give a
        // cation-only solution reading pH 13.55, which is not chemistry.
        //
        // Excluding every redox-active element instead was the first
        // attempt and was too blunt: it took copper with it, and copper's
        // oxide is not a redox problem at all.
        let balanced = counts.len() == 1
            && counts.iter().all(|(el, n)| {
                if !CATION_RESIDUE.contains(&el.as_str()) || *n <= 0.0 {
                    return false;
                }
                let implied = (2.0 * o - h) / n;
                // Resolved from the indexes in hand, never through
                // `booking_ion`: this runs inside the `derived()` one-time
                // initialiser, and asking `derived()` for anything from in
                // here deadlocks on its own lock.
                cation_charge(el, indexes).is_some_and(|q| (q - implied).abs() < 1e-9)
            });
        if !balanced {
            // Either an oxyanion (hypochlorite's oxygen is bound to
            // chlorine, not available as water) or an oxide whose
            // dissolution would be a redox step we do not model. Not
            // derivable, and said so rather than guessed at.
            return None;
        }
    }

    for (el, n) in counts {
        if !RESIDUE_OK.contains(&el.as_str()) {
            return None;
        }
        if !indexes.iter().any(|idx| idx.has_element(&el)) {
            return None;
        }
        // A redox-active element goes in carrying its oxidation state.
        // Entered plainly it is coupled to pe, and an open beaker's pe is
        // set by the air above it — so dissolving iron(II) sulfate produced
        // iron(III) before any reagent was added, because the atmosphere
        // had already oxidised it. That is thermodynamically true and
        // kinetically wrong: a titration is run with fresh iron(II)
        // solution precisely because it oxidises slowly. Tagging pins it.
        //
        // The state is the charge on the ion this lab books the element as,
        // which is the curated statement BOOKING_OVERRIDES already makes —
        // "when we say iron, we mean iron(II)". A compound that contained
        // the element in another state would need its own entry.
        let redox_active = indexes.iter().any(|idx| idx.redox_elements.contains(&el));
        let tagged = if redox_active {
            match hydroxide_state.or(halide_state) {
                Some(state) => {
                    let key = format!("{el}({state})");
                    if indexes.iter().any(|idx| idx.has_element(&key)) {
                        Some(key)
                    } else {
                        // The state the stoichiometry demands has no
                        // aqueous master species anywhere — dissolving this
                        // would be a redox step we do not model. Refused,
                        // like MnO2 above.
                        return None;
                    }
                }
                None => cation_charge(&el, indexes)
                    .filter(|q| *q > 0.0)
                    .map(|q| format!("{el}({})", q as i32)),
            }
        } else {
            None
        };
        contrib.push((tagged.unwrap_or(el), n));
    }
    if contrib.is_empty() {
        None
    } else {
        Some(contrib)
    }
}

/// ΔH of `HCO₃⁻ + H⁺ → H₂O(l) + CO₂(g)` in kJ/mol, positive endothermic,
/// assembled by Hess's law from the routed database's own `delta_h` rows.
///
/// This is the reaction under a bicarbonate meeting an acid — the school
/// volcano, and the reason it gets COLD. The bench had no number for it:
/// `aqueous.rs` discounts the carbonate route out of the neutralisation
/// heat precisely because charging it the strong-acid-strong-base enthalpy
/// gave the wrong magnitude and the wrong sign, and there was nothing
/// honest to charge instead.
///
/// Derived, not curated, and not transcribed from a book we do not have
/// open. In minteq.v4 the two rows are
///
/// ```text
///     H+ + CO3-2 = HCO3-        delta_h -14.6 kJ   # source: NIST46.4
///     CO2(g): CO2 + H2O = 2 H+ + CO3-2   delta_h 4.06 kJ
/// ```
///
/// and the target is the first reversed plus the second reversed:
///
/// ```text
///     HCO3-        -> H+ + CO3-2      +14.60
///     2H+ + CO3-2  -> CO2(g) + H2O     -4.06
///     ------------------------------------------
///     HCO3- + H+   -> CO2(g) + H2O    +10.54 kJ/mol
/// ```
///
/// Only minteq.v4 is answered, and deliberately so. The algebra above
/// depends on the SHAPE of the phase equation, not just its number:
/// wateq4f writes its CO2(g) dissolution as `CO2 = CO2` — gas to an
/// aqueous CO2 species rather than to the carbonate master species and a
/// proton — so the same subtraction there would be a different reaction
/// wearing this one's name. `carbonate_rows_are_the_shape_the_algebra_assumes`
/// pins both lines against the shipped file, so a database update that
/// changes either fails loudly instead of silently returning a number for
/// the wrong cycle.
///
/// Standard state, 25 °C: the phase enthalpy has no engine accessor to
/// temperature-correct it through, so neither side is corrected and the
/// pair stays consistent. Over the 0–100 °C a beaker sees, ΔH for this
/// reaction moves by a few percent — small against being absent.
pub fn carbonate_acid_enthalpy_kj(db_tag: &str) -> Option<f64> {
    if db_tag != "minteq.v4" {
        return None;
    }
    let idx = index_for(db_tag);
    let hco3 = idx.species_delta_h_kj.get("HCO3-")?;
    let co2_gas = idx.phases.get("CO2(g)")?.delta_h_kj?;
    Some(-hco3 - co2_gas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dissolves(key: &str) -> Vec<(String, f64)> {
        match role(key) {
            Some(DerivedRole::Dissolves(c)) => c.clone(),
            other => panic!("{key}: expected Dissolves, got {other:?}"),
        }
    }

    /// A condensed gas is not a mineral, and the matcher must not make
    /// one of it.
    ///
    /// `registry_solid_matching` pairs registry solids with database
    /// phases by composition, and `dry_ice` is a registry solid whose
    /// formula is CO2. Without the guard above, any database phase
    /// written with one carbon and two oxygens would have adopted it —
    /// and a carbonate solution could then precipitate "dry ice" at
    /// 25 °C, which is not a saturation index, it is a category error.
    /// This test walks the whole matched set rather than checking dry ice
    /// alone, so the next condensed gas the shelf gains is covered before
    /// anyone thinks about it.
    #[test]
    fn a_condensed_gas_is_never_a_database_mineral() {
        use kerotakis_core::phase_route::is_condensed_gas;
        // The predicate has to mean something, or the loop below passes
        // by being vacuous.
        assert!(
            is_condensed_gas("dry_ice"),
            "dry ice must be recognised as the condensed phase of a shipped gas"
        );
        assert!(!is_condensed_gas("CaCO3"), "chalk is a mineral");
        assert!(!is_condensed_gas("CO2"), "the gas itself is not condensed");

        for phase in derived().all_phases.iter().chain(&derived().phases) {
            assert!(
                !is_condensed_gas(phase.species),
                "{} was paired with the database phase {}",
                phase.species,
                phase.name
            );
        }
        assert!(
            !matches!(role("dry_ice"), Some(DerivedRole::Mineral { .. })),
            "dry ice must not carry a mineral role: {:?}",
            role("dry_ice")
        );
    }

    #[test]
    fn roles_are_derived_correctly() {
        // Minerals found by formula match, stablest polymorph chosen.
        for (key, phase) in [
            ("NaCl", "Halite"),
            ("AgCl", "Cerargyrite"),
            ("CaCO3", "Calcite"),
            ("gypsum", "Gypsum"),
        ] {
            match role(key) {
                Some(DerivedRole::Mineral { phase: p, .. }) => assert_eq!(p, phase, "{key}"),
                other => panic!("{key}: expected Mineral, got {other:?}"),
            }
        }

        // Freely-soluble compounds decomposed by group + residue.
        assert_eq!(
            dissolves("AgNO3"),
            vec![("N(5)".into(), 1.0), ("Ag".into(), 1.0)]
        );
        assert_eq!(dissolves("HCl"), vec![("Cl".into(), 1.0)]);
        assert_eq!(dissolves("NaOH"), vec![("Na".into(), 1.0)]);
        assert_eq!(
            dissolves("CaCl2"),
            vec![("Ca".into(), 1.0), ("Cl".into(), 2.0)]
        );
        assert_eq!(dissolves("H3PO4"), vec![("P".into(), 1.0)]);
        assert_eq!(dissolves("CH3COOH"), vec![("Acetate".into(), 1.0)]);
        // NaHCO3 may be Nahcolite-backed (derived from the databases) or
        // freely soluble; either way its elements are Na + C.
        let mut nahco3: Vec<String> = match role("NaHCO3").expect("NaHCO3 has a role") {
            DerivedRole::Dissolves(c) | DerivedRole::Mineral { elements: c, .. } => {
                c.iter().map(|(el, _)| el.clone()).collect()
            }
            DerivedRole::Solvent => panic!("NaHCO3 is not the solvent"),
        };
        nahco3.sort();
        assert_eq!(nahco3, vec!["C".to_string(), "Na".to_string()]);

        // Ions map through their formulas.
        assert_eq!(dissolves("SO4-2"), vec![("S(6)".into(), 1.0)]);
        assert_eq!(dissolves("HCO3-"), vec![("C".into(), 1.0)]);

        // BRD-012.S02. Ammonium is a valence-carrying group like an
        // oxyanion: the nitrogen goes in reduced, not as the nitrate
        // master species an untagged "N" would select.
        assert_eq!(
            dissolves("NH4Cl"),
            vec![("N(-3)".into(), 1.0), ("Cl".into(), 1.0)]
        );
        assert_eq!(dissolves("NH4+"), vec![("N(-3)".into(), 1.0)]);
        // A metal halide's stoichiometry fixes the metal's oxidation
        // state — three chlorides mean iron(III), whatever this lab's
        // default for bare iron is.
        assert_eq!(
            dissolves("FeCl3"),
            vec![("Cl".into(), 3.0), ("Fe(3)".into(), 1.0)]
        );
        // …and only where the halide says so: the sulfate is still the
        // lab's iron(II).
        assert_eq!(
            dissolves("FeSO4"),
            vec![("S(6)".into(), 1.0), ("Fe(2)".into(), 1.0)]
        );
        // Barium is not redox-active in these databases, so no tag.
        assert_eq!(
            dissolves("BaCl2"),
            vec![("Ba".into(), 1.0), ("Cl".into(), 2.0)]
        );
        assert_eq!(dissolves("Ba(OH)2"), vec![("Ba".into(), 1.0)]);

        // Ammonia is reduced nitrogen, exactly as ammonium is. This used
        // to assert `is_none()` — the gap was documented as though it were
        // a limit, and it was a missing table row: both shipped databases
        // that carry nitrogen speciate `NH4+ = NH3 + H+`.
        assert_eq!(dissolves("NH3"), vec![("N(-3)".into(), 1.0)]);
        // And the ordering that makes that safe. Group extraction is
        // greedy, so NH3 taken before NH4 would strip the base out of
        // ammonium chloride and leave a proton behind — booking a school
        // salt as ammonia plus hydrochloric acid. One ammonium, one
        // chloride, no stray hydrogen.
        assert_eq!(
            dissolves("NH4Cl"),
            vec![("N(-3)".into(), 1.0), ("Cl".into(), 1.0)]
        );

        // Bleach is NOT on this list any more, and the comment that used
        // to put it here is why it is worth saying so. It claimed that
        // "every `.dat` vendored with iphreeqc was searched by name on
        // 2026-09-03 for HClO, ClO-, Cl(1) and the word hypochlorite: not
        // one of them defines the species". llnl.dat, in this repository,
        // defines all three spellings — `Cl(1)  ClO-` at line 107, the
        // formation at 898, `H+ + ClO- = HClO  log_k 7.5692` at 4493. The
        // `o > h` guard's refusal was sound and is still what refuses
        // NaClO3; the claim built on top of it was not. Bleach dissolves as
        // sodium and hypochlorite now.
        assert_eq!(
            dissolves("NaOCl"),
            vec![("Hypochlorite".into(), 1.0), ("Na".into(), 1.0)]
        );
        assert_eq!(dissolves("ClO-"), vec![("Hypochlorite".into(), 1.0)]);
        assert_eq!(dissolves("HClO"), vec![("Hypochlorite".into(), 1.0)]);

        // And the thing `extract_hypochlorite` exists to protect. Atacamite
        // is Cu2ClH3O3 — a hydroxychloride, not an oxychlorine — and the
        // first version of this, a greedy `ClO` group in
        // `oxyanion_groups`, took its chloride and one of its three
        // hydroxide oxygens and called them hypochlorite. The phase lost
        // its registry match and `copper-patina.lab` went back to saying
        // nothing could precipitate. The databases are full of the same
        // shape: `Zn2(OH)3Cl`, `CdOHCl`, `Pb2(OH)3Cl`.
        assert!(
            matches!(role("atacamite"), Some(DerivedRole::Mineral { .. })),
            "atacamite is a mineral, not a hypochlorite salt: {:?}",
            role("atacamite")
        );

        // Honestly unmappable: organics (residual C), gases.
        assert!(role("ethanol").is_none());
        assert!(role("Cl2").is_none());
        // These roles describe already-aqueous analytical feeds. Physical
        // gas portions still enter through the separate finite-gas owner.
        assert_eq!(dissolves("CO2"), vec![("C".into(), 1.0)]);
        assert_eq!(dissolves("HBr"), vec![("Br".into(), 1.0)]);
    }

    #[test]
    fn bookings_cover_every_derivable_element() {
        // Every element any derived role can produce must have a booking ion
        // that exists in the registry.
        let mut elements: Vec<String> = Vec::new();
        for s in species::REGISTRY {
            match role(s.key) {
                Some(DerivedRole::Dissolves(c))
                | Some(DerivedRole::Mineral { elements: c, .. }) => {
                    for (el, _) in c {
                        if !elements.contains(el) {
                            elements.push(el.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        for el in &elements {
            let ion = booking_ion(el).unwrap_or_else(|| panic!("no booking ion for {el}"));
            assert!(
                species::lookup_key(ion).is_some(),
                "booking ion {ion} for {el} missing from registry"
            );
        }
    }

    #[test]
    fn phase_table_is_derived() {
        let sylvite = phase_by_name("Sylvite").expect("Sylvite derived from pitzer");
        assert_eq!(sylvite.species, "KCl");
        let gypsum = phase_by_name("Gypsum").expect("Gypsum derived");
        assert_eq!(gypsum.waters, 2.0);
        let epsomite = phase_by_name("Epsomite").expect("Epsomite derived");
        assert_eq!(epsomite.species, "epsomite");
        assert_eq!(epsomite.waters, 7.0);
        // BRD-012.S02: the sulfate test's solid is the database's own
        // Barite phase, paired to the registry by formula alone.
        let barite = phase_by_name("Barite").expect("Barite derived from wateq4f");
        assert_eq!(barite.species, "BaSO4");
        assert_eq!(barite.waters, 0.0);
        assert!(candidate_phases().iter().any(|p| p.name == "Barite"));
        // Aragonite lost the polymorph dedupe to calcite, so it is not a
        // *candidate* — but it stays resolvable by name, because readback
        // must be able to name whatever polymorph a routed database posed.
        assert!(!candidate_phases().iter().any(|p| p.name == "Aragonite"));
        let aragonite = phase_by_name("Aragonite").expect("polymorphs stay resolvable");
        assert_eq!(aragonite.species, "CaCO3");
        assert!(phase_by_name("Calcite").is_some());
    }

    /// Pinned to the vendored databases. These numbers are rendered to the
    /// reader, so a vendor bump that moves them should be noticed and the
    /// sentence re-read, not silently re-printed.
    ///
    /// Re-pinned 672 → 683 on 2026-08-23 (OPT-8): the unified formula
    /// parser reads nested parentheses and decimal solid-solution
    /// occupancies the old dbindex parser was blind to — eleven real
    /// minerals (the cobalt ammine complexes, jarosite and its
    /// hydronium/sodium family, the autunites) joined the index, each
    /// hand-verified against its formula.
    #[test]
    fn phase_coverage_reports_how_little_the_datasets_share() {
        let c = phase_coverage();
        assert_eq!(c.total, 683, "distinct minerals across all three datasets");
        assert_eq!(c.shared, 21, "minerals every dataset defines");
        assert_eq!(
            c.per_database,
            vec![("wateq4f", 296), ("minteq.v4", 547), ("pitzer", 64)]
        );
        // Gases are excluded, and one of them would otherwise land in the
        // shared count: all three datasets define CO2(g). Counting phases
        // rather than minerals reads as 22 of 707, which overstates both
        // the agreement and the shelf it is agreement about.
        let idx: Vec<_> = DB_TAGS.iter().map(|t| index_for(t)).collect();
        let mut every: BTreeSet<&str> = BTreeSet::new();
        for i in &idx {
            every.extend(i.phases.keys().map(String::as_str));
        }
        // 696 → 707 with OPT-8: the same eleven parser-dividend minerals,
        // counted here before polymorph dedupe.
        // The reviewed HBr gas adds one phase, not a shelf mineral.
        assert_eq!(every.len(), 708, "phases, gases included");
        assert_eq!(every.iter().filter(|n| n.ends_with("(g)")).count(), 25);
        assert!(
            idx.iter().all(|i| i.has_phase("CO2(g)")),
            "CO2(g) is shared by all three, so it inflates the shared count"
        );
        // The point the renderer needs to be able to make: agreement is the
        // exception, so three answers are partly about three different
        // shelves of admissible solids.
        assert!(c.shared * 10 < c.total);
    }

    #[test]
    fn citric_acid_is_one_citrate_and_not_two_acetates() {
        // C6H8O7 admits two whole C2H3O2 units by arithmetic, which is
        // why the citrate row has to come first in the group table. What
        // the databases actually define is Citrate, and minteq.v4 alone
        // defines it.
        assert_eq!(dissolves("citric_acid"), vec![("Citrate".into(), 1.0)]);
        assert!(
            index_for("minteq.v4").has_element("Citrate"),
            "minteq.v4 is the only shipped database with citrate chemistry"
        );
        for tag in ["wateq4f", "pitzer"] {
            assert!(
                !index_for(tag).has_element("Citrate"),
                "{tag} is not expected to define Citrate"
            );
        }
        // The citrate the solve books back has to be a registry species,
        // or the rebuild panics instead of returning the mass.
        assert_eq!(booking_ion("Citrate"), Some("C6H5O7-3"));
    }

    #[test]
    fn sugars_and_malic_acid_are_not_decomposed_into_acetate() {
        // The free-acid guard. Each of these is a whole number of acetate
        // units plus protons, and before the guard each of them entered
        // solution as acetate and acidified it:
        //   glucose/fructose C6H12O6 = 3 x C2H3O2 + 3 H
        //   malic acid       C4H6O5  = 2 x C2H3O2 + O
        // None of them has a cation to balance those units, so none of
        // them is that salt.
        //   lactic acid      C3H6O3  = one and a half acetate units, which
        //                              is not a whole number of them, so
        //                              the arithmetic cannot even reach for
        //                              the salt — and the residue rules
        //                              reject the leftover carbon anyway.
        // Lactic acid is NOT in this list any more, and the distinction is
        // the point of the guard rather than an exception to it. The guard
        // rejects a molecule that would enter as SEVERAL anion skeletons —
        // glucose as three acetates, malic acid as two — because a free
        // acid carries one. Lactic acid carries exactly one lactate, so it
        // was never the arithmetic-dressed-as-speciation case; it was
        // simply an anion no loaded database defined. Now that
        // `databases::minteq_v4()` defines it, the honest answer is one
        // lactate, and `lactate_is_defined_in_exactly_one_loaded_database`
        // asserts it.
        for key in ["glucose", "fructose", "malic_acid"] {
            assert!(
                role(key).is_none(),
                "{key} must have no derived aqueous role, got {:?}",
                role(key)
            );
        }
        // Cellulose has none either, but for the ordinary reason: its
        // residue simply is not derivable.
        assert!(role("cellulose").is_none());
        // The guard must not have cost the salts anything: a real acetate
        // and a real acetate salt still decompose.
        assert_eq!(dissolves("CH3COOH"), vec![("Acetate".into(), 1.0)]);
        assert_eq!(
            dissolves("NaOAc"),
            vec![("Acetate".into(), 1.0), ("Na".into(), 1.0)]
        );
        // And lactic acid enters as ONE lactate, not as an acetate unit
        // plus a leftover the residue rules would have to swallow.
        assert_eq!(dissolves("lactic_acid"), vec![("Lactate".into(), 1.0)]);
    }

    #[test]
    fn malate_is_in_none_of_the_shipped_databases() {
        // The premise of the spoken refusal. If a database ever gains a
        // malate species this test fails, and the refusal should be
        // replaced by a computed pH rather than left standing.
        for tag in DB_TAGS {
            assert!(
                !index_for(tag).has_element("Malate"),
                "{tag} now defines Malate — malic acid can be speciated, so \
                 UNSPECIATED_ACIDS should lose its row"
            );
        }
        assert!(UNSPECIATED_ACIDS.iter().any(|(k, _)| *k == "malic_acid"));
    }

    #[test]
    fn lactate_is_defined_in_exactly_one_loaded_database() {
        // This test used to assert the opposite, and said so: "if a loaded
        // database ever gains the species this test says the refusal must
        // go." It has, so it did.
        //
        // The species is not in the vendored file. `databases::minteq_v4()`
        // is minteq.v4 plus one reviewed lactate definition, taken from
        // llnl-organics' own log K — see that constant for why the
        // enthalpy is deliberately left behind. So the assertion is
        // narrow on purpose: exactly one database has it, because that is
        // the one this lab extended, and the other two must not silently
        // acquire it.
        assert!(
            index_for("minteq.v4").has_element("Lactate"),
            "the lactate extension is not reaching the parsed index — a \
             species the engine has and the ledger does not is worse than \
             one neither has"
        );
        for tag in ["wateq4f", "pitzer"] {
            assert!(
                !index_for(tag).has_element("Lactate"),
                "{tag} now defines Lactate; the extension is meant for minteq.v4 alone"
            );
        }
        // And the refusal is gone, because the acidity is modelled now.
        assert!(
            !UNSPECIATED_ACIDS.iter().any(|(k, _)| *k == "lactic_acid"),
            "lactic acid can be speciated; its UNSPECIATED_ACIDS row is a \
             false statement about what this bench cannot do"
        );
        // Malate's row stays: nothing has been added for it.
        assert!(UNSPECIATED_ACIDS.iter().any(|(k, _)| *k == "malic_acid"));
    }

    /// EXP-57: the atmospheric CO2 pressure exists in two places now —
    /// here as a log10, because a saturation index is what PHREEQC wants,
    /// and in `kerotakis_core::clock` as an absolute pressure, because
    /// that is what a rate multiplies. They are ONE physical constant, and
    /// somebody editing either has no way to see the other. This notices.
    ///
    /// The same discipline as the enthalpy basis: a number that has been
    /// derived a second way is only trustworthy while something still
    /// compares the two.
    #[test]
    fn the_atmospheric_co2_pressure_agrees_with_the_clock() {
        let (_, _, log_p) = ATMOSPHERIC
            .iter()
            .find(|(phase, ..)| *phase == "CO2(g)")
            .expect("CO2 is the atmospheric reservoir");
        let as_atm = 10f64.powf(*log_p);
        let clock = kerotakis_core::clock::ATMOSPHERIC_CO2_ATM;
        assert!(
            (as_atm - clock).abs() < clock * 1e-3,
            "10^{log_p} is {as_atm} atm but the gas-exchange clock drives \
             at {clock} atm; they are the same constant"
        );
    }
}
