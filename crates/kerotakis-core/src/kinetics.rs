//! Rates: how fast, not just how far.
//!
//! Every solver above this one answers the same question — *where does this
//! settle?* — and is structurally blind to the one a school lab actually
//! spends its time on. Equilibrium says magnesium should have burned on the
//! shelf. Free-energy minimisation says the blue precipitate should be black.
//! Both are right about the destination and silent about the journey, which
//! is why the thermal solver stands down below 500 K and why copper needs a
//! curated metastability threshold: those are two places where the engine
//! has already had to admit that rate exists.
//!
//! This module is the admission made properly. It is deliberately the
//! *cheap* version, and the boundary is worth stating plainly:
//!
//! - **We integrate curated rate laws.** rate = k·Π[Xᵢ]^nᵢ with k from
//!   Arrhenius. The orders are experimental facts, taken from the
//!   literature with provenance, not read off the stoichiometry — reaction
//!   order is a statement about mechanism and is frequently not the
//!   coefficient, which is itself one of the things worth teaching.
//! - **We do not derive mechanisms.** Elementary steps, rate-determining
//!   steps and steady-state approximations are P5's job and need a real
//!   mechanism engine. Nothing here discovers a rate law; it applies one.
//!
//! That buys the whole school treatment of kinetics — the thiosulfate
//! disappearing cross, catalysis, the temperature rule, order by initial
//! rates — for a few hundred lines, years before P5 delivers mechanism
//! chemistry. Same trade as the L6 colour decision: most of the pedagogy
//! for a small fraction of the cost, with the gap stated rather than hidden.
//!
//! **Time becomes a state dimension here**, which is a real change to the
//! bench rather than a data addition: vessels carry a clock, and `wait`
//! advances every vessel at once because time is not a per-beaker quantity.
//! That last point is what makes a fair test possible — two beakers, one
//! variable, the same thirty seconds.

use serde::{Deserialize, Serialize};

use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Gas constant, J·mol⁻¹·K⁻¹.
pub const R: f64 = 8.314_462_618;

/// A rate law: the orders, and the Arrhenius parameters behind `k`.
#[derive(Debug, Clone, Copy)]
pub struct RateLaw {
    /// Pre-exponential factor A, in units that make k·Π[X]^n come out as
    /// mol·L⁻¹·s⁻¹ for the stated orders.
    pub pre_exponential: f64,
    /// Activation energy, J/mol.
    pub activation_energy: f64,
}

impl RateLaw {
    /// k(T) = A·exp(−Ea/RT).
    pub fn rate_constant(&self, temperature_k: f64) -> f64 {
        self.pre_exponential * (-self.activation_energy / (R * temperature_k.max(1.0))).exp()
    }
}

/// One reaction whose *rate* we model.
#[derive(Debug, Clone, Copy)]
pub struct KineticReaction {
    pub id: &'static str,
    pub equation: &'static str,
    /// Species consumed, with stoichiometric coefficients.
    pub reactants: &'static [(&'static str, f64)],
    /// Species produced, with coefficients and the phase they appear in.
    pub products: &'static [(&'static str, f64, Phase)],
    /// The concentration dependence: (species, order). Orders are
    /// experimental, and deliberately not inferred from `reactants`.
    pub orders: &'static [(&'static str, f64)],
    pub rate: RateLaw,
    /// A catalyst does not appear in the stoichiometry and is not consumed.
    /// It lowers the activation energy — which is the whole content of what
    /// a catalyst *is*, so it is modelled that way rather than as a fudge
    /// factor on the rate.
    ///
    /// **How much catalyst there is makes no difference here, deliberately
    /// and wrongly.** Presence is a boolean: a milligram of manganese
    /// dioxide and a spoonful give bit-identical rates. For a heterogeneous
    /// catalyst the real rate goes with available surface, so it depends on
    /// both the amount and how finely it is ground — and this engine has no
    /// particle-size or surface-area model to hang that on. Scaling the
    /// rate by mass would be a fabricated number wearing the shape of a
    /// real one. The gap is stated instead, and `codex/rates.toml` teaches
    /// it as a limit rather than hiding it.
    pub catalysts: &'static [Catalyst],
    pub provenance: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Catalyst {
    pub species: &'static str,
    /// The activation energy this catalyst provides, J/mol. Lower than the
    /// uncatalysed value; the ratio of the two is the speed-up.
    pub activation_energy: f64,
    pub provenance: &'static str,
}

