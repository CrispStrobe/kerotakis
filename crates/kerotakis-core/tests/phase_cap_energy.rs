//! Prospective phase-cap conservation controls. Not executed; synthetic activity
//! inputs are not native chemistry, measured pH, or calibration evidence.
use kerotakis_core::*;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

const MW: f64 = 0.018_015;
const R: f64 = 8.314_462_618_153_24;
const FUSION: f64 = 6010.0;
const REFERENCE_K: f64 = 273.15;
const PARTICLES: f64 = 0.02;
const WATER_TOTAL: f64 = 4.0;
const ENERGY_TOL_J: f64 = 0.001;
const AMOUNT_TOL_MOL: f64 = 1e-10;
const OFFSET_J: f64 = 0.1;

#[derive(Clone, Copy, Debug)]
enum Route {
    Ideal,
    Eutectic,
}
#[derive(Clone, Copy, Debug)]
enum Position {
    Inside,
    Exact,
    Outside,
}
impl Route {
    fn aw(self, particles_per_kg: f64) -> f64 {
        match self {
            Self::Ideal => 1.0 / (1.0 + MW * particles_per_kg),
            Self::Eutectic => (-MW * particles_per_kg).exp(),
        }
    }
    fn liquidus(self, liquid_moles: f64) -> f64 {
        let aw = self.aw(PARTICLES / (MW * liquid_moles));
        1.0 / (1.0 / REFERENCE_K - R / FUSION * aw.ln())
    }
    fn cap_molality(self) -> f64 {
        match self {
            Self::Ideal => 6.0,
            Self::Eutectic => (1.0 / 252.0 - 1.0 / REFERENCE_K) * FUSION / (R * MW),
        }
    }
    fn cap_liquid(self) -> f64 {
        PARTICLES / (MW * self.cap_molality())
    }
    fn reason(self) -> &'static str {
        match self {
            Self::Ideal => "not-modeled.activity-range-ideal",
            Self::Eutectic => "not-modeled.partial-freezing-cap",
        }
    }
}

