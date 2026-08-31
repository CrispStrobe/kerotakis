//! BRD-013: the USDA FoodData Central adapter, held to its own promises.
//!
//! Every test here runs from bytes that are checked into the repository. There
//! is no network client in this crate's dependency graph and no test reaches
//! for one; `no_network_client_is_reachable_from_this_crate` is the gate that
//! keeps it that way.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use kerotakis_data::usda::{
    candidates, canonical_report_bytes, conflicts, import_report, map_snapshot, parse_snapshot,
    promotion_policy, proposed_eligible_fields, CarbohydrateClosure, ComponentDisposition,
    Derivation, FoodComposition, FoodNutrient, FoodRecord, NutrientMeta, NutrientRejection,
    ReconciliationConflict, UnresolvedReason, ADAPTER_ID, AGGREGATE_ASH, AGGREGATE_FAT,
    AGGREGATE_FIBRE, AGGREGATE_OTHER_CARBOHYDRATE, AGGREGATE_PROTEIN, BASIS_GRAMS, BASIS_UNIT,
    LICENCE, MIN_TOLERANCE_GRAMS,
};
use kerotakis_data::{
    canonical_quarantine_bytes, default_runtime_data_licences, identity_conflicts, lint_promotion,
    normalize_quantity_for, review_candidates, Dimension, EligibleFieldList, FieldRejectionReason,
    PromotionLintInput, PromotionPolicy, QuarantinedCandidate, SnapshotManifest,
};

// The pinned Foundation Foods records this fixture is built from.
const SALT: u64 = 746775;
const GRANULATED_SUGAR: u64 = 746784;
const SOYBEAN_OIL: u64 = 748366;
const BUTTER: u64 = 789828;
const WHEAT_FLOUR: u64 = 789951;
const APPLE_JUICE: u64 = 2003590;
const ORANGE_JUICE: u64 = 2003597;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/quarantine/usda-fdc-v1")
        .join(name)
}

fn read(name: &str) -> Vec<u8> {
    std::fs::read(fixture(name)).unwrap_or_else(|error| panic!("read {name}: {error}"))
}

fn manifest() -> SnapshotManifest {
    serde_json::from_slice(&read("manifest.json")).expect("manifest parses")
}

fn compositions() -> Vec<FoodComposition> {
    let records = parse_snapshot(&read("raw/snapshot.json")).expect("snapshot parses");
    map_snapshot(&records)
}

fn find(compositions: &[FoodComposition], fdc_id: u64) -> &FoodComposition {
    compositions
        .iter()
        .find(|composition| composition.fdc_id == fdc_id)
        .unwrap_or_else(|| panic!("fixture is missing fdcId {fdc_id}"))
}

/// The species ids the shipped registry actually carries.
fn installed_species() -> BTreeSet<String> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/registry/registry-source-v1.json");
    let bytes = std::fs::read(&path).expect("registry source is readable");
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("registry parses");
    document["identities"]
        .as_array()
        .expect("identities is an array")
        .iter()
        .filter_map(|identity| identity["id"].as_str().map(str::to_owned))
        .collect()
}

// ── the pinned snapshot ─────────────────────────────────────────────────────

#[test]
fn the_manifest_pins_the_committed_snapshot_bytes() {
    let manifest = manifest();
    assert_eq!(manifest.adapter_id, ADAPTER_ID);
    manifest
        .verify(&read("raw/snapshot.json"))
        .expect("the committed bytes hash to the manifest");
    assert_eq!(manifest.record_count as usize, compositions().len());
}

#[test]
fn a_snapshot_whose_bytes_moved_is_refused() {
    let mut raw = read("raw/snapshot.json");
    // One byte of one description.
    let index = raw.iter().position(|byte| *byte == b'F').expect("byte");
    raw[index] = b'f';
    let error = manifest().verify(&raw).expect_err("tampered bytes refuse");
    assert!(
        format!("{error}").contains("checksum mismatch"),
        "{error:?}"
    );
}

#[test]
fn rebuilding_from_the_pinned_snapshot_is_byte_identical() {
    let first = canonical_quarantine_bytes(candidates(&compositions())).unwrap();
    let second = canonical_quarantine_bytes(candidates(&compositions())).unwrap();
    assert_eq!(first, second, "the same bytes twice");
    assert_eq!(
        first,
        read("candidates.json"),
        "the checked-in candidate fixture is what this adapter produces"
    );
}

