//! What the curated Cp(T) curves must satisfy before anything is wired to
//! them.
//!
//! This is the data step of "heat capacity stops being a constant". No
//! ledger reads these curves yet; what these tests establish is that the
//! curves are the same substances the constants described, that they are
//! evaluated correctly, and — most importantly — that they cannot return
//! nonsense at a temperature a crucible can actually reach.

use kerotakis_core::heat_capacity::{CpForm, CpPolynomial, R};
use kerotakis_core::species::{Phase, SpeciesData, REGISTRY};
use kerotakis_core::states::{self, ICE_HEAT_CAPACITY, STEAM_HEAT_CAPACITY};

fn curves() -> Vec<(&'static SpeciesData, &'static CpPolynomial)> {
    REGISTRY
        .iter()
        .flat_map(|s| s.heat_capacity_polys.iter().map(move |c| (s, c)))
        .collect()
}

#[test]
fn the_bench_has_curves_for_the_things_that_get_hot() {
    let with_curves: Vec<&str> = REGISTRY
        .iter()
        .filter(|s| !s.heat_capacity_polys.is_empty())
        .map(|s| s.key)
        .collect();
    for wanted in [
        "CaCO3", "CaO", "MgO", "Mg", "Fe", "Fe2O3", "Cu", "CuO", "Al", "SiO2", "NaCl", "KCl",
        "Na2CO3", "CO2", "water", "N2", "O2",
    ] {
        assert!(
            with_curves.contains(&wanted),
            "{wanted} is heated on this bench and has no Cp(T) curve; \
             the registry carries curves for {with_curves:?}"
        );
    }
    assert!(
        with_curves.len() >= 30,
        "only {} species carry a curve: {with_curves:?}",
        with_curves.len()
    );
}

#[test]
fn every_curve_agrees_with_the_constant_it_stands_beside() {
    // The strongest available check that a curve belongs to the species it
    // is filed under. The constants were curated independently, from CRC
    // and CODATA rather than from NASA CEA, so agreement at 298 K is two
    // sources meeting rather than one source repeating itself.
    let mut worst = (0.0f64, "");
    for (species, curve) in curves() {
        if species.key == "water" && curve.phase != Phase::Liquid {
            // Ice and steam have their own constants; checked below.
            continue;
        }
        let from_curve = curve.cp(298.15).expect("a curve has a range");
        let constant = species.heat_capacity;
        let drift = (from_curve - constant).abs() / constant;
        if drift > worst.0 {
            worst = (drift, species.key);
        }
        assert!(
            drift < 0.03,
            "{} ({:?}): the curve says {from_curve:.2} J/(mol.K) at 298.15 K and the \
             registry constant says {constant:.2}, which is {:.1} % apart. Either the \
             curve is for a different substance or the constant is wrong, and both \
             are worth stopping for",
            species.key,
            curve.phase,
            100.0 * drift
        );
    }
    assert!(worst.0 > 0.0, "no curve was checked at all");
}

#[test]
fn ices_and_steams_own_constants_still_stand() {
    // EXP #452 gave water three heat capacities because cooling ice at
    // liquid water's 75.3 put a freezing mixture 39 K off. The curves must
    // not undo that: they replace those constants, so they had better agree
    // with them.
    let water = REGISTRY.iter().find(|s| s.key == "water").expect("water");
    let ice = states::heat_capacity_at(water, Phase::Solid, 273.15);
    let steam = states::heat_capacity_at(water, Phase::Gas, 373.15);
    let liquid = states::heat_capacity_at(water, Phase::Liquid, 298.15);
    assert!(
        (ice - ICE_HEAT_CAPACITY).abs() / ICE_HEAT_CAPACITY < 0.02,
        "ice at its melting point: curve {ice:.2} against constant {ICE_HEAT_CAPACITY}"
    );
    assert!(
        (steam - STEAM_HEAT_CAPACITY).abs() / STEAM_HEAT_CAPACITY < 0.02,
        "steam at its boiling point: curve {steam:.2} against constant {STEAM_HEAT_CAPACITY}"
    );
    assert!(
        (liquid - water.heat_capacity).abs() / water.heat_capacity < 0.01,
        "liquid water: curve {liquid:.2} against constant {}",
        water.heat_capacity
    );
}

#[test]
fn a_hundred_millilitres_of_water_still_costs_thirty_one_kilojoules() {
    // The number every school textbook prints, and the one thing about this
    // change that must NOT move: 100 mL of water from 25 to 100 degrees.
    let water = REGISTRY.iter().find(|s| s.key == "water").expect("water");
    let moles = 100.0 * water.density / water.molar_mass;
    let joules = moles * states::enthalpy_between(water, Phase::Liquid, 298.15, 373.15);
    let kj = joules / 1000.0;
    assert!(
        (kj - 31.4).abs() / 31.4 < 0.01,
        "heating 100 mL of water from 25 to 100 C should still cost about \
         31.4 kJ, the curve makes it {kj:.2} kJ"
    );
}

