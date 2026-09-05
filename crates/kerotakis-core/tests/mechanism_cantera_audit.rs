//! BRD-040 — executable half of the Cantera YAML feature/rejection matrix.
//!
//! Each case below is a real Cantera construct that the portable subset does
//! not model. Before this audit most of them were *dropped* by serde rather
//! than refused, so a valid mechanism file could be compiled into a network
//! that answered a different question than the file asked. The written matrix
//! lives in `provenance/brd-040-cantera-audit.md`; this file is the part that
//! CI can fail.
//!
//! Two conventions hold throughout:
//!
//! - a construct we cannot model must produce a **typed** error naming the
//!   offending field, never a silent drop and never a bare serde type error;
//! - a construct that genuinely cannot change any answer (provenance keys,
//!   transport blocks, `duplicate`) must still **parse**, so tightening the
//!   schema does not lock out ordinary ck2yaml output.

use kerotakis_core::kinetics::mechanism::{parse_yaml, MechanismError};

/// A minimal, balanced, irreversible gas mechanism used as the mutation base.
const BASE: &str = r"
description: BRD-040 audit fixture
units: {length: cm, quantity: mol, activation-energy: cal/mol}
phases:
- name: gas
  thermo: ideal-gas
  species: [H2, O2, H2O, N2]
species:
- name: H2
  composition: {H: 2}
- name: O2
  composition: {O: 2}
- name: H2O
  composition: {H: 2, O: 1}
- name: N2
  composition: {N: 2}
reactions:
- equation: 2 H2 + O2 => 2 H2O
  rate-constant: {A: 1.0e12, b: 0.5, Ea: 10000.0}
";

/// A second reaction in falloff form, appended to [`BASE`] where a falloff
/// sub-block is under test. Only species [`BASE`] declares are used.
const FALLOFF_REACTION: &str = "- equation: 2 H2O (+M) => 2 H2 + O2 (+M)
  type: falloff
  high-P-rate-constant: {A: 1.0e12, b: 0.0, Ea: 0.0}
  low-P-rate-constant: {A: 1.0e15, b: 0.0, Ea: 0.0}
";

/// Append extra keys to the single reaction entry of [`BASE`].
fn with_reaction_fields(extra: &str) -> String {
    format!("{BASE}{extra}")
}

/// Replace the phase block of [`BASE`].
fn with_phase(phase: &str) -> String {
    BASE.replace(
        "- name: gas\n  thermo: ideal-gas\n  species: [H2, O2, H2O, N2]\n",
        phase,
    )
}

