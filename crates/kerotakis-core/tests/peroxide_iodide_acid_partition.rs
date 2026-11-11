//! Where the iodide-peroxide pathway split is measured, and where it is not.
//!
//! Liebhafsky & Mohammad, J. Am. Chem. Soc. 55 (1933) 3977,
//! doi:10.1021/ja01337a010, measured that in perchloric acid with (H+)
//! 0.00762-0.841 M (Table I, Curve II starts at 0.762 x 10^-2 M) every peroxide consumed by H2O2 + I- ends as iodine.
//! The bench gives that step to the clock rows while a represented acid
//! inventory lets them run (CATALYST_PATHWAY_OWNERS). These tests pin the
//! two honesty edges of that ownership: an owner running below the
//! measured acidity is reported as an extrapolation, and an acidic
//! solution whose acidity the owners cannot consume (no represented acid)
//! reports that its catalysed oxygen is not the measured outcome.

use kerotakis_core::kinetics;
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{SolutionInfo, Vessel, VesselId};

fn solution(ph: f64) -> SolutionInfo {
    SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        particle_molality: None,
        redox: Vec::new(),
        pe: None,
        ph,
        ionic_strength: 0.04,
        species: Vec::new(),
        provenance: None,
    }
}

fn iodide_peroxide(represented_acid: f64, ph: Option<f64>) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(
        SpeciesId::new("water"),
        Moles(5.534276991396059),
        Phase::Liquid,
    );
    if represented_acid > 0.0 {
        v.deposit(
            SpeciesId::new("H+"),
            Moles(represented_acid),
            Phase::Aqueous,
        );
        v.deposit(
            SpeciesId::new("SO4-2"),
            Moles(represented_acid / 2.0),
            Phase::Aqueous,
        );
    }
    v.deposit(SpeciesId::new("I-"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("K+"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    v.temperature = Kelvin(298.15);
    v.solution = ph.map(solution);
    v
}

#[test]
fn acidity_below_the_measured_floor_is_reported() {
    // pH 2.5, (H+) 3.16e-3 M < 7.62e-3 M, the least acidic point of
    // Liebhafsky & Mohammad's Table I, Curve II (p. 3980).
    let v = iodide_peroxide(0.002, Some(2.5));
    for owner in ["iodide-peroxide-clock", "iodide-peroxide-clock-direct"] {
        let (h, floor) = kinetics::pathway_owner_below_measured_acid(owner, &v)
            .unwrap_or_else(|| panic!("{owner} must report the extrapolation"));
        assert!((h - 10f64.powf(-2.5)).abs() < 1e-12);
        assert_eq!(floor, 0.00762);
    }
}

#[test]
fn ki011_acidity_is_inside_the_measured_range() {
    // batch2:ki-011 after the KI step: pH 1.865, (H+) 0.0136 M. That is
    // above the measured floor 0.00762 M (the 0.0203 M floor read the
    // series' second row as its first), so no extrapolation is claimed.
    let v = iodide_peroxide(0.002, Some(1.865065588166));
    for owner in ["iodide-peroxide-clock", "iodide-peroxide-clock-direct"] {
        assert!(kinetics::pathway_owner_below_measured_acid(owner, &v).is_none());
    }
}

#[test]
fn measured_acidity_and_other_rows_are_not_reported() {
    let v = iodide_peroxide(0.02, Some(1.0));
    assert!(kinetics::pathway_owner_below_measured_acid("iodide-peroxide-clock", &v).is_none());
    assert!(
        kinetics::pathway_owner_below_measured_acid("iodide-peroxide-clock-direct", &v).is_none()
    );
    let low = iodide_peroxide(0.002, Some(1.865065588166));
    assert!(kinetics::pathway_owner_below_measured_acid("peroxide-decomposition", &low).is_none());
    assert!(kinetics::pathway_owner_below_measured_acid("thiosulfate-acid", &low).is_none());
    // No characterised solution: no acidity known, nothing claimed.
    let bare = iodide_peroxide(0.002, None);
    assert!(kinetics::pathway_owner_below_measured_acid("iodide-peroxide-clock", &bare).is_none());
}

#[test]
fn acidic_solution_without_represented_acid_flags_the_catalysed_oxygen() {
    // pH 1.0 but no H+ portion the clock rows could consume.
    let v = iodide_peroxide(0.0, Some(1.0));
    assert!(!kinetics::catalyst_pathway_owned_elsewhere(
        "peroxide-decomposition",
        "I-",
        &v
    ));
    assert!(kinetics::catalyst_branch_acid_unowned(
        "peroxide-decomposition",
        "I-",
        &v
    ));
    assert!(kinetics::catalyst_branch_acid_unowned(
        "peroxide-decomposition",
        "KI",
        &v
    ));
}

#[test]
fn owned_neutral_or_unlisted_catalyst_branches_are_not_flagged() {
    // Represented acid present: the clock owns the step, nothing to flag.
    let owned = iodide_peroxide(0.02, Some(1.0));
    assert!(kinetics::catalyst_pathway_owned_elsewhere(
        "peroxide-decomposition",
        "I-",
        &owned
    ));
    assert!(!kinetics::catalyst_branch_acid_unowned(
        "peroxide-decomposition",
        "I-",
        &owned
    ));
    // Neutral: the catalysed decomposition is the measured outcome.
    let neutral = iodide_peroxide(0.0, Some(7.0));
    assert!(!kinetics::catalyst_branch_acid_unowned(
        "peroxide-decomposition",
        "I-",
        &neutral
    ));
    // Weakly acidic, outside the measured range: not claimed either way.
    let weak = iodide_peroxide(0.0, Some(3.0));
    assert!(!kinetics::catalyst_branch_acid_unowned(
        "peroxide-decomposition",
        "I-",
        &weak
    ));
    // Catalysts without an owner entry are never flagged.
    let acid = iodide_peroxide(0.0, Some(1.0));
    for catalyst in ["MnO2", "catalase"] {
        assert!(!kinetics::catalyst_branch_acid_unowned(
            "peroxide-decomposition",
            catalyst,
            &acid
        ));
    }
}
