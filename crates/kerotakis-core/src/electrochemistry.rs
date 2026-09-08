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
    /// Required account of measurement scatter, fit sensitivity and omissions.
    pub uncertainty_note: String,
    /// Whether this record has been reviewed for the runtime allowlist.
    pub reviewed: bool,
}

/// Curated exchange-current records from reviewed sources.
///
/// Sources: Bard & Faulkner, Electrochemical Methods (2001), Table 3.6.2;
/// CRC Handbook of Chemistry and Physics, 97th ed.
pub const EXCHANGE_CURRENTS: &[ExchangeCurrentRecord] = &[
    // These are representative values; actual records should carry
    // full citation metadata per ELEC-003 requirements.
];

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
            && self.activities.iter().all(|bound| {
                !bound.species.trim().is_empty()
                    && bound.minimum.is_finite()
                    && bound.maximum.is_finite()
                    && bound.minimum >= 0.0
                    && bound.minimum <= bound.maximum
            })
    }

    pub fn accepts(&self, temperature_k: f64, activities: &[(String, f64)]) -> bool {
        self.validate()
            && temperature_k.is_finite()
            && temperature_k >= self.temperature_min_k
            && temperature_k <= self.temperature_max_k
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
    ) -> bool {
        self.reviewed
            && !self.id.trim().is_empty()
            && !self.reaction.trim().is_empty()
            && !self.source.trim().is_empty()
            && !self.uncertainty_note.trim().is_empty()
            && self
                .relative_uncertainty
                .is_none_or(|value| value.is_finite() && value >= 0.0)
            && self.electrode_material == electrode_material
            && self.kinetics.validate().is_ok()
            && self.validity.accepts(temperature_k, activities)
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
) -> Result<&'a ExchangeCurrentRecord, ParameterSelectionError> {
    let matching: Vec<_> = records
        .iter()
        .filter(|record| {
            record.reaction == reaction
                && record.applies_to(electrode_material, temperature_k, activities)
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
    pub maximum_iterations: usize,
}

impl Default for CurrentBalanceSolver {
    fn default() -> Self {
        Self {
            minimum_potential_v: -3.0,
            maximum_potential_v: 3.0,
            potential_tolerance_v: 1e-9,
            current_tolerance_a_per_m2: 1e-9,
            maximum_iterations: 160,
        }
    }
}

impl CurrentBalanceSolver {
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

        if f_lower.abs() <= self.current_tolerance_a_per_m2 {
            return Ok(lower_balance);
        }
        if f_upper.abs() <= self.current_tolerance_a_per_m2 {
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
            if f_middle.abs() <= self.current_tolerance_a_per_m2 {
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
        Err(ElectrochemistryError::DidNotConverge {
            residual: balance.net_current_density_a_per_m2 - target_current_density,
        })
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
                FaradaicReservoir::ElectrodeDeposit { electrode, species } => delta
                    .with_electrode_moles(
                        electrode,
                        crate::delta::ElectrodeInventory::Deposit {
                            species: crate::SpeciesId::new(species),
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
            note: "test domain".into(),
        };
        assert!(domain.accepts(298.15, &[("H+".into(), 0.1)]));
        assert!(!domain.accepts(298.15, &[]));
        assert!(!domain.accepts(320.0, &[("H+".into(), 0.1)]));
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
                note: "test-only exact parameter".into(),
            },
            source: "project-authored exact test".into(),
            license: PermissiveDataLicense::Mit,
            relative_uncertainty: Some(0.0),
            uncertainty_note: "exact synthetic test parameter".into(),
            reviewed: true,
        }
    }

    #[test]
    fn parameter_selection_refuses_gaps_and_overlap() {
        let activities = [("M+2".into(), 0.05)];
        let first = parameter_record("first");
        assert_eq!(
            select_exchange_current(&[first.clone()], "M+2/M", "M", 298.15, &activities)
                .unwrap()
                .id,
            "first"
        );
        assert!(matches!(
            select_exchange_current(&[first.clone()], "M+2/M", "M", 320.0, &activities),
            Err(ParameterSelectionError::NoApplicableRecord { .. })
        ));
        assert!(matches!(
            select_exchange_current(
                &[first, parameter_record("second")],
                "M+2/M",
                "M",
                298.15,
                &activities
            ),
            Err(ParameterSelectionError::AmbiguousRecords { .. })
        ));
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
            &records, "M+2/M", "M", -0.5, 298.15, &domain, &quotient, 1.0, None, None,
        )
        .unwrap();
        let expected = -0.5 + crate::relations::nernst_slope(crate::Kelvin::STANDARD) / 2.0 * -2.0;
        assert!((partial.equilibrium_potential_v - expected).abs() < 1e-12);
        assert_eq!(partial.parameter_record_id, Some("measured-set"));
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
}
