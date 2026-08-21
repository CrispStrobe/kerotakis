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
//! - **We integrate curated rate laws through a reaction-network IR.**
//!   rate = k·Π[Xᵢ]^nᵢ with k from Arrhenius. The orders are experimental
//!   facts, taken from the literature with provenance, not read off the
//!   stoichiometry — reaction order is a statement about mechanism and is
//!   frequently not the coefficient, which is itself one of the things
//!   worth teaching.
//! - **We do not derive or import mechanisms yet.** The IR can execute
//!   reversible, consecutive and competing reactions, but every admitted
//!   rate expression is still curated. Nothing here discovers a rate law;
//!   it applies one.
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

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

#[path = "kinetics_integrator.rs"]
mod integrator;
pub mod mechanism;

pub use integrator::{
    advance_network_with_options, IntegrationError, IntegrationOptions, IntegrationReport,
    IntegrationStatistics,
};

/// Gas constant, J·mol⁻¹·K⁻¹.
pub const R: f64 = 8.314_462_618;

/// Arrhenius parameters behind a mass-action rate expression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateLaw {
    /// Pre-exponential factor A, in units that make k·Π[X]^n come out as
    /// mol·L⁻¹·s⁻¹ for the stated orders.
    pub pre_exponential: f64,
    /// Temperature exponent b in the modified Arrhenius expression. The
    /// curated school-level laws use zero; mechanism formats commonly carry
    /// a measured non-zero exponent.
    pub temperature_exponent: f64,
    /// Activation energy, J/mol.
    pub activation_energy: f64,
}

/// One signed entry in a reaction's stoichiometric vector.
///
/// Negative coefficients consume material and positive coefficients produce
/// it. Keeping one vector instead of unrelated reactant and product lists is
/// what lets the conservation lint inspect exactly what the evaluator applies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoichiometricTerm<'a> {
    pub species: &'a str,
    pub coefficient: f64,
    pub phase: Phase,
}

/// One concentration/activity dependency in a mass-action expression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrderTerm<'a> {
    pub species: &'a str,
    /// `None` is reserved for derived activities such as H+; ordinary
    /// species should name the phase whose concentration drives the rate.
    pub phase: Option<Phase>,
    pub order: f64,
}

/// Dimensional exponents of a rate constant.
///
/// For a concentration law of total order `n`, `k` has dimensions
/// mol^(1-n) L^(n-1) s^-1. Exposing that fact prevents a mechanism importer
/// from treating every pre-exponential factor as though it had the same unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateDimensions {
    pub mole: f64,
    pub litre: f64,
    pub second: f64,
}

/// A rate expression in the network IR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RateExpression<'a> {
    pub arrhenius: RateLaw,
    pub orders: &'a [OrderTerm<'a>],
}

impl RateExpression<'_> {
    pub fn dimensions(&self) -> RateDimensions {
        let order: f64 = self.orders.iter().map(|term| term.order).sum();
        RateDimensions {
            mole: 1.0 - order,
            litre: order - 1.0,
            second: -1.0,
        }
    }
}

/// Where a kinetic transition is allowed to occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locality {
    Bulk(Phase),
    Interface { from: Phase, to: Phase },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    fn contains(self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

/// Conditions under which the parameters are claimed to apply.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Validity<'a> {
    pub temperature_k: Option<Range>,
    pub pressure_pa: Option<Range>,
    pub note: &'a str,
}

/// Quantified parameter confidence where one exists, with an honest note
/// where it does not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uncertainty<'a> {
    pub relative: Option<f64>,
    pub note: &'a str,
}

/// Site bookkeeping is separate from molecular stoichiometry. Negative
/// coefficients consume a site state and positive coefficients create one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SiteTerm<'a> {
    pub site: &'a str,
    pub coefficient: f64,
}

impl RateLaw {
    /// k(T) = A·T^b·exp(−Ea/RT).
    pub fn rate_constant(&self, temperature_k: f64) -> f64 {
        let temperature_k = temperature_k.max(1.0);
        self.pre_exponential
            * temperature_k.powf(self.temperature_exponent)
            * (-self.activation_energy / (R * temperature_k)).exp()
    }
}