#[test]
fn chalk_and_lime_get_dearer_to_heat_and_that_is_the_whole_point() {
    let chalk = REGISTRY.iter().find(|s| s.key == "CaCO3").expect("CaCO3");
    let lime = REGISTRY.iter().find(|s| s.key == "CaO").expect("CaO");
    let chalk_hot = states::heat_capacity_at(chalk, Phase::Solid, 1500.0);
    let lime_hot = states::heat_capacity_at(lime, Phase::Solid, 1500.0);
    assert!(
        chalk_hot > 120.0,
        "calcite at 1500 K should be well above its 82 J/(mol.K) bench value, \
         got {chalk_hot:.1}"
    );
    assert!(
        lime_hot > 50.0,
        "lime at 1500 K should be above its 42 J/(mol.K) bench value, got {lime_hot:.1}"
    );
    // And the integral is what the ledger will actually spend.
    let warming = 0.1 * states::enthalpy_between(lime, Phase::Solid, 298.15, 1773.15);
    let flat = 0.1 * lime.heat_capacity * (1773.15 - 298.15);
    assert!(
        warming > flat * 1.15,
        "0.1 mol of lime taken to 1500 C costs {warming:.0} J on the curve \
         against {flat:.0} J at the constant; if those were close there would \
         be nothing here to fix"
    );
}

#[test]
fn no_curve_returns_nonsense_anywhere_a_crucible_can_reach() {
    // The reason `CpPolynomial::cp` clamps the temperature rather than only
    // the interval. Quartz's alpha polynomial is fitted to 848 K and reads
    // 1007 J/(mol.K) at 1500; sodium carbonate's reads -2038; lead's -1259.
    // A negative heat capacity does not degrade an energy balance, it
    // inverts it, and the vessel cools when heated.
    for (species, curve) in curves() {
        let (lo, hi) = curve.range().expect("a curve has a range");
        let reference = curve.cp(0.5 * (lo + hi)).expect("mid-range value");
        for t in [
            1.0, 100.0, 200.0, 298.15, 500.0, 1000.0, 1500.0, 3000.0, 6000.0,
        ] {
            let cp = curve.cp(t).expect("a curve answers everywhere");
            assert!(
                cp.is_finite() && cp > 0.0,
                "{} ({:?}) reports {cp} J/(mol.K) at {t} K, outside its \
                 tabulated {lo}-{hi} K",
                species.key,
                curve.phase
            );
            assert!(
                cp < 20.0 * reference.max(1.0),
                "{} ({:?}) reports {cp:.1} J/(mol.K) at {t} K against {reference:.1} \
                 mid-range: that is an extrapolation, not a held endpoint",
                species.key,
                curve.phase
            );
        }
    }
}

#[test]
fn every_curve_tiles_its_range_in_order() {
    for (species, curve) in curves() {
        let mut previous: Option<f64> = None;
        assert!(
            !curve.intervals.is_empty(),
            "{}: a curve with no interval claims nothing",
            species.key
        );
        for interval in curve.intervals {
            assert!(
                interval.t_max > interval.t_min,
                "{}: interval {}-{} is not ascending",
                species.key,
                interval.t_min,
                interval.t_max
            );
            if let Some(end) = previous {
                assert!(
                    (end - interval.t_min).abs() < 1e-6,
                    "{}: a gap between {end} K and {} K would silently fall \
                     back to an endpoint the bench never announced",
                    species.key,
                    interval.t_min
                );
            }
            previous = Some(interval.t_max);
            assert!(
                !interval.reference.is_empty(),
                "{}: an interval with no literature line is a number with no book",
                species.key
            );
        }
    }
}

#[test]
fn the_analytic_integral_is_the_area_under_the_curve() {
    // If the antiderivative and the polynomial ever disagree, every heat
    // ledger built on the pair is wrong in a way no single value reveals.
    for (species, curve) in curves() {
        let (lo, hi) = curve.range().expect("a curve has a range");
        let (t0, t1) = (lo + 0.05 * (hi - lo), lo + 0.95 * (hi - lo));
        let n = 4_000;
        let h = (t1 - t0) / n as f64;
        let mut numeric = 0.0;
        for k in 0..n {
            let a = t0 + k as f64 * h;
            numeric += 0.5 * h * (curve.cp(a).unwrap() + curve.cp(a + h).unwrap());
        }
        let analytic = curve.enthalpy_between(t0, t1).unwrap();
        let tolerance = 1e-4 * numeric.abs().max(1.0) + 1.0;
        assert!(
            (analytic - numeric).abs() < tolerance,
            "{} ({:?}): analytic {analytic:.3} J/mol against numeric \
             {numeric:.3} J/mol over {t0:.1}-{t1:.1} K",
            species.key,
            curve.phase
        );
    }
}

