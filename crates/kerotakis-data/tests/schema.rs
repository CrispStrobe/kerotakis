use std::collections::BTreeMap;

use kerotakis_data::*;

fn unit(symbol: &str, dimension: Dimension) -> Unit {
    Unit {
        symbol: symbol.to_string(),
        dimension,
    }
}

fn number(value: f64, symbol: &str, dimension: Dimension) -> NumericRecord {
    NumericRecord {
        value,
        unit: unit(symbol, dimension),
        conditions: Applicability::default(),
        uncertainty: Uncertainty::NotReported,
        source_id: "project-fixture".to_string(),
        method: Method::Editorial("schema test fixture".to_string()),
    }
}

fn evidence(detail: &str) -> Evidence {
    Evidence {
        source_id: "project-fixture".to_string(),
        method: Method::Editorial(detail.to_string()),
    }
}

fn document() -> RegistryDocument {
    let mut document = RegistryDocument::empty();
    document.sources.push(SourceRecord {
        id: "project-fixture".to_string(),
        citation: "Project-authored schema fixture".to_string(),
        licence: "AGPL-3.0-or-later".to_string(),
        lane: SourceLane::Runtime,
        origin: None,
        revision: None,
        retrieved: None,
    });
    document.identities.push(IdentityRecord {
        id: "water".to_string(),
        canonical_key: "XLYOFNOQVPJJNP-UHFFFAOYSA-N".to_string(),
        name: "water".to_string(),
        identifiers: BTreeMap::from([(
            "inchikey".to_string(),
            "XLYOFNOQVPJJNP-UHFFFAOYSA-N".to_string(),
        )]),
        synonyms: vec!["oxidane".to_string()],
        evidence: evidence("reviewed identity fixture"),
    });
    document.compositions.push(CompositionRecord {
        id: "water-composition".to_string(),
        species_id: "water".to_string(),
        formula: "H2O".to_string(),
        elements: vec![
            ElementAmount {
                element: "H".to_string(),
                count: NumericRecord {
                    uncertainty: Uncertainty::Exact,
                    method: Method::Derived("parsed from reviewed formula".to_string()),
                    ..number(2.0, "1", Dimension::Dimensionless)
                },
            },
            ElementAmount {
                element: "O".to_string(),
                count: NumericRecord {
                    uncertainty: Uncertainty::Exact,
                    method: Method::Derived("parsed from reviewed formula".to_string()),
                    ..number(1.0, "1", Dimension::Dimensionless)
                },
            },
        ],
        net_charge: NumericRecord {
            uncertainty: Uncertainty::Exact,
            ..number(0.0, "1", Dimension::Dimensionless)
        },
        evidence: evidence("reviewed formula fixture"),
    });
    document
        .phase_thermodynamics
        .push(PhaseThermodynamicRecord {
            id: "water-liquid-cp".to_string(),
            species_id: "water".to_string(),
            phase: Phase::Liquid,
            property: PhaseProperty::MolarHeatCapacity,
            quantity: NumericRecord {
                conditions: Applicability {
                    temperature: Some(Interval {
                        lower: 298.0,
                        upper: 298.2,
                        unit: unit("K", Dimension::Temperature),
                    }),
                    pressure: Some(Interval {
                        lower: 100_000.0,
                        upper: 102_000.0,
                        unit: unit("Pa", Dimension::Pressure),
                    }),
                    phase: Some(Phase::Liquid),
                    ..Applicability::default()
                },
                uncertainty: Uncertainty::Absolute { plus_minus: 0.1 },
                method: Method::Measured("constant-pressure calorimetry".to_string()),
                ..number(75.3, "J.mol-1.K-1", Dimension::MolarHeatCapacity)
            },
        });
    document.transport.push(TransportRecord {
        id: "water-liquid-viscosity".to_string(),
        species_id: "water".to_string(),
        phase: Phase::Liquid,
        property: TransportProperty::DynamicViscosity,
        quantity: number(0.000_89, "Pa.s", Dimension::DynamicViscosity),
    });
    document.optical.push(OpticalRecord {
        id: "water-visible".to_string(),
        species_id: "water".to_string(),
        phase: Phase::Liquid,
        appearance: Some("colourless".to_string()),
        flame_colour: None,
        reflective_srgb: Some("#FFFFFF".to_string()),
        spectrum: vec![SpectralSample {
            wavelength: number(500.0, "nm", Dimension::Wavelength),
            molar_absorptivity: number(0.0, "L.mol-1.cm-1", Dimension::MolarAbsorptivity),
        }],
        evidence: evidence("visual observation fixture"),
    });
    document.safety.push(SafetyRecord {
        id: "water-safety".to_string(),
        species_id: "water".to_string(),
        classifications: vec!["not-classified".to_string()],
        statements: Vec::new(),
        limits: Vec::new(),
        evidence: evidence("safety classification fixture"),
    });
    document.microstates.push(MicrostateRecord {
        id: "water-neutral".to_string(),
        species_id: "water".to_string(),
        label: "neutral singlet".to_string(),
        kind: MicrostateKind::Protonation,
        formal_charge: NumericRecord {
            uncertainty: Uncertainty::Exact,
            ..number(0.0, "1", Dimension::Dimensionless)
        },
        relative_energy: Some(number(0.0, "kJ.mol-1", Dimension::MolarEnergy)),
        equilibrium_fraction: Some(number(1.0, "1", Dimension::Dimensionless)),
        evidence: evidence("microstate fixture"),
    });
    document.model_parameters.push(ModelParameterRecord {
        id: "water-additive-volume".to_string(),
        subject: ModelSubject::Species("water".to_string()),
        model: "additive-liquid-volume".to_string(),
        parameter: "reference-density".to_string(),
        quantity: NumericRecord {
            method: Method::Editorial("current runtime approximation".to_string()),
            ..number(0.997, "g.mL-1", Dimension::MassDensity)
        },
    });
    document.material_recipes.push(MaterialRecipe {
        id: "fixture/tap-water".to_string(),
        version: 1,
        canonical_key: "tap_water".to_string(),
        name: "tap water".to_string(),
        aliases: BTreeMap::from([
            ("en".to_string(), vec!["faucet water".to_string()]),
            ("de".to_string(), vec!["Leitungswasser".to_string()]),
        ]),
        basis: MaterialBasis::MassFraction,
        bulk_density: Some(number(0.997, "g/mL", Dimension::MassDensity)),
        components: vec![MaterialComponent {
            species_id: "water".to_string(),
            fraction: FractionRange {
                lower: 0.98,
                upper: 0.98,
            },
            evidence: evidence("resolved water fraction fixture"),
        }],
        unresolved_fraction: Some(FractionRange {
            lower: 0.02,
            upper: 0.02,
        }),
        physical_form: MaterialPhysicalForm::HomogeneousLiquid,
        roles: Vec::new(),
        preparation: Some("representative unbranded fixture".to_string()),
        lot_assumptions: vec!["room-temperature sample".to_string()],
        substitutions: Vec::new(),
        confidence: MaterialConfidence::Curated,
        expansion_policy: MaterialExpansionPolicy::Fixed,
        evidence: evidence("material identity fixture"),
    });
    document
}