/// The reactions whose rates we model.
///
/// Small on purpose. Each is a school experiment that exists to *measure* a
/// rate, and each carries the literature its numbers came from. Adding one
/// is a curation act with a citation, not a tuning exercise.
pub const REGISTRY: &[KineticReaction] = &[
    KineticReaction {
        id: "thiosulfate-acid",
        equation: "Na₂S₂O₃ → S↓ + Na₂SO₃",
        reactants: &[("Na2S2O3", 1.0)],
        // Atoms must balance, and they did not: producing S and SO2 from
        // Na2S2O3 destroyed Na2O on every extent — 62 g/mol, straight off
        // the balance. The full chemistry is S2O3(2-) + 2H+ → S + SO2 + H2O,
        // and it cannot be written here because the proton is not a vessel
        // portion: it lives in PHREEQC's charge balance, so it can be read
        // (see PROTON) but not withdrawn. What is modelled is therefore the
        // sulfur-releasing half — which is the observable the practical
        // times — with the sulfite left in solution. The second step,
        // sulfite plus acid giving the SO2 you can smell, is stated as not
        // modelled rather than faked by inventing hydrogen.
        products: &[("S", 1.0, Phase::Solid), ("Na2SO3", 1.0, Phase::Aqueous)],
        // First order in each: the classic result of the initial-rates
        // experiment this reaction exists to teach. The acid term is read
        // from the solution's computed pH rather than from an inventory
        // amount, because the proton is not a vessel portion — it lives in
        // PHREEQC's charge balance. It is also not consumed here, which is
        // a stated approximation: the practical runs with acid in large
        // excess, so [H+] barely moves while the thiosulfate is used up.
        orders: &[("Na2S2O3", 1.0), (PROTON, 1.0)],
        rate: RateLaw {
            // Calibrated, and the calibration is the honest part: A is set
            // so that 0.05 M thiosulfate at pH 1.7 and 25 °C takes about
            // forty seconds to deposit enough sulfur to hide the cross,
            // which is the range the practical is designed around. Ea and
            // the orders are the literature's; A is ours.
            pre_exponential: 2.2e8,
            activation_energy: 51_000.0,
        },
        catalysts: &[],
        provenance: "Orders (1,1) and Ea ≈ 51 kJ/mol are the standard results of the disappearing-cross experiment (school practical literature; Ea commonly reported 45–60 kJ/mol). Editorial judgement (Kerotakis): the pre-exponential is fixed by matching the observable rather than measured, and the acid is treated as a rate influence read from the solution's pH rather than as a consumed reactant — the practical runs with acid in large excess, and the vessel has no proton portion to draw down",
    },
    KineticReaction {
        id: "peroxide-decomposition",
        equation: "2 H₂O₂ → 2 H₂O + O₂↑",
        reactants: &[("H2O2", 2.0)],
        // The water is not optional. Leaving it out of the products
        // destroyed 36 g/mol of matter per extent while the equation string
        // right above claimed otherwise.
        products: &[("water", 2.0, Phase::Liquid), ("O2", 1.0, Phase::Gas)],
        orders: &[("H2O2", 1.0)],
        rate: RateLaw {
            // Calibrated so undisturbed peroxide has a half-life near a
            // day, which is the point of the practical: without a catalyst
            // *nothing happens while you watch*. An earlier value gave
            // 299 s, so the bottle emptied itself in an afternoon and the
            // catalyst looked like a convenience rather than the whole
            // reason the reaction is usable. Real bottled peroxide is
            // stabilised and keeps far longer; a day is chosen to read as
            // clearly-nothing on a lesson timescale without pretending to
            // model stabilisers.
            pre_exponential: 5.6e7,
            activation_energy: 75_000.0,
        },
        catalysts: &[
            Catalyst {
                species: "MnO2",
                activation_energy: 58_000.0,
                provenance: "Manganese dioxide on hydrogen peroxide, Ea ≈ 58 kJ/mol (standard physical chemistry texts)",
            },
            Catalyst {
                species: "catalase",
                activation_energy: 23_000.0,
                provenance: "Catalase, Ea ≈ 23 kJ/mol — the enzyme is dramatically better than the mineral, which is the point of putting them side by side",
            },
        ],
        provenance: "Uncatalysed decomposition Ea ≈ 75 kJ/mol (standard physical chemistry texts); catalysed barriers cited per catalyst. Editorial judgement (Kerotakis): the pre-exponential is chosen so the uncatalysed half-life is about a day and the catalysed reaction is watchable, not measured. Absolute rates are therefore indicative — the amount and surface area of a solid catalyst are not modelled at all, so it is the comparison between catalysts that carries meaning, not the seconds",
    },
];