/// One reaction in the runtime reaction-network IR.
#[derive(Debug, Clone, Copy)]
pub struct KineticReaction<'a> {
    pub id: &'a str,
    pub equation: &'a str,
    pub stoichiometry: &'a [StoichiometricTerm<'a>],
    pub locality: Locality,
    pub forward: RateExpression<'a>,
    /// A reverse expression makes the reaction reversible. Absence means the
    /// runtime is explicitly claiming only the forward direction.
    pub reverse: Option<RateExpression<'a>>,
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
    pub catalysts: &'a [Catalyst<'a>],
    pub sites: &'a [SiteTerm<'a>],
    /// Electrons produced (positive) or consumed (negative) per mole of
    /// extent. The conservation lint balances their charge separately.
    pub electrons: f64,
    pub validity: Validity<'a>,
    pub uncertainty: Uncertainty<'a>,
    pub source_ids: &'a [&'a str],
    pub provenance: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct Catalyst<'a> {
    pub species: &'a str,
    /// The activation energy this catalyst provides, J/mol. Lower than the
    /// uncatalysed value; the ratio of the two is the speed-up.
    pub activation_energy: f64,
    pub provenance: &'a str,
}

/// The reactions whose rates we model.
///
/// Small on purpose. Each is a school experiment that exists to *measure* a
/// rate, and each carries the literature its numbers came from. Adding one
/// is a curation act with a citation, not a tuning exercise.
pub const REGISTRY: &[KineticReaction<'static>] = &[
    KineticReaction {
        id: "thiosulfate-acid",
        equation: "Na₂S₂O₃ → S↓ + Na₂SO₃",
        stoichiometry: &[
            StoichiometricTerm {
                species: "Na2S2O3",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "S",
                coefficient: 1.0,
                phase: Phase::Solid,
            },
            StoichiometricTerm {
                species: "Na2SO3",
                coefficient: 1.0,
                phase: Phase::Aqueous,
            },
        ],
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
        locality: Locality::Bulk(Phase::Aqueous),
        // First order in each: the classic result of the initial-rates
        // experiment this reaction exists to teach. The acid term is read
        // from the solution's computed pH rather than from an inventory
        // amount, because the proton is not a vessel portion — it lives in
        // PHREEQC's charge balance. It is also not consumed here, which is
        // a stated approximation: the practical runs with acid in large
        // excess, so [H+] barely moves while the thiosulfate is used up.
        forward: RateExpression {
            orders: &[
                OrderTerm {
                    species: "Na2S2O3",
                    phase: Some(Phase::Aqueous),
                    order: 1.0,
                },
                OrderTerm {
                    species: PROTON,
                    phase: None,
                    order: 1.0,
                },
            ],
            arrhenius: RateLaw {
                // Calibrated, and the calibration is the honest part: A is set
                // so that 0.05 M thiosulfate at pH 1.7 and 25 °C takes about
                // forty seconds to deposit enough sulfur to hide the cross,
                // which is the range the practical is designed around. Ea and
                // the orders are the literature's; A is ours.
                pre_exponential: 2.2e8,
                temperature_exponent: 0.0,
                activation_energy: 51_000.0,
            },
        },
        reverse: None,
        catalysts: &[],
        sites: &[],
        electrons: 0.0,
        validity: Validity {
            temperature_k: None,
            pressure_pa: None,
            note: "calibrated near room temperature for aqueous school-practical conditions with acid in excess; Arrhenius extrapolation is not independently validated",
        },
        uncertainty: Uncertainty {
            relative: None,
            note: "absolute rate is calibrated to the disappearing-cross observation",
        },
        source_ids: &["kerotakis:kinetics:thiosulfate-acid"],
        provenance: "Orders (1,1) and Ea ≈ 51 kJ/mol are the standard results of the disappearing-cross experiment (school practical literature; Ea commonly reported 45–60 kJ/mol). Editorial judgement (Kerotakis): the pre-exponential is fixed by matching the observable rather than measured, and the acid is treated as a rate influence read from the solution's pH rather than as a consumed reactant — the practical runs with acid in large excess, and the vessel has no proton portion to draw down",
    },
    KineticReaction {
        id: "peroxide-decomposition",
        equation: "2 H₂O₂ → 2 H₂O + O₂↑",
        stoichiometry: &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -2.0,
                phase: Phase::Liquid,
            },
            StoichiometricTerm {
                species: "water",
                coefficient: 2.0,
                phase: Phase::Liquid,
            },
            StoichiometricTerm {
                species: "O2",
                coefficient: 1.0,
                phase: Phase::Gas,
            },
        ],
        // The water is not optional. Leaving it out of the products
        // destroyed 36 g/mol of matter per extent while the equation string
        // right above claimed otherwise.
        locality: Locality::Bulk(Phase::Aqueous),
        forward: RateExpression {
            orders: &[OrderTerm {
                species: "H2O2",
                phase: Some(Phase::Liquid),
                order: 1.0,
            }],
            arrhenius: RateLaw {
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
                temperature_exponent: 0.0,
                activation_energy: 75_000.0,
            },
        },
        reverse: None,
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
        sites: &[],
        electrons: 0.0,
        validity: Validity {
            temperature_k: None,
            pressure_pa: None,
            note: "calibrated near room temperature for dilute aqueous peroxide; stabilisers are not represented and Arrhenius extrapolation is not independently validated",
        },
        uncertainty: Uncertainty {
            relative: None,
            note: "absolute rates are indicative; catalyst surface area is not represented",
        },
        source_ids: &["kerotakis:kinetics:peroxide-decomposition"],
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