#[test]
fn every_record_family_round_trips_and_validates() {
    let document = document();
    document.validate().expect("valid complete schema fixture");

    let json = serde_json::to_string_pretty(&document).expect("serialise");
    for family in [
        "identities",
        "compositions",
        "phase_thermodynamics",
        "transport",
        "optical",
        "safety",
        "microstates",
        "model_parameters",
        "material_recipes",
    ] {
        assert!(json.contains(&format!("\"{family}\"")), "missing {family}");
    }
    let decoded: RegistryDocument = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(decoded, document);
}

#[test]
fn material_aliases_are_localized_and_never_replace_species() {
    let mut document = document();
    assert_eq!(
        document
            .material_recipe("  LEITUNGSWASSER ", Some("de"))
            .map(|recipe| recipe.canonical_key.as_str()),
        Some("tap_water")
    );
    assert!(document
        .material_recipe("Leitungswasser", Some("en"))
        .is_none());

    document.material_recipes[0].aliases.insert(
        "en".to_string(),
        vec!["water".to_string(), "oxidane".to_string()],
    );
    let text = document
        .validate()
        .expect_err("a material may not shadow a species")
        .to_string();
    assert!(text.contains("overrides a canonical species"), "{text}");
}

#[test]
fn ranged_material_expansion_is_seeded_replayable_and_conserved() {
    let mut document = document();
    {
        let recipe = &mut document.material_recipes[0];
        recipe.components[0].fraction = FractionRange {
            lower: 0.96,
            upper: 0.99,
        };
        recipe.unresolved_fraction = Some(FractionRange {
            lower: 0.01,
            upper: 0.04,
        });
        recipe.expansion_policy = MaterialExpansionPolicy::Seeded {
            salt: "fixture-v1".to_string(),
        };
    }
    document.validate().expect("valid ranged recipe");

    let recipe = &document.material_recipes[0];
    let first = recipe.expand(250.0, 42).expect("positive amount");
    let replay = recipe.expand(250.0, 42).expect("same expansion");
    assert_eq!(first, replay);
    assert_ne!(first, recipe.expand(250.0, 43).unwrap());
    let accounted = first
        .components
        .iter()
        .map(|component| component.amount)
        .sum::<f64>()
        + first.unresolved_amount;
    assert!((accounted - first.total_amount).abs() < 1e-10);
    assert_eq!(first.recipe_version, 1);
}

