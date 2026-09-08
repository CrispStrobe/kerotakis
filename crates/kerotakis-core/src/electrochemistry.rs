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
        let kinetic = self.kinetics.current_density(
            electrode_potential_v - self.equilibrium_potential_v,
            temperature_k,
        );
        let limit = if kinetic >= 0.0 {
            self.limiting_current_anodic_a_per_m2
        } else {
            self.limiting_current_cathodic_a_per_m2
        };
        let current = transport_limited_flux(kinetic, limit) * self.reactive_area_ratio;
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
    /// Atom-balanced matter bookkeeping in the anodic direction.
    pub anodic_terms: &'a [FaradaicTerm<'a>],
    pub electrons_produced: f64,
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
        electrons_per_mole: f64,
    ) -> Result<f64, crate::heterogeneous::SurfaceRateError> {
        match self {
            Self::MeasuredCurrentDensity { amperes_per_m2 }
                if amperes_per_m2.is_finite() && amperes_per_m2 > 0.0 =>
            {
                Ok(amperes_per_m2)
            }
            Self::MeasuredCurrentDensity { .. } => {
                Err(crate::heterogeneous::SurfaceRateError::InvalidTransport)
            }
            Self::DiffusionLayer(model) => model.current_density_limit(electrons_per_mole),
            Self::RotatingDisk(model) => model.current_density_limit(electrons_per_mole),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElectrochemicalStepProposal {
    pub balance: CurrentBalance,
    /// Unbounded Faraday-law proposal, retained for diagnostics.
    pub requested_delta: crate::delta::StateDelta,
    /// Uniformly inventory-limited proposal safe to conservation-check and
    /// commit atomically.
    pub accepted_delta: crate::delta::StateDelta,
    pub accepted_fraction: f64,
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
    Inventory(Vec<crate::delta::DeltaError>),
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
    let mut reaction_ids = std::collections::BTreeSet::new();
    for definition in definitions {
        if definition.id.trim().is_empty()
            || !reaction_ids.insert(definition.id)
            || !definition.electrons_produced.is_finite()
            || definition.electrons_produced <= 0.0
            || definition.anodic_terms.is_empty()
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
            .map(|model| model.current_density_limit(definition.electrons_produced))
            .transpose()
            .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                reaction: definition.id.to_owned(),
                reason: format!("invalid anodic transport model: {reason:?}"),
            })?;
        let cathodic_limit = definition
            .cathodic_transport
            .map(|model| model.current_density_limit(definition.electrons_produced))
            .transpose()
            .map_err(|reason| ElectrochemicalStepError::InvalidDefinition {
                reaction: definition.id.to_owned(),
                reason: format!("invalid cathodic transport model: {reason:?}"),
            })?;
        partial_reactions.push(
            parameterized_partial_reaction(
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
            })?,
        );
        half_reactions.push(FaradaicHalfReaction {
            id: definition.id,
            electrons_produced: definition.electrons_produced,
            terms: definition.anodic_terms,
        });
    }

    let balance = solver
        .solve_control(
            &partial_reactions,
            temperature_k,
            electrode.area_m2,
            control,
            transport,
        )
        .map_err(ElectrochemicalStepError::Balance)?;
    let requested_delta = faradaic_state_delta(
        &balance,
        &partial_reactions,
        &half_reactions,
        electrode.area_m2,
        seconds,
    )
    .map_err(ElectrochemicalStepError::Balance)?;
    let limited = requested_delta
        .inventory_limited(vessel)
        .map_err(ElectrochemicalStepError::Inventory)?;
    Ok(ElectrochemicalStepProposal {
        balance,
        requested_delta,
        accepted_delta: limited.delta,
        accepted_fraction: limited.accepted_fraction,
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
    fn transport_limit_caps_one_partial_current() {
        let mut cathodic = reaction("oxygen", 0.0);
        cathodic.limiting_current_cathodic_a_per_m2 = Some(2.0);
        let current = cathodic.current_density(-1.0, 298.15).unwrap();
        assert!(current < 0.0 && current.abs() < 2.0);
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
                cathodic_transport: None,
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
        let mut committed = vessel.clone();
        proposal
            .accepted_delta
            .commit_conserved(&mut committed, 1e-10)
            .unwrap();
        assert!(committed.moles_of(&crate::SpeciesId::new("H2")).0 > 0.0);
        assert!(committed.moles_of(&crate::SpeciesId::new("H+")).0 < 1e-15);
        assert!(committed.electrodes[0].substrate_moles.unwrap() < 0.01);
    }
}