/// A compiled set of reactions evaluated together.
#[derive(Debug, Clone, Copy)]
pub struct ReactionNetwork<'a> {
    pub id: &'a str,
    pub reactions: &'a [KineticReaction<'a>],
}

pub const NETWORK: ReactionNetwork<'static> = ReactionNetwork {
    id: "kerotakis-curated-kinetics",
    reactions: REGISTRY,
};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum NetworkLintError {
    #[error("{reaction}: stoichiometric coefficient for {species} is not finite and non-zero")]
    InvalidCoefficient { reaction: String, species: String },
    #[error("{reaction}: species '{species}' is absent from the registry")]
    UnknownSpecies { reaction: String, species: String },
    #[error("{reaction}: formula for '{species}' cannot be parsed: {detail}")]
    InvalidFormula {
        reaction: String,
        species: String,
        detail: String,
    },
    #[error("{reaction}: element {element} has net coefficient {imbalance:+.6}")]
    ElementImbalance {
        reaction: String,
        element: String,
        imbalance: f64,
    },
    #[error("{reaction}: charge/electron bookkeeping has net coefficient {imbalance:+.6}")]
    ChargeImbalance { reaction: String, imbalance: f64 },
    #[error("{reaction}: site '{site}' has net coefficient {imbalance:+.6}")]
    SiteImbalance {
        reaction: String,
        site: String,
        imbalance: f64,
    },
}