#[test]
fn invalid_recipe_ranges_and_unresolved_remainders_are_rejected() {
    let mut document = document();
    let recipe = &mut document.material_recipes[0];
    recipe.components[0].fraction = FractionRange {
        lower: 0.8,
        upper: 0.9,
    };
    recipe.unresolved_fraction = Some(FractionRange {
        lower: 0.01,
        upper: 0.02,
    });

    let text = document
        .validate()
        .expect_err("the unresolved range does not conserve the recipe")
        .to_string();
    assert!(
        text.contains("must contain the conserved remainder"),
        "{text}"
    );
    assert!(
        text.contains("fixed expansion requires an exact component fraction"),
        "{text}"
    );
}

#[test]
fn only_reviewed_runtime_sources_may_enter_a_distributed_pack() {
    assert!(SourceLane::Runtime.may_enter_runtime_pack());
    assert!(!SourceLane::BuildOracle.may_enter_runtime_pack());
    assert!(!SourceLane::ExternalOracle.may_enter_runtime_pack());

    assert_eq!(
        serde_json::to_string(&SourceLane::BuildOracle).unwrap(),
        "\"build_oracle\""
    );
    assert_eq!(
        serde_json::to_string(&SourceLane::ExternalOracle).unwrap(),
        "\"external_oracle\""
    );
}

#[test]
fn a_number_requires_a_real_source_and_a_described_method() {
    let mut document = document();
    let quantity = &mut document.phase_thermodynamics[0].quantity;
    quantity.source_id = "missing".to_string();
    quantity.method = Method::Calculated("  ".to_string());

    let error = document.validate().expect_err("invalid provenance");
    let text = error.to_string();
    assert!(text.contains("unknown source id 'missing'"), "{text}");
    assert!(text.contains("method detail is empty"), "{text}");
}

#[test]
fn non_finite_values_and_invalid_uncertainties_are_rejected() {
    let mut document = document();
    let quantity = &mut document.transport[0].quantity;
    quantity.value = f64::NAN;
    quantity.uncertainty = Uncertainty::Relative { fraction: -0.1 };

    let text = document.validate().expect_err("invalid number").to_string();
    assert!(text.contains("must be finite"), "{text}");
    assert!(text.contains("must be finite and non-negative"), "{text}");
}

#[test]
fn property_dimensions_and_condition_dimensions_are_checked() {
    let mut document = document();
    document.phase_thermodynamics[0].quantity.unit.dimension = Dimension::Pressure;
    document.phase_thermodynamics[0]
        .quantity
        .conditions
        .temperature
        .as_mut()
        .unwrap()
        .unit
        .dimension = Dimension::Time;

    let text = document
        .validate()
        .expect_err("invalid dimensions")
        .to_string();
    assert!(text.contains("expected MolarHeatCapacity"), "{text}");
    assert!(text.contains("expected Temperature"), "{text}");
}

#[test]
fn dangling_species_and_duplicate_family_ids_are_rejected() {
    let mut document = document();
    let mut duplicate = document.transport[0].clone();
    duplicate.species_id = "unknown".to_string();
    document.transport.push(duplicate);

    let text = document
        .validate()
        .expect_err("invalid references")
        .to_string();
    assert!(text.contains("duplicate record id"), "{text}");
    assert!(text.contains("unknown species id 'unknown'"), "{text}");
}

#[test]
fn uncertainty_intervals_must_be_ordered_and_contain_the_value() {
    let mut document = document();
    document.model_parameters[0].quantity.uncertainty = Uncertainty::Interval {
        lower: 1.0,
        upper: 2.0,
    };

    let text = document
        .validate()
        .expect_err("value outside interval")
        .to_string();
    assert!(
        text.contains("interval does not contain the value"),
        "{text}"
    );
}
