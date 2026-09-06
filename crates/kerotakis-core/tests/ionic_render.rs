//! The net ionic equation reaches a reader at the right register, in the
//! right language (GUI-092).
//!
//! The equation itself is chemical notation and stays identical in every
//! language; only the label around it is translated. A test rather than a
//! comment, because "translate the equation too" is a plausible-looking
//! thing for a future contributor to do.

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
    let net = kerotakis_core::net_ionic(&precipitation(), &brine_with_silver())
        .expect("a solved precipitation is derivable");

    assert_eq!(render_ionic(&net, Register::LV1), None);
    assert_eq!(
        render_ionic(&net, Register::LV2).as_deref(),
        Some("v1: net ionic: Ag⁺(aq) + Cl⁻(aq) → AgCl(s)")
    );
}

#[test]
fn lv3_names_the_ions_that_stayed_out_of_it() {
    let net = kerotakis_core::net_ionic(&precipitation(), &brine_with_silver()).unwrap();
    let line = render_ionic(&net, Register::LV3).expect("lv3 renders");
    assert!(line.contains("Ag⁺(aq) + Cl⁻(aq) → AgCl(s)"), "{line}");
    assert!(line.contains("spectator ions: Na⁺, NO₃⁻"), "{line}");
}

#[test]
fn german_translates_the_label_and_leaves_the_chemistry_alone() {
    let net = kerotakis_core::net_ionic(&precipitation(), &brine_with_silver()).unwrap();
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
    let net = kerotakis_core::net_ionic(&precipitation(), &brine_with_silver()).unwrap();
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