/// Audit the exact stoichiometric vectors the runtime applies.
pub fn lint_network(network: &ReactionNetwork<'_>) -> Result<(), Vec<NetworkLintError>> {
    let mut errors = Vec::new();
    for reaction in network.reactions {
        errors.extend(lint_reaction(reaction));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn lint_reaction(reaction: &KineticReaction<'_>) -> Vec<NetworkLintError> {
    const TOLERANCE: f64 = 1e-9;
    let mut errors = Vec::new();
    let mut elements: BTreeMap<String, f64> = BTreeMap::new();
    let mut charge = 0.0;

    for term in reaction.stoichiometry {
        if !term.coefficient.is_finite() || term.coefficient == 0.0 {
            errors.push(NetworkLintError::InvalidCoefficient {
                reaction: reaction.id.to_string(),
                species: term.species.to_string(),
            });
            continue;
        }
        let Some(data) = crate::species::lookup_key(term.species) else {
            errors.push(NetworkLintError::UnknownSpecies {
                reaction: reaction.id.to_string(),
                species: term.species.to_string(),
            });
            continue;
        };
        let formula = match crate::stoich::parse_formula(data.formula) {
            Ok(formula) => formula,
            Err(error) => {
                errors.push(NetworkLintError::InvalidFormula {
                    reaction: reaction.id.to_string(),
                    species: term.species.to_string(),
                    detail: error.to_string(),
                });
                continue;
            }
        };
        for (element, count) in formula.counts {
            *elements.entry(element).or_default() += term.coefficient * count;
        }
        charge += term.coefficient * formula.charge;
    }

    for (element, imbalance) in elements {
        if imbalance.abs() > TOLERANCE {
            errors.push(NetworkLintError::ElementImbalance {
                reaction: reaction.id.to_string(),
                element,
                imbalance,
            });
        }
    }

    // An electron on the product side has coefficient +1 and charge -1.
    // `electrons` therefore subtracts from the molecular charge delta.
    let charge_imbalance = charge - reaction.electrons;
    if charge_imbalance.abs() > TOLERANCE {
        errors.push(NetworkLintError::ChargeImbalance {
            reaction: reaction.id.to_string(),
            imbalance: charge_imbalance,
        });
    }

    let mut sites: BTreeMap<&str, f64> = BTreeMap::new();
    for term in reaction.sites {
        *sites.entry(term.site).or_default() += term.coefficient;
    }
    for (site, imbalance) in sites {
        if imbalance.abs() > TOLERANCE {
            errors.push(NetworkLintError::SiteImbalance {
                reaction: reaction.id.to_string(),
                site: site.to_string(),
                imbalance,
            });
        }
    }
    errors
}

/// Concentration to use for one order term, mol/L.
fn phase_moles(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == key && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

fn term_concentration(vessel: &Vessel, term: &OrderTerm<'_>, litres: f64) -> Option<f64> {
    if term.species == PROTON {
        // No characterised solution means no known acidity: the rate is
        // unknown rather than zero, and the caller treats it as "cannot
        // proceed" rather than "does not react".
        return vessel.solution.as_ref().map(|s| 10f64.powf(-s.ph));
    }
    let moles = term
        .phase
        .map(|phase| phase_moles(vessel, term.species, phase))
        .unwrap_or_else(|| vessel.moles_of(&SpeciesId::new(term.species)).0);
    Some(moles / litres)
}

pub fn lookup(id: &str) -> Option<&'static KineticReaction<'static>> {
    REGISTRY.iter().find(|r| r.id == id)
}

/// Which reactions have all their reactants present in this vessel.
pub fn applicable(vessel: &Vessel) -> Vec<&'static KineticReaction<'static>> {
    REGISTRY
        .iter()
        .filter(|reaction| reaction.can_run(vessel))
        .collect()
}

impl<'a> KineticReaction<'a> {
    pub fn reactants(&self) -> impl Iterator<Item = &StoichiometricTerm<'a>> {
        self.stoichiometry
            .iter()
            .filter(|term| term.coefficient < 0.0)
    }

    pub fn products(&self) -> impl Iterator<Item = &StoichiometricTerm<'a>> {
        self.stoichiometry
            .iter()
            .filter(|term| term.coefficient > 0.0)
    }

    pub fn is_reversible(&self) -> bool {
        self.reverse.is_some()
    }

    fn in_validity_domain(&self, vessel: &Vessel) -> bool {
        self.validity
            .temperature_k
            .is_none_or(|range| range.contains(vessel.temperature.0))
            && self
                .validity
                .pressure_pa
                .is_none_or(|range| range.contains(vessel.pressure.0))
    }

    fn direction_available(&self, vessel: &Vessel, forward: bool) -> bool {
        self.stoichiometry.iter().all(|term| {
            let consumed = if forward {
                term.coefficient < 0.0
            } else {
                term.coefficient > 0.0
            };
            !consumed || phase_moles(vessel, term.species, term.phase) > DEPLETED
        })
    }

    pub fn can_run(&self, vessel: &Vessel) -> bool {
        self.in_validity_domain(vessel)
            && (self.direction_available(vessel, true)
                || (self.reverse.is_some() && self.direction_available(vessel, false)))
    }

    /// The activation energy in force, given what is in the vessel: the
    /// best catalyst present, or the uncatalysed value.
    pub fn effective_activation_energy(&self, vessel: &Vessel) -> (f64, Option<&Catalyst<'a>>) {
        let mut best: Option<&Catalyst<'a>> = None;
        for c in self.catalysts {
            if vessel.moles_of(&SpeciesId::new(c.species)).0 > 0.0
                && best.is_none_or(|b| c.activation_energy < b.activation_energy)
            {
                best = Some(c);
            }
        }
        match best {
            Some(c) => (c.activation_energy, Some(c)),
            None => (self.forward.arrhenius.activation_energy, None),
        }
    }

    fn expression_rate(&self, vessel: &Vessel, expression: RateExpression<'a>) -> f64 {
        let litres = vessel.liquid_volume().0;
        if litres <= 0.0 {
            return 0.0;
        }
        let catalyst_ea = self
            .catalysts
            .iter()
            .filter(|catalyst| vessel.moles_of(&SpeciesId::new(catalyst.species)).0 > 0.0)
            .map(|catalyst| catalyst.activation_energy)
            .reduce(f64::min);
        let ea = catalyst_ea
            .map(|candidate| candidate.min(expression.arrhenius.activation_energy))
            .unwrap_or(expression.arrhenius.activation_energy);
        let law = RateLaw {
            pre_exponential: expression.arrhenius.pre_exponential,
            temperature_exponent: expression.arrhenius.temperature_exponent,
            activation_energy: ea,
        };
        let k = law.rate_constant(vessel.temperature.0);
        let mut rate = k;
        for term in expression.orders {
            let Some(c) = term_concentration(vessel, term, litres) else {
                return 0.0;
            };
            if c <= 0.0 {
                return 0.0;
            }
            rate *= c.powf(term.order);
        }
        rate
    }

    /// Net rate in mol·L⁻¹·s⁻¹. Positive follows the declared equation;
    /// negative follows the reverse expression.
    pub fn rate_now(&self, vessel: &Vessel) -> f64 {
        if !self.in_validity_domain(vessel) {
            return 0.0;
        }
        let forward = if self.direction_available(vessel, true) {
            self.expression_rate(vessel, self.forward)
        } else {
            0.0
        };
        let reverse = self
            .reverse
            .filter(|_| self.direction_available(vessel, false))
            .map(|expression| self.expression_rate(vessel, expression))
            .unwrap_or(0.0);
        forward - reverse
    }
}

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
pub(super) const DEPLETED: f64 = 1e-12;

/// How far each reaction has run, in mol/L of "extent".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub steps: usize,
}

