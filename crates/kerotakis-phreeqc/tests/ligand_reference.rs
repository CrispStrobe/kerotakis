//! Native law-of-mass-action checks, not saved experimental outputs.
#![cfg(feature = "engine")]
use kerotakis_phreeqc::{databases, Phreeqc};

#[test]
fn concentrated_ligand_feed_discloses_activity_model_fallback() {
    use kerotakis_core::*;
    let mut eq = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap();
    for amount in [0.001, 0.6] {
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.deposit(SpeciesId::new("water"), Moles(55.5), Phase::Liquid);
        vessel.deposit(SpeciesId::new("KSCN"), Moles(amount), Phase::Aqueous);
        eq.equilibrate(&mut vessel).unwrap();
        let routing = &vessel
            .solution
            .as_ref()
            .unwrap()
            .provenance
            .as_ref()
            .unwrap()
            .routing;
        assert_eq!(
            routing.contains("not a validated concentrated-mixture prediction"),
            amount > 0.5,
            "{routing}"
        );
    }
}

#[test]
fn reported_thiocyanate_complexes_keep_unknown_spectra_explicit() {
    use kerotakis_core::{
        vessel::{SolutionInfo, SpeciesDetail, Vessel},
        Moles, Phase, SpeciesId, VesselId,
    };
    use kerotakis_phreeqc::complexation::native_to_physical;

    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.solution = Some(SolutionInfo {
        solvent_kg: Some(0.1),
        ph: 2.0,
        pe: None,
        redox: vec![],
        ionic_strength: 0.1,
        provenance: None,
        species: ["Fe(Thiocyanate)+2", "Fe(Thiocyanate)2+"]
            .into_iter()
            .map(|native| SpeciesDetail {
                name: native_to_physical(native).into(),
                molality: 0.001,
                activity: 0.0005,
            })
            .collect(),
    });
    assert_eq!(
        kerotakis_core::solution_optics::spectral_gaps(&vessel),
        vec!["Fe(SCN)+2", "Fe(SCN)2+"]
    );
    assert!(kerotakis_core::appearance::observe(&vessel)
        .words
        .contains("Colour is incomplete"));
}

#[test]
fn reviewed_ligand_ladders_obey_activity_mass_action_on_both_database_routes() {
    let mut checked = 0;
    for (tag, db) in [
        ("wateq4f", databases::wateq4f()),
        ("minteq", databases::minteq_v4()),
    ] {
        let mut engine = Phreeqc::with_database(db).unwrap();
        for scale in [0.001, 0.01, 0.1, 1.0] {
            for ligand in [0.001, 0.01, 0.1] {
                // SOLUTION characterization keeps specified oxidation states;
                // adapter batch redox isolation is checked separately.
                engine.run(&format!("SOLUTION 1\n temp 25\n pH 9\n units mol/kgw\n -water {scale}\n Cu 1e-5\n N(-3) {ligand}\nSELECTED_OUTPUT\n -reset false\n -high_precision true\nUSER_PUNCH\n -headings q1 q2 q3 q4\n10 PUNCH LA(\"CuNH3+2\")-LA(\"Cu+2\")-LA(\"NH3\"), LA(\"Cu(NH3)2+2\")-LA(\"Cu+2\")-2*LA(\"NH3\"), LA(\"Cu(NH3)3+2\")-LA(\"Cu+2\")-3*LA(\"NH3\"), LA(\"Cu(NH3)4+2\")-LA(\"Cu+2\")-4*LA(\"NH3\")\nEND\n")).unwrap();
                for (key, expected) in [("q1", 4.0), ("q2", 7.5), ("q3", 10.3), ("q4", 11.8)] {
                    let actual = engine.last_value(key).unwrap();
                    assert!(
                        (actual - expected).abs() < 1e-7,
                        "{tag}: {key}: {actual} != {expected}"
                    );
                }
                engine.run(&format!("SOLUTION 1\n temp 25\n pH 2\n units mol/kgw\n -water {scale}\n Fe(3) 1e-5\n Thiocyanate {ligand}\nSELECTED_OUTPUT\n -reset false\n -high_precision true\nUSER_PUNCH\n -headings q1 q2 ligand_total\n10 PUNCH LA(\"Fe(Thiocyanate)+2\")-LA(\"Fe+3\")-LA(\"Thiocyanate-\"), LA(\"Fe(Thiocyanate)2+\")-LA(\"Fe+3\")-2*LA(\"Thiocyanate-\"), TOT(\"Thiocyanate\")\nEND\n")).unwrap();
                for (key, expected) in [("q1", 3.0), ("q2", 3.6), ("ligand_total", ligand)] {
                    let actual = engine.last_value(key).unwrap();
                    assert!(
                        (actual - expected).abs() < 1e-7,
                        "{tag}: {key}: {actual} != {expected}"
                    );
                }
                checked += 2;
            }
        }
    }
    assert_eq!(checked, 48);
}