#[test]
fn the_checked_in_policy_report_and_eligible_lists_are_what_the_adapter_produces() {
    let compositions = compositions();
    let candidates = candidates(&compositions);
    assert_eq!(
        canonical_report_bytes(&promotion_policy(&candidates)).unwrap(),
        read("policy.json")
    );
    assert_eq!(
        canonical_report_bytes(&proposed_eligible_fields(&candidates)).unwrap(),
        read("eligible.json")
    );
    assert_eq!(
        canonical_report_bytes(&import_report(&compositions)).unwrap(),
        read("import-report.json")
    );
}

#[test]
fn only_foundation_records_parse() {
    let mut records = parse_snapshot(&read("raw/snapshot.json")).unwrap();
    records[0].data_type = "Branded".to_owned();
    let raw = serde_json::to_vec(&records).unwrap();
    let error = parse_snapshot(&raw).expect_err("a Branded record is refused");
    assert!(format!("{error}").contains("not Foundation"), "{error:?}");
}

#[test]
fn a_snapshot_that_repeats_a_record_is_refused() {
    let mut records = parse_snapshot(&read("raw/snapshot.json")).unwrap();
    let duplicate = records[0].clone();
    records.push(duplicate);
    let raw = serde_json::to_vec(&records).unwrap();
    let error = parse_snapshot(&raw).expect_err("a repeated record is refused");
    assert!(format!("{error}").contains("twice"), "{error:?}");
}

// ── reconciliation ──────────────────────────────────────────────────────────

#[test]
fn every_food_either_reconciles_or_is_a_reported_conflict() {
    let compositions = compositions();
    let shipped: BTreeSet<u64> = candidates(&compositions)
        .iter()
        .map(|candidate| candidate.external_record_id.parse().unwrap())
        .collect();
    let reported: BTreeSet<u64> = conflicts(&compositions)
        .iter()
        .map(|composition| composition.fdc_id)
        .collect();

    assert!(!shipped.is_empty(), "some food must reconcile");
    assert!(
        !reported.is_empty(),
        "the fixture must exercise the refusal path too"
    );
    assert!(shipped.is_disjoint(&reported));
    assert_eq!(shipped.len() + reported.len(), compositions.len());

    for composition in &compositions {
        let reconciliation = &composition.reconciliation;
        assert_eq!(reconciliation.basis_grams, BASIS_GRAMS);
        assert!(
            reconciliation.tolerance_grams >= MIN_TOLERANCE_GRAMS,
            "{}: tolerance never falls below the reporting precision",
            composition.description
        );
        if reconciliation.reconciles() {
            let ledger = reconciliation.resolved_grams + reconciliation.named_unresolved_grams;
            assert!(
                (ledger - reconciliation.total_grams).abs() < 1e-9,
                "{}: the two halves of the ledger are the whole ledger",
                composition.description
            );
            assert!(
                reconciliation.residual_grams.abs() <= reconciliation.tolerance_grams,
                "{}: {} g residual against {} g of stated uncertainty",
                composition.description,
                reconciliation.residual_grams,
                reconciliation.tolerance_grams
            );
        }
    }
}

#[test]
fn a_record_missing_a_proximate_is_a_conflict_rather_than_an_assumed_zero() {
    let compositions = compositions();
    for fdc_id in [SALT, BUTTER, SOYBEAN_OIL] {
        let composition = find(&compositions, fdc_id);
        let missing = composition
            .reconciliation
            .conflicts
            .iter()
            .find_map(|conflict| match conflict {
                ReconciliationConflict::MissingProximate { missing } => Some(missing),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} states every proximate; the fixture changed",
                    composition.description
                )
            });
        assert!(!missing.is_empty());
        // Nothing was invented to fill the hole.
        assert!(
            composition.reconciliation.total_grams < BASIS_GRAMS,
            "{}: an unstated proximate must leave the ledger short, not padded",
            composition.description
        );
    }
}

