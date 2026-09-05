//! BRD-031: the component-identity seam.
//!
//! A row in `kerotakis_thermo::pack` is bound to a substance by its
//! Standard InChIKey and by nothing else. The pack cannot check that on its
//! own — it does not know the registry — so the join is proved from this
//! side, where both halves are visible.
//!
//! What this file is defending against is specific. The BRD-030 spike
//! reached a vapour-pressure correlation by matching an uppercased display
//! name (`"WATER" => vle::WATER`), which is the seam BREADTH calls
//! positional and forbids for BRD-032. A name join fails silently in two
//! directions: a renamed species quietly loses its parameters, and two
//! substances sharing a common name quietly share one. An InChIKey join
//! fails loudly or not at all.

use kerotakis_core::species::{self, SpeciesId};
use kerotakis_thermo::pack::{self, FluidParameter};

#[test]
fn every_pack_row_resolves_to_exactly_one_registry_species_by_inchikey() {
    for row in pack::rows() {
        let matches: Vec<&'static str> = species::registry()
            .iter()
            .filter(|data| data.inchikey == row.identity.inchikey)
            .map(|data| data.key)
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "{} ({}) resolves to {:?} registry species, not one",
            row.identity.species_key,
            row.identity.inchikey,
            matches.len()
        );
        assert_eq!(
            matches[0], row.identity.species_key,
            "{} claims registry key '{}' but its InChIKey resolves to '{}'",
            row.identity.inchikey, row.identity.species_key, matches[0]
        );
    }
}

#[test]
fn every_pack_row_agrees_with_the_registry_in_both_directions() {
    for row in pack::rows() {
        let data = species::lookup(&SpeciesId::new(row.identity.species_key))
            .unwrap_or_else(|| panic!("registry has no species '{}'", row.identity.species_key));
        assert_eq!(
            data.inchikey, row.identity.inchikey,
            "'{}' carries InChIKey {} in the registry and {} in the pack",
            row.identity.species_key, data.inchikey, row.identity.inchikey
        );
    }
}

#[test]
fn a_registry_key_is_not_a_pack_key() {
    // The whole point of the seam: the lookup takes an InChIKey, so the
    // runtime key and the display name cannot reach a correlation even by
    // accident.
    for row in pack::rows() {
        assert!(
            pack::row_by_inchikey(row.identity.species_key).is_none(),
            "'{}' selected a pack row by registry key",
            row.identity.species_key
        );
        assert!(
            pack::row_by_inchikey(row.identity.name).is_none(),
            "'{}' selected a pack row by display name",
            row.identity.name
        );
    }
}

#[test]
fn the_pack_covers_the_fluids_the_corpus_and_kids_lab_reach_for() {
    // Coverage here means "has an identity and a recorded rights position",
    // not "has numbers". Four of these refuse every parameter, and saying
    // so by name is the deliverable.
    for key in [
        "water",
        "ethanol",
        "methanol",
        "propanone",
        "isopropanol",
        "CH3COOH",
        "ethyl_acetate",
        "hexane",
        "CO2",
        "N2",
        "O2",
    ] {
        let data = species::lookup(&SpeciesId::new(key))
            .unwrap_or_else(|| panic!("registry has no species '{key}'"));
        assert!(
            pack::row_by_inchikey(data.inchikey).is_some(),
            "no pack row for {key} ({})",
            data.inchikey
        );
    }
}

#[test]
fn a_species_outside_the_pack_refuses_rather_than_borrowing_a_neighbour() {
    let salt = species::lookup(&SpeciesId::new("NaCl")).expect("registry has NaCl");
    assert!(
        pack::row_by_inchikey(salt.inchikey).is_none(),
        "sodium chloride found a fluid row"
    );
}

#[test]
fn no_fluid_in_the_pack_claims_a_liquid_density_the_registry_cannot_back() {
    // The registry's `density` is one reviewed number near 25 °C. BRD-031
    // has cleared no rho(T) model for any fluid, so every row must refuse
    // the temperature-dependent question rather than return the constant
    // dressed up as a correlation.
    for row in pack::rows() {
        let refusal = row
            .liquid_density_g_per_ml(60.0)
            .expect_err("no fluid has a cleared density correlation");
        assert_eq!(refusal.parameter(), FluidParameter::LiquidDensity);
    }
}
