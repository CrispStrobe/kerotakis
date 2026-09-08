//! ELEC-005/006/008: Electrochemical control modes, transport limits,
//! and deposit/passivation tracking.

use serde::{Deserialize, Serialize};

use crate::butler_volmer::ButlerVolmerParams;
pub use crate::constants::FARADAY;
use crate::constants::GAS_CONSTANT;
pub use crate::heterogeneous::{transport_limited_flux, ReactiveSurface};

/// Backward-compatible public name for the canonical gas constant.
pub const R_GAS: f64 = GAS_CONSTANT;

// ── ELEC-005: Galvanostatic and potentiostatic control ─────────────

/// How the electrochemical cell is driven.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CellControl {
    /// Constant current (galvanostatic): the power supply delivers a
    /// fixed current regardless of voltage.
    Galvanostatic { current_amps: f64 },
    /// Constant voltage (potentiostatic): the power supply maintains a
    /// fixed potential difference.
    Potentiostatic { voltage: f64 },
    /// Open circuit: no external current flows.
    OpenCircuit,
}

// ── ELEC-006: Ohmic and diffusion limits ───────────────────────────

/// Transport limitations that reduce the observed current from the
/// kinetic (Butler–Volmer) limit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransportLimits {
    /// Solution resistance between electrodes, Ω.
    pub solution_resistance_ohm: f64,
    /// Limiting current density for the cathodic reaction, A/m².
    /// When |j| approaches this, diffusion controls the rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiting_current_cathodic: Option<f64>,
    /// Limiting current density for the anodic reaction, A/m².
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiting_current_anodic: Option<f64>,
}

impl TransportLimits {
    /// IR drop: voltage lost to solution resistance at a given current.
    pub fn ir_drop(&self, current_amps: f64) -> f64 {
        current_amps.abs() * self.solution_resistance_ohm
    }

    /// Signed solution-potential drop for implicit current-control solves.
    pub fn signed_ir_drop(&self, current_amps: f64) -> f64 {
        current_amps * self.solution_resistance_ohm
    }

    /// Whether the current is diffusion-limited on the cathodic side.
    pub fn is_cathodic_limited(&self, j: f64) -> bool {
        self.limiting_current_cathodic
            .map(|lim| j.abs() > 0.95 * lim)
            .unwrap_or(false)
    }
}

// ── ELEC-008: Deposit and passivation ──────────────────────────────

/// How an electrode's surface condition changes with deposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassivationEffect {
    /// The deposit conducts and doesn't block further reaction.
    Conductive,
    /// The deposit is an insulating oxide layer that blocks current.
    Passivating,
    /// The deposit partially blocks: current decreases with coverage.
    SemiPassivating,
}

/// Current coverage state of an electrode surface.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurfaceCoverage {
    /// Fraction of surface covered by deposit (0.0–1.0).
    pub theta: f64,
    /// How the deposit affects further reaction.
    pub effect: PassivationEffect,
}

impl SurfaceCoverage {
    /// Effective area fraction available for reaction.
    pub fn active_fraction(&self) -> f64 {
        match self.effect {
            PassivationEffect::Conductive => 1.0,
            PassivationEffect::Passivating => 1.0 - self.theta,
            PassivationEffect::SemiPassivating => (1.0 - self.theta).sqrt(),
        }
    }
}

// ── ELEC-003: Reviewed kinetic parameter records ──────────────────

/// Exchange-current density record with provenance (ELEC-003).
///
/// No folklore table enters runtime. Each value needs an allowlisted
/// source and stated validity conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCurrentRecord {
    /// Stable identifier for this measured parameter set.
    pub id: String,
    /// The electrode reaction (e.g. "Cu²⁺/Cu").
    pub reaction: String,
    /// Electrode substrate or alloy on which the value was measured.
    pub electrode_material: String,
    /// Canonical SI kinetic parameters. Current density is A/m².
    pub kinetics: ButlerVolmerParams,
    /// Conditions under which this record may be applied.
    pub validity: KineticValidityDomain,
    /// Source citation.
    pub source: String,
    /// Licence applying to the distributable parameter record.
    pub license: PermissiveDataLicense,
    /// Relative standard uncertainty when the source supports one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_uncertainty: Option<f64>,
    /// Joint admissible range of fitted kinetic parameters. This records
    /// protocol sensitivity and scatter without pretending the parameters are
    /// independent Gaussian errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_envelope: Option<KineticParameterEnvelope>,
    /// Required account of measurement scatter, fit sensitivity and omissions.
    pub uncertainty_note: String,
    /// Whether this record has been reviewed for the runtime allowlist.
    pub reviewed: bool,
}

/// Curated exchange-current records from reviewed sources.
/// Empty until a permissively licensed measurement can be represented with
/// its complete material, surface, electrolyte and uncertainty domain.
pub const EXCHANGE_CURRENTS: &[ExchangeCurrentRecord] = &[];

/// Closed allowlist for parameter records that may ship with the engine.
/// An NC, copyleft or unknown licence cannot be represented accidentally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissiveDataLicense {
    PublicDomain,
    Cc0,
    CcBy40,
    Mit,
    Bsd3Clause,
    Apache20,
}

/// One activity constraint carried by a kinetic parameter record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityBound {
    pub species: String,
    pub minimum: f64,
    pub maximum: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParameterBound {
    pub minimum: f64,
    pub maximum: f64,
}

impl ParameterBound {
    fn validate(self, strictly_positive: bool) -> bool {
        self.minimum.is_finite()
            && self.maximum.is_finite()
            && if strictly_positive {
                self.minimum > 0.0
            } else {
                self.minimum >= 0.0
            }
            && self.minimum <= self.maximum
    }

    fn contains(self, value: f64) -> bool {
        value.is_finite() && value >= self.minimum && value <= self.maximum
    }
}

/// Correlated bounds reported for one fitted parameter set. Runtime uses the
/// reviewed nominal parameters; sensitivity tools may evaluate this envelope
/// without manufacturing a probability distribution the source did not give.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KineticParameterEnvelope {
    pub exchange_current_density_a_per_m2: ParameterBound,
    pub alpha_anodic: ParameterBound,
    pub alpha_cathodic: ParameterBound,
}

impl KineticParameterEnvelope {
    fn validates_nominal(self, nominal: ButlerVolmerParams) -> bool {
        self.exchange_current_density_a_per_m2.validate(true)
            && self.alpha_anodic.validate(true)
            && self.alpha_cathodic.validate(true)
            && self.exchange_current_density_a_per_m2.contains(nominal.j0)
            && self.alpha_anodic.contains(nominal.alpha_a)
            && self.alpha_cathodic.contains(nominal.alpha_c)
    }
}

/// Actual flow state at the electrode. Unspecified quantities cannot satisfy
/// a record that declares a bound for them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct HydrodynamicCondition {
    pub rotation_rate_rpm: Option<f64>,
    pub fluid_velocity_m_per_s: Option<f64>,
    pub diffusion_layer_m: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct HydrodynamicDomain {
    pub rotation_rate_rpm: Option<ParameterBound>,
    pub fluid_velocity_m_per_s: Option<ParameterBound>,
    pub diffusion_layer_m: Option<ParameterBound>,
}

impl HydrodynamicDomain {
    fn validate(self) -> bool {
        [
            self.rotation_rate_rpm,
            self.fluid_velocity_m_per_s,
            self.diffusion_layer_m,
        ]
        .into_iter()
        .flatten()
        .all(|bound| bound.validate(false))
    }

    fn accepts(self, actual: HydrodynamicCondition) -> bool {
        self.validate()
            && self
                .rotation_rate_rpm
                .is_none_or(|bound| actual.rotation_rate_rpm.is_some_and(|v| bound.contains(v)))
            && self.fluid_velocity_m_per_s.is_none_or(|bound| {
                actual
                    .fluid_velocity_m_per_s
                    .is_some_and(|v| bound.contains(v))
            })
            && self
                .diffusion_layer_m
                .is_none_or(|bound| actual.diffusion_layer_m.is_some_and(|v| bound.contains(v)))
    }
}

/// The measured domain of an electrochemical kinetic parameter set.
///
/// Thermodynamic feasibility may be extrapolated by the equilibrium solver;
/// kinetic measurements may not. A caller that cannot satisfy every bound
/// must return an unquantifiable result instead of borrowing the record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KineticValidityDomain {
    pub temperature_min_k: f64,
    pub temperature_max_k: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub activities: Vec<ActivityBound>,
    pub surface_preparation: String,
    #[serde(default)]
    pub hydrodynamics: HydrodynamicDomain,
    pub note: String,
}

impl KineticValidityDomain {
    pub fn validate(&self) -> bool {
        self.temperature_min_k.is_finite()
            && self.temperature_max_k.is_finite()
            && self.temperature_min_k > 0.0
            && self.temperature_min_k <= self.temperature_max_k
            && !self.surface_preparation.trim().is_empty()
            && !self.note.trim().is_empty()
            && self.hydrodynamics.validate()
            && self.activities.iter().all(|bound| {
                !bound.species.trim().is_empty()
                    && bound.minimum.is_finite()
                    && bound.maximum.is_finite()
                    && bound.minimum >= 0.0
                    && bound.minimum <= bound.maximum
            })
    }

    pub fn accepts(
        &self,
        temperature_k: f64,
        activities: &[(String, f64)],
        surface_preparation: Option<&str>,
        hydrodynamics: HydrodynamicCondition,
    ) -> bool {
        self.validate()
            && temperature_k.is_finite()
            && temperature_k >= self.temperature_min_k
            && temperature_k <= self.temperature_max_k
            && surface_preparation == Some(self.surface_preparation.as_str())
            && self.hydrodynamics.accepts(hydrodynamics)
            && self.activities.iter().all(|bound| {
                activities
                    .iter()
                    .find(|(species, _)| species == &bound.species)
                    .map(|(_, activity)| {
                        activity.is_finite()
                            && *activity >= bound.minimum
                            && *activity <= bound.maximum
                    })
                    .unwrap_or(false)
            })
    }
}

impl ExchangeCurrentRecord {
    pub fn applies_to(
        &self,
        electrode_material: &str,
        temperature_k: f64,
        activities: &[(String, f64)],
        surface_preparation: Option<&str>,
        hydrodynamics: HydrodynamicCondition,
    ) -> bool {
        self.reviewed
            && !self.id.trim().is_empty()
            && !self.reaction.trim().is_empty()
            && !self.source.trim().is_empty()
            && !self.uncertainty_note.trim().is_empty()
            && self
                .relative_uncertainty
                .is_none_or(|value| value.is_finite() && value >= 0.0)
            && self
                .parameter_envelope
                .is_none_or(|envelope| envelope.validates_nominal(self.kinetics))
            && self.electrode_material == electrode_material
            && self.kinetics.validate().is_ok()
            && self.validity.accepts(
                temperature_k,
                activities,
                surface_preparation,
                hydrodynamics,
            )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterSelectionError {
    NoApplicableRecord {
        reaction: String,
        electrode_material: String,
    },
    AmbiguousRecords {
        record_ids: Vec<String>,
    },
}

/// Select one reviewed parameter domain without an order-dependent fallback.
/// Overlapping records are an ambiguity to curate, not permission to choose
/// whichever happened to be listed first.
pub fn select_exchange_current<'a>(
    records: &'a [ExchangeCurrentRecord],
    reaction: &str,
    electrode_material: &str,
    temperature_k: f64,
    activities: &[(String, f64)],
    surface_preparation: Option<&str>,
    hydrodynamics: HydrodynamicCondition,
) -> Result<&'a ExchangeCurrentRecord, ParameterSelectionError> {
    let matching: Vec<_> = records
        .iter()
        .filter(|record| {
            record.reaction == reaction
                && record.applies_to(
                    electrode_material,
                    temperature_k,
                    activities,
                    surface_preparation,
                    hydrodynamics,
                )
        })
        .collect();
    match matching.as_slice() {
        [record] => Ok(record),
        [] => Err(ParameterSelectionError::NoApplicableRecord {
            reaction: reaction.to_owned(),
            electrode_material: electrode_material.to_owned(),
        }),
        records => Err(ParameterSelectionError::AmbiguousRecords {
            record_ids: records.iter().map(|record| record.id.clone()).collect(),
        }),
    }
}

// ── ELEC-004: Butler-Volmer / Tafel kinetics ──────────────────────

/// Butler-Volmer current density at a given overpotential.
///
/// j = j₀ [exp(α_a·F·η/RT) − exp(−α_c·F·η/RT)]
///
/// Positive = anodic (oxidation).
pub fn butler_volmer(
    exchange_current_density: f64,
    alpha_a: f64,
    alpha_c: f64,
    overpotential_v: f64,
    temperature_k: f64,
) -> f64 {
    ButlerVolmerParams {
        j0: exchange_current_density,
        alpha_a,
        alpha_c,
        n: 1.0,
    }
    .current_density(overpotential_v, temperature_k)
}

/// Tafel approximation for high overpotentials (|η| >> RT/F).
pub fn tafel_overpotential(
    current_density: f64,
    exchange_current_density: f64,
    alpha: f64,
    temperature_k: f64,
) -> f64 {
    let b = 2.303 * GAS_CONSTANT * temperature_k / (alpha * FARADAY);
    b * (current_density.abs() / exchange_current_density)
        .max(1e-30)
        .log10()
}

/// Diffusion-limited current density (Levich boundary-layer model).
pub fn limiting_current_density(
    electrons: f64,
    diffusivity_cm2_per_s: f64,
    bulk_concentration_mol_per_cm3: f64,
    diffusion_layer_cm: f64,
) -> f64 {
    electrons * FARADAY * diffusivity_cm2_per_s * bulk_concentration_mol_per_cm3
        / diffusion_layer_cm
}

