//! The net ionic equation reaches a reader at the right register, in the
//! right language (GUI-092).
//!
//! The equation itself is chemical notation and stays identical in every
//! language; only the label around it — and the provenance line under it
//! — is translated. A test rather than a comment, because "translate the
//! equation too" is a plausible-looking thing for a future contributor to
//! do.
//!
//! `NetIonic.provenance` was the SIXTH and last member of the welded-prose
//! family (`Inert.why` #626, `NotYetModeled.what` #632, `scene_vessel`
//! #628, `Provenance.routing` #642, `Provenance.dataset`/`.model` #655).
//! It read the English `dataset` and `model` fields because `net_ionic_for`
//! had no `Locale` to render their recipes with, and the German drawer
//! therefore said *… · ion interaction (Pitzer)*. Threading the locale is
//! what these tests hold in place.

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Provenance, SolutionInfo, SpeciesDetail, Vessel, VesselId};
use kerotakis_core::{render_ionic, render_ionic_for, render_ionic_in, Event, Locale, Register};

fn detail(name: &str, molality: f64) -> SpeciesDetail {
    SpeciesDetail {
        name: name.to_string(),
        molality,
        activity: molality,
    }
}

/// A beaker of silver nitrate poured into brine, as the solver leaves it.
fn brine_with_silver() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.temperature = Kelvin::STANDARD;
    v.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: Vec::new(),
        ph: 6.8,
        ionic_strength: 0.09,
        species: vec![
            detail("Na+", 0.086),
            detail("NO3-", 0.059),
            detail("Cl-", 0.027),
            detail("AgCl", 3.6e-7),
            detail("Ag+", 6.7e-9),
            detail("H+", 1.6e-7),
            detail("OH-", 6.3e-8),
        ],
        provenance: Some(Provenance {
            engine: "PHREEQC (IPhreeqc)".into(),
            dataset: "wateq4f.dat".into(),
            model: "Debye–Hückel".into(),
            dataset_sources: Vec::new(),
            routing: "the only aqueous engine wired in this test".into(),
            routing_phrase: None,
            dataset_phrase: None,
            model_phrase: None,
        }),
    });
    v
}

fn precipitation() -> Event {
    Event::Precipitated {
        vessel: VesselId(0),
        species: SpeciesId::new("AgCl"),
        moles: Moles(0.0058),
        dry: false,
    }
}

#[test]
fn lv1_is_told_nothing_and_lv2_gets_the_equation() {
    let net = kerotakis_core::net_ionic(&precipitation(), &brine_with_silver(), Locale::EN)
        .expect("a solved precipitation is derivable");

    assert_eq!(render_ionic(&net, Register::LV1), None);
    assert_eq!(
        render_ionic(&net, Register::LV2).as_deref(),
        Some("v1: net ionic: Ag⁺(aq) + Cl⁻(aq) → AgCl(s)")
    );
}

#[test]
fn lv3_names_the_ions_that_stayed_out_of_it() {
    let net =
        kerotakis_core::net_ionic(&precipitation(), &brine_with_silver(), Locale::EN).unwrap();
    let line = render_ionic(&net, Register::LV3).expect("lv3 renders");
    assert!(line.contains("Ag⁺(aq) + Cl⁻(aq) → AgCl(s)"), "{line}");
    assert!(line.contains("spectator ions: Na⁺, NO₃⁻"), "{line}");
}

#[test]
fn german_translates_the_label_and_leaves_the_chemistry_alone() {
    let net =
        kerotakis_core::net_ionic(&precipitation(), &brine_with_silver(), Locale::parse("de"))
            .unwrap();
    let line = render_ionic_in(&net, Register::LV3, Locale::parse("de")).expect("lv3 renders");
    assert!(line.contains("Netto-Ionengleichung"), "{line}");
    assert!(line.contains("Zuschauerionen"), "{line}");
    // The equation is notation, not prose: it must be character-identical
    // to the English one.
    assert!(line.contains("Ag⁺(aq) + Cl⁻(aq) → AgCl(s)"), "{line}");
}