/// The order key that means "the proton activity of this solution".
///
/// Acid dependence cannot be read from the inventory: adding HCl to water
/// leaves chloride in the vessel and puts the proton into the solution's
/// charge balance, where PHREEQC computes its activity. So a rate law that
/// depends on acid reads 10^-pH — the *activity* the aqueous engine
/// actually solved for, not a nominal concentration.
pub const PROTON: &str = "H+";

/// Concentration to use for one order term, mol/L.
fn term_concentration(vessel: &Vessel, key: &str, litres: f64) -> Option<f64> {
    if key == PROTON {
        // No characterised solution means no known acidity: the rate is
        // unknown rather than zero, and the caller treats it as "cannot
        // proceed" rather than "does not react".
        return vessel.solution.as_ref().map(|s| 10f64.powf(-s.ph));
    }
    Some(vessel.moles_of(&SpeciesId::new(key)).0 / litres)
}

pub fn lookup(id: &str) -> Option<&'static KineticReaction> {
    REGISTRY.iter().find(|r| r.id == id)
}

/// Which reactions have all their reactants present in this vessel.
pub fn applicable(vessel: &Vessel) -> Vec<&'static KineticReaction> {
    REGISTRY
        .iter()
        .filter(|r| {
            r.reactants
                .iter()
                .all(|(key, _)| vessel.moles_of(&SpeciesId::new(key)).0 > 0.0)
        })
        .collect()
}

impl KineticReaction {
    /// The activation energy in force, given what is in the vessel: the
    /// best catalyst present, or the uncatalysed value.
    pub fn effective_activation_energy(&self, vessel: &Vessel) -> (f64, Option<&'static Catalyst>) {
        let mut best: Option<&'static Catalyst> = None;
        for c in self.catalysts {
            if vessel.moles_of(&SpeciesId::new(c.species)).0 > 0.0
                && best.is_none_or(|b| c.activation_energy < b.activation_energy)
            {
                best = Some(c);
            }
        }
        match best {
            Some(c) => (c.activation_energy, Some(c)),
            None => (self.rate.activation_energy, None),
        }
    }

    /// Rate in mol·L⁻¹·s⁻¹ for the vessel's current state.
    pub fn rate_now(&self, vessel: &Vessel) -> f64 {
        let litres = vessel.liquid_volume().0;
        if litres <= 0.0 {
            return 0.0;
        }
        // Nothing left to react is not a slow reaction, it is a finished
        // one — and saying so is what lets the clock move on.
        for (key, _) in self.reactants {
            if vessel.moles_of(&SpeciesId::new(key)).0 <= DEPLETED {
                return 0.0;
            }
        }
        let (ea, _) = self.effective_activation_energy(vessel);
        let law = RateLaw {
            pre_exponential: self.rate.pre_exponential,
            activation_energy: ea,
        };
        let k = law.rate_constant(vessel.temperature.0);
        let mut rate = k;
        for (key, order) in self.orders {
            let Some(c) = term_concentration(vessel, key, litres) else {
                return 0.0;
            };
            if c <= 0.0 {
                return 0.0;
            }
            rate *= c.powf(*order);
        }
        rate
    }
}