/// Diffusion-limited current density in canonical SI units.
pub fn limiting_current_density_si(
    electrons: f64,
    diffusivity_m2_per_s: f64,
    bulk_concentration_mol_per_m3: f64,
    diffusion_layer_m: f64,
) -> f64 {
    electrons * FARADAY * diffusivity_m2_per_s * bulk_concentration_mol_per_m3 / diffusion_layer_m
}

// ── Coupled partial-current solver ─────────────────────────────────

/// One independently parameterised redox couple available on a surface.
/// Its equilibrium potential is computed by the thermodynamic layer; this
/// type supplies kinetics and optional mass-transfer ceilings.
#[derive(Debug, Clone, Copy)]
pub struct PartialReaction<'a> {
    pub id: &'a str,
    /// Exact reviewed parameter record, distinct from reaction identity.
    pub parameter_record_id: Option<&'a str>,
    pub equilibrium_potential_v: f64,
    pub kinetics: ButlerVolmerParams,
    /// Reactive area for this reaction divided by geometric electrode area.
    pub reactive_area_ratio: f64,
    /// Area-specific resistance of surface films, Ω·m².
    pub film_resistance_ohm_m2: f64,
    pub limiting_current_anodic_a_per_m2: Option<f64>,
    pub limiting_current_cathodic_a_per_m2: Option<f64>,
}