#[test]
fn the_tolerance_comes_from_the_record_and_not_from_the_adapter() {
    let compositions = compositions();
    let juice = find(&compositions, ORANGE_JUICE);
    let stated: f64 = juice
        .components
        .iter()
        .filter_map(|component| component.spread_grams)
        .sum();
    assert!(stated > MIN_TOLERANCE_GRAMS);
    assert!((juice.reconciliation.tolerance_grams - stated).abs() < 1e-9);

    // A record that states no spread at all falls back to the reporting
    // precision, never to something roomier.
    let bare = kerotakis_data::usda::map_record(&synthetic_record(vec![
        nutrient(1051, "Water", "g", 50.0, None),
        nutrient(1003, "Protein", "g", 10.0, None),
        nutrient(1004, "Total lipid (fat)", "g", 10.0, None),
        nutrient(1007, "Ash", "g", 5.0, None),
        nutrient(1005, "Carbohydrate, by difference", "g", 25.0, None),
    ]));
    assert_eq!(bare.reconciliation.tolerance_grams, MIN_TOLERANCE_GRAMS);
    assert!(bare.reconciliation.reconciles());
}

#[test]
fn a_carbohydrate_that_cannot_hold_its_own_determinations_is_a_conflict() {
    let over = kerotakis_data::usda::map_record(&synthetic_record(vec![
        nutrient(1051, "Water", "g", 10.0, None),
        nutrient(1003, "Protein", "g", 5.0, None),
        nutrient(1004, "Total lipid (fat)", "g", 5.0, None),
        nutrient(1007, "Ash", "g", 5.0, None),
        nutrient(1005, "Carbohydrate, by difference", "g", 75.0, None),
        nutrient(1010, "Sucrose", "g", 90.0, None),
    ]));
    let conflict = over
        .reconciliation
        .conflicts
        .iter()
        .find(|conflict| {
            matches!(
                conflict,
                ReconciliationConflict::CarbohydrateOverSubscribed { .. }
            )
        })
        .expect("15 g of carbohydrate cannot hold 90 g of sucrose");
    match conflict {
        ReconciliationConflict::CarbohydrateOverSubscribed { excess_grams, .. } => {
            assert!((excess_grams - 15.0).abs() < 1e-9);
        }
        other => panic!("{other:?}"),
    }
    assert!(candidates(&[over]).is_empty(), "a conflict does not ship");
}

// ── mapping honesty ─────────────────────────────────────────────────────────

#[test]
fn aggregate_nutrients_never_become_molecules() {
    for composition in compositions() {
        for label in [
            AGGREGATE_PROTEIN,
            AGGREGATE_FAT,
            AGGREGATE_ASH,
            AGGREGATE_FIBRE,
            AGGREGATE_OTHER_CARBOHYDRATE,
        ] {
            let Some(component) = composition.component(label) else {
                continue;
            };
            assert_eq!(
                component.disposition,
                ComponentDisposition::NamedUnresolved {
                    reason: UnresolvedReason::AggregatePopulation
                },
                "{}: {label} must stay a named population",
                composition.description
            );
        }
    }
}

#[test]
fn a_food_without_individual_sugar_determinations_keeps_its_carbohydrate_unresolved() {
    let compositions = compositions();
    let flour = find(&compositions, WHEAT_FLOUR);
    assert!(
        !flour.sugars_reported_individually,
        "the flour record determines no individual sugar"
    );
    for label in ["sucrose", "glucose", "fructose", "maltose", "starch"] {
        assert!(
            flour.component(label).is_none(),
            "{label} was invented for a record that never determined it"
        );
    }
    let other = flour
        .component(AGGREGATE_OTHER_CARBOHYDRATE)
        .expect("all of it stays unresolved");
    // 73.2 g of carbohydrate by difference, none of it accounted for.
    assert!((other.grams_per_basis - 73.2).abs() < 1e-9, "{other:?}");
    // Water is the only molecule the record establishes.
    let water = flour.component("water").expect("water");
    assert!(
        (flour.reconciliation.resolved_grams - water.grams_per_basis).abs() < 1e-9,
        "{:?}",
        flour.reconciliation
    );
}