#[test]
fn heating_and_asking_where_it_landed_round_trips() {
    for (species, curve) in curves() {
        let (lo, hi) = curve.range().expect("a curve has a range");
        let from = lo + 0.1 * (hi - lo);
        let to = lo + 0.9 * (hi - lo);
        let joules = curve.enthalpy_between(from, to).unwrap();
        let landed = curve.temperature_after(from, joules).unwrap();
        assert!(
            (landed - to).abs() < 1e-2,
            "{}: putting {joules:.1} J/mol into {from:.1} K should land at \
             {to:.1} K, landed at {landed:.3} K",
            species.key
        );
    }
}

#[test]
fn a_species_without_a_curve_behaves_exactly_as_it_did() {
    // The whole point of keeping the constant beside the curve: nothing
    // that has no curve changes at all.
    let sugar = REGISTRY
        .iter()
        .find(|s| s.key == "sucrose")
        .expect("sucrose");
    assert!(
        sugar.heat_capacity_polys.is_empty(),
        "sucrose has no published NASA-9 or Shomate curve, and inventing \
         one would be worse than saying so"
    );
    let at_400 = states::heat_capacity_at(sugar, Phase::Solid, 400.0);
    assert!(
        (at_400 - sugar.heat_capacity).abs() < 1e-12,
        "with no curve the constant is returned unchanged at every \
         temperature, got {at_400}"
    );
    let over_a_hundred = states::enthalpy_between(sugar, Phase::Solid, 298.15, 398.15);
    assert!(
        (over_a_hundred - sugar.heat_capacity * 100.0).abs() < 1e-9,
        "and the integral is still Cp times delta T, got {over_a_hundred}"
    );
}

#[test]
fn the_mean_over_an_interval_carries_the_same_energy() {
    let chalk = REGISTRY.iter().find(|s| s.key == "CaCO3").expect("CaCO3");
    let (t0, t1) = (400.0, 1400.0);
    let mean = states::mean_heat_capacity_between(chalk, Phase::Solid, t0, t1);
    let exact = states::enthalpy_between(chalk, Phase::Solid, t0, t1);
    assert!(
        (mean * (t1 - t0) - exact).abs() < 1e-6,
        "a mean heat capacity that does not carry the interval's own energy \
         is no use to anyone: {mean} against {exact}"
    );
    let cold = states::heat_capacity_at(chalk, Phase::Solid, t0);
    let hot = states::heat_capacity_at(chalk, Phase::Solid, t1);
    assert!(
        mean > cold && mean < hot,
        "and it lies between the endpoints: {cold} < {mean} < {hot}"
    );
}

#[test]
fn the_gas_constant_is_the_gas_constant() {
    // The NASA-9 form is Cp/R, so a wrong R is a silent 0.1 % everywhere.
    assert!((R - 8.314_462_618_153_24).abs() < 1e-12);
    let n2 = REGISTRY.iter().find(|s| s.key == "N2").expect("N2");
    let curve = n2
        .heat_capacity_polys
        .first()
        .expect("nitrogen has a curve");
    assert_eq!(curve.phase, Phase::Gas);
    assert_eq!(curve.intervals[0].form, CpForm::Nasa9);
    let cp = curve.cp(298.15).unwrap();
    assert!(
        (cp - 3.5 * R).abs() < 0.2,
        "a diatomic gas near room temperature is close to 7R/2 = {:.2}, \
         nitrogen's curve says {cp:.2}",
        3.5 * R
    );
}

#[test]
fn every_curve_carries_the_book_it_came_from() {
    for (species, curve) in curves() {
        assert!(
            curve.source.contains("NASA") || curve.source.contains("NIST"),
            "{}: a curve's citation must name where the coefficients were \
             published, got {:?}",
            species.key,
            curve.source
        );
        assert!(
            !curve.method.is_empty() && !curve.boundary.is_empty(),
            "{}: a curve must say how it got here and what it does not claim",
            species.key
        );
        assert!(
            curve.boundary.contains("held at the endpoint"),
            "{}: the boundary must say what happens outside the table, \
             because that is the question a crucible asks",
            species.key
        );
    }
}
