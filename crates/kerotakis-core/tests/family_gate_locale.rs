//! Presentation controls for the real family gate router, not chemical yields.
//! Ledger-only fixture records bypass structural matching. The oracle supplies
//! one named group solely for the forbidden-group diagnostic control.
use std::collections::BTreeMap;

use kerotakis_core::family::*;
use kerotakis_core::{
    Equilibrator, Event, Kelvin, Locale, Moles, Phase, SpeciesId, Vessel, VesselId,
};

struct GateOracle;
impl StructureOracle for GateOracle {
    fn groups_of(&self, key: &str) -> Option<Vec<String>> {
        (key == "ethanol").then(|| vec!["alcohol".into()])
    }
    fn apply(
        &self,
        _: &FamilyRecord,
        _: &[&str],
    ) -> Result<Option<Vec<String>>, kerotakis_core::Phrase> {
        panic!("ledger-only gate fixtures must not invoke structural matching")
    }
}

fn router(gates: GateSet) -> FamilyRouter<GateOracle> {
    FamilyRouter::new(
        GateOracle,
        vec![FamilyRecord {
            id: "gate-locale-control".into(),
            version: 7,
            smirks: String::new(),
            substrates: Vec::new(),
            ledger_reactants: BTreeMap::from([("ethanol".into(), 1.0)]),
            ledger_products: BTreeMap::from([("ethanol".into(), 1.0)]),
            gates,
            priority: 0,
            outcome: OutcomeModel::ToCompletion,
            confidence: FamilyConfidence::CuratedInstance,
            provenance: "Presentation fixture; no chemical yield assertion".into(),
            refusal_domain: "Gate diagnostics only".into(),
        }],
    )
}

fn vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("ethanol"), Moles(0.01), Phase::Liquid);
    v
}

fn check(router: &FamilyRouter<GateOracle>, v: &Vessel, gate: &str, de: &str, fr: &str) {
    let before = serde_json::to_value(v).unwrap();
    let evaluation = router.evaluate(v);
    assert_eq!(evaluation.declined.len(), 1);
    assert!(evaluation.ready.is_empty());
    let decline = &evaluation.declined[0];
    assert_eq!(decline.gate, gate);
    let detail = decline
        .detail
        .as_ref()
        .expect("dynamic reason retains its recipe");
    assert_eq!(detail.render(Locale::EN), decline.reason);
    let events = router.time_boundaries(v);
    let phrase = match &events[..] {
        [Event::NotYetModeled {
            reason: Some(reason),
            ..
        }] => reason,
        _ => panic!("one real family time-boundary event expected: {events:?}"),
    };
    for (locale, label) in [(Locale::parse("de"), de), (Locale::parse("fr"), fr)] {
        let text = phrase.render(locale);
        assert!(text.contains(label), "{text}");
        assert!(
            !text.contains(&decline.reason),
            "English reason leaked: {text}"
        );
    }
    assert_eq!(serde_json::to_value(v).unwrap(), before);
}

#[test]
fn dynamic_temperature_windows_are_localized_at_multiple_coordinates() {
    for (min, max) in [
        (Some(350.0), Some(400.0)),
        (Some(350.0), None),
        (None, Some(280.0)),
    ] {
        let router = router(GateSet {
            temperature_k: Some(KelvinWindow { min, max }),
            ..GateSet::default()
        });
        for t in [298.15, 310.0] {
            let mut v = vessel();
            v.temperature = Kelvin(t);
            check(&router, &v, "temperature_k", "Temperatur", "température");
        }
    }
}

#[test]
fn missing_ph_and_catalyst_alternatives_keep_localized_reasons() {
    let v = vessel();
    let cases = [
        (
            GateSet {
                ph: Some(PhWindow {
                    min: Some(3.0),
                    max: Some(6.0),
                }),
                ..GateSet::default()
            },
            "ph",
        ),
        (
            GateSet {
                catalyst: Some(CatalystGate::AnyOf {
                    keys: vec!["H2SO4".into(), "H+".into(), "unlisted-catalyst".into()],
                }),
                ..GateSet::default()
            },
            "catalyst",
        ),
    ];
    for (gates, gate) in cases {
        let router = router(gates);
        check(
            &router,
            &v,
            gate,
            if gate == "ph" { "pH" } else { "Katalysator" },
            if gate == "ph" { "pH" } else { "catalyseur" },
        );
        if gate == "catalyst" {
            let e = router.time_boundaries(&v);
            let Event::NotYetModeled {
                reason: Some(p), ..
            } = &e[0]
            else {
                unreachable!()
            };
            assert!(p.render(Locale::parse("de")).contains(" oder "));
            assert!(p.render(Locale::parse("fr")).contains(" ou "));
            assert!(p.render(Locale::parse("de")).contains("unlisted-catalyst"));
        }
    }
}

#[test]
fn medium_light_and_unknown_solvent_labels_use_localized_explanations() {
    for medium in [
        MediumGate::Aqueous,
        MediumGate::Dry,
        MediumGate::OrganicSolvent {
            solvent: "unlisted-solvent".into(),
        },
    ] {
        check(
            &router(GateSet {
                medium: Some(medium),
                ..GateSet::default()
            }),
            &vessel(),
            "medium",
            "Medium",
            "milieu",
        );
    }
    check(
        &router(GateSet {
            light: Some(LightGate {
                max_wavelength_nm: 400.0,
            }),
            ..GateSet::default()
        }),
        &vessel(),
        "light",
        "Licht",
        "lumière",
    );
    let mut wet = vessel();
    wet.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    check(
        &router(GateSet {
            medium: Some(MediumGate::Anhydrous),
            ..GateSet::default()
        }),
        &wet,
        "medium",
        "Medium",
        "milieu",
    );
}

#[test]
fn required_and_forbidden_group_explanations_are_localized() {
    check(
        &router(GateSet {
            required_groups: vec!["ester".into()],
            ..GateSet::default()
        }),
        &vessel(),
        "required_groups",
        "erforderliche",
        "requis",
    );
    check(
        &router(GateSet {
            forbidden_groups: vec!["alcohol".into()],
            ..GateSet::default()
        }),
        &vessel(),
        "forbidden_groups",
        "ausgeschlossene",
        "exclus",
    );
}

#[test]
fn gate_order_english_legacy_and_machine_identity_are_preserved() {
    let router = router(GateSet {
        medium: Some(MediumGate::Aqueous),
        temperature_k: Some(KelvinWindow {
            min: Some(500.0),
            max: None,
        }),
        ..GateSet::default()
    });
    let v = vessel();
    let e = router.evaluate(&v);
    assert_eq!(e.declined.len(), 1);
    let d = &e.declined[0];
    assert_eq!(d.family, "gate-locale-control");
    assert_eq!(d.version, 7);
    assert_eq!(d.gate, "medium");
    assert_eq!(
        d.reason,
        "the family runs in water, and this vessel holds no water-majority liquid"
    );
    let capability = router.capability(&v);
    let kerotakis_core::Applicability::NotApplicable { reason } = capability.applicability else {
        panic!("must decline")
    };
    assert_eq!(reason, "gate-locale-control v7 declined at medium: the family runs in water, and this vessel holds no water-majority liquid");
}