/// The largest fractional change any one substep may make to a reactant.
/// Chosen for accuracy against the closed-form solution — see the oracle
/// test — rather than for speed.
const STEP_FRACTION: f64 = 0.01;

/// Below this amount a reactant counts as gone.
///
/// Not cosmetic: the step limiter sizes `dt` as a fraction of what remains,
/// and for a fast reaction the rate falls in proportion, so `dt` stops
/// shrinking and the clock stops advancing. A catalysed decomposition that
/// finishes in microseconds would then grind out millions of substeps
/// trying to reach the two-second mark. A reaction with nothing left to
/// consume is over.
///
/// It must also sit *above* the threshold at which `Vessel::withdraw`
/// discards a spent portion (1e-15 mol), and that is not a detail. The
/// midpoint evaluation works on a copy of the vessel; once a half-step
/// pushed the copy's last reactant under that threshold the portion
/// disappeared there, the midpoint rate came back as zero, and the full
/// step then applied nothing at all — the integrator froze, burning two
/// million substeps to advance the clock by seven milliseconds. A
/// picomole is far below anything observable and clears the floor.
const DEPLETED: f64 = 1e-12;

/// How far each reaction has run, in mol/L of "extent".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub steps: usize,
}

/// Advance this vessel's chemistry by `seconds`.
///
/// Returns the extent of each reaction, in moles. The integration is
/// explicit with adaptive substeps: a rate law is stiff exactly when it is
/// interesting (a catalysed reaction can consume its reactant in under a
/// second), so the step is chosen from the fastest relative change rather
/// than fixed, and no substep is allowed to consume more than a small
/// fraction of any reactant. That keeps a fast reaction from integrating
/// past zero and producing negative concentrations, which is the classic
/// way a naive Euler step lies.
pub fn advance(vessel: &mut Vessel, seconds: f64) -> Vec<(&'static KineticReaction, Moles)> {
    let reactions = applicable(vessel);
    if reactions.is_empty() || seconds <= 0.0 {
        return Vec::new();
    }
    let mut extents: Vec<f64> = vec![0.0; reactions.len()];
    let mut t = 0.0;
    // Scratch vessel for the midpoint evaluation, reused rather than
    // allocated per substep.
    let mut probe = vessel.clone();

    // A hard cap on substeps: an integration that will not finish is a
    // solver failure, not something to grind at forever.
    const MAX_STEPS: usize = 2_000_000;
    let mut steps = 0;

    while t < seconds && steps < MAX_STEPS {
        steps += 1;
        let litres = vessel.liquid_volume().0;
        if litres <= 0.0 {
            break;
        }
        let rates: Vec<f64> = reactions.iter().map(|r| r.rate_now(vessel)).collect();
        if rates.iter().all(|r| *r <= 1e-18) {
            break;
        }
        let mut dt = seconds - t;
        for (r, rate) in reactions.iter().zip(&rates) {
            if *rate <= 0.0 {
                continue;
            }
            for (key, coeff) in r.reactants {
                let available = vessel.moles_of(&SpeciesId::new(key)).0 / litres;
                // No substep may change any reactant by more than this
                // fraction — see STEP_FRACTION.
                let limit = STEP_FRACTION * available / (rate * coeff);
                if limit > 0.0 && limit < dt {
                    dt = limit;
                }
            }
        }
        // NaN-safe: a rate that came out non-finite must stop the loop
        // rather than propagate through the composition.
        if dt.is_nan() || dt < 1e-12 {
            break;
        }

        // Midpoint (RK2): step half way on the current rates, re-evaluate
        // there, and take the whole step on *those*. Plain Euler always
        // errs the same direction on a decaying reaction and the error
        // accumulates — it drifted 0.7% from the closed-form answer over
        // two minutes, which the analytic oracle caught. Evaluating in the
        // middle cancels the first-order term.
        probe.clone_from(vessel);
        for (r, rate) in reactions.iter().zip(&rates) {
            let half = rate * (dt * 0.5) * litres;
            if half <= 0.0 {
                continue;
            }
            for (key, coeff) in r.reactants {
                probe.withdraw(&SpeciesId::new(key), Moles(half * coeff));
            }
            for (key, coeff, phase) in r.products {
                probe.deposit(SpeciesId::new(key), Moles(half * coeff), *phase);
            }
        }
        let mid: Vec<f64> = reactions.iter().map(|r| r.rate_now(&probe)).collect();

        for (i, (r, rate)) in reactions.iter().zip(&mid).enumerate() {
            let extent = rate * dt * litres; // moles
            if extent <= 0.0 {
                continue;
            }
            for (key, coeff) in r.reactants {
                vessel.withdraw(&SpeciesId::new(key), Moles(extent * coeff));
            }
            for (key, coeff, phase) in r.products {
                vessel.deposit(SpeciesId::new(key), Moles(extent * coeff), *phase);
            }
            extents[i] += extent;
        }
        t += dt;
    }

    reactions
        .into_iter()
        .zip(extents)
        .filter(|(_, e)| *e > 0.0)
        .map(|(r, e)| (r, Moles(e)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn vessel_with(items: &[(&str, f64, Phase)], celsius: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        for (key, moles, phase) in items {
            v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
        }
        v.temperature = Kelvin(273.15 + celsius);
        v
    }

    fn thiosulfate(celsius: f64, thio: f64) -> Vessel {
        let mut v = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("Na2S2O3", thio, Phase::Aqueous),
            ],
            celsius,
        );
        // Acid enters through the solution's pH, as it does on the bench.
        v.solution = Some(crate::vessel::SolutionInfo {
            redox: Vec::new(),
            pe: None,
            ph: 1.7,
            ionic_strength: 0.02,
            species: Vec::new(),
            provenance: None,
        });
        v
    }

    #[test]
    fn arrhenius_is_the_arrhenius_equation() {
        let law = RateLaw {
            pre_exponential: 1.0e10,
            activation_energy: 50_000.0,
        };
        let k = law.rate_constant(298.15);
        let expected = 1.0e10 * (-50_000.0f64 / (R * 298.15)).exp();
        assert!((k - expected).abs() / expected < 1e-12);
    }

    #[test]
    fn the_ten_degree_rule_falls_out_rather_than_being_applied() {
        // "Rate roughly doubles every ten degrees" is a consequence of an
        // activation energy near 50 kJ/mol at room temperature, not a law.
        // The engine should reproduce it without being told.
        let law = RateLaw {
            pre_exponential: 1.0,
            activation_energy: 50_000.0,
        };
        let ratio = law.rate_constant(308.15) / law.rate_constant(298.15);
        assert!(
            (1.9..2.3).contains(&ratio),
            "rate ratio over ten degrees: {ratio:.2}"
        );
        // And a much larger barrier breaks the rule of thumb, which is the
        // point of calling it a rule of thumb.
        let steep = RateLaw {
            pre_exponential: 1.0,
            activation_energy: 150_000.0,
        };
        let steep_ratio = steep.rate_constant(308.15) / steep.rate_constant(298.15);
        assert!(steep_ratio > 5.0, "{steep_ratio:.2}");
    }

    #[test]
    fn concentration_sets_the_rate_through_the_order() {
        let a = thiosulfate(25.0, 0.005);
        let b = thiosulfate(25.0, 0.010);
        let r = lookup("thiosulfate-acid").unwrap();
        let ratio = r.rate_now(&b) / r.rate_now(&a);
        assert!(
            (1.95..2.05).contains(&ratio),
            "first order: doubling concentration doubles rate, got {ratio:.3}"
        );
    }

    #[test]
    fn a_warmer_beaker_clouds_sooner() {
        // Compared before either runs out of thiosulfate — at long times
        // both go to completion and the difference in *rate* is invisible,
        // which is exactly the mistake the practical is designed to avoid
        // by timing an early, fixed amount of cloudiness.
        let mut cold = thiosulfate(20.0, 0.005);
        let mut warm = thiosulfate(40.0, 0.005);
        advance(&mut cold, 20.0);
        advance(&mut warm, 20.0);
        let sulfur = |v: &Vessel| v.moles_of(&SpeciesId::new("S")).0;
        let ratio = sulfur(&warm) / sulfur(&cold);
        assert!(
            ratio > 2.5,
            "20 °C gave {:.3e} mol of sulfur, 40 °C gave {:.3e} (×{ratio:.2})",
            sulfur(&cold),
            sulfur(&warm)
        );
        // Neither may have finished, or the comparison means nothing.
        assert!(
            sulfur(&warm) < 0.004,
            "the warm beaker must not have run out: {:.4e}",
            sulfur(&warm)
        );
    }

    #[test]
    fn the_cross_disappears_in_about_the_time_the_practical_expects() {
        // The observable this rate law is calibrated against, checked so a
        // later edit cannot quietly move it. Enough sulfur to obscure the
        // cross is taken as 0.01 mol/L.
        let mut v = thiosulfate(25.0, 0.005);
        let mut seconds = 0.0;
        while v.moles_of(&SpeciesId::new("S")).0 / 0.1 < 0.01 && seconds < 300.0 {
            advance(&mut v, 1.0);
            seconds += 1.0;
        }
        assert!(
            (20.0..70.0).contains(&seconds),
            "cross obscured after {seconds} s, which is outside the practical's range"
        );
    }

    #[test]
    fn integration_never_produces_a_negative_amount() {
        // The way a naive Euler step lies: a fast reaction integrated with
        // too large a step consumes more than exists.
        let mut v = thiosulfate(80.0, 0.005);
        advance(&mut v, 3600.0);
        for p in &v.contents {
            assert!(p.moles.0 >= 0.0, "{:?}", p);
        }
        assert!(v.moles_of(&SpeciesId::new("Na2S2O3")).0 >= 0.0);
    }

    #[test]
    fn a_catalyst_lowers_the_barrier_rather_than_scaling_the_rate() {
        let plain = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
            ],
            25.0,
        );
        let with_mno2 = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
                ("MnO2", 0.001, Phase::Solid),
            ],
            25.0,
        );
        let r = lookup("peroxide-decomposition").unwrap();
        let (ea_plain, none) = r.effective_activation_energy(&plain);
        let (ea_cat, some) = r.effective_activation_energy(&with_mno2);
        assert!(none.is_none());
        assert!(some.is_some());
        assert!(ea_cat < ea_plain);
        assert!(
            r.rate_now(&with_mno2) > r.rate_now(&plain) * 100.0,
            "the speed-up is a consequence of the barrier, not a factor"
        );
    }

    #[test]
    fn the_catalyst_is_not_consumed() {
        let mut v = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
                ("MnO2", 0.001, Phase::Solid),
            ],
            25.0,
        );
        advance(&mut v, 60.0);
        assert!(
            (v.moles_of(&SpeciesId::new("MnO2")).0 - 0.001).abs() < 1e-12,
            "a catalyst comes out as it went in"
        );
        assert!(v.moles_of(&SpeciesId::new("H2O2")).0 < 0.1);
    }

    #[test]
    fn the_integrator_matches_the_analytic_solution() {
        // The oracle that needs no dependency and never goes stale: a
        // first-order decay has a closed form, so the adaptive integrator
        // can be checked against exact arithmetic rather than against
        // another implementation of the same guess.
        //
        // d[H2O2]/dt = -2·k·[H2O2]  (order 1, stoichiometric coefficient 2)
        //   =>  [H2O2](t) = [H2O2]0 · exp(-2kt)
        let r = lookup("peroxide-decomposition").unwrap();
        let c0 = 0.5;
        let litres = 0.1;
        for seconds in [1.0, 10.0, 120.0, 600.0] {
            let mut v = vessel_with(
                &[
                    ("water", 5.5343, Phase::Liquid),
                    ("H2O2", c0 * litres, Phase::Aqueous),
                ],
                25.0,
            );
            let k = r.rate.rate_constant(298.15);
            advance(&mut v, seconds);
            let got = v.moles_of(&SpeciesId::new("H2O2")).0 / litres;
            let exact = c0 * (-2.0 * k * seconds).exp();
            let error = (got - exact).abs() / exact;
            assert!(
                error < 1e-3,
                "after {seconds} s: integrator {got:.6e}, exact {exact:.6e} ({:.3}% off)",
                error * 100.0
            );
        }
    }

    #[test]
    fn the_integrator_matches_the_analytic_solution_when_catalysed() {
        // The stiff case, which is where an integrator actually earns its
        // keep: catalase drops the barrier so far that the reaction is
        // essentially over in seconds, and a fixed step would overshoot
        // into negative concentrations.
        let r = lookup("peroxide-decomposition").unwrap();
        let litres = 0.1;
        let c0 = 0.5;
        let mut v = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", c0 * litres, Phase::Aqueous),
                ("catalase", 1e-6, Phase::Aqueous),
            ],
            25.0,
        );
        let law = RateLaw {
            pre_exponential: r.rate.pre_exponential,
            activation_energy: 23_000.0,
        };
        let k = law.rate_constant(298.15);
        let seconds = 2.0;
        advance(&mut v, seconds);
        let got = v.moles_of(&SpeciesId::new("H2O2")).0 / litres;
        let exact = c0 * (-2.0 * k * seconds).exp();
        assert!(
            (got - exact).abs() / exact.max(1e-30) < 5e-3 || (got.abs() < 1e-9 && exact < 1e-9),
            "catalysed: integrator {got:.6e}, exact {exact:.6e}"
        );
    }

    /// Element totals of a vessel, so a reaction can be audited the way a
    /// balance audits a real one.
    fn elements(v: &Vessel) -> std::collections::BTreeMap<String, f64> {
        let mut totals: std::collections::BTreeMap<String, f64> = Default::default();
        for p in &v.contents {
            let Some(data) = species::lookup(&p.species) else {
                panic!("{} is not in the registry", p.species.0)
            };
            let f = crate::stoich::parse_formula(data.formula)
                .unwrap_or_else(|e| panic!("{}: {e}", data.formula));
            for (el, n) in f.counts {
                *totals.entry(el).or_insert(0.0) += n * p.moles.0;
            }
        }
        totals
    }

    fn assert_conserved(before: &Vessel, after: &Vessel, what: &str) {
        let (a, b) = (elements(before), elements(after));
        let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
        keys.sort();
        keys.dedup();
        for k in keys {
            let x = a.get(k).copied().unwrap_or(0.0);
            let y = b.get(k).copied().unwrap_or(0.0);
            let drift = (y - x).abs() / x.max(1e-12);
            assert!(
                drift < 1e-9,
                "{what}: {k} went in at {x:.6} mol and came out at {y:.6} mol"
            );
        }
    }

    #[test]
    fn every_rate_law_conserves_its_atoms() {
        // The check that was missing. Both curated rate laws destroyed
        // matter — peroxide lost the water it should have produced, and
        // thiosulfate lost Na2O — and no test could see it, because the
        // conservation proptest never issues a `wait` and tracks only
        // water, ethanol and salt. A rate law is a reaction and has to
        // balance like one.
        for r in REGISTRY {
            let mut v = Vessel::new(VesselId(0), "beaker");
            v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
            for (key, _) in r.reactants {
                v.deposit(SpeciesId::new(key), Moles(0.02), Phase::Aqueous);
            }
            v.solution = Some(crate::vessel::SolutionInfo {
                redox: Vec::new(),
                pe: None,
                ph: 1.7,
                ionic_strength: 0.02,
                species: Vec::new(),
                provenance: None,
            });
            let before = v.clone();
            let moved = advance(&mut v, 600.0);
            assert!(!moved.is_empty(), "{} did not run at all", r.id);
            assert_conserved(&before, &v, r.id);
        }
    }

    #[test]
    fn the_declared_equation_balances_too() {
        // A rate law carries an equation string for the learner. If the
        // string and the modelled stoichiometry disagree, one of them is
        // lying, and it is usually the code.
        for r in REGISTRY {
            let eq = crate::stoich::parse_equation(r.equation)
                .unwrap_or_else(|e| panic!("{}: {e}", r.id));
            assert!(
                eq.is_balanced(),
                "{}: declared equation does not balance: {:?}",
                r.id,
                eq.element_imbalance()
            );
        }
    }
}