#[test]
fn individually_determined_sugars_and_acids_become_species_proposals() {
    let compositions = compositions();
    let juice = find(&compositions, ORANGE_JUICE);
    assert!(juice.sugars_reported_individually);
    let proposed: BTreeMap<&str, &str> = juice
        .components
        .iter()
        .filter_map(|component| {
            component
                .species_id()
                .map(|species| (component.label.as_str(), species))
        })
        .collect();
    for (label, species) in [
        ("water", "water"),
        ("sucrose", "sucrose"),
        ("glucose", "glucose"),
        ("fructose", "fructose"),
        ("citric_acid", "citric_acid"),
        ("malic_acid", "malic_acid"),
        ("ascorbic_acid", "ascorbic_acid"),
    ] {
        assert_eq!(proposed.get(label), Some(&species), "{label}");
    }
    // A determined zero is not a component: the record says lactose is absent.
    assert!(juice.component("lactose").is_none());

    // Apple juice is the same machinery reaching a different answer: malic
    // acid is determined, citric acid is not, and nothing fills the gap.
    let apple = find(&compositions, APPLE_JUICE);
    assert!(apple.component("malic_acid").is_some());
    assert!(
        apple.component("citric_acid").is_none(),
        "the apple-juice record determines no citric acid"
    );
}

#[test]
fn a_compound_the_registry_cannot_name_stays_named_and_unresolved() {
    // Lactose and galactose are determined individually, so their mass is
    // known — but Kerotakis has no species for either. Folding them into an
    // anonymous remainder would lose the one thing upstream did establish.
    let composition = kerotakis_data::usda::map_record(&synthetic_record(vec![
        nutrient(1051, "Water", "g", 87.0, None),
        nutrient(1003, "Protein", "g", 3.0, None),
        nutrient(1004, "Total lipid (fat)", "g", 3.0, None),
        nutrient(1007, "Ash", "g", 2.0, None),
        nutrient(1005, "Carbohydrate, by difference", "g", 5.0, None),
        nutrient(1013, "Lactose", "g", 4.8, None),
    ]));
    let lactose = composition.component("lactose").expect("named, not merged");
    assert_eq!(
        lactose.disposition,
        ComponentDisposition::NamedUnresolved {
            reason: UnresolvedReason::NoRegistrySpecies
        }
    );
    assert!((lactose.grams_per_basis - 4.8).abs() < 1e-9);
    assert!(composition.reconciliation.reconciles());
}

#[test]
fn minerals_are_an_element_inventory_and_never_a_species() {
    // USDA measures how much sodium is in the food, never which salt it was
    // in. Nothing may turn that into an ion or a salt.
    let forbidden: BTreeSet<&str> = ["Na+", "K+", "Ca+2", "Mg+2", "Cl-", "NaCl", "CaCO3", "Fe"]
        .into_iter()
        .collect();
    for composition in compositions() {
        for component in &composition.components {
            if let Some(species) = component.species_id() {
                assert!(
                    !forbidden.contains(species),
                    "{}: {species} was speciated out of an elemental total",
                    composition.description
                );
            }
        }
        for mineral in &composition.mineral_elements {
            assert!(
                composition.rejections.iter().any(|rejection| matches!(
                    rejection,
                    NutrientRejection::ElementalTotalNotSpeciated { element, .. }
                        if element == &mineral.element
                )),
                "{}: {} is inventoried but not reported as unspeciated",
                composition.description,
                mineral.element
            );
        }
        // Whatever the minerals are, they are inside the ash that was weighed.
        if let Some(ash) = composition.component(AGGREGATE_ASH) {
            let minerals: f64 = composition
                .mineral_elements
                .iter()
                .map(|mineral| mineral.grams_per_basis)
                .sum();
            assert!(
                minerals <= ash.grams_per_basis + composition.reconciliation.tolerance_grams,
                "{}: {minerals} g of minerals inside {} g of ash",
                composition.description,
                ash.grams_per_basis
            );
        }
    }
}

#[test]
fn table_salt_does_not_become_sodium_chloride() {
    let compositions = compositions();
    let salt = find(&compositions, SALT);
    let sodium = salt
        .mineral_elements
        .iter()
        .find(|mineral| mineral.element == "Na")
        .expect("the record determines sodium");
    assert!(sodium.grams_per_basis > 35.0);
    assert!(
        salt.mineral_elements
            .iter()
            .all(|mineral| mineral.element != "Cl"),
        "the record determines no chlorine at all, so NaCl is an inference"
    );
    assert!(
        salt.components
            .iter()
            .all(|component| component.species_id() != Some("NaCl")),
        "an adapter that names NaCl here is guessing"
    );
}

