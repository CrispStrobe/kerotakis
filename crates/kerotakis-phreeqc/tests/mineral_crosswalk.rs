//! The crosswalk between two namespaces this repository ships: registry
//! solid keys and the phase names the vendored PHREEQC databases use.
//!
//! Why it is a test and not a table. `aqueous::potential_molality` caps
//! what an equilibrium phase may contribute to the routing estimate by the
//! reviewed solubility of the solid the phase is booked as, and counts the
//! solid in full where the registry holds none — deliberately, because
//! that fallback is what keeps eight molal sodium chloride routing to
//! pitzer. So the cap bites only where the data exists, and "how large is
//! the gap" is a question nobody can answer by looking at names: this
//! bench writes `AgCl`, and both databases that spell silver chloride
//! spell it `Cerargyrite`.
//!
//! Every number below is read out of `vendor/iphreeqc/**/*.dat` and
//! `data/registry/registry-source-v1.json` through the shipped parsers.
//! Nothing here is a list of what we expect to find — the scar this repo
//! carries is a lint whose denominator came from the catalogue instead of
//! the source and reported 100% coverage over 325 untranslated strings.
//! Pinning the totals is what makes a vendor bump or a registry addition
//! arrive as a failing test with the new numbers in the message, instead
//! of as a silently different answer.

use kerotakis_phreeqc::derived::{self, mineral_crosswalk, solubility_gap, SolubilityGap, DB_TAGS};

/// Pinned to the vendored databases and the shipped registry, 2026-09-21.
///
/// `PLAN.md` recorded nine — the names that happen to match as *strings*
/// across the two namespaces. The composition match finds 36, and the gap
/// it named was 33 rather than 9.
///
/// The gap is now 24. Nine of the 33 have been sourced to papers that were
/// read in full rather than to a handbook — four on the morning of
/// 2026-09-21 and five more the same day, once `saturation_moves` stopped
/// being solution-blind and the measurements it could not hold could ship.
/// The 24 that remain are still counted in full, which is the deliberate
/// pessimism, and `PLAN.md` says of each one why no value could be
/// sourced.
#[test]
fn the_gap_is_counted_from_both_datasets() {
    assert_eq!(
        solubility_gap(),
        SolubilityGap {
            registry_solids: 93,
            database_phases: 36,
            with_reviewed_solubility: 12,
            without_reviewed_solubility: 24,
            not_a_database_phase: 57,
            phases_without_a_registry_solid: 621,
            several_phase_names: 12,
            not_posable: 4,
        },
        "the crosswalk has drifted from the shipped databases or registry"
    );
}

/// The twelve solids where the routing cap can bite today, and the ones
/// where it cannot. Asserted as whole lists so a drift prints the new
/// membership rather than a changed integer.
///
/// THE LIST IS NO LONGER ALL SPARINGLY SOLUBLE, AND THAT IS THE POINT OF
/// THE CHANGE THAT PUT HALITE AND SYLVITE ON IT.
/// `kerotakis_core::solve::saturation_moves` reads the same field and
/// moves a solid into solution as an UNDISSOCIATED aqueous portion up to
/// it. It used to be SOLUTION-BLIND — it knew the limit and nothing about
/// what else was in the beaker — and that, rather than magnitude, is what
/// bounded this field. `AgCl` at 1.05e-5 mol/L is barely above chalk and
/// still broke `codex lint`: the `common-ion-effect` entry teaches that
/// silver chloride in 0.01 mol/L salt water dissolves nothing measurable,
/// and the cap dissolved it anyway because it could not see the chloride.
///
/// It can now. For a salt of two monatomic ions the pure-water figure is
/// read back as the solubility product it implies and answered in the
/// solution the beaker actually holds, so the five measurements #699
/// sourced and held back are shipped here. What has NOT changed is the
/// pessimism for a polyatomic anion: carbonate, sulfate and hydroxide
/// still keep the pure-water cap, because a product cannot be
/// reconstructed from one mass figure without assuming which species
/// carries the anion. `PLAN.md` says so at the ruling.
#[test]
fn which_database_phases_have_a_reviewed_solubility() {
    let rows = mineral_crosswalk();
    let with: Vec<&str> = rows
        .iter()
        .filter(|r| r.reviewed_solubility)
        .map(|r| r.species)
        .collect();
    assert_eq!(
        with,
        vec![
            "AgCl", "BaSO4", "Ca(OH)2", "CaCO3", "CuO", "Fe(OH)3", "KCl", "Mg(OH)2", "NaCl", "S",
            "SiO2", "gypsum",
        ],
        "solids that are database phases AND carry a reviewed solubility"
    );

    let without: Vec<&str> = rows
        .iter()
        .filter(|r| !r.reviewed_solubility)
        .map(|r| r.species)
        .collect();
    assert_eq!(
        without,
        vec![
            "Ag",
            "Ca3(PO4)2",
            "CaO",
            "Cu",
            "Cu(OH)2",
            "CuSO4",
            "Fe(OH)2",
            "Fe2O3",
            "MgO",
            "MnO2",
            "Na2SO4",
            "NaHCO3",
            "Pb",
            "Zn",
            "Zn(OH)2",
            "ZnSO4",
            "antlerite",
            "atacamite",
            "brochantite",
            "chalcanthite",
            "epsomite",
            "hydroxylapatite",
            "langite",
            "octacalcium_phosphate",
        ],
        "the gap: database phases the routing cap counts in full"
    );
}

