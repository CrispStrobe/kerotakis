//! ARCH-014: Coverage manifest — for every registered operation and species
//! family, report claimed models, validity, observables, and unsupported
//! dimensions.

use serde::{Deserialize, Serialize};

use crate::solve::Equilibrator;
use crate::species;
use crate::vessel::Vessel;

/// One solver's coverage claim for a single vessel state.
#[derive(Debug, Clone, Serialize)]
pub struct SolverCoverage {
    pub solver_name: &'static str,
    pub applicable: bool,
    pub is_chemistry: bool,
    pub validity_notes: Option<String>,
}

/// Coverage report for the full solver stack against a vessel.
#[derive(Debug, Clone, Serialize)]
pub struct CoverageReport {
    /// Registry species count.
    pub species_count: usize,
    /// Solver coverage claims.
    pub solvers: Vec<SolverCoverage>,
    /// Operations that no solver claims.
    pub uncovered: Vec<String>,
    /// Per-observable support at this state; solver applicability alone is not accuracy.
    pub observables: Vec<ObservableSupport>,
}

/// Generate a coverage manifest by querying each solver in the stack.
pub fn coverage_manifest(solvers: &[&dyn Equilibrator], vessel: &Vessel) -> CoverageReport {
    let solver_reports: Vec<SolverCoverage> = solvers
        .iter()
        .map(|s| {
            let cap = s.capability(vessel);
            SolverCoverage {
                solver_name: s.name(),
                applicable: cap.applicability.is_applicable(),
                is_chemistry: cap.is_chemistry,
                validity_notes: cap.validity.map(|v| format!("{v:?}")),
            }
        })
        .collect();

    let any_chemistry = solver_reports
        .iter()
        .any(|s| s.applicable && s.is_chemistry);
    let mut uncovered = Vec::new();
    if !any_chemistry {
        uncovered.push("No chemistry solver applicable for this vessel state".into());
    }

    CoverageReport {
        species_count: species::REGISTRY.len(),
        solvers: solver_reports,
        uncovered,
        observables: observable_manifest(vessel),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn coverage_manifest_reports_solvers() {
        let vessel = Vessel::new(VesselId(0), "test");
        let mixing = MixingEquilibrator;
        let honesty = HonestyEquilibrator;
        let solvers: Vec<&dyn Equilibrator> = vec![&mixing, &honesty];

        let report = coverage_manifest(&solvers, &vessel);
        assert_eq!(report.species_count, species::REGISTRY.len());
        assert_eq!(report.solvers.len(), 2);
        // MixingEquilibrator is not chemistry
        assert!(!report.solvers[0].is_chemistry);
    }
}

/// Model support is independent of an instrument's resolution or calibration.
/// `Computed` means a represented-state quantity, never independently validated accuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservableStatus {
    Computed,
    Estimated,
    Incomplete,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservableScope {
    WholeVessel,
    AqueousPhase,
    LiquidPhase,
    SolventOnly,
    DrySolid,
}

/// A derived capability, not additional mutable vessel state. Stable reason keys
/// are machine-readable; assumptions/provenance document the route rather than
/// making an experimental accuracy claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableSupport {
    pub observable: String,
    pub status: ObservableStatus,
    pub scope: ObservableScope,
    pub reasons: Vec<String>,
    pub assumptions: Vec<String>,
    pub provenance: Vec<String>,
    pub validity: Vec<ObservableValidity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservableValidity {
    pub quantity: String,
    pub unit: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
}

impl ObservableSupport {
    fn new(observable: &str, scope: ObservableScope) -> Self {
        Self {
            observable: observable.into(),
            status: ObservableStatus::Computed,
            scope,
            reasons: Vec::new(),
            assumptions: Vec::new(),
            provenance: Vec::new(),
            validity: Vec::new(),
        }
    }
    fn limited(&mut self, status: ObservableStatus, reason: &str) {
        let rank = |s| match s {
            ObservableStatus::Computed => 0,
            ObservableStatus::Estimated => 1,
            ObservableStatus::Incomplete => 2,
            ObservableStatus::Unsupported => 3,
        };
        if rank(status) > rank(self.status) {
            self.status = status;
        }
        self.reasons.push(reason.into());
    }
}

/// Discover current routes before executing an operation. Missing observations
/// are listed explicitly rather than omitted or represented as numeric zero.
pub fn observable_manifest(vessel: &Vessel) -> Vec<ObservableSupport> {
    [
        "temperature",
        "enthalpy",
        "mass",
        "pressure",
        "pH",
        "conductivity",
        "absorbance",
        "reaction_rate",
        "headspace_volume",
        "density",
        "activity",
    ]
    .iter()
    .map(|name| observable_support(vessel, name))
    .collect()
}

pub fn observable_support(vessel: &Vessel, observable: &str) -> ObservableSupport {
    use ObservableScope::*;
    use ObservableStatus::*;
    let aqueous_scope = if vessel
        .solution
        .as_ref()
        .is_some_and(|s| s.scope == crate::vessel::SolutionScope::SolventOnly)
    {
        SolventOnly
    } else {
        AqueousPhase
    };
    let mut support = ObservableSupport::new(observable, WholeVessel);
    match observable {
        "temperature" | "enthalpy" => {
            support.assumptions.push(
                "represented thermal inventory; omitted reaction heats are not zero heat".into(),
            );
            if observable == "enthalpy" {
                support.limited(Estimated, "sensible-enthalpy-reference-25c");
                support.assumptions.push("registry heat capacities; dissolved-ion partial molar heat capacities may be omitted".into());
            }
            if observable == "enthalpy"
                || matches!(vessel.thermal_mode, crate::vessel::ThermalMode::Adiabatic)
            {
                thermal_parameter_support(vessel, &mut support);
            }
            // A thermostat establishes temperature, but cannot determine the
            // missing reaction enthalpy. Never clear its history as a side effect.
            if !vessel.unpriced_heat.is_empty()
                && (observable == "enthalpy" || vessel.temperature_limitation().is_some())
            {
                support.limited(Incomplete, "unpriced-reaction-heat");
            }
        }
        "reaction_rate" => {
            support.assumptions.push("route coverage only, not an aggregate rate; each mechanism has progress rates in mol/L/s and its own locality, orders and catalyst inputs".into());
            let reactions = crate::kinetics::applicable(vessel);
            if reactions.is_empty() {
                support.limited(Unsupported, "no-selected-rate-model");
                support.assumptions.push("absence of a selected rate model is not zero rate or inertness; equilibrium coverage does not establish kinetics".into());
            } else {
                support.status = Estimated;
                for reaction in reactions {
                    support
                        .reasons
                        .push(format!("selected-rate-model:{}", reaction.id));
                    support
                        .provenance
                        .extend(reaction.source_ids.iter().map(|id| (*id).to_string()));
                    support.provenance.push(reaction.provenance.into());
                    support.assumptions.push(format!(
                        "{}: {:?}; {}; uncertainty: {}{}",
                        reaction.id,
                        reaction.locality,
                        reaction.validity.note,
                        reaction.uncertainty.note,
                        reaction
                            .uncertainty
                            .relative
                            .map(|value| format!("; relative parameter uncertainty {value}"))
                            .unwrap_or_default()
                    ));
                    for (quantity, unit, range, current) in [
                        (
                            "temperature",
                            "K",
                            reaction.validity.temperature_k,
                            vessel.temperature.0,
                        ),
                        (
                            "pressure",
                            "Pa",
                            reaction.validity.pressure_pa,
                            vessel.pressure.0,
                        ),
                    ] {
                        if let Some(range) = range {
                            support.validity.push(ObservableValidity {
                                quantity: format!("{quantity}:{}", reaction.id),
                                unit: unit.into(),
                                lower: Some(range.min),
                                upper: Some(range.max),
                            });
                            if current < range.min || current > range.max {
                                support.reasons.push(format!(
                                    "outside-rate-{quantity}-domain:{}",
                                    reaction.id
                                ));
                            }
                        }
                    }
                }
            }
        }
        "mass" => {
            support
                .assumptions
                .push("represented material inventory; excludes vessel tare".into());
        }
        "headspace_volume" => {
            support.assumptions.push(
                "sealed or pressure-controlled headspace geometry; excludes liquid volume".into(),
            );
            if !matches!(
                vessel.headspace,
                crate::vessel::Headspace::Sealed { .. }
                    | crate::vessel::Headspace::PressureControlled { .. }
            ) {
                support.limited(Unsupported, "open-vessel-no-headspace-volume");
            }
        }
        "density" => {
            if crate::buoyancy::liquid_density_g_per_ml(vessel).is_some() {
                support.scope = LiquidPhase;
                support.limited(Estimated, "represented-liquid-volume-density");
                support.assumptions.push("hydrometer samples liquid only; solids excluded; additive represented volumes, not a mixture equation of state".into());
                if crate::buoyancy::ionic_volume_unaccounted(vessel) {
                    support.limited(Incomplete, "missing-dissolved-ion-volume");
                }
            } else {
                let solids: Vec<_> = vessel
                    .contents
                    .iter()
                    .filter(|p| {
                        p.phase == crate::species::Phase::Solid
                            && p.moles.0 > crate::OBSERVABLE_MOLES
                    })
                    .collect();
                let datum = match (solids.as_slice(), vessel.unresolved_materials.as_slice()) {
                    ([portion], []) => species::lookup(&portion.species).map(|data| data.density),
                    ([], [portion]) => crate::material::lookup(&portion.material, None)
                        .and_then(|recipe| recipe.bulk_density.map(|density| density.value)),
                    _ => None,
                };
                if datum.is_some_and(|value| value.is_finite() && value > 0.0) {
                    support.scope = DrySolid;
                    support.limited(Estimated, "curated-single-sample-density");
                    support
                        .assumptions
                        .push("one dry sample; registry bulk density reference".into());
                } else {
                    support.limited(Unsupported, "no-density-model-route");
                }
            }
        }
        "activity" => {
            support.assumptions.push(
                "sum of registered nuclide activities; detector geometry and efficiency excluded"
                    .into(),
            );
        }
        "pressure" => {
            support
                .assumptions
                .push("represented headspace and imposed pressure boundary".into());
            // Imposed/open pressure is known independently of an incomplete
            // adiabatic heat balance. A sealed ideal-gas pressure is not.
            if matches!(vessel.headspace, crate::vessel::Headspace::Sealed { .. }) {
                let temperature = observable_support(vessel, "temperature");
                if temperature.status != Computed {
                    support.limited(
                        temperature.status,
                        "headspace-pressure-uses-limited-temperature",
                    );
                }
            }
        }
        "pH" | "conductivity" => {
            support.scope = aqueous_scope;
            if let Some(solution) = &vessel.solution {
                if let Some(p) = &solution.provenance {
                    support.provenance.extend([
                        p.engine.clone(),
                        p.dataset.clone(),
                        p.model.clone(),
                    ]);
                } else {
                    support
                        .reasons
                        .push("solver-provenance-not-reported".into());
                }
                if aqueous_scope == SolventOnly {
                    support.limited(Incomplete, "uncovered-solutes");
                }
                aqueous_domain_support(vessel, &mut support);
                if observable_support(vessel, "temperature").status != Computed {
                    support.limited(Estimated, "aqueous-result-uses-limited-temperature");
                }
                if observable == "conductivity" {
                    let estimate = crate::conductivity::specific_conductance(solution);
                    support
                        .provenance
                        .push(crate::conductivity::LAMBDA_SOURCE.into());
                    support
                        .provenance
                        .push(crate::conductivity::FIT_SOURCE.into());
                    support.validity.push(ObservableValidity {
                        quantity: "temperature".into(),
                        unit: "K".into(),
                        lower: Some(298.15),
                        upper: Some(298.15),
                    });
                    support.validity.push(ObservableValidity {
                        quantity: "ionic_strength".into(),
                        unit: "mol/kgw".into(),
                        lower: Some(0.0),
                        upper: Some(crate::conductivity::DILUTE_LIMIT_MOLAL),
                    });
                    match &estimate.basis {
                        crate::conductivity::Basis::MeanMobility => {
                            support.limited(Estimated, "mean-mobility-without-speciation");
                        }
                        crate::conductivity::Basis::Kohlrausch { omitted, .. }
                            if !omitted.is_empty() =>
                        {
                            support.limited(Incomplete, "missing-ion-mobility");
                        }
                        _ => {}
                    }
                    if !estimate.within_dilute_limit {
                        // Missing coverage must not be demoted to a mere estimate.
                        if support.status == Computed {
                            support.status = Estimated;
                        }
                        support
                            .reasons
                            .push("outside-dilute-conductivity-domain".into());
                    }
                    if (vessel.temperature.0 - 298.15).abs() > 0.01 {
                        if support.status == Computed {
                            support.status = Estimated;
                        }
                        support
                            .reasons
                            .push("conductivity-temperature-not-calibrated".into());
                    }
                    support.assumptions.push(
                        "25 C limiting mobility; alkali-halide concentration correction".into(),
                    );
                }
            } else if crate::conductivity::neutral_aqueous_ph(vessel).is_some() {
                support.limited(Estimated, "nonionic-water-baseline");
                support
                    .assumptions
                    .push("nonionic solutes do not alter water dissociation or mobility".into());
            } else if observable == "conductivity"
                && crate::conductivity::dry_solid_conductance(vessel).is_some()
            {
                support.scope = DrySolid;
                support.limited(Estimated, "curated-bulk-resistivity");
                let solid = crate::conductivity::dry_solid_conductance(vessel)
                    .expect("route checked above");
                support.provenance.push(solid.source.to_string());
                support.validity.push(ObservableValidity {
                    quantity: "temperature".into(),
                    unit: "K".into(),
                    lower: Some(293.15),
                    upper: Some(293.15),
                });
                support
                    .assumptions
                    .push("single dry sample; no solution; bulk reference property".into());
            } else {
                support.limited(Unsupported, "no-aqueous-model-route");
            }
        }
        "absorbance" => {
            support.scope = aqueous_scope;
            let gaps = crate::solution_optics::spectral_gaps(vessel);
            if !gaps.is_empty() {
                support.limited(Incomplete, "missing-spectral-data");
                support.reasons.extend(
                    gaps.into_iter()
                        .map(|key| format!("missing-spectrum:{key}")),
                );
            } else if !vessel
                .contents
                .iter()
                .any(|p| p.phase == crate::species::Phase::Aqueous && p.moles.0 > 0.0)
            {
                support.limited(Unsupported, "no-aqueous-sample");
            } else {
                support.status = Estimated;
            }
            if aqueous_scope == SolventOnly {
                support.limited(Incomplete, "uncovered-solutes");
            }
            support
                .assumptions
                .push("Beer-Lambert additive bands; caller supplies optical path".into());
        }
        _ => support.limited(Unsupported, "observable-route-not-described"),
    }
    support
}

/// The operator and ideal-instrument paths share observable names and support.
pub fn instrument_support(
    vessel: &Vessel,
    instrument: crate::ops::Instrument,
) -> ObservableSupport {
    use crate::ops::Instrument;
    let observable = match instrument {
        Instrument::Thermometer => "temperature",
        Instrument::Balance => "mass",
        Instrument::PhMeter => "pH",
        Instrument::PressureGauge => "pressure",
        Instrument::VolumeMeter => "headspace_volume",
        Instrument::ConductivityMeter => "conductivity",
        Instrument::Densitometer => "density",
        Instrument::Spectrophotometer => "absorbance",
        Instrument::Calorimeter => "enthalpy",
        Instrument::GeigerCounter => "activity",
        _ => "observable-route-not-described",
    };
    observable_support(vessel, observable)
}

fn thermal_parameter_support(vessel: &Vessel, support: &mut ObservableSupport) {
    use ObservableStatus::*;
    for portion in vessel
        .contents
        .iter()
        .filter(|p| p.moles.0 > crate::OBSERVABLE_MOLES)
    {
        let Some(data) = species::lookup(&portion.species) else {
            support.limited(Incomplete, "missing-heat-capacity");
            support
                .reasons
                .push(format!("missing-heat-capacity:{}", portion.species.0));
            continue;
        };
        let cp = crate::states::heat_capacity_at(data, portion.phase, vessel.temperature.0);
        if !cp.is_finite() || cp <= 0.0 {
            if portion.phase == crate::species::Phase::Aqueous && data.heat_capacity == 0.0 {
                support.limited(Estimated, "aqueous-partial-heat-capacity-omitted");
            } else {
                support.limited(Incomplete, "missing-heat-capacity");
                support
                    .reasons
                    .push(format!("missing-heat-capacity:{}", portion.species.0));
            }
        }
        if let Some(curve) = crate::states::heat_capacity_curve(data, portion.phase) {
            if let Some((lower, upper)) = curve.range() {
                support.validity.push(ObservableValidity {
                    quantity: format!("heat-capacity-temperature:{}", portion.species.0),
                    unit: "K".into(),
                    lower: Some(lower),
                    upper: Some(upper),
                });
                if !(lower..=upper).contains(&vessel.temperature.0) {
                    support.limited(Estimated, "heat-capacity-held-at-curve-endpoint");
                }
                support.provenance.push(curve.source.into());
            }
        } else if cp > 0.0 && (vessel.temperature.0 - 298.15).abs() > 0.01 {
            support.limited(Estimated, "constant-heat-capacity-away-from-reference");
        }
    }
    for portion in vessel
        .unresolved_materials
        .iter()
        .filter(|p| p.amount > 0.0)
    {
        let cp = crate::material::lookup_versioned(&portion.recipe_id, portion.recipe_version)
            .filter(|_| portion.basis == crate::material::MaterialBasis::MassFraction)
            .and_then(|recipe| {
                recipe.roles.iter().find_map(|role| match role {
                    crate::material::MaterialRole::PolymerHeatResponse {
                        specific_heat_j_per_g_k,
                        ..
                    } => Some(*specific_heat_j_per_g_k),
                    _ => None,
                })
            });
        if !cp.is_some_and(|value| value.is_finite() && value > 0.0) {
            support.limited(Incomplete, "missing-material-heat-capacity");
            support.reasons.push(format!(
                "missing-material-heat-capacity:{}",
                portion.material
            ));
        } else {
            support.limited(Estimated, "curated-material-heat-capacity");
        }
    }
    support.reasons.sort();
    support.reasons.dedup();
    support.provenance.sort();
    support.provenance.dedup();
}

fn aqueous_domain_support(vessel: &Vessel, support: &mut ObservableSupport) {
    support
        .assumptions
        .push("aqueous activity models assume water as solvent".into());
    support.validity.push(ObservableValidity {
        quantity: "temperature".into(),
        unit: "K".into(),
        lower: None,
        upper: Some(crate::solve::AQUEOUS_MODEL_CEILING_K),
    });
    if vessel.temperature.0 > crate::solve::AQUEOUS_MODEL_CEILING_K {
        support.limited(
            ObservableStatus::Unsupported,
            "outside-aqueous-temperature-domain",
        );
    }
    // A separate organic layer does not change the solvent of the aqueous
    // phase. Exclude only that layer: ethanol beside water+hexane remains a
    // co-solvent and still deserves a limitation.
    let excluded = crate::solve::layered_pair(vessel).map(|(upper, _)| upper);
    let (water, organic) = vessel
        .contents
        .iter()
        .filter(|portion| {
            matches!(
                portion.phase,
                crate::species::Phase::Liquid | crate::species::Phase::Aqueous
            )
        })
        .filter(|portion| excluded != Some(portion.species.0.as_str()))
        .fold((0.0, 0.0), |(water, organic), portion| {
            if portion.species.0 == "water" {
                (water + portion.moles.0, organic)
            } else if crate::nonaqueous::KNOWN_SOLVENTS.contains(&portion.species.0.as_str()) {
                (water, organic + portion.moles.0)
            } else {
                (water, organic)
            }
        });
    if water > 0.0 && organic > 0.0 {
        let fraction = water / (water + organic);
        support.validity.push(ObservableValidity {
            quantity: "aqueous-water-mole-fraction".into(),
            unit: "1".into(),
            lower: Some(crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR),
            upper: Some(1.0),
        });
        if fraction < crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR {
            support.limited(
                ObservableStatus::Unsupported,
                "mostly-organic-aqueous-route-declined",
            );
        } else {
            support.limited(
                ObservableStatus::Estimated,
                "mixed-solvent-activity-not-calibrated",
            );
        }
    }
}