#[test]
fn determinations_that_are_not_masses_are_typed_rejections_that_keep_their_spelling() {
    let compositions = compositions();
    let mut spellings: BTreeSet<String> = BTreeSet::new();
    for composition in &compositions {
        for rejection in &composition.rejections {
            if let NutrientRejection::UnitIsNotAMass { unit, .. } = rejection {
                spellings.insert(unit.clone());
            }
        }
    }
    assert!(
        spellings.contains("kcal"),
        "energy must be refused, not converted: {spellings:?}"
    );
    assert!(spellings.iter().any(|unit| unit == "kJ" || unit == "IU"));
}

#[test]
fn a_total_that_restates_the_ledger_is_rejected_rather_than_double_counted() {
    // A record that states `Sugars, Total` and no individual sugar has not
    // determined any sugar. The total is refused and the carbohydrate stays
    // unresolved.
    let composition = kerotakis_data::usda::map_record(&synthetic_record(vec![
        nutrient(1051, "Water", "g", 10.0, None),
        nutrient(1003, "Protein", "g", 10.0, None),
        nutrient(1004, "Total lipid (fat)", "g", 5.0, None),
        nutrient(1007, "Ash", "g", 5.0, None),
        nutrient(1005, "Carbohydrate, by difference", "g", 70.0, None),
        nutrient(1063, "Sugars, Total", "g", 40.0, Some("AS")),
    ]));
    assert!(!composition.sugars_reported_individually);
    assert!(composition.component("sucrose").is_none());
    assert!(
        (composition
            .component(AGGREGATE_OTHER_CARBOHYDRATE)
            .unwrap()
            .grams_per_basis
            - 70.0)
            .abs()
            < 1e-9
    );
    assert!(composition.rejections.iter().any(|rejection| matches!(
        rejection,
        NutrientRejection::DuplicateTotal {
            nutrient_id: 1063,
            ..
        }
    )));
    assert!(composition.reconciliation.reconciles());
}

#[test]
fn a_by_difference_closure_is_reported_as_one() {
    let compositions = compositions();
    assert_eq!(
        find(&compositions, WHEAT_FLOUR).reconciliation.closure,
        CarbohydrateClosure::ByDifference,
        "upstream's own closure term must not be presented as an independent check"
    );
    assert_eq!(
        find(&compositions, SOYBEAN_OIL).reconciliation.closure,
        CarbohydrateClosure::Absent
    );
}

#[test]
fn granulated_sugar_clamps_its_oversubscription_instead_of_carrying_it_negative() {
    let compositions = compositions();
    let sugar = find(&compositions, GRANULATED_SUGAR);
    assert!(
        sugar.reconciliation.oversubscribed_within_uncertainty,
        "99.83 g of determined sugars against 99.6 g of carbohydrate"
    );
    assert!(sugar.component(AGGREGATE_OTHER_CARBOHYDRATE).is_none());
    assert!(sugar
        .components
        .iter()
        .all(|component| component.grams_per_basis >= 0.0));
    assert!(sugar.reconciliation.reconciles());
}

#[test]
fn every_registry_gap_is_named_and_every_other_proposal_already_exists() {
    let compositions = compositions();
    let installed = installed_species();
    let gaps: BTreeSet<String> = kerotakis_data::usda::registry_gaps(&compositions, &installed)
        .into_iter()
        .map(|gap| gap.species_id)
        .collect();

    for species in ["water", "sucrose", "maltose", "starch", "ascorbic_acid"] {
        assert!(
            installed.contains(species),
            "{species} is expected in the shipped registry"
        );
        assert!(!gaps.contains(species));
    }
    for composition in &compositions {
        for component in &composition.components {
            if let Some(species) = component.species_id() {
                assert!(
                    installed.contains(species) || gaps.contains(species),
                    "{species} is neither installed nor reported as a gap"
                );
            }
        }
    }
    // Nothing here promotes a species; a gap is a report, not a blocker.
    assert!(!candidates(&compositions).is_empty());
}

// ── quarantine, review, promotion ───────────────────────────────────────────