/// The two solids from the transcript that started this, and the reason a
/// string match could not have found either of them.
#[test]
fn a_registry_key_is_not_a_phase_name() {
    let rows = mineral_crosswalk();
    let names = |key: &str| -> Vec<String> {
        let row = rows
            .iter()
            .find(|r| r.species == key)
            .unwrap_or_else(|| panic!("{key} is not in the crosswalk"));
        let mut n: Vec<String> = row.phases.iter().map(|(_, n)| n.clone()).collect();
        n.sort();
        n.dedup();
        n
    };

    // Silver chloride. PLAN.md guessed `Chlorargyrite`, which is what
    // llnl.dat and Thermoddem call it — neither of which this bench
    // loads. The two databases we ship both write `Cerargyrite`.
    assert_eq!(names("AgCl"), vec!["Cerargyrite".to_string()]);
    for tag in DB_TAGS {
        assert!(
            !derived::index_for(tag).has_phase("Chlorargyrite"),
            "{tag} does not spell silver chloride that way"
        );
    }

    // Manganese dioxide is three phase names, not one and not two, and
    // the row carries all three: they are polymorphs of the same solid
    // and choosing between them is mineralogy, not bookkeeping.
    assert_eq!(
        names("MnO2"),
        vec![
            "Birnessite".to_string(),
            "Nsutite".to_string(),
            "Pyrolusite".to_string()
        ]
    );
}

/// The crosswalk claims a phase name in a named database. Check every one
/// of those claims against that database's own index — this is the guard
/// that fails if the vendored files move under us.
#[test]
fn every_claimed_phase_exists_in_the_database_it_is_claimed_for() {
    for row in mineral_crosswalk() {
        assert!(
            !row.phases.is_empty(),
            "{} is in the crosswalk with no phase",
            row.species
        );
        for (tag, name) in &row.phases {
            let idx = derived::index_for(tag);
            assert!(
                idx.has_phase(name),
                "{} claims {name} in {tag}, which does not define it",
                row.species
            );
            let info = &idx.phases[name];
            assert!(!info.is_gas, "{name} is a gas, not a mineral");
            assert_eq!(
                derived::registry_solid_for(&info.composition, info.waters),
                Some(row.species),
                "{name} in {tag} no longer composes to {}",
                row.species
            );
        }
    }
}

/// Four rows are database phases that are deliberately never posed as
/// equilibrium phases. They stay in the crosswalk because they are part of
/// the honest answer to "which registry solids do the databases spell";
/// they are excluded from the candidate list for reasons recorded in
/// `derived::Derived::build`, and a count that dropped them silently would
/// be answering a quieter question.
#[test]
fn the_metals_are_phases_that_are_never_posed() {
    let rows = mineral_crosswalk();
    let not_posable: Vec<&str> = rows
        .iter()
        .filter(|r| !r.posable)
        .map(|r| r.species)
        .collect();
    assert_eq!(not_posable, vec!["Ag", "Cu", "Pb", "Zn"]);
    for key in &not_posable {
        assert!(
            !derived::candidate_phases()
                .iter()
                .any(|p| p.species == *key),
            "{key} must not be a candidate equilibrium phase"
        );
    }
}
