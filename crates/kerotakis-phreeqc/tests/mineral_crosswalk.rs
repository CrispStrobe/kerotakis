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
/// 2026-11-10: the registry-solid coverage lane adds eight common
/// laboratory precipitates the routed databases already defined (AgBr,
/// AgI, Ag2CO3, Ag3PO4, BaCO3, PbCO3, PbSO4, PbI2; see
/// `docs/registry-solid-coverage-20261110.md`). Each pairs with exactly one
/// phase name shared by wateq4f and minteq.v4, so the crosswalk grows by
/// eight rows, eight phase names leave the unmatched count, and none of the
/// eight carries a reviewed solubility: the gap grows from 24 to 32.
///
/// 2026-11-21: the soluble-salt class (A2) adds KBr, CuCl2, Na3PO4 and
/// Al2(SO4)3. Only CuCl2 is a database phase (Melanothallite), so the gap
/// grows by one and the other three join the non-database solids.
///
/// 2026-11-21 (H3): potassium dichromate and potassium chromate join the
/// registry; minteq.v4 spells both as phases and both carry a reviewed
/// solubility (CRC/Merck via HSDB), so two rows join the crosswalk with a
/// solubility. Both are unbookable (no registry chromium ion) and are never
/// posed. Sodium nitrite joins too and is not a database phase.
///
/// 2026-11-24: the alkali-metal + water class (A3) adds the metals Na, K
/// and Ca. None is a database phase, so the gap is unchanged and all three
/// join the non-database solids.
///
/// 2026-11-24 (C1 sulfur lane): CuS, PbS and ZnS join the registry so
/// aqueous sulfide can precipitate them; all three are database phases
/// (covellite, galena, sphalerite, ZnS under several names), with no
/// reviewed solubility claimed, so the gap grows by three.
///
/// The gap was 24 before that. Nine of the 33 have been sourced to papers that were
/// read in full rather than to a handbook — four on the morning of
/// 2026-09-21 and five more the same day, once `saturation_moves` stopped
/// being solution-blind and the measurements it could not hold could ship.
/// The 24 that remain are still counted in full, which is the deliberate
/// pessimism, and `PLAN.md` says of each one why no value could be
/// sourced.
///
/// 2026-11-24 (missing-species lane): five registry solids join. MgCl2 and
/// urea are not database phases. CoCl2, bischofite (MgCl2·6H2O) and
/// magnetite are database phases without a reviewed solubility, so the gap
/// grows by three; magnetite is not posable (the route books iron as
/// iron(II), and Fe3O4 holds iron(II) and iron(III) together).
///
/// 2026-11-25 (redox species): KHSO4, K2SO4 and Cu(NO3)2 join. K2SO4 is a
/// database phase (arcanite) without a reviewed solubility; the other two
/// are not database phases.
///
/// 2026-11-25 (complex salts): five registry solids join. K4[Fe(CN)6] and
/// K3[Fe(CN)6] dissolve and are not phases; Prussian blue, Cu2[Fe(CN)6] and
/// Zn2[Fe(CN)6] are minteq.v4 slice phases (cited Ksp) matched through the
/// conserved hexacyanoferrate pseudo-element, so the gap grows by three.
/// Seven slice phases join the 683 distinct minerals (690).
///
/// 2026-11-26 (chromium(VI) speciation): minteq.v4.dat's K2CrO4 and
/// K2Cr2O7 phases become posable now that a Cr(6) total books on its own
/// ions, so two rows leave the not-posable count.
///
/// 2026-11-27 (chromate solids): PbCrO4, Ag2CrO4 and BaCrO4 join, each a
/// minteq.v4 phase under its formula without a reviewed solubility, so the
/// gap grows by three and three phases gain a registry solid.
///
/// 2026-11-27 (hydroxide redissolution): Al(OH)3 and Ni(OH)2 join, both
/// database phases without a reviewed solubility (gap +2). Al(OH)3 matches
/// three phase names (Gibbsite, wateq4f `Al(OH)3(a)`, minteq.v4
/// `Al(OH)3(am)`), Ni(OH)2 one, so four names gain a registry solid and
/// one more row has several names. NOT YET CHECKED BY A RUN: pinned from
/// the phase blocks by hand; a failure prints the true totals.
///
/// 2026-11-28 (nitrate decomposition): potassium nitrite joins as a registry
/// solid that is not a database phase (registry solids and the
/// not-a-database-phase count each +1; the gate printed these totals).
/// 2026-11-28 (haloform): iodoform joins the same way (+1 each).
#[test]
fn the_gap_is_counted_from_both_datasets() {
    assert_eq!(
        solubility_gap(),
        SolubilityGap {
            registry_solids: 144,
            database_phases: 68,
            with_reviewed_solubility: 14,
            without_reviewed_solubility: 54,
            not_a_database_phase: 76,
            phases_without_a_registry_solid: 588,
            several_phase_names: 15,
            not_posable: 12,
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
            "AgCl", "BaSO4", "Ca(OH)2", "CaCO3", "CuO", "Fe(OH)3", "K2Cr2O7", "K2CrO4", "KCl",
            "Mg(OH)2", "NaCl", "S", "SiO2", "gypsum",
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
            "Ag2CO3",
            // 2026-11-27: the chromate solids, minteq.v4 log K only.
            "Ag2CrO4",
            "Ag3PO4",
            "AgBr",
            "AgI",
            // 2026-11-27: gibbsite and the amorphous hydroxides; no reviewed
            // solubility is claimed.
            "Al(OH)3",
            "Al2O3",
            "BaCO3",
            "BaCrO4",
            "Ca3(PO4)2",
            "CaO",
            "CoCl2",
            "Cu",
            "Cu(OH)2",
            "Cu2(OH)2CO3",
            // 2026-11-25: hexacyanoferrate precipitates carry a cited Ksp in
            // the minteq.v4 slice, not a reviewed bench solubility.
            "Cu2Fe(CN)6",
            "Cu2O",
            // A2 (2026-11-21): anhydrous CuCl2 is Melanothallite in both
            // routed databases; no reviewed solubility is claimed.
            "CuCl2",
            // 2026-11-24: covellite, galena and sphalerite; no reviewed
            // solubility is claimed for any of the three sulfides.
            "CuS",
            "CuSO4",
            "Fe(OH)2",
            "Fe2O3",
            "Fe3O4",
            // 2026-11-25: arcanite; no reviewed solubility is claimed.
            "K2SO4",
            "MgCO3",
            "MgO",
            "MnO2",
            "Na2SO4",
            "NaHCO3",
            "Ni(OH)2",
            "Pb",
            "PbCO3",
            "PbCrO4",
            "PbI2",
            "PbS",
            "PbSO4",
            "Prussian_blue",
            "Zn",
            "Zn(OH)2",
            "Zn2Fe(CN)6",
            "ZnCO3",
            "ZnO",
            "ZnS",
            "ZnSO4",
            "antlerite",
            "atacamite",
            "bischofite",
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

/// Named phases may remain outside aqueous coverage because the route
/// cannot book their oxidation states or elemental components. They stay in the crosswalk because they are part of
/// the honest answer to "which registry solids do the databases spell";
/// they are excluded from the candidate list for reasons recorded in
/// `derived::Derived::build`, and a count that dropped them silently would
/// be answering a quieter question.
#[test]
fn unbookable_database_phases_are_never_posed() {
    let rows = mineral_crosswalk();
    let not_posable: Vec<&str> = rows
        .iter()
        .filter(|r| !r.posable)
        .map(|r| r.species)
        .collect();
    assert_eq!(
        not_posable,
        vec![
            "Ag",
            "Al2O3",
            "Cu",
            "Cu2(OH)2CO3",
            "Cu2O",
            "Fe2O3",
            "Fe3O4",
            "MnO2",
            "Pb",
            "S",
            "SiO2",
            "Zn"
        ]
    );
    for key in &not_posable {
        assert!(
            !derived::candidate_phases()
                .iter()
                .any(|p| p.species == *key),
            "{key} must not be a candidate equilibrium phase"
        );
    }
}

/// `docs/registry-solid-coverage-20261110.json` is generated by
/// `tools/registry_solid_coverage.py` from the vendored database files and
/// the registry document, with its own reader. This holds the runtime
/// crosswalk to it in both directions, so neither the generator's reading
/// nor the runtime's can drift without a failing test.
#[test]
fn registry_solid_coverage_is_current() {
    use std::collections::{BTreeMap, BTreeSet};
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/registry-solid-coverage-20261110.json"
    );
    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("coverage JSON")).unwrap();
    let mut table: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for row in doc["formulas"].as_array().unwrap() {
        let Some(species) = row["registry_species"].as_str() else {
            assert_ne!(row["class"], "registry-solid", "{row}");
            continue;
        };
        assert_eq!(row["class"], "registry-solid", "{row}");
        let entry = table.entry(species.to_string()).or_default();
        for phase in row["phases"].as_array().unwrap() {
            entry.insert((
                phase["database"].as_str().unwrap().to_string(),
                phase["phase"].as_str().unwrap().to_string(),
            ));
        }
    }
    let runtime: BTreeMap<String, BTreeSet<(String, String)>> = mineral_crosswalk()
        .into_iter()
        .map(|row| {
            (
                row.species.to_string(),
                row.phases
                    .into_iter()
                    .map(|(tag, name)| (tag.to_string(), name))
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        runtime, table,
        "the runtime crosswalk and docs/registry-solid-coverage-20261110.json disagree; \
         rerun tools/registry_solid_coverage.py and review"
    );
}

/// The eight laboratory precipitates the coverage lane added, each paired
/// with the phase both Debye-Hueckel routes spell for it and booked as a
/// mineral role, which is what the precipitation readback keys on.
#[test]
fn the_added_laboratory_precipitates_are_routed_minerals() {
    for (key, phase) in [
        ("AgBr", "Bromyrite"),
        ("AgI", "Iodyrite"),
        ("Ag2CO3", "Ag2CO3"),
        ("Ag3PO4", "Ag3PO4"),
        ("BaCO3", "Witherite"),
        ("PbCO3", "Cerussite"),
        ("PbSO4", "Anglesite"),
        ("PbI2", "PbI2"),
    ] {
        for tag in ["wateq4f", "minteq.v4"] {
            assert_eq!(
                derived::routed_phase_for_solid(key, tag),
                Some(phase),
                "{key} on {tag}"
            );
        }
        assert_eq!(
            derived::routed_phase_for_solid(key, "pitzer"),
            None,
            "{key}: pitzer defines no such phase"
        );
        match derived::role(key) {
            Some(derived::DerivedRole::Mineral { phase: p, .. }) => {
                assert_eq!(p, phase, "{key}")
            }
            other => panic!("{key}: expected a mineral role, got {other:?}"),
        }
        let solid = derived::phase_by_name(phase).expect("candidate phase");
        assert_eq!(solid.species, key, "{phase} must read back as {key}");
    }
}