impl PartialReaction<'_> {
    pub fn current_density(
        &self,
        electrode_potential_v: f64,
        temperature_k: f64,
    ) -> Result<f64, ElectrochemistryError> {
        self.kinetics
            .validate()
            .map_err(ElectrochemistryError::InvalidKinetics)?;
        if !self.equilibrium_potential_v.is_finite()
            || !electrode_potential_v.is_finite()
            || !temperature_k.is_finite()
            || temperature_k <= 0.0
        {
            return Err(ElectrochemistryError::InvalidCondition(
                "potentials and positive temperature must be finite",
            ));
        }
        if !self.reactive_area_ratio.is_finite() || self.reactive_area_ratio < 0.0 {
            return Err(ElectrochemistryError::InvalidSurface(
                "reactive-area ratio must be finite and non-negative",
            ));
        }
        if !self.film_resistance_ohm_m2.is_finite() || self.film_resistance_ohm_m2 < 0.0 {
            return Err(ElectrochemistryError::InvalidSurface(
                "film resistance must be finite and non-negative",
            ));
        }
        let overpotential = electrode_potential_v - self.equilibrium_potential_v;
        let unconstrained = self.kinetics.current_density(overpotential, temperature_k);
        let limit = if unconstrained >= 0.0 {
            self.limiting_current_anodic_a_per_m2
        } else {
            self.limiting_current_cathodic_a_per_m2
        };
        let current_without_film = transport_limited_flux(unconstrained, limit);
        let current = if self.film_resistance_ohm_m2 == 0.0 || current_without_film == 0.0 {
            current_without_film
        } else {
            let current_at = |trial: f64| {
                let kinetic = self.kinetics.current_density(
                    overpotential - trial * self.film_resistance_ohm_m2,
                    temperature_k,
                );
                trial - transport_limited_flux(kinetic, limit)
            };
            let (mut lower, mut upper) = if current_without_film > 0.0 {
                (0.0, current_without_film)
            } else {
                (current_without_film, 0.0)
            };
            let mut f_lower = current_at(lower);
            for _ in 0..160 {
                let middle = 0.5 * (lower + upper);
                let f_middle = current_at(middle);
                if f_middle.abs() <= 1e-12 * (1.0 + middle.abs()) {
                    lower = middle;
                    upper = middle;
                    break;
                }
                if f_middle.signum() == f_lower.signum() {
                    lower = middle;
                    f_lower = f_middle;
                } else {
                    upper = middle;
                }
            }
            0.5 * (lower + upper)
        } * self.reactive_area_ratio;
        if !current.is_finite() {
            return Err(ElectrochemistryError::InvalidCondition(
                "current or transport limit is outside its physical domain",
            ));
        }
        Ok(current)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialCurrent {
    pub reaction_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_record_id: Option<String>,
    /// Signed A/m² referred to geometric electrode area.
    pub current_density_a_per_m2: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentBalance {
    /// Potential at the reacting interface after solution-resistance loss.
    pub electrode_potential_v: f64,
    /// Potential imposed or measured outside the solution resistance.
    pub terminal_potential_v: f64,
    pub partial_currents: Vec<PartialCurrent>,
    pub net_current_density_a_per_m2: f64,
    /// Non-Faradaic charging current. Zero in a steady-state solve.
    #[serde(default)]
    pub capacitive_current_density_a_per_m2: f64,
    /// Current seen at the terminal: Faradaic plus capacitive.
    #[serde(default)]
    pub total_current_density_a_per_m2: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaradaicExtent {
    pub reaction_id: String,
    pub current_amps: f64,
    /// Signed extent: positive selects the anodic direction declared by the
    /// half-reaction; negative selects its cathodic direction.
    pub extent_moles: f64,
}

impl CurrentBalance {
    pub fn extents(
        &self,
        reactions: &[PartialReaction<'_>],
        geometric_area_m2: f64,
        seconds: f64,
    ) -> Result<Vec<FaradaicExtent>, ElectrochemistryError> {
        if reactions.len() != self.partial_currents.len()
            || !geometric_area_m2.is_finite()
            || geometric_area_m2 <= 0.0
        {
            return Err(ElectrochemistryError::InvalidCondition(
                "current balance and reaction list must match on a positive electrode area",
            ));
        }
        self.partial_currents
            .iter()
            .zip(reactions)
            .map(|(partial, reaction)| {
                if partial.reaction_id != reaction.id {
                    return Err(ElectrochemistryError::InvalidCondition(
                        "current balance and reaction order do not match",
                    ));
                }
                let current_amps = partial.current_density_a_per_m2 * geometric_area_m2;
                Ok(FaradaicExtent {
                    reaction_id: partial.reaction_id.clone(),
                    current_amps,
                    extent_moles: faradaic_extent_moles(
                        current_amps,
                        seconds,
                        reaction.kinetics.n,
                    )?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElectrochemistryError {
    InvalidKinetics(&'static str),
    InvalidSurface(&'static str),
    InvalidCondition(&'static str),
    NoPartialReactions,
    NoActivePartialReactions,
    CurrentNotBracketed {
        lower_current_density: f64,
        upper_current_density: f64,
        target_current_density: f64,
    },
    ControlNotBracketed {
        lower_residual_v: f64,
        upper_residual_v: f64,
    },
    DidNotConverge {
        residual: f64,
    },
}

/// Bounded deterministic solver for a mixed potential or imposed current.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrentBalanceSolver {
    pub minimum_potential_v: f64,
    pub maximum_potential_v: f64,
    pub potential_tolerance_v: f64,
    pub current_tolerance_a_per_m2: f64,
    /// Relative cancellation tolerance against the sum of absolute partial
    /// currents. Needed when two large physical currents balance near zero.
    pub relative_current_tolerance: f64,
    pub maximum_iterations: usize,
}

impl Default for CurrentBalanceSolver {
    fn default() -> Self {
        Self {
            minimum_potential_v: -3.0,
            maximum_potential_v: 3.0,
            potential_tolerance_v: 1e-9,
            current_tolerance_a_per_m2: 1e-9,
            relative_current_tolerance: 1e-14,
            maximum_iterations: 160,
        }
    }
}

impl CurrentBalanceSolver {
    fn current_converged(&self, balance: &CurrentBalance, target: f64) -> bool {
        let scale = balance
            .partial_currents
            .iter()
            .map(|partial| partial.current_density_a_per_m2.abs())
            .sum::<f64>()
            .max(target.abs());
        (balance.net_current_density_a_per_m2 - target).abs()
            <= self.current_tolerance_a_per_m2 + self.relative_current_tolerance * scale
    }

    fn currents_at(
        &self,
        reactions: &[PartialReaction<'_>],
        potential_v: f64,
        temperature_k: f64,
    ) -> Result<CurrentBalance, ElectrochemistryError> {
        let mut partial_currents = Vec::with_capacity(reactions.len());
        let mut net = 0.0;
        for reaction in reactions {
            let current = reaction.current_density(potential_v, temperature_k)?;
            net += current;
            partial_currents.push(PartialCurrent {
                reaction_id: reaction.id.to_owned(),
                parameter_record_id: reaction.parameter_record_id.map(str::to_owned),
                current_density_a_per_m2: current,
            });
        }
        Ok(CurrentBalance {
            electrode_potential_v: potential_v,
            terminal_potential_v: potential_v,
            partial_currents,
            net_current_density_a_per_m2: net,
            capacitive_current_density_a_per_m2: 0.0,
            total_current_density_a_per_m2: net,
        })
    }

    /// Solve Σjᵢ(E) = target. A target of zero is a freely corroding mixed
    /// potential: individual anodic and cathodic currents may remain large.
    pub fn solve_for_current_density(
        &self,
        reactions: &[PartialReaction<'_>],
        temperature_k: f64,
        target_current_density: f64,
    ) -> Result<CurrentBalance, ElectrochemistryError> {
        if reactions.is_empty() {
            return Err(ElectrochemistryError::NoPartialReactions);
        }
        if !reactions
            .iter()
            .any(|reaction| reaction.reactive_area_ratio > 0.0)
        {
            return Err(ElectrochemistryError::NoActivePartialReactions);
        }
        if !target_current_density.is_finite()
            || !self.minimum_potential_v.is_finite()
            || !self.maximum_potential_v.is_finite()
            || self.minimum_potential_v >= self.maximum_potential_v
            || !self.current_tolerance_a_per_m2.is_finite()
            || self.current_tolerance_a_per_m2 <= 0.0
            || !self.relative_current_tolerance.is_finite()
            || self.relative_current_tolerance < 0.0
            || self.maximum_iterations == 0
        {
            return Err(ElectrochemistryError::InvalidCondition(
                "current target and ordered potential bounds must be finite",
            ));
        }

        let mut lower = self.minimum_potential_v;
        let mut upper = self.maximum_potential_v;
        let lower_balance = self.currents_at(reactions, lower, temperature_k)?;
        let upper_balance = self.currents_at(reactions, upper, temperature_k)?;
        let mut f_lower = lower_balance.net_current_density_a_per_m2 - target_current_density;
        let f_upper = upper_balance.net_current_density_a_per_m2 - target_current_density;

        if self.current_converged(&lower_balance, target_current_density) {
            return Ok(lower_balance);
        }
        if self.current_converged(&upper_balance, target_current_density) {
            return Ok(upper_balance);
        }
        if f_lower.signum() == f_upper.signum() {
            return Err(ElectrochemistryError::CurrentNotBracketed {
                lower_current_density: lower_balance.net_current_density_a_per_m2,
                upper_current_density: upper_balance.net_current_density_a_per_m2,
                target_current_density,
            });
        }

        for _ in 0..self.maximum_iterations {
            let middle = 0.5 * (lower + upper);
            let balance = self.currents_at(reactions, middle, temperature_k)?;
            let f_middle = balance.net_current_density_a_per_m2 - target_current_density;
            if self.current_converged(&balance, target_current_density) {
                return Ok(balance);
            }
            if middle <= lower || middle >= upper {
                return Err(ElectrochemistryError::DidNotConverge { residual: f_middle });
            }
            if f_middle.signum() == f_lower.signum() {
                lower = middle;
                f_lower = f_middle;
            } else {
                upper = middle;
            }
        }
        let balance = self.currents_at(reactions, 0.5 * (lower + upper), temperature_k)?;
        if self.current_converged(&balance, target_current_density) {
            Ok(balance)
        } else {
            Err(ElectrochemistryError::DidNotConverge {
                residual: balance.net_current_density_a_per_m2 - target_current_density,
            })
        }
    }

    pub fn solve_mixed_potential(
        &self,
        reactions: &[PartialReaction<'_>],
        temperature_k: f64,
    ) -> Result<CurrentBalance, ElectrochemistryError> {
        self.solve_for_current_density(reactions, temperature_k, 0.0)
    }

    /// Backward-Euler double-layer transient for one control interval.
    /// Faradaic partial currents remain separate, so capacitance never creates
    /// reaction extent.
    #[allow(clippy::too_many_arguments)]
    pub fn solve_transient_control(
        &self,
        reactions: &[PartialReaction<'_>],
        temperature_k: f64,
        geometric_area_m2: f64,
        control: CellControl,
        transport: TransportLimits,
        previous_potential_v: f64,
        capacitance_f_per_m2: f64,
        seconds: f64,
    ) -> Result<CurrentBalance, ElectrochemistryError> {
        if reactions.is_empty() {
            return Err(ElectrochemistryError::NoPartialReactions);
        }
        if !previous_potential_v.is_finite()
            || !capacitance_f_per_m2.is_finite()
            || capacitance_f_per_m2 <= 0.0
            || !seconds.is_finite()
            || seconds <= 0.0
            || !geometric_area_m2.is_finite()
            || geometric_area_m2 <= 0.0
            || !transport.solution_resistance_ohm.is_finite()
            || transport.solution_resistance_ohm < 0.0
        {
            return Err(ElectrochemistryError::InvalidCondition(
                "transient capacitance, time, area and resistance must be physical",
            ));
        }
        let target_density = match control {
            CellControl::Galvanostatic { current_amps } if current_amps.is_finite() => {
                Some(current_amps / geometric_area_m2)
            }
            CellControl::Galvanostatic { .. } => {
                return Err(ElectrochemistryError::InvalidCondition(
                    "applied current must be finite",
                ));
            }
            CellControl::OpenCircuit | CellControl::Potentiostatic { .. } => None,
        };
        let residual = |potential: f64| -> Result<(f64, CurrentBalance), ElectrochemistryError> {
            let mut balance = self.currents_at(reactions, potential, temperature_k)?;
            let capacitive = capacitance_f_per_m2 * (potential - previous_potential_v) / seconds;
            let total = balance.net_current_density_a_per_m2 + capacitive;
            balance.capacitive_current_density_a_per_m2 = capacitive;
            balance.total_current_density_a_per_m2 = total;
            let value = match control {
                CellControl::OpenCircuit => total,
                CellControl::Galvanostatic { .. } => total - target_density.unwrap_or(0.0),
                CellControl::Potentiostatic { voltage } if voltage.is_finite() => {
                    potential + total * geometric_area_m2 * transport.solution_resistance_ohm
                        - voltage
                }
                CellControl::Potentiostatic { .. } => {
                    return Err(ElectrochemistryError::InvalidCondition(
                        "applied potential must be finite",
                    ));
                }
            };
            Ok((value, balance))
        };
        let mut lower = self.minimum_potential_v;
        let mut upper = self.maximum_potential_v;
        let (mut f_lower, _) = residual(lower)?;
        let (f_upper, _) = residual(upper)?;
        if f_lower.signum() == f_upper.signum() {
            return Err(ElectrochemistryError::ControlNotBracketed {
                lower_residual_v: f_lower,
                upper_residual_v: f_upper,
            });
        }
        for _ in 0..self.maximum_iterations {
            let middle = 0.5 * (lower + upper);
            let (f_middle, mut balance) = residual(middle)?;
            let converged = match control {
                CellControl::Potentiostatic { .. } => f_middle.abs() <= self.potential_tolerance_v,
                _ => {
                    let scale = balance
                        .partial_currents
                        .iter()
                        .map(|partial| partial.current_density_a_per_m2.abs())
                        .sum::<f64>()
                        + balance.capacitive_current_density_a_per_m2.abs();
                    f_middle.abs()
                        <= self.current_tolerance_a_per_m2 + self.relative_current_tolerance * scale
                }
            };
            if converged {
                let total_current = balance.total_current_density_a_per_m2 * geometric_area_m2;
                balance.terminal_potential_v = match control {
                    CellControl::Potentiostatic { voltage } => voltage,
                    _ => balance.electrode_potential_v + transport.signed_ir_drop(total_current),
                };
                return Ok(balance);
            }
            if f_middle.signum() == f_lower.signum() {
                lower = middle;
                f_lower = f_middle;
            } else {
                upper = middle;
            }
        }
        let (final_residual, _) = residual(0.5 * (lower + upper))?;
        Err(ElectrochemistryError::DidNotConverge {
            residual: final_residual,
        })
    }

    /// Evaluate potentiostatic, galvanostatic or freely corroding operation
    /// using the same partial-current model. Solution resistance is solved
    /// implicitly for potentiostatic control rather than subtracted once.
    pub fn solve_control(
        &self,
        reactions: &[PartialReaction<'_>],
        temperature_k: f64,
        geometric_area_m2: f64,
        control: CellControl,
        transport: TransportLimits,
    ) -> Result<CurrentBalance, ElectrochemistryError> {
        if !geometric_area_m2.is_finite()
            || geometric_area_m2 <= 0.0
            || !transport.solution_resistance_ohm.is_finite()
            || transport.solution_resistance_ohm < 0.0
        {
            return Err(ElectrochemistryError::InvalidCondition(
                "electrode area must be positive and solution resistance non-negative",
            ));
        }
        match control {
            CellControl::OpenCircuit => self.solve_mixed_potential(reactions, temperature_k),
            CellControl::Galvanostatic { current_amps } => {
                let mut balance = self.solve_for_current_density(
                    reactions,
                    temperature_k,
                    current_amps / geometric_area_m2,
                )?;
                balance.terminal_potential_v =
                    balance.electrode_potential_v + transport.signed_ir_drop(current_amps);
                Ok(balance)
            }
            CellControl::Potentiostatic { voltage } => {
                if !voltage.is_finite() {
                    return Err(ElectrochemistryError::InvalidCondition(
                        "applied potential must be finite",
                    ));
                }
                if !self.potential_tolerance_v.is_finite()
                    || self.potential_tolerance_v <= 0.0
                    || self.maximum_iterations == 0
                {
                    return Err(ElectrochemistryError::InvalidCondition(
                        "potential tolerance and iteration count must be positive",
                    ));
                }
                if transport.solution_resistance_ohm == 0.0 {
                    let mut balance = self.currents_at(reactions, voltage, temperature_k)?;
                    balance.terminal_potential_v = voltage;
                    return Ok(balance);
                }
                let residual =
                    |potential: f64| -> Result<(f64, CurrentBalance), ElectrochemistryError> {
                        let balance = self.currents_at(reactions, potential, temperature_k)?;
                        let current_amps = balance.net_current_density_a_per_m2 * geometric_area_m2;
                        Ok((
                            potential + transport.signed_ir_drop(current_amps) - voltage,
                            balance,
                        ))
                    };
                let (mut f_lower, _) = residual(self.minimum_potential_v)?;
                let (f_upper, _) = residual(self.maximum_potential_v)?;
                if f_lower.signum() == f_upper.signum() {
                    return Err(ElectrochemistryError::ControlNotBracketed {
                        lower_residual_v: f_lower,
                        upper_residual_v: f_upper,
                    });
                }
                let mut lower = self.minimum_potential_v;
                let mut upper = self.maximum_potential_v;
                for _ in 0..self.maximum_iterations {
                    let middle = 0.5 * (lower + upper);
                    let (f_middle, mut balance) = residual(middle)?;
                    if f_middle.abs() <= self.potential_tolerance_v
                        || upper - lower <= self.potential_tolerance_v
                    {
                        balance.terminal_potential_v = voltage;
                        return Ok(balance);
                    }
                    if f_middle.signum() == f_lower.signum() {
                        lower = middle;
                        f_lower = f_middle;
                    } else {
                        upper = middle;
                    }
                }
                let (_, mut balance) = residual(0.5 * (lower + upper))?;
                let current_amps = balance.net_current_density_a_per_m2 * geometric_area_m2;
                let final_residual = balance.electrode_potential_v
                    + transport.signed_ir_drop(current_amps)
                    - voltage;
                balance.terminal_potential_v = voltage;
                if final_residual.abs() <= self.potential_tolerance_v {
                    Ok(balance)
                } else {
                    Err(ElectrochemistryError::DidNotConverge {
                        residual: final_residual,
                    })
                }
            }
        }
    }
}

/// Two finite electrodes joined by one external circuit. Positive current is
/// anodic at `left` and cathodic at `right`; unequal areas therefore receive
/// equal and opposite total currents, not equal current densities.
#[derive(Debug, Clone, Copy)]
pub struct ConnectedElectrode<'a> {
    pub reactions: &'a [PartialReaction<'a>],
    pub temperature_k: f64,
    pub geometric_area_m2: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectedCellBalance {
    pub current_amps: f64,
    pub left: CurrentBalance,
    pub right: CurrentBalance,
    /// Right metal potential minus left metal potential after solution iR.
    pub terminal_voltage_v: f64,
    pub solution_resistance_ohm: f64,
}

impl ConnectedCellBalance {
    pub fn irreversible_solution_heat_j(&self, seconds: f64) -> Result<f64, ElectrochemistryError> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(ElectrochemistryError::InvalidCondition(
                "connected-cell duration must be finite and non-negative",
            ));
        }
        Ok(self.current_amps.powi(2) * self.solution_resistance_ohm * seconds)
    }
}

/// Deterministic circuit solver around the same per-electrode current solver.
/// Explicit current bounds are part of the numerical contract: a caller must
/// not silently extrapolate polarization data to manufacture a bracket.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectedCellSolver {
    pub electrode_solver: CurrentBalanceSolver,
    pub minimum_current_amps: f64,
    pub maximum_current_amps: f64,
    pub current_tolerance_amps: f64,
    pub voltage_tolerance_v: f64,
    pub maximum_iterations: usize,
}

impl Default for ConnectedCellSolver {
    fn default() -> Self {
        Self {
            electrode_solver: CurrentBalanceSolver::default(),
            minimum_current_amps: -100.0,
            maximum_current_amps: 100.0,
            current_tolerance_amps: 1e-9,
            voltage_tolerance_v: 1e-9,
            maximum_iterations: 160,
        }
    }
}

impl ConnectedCellSolver {
    fn at_current(
        &self,
        left: ConnectedElectrode<'_>,
        right: ConnectedElectrode<'_>,
        current_amps: f64,
        solution_resistance_ohm: f64,
    ) -> Result<ConnectedCellBalance, ElectrochemistryError> {
        if !left.geometric_area_m2.is_finite()
            || left.geometric_area_m2 <= 0.0
            || !right.geometric_area_m2.is_finite()
            || right.geometric_area_m2 <= 0.0
            || !solution_resistance_ohm.is_finite()
            || solution_resistance_ohm < 0.0
            || !current_amps.is_finite()
        {
            return Err(ElectrochemistryError::InvalidCondition(
                "connected electrodes require physical area, current and resistance",
            ));
        }
        let left_balance = self.electrode_solver.solve_for_current_density(
            left.reactions,
            left.temperature_k,
            current_amps / left.geometric_area_m2,
        )?;
        let right_balance = self.electrode_solver.solve_for_current_density(
            right.reactions,
            right.temperature_k,
            -current_amps / right.geometric_area_m2,
        )?;
        let terminal_voltage = right_balance.electrode_potential_v
            - left_balance.electrode_potential_v
            - current_amps * solution_resistance_ohm;
        Ok(ConnectedCellBalance {
            current_amps,
            left: left_balance,
            right: right_balance,
            terminal_voltage_v: terminal_voltage,
            solution_resistance_ohm,
        })
    }

    pub fn solve(
        &self,
        left: ConnectedElectrode<'_>,
        right: ConnectedElectrode<'_>,
        control: CellControl,
        solution_resistance_ohm: f64,
    ) -> Result<ConnectedCellBalance, ElectrochemistryError> {
        match control {
            CellControl::OpenCircuit => self.at_current(left, right, 0.0, solution_resistance_ohm),
            CellControl::Galvanostatic { current_amps } => {
                self.at_current(left, right, current_amps, solution_resistance_ohm)
            }
            CellControl::Potentiostatic { voltage } => {
                if !voltage.is_finite()
                    || !self.minimum_current_amps.is_finite()
                    || !self.maximum_current_amps.is_finite()
                    || self.minimum_current_amps >= self.maximum_current_amps
                    || !self.current_tolerance_amps.is_finite()
                    || self.current_tolerance_amps <= 0.0
                    || !self.voltage_tolerance_v.is_finite()
                    || self.voltage_tolerance_v <= 0.0
                    || self.maximum_iterations == 0
                {
                    return Err(ElectrochemistryError::InvalidCondition(
                        "connected-cell voltage solve requires physical bounds and tolerances",
                    ));
                }
                let mut lower = self.minimum_current_amps;
                let mut upper = self.maximum_current_amps;
                let lower_balance = self.at_current(left, right, lower, solution_resistance_ohm)?;
                let upper_balance = self.at_current(left, right, upper, solution_resistance_ohm)?;
                let mut f_lower = lower_balance.terminal_voltage_v - voltage;
                let f_upper = upper_balance.terminal_voltage_v - voltage;
                if f_lower.abs() <= self.voltage_tolerance_v {
                    return Ok(lower_balance);
                }
                if f_upper.abs() <= self.voltage_tolerance_v {
                    return Ok(upper_balance);
                }
                if f_lower.signum() == f_upper.signum() {
                    return Err(ElectrochemistryError::ControlNotBracketed {
                        lower_residual_v: f_lower,
                        upper_residual_v: f_upper,
                    });
                }
                for _ in 0..self.maximum_iterations {
                    let middle = 0.5 * (lower + upper);
                    let balance = self.at_current(left, right, middle, solution_resistance_ohm)?;
                    let residual = balance.terminal_voltage_v - voltage;
                    if residual.abs() <= self.voltage_tolerance_v
                        || upper - lower <= self.current_tolerance_amps
                    {
                        return Ok(balance);
                    }
                    if residual.signum() == f_lower.signum() {
                        lower = middle;
                        f_lower = residual;
                    } else {
                        upper = middle;
                    }
                }
                Err(ElectrochemistryError::DidNotConverge {
                    residual: upper - lower,
                })
            }
        }
    }
}

/// Convert a signed partial current into signed reaction extent by Faraday's
/// law. Stoichiometric inventory limiting is deliberately a later, separate
/// operation so every consumer uses the normal conservation ledger.
pub fn faradaic_extent_moles(
    current_amps: f64,
    seconds: f64,
    electrons_per_extent: f64,
) -> Result<f64, ElectrochemistryError> {
    if !current_amps.is_finite()
        || !seconds.is_finite()
        || seconds < 0.0
        || !electrons_per_extent.is_finite()
        || electrons_per_extent <= 0.0
    {
        return Err(ElectrochemistryError::InvalidCondition(
            "current, non-negative time and positive electron count must be finite",
        ));
    }
    Ok(current_amps * seconds / (electrons_per_extent * FARADAY))
}

/// Irreversible Joule heat in solution and characterised surface films.
/// Reversible electrical/chemical work is deliberately not called heat.
pub fn irreversible_ohmic_heat_j(
    balance: &CurrentBalance,
    reactions: &[PartialReaction<'_>],
    geometric_area_m2: f64,
    solution_resistance_ohm: f64,
    seconds: f64,
) -> Result<f64, ElectrochemistryError> {
    if reactions.len() != balance.partial_currents.len()
        || !geometric_area_m2.is_finite()
        || geometric_area_m2 <= 0.0
        || !solution_resistance_ohm.is_finite()
        || solution_resistance_ohm < 0.0
        || !seconds.is_finite()
        || seconds < 0.0
    {
        return Err(ElectrochemistryError::InvalidCondition(
            "ohmic heat requires matching currents and physical area, resistance and time",
        ));
    }
    let terminal_current_a = balance.total_current_density_a_per_m2 * geometric_area_m2;
    let solution_power_w = terminal_current_a * terminal_current_a * solution_resistance_ohm;
    let film_power_w = balance
        .partial_currents
        .iter()
        .zip(reactions)
        .map(|(current, reaction)| {
            if reaction.reactive_area_ratio <= 0.0 {
                0.0
            } else {
                current.current_density_a_per_m2.powi(2) * geometric_area_m2
                    / reaction.reactive_area_ratio
                    * reaction.film_resistance_ohm_m2
            }
        })
        .sum::<f64>();
    let heat = (solution_power_w + film_power_w) * seconds;
    if heat.is_finite() && heat >= 0.0 {
        Ok(heat)
    } else {
        Err(ElectrochemistryError::InvalidCondition(
            "computed ohmic heat is not finite",
        ))
    }
}

/// One species activity in a reduction half-reaction. Coefficients follow the
/// usual reaction quotient convention: products positive, reactants negative;
/// pure solids and liquids are omitted because their activity is one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquilibriumActivity<'a> {
    pub species: &'a str,
    pub coefficient: f64,
    pub activity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivitySource {
    ResolvedAqueous,
    OwnedIdealGas,
    /// Unit fugacity used by a parameterisation explicitly referred to the
    /// standard gas state (for example a standard-state hydrogen-evolution
    /// polarization curve). This is a declared thermodynamic reference, not
    /// an estimate of absent headspace gas and not an arbitrary activity
    /// floor.
    StandardStateGas,
    PurePhase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActivityRequirement<'a> {
    pub species: &'a str,
    pub coefficient: f64,
    pub source: ActivitySource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityResolutionError {
    pub species: String,
    pub source: ActivitySource,
}

/// Resolve a reaction quotient from authoritative vessel state. A missing gas
/// in an open boundary or an unsolved aqueous activity is a typed gap; only a
/// declared pure phase receives unit activity.
pub fn resolve_equilibrium_activities<'a>(
    vessel: &crate::Vessel,
    requirements: &[ActivityRequirement<'a>],
) -> Result<Vec<EquilibriumActivity<'a>>, ActivityResolutionError> {
    requirements
        .iter()
        .map(|requirement| {
            let species_id = crate::SpeciesId::new(requirement.species);
            let activity = match requirement.source {
                ActivitySource::ResolvedAqueous => vessel.resolved_aqueous_activity(&species_id),
                ActivitySource::OwnedIdealGas => vessel.ideal_gas_activity(&species_id),
                ActivitySource::StandardStateGas => crate::species::lookup(&species_id)
                    .filter(|species| species.standard_phase == crate::Phase::Gas)
                    .map(|_| 1.0),
                ActivitySource::PurePhase => crate::species::lookup(&species_id)
                    .filter(|species| {
                        matches!(
                            species.standard_phase,
                            crate::Phase::Solid | crate::Phase::Liquid
                        )
                    })
                    .map(|_| 1.0),
            }
            .ok_or_else(|| ActivityResolutionError {
                species: requirement.species.to_owned(),
                source: requirement.source,
            })?;
            Ok(EquilibriumActivity {
                species: requirement.species,
                coefficient: requirement.coefficient,
                activity,
            })
        })
        .collect()
}

/// Compute the equilibrium potential from a complete activity quotient.
pub fn equilibrium_potential_v(
    standard_reduction_potential_v: f64,
    electrons: f64,
    temperature_k: f64,
    activities: &[EquilibriumActivity<'_>],
) -> Result<f64, ElectrochemistryError> {
    if activities.iter().any(|term| {
        term.species.trim().is_empty()
            || !term.coefficient.is_finite()
            || !term.activity.is_finite()
            || term.activity <= 0.0
    }) {
        return Err(ElectrochemistryError::InvalidCondition(
            "reaction-quotient activities must be named, finite and positive",
        ));
    }
    let ln_q = activities
        .iter()
        .map(|term| term.coefficient * term.activity.ln())
        .sum();
    crate::relations::nernst_reaction_quotient(
        standard_reduction_potential_v,
        electrons,
        ln_q,
        crate::Kelvin(temperature_k),
    )
    .ok_or(ElectrochemistryError::InvalidCondition(
        "standard potential, electron count and temperature must be physical",
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub enum CandidateReactionError {
    Parameters(ParameterSelectionError),
    Equilibrium(ElectrochemistryError),
}

/// Join thermodynamic activities to exactly one valid kinetic record. This is
/// the generic adapter used by acid-metal, corrosion, plating and battery
/// routes; it contains no reaction-specific branching.
#[allow(clippy::too_many_arguments)]
pub fn parameterized_partial_reaction<'a>(
    records: &'a [ExchangeCurrentRecord],
    reaction: &'a str,
    electrode_material: &str,
    standard_reduction_potential_v: f64,
    temperature_k: f64,
    domain_activities: &[(String, f64)],
    surface_preparation: Option<&str>,
    hydrodynamics: HydrodynamicCondition,
    quotient_activities: &[EquilibriumActivity<'_>],
    reactive_area_ratio: f64,
    limiting_current_anodic_a_per_m2: Option<f64>,
    limiting_current_cathodic_a_per_m2: Option<f64>,
) -> Result<PartialReaction<'a>, CandidateReactionError> {
    let record = select_exchange_current(
        records,
        reaction,
        electrode_material,
        temperature_k,
        domain_activities,
        surface_preparation,
        hydrodynamics,
    )
    .map_err(CandidateReactionError::Parameters)?;
    let equilibrium_potential_v = equilibrium_potential_v(
        standard_reduction_potential_v,
        record.kinetics.n,
        temperature_k,
        quotient_activities,
    )
    .map_err(CandidateReactionError::Equilibrium)?;
    Ok(PartialReaction {
        id: reaction,
        parameter_record_id: Some(&record.id),
        equilibrium_potential_v,
        kinetics: record.kinetics,
        reactive_area_ratio,
        limiting_current_anodic_a_per_m2,
        limiting_current_cathodic_a_per_m2,
        film_resistance_ohm_m2: 0.0,
    })
}

/// Matter reservoir changed by an electrochemical half-reaction written in
/// its anodic direction (electrons produced). Negative solved extent applies
/// the reverse, cathodic direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaradaicReservoir<'a> {
    Bulk {
        species: &'a str,
        phase: crate::Phase,
    },
    ElectrodeSubstrate {
        electrode: &'a str,
    },
    ElectrodeDeposit {
        electrode: &'a str,
        species: &'a str,
        growth: Option<crate::compartment::DepositGrowthModel>,
        effect: Option<PassivationEffect>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaradaicTerm<'a> {
    pub reservoir: FaradaicReservoir<'a>,
    /// Moles created per mole of anodic extent; negative consumes.
    pub coefficient: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaradaicHalfReaction<'a> {
    pub id: &'a str,
    pub electrons_produced: f64,
    pub terms: &'a [FaradaicTerm<'a>],
}

/// A complete, data-driven electrode reaction available to the runtime.
///
/// Quotient inputs determine the reversible potential. Kinetic-domain inputs
/// select a measured parameter record and are deliberately separate: surface
/// poisons, catalysts and supporting electrolyte can constrain a fit without
/// belonging in the balanced half-reaction quotient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectrochemicalReactionDefinition<'a> {
    pub id: &'a str,
    pub standard_reduction_potential_v: f64,
    pub quotient_requirements: &'a [ActivityRequirement<'a>],
    pub kinetic_domain_requirements: &'a [ActivityRequirement<'a>],
    /// How this reaction obtains the currently available area.
    pub surface_availability: SurfaceAvailabilityModel,
    /// Mass-transfer ceilings per real reactive area, before the surface-area
    /// multiplier is applied.
    pub anodic_transport: Option<CurrentLimitModel>,
    pub cathodic_transport: Option<CurrentLimitModel>,
    pub anodic_transported_species: Option<TransportedSpecies<'a>>,
    pub cathodic_transported_species: Option<TransportedSpecies<'a>>,
    pub film_resistance: FilmResistanceModel,
    /// Atom-balanced matter bookkeeping in the anodic direction.
    pub anodic_terms: &'a [FaradaicTerm<'a>],
    pub electrons_produced: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportedSpecies<'a> {
    pub species: &'a str,
    /// Positive reactant moles consumed per mole of reaction extent.
    pub moles_per_extent: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfacialCondition {
    pub reaction_id: String,
    pub species: String,
    pub bulk_concentration_mol_per_m3: f64,
    pub surface_concentration_mol_per_m3: f64,
    pub bulk_activity: f64,
    pub surface_activity: f64,
    pub depleted_at_surface: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_ph: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "model", rename_all = "snake_case")]
pub enum FilmResistanceModel {
    None,
    Measured { ohm_m2: f64 },
    FromDeposits,
}

impl FilmResistanceModel {
    fn resolve(self, electrode: &crate::ElectrodeState) -> Result<f64, &'static str> {
        match self {
            Self::None => Ok(0.0),
            Self::Measured { ohm_m2 } if ohm_m2.is_finite() && ohm_m2 >= 0.0 => Ok(ohm_m2),
            Self::Measured { .. } => {
                Err("measured film resistance must be finite and non-negative")
            }
            Self::FromDeposits => electrode.deposit_film_resistance_ohm_m2(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "model", rename_all = "snake_case")]
pub enum SurfaceAvailabilityModel {
    /// Reaction-specific measured or modelled availability.
    Explicit { fraction: f64 },
    /// Conservative availability computed from characterised deposit layers.
    FromDeposits,
}

impl SurfaceAvailabilityModel {
    fn resolve(self, electrode: &crate::ElectrodeState) -> Result<f64, &'static str> {
        match self {
            Self::Explicit { fraction }
                if fraction.is_finite() && (0.0..=1.0).contains(&fraction) =>
            {
                Ok(fraction)
            }
            Self::Explicit { .. } => {
                Err("explicit surface availability must be within zero and one")
            }
            Self::FromDeposits => electrode.deposit_available_fraction(),
        }
    }
}

/// How a partial reaction obtains its transport ceiling. A directly measured
/// ceiling remains representable, while common geometries compute it from
/// physical inputs instead of scripting a result.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "model", rename_all = "snake_case")]
pub enum CurrentLimitModel {
    MeasuredCurrentDensity { amperes_per_m2: f64 },
    DiffusionLayer(crate::heterogeneous::DiffusionLayerTransport),
    RotatingDisk(crate::heterogeneous::RotatingDiskTransport),
}

impl CurrentLimitModel {
    pub fn current_density_limit(
        self,
        electrons_per_extent: f64,
        transported_moles_per_extent: f64,
    ) -> Result<f64, crate::heterogeneous::SurfaceRateError> {
        if !transported_moles_per_extent.is_finite() || transported_moles_per_extent <= 0.0 {
            return Err(crate::heterogeneous::SurfaceRateError::InvalidTransport);
        }
        match self {
            Self::MeasuredCurrentDensity { amperes_per_m2 }
                if amperes_per_m2.is_finite() && amperes_per_m2 > 0.0 =>
            {
                Ok(amperes_per_m2)
            }
            Self::MeasuredCurrentDensity { .. } => {
                Err(crate::heterogeneous::SurfaceRateError::InvalidTransport)
            }
            Self::DiffusionLayer(model) => {
                model.current_density_limit(electrons_per_extent / transported_moles_per_extent)
            }
            Self::RotatingDisk(model) => {
                model.current_density_limit(electrons_per_extent / transported_moles_per_extent)
            }
        }
    }

    fn surface_concentration(
        self,
        current_density_per_real_area: f64,
        electrons_per_extent: f64,
        moles_per_extent: f64,
    ) -> Result<(f64, f64), crate::heterogeneous::SurfaceRateError> {
        if !current_density_per_real_area.is_finite()
            || !electrons_per_extent.is_finite()
            || electrons_per_extent <= 0.0
            || !moles_per_extent.is_finite()
            || moles_per_extent <= 0.0
        {
            return Err(crate::heterogeneous::SurfaceRateError::InvalidTransport);
        }
        let consumed_flux = current_density_per_real_area.abs() / (electrons_per_extent * FARADAY)
            * moles_per_extent;
        match self {
            Self::DiffusionLayer(model) => Ok((
                model.bulk_concentration_mol_per_m3,
                model.surface_concentration(consumed_flux)?,
            )),
            Self::RotatingDisk(model) => {
                let limit = model.current_density_limit(electrons_per_extent / moles_per_extent)?;
                let fraction = (1.0 - current_density_per_real_area.abs() / limit).clamp(0.0, 1.0);
                Ok((
                    model.bulk_concentration_mol_per_m3,
                    model.bulk_concentration_mol_per_m3 * fraction,
                ))
            }
            Self::MeasuredCurrentDensity { .. } => {
                Err(crate::heterogeneous::SurfaceRateError::InvalidTransport)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElectrochemicalStepProposal {
    pub balance: CurrentBalance,
    pub interfacial_conditions: Vec<InterfacialCondition>,
    /// Unbounded Faraday-law proposal, retained for diagnostics.
    pub requested_delta: crate::delta::StateDelta,
    /// Uniformly inventory-limited proposal safe to conservation-check and
    /// commit atomically.
    pub accepted_delta: crate::delta::StateDelta,
    pub accepted_fraction: f64,
}

#[derive(Debug, Clone)]
pub struct ElectrochemicalAdvanceSegment {
    pub seconds: f64,
    pub balance: CurrentBalance,
    pub interfacial_conditions: Vec<InterfacialCondition>,
    pub delta: crate::delta::StateDelta,
    pub inventory_limited: bool,
}

#[derive(Debug, Clone)]
pub struct ElectrochemicalAdvanceReport {
    pub requested_seconds: f64,
    pub elapsed_seconds: f64,
    pub segments: Vec<ElectrochemicalAdvanceSegment>,
    /// A boundary after at least one committed segment. The caller receives
    /// the physically valid prefix and an exact reason the remainder cannot be
    /// quantified. An error before any segment is still returned as `Err`.
    pub boundary: Option<ElectrochemicalStepError>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectrochemicalAdvanceOptions {
    pub conservation_tolerance: f64,
    pub maximum_depletion_substeps: usize,
    pub minimum_substep_seconds: f64,
}

impl Default for ElectrochemicalAdvanceOptions {
    fn default() -> Self {
        Self {
            conservation_tolerance: 1e-10,
            maximum_depletion_substeps: 32,
            minimum_substep_seconds: 1e-12,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElectrochemicalStepError {
    InvalidCondition {
        reason: String,
    },
    InvalidDefinition {
        reaction: String,
        reason: String,
    },
    ElectrodeNotFound {
        electrode: String,
    },
    AmbiguousElectrode {
        electrode: String,
    },
    InvalidElectrode {
        electrode: String,
        reason: String,
    },
    Activity {
        reaction: String,
        error: ActivityResolutionError,
    },
    Candidate {
        reaction: String,
        error: CandidateReactionError,
    },
    Balance(ElectrochemistryError),
    InterfacialDidNotConverge {
        maximum_potential_change_v: f64,
    },
    VanishedInterfacialActivity {
        reaction: String,
        species: String,
    },
    Inventory(Vec<crate::delta::DeltaError>),
    Commit(Vec<crate::delta::DeltaError>),
    DepletionDidNotAdvance,
    TooManyDepletionSubsteps,
}

/// Assemble and solve any set of competing electrode reactions against one
/// vessel snapshot. The function is intentionally non-mutating: a clock owner
/// may inspect depletion, shorten the interval, re-equilibrate, and then commit
/// `accepted_delta` exactly once.
#[allow(clippy::too_many_arguments)]
pub fn propose_electrochemical_step<'a>(
    vessel: &crate::Vessel,
    electrode_label: &str,
    records: &'a [ExchangeCurrentRecord],
    definitions: &'a [ElectrochemicalReactionDefinition<'a>],
    seconds: f64,
    hydrodynamics: HydrodynamicCondition,
    control: CellControl,
    transport: TransportLimits,
    solver: CurrentBalanceSolver,
) -> Result<ElectrochemicalStepProposal, ElectrochemicalStepError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(ElectrochemicalStepError::InvalidCondition {
            reason: "duration must be finite and non-negative".into(),
        });
    }
    let temperature_k = vessel.temperature.0;
    if !temperature_k.is_finite() || temperature_k <= 0.0 {
        return Err(ElectrochemicalStepError::InvalidCondition {
            reason: "vessel temperature must be finite and positive".into(),
        });
    }
    let matching: Vec<_> = vessel
        .electrodes
        .iter()
        .filter(|electrode| electrode.label == electrode_label)
        .collect();
    let electrode = match matching.as_slice() {
        [electrode] => *electrode,
        [] => {
            return Err(ElectrochemicalStepError::ElectrodeNotFound {
                electrode: electrode_label.to_owned(),
            });
        }
        _ => {
            return Err(ElectrochemicalStepError::AmbiguousElectrode {
                electrode: electrode_label.to_owned(),
            });
        }
    };
    electrode
        .validate()
        .map_err(|reason| ElectrochemicalStepError::InvalidElectrode {
            electrode: electrode_label.to_owned(),
            reason: reason.to_owned(),
        })?;

    let mut partial_reactions = Vec::with_capacity(definitions.len());
    let mut half_reactions = Vec::with_capacity(definitions.len());
    let mut resolved_quotients = Vec::with_capacity(definitions.len());
    let mut reaction_ids = std::collections::BTreeSet::new();
    for definition in definitions {
        if definition.id.trim().is_empty()
            || !reaction_ids.insert(definition.id)
            || !definition.electrons_produced.is_finite()
            || definition.electrons_produced <= 0.0
            || definition.anodic_terms.is_empty()
            || definition.anodic_transported_species.is_some_and(|term| {
                term.species.trim().is_empty()
                    || !term.moles_per_extent.is_finite()
                    || term.moles_per_extent <= 0.0
            })
            || definition.cathodic_transported_species.is_some_and(|term| {
                term.species.trim().is_empty()
                    || !term.moles_per_extent.is_finite()
                    || term.moles_per_extent <= 0.0
            })
            || definition.anodic_transport.is_some()
                != definition.anodic_transported_species.is_some()
            || definition.cathodic_transport.is_some()
                != definition.cathodic_transported_species.is_some()
        {
            return Err(ElectrochemicalStepError::InvalidDefinition {
                reaction: definition.id.to_owned(),
                reason: "reaction ids must be unique and named; electron counts, terms and optional transport limits must be physical".into(),
            });
        }
        let quotient = resolve_equilibrium_activities(vessel, definition.quotient_requirements)
            .map_err(|error| ElectrochemicalStepError::Activity {
                reaction: definition.id.to_owned(),
                error,
            })?;
        let domain = resolve_equilibrium_activities(vessel, definition.kinetic_domain_requirements)
            .map_err(|error| ElectrochemicalStepError::Activity {
                reaction: definition.id.to_owned(),
                error,
            })?;
        let domain: Vec<_> = domain
            .iter()
            .map(|term| (term.species.to_owned(), term.activity))
            .collect();
        let available_fraction =
            definition
                .surface_availability
                .resolve(electrode)
                .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                    reaction: definition.id.to_owned(),
                    reason: reason.to_owned(),
                })?;
        let surface = electrode.reactive_surface(available_fraction);
        surface
            .validate()
            .map_err(|reason| ElectrochemicalStepError::InvalidElectrode {
                electrode: electrode_label.to_owned(),
                reason: format!("{reason:?}"),
            })?;
        let reactive_area_m2 = surface.reactive_area_m2().map_err(|reason| {
            ElectrochemicalStepError::InvalidElectrode {
                electrode: electrode_label.to_owned(),
                reason: format!("{reason:?}"),
            }
        })?;
        let anodic_limit = definition
            .anodic_transport
            .zip(definition.anodic_transported_species)
            .map(|(model, transported)| {
                model.current_density_limit(
                    definition.electrons_produced,
                    transported.moles_per_extent,
                )
            })
            .transpose()
            .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                reaction: definition.id.to_owned(),
                reason: format!("invalid anodic transport model: {reason:?}"),
            })?;
        let cathodic_limit = definition
            .cathodic_transport
            .zip(definition.cathodic_transported_species)
            .map(|(model, transported)| {
                model.current_density_limit(
                    definition.electrons_produced,
                    transported.moles_per_extent,
                )
            })
            .transpose()
            .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                reaction: definition.id.to_owned(),
                reason: format!("invalid cathodic transport model: {reason:?}"),
            })?;
        let film_resistance = definition
            .film_resistance
            .resolve(electrode)
            .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                reaction: definition.id.to_owned(),
                reason: reason.to_owned(),
            })?;
        let mut partial = parameterized_partial_reaction(
            records,
            definition.id,
            &electrode.material,
            definition.standard_reduction_potential_v,
            temperature_k,
            &domain,
            electrode.surface_preparation.as_deref(),
            hydrodynamics,
            &quotient,
            reactive_area_m2 / surface.geometric_area_m2,
            anodic_limit,
            cathodic_limit,
        )
        .map_err(|error| ElectrochemicalStepError::Candidate {
            reaction: definition.id.to_owned(),
            error,
        })?;
        partial.film_resistance_ohm_m2 = film_resistance;
        partial_reactions.push(partial);
        resolved_quotients.push(quotient);
        half_reactions.push(FaradaicHalfReaction {
            id: definition.id,
            electrons_produced: definition.electrons_produced,
            terms: definition.anodic_terms,
        });
    }

    let transient = electrode
        .double_layer_capacitance_f_per_m2
        .zip(electrode.interfacial_potential_v);
    let interfacial_conditions = |balance: &CurrentBalance, reactions: &[PartialReaction<'_>]| {
        balance
            .partial_currents
            .iter()
            .zip(reactions)
            .zip(definitions)
            .zip(&resolved_quotients)
            .filter_map(|(((current, partial), definition), quotient)| {
                if partial.reactive_area_ratio <= 0.0 {
                    return None;
                }
                let (model, transported) = if current.current_density_a_per_m2 >= 0.0 {
                    (
                        definition.anodic_transport,
                        definition.anodic_transported_species,
                    )
                } else {
                    (
                        definition.cathodic_transport,
                        definition.cathodic_transported_species,
                    )
                };
                let model = model?;
                let transported = transported?;
                let bulk_activity = quotient
                    .iter()
                    .find(|term| term.species == transported.species)
                    .map(|term| term.activity);
                Some((current, partial, model, transported, bulk_activity))
            })
            .map(|(current, partial, model, transported, bulk_activity)| {
                let bulk_activity =
                    bulk_activity.ok_or_else(|| ElectrochemicalStepError::InvalidDefinition {
                        reaction: partial.id.to_owned(),
                        reason: format!(
                            "transported species {} is absent from the equilibrium quotient",
                            transported.species
                        ),
                    })?;
                let (bulk_concentration, surface_concentration) = model
                    .surface_concentration(
                        current.current_density_a_per_m2 / partial.reactive_area_ratio,
                        partial.kinetics.n,
                        transported.moles_per_extent,
                    )
                    .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                        reaction: partial.id.to_owned(),
                        reason: format!("cannot resolve interfacial transport: {reason:?}"),
                    })?;
                let surface_activity = if bulk_concentration > 0.0 {
                    bulk_activity * surface_concentration / bulk_concentration
                } else {
                    0.0
                };
                Ok(InterfacialCondition {
                    reaction_id: partial.id.to_owned(),
                    species: transported.species.to_owned(),
                    bulk_concentration_mol_per_m3: bulk_concentration,
                    surface_concentration_mol_per_m3: surface_concentration,
                    bulk_activity,
                    surface_activity,
                    depleted_at_surface: surface_concentration <= f64::EPSILON * bulk_concentration,
                    surface_ph: (transported.species == "H+" && surface_activity > 0.0)
                        .then(|| -surface_activity.log10()),
                })
            })
            .collect::<Result<Vec<_>, _>>()
    };
    let solve_balance_once = |duration: f64, reactions: &[PartialReaction<'_>]| {
        match transient {
            Some((capacitance, previous_potential)) if duration > 0.0 => solver
                .solve_transient_control(
                    reactions,
                    temperature_k,
                    electrode.area_m2,
                    control,
                    transport,
                    previous_potential,
                    capacitance,
                    duration,
                ),
            Some(_) => Err(ElectrochemistryError::InvalidCondition(
                "a capacitive electrode requires positive elapsed time",
            )),
            None => solver.solve_control(
                reactions,
                temperature_k,
                electrode.area_m2,
                control,
                transport,
            ),
        }
        .map_err(ElectrochemicalStepError::Balance)
    };
    // Couple mass transfer back into the reversible potential.  The bulk
    // quotient seeds the iteration; each solved current gives a surface
    // concentration, which updates that reaction's quotient and hence its
    // Nernst potential. Under-relaxation makes strongly transport-limited
    // multi-reaction systems converge without reaction-specific tuning.
    let solve_interfacial = |duration: f64| {
        let mut reactions = partial_reactions.clone();
        let mut last_change = f64::INFINITY;
        for _ in 0..128 {
            let balance = solve_balance_once(duration, &reactions)?;
            let conditions = interfacial_conditions(&balance, &reactions)?;
            let mut targets = reactions.clone();
            last_change = 0.0_f64;
            for condition in &conditions {
                if condition.surface_activity <= 0.0 {
                    return Err(ElectrochemicalStepError::VanishedInterfacialActivity {
                        reaction: condition.reaction_id.clone(),
                        species: condition.species.clone(),
                    });
                }
                let index = definitions
                    .iter()
                    .position(|definition| definition.id == condition.reaction_id)
                    .expect("validated reaction ids remain aligned");
                let mut local_quotient = resolved_quotients[index].clone();
                let local_term = local_quotient
                    .iter_mut()
                    .find(|term| term.species == condition.species)
                    .expect("transported species was validated against quotient");
                local_term.activity = condition.surface_activity;
                let target = equilibrium_potential_v(
                    definitions[index].standard_reduction_potential_v,
                    reactions[index].kinetics.n,
                    temperature_k,
                    &local_quotient,
                )
                .map_err(ElectrochemicalStepError::Balance)?;
                let relaxed = 0.5 * (reactions[index].equilibrium_potential_v + target);
                last_change =
                    last_change.max((relaxed - reactions[index].equilibrium_potential_v).abs());
                targets[index].equilibrium_potential_v = relaxed;
            }
            if last_change <= solver.potential_tolerance_v.max(1e-10) {
                return Ok((balance, conditions, reactions));
            }
            reactions = targets;
        }
        Err(ElectrochemicalStepError::InterfacialDidNotConverge {
            maximum_potential_change_v: last_change,
        })
    };
    let (initial_balance, initial_interfaces, initial_partials) = solve_interfacial(seconds)?;
    let mut initial_matter_delta = faradaic_state_delta(
        &initial_balance,
        &initial_partials,
        &half_reactions,
        electrode.area_m2,
        seconds,
    )
    .map_err(ElectrochemicalStepError::Balance)?;
    let initial_heat = irreversible_ohmic_heat_j(
        &initial_balance,
        &initial_partials,
        electrode.area_m2,
        transport.solution_resistance_ohm,
        seconds,
    )
    .map_err(ElectrochemicalStepError::Balance)?;
    if initial_heat > 0.0 {
        initial_matter_delta = initial_matter_delta.with_thermal(
            crate::delta::ThermalDelta::AddEnergy(crate::Joules(initial_heat)),
        );
    }
    let requested_delta = if transient.is_some() {
        initial_matter_delta
            .clone()
            .with_electrode_potential(electrode_label, initial_balance.electrode_potential_v)
    } else {
        initial_matter_delta.clone()
    };

    if transient.is_none() {
        let limited = initial_matter_delta
            .inventory_limited(vessel)
            .map_err(ElectrochemicalStepError::Inventory)?;
        return Ok(ElectrochemicalStepProposal {
            balance: initial_balance,
            interfacial_conditions: initial_interfaces,
            requested_delta,
            accepted_delta: limited.delta,
            accepted_fraction: limited.accepted_fraction,
        });
    }

    // A capacitive current depends on the interval, so a depleted proposal
    // cannot be linearly scaled. Shorten the interval and solve it again until
    // the transient and inventory boundary agree.
    let mut accepted_seconds = seconds;
    let mut balance = initial_balance;
    let mut interfaces = initial_interfaces;
    let mut solved_partials = initial_partials;
    for _ in 0..32 {
        let mut matter_delta = faradaic_state_delta(
            &balance,
            &solved_partials,
            &half_reactions,
            electrode.area_m2,
            accepted_seconds,
        )
        .map_err(ElectrochemicalStepError::Balance)?;
        let heat = irreversible_ohmic_heat_j(
            &balance,
            &solved_partials,
            electrode.area_m2,
            transport.solution_resistance_ohm,
            accepted_seconds,
        )
        .map_err(ElectrochemicalStepError::Balance)?;
        if heat > 0.0 {
            matter_delta = matter_delta
                .with_thermal(crate::delta::ThermalDelta::AddEnergy(crate::Joules(heat)));
        }
        let limited = matter_delta
            .inventory_limited(vessel)
            .map_err(ElectrochemicalStepError::Inventory)?;
        if limited.accepted_fraction >= 1.0 - 1e-12 {
            return Ok(ElectrochemicalStepProposal {
                interfacial_conditions: interfaces,
                balance: balance.clone(),
                requested_delta,
                accepted_delta: limited
                    .delta
                    .with_electrode_potential(electrode_label, balance.electrode_potential_v),
                accepted_fraction: accepted_seconds / seconds,
            });
        }
        accepted_seconds *= limited.accepted_fraction;
        if accepted_seconds <= seconds.max(1.0) * f64::EPSILON {
            return Err(ElectrochemicalStepError::DepletionDidNotAdvance);
        }
        (balance, interfaces, solved_partials) = solve_interfacial(accepted_seconds)?;
    }
    Err(ElectrochemicalStepError::TooManyDepletionSubsteps)
}

/// Advance a data-driven electrode network through depletion boundaries.
///
/// `equilibrate` owns fast speciation between Faradaic segments. This function
/// owns only electrode kinetics and commits every competing half-reaction as
/// one conserved delta. It is therefore suitable for a clock adapter that
/// excludes legacy displacement/corrosion ownership for the same vessel.
#[allow(clippy::too_many_arguments)]
pub fn advance_electrochemical<'a>(
    vessel: &mut crate::Vessel,
    electrode_label: &str,
    records: &'a [ExchangeCurrentRecord],
    definitions: &'a [ElectrochemicalReactionDefinition<'a>],
    seconds: f64,
    hydrodynamics: HydrodynamicCondition,
    control: CellControl,
    transport: TransportLimits,
    solver: CurrentBalanceSolver,
    options: ElectrochemicalAdvanceOptions,
    equilibrate: &mut dyn FnMut(&mut crate::Vessel),
) -> Result<ElectrochemicalAdvanceReport, ElectrochemicalStepError> {
    if !options.conservation_tolerance.is_finite()
        || options.conservation_tolerance < 0.0
        || !options.minimum_substep_seconds.is_finite()
        || options.minimum_substep_seconds <= 0.0
        || options.maximum_depletion_substeps == 0
    {
        return Err(ElectrochemicalStepError::InvalidCondition {
            reason: "advance tolerances and substep count must be physical".into(),
        });
    }
    if seconds == 0.0 {
        return Ok(ElectrochemicalAdvanceReport {
            requested_seconds: seconds,
            elapsed_seconds: 0.0,
            segments: Vec::new(),
            boundary: None,
        });
    }

    let mut elapsed = 0.0;
    let mut segments = Vec::new();
    for _ in 0..options.maximum_depletion_substeps {
        let remaining = seconds - elapsed;
        if remaining <= seconds.max(1.0) * f64::EPSILON {
            return Ok(ElectrochemicalAdvanceReport {
                requested_seconds: seconds,
                elapsed_seconds: elapsed.min(seconds),
                segments,
                boundary: None,
            });
        }
        let proposal = match propose_electrochemical_step(
            vessel,
            electrode_label,
            records,
            definitions,
            remaining,
            hydrodynamics,
            control,
            transport,
            solver,
        ) {
            Ok(proposal) => proposal,
            Err(error) if !segments.is_empty() => {
                return Ok(ElectrochemicalAdvanceReport {
                    requested_seconds: seconds,
                    elapsed_seconds: elapsed,
                    segments,
                    boundary: Some(error),
                });
            }
            Err(error) => return Err(error),
        };
        let segment_seconds = remaining * proposal.accepted_fraction;
        if !segment_seconds.is_finite() || segment_seconds < options.minimum_substep_seconds {
            let error = ElectrochemicalStepError::DepletionDidNotAdvance;
            if segments.is_empty() {
                return Err(error);
            }
            return Ok(ElectrochemicalAdvanceReport {
                requested_seconds: seconds,
                elapsed_seconds: elapsed,
                segments,
                boundary: Some(error),
            });
        }
        proposal
            .accepted_delta
            .commit_conserved(vessel, options.conservation_tolerance)
            .map_err(ElectrochemicalStepError::Commit)?;
        let inventory_limited = proposal.accepted_fraction < 1.0;
        segments.push(ElectrochemicalAdvanceSegment {
            seconds: segment_seconds,
            balance: proposal.balance,
            interfacial_conditions: proposal.interfacial_conditions,
            delta: proposal.accepted_delta,
            inventory_limited,
        });
        elapsed += segment_seconds;
        equilibrate(vessel);
        if !inventory_limited {
            return Ok(ElectrochemicalAdvanceReport {
                requested_seconds: seconds,
                elapsed_seconds: elapsed.min(seconds),
                segments,
                boundary: None,
            });
        }
    }
    Ok(ElectrochemicalAdvanceReport {
        requested_seconds: seconds,
        elapsed_seconds: elapsed,
        segments,
        boundary: Some(ElectrochemicalStepError::TooManyDepletionSubsteps),
    })
}

/// Convert a solved current balance into one atomic state proposal. No vessel
/// is mutated here; inventory limits and element conservation are enforced by
/// `StateDelta::commit_conserved` after all coupled half-reactions are present.
pub fn faradaic_state_delta(
    balance: &CurrentBalance,
    partial_reactions: &[PartialReaction<'_>],
    half_reactions: &[FaradaicHalfReaction<'_>],
    geometric_area_m2: f64,
    seconds: f64,
) -> Result<crate::delta::StateDelta, ElectrochemistryError> {
    if partial_reactions.len() != half_reactions.len()
        || partial_reactions
            .iter()
            .zip(half_reactions)
            .any(|(partial, half)| {
                partial.id != half.id
                    || !half.electrons_produced.is_finite()
                    || half.electrons_produced <= 0.0
                    || (partial.kinetics.n - half.electrons_produced).abs() > 1e-12
                    || half.terms.iter().any(|term| !term.coefficient.is_finite())
            })
    {
        return Err(ElectrochemistryError::InvalidCondition(
            "partial currents and balanced anodic half-reactions must match",
        ));
    }
    let extents = balance.extents(partial_reactions, geometric_area_m2, seconds)?;
    let mut delta = crate::delta::StateDelta::new("electrochemical current balance");
    for (extent, half) in extents.iter().zip(half_reactions) {
        for term in half.terms {
            let moles = term.coefficient * extent.extent_moles;
            delta = match term.reservoir {
                FaradaicReservoir::Bulk { species, phase } => {
                    delta.with_moles(crate::SpeciesId::new(species), phase, moles)
                }
                FaradaicReservoir::ElectrodeSubstrate { electrode } => delta.with_electrode_moles(
                    electrode,
                    crate::delta::ElectrodeInventory::Substrate,
                    moles,
                ),
                FaradaicReservoir::ElectrodeDeposit {
                    electrode,
                    species,
                    growth,
                    effect,
                } => delta.with_electrode_moles(
                    electrode,
                    crate::delta::ElectrodeInventory::Deposit {
                        species: crate::SpeciesId::new(species),
                        growth,
                        effect,
                    },
                    moles,
                ),
            };
        }
    }
    Ok(delta)
}

// ── ELEC-007: Competing reactions ─────────────────────────────────

/// Outcome of thermodynamic/kinetic competition at an electrode.
#[derive(Debug, Clone, PartialEq)]
pub enum ReactionOutcome {
    /// The reaction proceeds with a quantitative efficiency.
    Proceeds { efficiency: f64 },
    /// Gas evolution competes.
    GasEvolution { gas: &'static str, fraction: f64 },
    /// Insufficient kinetic data to make a quantitative claim.
    Unquantifiable { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn galvanostatic_round_trips() {
        let ctrl = CellControl::Galvanostatic { current_amps: 0.5 };
        let json = serde_json::to_string(&ctrl).unwrap();
        let loaded: CellControl = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, ctrl);
    }

    #[test]
    fn ir_drop_scales_linearly() {
        let limits = TransportLimits {
            solution_resistance_ohm: 10.0,
            limiting_current_cathodic: None,
            limiting_current_anodic: None,
        };
        assert!((limits.ir_drop(0.5) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn passivating_surface_reduces_active_area() {
        let cov = SurfaceCoverage {
            theta: 0.8,
            effect: PassivationEffect::Passivating,
        };
        assert!((cov.active_fraction() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn conductive_deposit_leaves_full_area() {
        let cov = SurfaceCoverage {
            theta: 0.9,
            effect: PassivationEffect::Conductive,
        };
        assert!((cov.active_fraction() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn butler_volmer_zero_at_equilibrium() {
        let j = butler_volmer(0.01, 0.5, 0.5, 0.0, 298.15);
        assert!(j.abs() < 1e-15);
    }

    #[test]
    fn butler_volmer_anodic_positive() {
        let j = butler_volmer(0.01, 0.5, 0.5, 0.1, 298.15);
        assert!(j > 0.0);
    }

    #[test]
    fn butler_volmer_cathodic_negative() {
        let j = butler_volmer(0.01, 0.5, 0.5, -0.1, 298.15);
        assert!(j < 0.0);
    }

    #[test]
    fn tafel_agrees_with_bv_at_high_eta() {
        let j0 = 0.001;
        let alpha = 0.5;
        let t = 298.15;
        let eta = 0.3;
        let j_bv = butler_volmer(j0, alpha, alpha, eta, t);
        let eta_tafel = tafel_overpotential(j_bv, j0, alpha, t);
        assert!((eta - eta_tafel).abs() < 0.01);
    }

    #[test]
    fn limiting_current_positive() {
        let j_l = limiting_current_density(2.0, 1e-5, 1e-6, 0.05);
        assert!(j_l > 0.0);
    }

    fn reaction(id: &'static str, equilibrium_potential_v: f64) -> PartialReaction<'static> {
        PartialReaction {
            id,
            parameter_record_id: None,
            equilibrium_potential_v,
            kinetics: ButlerVolmerParams {
                j0: 1.0,
                alpha_a: 0.5,
                alpha_c: 0.5,
                n: 1.0,
            },
            reactive_area_ratio: 1.0,
            film_resistance_ohm_m2: 0.0,
            limiting_current_anodic_a_per_m2: None,
            limiting_current_cathodic_a_per_m2: None,
        }
    }

    #[test]
    fn open_circuit_balances_nonzero_partial_currents() {
        let reactions = [reaction("metal", -1.0), reaction("hydrogen", 0.0)];
        let balance = CurrentBalanceSolver::default()
            .solve_mixed_potential(&reactions, 298.15)
            .unwrap();
        assert!((balance.electrode_potential_v + 0.5).abs() < 1e-8);
        assert!(balance.net_current_density_a_per_m2.abs() < 1e-8);
        assert!(balance.partial_currents[0].current_density_a_per_m2 > 0.0);
        assert!(balance.partial_currents[1].current_density_a_per_m2 < 0.0);
        assert!(balance.partial_currents[0].current_density_a_per_m2.abs() > 1.0);
    }

    #[test]
    fn area_ratio_changes_the_mixed_potential() {
        let anodic = reaction("metal", -1.0);
        let mut cathodic = reaction("hydrogen", 0.0);
        cathodic.reactive_area_ratio = 10.0;
        let balance = CurrentBalanceSolver::default()
            .solve_mixed_potential(&[anodic, cathodic], 298.15)
            .unwrap();
        assert!(balance.electrode_potential_v > -0.5);
    }

    #[test]
    fn connected_cell_enforces_one_current_across_unequal_areas() {
        let left_reactions = [reaction("left-metal", -0.76)];
        let right_reactions = [reaction("right-metal", 0.34)];
        let left = ConnectedElectrode {
            reactions: &left_reactions,
            temperature_k: 298.15,
            geometric_area_m2: 0.01,
        };
        let right = ConnectedElectrode {
            reactions: &right_reactions,
            temperature_k: 298.15,
            geometric_area_m2: 0.02,
        };
        let solver = ConnectedCellSolver {
            minimum_current_amps: -0.1,
            maximum_current_amps: 0.1,
            ..ConnectedCellSolver::default()
        };
        let open = solver
            .solve(left, right, CellControl::OpenCircuit, 2.0)
            .unwrap();
        assert!((open.terminal_voltage_v - 1.10).abs() < 1e-8);

        let driven = solver
            .solve(
                left,
                right,
                CellControl::Potentiostatic { voltage: 1.0 },
                2.0,
            )
            .unwrap();
        assert!(driven.current_amps > 0.0);
        assert!((driven.terminal_voltage_v - 1.0).abs() < 1e-8);
        assert!(
            (driven.left.net_current_density_a_per_m2 * 0.01 - driven.current_amps).abs() < 1e-9
        );
        assert!(
            (driven.right.net_current_density_a_per_m2 * 0.02 + driven.current_amps).abs() < 1e-9
        );
        assert!(driven.irreversible_solution_heat_j(10.0).unwrap() > 0.0);
    }

    #[test]
    fn transport_limit_caps_one_partial_current() {
        let mut cathodic = reaction("oxygen", 0.0);
        cathodic.limiting_current_cathodic_a_per_m2 = Some(2.0);
        let current = cathodic.current_density(-1.0, 298.15).unwrap();
        assert!(current < 0.0 && current.abs() < 2.0);
    }

    #[test]
    fn resistive_film_is_solved_implicitly_and_reduces_current() {
        let bare = reaction("bare", 0.0);
        let mut filmed = bare;
        filmed.id = "filmed";
        filmed.film_resistance_ohm_m2 = 0.1;
        let bare_current = bare.current_density(0.2, 298.15).unwrap();
        let filmed_current = filmed.current_density(0.2, 298.15).unwrap();
        assert!(filmed_current > 0.0);
        assert!(filmed_current < bare_current);
        let activation_drop = filmed_current * filmed.film_resistance_ohm_m2;
        let expected = filmed
            .kinetics
            .current_density(0.2 - activation_drop, 298.15);
        assert!((filmed_current - expected).abs() < 1e-9);
        let balance = CurrentBalanceSolver::default()
            .currents_at(&[filmed], 0.2, 298.15)
            .unwrap();
        let heat = irreversible_ohmic_heat_j(&balance, &[filmed], 0.01, 0.0, 10.0).unwrap();
        assert!(heat > 0.0);
    }

    #[test]
    fn double_layer_transient_separates_charging_from_faradaic_current() {
        let reactions = [reaction("couple", 0.0)];
        let balance = CurrentBalanceSolver::default()
            .solve_transient_control(
                &reactions,
                298.15,
                0.01,
                CellControl::OpenCircuit,
                TransportLimits {
                    solution_resistance_ohm: 0.0,
                    limiting_current_cathodic: None,
                    limiting_current_anodic: None,
                },
                0.2,
                0.2,
                0.01,
            )
            .unwrap();
        assert!(balance.electrode_potential_v > 0.0);
        assert!(balance.electrode_potential_v < 0.2);
        assert!(balance.net_current_density_a_per_m2 > 0.0);
        assert!(balance.capacitive_current_density_a_per_m2 < 0.0);
        assert!(balance.total_current_density_a_per_m2.abs() < 1e-8);
    }

    #[test]
    fn faraday_conversion_is_extensive_in_current_and_time() {
        let one = faradaic_extent_moles(FARADAY, 1.0, 1.0).unwrap();
        let two = faradaic_extent_moles(2.0 * FARADAY, 1.0, 1.0).unwrap();
        assert!((one - 1.0).abs() < 1e-12);
        assert!((two - 2.0).abs() < 1e-12);
    }

    #[test]
    fn kinetic_domains_require_every_declared_activity() {
        let domain = KineticValidityDomain {
            temperature_min_k: 290.0,
            temperature_max_k: 310.0,
            activities: vec![ActivityBound {
                species: "H+".into(),
                minimum: 0.05,
                maximum: 0.2,
            }],
            surface_preparation: "test surface".into(),
            hydrodynamics: HydrodynamicDomain::default(),
            note: "test domain".into(),
        };
        assert!(domain.accepts(
            298.15,
            &[("H+".into(), 0.1)],
            Some("test surface"),
            HydrodynamicCondition::default(),
        ));
        assert!(!domain.accepts(
            298.15,
            &[],
            Some("test surface"),
            HydrodynamicCondition::default(),
        ));
        assert!(!domain.accepts(
            320.0,
            &[("H+".into(), 0.1)],
            Some("test surface"),
            HydrodynamicCondition::default(),
        ));
        assert!(!domain.accepts(
            298.15,
            &[("H+".into(), 0.1)],
            Some("different surface"),
            HydrodynamicCondition::default(),
        ));
    }

    fn parameter_record(id: &str) -> ExchangeCurrentRecord {
        ExchangeCurrentRecord {
            id: id.into(),
            reaction: "M+2/M".into(),
            electrode_material: "M".into(),
            kinetics: ButlerVolmerParams {
                j0: 1.0,
                alpha_a: 0.5,
                alpha_c: 0.5,
                n: 2.0,
            },
            validity: KineticValidityDomain {
                temperature_min_k: 290.0,
                temperature_max_k: 310.0,
                activities: vec![ActivityBound {
                    species: "M+2".into(),
                    minimum: 0.01,
                    maximum: 0.1,
                }],
                surface_preparation: "project-authored test surface".into(),
                hydrodynamics: HydrodynamicDomain::default(),
                note: "test-only exact parameter".into(),
            },
            source: "project-authored exact test".into(),
            license: PermissiveDataLicense::Mit,
            relative_uncertainty: Some(0.0),
            parameter_envelope: Some(KineticParameterEnvelope {
                exchange_current_density_a_per_m2: ParameterBound {
                    minimum: 1.0,
                    maximum: 1.0,
                },
                alpha_anodic: ParameterBound {
                    minimum: 0.5,
                    maximum: 0.5,
                },
                alpha_cathodic: ParameterBound {
                    minimum: 0.5,
                    maximum: 0.5,
                },
            }),
            uncertainty_note: "exact synthetic test parameter".into(),
            reviewed: true,
        }
    }

    #[test]
    fn parameter_selection_refuses_gaps_and_overlap() {
        let activities = [("M+2".into(), 0.05)];
        let first = parameter_record("first");
        assert_eq!(
            select_exchange_current(
                &[first.clone()],
                "M+2/M",
                "M",
                298.15,
                &activities,
                Some("project-authored test surface"),
                HydrodynamicCondition::default(),
            )
            .unwrap()
            .id,
            "first"
        );
        assert!(matches!(
            select_exchange_current(
                &[first.clone()],
                "M+2/M",
                "M",
                320.0,
                &activities,
                Some("project-authored test surface"),
                HydrodynamicCondition::default(),
            ),
            Err(ParameterSelectionError::NoApplicableRecord { .. })
        ));
        assert!(matches!(
            select_exchange_current(
                &[first, parameter_record("second")],
                "M+2/M",
                "M",
                298.15,
                &activities,
                Some("project-authored test surface"),
                HydrodynamicCondition::default(),
            ),
            Err(ParameterSelectionError::AmbiguousRecords { .. })
        ));
    }

    #[test]
    fn parameter_selection_enforces_preparation_flow_and_envelope() {
        let activities = [("M+2".into(), 0.05)];
        let mut record = parameter_record("rde-record");
        record.validity.hydrodynamics.rotation_rate_rpm = Some(ParameterBound {
            minimum: 1000.0,
            maximum: 1400.0,
        });
        assert!(select_exchange_current(
            &[record.clone()],
            "M+2/M",
            "M",
            298.15,
            &activities,
            Some("project-authored test surface"),
            HydrodynamicCondition {
                rotation_rate_rpm: Some(1200.0),
                ..HydrodynamicCondition::default()
            },
        )
        .is_ok());
        assert!(select_exchange_current(
            &[record.clone()],
            "M+2/M",
            "M",
            298.15,
            &activities,
            Some("project-authored test surface"),
            HydrodynamicCondition::default(),
        )
        .is_err());
        assert!(select_exchange_current(
            &[record.clone()],
            "M+2/M",
            "M",
            298.15,
            &activities,
            Some("unpolished"),
            HydrodynamicCondition {
                rotation_rate_rpm: Some(1200.0),
                ..HydrodynamicCondition::default()
            },
        )
        .is_err());

        record
            .parameter_envelope
            .as_mut()
            .unwrap()
            .exchange_current_density_a_per_m2
            .maximum = 0.5;
        assert!(select_exchange_current(
            &[record],
            "M+2/M",
            "M",
            298.15,
            &activities,
            Some("project-authored test surface"),
            HydrodynamicCondition {
                rotation_rate_rpm: Some(1200.0),
                ..HydrodynamicCondition::default()
            },
        )
        .is_err());
    }

    #[test]
    fn candidate_joins_full_nernst_quotient_to_reviewed_kinetics() {
        let records = [parameter_record("measured-set")];
        let domain = [("M+2".into(), 0.01)];
        let quotient = [EquilibriumActivity {
            species: "M+2",
            coefficient: -1.0,
            activity: 0.01,
        }];
        let partial = parameterized_partial_reaction(
            &records,
            "M+2/M",
            "M",
            -0.5,
            298.15,
            &domain,
            Some("project-authored test surface"),
            HydrodynamicCondition::default(),
            &quotient,
            1.0,
            None,
            None,
        )
        .unwrap();
        let expected = -0.5 + crate::relations::nernst_slope(crate::Kelvin::STANDARD) / 2.0 * -2.0;
        assert!((partial.equilibrium_potential_v - expected).abs() < 1e-12);
        assert_eq!(partial.parameter_record_id, Some("measured-set"));
    }

    #[test]
    fn vessel_exposes_only_resolved_aqueous_and_owned_gas_activities() {
        let mut vessel = crate::Vessel::new(crate::VesselId(0), "activity cell");
        vessel.solution = Some(crate::SolutionInfo {
            scope: crate::SolutionScope::Complete,
            solvent_kg: Some(1.0),
            redox: Vec::new(),
            pe: None,
            ph: 2.0,
            ionic_strength: 0.1,
            species: vec![crate::SpeciesDetail {
                name: "Zn+2".into(),
                molality: 0.1,
                activity: 0.075,
            }],
            provenance: None,
        });
        assert!(
            (vessel
                .resolved_aqueous_activity(&crate::SpeciesId::new("H+"))
                .unwrap()
                - 0.01)
                .abs()
                < 1e-12
        );
        assert_eq!(
            vessel.resolved_aqueous_activity(&crate::SpeciesId::new("Zn+2")),
            Some(0.075)
        );
        assert_eq!(
            vessel.ideal_gas_activity(&crate::SpeciesId::new("H2")),
            None
        );
        let hydrogen_quotient = [
            ActivityRequirement {
                species: "H+",
                coefficient: -2.0,
                source: ActivitySource::ResolvedAqueous,
            },
            ActivityRequirement {
                species: "H2",
                coefficient: 1.0,
                source: ActivitySource::OwnedIdealGas,
            },
        ];
        assert_eq!(
            resolve_equilibrium_activities(&vessel, &hydrogen_quotient),
            Err(ActivityResolutionError {
                species: "H2".into(),
                source: ActivitySource::OwnedIdealGas,
            })
        );
        let reference_hydrogen = [ActivityRequirement {
            species: "H2",
            coefficient: 1.0,
            source: ActivitySource::StandardStateGas,
        }];
        assert_eq!(
            resolve_equilibrium_activities(&vessel, &reference_hydrogen).unwrap()[0].activity,
            1.0
        );
        let water_is_not_a_standard_gas = [ActivityRequirement {
            species: "water",
            coefficient: 1.0,
            source: ActivitySource::StandardStateGas,
        }];
        assert!(resolve_equilibrium_activities(&vessel, &water_is_not_a_standard_gas).is_err());

        vessel.headspace = crate::Headspace::Sealed {
            volume: crate::Liters(1.0),
        };
        let one_atm_moles = crate::constants::STANDARD_ATMOSPHERE * 1e-3
            / (crate::constants::GAS_CONSTANT * vessel.temperature.0);
        vessel.deposit(
            crate::SpeciesId::new("H2"),
            crate::Moles(one_atm_moles),
            crate::Phase::Gas,
        );
        assert!(
            (vessel
                .ideal_gas_activity(&crate::SpeciesId::new("H2"))
                .unwrap()
                - 1.0)
                .abs()
                < 1e-12
        );
        let activities = resolve_equilibrium_activities(&vessel, &hydrogen_quotient).unwrap();
        let potential = equilibrium_potential_v(0.0, 2.0, 298.15, &activities).unwrap();
        assert!(
            (potential + 2.0 * crate::relations::nernst_slope(crate::Kelvin::STANDARD)).abs()
                < 1e-12
        );
    }

    #[test]
    fn galvanostatic_control_keeps_current_and_books_ir_drop() {
        let reactions = [reaction("couple", 0.0)];
        let balance = CurrentBalanceSolver::default()
            .solve_control(
                &reactions,
                298.15,
                2.0,
                CellControl::Galvanostatic { current_amps: 4.0 },
                TransportLimits {
                    solution_resistance_ohm: 0.25,
                    limiting_current_cathodic: None,
                    limiting_current_anodic: None,
                },
            )
            .unwrap();
        assert!((balance.net_current_density_a_per_m2 - 2.0).abs() < 1e-8);
        assert!((balance.terminal_potential_v - balance.electrode_potential_v - 1.0).abs() < 1e-8);
    }

    #[test]
    fn a_fully_blocked_surface_has_no_invented_potential() {
        let mut blocked = reaction("blocked", 0.0);
        blocked.reactive_area_ratio = 0.0;
        assert_eq!(
            CurrentBalanceSolver::default().solve_mixed_potential(&[blocked], 298.15),
            Err(ElectrochemistryError::NoActivePartialReactions)
        );
    }

    #[test]
    fn potentiostatic_control_solves_ir_drop_implicitly() {
        let reactions = [reaction("couple", 0.0)];
        let balance = CurrentBalanceSolver::default()
            .solve_control(
                &reactions,
                298.15,
                0.01,
                CellControl::Potentiostatic { voltage: 0.2 },
                TransportLimits {
                    solution_resistance_ohm: 10.0,
                    limiting_current_cathodic: None,
                    limiting_current_anodic: None,
                },
            )
            .unwrap();
        let current = balance.net_current_density_a_per_m2 * 0.01;
        assert!((balance.electrode_potential_v + current * 10.0 - 0.2).abs() < 1e-8);
        assert_eq!(balance.terminal_potential_v, 0.2);
    }

    #[test]
    fn mixed_current_converts_each_half_reaction_to_extent() {
        let reactions = [reaction("metal", -1.0), reaction("hydrogen", 0.0)];
        let balance = CurrentBalanceSolver::default()
            .solve_mixed_potential(&reactions, 298.15)
            .unwrap();
        let extents = balance.extents(&reactions, 0.01, 60.0).unwrap();
        assert_eq!(extents.len(), 2);
        assert!(extents[0].extent_moles > 0.0);
        assert!(extents[1].extent_moles < 0.0);
        assert!((extents[0].extent_moles + extents[1].extent_moles).abs() < 1e-10);
    }

    #[test]
    fn paired_half_reactions_commit_one_conserved_acid_metal_step() {
        const ZINC_TERMS: &[FaradaicTerm<'static>] = &[
            FaradaicTerm {
                reservoir: FaradaicReservoir::ElectrodeSubstrate { electrode: "zinc" },
                coefficient: -1.0,
            },
            FaradaicTerm {
                reservoir: FaradaicReservoir::Bulk {
                    species: "Zn+2",
                    phase: crate::Phase::Aqueous,
                },
                coefficient: 1.0,
            },
        ];
        // Anodic convention: H2 -> 2 H+ + 2 e-. Cathodic current makes the
        // signed extent negative and therefore runs these terms backward.
        const HYDROGEN_TERMS: &[FaradaicTerm<'static>] = &[
            FaradaicTerm {
                reservoir: FaradaicReservoir::Bulk {
                    species: "H2",
                    phase: crate::Phase::Gas,
                },
                coefficient: -1.0,
            },
            FaradaicTerm {
                reservoir: FaradaicReservoir::Bulk {
                    species: "H+",
                    phase: crate::Phase::Aqueous,
                },
                coefficient: 2.0,
            },
        ];
        let mut zinc = reaction("zinc", -1.0);
        zinc.kinetics.n = 2.0;
        zinc.kinetics.j0 = 1e-3;
        let mut hydrogen = reaction("hydrogen", 0.0);
        hydrogen.kinetics.n = 2.0;
        hydrogen.kinetics.j0 = 1e-3;
        let partials = [zinc, hydrogen];
        let halves = [
            FaradaicHalfReaction {
                id: "zinc",
                electrons_produced: 2.0,
                terms: ZINC_TERMS,
            },
            FaradaicHalfReaction {
                id: "hydrogen",
                electrons_produced: 2.0,
                terms: HYDROGEN_TERMS,
            },
        ];
        let balance = CurrentBalanceSolver::default()
            .solve_mixed_potential(&partials, 298.15)
            .unwrap();

        let mut vessel = crate::Vessel::new(crate::VesselId(0), "acid cell");
        vessel.electrodes.push(crate::ElectrodeState {
            label: "zinc".into(),
            material: "Zn".into(),
            surface_preparation: None,
            substrate_moles: Some(0.01),
            area_m2: 1e-6,
            roughness: 1.0,
            double_layer_capacitance_f_per_m2: None,
            interfacial_potential_v: None,
            deposits: Vec::new(),
        });
        vessel.deposit(
            crate::SpeciesId::new("H+"),
            crate::Moles(1e-6),
            crate::Phase::Aqueous,
        );
        let delta = faradaic_state_delta(&balance, &partials, &halves, 1e-6, 1.0)
            .unwrap()
            .limited_to_inventory(&vessel)
            .unwrap();
        delta.commit_conserved(&mut vessel, 1e-10).unwrap();

        assert!(vessel.moles_of(&crate::SpeciesId::new("Zn+2")).0 > 0.0);
        assert!(vessel.moles_of(&crate::SpeciesId::new("H2")).0 > 0.0);
        assert!(vessel.electrodes[0].substrate_moles.unwrap() < 0.01);
        assert!(vessel.moles_of(&crate::SpeciesId::new("H+")).0 < 1e-15);
        assert!((vessel.moles_of(&crate::SpeciesId::new("H2")).0 - 0.5e-6).abs() < 1e-12);
    }

    #[test]
    fn generic_step_pipeline_resolves_selects_solves_and_limits() {
        const ZINC_QUOTIENT: &[ActivityRequirement<'static>] = &[ActivityRequirement {
            species: "Zn+2",
            coefficient: -1.0,
            source: ActivitySource::ResolvedAqueous,
        }];
        const HYDROGEN_QUOTIENT: &[ActivityRequirement<'static>] = &[
            ActivityRequirement {
                species: "H+",
                coefficient: -2.0,
                source: ActivitySource::ResolvedAqueous,
            },
            ActivityRequirement {
                species: "H2",
                coefficient: 1.0,
                source: ActivitySource::StandardStateGas,
            },
        ];
        const ZINC_TERMS: &[FaradaicTerm<'static>] = &[
            FaradaicTerm {
                reservoir: FaradaicReservoir::ElectrodeSubstrate { electrode: "zinc" },
                coefficient: -1.0,
            },
            FaradaicTerm {
                reservoir: FaradaicReservoir::Bulk {
                    species: "Zn+2",
                    phase: crate::Phase::Aqueous,
                },
                coefficient: 1.0,
            },
        ];
        const HYDROGEN_TERMS: &[FaradaicTerm<'static>] = &[
            FaradaicTerm {
                reservoir: FaradaicReservoir::Bulk {
                    species: "H2",
                    phase: crate::Phase::Gas,
                },
                coefficient: -1.0,
            },
            FaradaicTerm {
                reservoir: FaradaicReservoir::Bulk {
                    species: "H+",
                    phase: crate::Phase::Aqueous,
                },
                coefficient: 2.0,
            },
        ];

        let mut zinc_record = parameter_record("synthetic-zinc");
        zinc_record.reaction = "Zn+2/Zn".into();
        zinc_record.electrode_material = "Zn".into();
        zinc_record.validity.activities[0].species = "Zn+2".into();
        let mut hydrogen_record = parameter_record("synthetic-hydrogen-on-zinc");
        hydrogen_record.reaction = "H+/H2".into();
        hydrogen_record.electrode_material = "Zn".into();
        hydrogen_record.validity.activities[0].species = "H+".into();
        let records = [zinc_record, hydrogen_record];
        let definitions = [
            ElectrochemicalReactionDefinition {
                id: "Zn+2/Zn",
                standard_reduction_potential_v: -1.0,
                quotient_requirements: ZINC_QUOTIENT,
                kinetic_domain_requirements: ZINC_QUOTIENT,
                surface_availability: SurfaceAvailabilityModel::Explicit { fraction: 1.0 },
                anodic_transport: None,
                cathodic_transport: None,
                anodic_transported_species: None,
                cathodic_transported_species: None,
                film_resistance: FilmResistanceModel::None,
                anodic_terms: ZINC_TERMS,
                electrons_produced: 2.0,
            },
            ElectrochemicalReactionDefinition {
                id: "H+/H2",
                standard_reduction_potential_v: 0.0,
                quotient_requirements: HYDROGEN_QUOTIENT,
                kinetic_domain_requirements: &HYDROGEN_QUOTIENT[..1],
                surface_availability: SurfaceAvailabilityModel::Explicit { fraction: 1.0 },
                anodic_transport: None,
                cathodic_transport: Some(CurrentLimitModel::DiffusionLayer(
                    crate::heterogeneous::DiffusionLayerTransport {
                        diffusivity_m2_per_s: 1e-9,
                        bulk_concentration_mol_per_m3: 10.0,
                        diffusion_layer_m: 1e-4,
                    },
                )),
                anodic_transported_species: None,
                cathodic_transported_species: Some(TransportedSpecies {
                    species: "H+",
                    moles_per_extent: 2.0,
                }),
                film_resistance: FilmResistanceModel::None,
                anodic_terms: HYDROGEN_TERMS,
                electrons_produced: 2.0,
            },
        ];
        let mut vessel = crate::Vessel::new(crate::VesselId(0), "generic acid cell");
        vessel.electrodes.push(crate::ElectrodeState {
            label: "zinc".into(),
            material: "Zn".into(),
            surface_preparation: Some("project-authored test surface".into()),
            substrate_moles: Some(0.01),
            area_m2: 0.1,
            roughness: 2.0,
            double_layer_capacitance_f_per_m2: None,
            interfacial_potential_v: None,
            deposits: Vec::new(),
        });
        vessel.deposit(
            crate::SpeciesId::new("H+"),
            crate::Moles(1e-6),
            crate::Phase::Aqueous,
        );
        vessel.deposit(
            crate::SpeciesId::new("Zn+2"),
            crate::Moles(1e-9),
            crate::Phase::Aqueous,
        );
        vessel.solution = Some(crate::SolutionInfo {
            scope: crate::SolutionScope::Complete,
            solvent_kg: Some(1.0),
            redox: Vec::new(),
            pe: None,
            ph: 2.0,
            ionic_strength: 0.1,
            species: vec![crate::SpeciesDetail {
                name: "Zn+2".into(),
                molality: 0.05,
                activity: 0.05,
            }],
            provenance: None,
        });

        let proposal = propose_electrochemical_step(
            &vessel,
            "zinc",
            &records,
            &definitions,
            60.0,
            HydrodynamicCondition::default(),
            CellControl::OpenCircuit,
            TransportLimits {
                solution_resistance_ohm: 0.0,
                limiting_current_cathodic: None,
                limiting_current_anodic: None,
            },
            CurrentBalanceSolver::default(),
        )
        .unwrap();
        assert!(proposal.accepted_fraction < 1.0);
        assert!(proposal.accepted_fraction > 0.0);
        assert!(proposal.balance.partial_currents[0].current_density_a_per_m2 > 0.0);
        assert!(proposal.balance.partial_currents[1].current_density_a_per_m2 < 0.0);
        assert_eq!(proposal.interfacial_conditions.len(), 1);
        let interface = &proposal.interfacial_conditions[0];
        assert_eq!(interface.species, "H+");
        assert!(interface.surface_concentration_mol_per_m3 < 10.0);
        assert!(interface.depleted_at_surface || interface.surface_ph.is_some_and(|ph| ph > 2.0));
        let mut committed = vessel.clone();
        proposal
            .accepted_delta
            .commit_conserved(&mut committed, 1e-10)
            .unwrap();
        assert!(committed.moles_of(&crate::SpeciesId::new("H2")).0 > 0.0);
        assert!(committed.moles_of(&crate::SpeciesId::new("H+")).0 < 1e-15);
        assert!(committed.electrodes[0].substrate_moles.unwrap() < 0.01);

        let mut capacitive = vessel.clone();
        capacitive.electrodes[0].double_layer_capacitance_f_per_m2 = Some(0.2);
        capacitive.electrodes[0].interfacial_potential_v = Some(-0.2);
        let transient = propose_electrochemical_step(
            &capacitive,
            "zinc",
            &records,
            &definitions,
            60.0,
            HydrodynamicCondition::default(),
            CellControl::OpenCircuit,
            TransportLimits {
                solution_resistance_ohm: 0.0,
                limiting_current_cathodic: None,
                limiting_current_anodic: None,
            },
            CurrentBalanceSolver::default(),
        )
        .unwrap();
        assert!(transient.accepted_fraction > 0.0 && transient.accepted_fraction < 1.0);
        assert_eq!(
            transient.accepted_delta.electrode_potential_changes.len(),
            1
        );
        transient
            .accepted_delta
            .commit_conserved(&mut capacitive, 1e-10)
            .unwrap();

        let mut advanced = vessel;
        let report = advance_electrochemical(
            &mut advanced,
            "zinc",
            &records,
            &definitions,
            60.0,
            HydrodynamicCondition::default(),
            CellControl::OpenCircuit,
            TransportLimits {
                solution_resistance_ohm: 0.0,
                limiting_current_cathodic: None,
                limiting_current_anodic: None,
            },
            CurrentBalanceSolver::default(),
            ElectrochemicalAdvanceOptions::default(),
            &mut |state| {
                if state.moles_of(&crate::SpeciesId::new("H+")).0 < 1e-15 {
                    state.solution = None;
                }
            },
        )
        .unwrap();
        assert_eq!(report.segments.len(), 1);
        assert!(report.segments[0].inventory_limited);
        assert!(report.elapsed_seconds > 0.0 && report.elapsed_seconds < 60.0);
        assert!(matches!(
            report.boundary,
            Some(ElectrochemicalStepError::Activity { .. })
        ));
    }
}