#[derive(Clone)]
struct Interval {
    lo: f64,
    hi: f64,
    a: [f64; 7],
}
struct Reference {
    liquid: Vec<Interval>,
    solid: Vec<Interval>,
}
impl Reference {
    fn new() -> Self {
        assert_eq!(
            states::WATER_H_FUS,
            FUSION,
            "Pinned thermodynamic input changed"
        );
        assert_eq!(constants::WATER_MOLAR_MASS_KG_PER_MOL, MW);
        let rows: Vec<Value> = serde_json::from_str(include_str!("golden/registry.json")).unwrap();
        for key in ["Na+", "Cl-"] {
            let row = rows.iter().find(|row| row["key"] == key).unwrap();
            assert_eq!(row["heat_capacity"].as_f64(), Some(0.0));
            assert!(row["heat_capacity_polys"].as_array().unwrap().is_empty());
        }
        let water = rows.iter().find(|row| row["key"] == "water").unwrap();
        let curve = |phase: &str| {
            let record = water["heat_capacity_polys"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["phase"] == phase)
                .unwrap();
            record["intervals"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| {
                    assert_eq!(p["form"], "nasa9");
                    let coefficients: Vec<f64> = p["coefficients"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_f64().unwrap())
                        .collect();
                    Interval {
                        lo: p["t_min"].as_f64().unwrap(),
                        hi: p["t_max"].as_f64().unwrap(),
                        a: coefficients.try_into().unwrap(),
                    }
                })
                .collect()
        };
        Self {
            liquid: curve("liquid"),
            solid: curve("solid"),
        }
    }
    fn cp(curve: &[Interval], t: f64) -> f64 {
        let t = t.clamp(curve.first().unwrap().lo, curve.last().unwrap().hi);
        let a = &curve.iter().find(|p| p.lo <= t && t <= p.hi).unwrap().a;
        R * (a[0] / t.powi(2)
            + a[1] / t
            + a[2]
            + a[3] * t
            + a[4] * t.powi(2)
            + a[5] * t.powi(3)
            + a[6] * t.powi(4))
    }
    // Composite Simpson quadrature is independent of the production analytic
    // enthalpy integrals, inverse, liquidus implementation and phase root.
    fn sensible(curve: &[Interval], t: f64, n: usize) -> f64 {
        assert_eq!(n % 2, 0);
        let dt = (t - REFERENCE_K) / n as f64;
        let mut total = 0.0;
        for i in 0..=n {
            let weight = if i == 0 || i == n {
                1.0
            } else if i % 2 == 0 {
                2.0
            } else {
                4.0
            };
            total += weight * Self::cp(curve, REFERENCE_K + dt * i as f64);
        }
        total * dt / 3.0
    }
    fn inventory(&self, t: f64, liquid: f64, solid: f64, n: usize) -> f64 {
        liquid * Self::sensible(&self.liquid, t, n)
            + solid * (Self::sensible(&self.solid, t, n) - FUSION)
    }
    fn energy(&self, vessel: &Vessel) -> f64 {
        assert_eq!(
            vessel.excess_enthalpy_j, 0.0,
            "No unrelated mixing-energy sink is permitted"
        );
        let a = self.inventory(
            vessel.temperature.0,
            water(vessel, Phase::Liquid),
            water(vessel, Phase::Solid),
            4096,
        );
        let b = self.inventory(
            vessel.temperature.0,
            water(vessel, Phase::Liquid),
            water(vessel, Phase::Solid),
            8192,
        );
        assert!(
            (a - b).abs() < 1e-7,
            "Reference quadrature not resolved: {a} / {b}"
        );
        b
    }
    fn cap_energy(&self, route: Route) -> f64 {
        let liquid = route.cap_liquid();
        let t = route.liquidus(liquid);
        let a = self.inventory(t, liquid, WATER_TOTAL - liquid, 4096);
        let b = self.inventory(t, liquid, WATER_TOTAL - liquid, 8192);
        assert!((a - b).abs() < 1e-7);
        b
    }
    fn entry_temperature(&self, route: Route, position: Position) -> f64 {
        let target = self.cap_energy(route)
            + match position {
                Position::Inside => OFFSET_J,
                Position::Exact => 0.0,
                Position::Outside => -OFFSET_J,
            };
        let (mut lo, mut hi) = (200.0, REFERENCE_K);
        assert!(
            self.inventory(lo, 1.0, 3.0, 8192) < target
                && self.inventory(hi, 1.0, 3.0, 8192) > target
        );
        // This inverse constructs the ENTERING energy budget, not the expected
        // post-transition amount or any beyond-cap equilibrium state.
        for _ in 0..64 {
            let mid = (lo + hi) / 2.0;
            if self.inventory(mid, 1.0, 3.0, 8192) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t = (lo + hi) / 2.0;
        assert!((self.inventory(t, 1.0, 3.0, 8192) - target).abs() < 1e-7);
        t
    }
}
fn water(v: &Vessel, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}
fn fixture(route: Route, t: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "declared cap-energy input");
    v.temperature = Kelvin(t);
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.deposit(SpeciesId::new("water"), Moles(3.0), Phase::Solid);
    for key in ["Na+", "Cl-"] {
        v.deposit(SpeciesId::new(key), Moles(PARTICLES / 2.0), Phase::Aqueous);
    }
    declare_activity(&mut v, route);
    v
}
fn declare_activity(v: &mut Vessel, route: Route) {
    let kg = water(v, Phase::Liquid) * MW;
    assert!(kg > 0.0);
    let particle_molality = PARTICLES / kg;
    // Explicit synthetic input only. No native solve, trust receipt, pH
    // inference, chemical yield, empirical activity or calibration is asserted.
    v.solution = Some(SolutionInfo {
        scope: Default::default(),
        solvent_kg: Some(kg),
        pe: None,
        redox: vec![],
        ph: 7.0,
        ionic_strength: particle_molality / 2.0,
        species: ["Na+", "Cl-"]
            .into_iter()
            .map(|name| SpeciesDetail {
                name: name.into(),
                molality: particle_molality / 2.0,
                activity: particle_molality / 2.0,
            })
            .collect(),
        provenance: None,
        solvent_activity: match route {
            Route::Ideal => None,
            Route::Eutectic => Some(kerotakis_core::vessel::SolventActivityProvenance {
                dataset: "synthetic cap-control input; no native dataset solve".into(),
                model: "declared phi=1, activity ceiling=40 mol/kg".into(),
                dataset_phrase: None,
                model_phrase: None,
                water_activity: (-MW * particle_molality).exp(),
                particle_molality,
                ionic_strength: particle_molality / 2.0,
            }),
        },
    });
}

