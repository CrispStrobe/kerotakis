//! The dilute acid and alkali bottles a school chemistry kit ships empty and
//! the learner fills from a supplier: 25% sodium hydroxide, 20% hydrochloric
//! acid, 30% sulfuric acid and 24% ammonia water. Each is a fixed mass-fraction
//! recipe over species the registry already models, so the label is the whole
//! composition claim; each density is the 20 degrees Celsius value of a primary
//! printed table (International Critical Tables vol. III, 1928; NBS Circular 19,
//! 6th ed., 1924), cited on its bulk-density record.

use kerotakis_core::material::lookup;
use kerotakis_core::script::parse_command;
use kerotakis_core::species::SpeciesId;
use kerotakis_core::{Bench, Locale};

const BOTTLES: &[(&str, &str, &str, &str, f64)] = &[
    (
        "sodium_hydroxide_solution_25_percent",
        "Natronlauge 25%",
        "lessive de soude 25%",
        "NaOH",
        0.25,
    ),
    (
        "hydrochloric_acid_20_percent",
        "Salzsäure 20%",
        "acide chlorhydrique 20%",
        "HCl",
        0.20,
    ),
    (
        "sulfuric_acid_30_percent",
        "Schwefelsäure 30%",
        "acide sulfurique 30%",
        "H2SO4",
        0.30,
    ),
    (
        "ammonia_solution_24_percent",
        "Ammoniakwasser 24%",
        "ammoniaque 24%",
        "NH3",
        0.24,
    ),
];

#[test]
fn every_bottle_resolves_in_three_languages_to_its_label_composition() {
    for (key, de, fr, species, fraction) in BOTTLES {
        for (name, lang) in [(*key, None), (*de, Some("de")), (*fr, Some("fr"))] {
            let recipe = lookup(name, lang).unwrap_or_else(|| panic!("{name} resolves"));
            assert_eq!(recipe.canonical_key, *key);
            let expansion = recipe.expand(100.0, 0).expect("fixed expansion");
            let solute = expansion
                .components
                .iter()
                .find(|c| c.species_id == *species)
                .expect("solute present");
            assert!((solute.amount - 100.0 * fraction).abs() < 1e-9, "{key}");
            let water = expansion
                .components
                .iter()
                .find(|c| c.species_id == "water")
                .expect("water present");
            assert!((solute.amount + water.amount - 100.0).abs() < 1e-9, "{key}");
        }
    }
}

#[test]
fn a_bare_lye_name_does_not_pick_a_concentration() {
    // "Natronlauge" names sodium hydroxide solution of any strength; only the
    // labelled bottle may resolve to 25%.
    assert!(lookup("Natronlauge", Some("de")).is_none());
    // The household 5% ammonia cleaner stays its own bottle.
    let cleaner = lookup("ammonia_cleaner_5_percent", None).unwrap();
    let kit = lookup("Ammoniakwasser 24%", Some("de")).unwrap();
    assert_ne!(cleaner.canonical_key, kit.canonical_key);
}

#[test]
fn the_acid_bottle_makes_the_beaker_acidic_and_the_lye_alkaline() {
    let mut acid = Bench::new();
    for line in [
        "new beaker",
        "add v1 water 50mL",
        "add v1 Salzsäure_20% 5mL",
    ] {
        let command = parse_command(line, Locale::EN).expect(line);
        acid.step(command.operator.unwrap()).expect(line);
    }
    // 5 mL at 1.098 g/mL and 20% w/w is 1.098 g of HCl: all of its chlorine is in the beaker.
    let chlorine = acid.total_moles(&SpeciesId("Cl-".to_string())).0
        + acid.total_moles(&SpeciesId("HCl".to_string())).0;
    let expected = 5.0 * 1.098 * 0.20 / 36.461;
    assert!(
        (chlorine - expected).abs() / expected < 1e-3,
        "{chlorine} vs {expected}"
    );

    let mut lye = Bench::new();
    for line in [
        "new beaker",
        "add v1 water 50mL",
        "add v1 Natronlauge_25% 5mL",
    ] {
        let command = parse_command(line, Locale::EN).expect(line);
        lye.step(command.operator.unwrap()).expect(line);
    }
    let sodium = lye.total_moles(&SpeciesId("Na+".to_string())).0
        + lye.total_moles(&SpeciesId("NaOH".to_string())).0;
    let expected = 5.0 * 1.274 * 0.25 / 39.997;
    assert!(
        (sodium - expected).abs() / expected < 1e-3,
        "{sodium} vs {expected}"
    );
    // Negative control: the lye beaker holds no chloride.
    assert_eq!(lye.total_moles(&SpeciesId("Cl-".to_string())).0, 0.0);
}