/// Advance this vessel's chemistry by `seconds` with adaptive implicit BDF.
///
/// The fallible API is intentional: non-convergence is a simulation error,
/// not evidence that no chemistry happened. The bench propagates it to CLI
/// and GUI clients instead of silently accepting a partial trajectory.
pub fn advance_network<'a>(
    vessel: &mut Vessel,
    seconds: f64,
    network: &'a ReactionNetwork<'a>,
) -> Result<Vec<(&'a KineticReaction<'a>, Moles)>, IntegrationError> {
    advance_network_with_options(vessel, seconds, network, IntegrationOptions::default())
        .map(|report| report.extents)
}

/// Commit all reaction extents as one state delta.
///
/// Rates are evaluated from one state, so their changes must be committed as
/// one state transition too. Applying reactions one by one lets an earlier
/// entry's product feed a later entry inside a nominally simultaneous step,
/// making results depend on registry order. Aggregation also gives one place
/// to scale an unexpectedly aggressive midpoint step before any reactant can
/// become negative.
fn apply_coupled_extents(
    vessel: &mut Vessel,
    reactions: &[KineticReaction<'_>],
    extents: &[f64],
) -> f64 {
    let mut deltas: Vec<(&str, Phase, f64)> = Vec::new();
    for (reaction, extent) in reactions.iter().zip(extents) {
        for term in reaction.stoichiometry {
            let change = term.coefficient * extent;
            if let Some((_, _, total)) = deltas
                .iter_mut()
                .find(|(species, phase, _)| *species == term.species && *phase == term.phase)
            {
                *total += change;
            } else {
                deltas.push((term.species, term.phase, change));
            }
        }
    }

    let accepted_fraction = deltas.iter().filter(|(_, _, change)| *change < 0.0).fold(
        1.0f64,
        |fraction, (species, phase, change)| {
            let available = phase_moles(vessel, species, *phase);
            fraction.min((available / -change).clamp(0.0, 1.0))
        },
    );

    // Withdraw first, then deposit. Because changes have been aggregated by
    // species and phase, neither loop can observe an intermediate produced by
    // another reaction in this same substep.
    for (species, phase, change) in &deltas {
        let accepted = change * accepted_fraction;
        if accepted < 0.0 {
            withdraw_phase(vessel, species, *phase, -accepted);
        }
    }
    for (species, phase, change) in deltas {
        let accepted = change * accepted_fraction;
        if accepted > 0.0 {
            vessel.deposit(SpeciesId::new(species), Moles(accepted), phase);
        }
    }
    accepted_fraction
}

fn withdraw_phase(vessel: &mut Vessel, species: &str, phase: Phase, moles: f64) -> f64 {
    let mut remaining = moles;
    for portion in &mut vessel.contents {
        if portion.species.0 == species && portion.phase == phase && remaining > 0.0 {
            let take = portion.moles.0.min(remaining);
            portion.moles.0 -= take;
            remaining -= take;
        }
    }
    vessel.contents.retain(|portion| portion.moles.0 > 1e-15);
    moles - remaining
}

/// Advance the built-in curated network. This is the stable bench-facing API.
pub fn advance(
    vessel: &mut Vessel,
    seconds: f64,
) -> Result<Vec<(&'static KineticReaction<'static>, Moles)>, IntegrationError> {
    advance_network(vessel, seconds, &NETWORK)
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
            temperature_exponent: 0.0,
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
            temperature_exponent: 0.0,
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
            temperature_exponent: 0.0,
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
        advance(&mut cold, 20.0).unwrap();
        advance(&mut warm, 20.0).unwrap();
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
            advance(&mut v, 1.0).unwrap();
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
        advance(&mut v, 3600.0).unwrap();
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
                ("H2O2", 0.1, Phase::Liquid),
            ],
            25.0,
        );
        let with_mno2 = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Liquid),
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
                ("H2O2", 0.1, Phase::Liquid),
                ("MnO2", 0.001, Phase::Solid),
            ],
            25.0,
        );
        advance(&mut v, 60.0).unwrap();
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
        // Keep peroxide dilute so the closed-form constant-volume assumption
        // is the same approximation the model makes for an aqueous mixture.
        let initial_moles = 1e-4;
        for seconds in [1.0, 10.0, 120.0, 600.0] {
            let mut v = vessel_with(
                &[
                    ("water", 5.5343, Phase::Liquid),
                    ("H2O2", initial_moles, Phase::Liquid),
                ],
                25.0,
            );
            let litres = v.liquid_volume().0;
            let c0 = initial_moles / litres;
            let k = r.forward.arrhenius.rate_constant(298.15);
            advance(&mut v, seconds).unwrap();
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
        let initial_moles = 1e-4;
        let mut v = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", initial_moles, Phase::Liquid),
                ("catalase", 1e-6, Phase::Aqueous),
            ],
            25.0,
        );
        let litres = v.liquid_volume().0;
        let c0 = initial_moles / litres;
        let law = RateLaw {
            pre_exponential: r.forward.arrhenius.pre_exponential,
            temperature_exponent: r.forward.arrhenius.temperature_exponent,
            activation_energy: 23_000.0,
        };
        let k = law.rate_constant(298.15);
        let seconds = 2.0;
        advance(&mut v, seconds).unwrap();
        let got = v.moles_of(&SpeciesId::new("H2O2")).0 / litres;
        let exact = c0 * (-2.0 * k * seconds).exp();
        assert!(
            (got - exact).abs() / exact.max(1e-30) < 5e-3 || (got.abs() < 1e-9 && exact < 1e-9),
            "catalysed: integrator {got:.6e}, exact {exact:.6e}"
        );
    }

    #[test]
    fn stiff_depletion_is_detected_after_rejected_steps() {
        let mut vessel = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 1e-4, Phase::Liquid),
                ("catalase", 1e-6, Phase::Aqueous),
            ],
            25.0,
        );
        let report = advance_network_with_options(
            &mut vessel,
            0.1,
            &NETWORK,
            IntegrationOptions {
                relative_tolerance: 1e-10,
                absolute_tolerance_moles: 1e-16,
                // Deliberately far too large for a millisecond-scale decay.
                // Error control must reject and retry it rather than step
                // across zero.
                initial_step_seconds: 0.1,
            },
        )
        .unwrap();

        assert!(report.statistics.accepted_steps > 0);
        assert!(
            report.statistics.rejected_steps > 0,
            "the deliberately overlarge first step was not rejected: {:?}",
            report.statistics
        );
        assert!(
            report.statistics.depletion_events > 0,
            "reactant depletion did not produce a root event: {:?}",
            report.statistics
        );
        assert!(phase_moles(&vessel, "H2O2", Phase::Liquid) >= 0.0);
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
            for term in r.reactants() {
                v.deposit(SpeciesId::new(term.species), Moles(0.02), term.phase);
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
            let moved = advance(&mut v, 600.0).unwrap();
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

    const TEST_FORWARD_AQUEOUS: RateExpression = RateExpression {
        arrhenius: RateLaw {
            pre_exponential: 1.0,
            temperature_exponent: 0.0,
            activation_energy: 0.0,
        },
        orders: &[OrderTerm {
            species: "H2O2",
            phase: Some(Phase::Aqueous),
            order: 1.0,
        }],
    };

    const TEST_FORWARD_LIQUID: RateExpression = RateExpression {
        arrhenius: RateLaw {
            pre_exponential: 1.0,
            temperature_exponent: 0.0,
            activation_energy: 0.0,
        },
        orders: &[OrderTerm {
            species: "H2O2",
            phase: Some(Phase::Liquid),
            order: 1.0,
        }],
    };

    fn test_reaction(
        id: &'static str,
        stoichiometry: &'static [StoichiometricTerm<'static>],
        forward: RateExpression<'static>,
        reverse: Option<RateExpression<'static>>,
    ) -> KineticReaction<'static> {
        KineticReaction {
            id,
            equation: "H₂O₂ → H₂O₂",
            stoichiometry,
            locality: Locality::Bulk(Phase::Aqueous),
            forward,
            reverse,
            catalysts: &[],
            sites: &[],
            electrons: 0.0,
            validity: Validity::default(),
            uncertainty: Uncertainty {
                relative: Some(0.0),
                note: "exact test parameter",
            },
            source_ids: &["kerotakis:test"],
            provenance: "project-authored test mechanism",
        }
    }

    #[test]
    fn built_in_network_passes_structural_conservation_lint() {
        lint_network(&NETWORK).unwrap();
        for reaction in REGISTRY {
            assert!(!reaction.source_ids.is_empty(), "{}", reaction.id);
            assert!(!reaction.validity.note.is_empty(), "{}", reaction.id);
            assert!(!reaction.uncertainty.note.is_empty(), "{}", reaction.id);
        }
        assert_eq!(
            lookup("thiosulfate-acid").unwrap().forward.dimensions(),
            RateDimensions {
                mole: -1.0,
                litre: 1.0,
                second: -1.0,
            }
        );
    }

    #[test]
    fn lint_reports_elements_charge_sites_and_declared_electrons() {
        const BAD_MATTER: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "water",
                coefficient: 1.0,
                phase: Phase::Liquid,
            },
        ];
        let mut bad = test_reaction("bad-network-entry", BAD_MATTER, TEST_FORWARD_AQUEOUS, None);
        bad.sites = &[SiteTerm {
            site: "surface-vacancy",
            coefficient: -1.0,
        }];
        let errors = lint_reaction(&bad);
        assert!(errors.iter().any(|e| matches!(
            e,
            NetworkLintError::ElementImbalance { element, .. } if element == "O"
        )));
        assert!(errors
            .iter()
            .any(|e| matches!(e, NetworkLintError::SiteImbalance { .. })));

        const ZINC_HALF_REACTION: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "Zn",
                coefficient: -1.0,
                phase: Phase::Solid,
            },
            StoichiometricTerm {
                species: "Zn+2",
                coefficient: 1.0,
                phase: Phase::Aqueous,
            },
        ];
        let mut half = test_reaction(
            "zinc-half-reaction",
            ZINC_HALF_REACTION,
            TEST_FORWARD_AQUEOUS,
            None,
        );
        assert!(lint_reaction(&half)
            .iter()
            .any(|e| matches!(e, NetworkLintError::ChargeImbalance { .. })));
        half.electrons = 2.0;
        assert!(lint_reaction(&half).is_empty());
    }

    #[test]
    fn a_product_can_activate_the_next_reaction_during_one_wait() {
        const TO_LIQUID: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Liquid,
            },
        ];
        const TO_GAS: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Liquid,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Gas,
            },
        ];
        let reactions = [
            test_reaction("aqueous-to-liquid", TO_LIQUID, TEST_FORWARD_AQUEOUS, None),
            test_reaction("liquid-to-gas", TO_GAS, TEST_FORWARD_LIQUID, None),
        ];
        let network = ReactionNetwork {
            id: "consecutive-test",
            reactions: &reactions,
        };
        lint_network(&network).unwrap();

        let mut vessel = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
            ],
            25.0,
        );
        assert_eq!(phase_moles(&vessel, "H2O2", Phase::Gas), 0.0);
        let moved = advance_network(&mut vessel, 1.0, &network).unwrap();
        assert_eq!(
            moved.len(),
            2,
            "the second step must activate in the same wait"
        );
        assert!(phase_moles(&vessel, "H2O2", Phase::Gas) > 0.0);
    }

    #[test]
    fn competing_paths_share_a_reactant_without_overdrawing_it() {
        const TO_LIQUID: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Liquid,
            },
        ];
        const TO_GAS: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Gas,
            },
        ];
        let reactions = [
            test_reaction("path-liquid", TO_LIQUID, TEST_FORWARD_AQUEOUS, None),
            test_reaction("path-gas", TO_GAS, TEST_FORWARD_AQUEOUS, None),
        ];
        let network = ReactionNetwork {
            id: "competing-test",
            reactions: &reactions,
        };
        let mut vessel = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
            ],
            25.0,
        );
        let mut reversed_vessel = vessel.clone();
        let moved = advance_network(&mut vessel, 20.0, &network).unwrap();
        assert_eq!(moved.len(), 2);
        assert!(phase_moles(&vessel, "H2O2", Phase::Aqueous) >= 0.0);
        let total = [Phase::Aqueous, Phase::Liquid, Phase::Gas]
            .into_iter()
            .map(|phase| phase_moles(&vessel, "H2O2", phase))
            .sum::<f64>();
        assert!(
            (total - 0.1).abs() < 1e-9,
            "network changed total matter: {total}"
        );

        let reversed_reactions = [
            test_reaction("path-gas", TO_GAS, TEST_FORWARD_AQUEOUS, None),
            test_reaction("path-liquid", TO_LIQUID, TEST_FORWARD_AQUEOUS, None),
        ];
        let reversed_network = ReactionNetwork {
            id: "reversed-competing-test",
            reactions: &reversed_reactions,
        };
        advance_network(&mut reversed_vessel, 20.0, &reversed_network).unwrap();
        for phase in [Phase::Aqueous, Phase::Liquid, Phase::Gas] {
            let forward = phase_moles(&vessel, "H2O2", phase);
            let reversed = phase_moles(&reversed_vessel, "H2O2", phase);
            assert!(
                (forward - reversed).abs() < 1e-12,
                "registry order changed {phase:?}: {forward} versus {reversed}"
            );
        }
    }

    #[test]
    fn an_overlarge_coupled_step_is_scaled_before_any_product_is_deposited() {
        const TO_LIQUID: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Liquid,
            },
        ];
        const TO_GAS: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Gas,
            },
        ];
        let reactions = [
            test_reaction("path-liquid", TO_LIQUID, TEST_FORWARD_AQUEOUS, None),
            test_reaction("path-gas", TO_GAS, TEST_FORWARD_AQUEOUS, None),
        ];
        let mut vessel = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
            ],
            25.0,
        );

        // Each path proposes consuming the entire inventory. The coupled
        // commit must accept half of both, not let the first path win and
        // then create the second path's product from a truncated withdrawal.
        let accepted = apply_coupled_extents(&mut vessel, &reactions, &[0.1, 0.1]);
        assert!((accepted - 0.5).abs() < 1e-12);
        assert!(phase_moles(&vessel, "H2O2", Phase::Aqueous) < 1e-12);
        assert!((phase_moles(&vessel, "H2O2", Phase::Liquid) - 0.05).abs() < 1e-12);
        assert!((phase_moles(&vessel, "H2O2", Phase::Gas) - 0.05).abs() < 1e-12);
    }

    #[test]
    fn a_reverse_expression_drives_back_toward_balance() {
        const PHASE_CHANGE: &[StoichiometricTerm] = &[
            StoichiometricTerm {
                species: "H2O2",
                coefficient: -1.0,
                phase: Phase::Aqueous,
            },
            StoichiometricTerm {
                species: "H2O2",
                coefficient: 1.0,
                phase: Phase::Liquid,
            },
        ];
        let reaction = test_reaction(
            "reversible-test",
            PHASE_CHANGE,
            TEST_FORWARD_AQUEOUS,
            Some(TEST_FORWARD_LIQUID),
        );
        assert!(reaction.is_reversible());
        let reactions = [reaction];
        let network = ReactionNetwork {
            id: "reversible-network",
            reactions: &reactions,
        };
        let mut vessel = vessel_with(
            &[
                ("water", 5.5343, Phase::Liquid),
                ("H2O2", 0.1, Phase::Aqueous),
            ],
            25.0,
        );
        advance_network(&mut vessel, 10.0, &network).unwrap();
        let aqueous = phase_moles(&vessel, "H2O2", Phase::Aqueous);
        let liquid = phase_moles(&vessel, "H2O2", Phase::Liquid);
        assert!((aqueous - liquid).abs() < 2e-3, "{aqueous} versus {liquid}");
    }
}