fn material(v: &Vessel) {
    assert!(
        (water(v, Phase::Liquid) + water(v, Phase::Solid) - WATER_TOTAL).abs() < AMOUNT_TOL_MOL
    );
    assert!((v.moles_of(&SpeciesId::new("Na+")).0 - PARTICLES / 2.0).abs() < AMOUNT_TOL_MOL);
    assert!((v.moles_of(&SpeciesId::new("Cl-")).0 - PARTICLES / 2.0).abs() < AMOUNT_TOL_MOL);
    assert!(v.contents.iter().all(|p| p.moles.0.is_finite()
        && p.moles.0 >= 0.0
        && matches!(
            (p.species.0.as_str(), p.phase),
            ("water", Phase::Liquid | Phase::Solid) | ("Na+" | "Cl-", Phase::Aqueous)
        )));
    assert_eq!(v.excess_enthalpy_j, 0.0);
}
// A conserved exact boundary may report its route directly or preserve the
// existing "ice removed; brine retained" wrapper with a typed boundary slot.
// Never infer identity from translated prose or an arbitrary nested string.
fn exact_boundary_notice(phrase: &Phrase, route: Route) -> bool {
    if phrase.key == route.reason() {
        return true;
    }
    if phrase.key != "not-modeled.brine-retained-at-boundary" {
        return false;
    }
    let slots: Vec<&Slot> = phrase
        .slots
        .iter()
        .filter(|(name, _)| name == "boundary")
        .map(|(_, value)| value)
        .collect();
    matches!(slots.as_slice(), [Slot::Phrase { phrase }] if phrase.key == route.reason())
}
fn check_result(
    reference: &Reference,
    route: Route,
    position: Position,
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
) {
    material(before);
    material(after);
    let boundary_notes: Vec<&Phrase> = events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                vessel: VesselId(0),
                cause: ops::NotModelledCause::ModelBoundary,
                reason: Some(p),
                ..
            } => Some(p),
            _ => None,
        })
        .collect();
    let reasons: Vec<&str> = boundary_notes
        .iter()
        .map(|phrase| phrase.key.as_str())
        .collect();
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    if matches!(position, Position::Outside) {
        assert_eq!(
            serde_json::to_value(after).unwrap(),
            serde_json::to_value(before).unwrap(),
            "Refusal must precede ANY state mutation"
        );
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::StateChanged { .. })));
        assert!(reasons.contains(&route.reason()));
        assert!(reasons.iter().all(|key| *key == route.reason()));
        assert!((reference.energy(after) - reference.energy(before)).abs() < ENERGY_TOL_J);
        let mut repeated = after.clone();
        for _ in 0..3 {
            let again = StateEquilibrator.equilibrate(&mut repeated).unwrap();
            assert_eq!(
                serde_json::to_value(&repeated).unwrap(),
                serde_json::to_value(before).unwrap()
            );
            assert!(!again
                .iter()
                .any(|e| matches!(e, Event::StateChanged { .. })));
            assert!(again.iter().any(|e| matches!(e, Event::NotYetModeled { vessel: VesselId(0), cause: ops::NotModelledCause::ModelBoundary, reason: Some(p), .. } if p.key == route.reason())));
        }
    } else {
        assert!((reference.energy(after) - reference.energy(before)).abs() < ENERGY_TOL_J);
        assert!((after.temperature.0 - route.liquidus(water(after, Phase::Liquid))).abs() < 1e-8);
        let moved = 1.0 - water(after, Phase::Liquid);
        assert!(moved > AMOUNT_TOL_MOL && water(after, Phase::Liquid) > 0.0);
        for event in events {
            if let Event::StateChanged {
                vessel,
                species,
                from,
                to,
                moles,
                ..
            } = event
            {
                assert_eq!(*vessel, VesselId(0));
                assert_eq!(species.0, "water");
                assert_eq!((*from, *to), (Phase::Liquid, Phase::Solid));
                assert!(moles.is_some_and(|n| n.0.is_finite() && n.0 > 0.0));
            }
        }
        let reported: f64 = events
            .iter()
            .filter_map(|e| match e {
                Event::StateChanged {
                    species,
                    from: Phase::Liquid,
                    to: Phase::Solid,
                    moles: Some(n),
                    ..
                } if species.0 == "water" => Some(n.0),
                _ => None,
            })
            .sum();
        assert!((reported - moved).abs() < AMOUNT_TOL_MOL);
        match position {
            Position::Inside => {
                assert!(water(after, Phase::Liquid) > route.cap_liquid() + AMOUNT_TOL_MOL);
                assert!(reasons.is_empty(), "No false boundary of either route");
            }
            Position::Exact => {
                assert!((water(after, Phase::Liquid) - route.cap_liquid()).abs() < AMOUNT_TOL_MOL);
                assert!(!boundary_notes.is_empty());
                assert!(boundary_notes
                    .iter()
                    .all(|phrase| exact_boundary_notice(phrase, route)));
            }
            Position::Outside => unreachable!(),
        }
    }
}
fn direct(route: Route, position: Position) {
    let reference = Reference::new();
    let before = fixture(route, reference.entry_temperature(route, position));
    // Admission check is a precondition, never the independent energy oracle.
    let admitted = solve::vessel_transitions(&before).0;
    assert!((admitted.solute_molality - PARTICLES / MW).abs() < 1e-10);
    assert!((admitted.freezing_k - route.liquidus(1.0)).abs() < 1e-8);
    let mut after = before.clone();
    let events = StateEquilibrator.equilibrate(&mut after).unwrap();
    check_result(&reference, route, position, &before, &after, &events);
}
#[derive(Clone)]
struct RecordedStates {
    route: Route,
    captures: Rc<RefCell<Vec<(Vessel, Vessel)>>>,
}
impl Equilibrator for RecordedStates {
    fn name(&self) -> &'static str {
        "states"
    }
    fn chemistry_applies(&self, _: &Vessel) -> bool {
        false
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        // Bench deliberately invalidates SolutionInfo before dispatch. Supply
        // this DECLARED synthetic particle/activity input at that actual
        // boundary, then capture States entry. This is not an aqueous solve.
        declare_activity(vessel, self.route);
        let before = vessel.clone();
        let result = StateEquilibrator.equilibrate(vessel);
        self.captures.borrow_mut().push((before, vessel.clone()));
        result
    }
}
fn bench_cool(route: Route, position: Position) {
    let reference = Reference::new();
    let target = reference.entry_temperature(route, position);
    let initial = fixture(route, route.liquidus(1.0));
    let requested = reference.energy(&initial) - reference.energy(&fixture(route, target));
    assert!(requested.is_finite() && requested > 0.0);
    let captures = Rc::new(RefCell::new(Vec::new()));
    let mut stack = SolverStack::new(vec![Box::new(RecordedStates {
        route,
        captures: captures.clone(),
    })]);
    let mut bench = Bench::new();
    bench.vessels[0] = initial.clone();
    let events = bench
        .step_with(
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(requested),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    let delivered: Vec<f64> = events
        .iter()
        .filter_map(|e| match e {
            Event::EnergyTransferred {
                vessel: VesselId(0),
                heating: false,
                requested_j,
                delivered_j,
                ..
            } => {
                assert!((*requested_j - requested).abs() < 1e-7);
                Some(*delivered_j)
            }
            _ => None,
        })
        .collect();
    assert_eq!(delivered.len(), 1);
    assert!((delivered[0] - requested).abs() < ENERGY_TOL_J);
    let calls = captures.borrow();
    assert_eq!(
        calls.len(),
        1,
        "One actual States invocation is the captured phase-owner boundary"
    );
    let (entering, settled) = &calls[0];
    assert!((entering.temperature.0 - target).abs() < 1e-7);
    assert!(
        (reference.energy(&initial) - reference.energy(entering) - delivered[0]).abs()
            < ENERGY_TOL_J
    );
    check_result(&reference, route, position, entering, settled, &events);
    material(&bench.vessels[0]);
    assert_eq!(bench.vessels[0].contents, settled.contents);
    assert_eq!(bench.vessels[0].temperature, settled.temperature);
    assert!(
        (reference.energy(&initial) - reference.energy(&bench.vessels[0]) - delivered[0]).abs()
            < ENERGY_TOL_J
    );
    assert_eq!(bench.vessels[0].elapsed_seconds, 0.0);
}
#[test]
fn ideal_inside_direct_conserves_without_false_refusal() {
    direct(Route::Ideal, Position::Inside);
}
#[test]
fn ideal_exact_direct_commits_only_closed_boundary() {
    direct(Route::Ideal, Position::Exact);
}
#[test]
fn ideal_outside_direct_refuses_before_mutation_and_repeats() {
    direct(Route::Ideal, Position::Outside);
}
#[test]
fn eutectic_inside_direct_conserves_without_false_refusal() {
    direct(Route::Eutectic, Position::Inside);
}
#[test]
fn eutectic_exact_direct_commits_only_closed_boundary() {
    direct(Route::Eutectic, Position::Exact);
}
#[test]
fn eutectic_outside_direct_refuses_before_mutation_and_repeats() {
    direct(Route::Eutectic, Position::Outside);
}
#[test]
fn ideal_inside_real_cool_closes_delivered_energy() {
    bench_cool(Route::Ideal, Position::Inside);
}
#[test]
fn ideal_exact_real_cool_closes_delivered_energy() {
    bench_cool(Route::Ideal, Position::Exact);
}
#[test]
fn ideal_outside_real_cool_preserves_delivered_energy_and_refuses() {
    bench_cool(Route::Ideal, Position::Outside);
}
#[test]
fn eutectic_inside_real_cool_closes_delivered_energy() {
    bench_cool(Route::Eutectic, Position::Inside);
}
#[test]
fn eutectic_exact_real_cool_closes_delivered_energy() {
    bench_cool(Route::Eutectic, Position::Exact);
}
#[test]
fn eutectic_outside_real_cool_preserves_delivered_energy_and_refuses() {
    bench_cool(Route::Eutectic, Position::Outside);
}