fn field_error(yaml: &str) -> String {
    match parse_yaml(yaml) {
        Ok(parsed) => panic!(
            "expected a typed rejection, but the document parsed into {} reactions",
            parsed.summary().reactions
        ),
        Err(error) => error.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Rate-law modifiers that used to be dropped silently
// ---------------------------------------------------------------------------

/// Cantera's `orders` replaces the stoichiometric exponents *and* changes the
/// units of `A`. Ignoring it produced a mass-action rate law the file never
/// asked for.
#[test]
fn explicit_reaction_orders_are_refused_by_name() {
    let error = field_error(&with_reaction_fields("  orders: {H2: 0.25, O2: 1.5}\n"));
    assert!(error.contains("reaction 1"), "{error}");
    assert!(error.contains("orders"), "{error}");
}

#[test]
fn reaction_order_relaxation_flags_are_refused_by_name() {
    for flag in ["negative-orders", "nonreactant-orders"] {
        let error = field_error(&with_reaction_fields(&format!("  {flag}: true\n")));
        assert!(error.contains(flag), "{error}");
    }
}

/// A negative pre-exponential is a fitting artifact that only makes sense
/// summed with its duplicate partner. The rate guard already refused `A <= 0`;
/// the flag itself must be refused too so the intent is not mistaken for noise.
#[test]
fn negative_pre_exponential_flag_is_refused_by_name() {
    let error = field_error(&with_reaction_fields("  negative-A: true\n"));
    assert!(error.contains("negative-A"), "{error}");
}

/// Cantera dispatches falloff functions in the order Troe, SRI, Tsang,
/// Lindemann. With no `Troe` block present, an `SRI` or `Tsang` block was
/// previously dropped and the reaction silently degraded to Lindemann.
#[test]
fn non_troe_falloff_blocks_are_refused_instead_of_degrading_to_lindemann() {
    for (name, block) in [
        ("SRI", "  SRI: {A: 1.1, B: 700.0, C: 1234.0}\n"),
        ("Tsang", "  Tsang: {A: 0.95, B: -1.0e-4}\n"),
    ] {
        let yaml = format!("{BASE}{FALLOFF_REACTION}{block}");
        let error = field_error(&yaml);
        assert!(error.contains(name), "{name}: {error}");
    }
}

/// A `units` mapping is legal inside an individual reaction entry and overrides
/// the file-level directive. Honouring only the top-level mapping silently
/// applied the wrong scale to `A` and `Ea`.
#[test]
fn per_reaction_units_overrides_are_refused_by_name() {
    let error = field_error(&with_reaction_fields(
        "  units: {activation-energy: kJ/mol}\n",
    ));
    assert!(error.contains("units"), "{error}");
}

/// Anything Cantera adds in future is refused by default rather than dropped.
#[test]
fn unknown_reaction_keys_are_refused_rather_than_dropped() {
    let error = field_error(&with_reaction_fields("  coverage-dependencies: {H2: 1}\n"));
    assert!(error.contains("coverage-dependencies"), "{error}");
}

/// `duplicate` is an assertion, not a rate modifier: Cantera keeps duplicate
/// reactions separate and sums their rates, which compiling each entry
/// independently already does. Annotation keys must not become a new wall.
#[test]
fn benign_reaction_annotations_still_parse() {
    let yaml = with_reaction_fields("  duplicate: true\n  note: from the audit fixture\n");
    let parsed = parse_yaml(&yaml).expect("annotation-only keys must not block a mechanism");
    assert_eq!(parsed.summary().reactions, 1);
}

// ---------------------------------------------------------------------------
// Reaction-order arithmetic (a wrong answer, not a missing feature)
// ---------------------------------------------------------------------------

/// `H + 2 O2 => HO2 + O2` is second order in O2 even though O2's *net*
/// coefficient is -1. Deriving orders from the net stoichiometry produced a
/// second-order rate law with a third-order pre-exponential, mis-scaling `A`
/// by one concentration unit. Six of the twenty-nine reactions in Cantera's
/// own `h2o2.yaml` are written this way.
#[test]
fn spectator_species_keep_their_reactant_side_order() {
    let yaml = BASE.replace(
        "- equation: 2 H2 + O2 => 2 H2O\n  rate-constant: {A: 1.0e12, b: 0.5, Ea: 10000.0}\n",
        "- equation: 2 H2 + O2 + N2 => 2 H2O + N2\n  rate-constant: {A: 1.0e12, b: 0.0, Ea: 0.0}\n",
    );
    let summary = parse_yaml(&yaml)
        .expect("spectator form must parse")
        .summary();
    let reaction = &summary.reaction_details[0];
    // 2 H2 + 1 O2 + 1 N2, not the net 2 H2 + 1 O2.
    assert_eq!(reaction.total_order, 4.0, "{reaction:?}");
    // mol/cm^3 -> mol/L is a factor of 1000, and a fourth-order constant
    // carries C^-3: 1e12 * 1000^-3. Reading the order off the net vector would
    // have given the third-order scaling 1e12 * 1000^-2 = 1e6 instead.
    assert!(
        (reaction.pre_exponential - 1.0e3).abs() < 1.0e-6,
        "{reaction:?}"
    );
}

// ---------------------------------------------------------------------------
// Unit resolution
// ---------------------------------------------------------------------------

/// Cantera derives the default activation-energy unit from `energy` and
/// `quantity`. A fixed J/kmol default misread `units: {quantity: mol}` — a very
/// common ck2yaml output shape — by a factor of one thousand.
#[test]
fn activation_energy_units_follow_energy_and_quantity_defaults() {
    let cases = [
        ("units: {quantity: mol}", 1_000.0),
        ("units: {quantity: kmol}", 1.0),
        ("units: {energy: cal, quantity: mol}", 4_184.0),
        ("units: {energy: cal}", 4.184),
        (
            "units: {energy: cal, quantity: mol, activation-energy: J/kmol}",
            1.0,
        ),
    ];
    for (directive, expected_j_per_mol) in cases {
        let yaml = BASE
            .replace(
                "units: {length: cm, quantity: mol, activation-energy: cal/mol}",
                directive,
            )
            .replace("Ea: 10000.0", "Ea: 1000.0");
        let summary = parse_yaml(&yaml)
            .unwrap_or_else(|error| panic!("{directive}: {error}"))
            .summary();
        let actual = summary.reaction_details[0].activation_energy_j_per_mol;
        assert!(
            (actual - expected_j_per_mol).abs() < 1.0e-9 * expected_j_per_mol.max(1.0),
            "{directive}: expected {expected_j_per_mol} J/mol, got {actual}"
        );
    }
}

/// Cantera refuses any temperature scale with a non-unity conversion from
/// kelvin; dropping the key would have left the file looking accepted.
#[test]
fn non_kelvin_temperature_units_are_refused() {
    let yaml = BASE.replace(
        "units: {length: cm, quantity: mol, activation-energy: cal/mol}",
        "units: {length: cm, quantity: mol, activation-energy: cal/mol, temperature: C}",
    );
    let error = field_error(&yaml);
    assert!(error.contains("temperature"), "{error}");
}

#[test]
fn unknown_units_keys_are_refused() {
    let yaml = BASE.replace("quantity: mol", "quantity: mol, frobnication: 3");
    let error = field_error(&yaml);
    assert!(error.contains("frobnication"), "{error}");
}

/// Cantera allows `A: 1.0e12 cm^3/mol/s`. Per-value rate units are not
/// modelled; the failure must name the field rather than surface serde's
/// "invalid type: string".
#[test]
fn unit_bearing_pre_exponentials_are_refused_by_name() {
    let yaml = BASE.replace("A: 1.0e12", "A: 1.0e12 cm^3/mol/s");
    let error = field_error(&yaml);
    assert!(error.contains("pre-exponential"), "{error}");
    assert!(!error.contains("invalid type"), "{error}");
}

// ---------------------------------------------------------------------------
// Phase selectors
// ---------------------------------------------------------------------------

/// A phase can exclude reactions entirely, or take only those whose species it
/// declares. Ignoring the selector compiled every reaction in the file whatever
/// the phase said.
#[test]
fn phase_reaction_selectors_other_than_all_are_refused() {
    for selector in [
        "none",
        "declared-species",
        "[gri30.yaml/reactions]",
        "[{gri30.yaml/reactions: declared-species}]",
    ] {
        let yaml = with_phase(&format!(
            "- name: gas\n  thermo: ideal-gas\n  species: [H2, O2, H2O, N2]\n  reactions: {selector}\n"
        ));
        let error = field_error(&yaml);
        assert!(error.contains("reactions"), "{selector}: {error}");
    }
}

#[test]
fn phase_reactions_all_and_gas_kinetics_still_parse() {
    let yaml = with_phase(
        "- name: gas\n  thermo: ideal-gas\n  species: [H2, O2, H2O, N2]\n  kinetics: gas\n  reactions: all\n  transport: mixture-averaged\n  state: {T: 300.0, P: 1 atm}\n",
    );
    assert_eq!(
        parse_yaml(&yaml)
            .expect("ordinary phase")
            .summary()
            .reactions,
        1
    );
}

/// `kinetics: none` means Cantera builds no reaction set for the phase at all.
#[test]
fn non_gas_kinetics_managers_are_refused() {
    for kinetics in ["none", "surface", "edge"] {
        let yaml = with_phase(&format!(
            "- name: gas\n  thermo: ideal-gas\n  species: [H2, O2, H2O, N2]\n  kinetics: {kinetics}\n"
        ));
        let error = field_error(&yaml);
        assert!(error.contains("kinetics"), "{kinetics}: {error}");
    }
}

/// Cross-file species references are the shape multi-file mechanisms use. They
/// must be refused by name, not through a serde type error.
#[test]
fn cross_file_species_selectors_are_refused_by_name() {
    let yaml =
        with_phase("- name: gas\n  thermo: ideal-gas\n  species: [{gri30.yaml/species: all}]\n");
    let error = field_error(&yaml);
    assert!(error.contains("species"), "{error}");
    assert!(!error.contains("invalid type"), "{error}");
}

/// Cantera's default when the key is absent is every declared species.
#[test]
fn omitted_and_explicit_all_species_selectors_match_cantera_defaults() {
    for selector in ["", "  species: all\n"] {
        let yaml = with_phase(&format!("- name: gas\n  thermo: ideal-gas\n{selector}"));
        let summary = parse_yaml(&yaml)
            .unwrap_or_else(|error| panic!("selector {selector:?}: {error}"))
            .summary();
        assert_eq!(summary.species, 4, "selector {selector:?}");
    }
}

#[test]
fn a_species_claimed_by_two_phases_is_named_as_such() {
    let yaml = with_phase(
        "- name: gas\n  thermo: ideal-gas\n  species: [H2, O2, H2O, N2]\n- name: gas2\n  thermo: ideal-gas\n  species: [H2]\n",
    );
    assert!(
        matches!(
            parse_yaml(&yaml),
            Err(MechanismError::DuplicatePhaseAssignment { .. })
        ),
        "{:?}",
        parse_yaml(&yaml).err()
    );
}

// ---------------------------------------------------------------------------
// Species and thermo
// ---------------------------------------------------------------------------

/// Transport blocks are never read by the kinetics path, and real-gas
/// `equation-of-state` parameters — which Cantera's own `h2o2.yaml` and
/// `nDodecane_Reitz.yaml` carry on every species — are ignored by Cantera too
/// for an `ideal-gas` phase, the only phase model this subset accepts. Both may
/// therefore be dropped. Anything else on a species may not.
#[test]
fn species_transport_and_real_gas_parameters_are_ignored_but_other_keys_are_not() {
    let ignored = BASE.replace(
        "- name: N2\n  composition: {N: 2}\n",
        "- name: N2\n  composition: {N: 2}\n  note: air diluent\n  transport: {model: gas, geometry: linear, well-depth: 97.53, diameter: 3.621}\n  equation-of-state: {model: Redlich-Kwong, a: 1.43319e+11, b: 18.42802577}\n",
    );
    assert_eq!(
        parse_yaml(&ignored)
            .expect("transport and real-gas parameters must not block a mechanism")
            .summary()
            .species,
        4
    );

    let refused = BASE.replace(
        "- name: N2\n  composition: {N: 2}\n",
        "- name: N2\n  composition: {N: 2}\n  coverage-dependencies: {H2: 1}\n",
    );
    let error = field_error(&refused);
    assert!(error.contains("coverage-dependencies"), "{error}");
}

/// Every non-NASA7 thermo model must name itself in the rejection.
#[test]
fn unsupported_thermo_models_name_the_model_and_the_species() {
    for model in ["NASA9", "Shomate", "constant-cp", "piecewise-Gibbs"] {
        let yaml = BASE.replace(
            "- name: N2\n  composition: {N: 2}\n",
            &format!(
                "- name: N2\n  composition: {{N: 2}}\n  thermo:\n    model: {model}\n    temperature-ranges: [200.0, 1000.0, 6000.0]\n    data:\n    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n"
            ),
        );
        let error = field_error(&yaml);
        assert!(error.contains(model), "{model}: {error}");
        assert!(error.contains("N2"), "{model}: {error}");
    }
}

#[test]
fn unknown_thermo_keys_are_refused() {
    let yaml = BASE.replace(
        "- name: N2\n  composition: {N: 2}\n",
        "- name: N2\n  composition: {N: 2}\n  thermo:\n    model: NASA7\n    temperature-ranges: [200.0, 1000.0, 6000.0]\n    dimensionless: true\n    data:\n    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n    - [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]\n",
    );
    let error = field_error(&yaml);
    assert!(error.contains("dimensionless"), "{error}");
}

/// Charged species use the pseudo-element `E` with an inverted sign, so a
/// cation carries `E: -1`. Ionised chemistry is out of the subset; the refusal
/// must still say which species and which element it choked on.
#[test]
fn ionised_species_are_refused_with_the_element_named() {
    let yaml = BASE.replace(
        "- name: N2\n  composition: {N: 2}\n",
        "- name: N2\n  composition: {N: 2}\n- name: H3O+\n  composition: {H: 3, O: 1, E: -1}\n",
    );
    let error = field_error(&yaml);
    assert!(error.contains("H3O+"), "{error}");
    assert!(error.contains('E'), "{error}");
}

// ---------------------------------------------------------------------------
// Document structure
// ---------------------------------------------------------------------------

/// ck2yaml stamps every file it writes with provenance keys. They carry no
/// chemistry, so they must not be a wall.
#[test]
fn ck2yaml_provenance_keys_still_parse() {
    let yaml = format!(
        "generator: ck2yaml\ninput-files: [h2o2.inp, gri30_tran.dat]\ncantera-version: 2.5.0\ndate: Wed, 11 Dec 2019 16:59:04 -0500\n{BASE}"
    );
    assert_eq!(
        parse_yaml(&yaml)
            .expect("provenance keys")
            .summary()
            .reactions,
        1
    );
}

/// A named section cannot be attributed to a phase without modelling phase
/// selectors, so its contents might or might not be in play. Refuse rather
/// than guess.
#[test]
fn extra_named_document_sections_are_refused() {
    let yaml = format!("{BASE}\nozone-reactions:\n- equation: 2 H2 + O2 => 2 H2O\n  rate-constant: {{A: 1.0, b: 0, Ea: 0}}\n");
    assert!(
        matches!(
            parse_yaml(&yaml),
            Err(MechanismError::UnsupportedSection { .. })
        ),
        "{:?}",
        parse_yaml(&yaml).err()
    );
}

// ---------------------------------------------------------------------------
// Reaction types
// ---------------------------------------------------------------------------

/// The rate-law families outside the subset, plus the undocumented `type`
/// aliases Cantera also accepts. All must be refused with the type named; none
/// may be mistaken for an elementary reaction.
#[test]
fn every_unmodelled_reaction_type_is_refused_with_its_name() {
    for kind in [
        "Chebyshev",
        "chemically-activated",
        "linear-Burke",
        "electron-collision-plasma",
        "electron-collisions",
        "two-temperature-plasma",
        "Blowers-Masel",
        "interface-Arrhenius",
        "interface-Blowers-Masel",
        "sticking-Arrhenius",
        "sticking-Blowers-Masel",
        "electrochemical",
        // Undocumented aliases Cantera resolves internally. Refusing them is a
        // deliberate divergence: the subset only reads the canonical spellings.
        "Arrhenius",
        "three-body-Arrhenius",
        "three-body-Blowers-Masel",
        "Troe",
        "Lindemann",
        "SRI",
        "Tsang",
    ] {
        let yaml = BASE.replace(
            "- equation: 2 H2 + O2 => 2 H2O\n",
            &format!("- equation: 2 H2 + O2 => 2 H2O\n  type: {kind}\n"),
        );
        let error = field_error(&yaml);
        assert!(
            matches!(
                parse_yaml(&yaml),
                Err(MechanismError::UnsupportedReactionType { .. })
            ),
            "{kind}: {error}"
        );
        assert!(error.contains(kind), "{kind}: {error}");
    }
}

/// Cantera treats `(+M)` with a plain `rate-constant` and no `type` as an
/// ordinary three-body reaction — falloff notation degraded without a warning.
/// The subset refuses the form instead of picking either reading.
#[test]
fn falloff_notation_without_a_type_is_refused_rather_than_downgraded() {
    let yaml = BASE.replace(
        "- equation: 2 H2 + O2 => 2 H2O\n",
        "- equation: 2 H2 + O2 (+M) => 2 H2O (+M)\n",
    );
    let error = field_error(&yaml);
    assert!(error.contains("third-body marker"), "{error}");
}

/// Reversible pressure-dependent reactions are the dominant form in every real
/// mechanism file, so the refusal must say so plainly rather than fail on an
/// unrelated check downstream.
#[test]
fn reversible_pressure_dependent_reactions_are_refused_plainly() {
    let yaml = BASE.replace(
        "- equation: 2 H2 + O2 => 2 H2O\n  rate-constant: {A: 1.0e12, b: 0.5, Ea: 10000.0}\n",
        "- equation: 2 H2 + O2 + M <=> 2 H2O + M\n  type: three-body\n  rate-constant: {A: 1.0e12, b: 0.5, Ea: 10000.0}\n",
    );
    let error = field_error(&yaml);
    assert!(
        error.contains("reversible pressure-dependent reactions"),
        "{error}"
    );
}

/// Negative activation energies appear four times in Cantera's own
/// `h2o2.yaml` and thirty-two times in `gri30.yaml`. BRD-040 refused them
/// and recommended (§7, item 3) that the guard become a finiteness check;
/// BRD-041 needs exactly that, because `CO + OH -> CO2 + H` — the reaction
/// that decides whether carbon monoxide burns at all — is barrierless and
/// its recommended fit has a negative `Ea`. So they are accepted now, and
/// the sign survives into the compiled rate law rather than being clamped.
#[test]
fn negative_activation_energies_are_accepted_as_fitted_parameters() {
    let yaml = BASE.replace("Ea: 10000.0", "Ea: -1700.0");
    let mechanism = parse_yaml(&yaml).expect("a negative Ea is a legal fitted parameter");
    let detail = &mechanism.summary().reaction_details[0];
    assert!(
        (detail.activation_energy_j_per_mol - (-1700.0 * 4.184)).abs() < 1e-9,
        "the sign and the cal/mol scale both survive: {}",
        detail.activation_energy_j_per_mol
    );
}

/// A NaN activation energy is still refused, and by name. It is not a
/// fitted parameter; it is a rate law that poisons every number computed
/// from it, silently and forever.
#[test]
fn a_non_finite_activation_energy_is_refused_with_the_value_named() {
    let yaml = BASE.replace("Ea: 10000.0", "Ea: .nan");
    let error = field_error(&yaml);
    assert!(error.contains("Ea"), "{error}");
    assert!(error.contains("must be finite"), "{error}");
}

/// Real mechanisms ship Troe `A` outside [0, 1] — `n-heptane-NUIG-2016.yaml`
/// contains values from -73.91 to 2.545. The subset's narrower guard must say
/// which parameter it rejected.
#[test]
fn out_of_range_troe_parameters_are_refused_with_the_parameter_named() {
    let yaml = format!(
        "{BASE}{FALLOFF_REACTION}  Troe: {{A: -73.91, T3: 3.705e4, T1: 4.15e4, T2: 5220.0}}\n"
    );
    let error = field_error(&yaml);
    assert!(error.contains("Troe A"), "{error}");
}