#[test]
fn no_bare_mass_spelling_reaches_a_candidate() {
    for candidate in candidates(&compositions()) {
        for (name, field) in &candidate.fields {
            assert_eq!(field.licence, LICENCE, "{name}");
            assert!(!field.source_field.trim().is_empty(), "{name}");
            let Some(unit) = &field.unit else {
                continue;
            };
            assert_eq!(unit, BASIS_UNIT, "{name} carries a bare mass");
            let value = field.value.as_f64().expect("a quantity is a number");
            let _ = normalize_quantity_for(value, unit, &Dimension::MassPerMass)
                .unwrap_or_else(|error| panic!("{name}: {error}"));
        }
    }
}

#[test]
fn review_accepts_composition_and_refuses_import_bookkeeping() {
    let candidates = candidates(&compositions());
    let policy = promotion_policy(&candidates);
    let report = review_candidates(candidates.clone(), &policy);
    assert_eq!(report.reviews.len(), candidates.len());

    let juice = report
        .reviews
        .iter()
        .find(|review| review.external_record_id.parse::<u64>() == Ok(ORANGE_JUICE))
        .expect("orange juice was reviewed");
    assert!(juice.accepted.contains_key("material.component.water"));
    assert!(juice.accepted.contains_key("material.unresolved.protein"));
    assert!(juice.accepted.contains_key("material.mineral_element.K"));
    assert_eq!(
        juice.accepted["material.component.water"].unit,
        Some(kerotakis_data::Unit {
            symbol: "1".to_owned(),
            dimension: Dimension::MassPerMass,
        }),
        "a mass fraction, normalized out of the record's own basis"
    );
    // The record's own grams per 100 g become that mass fraction and nothing
    // else: 88.5 g/100g is 0.885.
    let grams = find(&compositions(), ORANGE_JUICE)
        .component("water")
        .expect("water")
        .grams_per_basis;
    assert!(
        (juice.accepted["material.component.water"]
            .value
            .as_f64()
            .unwrap()
            - grams / 100.0)
            .abs()
            < 1e-12,
        "{grams} g/100g"
    );

    let rejected: BTreeSet<&str> = juice
        .rejected
        .iter()
        .filter(|rejection| rejection.reason == FieldRejectionReason::UnallowlistedField)
        .map(|rejection| rejection.field.as_str())
        .collect();
    for bookkeeping in ["data_type", "ndb_number", "sample_input_foods"] {
        assert!(
            rejected.contains(bookkeeping),
            "{bookkeeping} must not be promotable: {rejected:?}"
        );
    }
}

#[test]
fn the_promotion_dry_run_passes_for_every_eligible_field() {
    let compositions = compositions();
    let candidates = candidates(&compositions);
    let policy = promotion_policy(&candidates);
    let eligible = proposed_eligible_fields(&candidates);
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest(),
        raw_snapshot: &read("raw/snapshot.json"),
        candidates: &candidates,
        policy: &policy,
        allowed_runtime_licences: &default_runtime_data_licences(),
        eligible_fields: &eligible,
    });
    assert!(
        !report.refuses(),
        "the clean flow must pass: {:?}",
        report.violations
    );
    assert_eq!(report.checked_records, candidates.len());
    assert!(report.checked_fields > 0);
}