#[test]
fn a_step_that_derives_nothing_renders_nothing() {
    let v = brine_with_silver();
    let dissolving = Event::Dissolved {
        vessel: VesselId(0),
        species: SpeciesId::new("NaCl"),
        moles: Moles(0.0086),
    };
    assert!(render_ionic_for(
        std::slice::from_ref(&dissolving),
        std::slice::from_ref(&v),
        Register::LV3,
        Locale::EN
    )
    .is_empty());
}

#[test]
fn the_terms_carry_their_charge_and_phase_for_a_client_that_lays_them_out() {
    let net =
        kerotakis_core::net_ionic(&precipitation(), &brine_with_silver(), Locale::EN).unwrap();
    let silver = net
        .reactants
        .iter()
        .find(|t| t.species == "Ag+")
        .expect("silver is a reactant");
    assert_eq!(silver.charge, 1);
    assert_eq!(silver.phase, Phase::Aqueous);
    assert_eq!(silver.label, "Ag⁺");
    assert_eq!(net.products[0].phase, Phase::Solid);
    assert_eq!(net.products[0].species, "AgCl");
}

/// The same beaker, answered by a dataset that carries RECIPES rather than
/// finished English — which is what the aqueous router actually composes.
///
/// `llnl.dat` is a bare NAME and carries no recipe by design: there is no
/// sentence welded to it to translate. The model is the ion-interaction
/// claim, and its German row is the half that used to reach a German
/// reader in English.
fn routed_by_pitzer() -> Vessel {
    let mut v = brine_with_silver();
    v.solution.as_mut().expect("characterised").provenance = Some(Provenance::new(
        "PHREEQC (IPhreeqc, USGS)",
        "llnl.dat",
        Phrase::new(
            "provenance.model.ion-interaction",
            "{name} specific-ion-interaction model (valid at high ionic strength)",
            vec![("name".to_string(), Slot::text("Pitzer"))],
        ),
        Vec::new(),
        Phrase::bare(
            "routing.default-inorganic",
            "the default inorganic aqueous dataset",
        ),
    ));
    v
}

/// The line a German drawer now reads.
///
/// The model is the reader's language and the two NAMES in it — the
/// program and the person — are untouched, because names travel in text
/// slots the catalogue never looks up.
#[test]
fn the_provenance_line_is_german_and_keeps_its_names() {
    let net = kerotakis_core::net_ionic(&precipitation(), &routed_by_pitzer(), Locale::parse("de"))
        .expect("a solved precipitation is derivable");
    let provenance = net.provenance.expect("the vessel records one");
    assert_eq!(
        provenance,
        "PHREEQC (IPhreeqc, USGS) · llnl.dat · Pitzer-Modell der spezifischen \
         Ionenwechselwirkung (gültig bei hoher Ionenstärke)"
    );
    assert!(
        !provenance.contains("specific-ion-interaction"),
        "no English left in it: {provenance}"
    );
}

/// English is unchanged, byte for byte — the property every consumer that
/// is not a reader rests on.
#[test]
fn english_says_exactly_what_it_said_before() {
    let net = kerotakis_core::net_ionic(&precipitation(), &routed_by_pitzer(), Locale::EN).unwrap();
    assert_eq!(
        net.provenance.as_deref(),
        Some(
            "PHREEQC (IPhreeqc, USGS) · llnl.dat · Pitzer specific-ion-interaction model (valid at high ionic strength)"
        )
    );
}

/// A provenance saved before the recipes existed has only its English, and
/// gets it back in every language rather than nothing.
#[test]
fn a_provenance_without_recipes_still_answers_in_german() {
    let net =
        kerotakis_core::net_ionic(&precipitation(), &brine_with_silver(), Locale::parse("de"))
            .unwrap();
    assert_eq!(
        net.provenance.as_deref(),
        Some("PHREEQC (IPhreeqc) · wateq4f.dat · Debye–Hückel")
    );
}
