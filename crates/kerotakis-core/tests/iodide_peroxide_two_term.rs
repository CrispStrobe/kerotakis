//! The iodide-peroxide clock as the measured two-term law, with one owner.
//!
//! rate = (k0 + k1[H+])[H2O2][I-], k0 = 0.69 L mol-1 min-1 and
//! k1 = 10.5 L2 mol-2 min-1 at 25 C (Liebhafsky & Mohammad, J. Am. Chem.
//! Soc. 55 (1933) 3977, doi:10.1021/ja01337a010). In acid the iodide
//! catalysed decomposition of peroxide must not run beside it: the same
//! elementary step H2O2 + I- would otherwise be integrated twice (the
//! batch2:ki-011 production run sent about half the peroxide to oxygen at
//! pH 1.9).

use kerotakis_core::kinetics::{self, advance};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{SolutionInfo, Vessel, VesselId};

const K0_25C: f64 = 0.69 / 60.0; // L mol-1 s-1
const K1_25C: f64 = 10.5 / 60.0; // L2 mol-2 s-1

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

/// batch2:ki-011's beaker as the solve leaves it, before the thiosulfate:
/// 100 mL water, 0.001 mol H2SO4 (two represented acid equivalents with
/// their sulfate), 0.002 mol KI as K+ and I-, 0.002 mol H2O2, and the
/// production pH 1.865065588166 printed after the KI step.
fn ki011_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(
        SpeciesId::new("water"),
        Moles(5.534276991396059),
        Phase::Liquid,
    );
    v.deposit(SpeciesId::new("H+"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("SO4-2"), Moles(0.001), Phase::Aqueous);
    v.deposit(SpeciesId::new("I-"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("K+"), Moles(0.002), Phase::Aqueous);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    v.temperature = Kelvin(298.15);
    v.solution = Some(solution(1.865065588166));
    v
}

fn moles(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

fn seconds_until_iodine(target_mol: f64) -> f64 {
    let mut v = ki011_vessel();
    let mut t = 0.0;
    while moles(&v, "I2") < target_mol && t < 600.0 {
        advance(&mut v, 0.5).unwrap();
        t += 0.5;
    }
    t
}

#[test]
fn rate_constants_are_the_measured_25c_values() {
    let direct = kinetics::lookup("iodide-peroxide-clock-direct").expect("k0 row");
    let acid = kinetics::lookup("iodide-peroxide-clock").expect("k1 row");
    let k0 = direct.forward.arrhenius.rate_constant(298.15);
    let k1 = acid.forward.arrhenius.rate_constant(298.15);
    assert!((k0 / K0_25C - 1.0).abs() < 1e-3, "k0 = {k0}");
    assert!((k1 / K1_25C - 1.0).abs() < 1e-3, "k1 = {k1}");
    // Table III of the source, converted from per minute, at 0 and 40 C.
    for (t, k0_tab, k1_tab) in [(273.15, 0.094, 2.05), (313.15, 2.19, 22.9)] {
        let k0 = direct.forward.arrhenius.rate_constant(t) * 60.0;
        let k1 = acid.forward.arrhenius.rate_constant(t) * 60.0;
        assert!((k0 / k0_tab - 1.0).abs() < 0.1, "k0({t}) = {k0}");
        assert!((k1 / k1_tab - 1.0).abs() < 0.1, "k1({t}) = {k1}");
    }
}

#[test]
fn clock_rate_is_linear_in_proton_activity_with_the_measured_intercept() {
    for ph in [1.0, 1.5, 2.0, 3.0] {
        let mut v = ki011_vessel();
        v.solution = Some(solution(ph));
        let litres = v.liquid_volume().0;
        let product = (moles(&v, "H2O2") / litres) * (moles(&v, "I-") / litres);
        let rate: f64 = ["iodide-peroxide-clock", "iodide-peroxide-clock-direct"]
            .iter()
            .map(|id| kinetics::lookup(id).unwrap().rate_now(&v))
            .sum();
        let expected = K0_25C + K1_25C * 10f64.powf(-ph);
        assert!(
            (rate / product / expected - 1.0).abs() < 2e-3,
            "pH {ph}: {} vs {expected}",
            rate / product
        );
    }
}

/// The hand computation (independent forward-Euler of the same law with
/// iodide, peroxide and represented acid all depleting, proton activity
/// scaled with the acid inventory): gross iodine reaches the 0.0001 mol
/// thiosulfate sink's capacity (5e-5 mol I2) at 93.9 s and the 0.0002 mol
/// sink's (1e-4 mol I2) at 196.4 s. With thiosulfate present iodide is
/// regenerated, which shortens these to 91.5 s and 186.3 s; acid
/// decomposition of the thiosulfate (`thiosulfate-acid`) shortens them
/// further on the bench.
#[test]
fn ki011_gross_iodine_meets_each_sink_at_the_hand_computed_time() {
    let small = seconds_until_iodine(5e-5);
    let large = seconds_until_iodine(1e-4);
    assert!((89.0..=99.0).contains(&small), "small sink: {small} s");
    assert!((186.0..=207.0).contains(&large), "large sink: {large} s");
}

#[test]
fn acid_iodide_peroxide_has_one_owner_no_catalytic_oxygen() {
    let v = ki011_vessel();
    let decomposition = kinetics::lookup("peroxide-decomposition").unwrap();
    let (_, catalyst) = decomposition.effective_activation_energy(&v);
    assert!(
        catalyst.is_none(),
        "iodide catalysis must stand down while the clock owns H2O2 + I-"
    );
    assert!(kinetics::catalyst_pathway_owned_elsewhere(
        "peroxide-decomposition",
        "I-",
        &v
    ));

    let mut run = ki011_vessel();
    advance(&mut run, 60.0).unwrap();
    let iodine = moles(&run, "I2");
    let oxygen = moles(&run, "O2");
    let peroxide_lost = 0.002 - moles(&run, "H2O2");
    assert!(iodine > 2e-5, "iodine {iodine}");
    // Only the uncatalysed (day-scale) decomposition may make oxygen: about
    // 1.5 % of the iodine here, against roughly twice the iodine when the
    // catalysed branch double-counted the step.
    assert!(oxygen < 0.05 * iodine, "O2 {oxygen} vs I2 {iodine}");
    assert!(
        (peroxide_lost - iodine).abs() < 0.05 * iodine,
        "peroxide lost {peroxide_lost} is (almost) all iodine {iodine}"
    );
}

#[test]
fn neutral_iodide_still_catalyses_decomposition_and_makes_no_iodine() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
    v.deposit(SpeciesId::new("I-"), Moles(0.005), Phase::Aqueous);
    v.deposit(SpeciesId::new("K+"), Moles(0.005), Phase::Aqueous);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.005), Phase::Liquid);
    v.temperature = Kelvin(298.15);
    let decomposition = kinetics::lookup("peroxide-decomposition").unwrap();
    let (_, catalyst) = decomposition.effective_activation_energy(&v);
    assert_eq!(catalyst.map(|c| c.species), Some("I-"));
    assert!(!kinetics::catalyst_pathway_owned_elsewhere(
        "peroxide-decomposition",
        "I-",
        &v
    ));
    advance(&mut v, 30.0).unwrap();
    assert_eq!(moles(&v, "I2"), 0.0);
    assert!(moles(&v, "O2") > 0.0);
}