#[test]
fn every_planted_violation_refuses() {
    let raw = read("raw/snapshot.json");
    let manifest = manifest();
    let clean = candidates(&compositions());
    let policy = promotion_policy(&clean);
    let licences = default_runtime_data_licences();

    let tainted: Vec<QuarantinedCandidate> =
        serde_json::from_slice(&read("candidates-tainted.json")).expect("tainted fixture parses");
    let invalid: Vec<EligibleFieldList> =
        serde_json::from_slice(&read("eligible-invalid.json")).expect("invalid fixture parses");
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: &raw,
        candidates: &tainted,
        policy: &policy,
        allowed_runtime_licences: &licences,
        eligible_fields: &invalid,
    });
    let reasons: BTreeSet<String> = report
        .violations
        .iter()
        .map(|violation| {
            serde_json::to_value(violation).unwrap()["violation"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    for expected in [
        "licence_not_allowed_for_runtime",
        "missing_field_provenance",
        "unit_not_normalized",
        "snapshot_adapter_mismatch",
        "eligible_field_not_on_record",
        "eligible_record_not_in_quarantine",
    ] {
        assert!(
            reasons.contains(expected),
            "missing {expected}: {reasons:?}"
        );
    }

    // A policy that would admit a ShareAlike licence is refused on its own,
    // before any candidate is looked at.
    let breach: PromotionPolicy =
        serde_json::from_slice(&read("policy-licence-breach.json")).expect("policy parses");
    let report = lint_promotion(&PromotionLintInput {
        manifest: &manifest,
        raw_snapshot: &raw,
        candidates: &clean,
        policy: &breach,
        allowed_runtime_licences: &licences,
        eligible_fields: &[],
    });
    assert!(report.violations.iter().any(|violation| {
        serde_json::to_value(violation).unwrap()["violation"] == "policy_admits_non_runtime_licence"
    }));
}

#[test]
fn the_offline_review_command_passes_the_fixture_and_refuses_the_tainted_one() {
    let run = |candidates: &str, eligible: &str| {
        Command::new(env!("CARGO_BIN_EXE_quarantine-review"))
            .args([
                "lint",
                fixture("manifest.json").to_str().unwrap(),
                fixture("raw/snapshot.json").to_str().unwrap(),
                fixture(candidates).to_str().unwrap(),
                fixture("policy.json").to_str().unwrap(),
                fixture(eligible).to_str().unwrap(),
            ])
            .output()
            .expect("run quarantine-review")
    };
    let clean = run("candidates.json", "eligible.json");
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    let tainted = run("candidates-tainted.json", "eligible-invalid.json");
    assert!(!tainted.status.success());
    assert!(String::from_utf8_lossy(&tainted.stderr).contains("promotion refused"));
}

#[test]
fn two_records_claiming_one_legacy_food_are_reported_not_merged() {
    let mut candidates = candidates(&compositions());
    let mut clone = candidates[0].clone();
    clone.external_record_id = format!("{}-duplicate", clone.external_record_id);
    clone.fields.insert(
        "component.water".to_owned(),
        kerotakis_data::CandidateField::new(serde_json::json!(1.0), "synthetic", LICENCE)
            .with_unit(BASIS_UNIT),
    );
    let key = clone
        .identity_key
        .clone()
        .expect("the fixture has NDB keys");
    candidates.push(clone);

    let conflicts = identity_conflicts(&candidates);
    let conflict = conflicts
        .iter()
        .find(|conflict| conflict.identity_key == key)
        .expect("the shared NDB number is reported");
    assert_eq!(conflict.records.len(), 2);
    assert!(conflict
        .differing_fields
        .iter()
        .any(|field| field == "component.water"));
}

// ── the offline promise ─────────────────────────────────────────────────────

#[test]
fn no_network_client_is_reachable_from_this_crate() {
    // BREADTH's inherited rule: network access is build-time only.
    // `tools/fetch-usda-snapshot.py` is the only thing that talks to
    // FoodData Central, and it is not in any build or test path.
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("this crate's manifest is readable");
    for client in [
        "reqwest",
        "ureq",
        "hyper",
        "curl",
        "isahc",
        "surf",
        "attohttpc",
        "tokio",
    ] {
        assert!(
            !manifest.contains(client),
            "kerotakis-data must not depend on {client}"
        );
    }
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/usda.rs"))
        .expect("the adapter source is readable");
    assert!(
        !source.contains("api.nal.usda.gov"),
        "the adapter reads pinned bytes, never a service"
    );
}

// ── fixture regeneration ────────────────────────────────────────────────────

/// Rewrite every derived fixture from the pinned snapshot.
///
/// Off by default: the other tests assert that what is checked in is what the
/// adapter produces, so regeneration is a deliberate act.
/// `KEROTAKIS_REGENERATE_USDA_FIXTURES=1 cargo test -p kerotakis-data usda`.
#[test]
fn regenerate_pinned_fixtures() {
    if std::env::var("KEROTAKIS_REGENERATE_USDA_FIXTURES").as_deref() != Ok("1") {
        return;
    }
    let compositions = compositions();
    let candidates = candidates(&compositions);
    let write = |name: &str, bytes: Vec<u8>| {
        std::fs::write(fixture(name), bytes).unwrap_or_else(|error| panic!("{name}: {error}"));
    };

    write(
        "candidates.json",
        canonical_quarantine_bytes(candidates.clone()).unwrap(),
    );
    write(
        "policy.json",
        canonical_report_bytes(&promotion_policy(&candidates)).unwrap(),
    );
    write(
        "eligible.json",
        canonical_report_bytes(&proposed_eligible_fields(&candidates)).unwrap(),
    );
    write(
        "import-report.json",
        canonical_report_bytes(&import_report(&compositions)).unwrap(),
    );
    // Written for review but deliberately not byte-asserted anywhere: it is a
    // statement about the registry, which other branches move. The invariant
    // that holds regardless lives in
    // `every_registry_gap_is_named_and_every_other_proposal_already_exists`.
    write(
        "registry-gaps.json",
        canonical_report_bytes(&kerotakis_data::usda::registry_gaps(
            &compositions,
            &installed_species(),
        ))
        .unwrap(),
    );

    // Four candidates, each carrying exactly one planted violation, so the
    // refusal report names four distinct reasons rather than one cascade.
    let usable: Vec<QuarantinedCandidate> = candidates
        .iter()
        .filter(|candidate| {
            candidate.fields.contains_key("component.water")
                && candidate.fields.contains_key("unresolved.protein")
        })
        .cloned()
        .collect();
    assert!(
        usable.len() >= 4,
        "the fixture needs four ordinary foods to plant violations in"
    );
    let pick = |index: usize| usable[index].clone();

    let mut wrong_licence = pick(0);
    wrong_licence
        .fields
        .get_mut("component.water")
        .expect("water")
        .licence = "CC-BY-SA-4.0".to_owned();

    let mut no_provenance = pick(1);
    no_provenance
        .fields
        .get_mut("component.water")
        .expect("water")
        .source_field = String::new();

    let mut wrong_unit = pick(2);
    wrong_unit
        .fields
        .get_mut("unresolved.protein")
        .expect("protein")
        .unit = Some("kcal".to_owned());

    let mut wrong_adapter = pick(3);
    wrong_adapter.adapter_id = "usda-fdc-forged".to_owned();

    let tainted = vec![wrong_licence, no_provenance, wrong_unit, wrong_adapter];
    write(
        "candidates-tainted.json",
        canonical_quarantine_bytes(tainted.clone()).unwrap(),
    );

    let invalid = vec![
        EligibleFieldList {
            adapter_id: ADAPTER_ID.to_owned(),
            external_record_id: tainted[0].external_record_id.clone(),
            fields: vec![
                "component.unobtainium".to_owned(),
                "description".to_owned(),
                "description".to_owned(),
            ],
        },
        EligibleFieldList {
            adapter_id: ADAPTER_ID.to_owned(),
            external_record_id: "0".to_owned(),
            fields: vec!["description".to_owned()],
        },
    ];
    write(
        "eligible-invalid.json",
        canonical_report_bytes(&invalid).unwrap(),
    );

    let mut breach = promotion_policy(&candidates);
    breach
        .fields
        .get_mut("component.water")
        .expect("water")
        .allowed_licences
        .insert("CC-BY-SA-4.0".to_owned());
    write(
        "policy-licence-breach.json",
        canonical_report_bytes(&breach).unwrap(),
    );
}

// ── synthetic records ───────────────────────────────────────────────────────

fn nutrient(
    id: u32,
    name: &str,
    unit: &str,
    amount: f64,
    derivation: Option<&str>,
) -> FoodNutrient {
    FoodNutrient {
        nutrient: NutrientMeta {
            id,
            number: None,
            name: name.to_owned(),
            unit_name: unit.to_owned(),
        },
        amount: Some(amount),
        data_points: None,
        min: None,
        max: None,
        median: None,
        derivation: derivation.map(|code| Derivation {
            code: Some(code.to_owned()),
            description: None,
        }),
    }
}

fn synthetic_record(nutrients: Vec<FoodNutrient>) -> FoodRecord {
    FoodRecord {
        fdc_id: 9_000_001,
        description: "Synthetic test food".to_owned(),
        data_type: "Foundation".to_owned(),
        food_class: Some("FinalFood".to_owned()),
        publication_date: Some("1/1/2026".to_owned()),
        ndb_number: None,
        footnote: None,
        food_category: None,
        food_nutrients: nutrients,
        input_foods: Vec::new(),
    }
}
