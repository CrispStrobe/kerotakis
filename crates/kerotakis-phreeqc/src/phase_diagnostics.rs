//! Explain unoffered thermodynamic phases without inventing precipitation kinetics.
use crate::derived;
use kerotakis_core::{ops::NotModelledCause, species, Event, VesselId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Exclusion {
    GasBoundary,
    TemperatureWithheld,
    RegisteredExcluded,
    MissingSolid,
    UnknownPhase,
}

fn classify(phase: &str, db_tag: &str, temperature_k: f64) -> Exclusion {
    let Some(info) = derived::index_for(db_tag).phases.get(phase) else {
        return Exclusion::UnknownPhase;
    };
    if info.is_gas {
        return Exclusion::GasBoundary;
    }
    // The candidate list deliberately omits some registered solids (for
    // example elemental metals owned by electron-balanced displacement).
    // Registry coverage is a composition question, not candidate membership.
    let Some(key) = derived::registry_solid_matching(&info.composition, info.waters) else {
        return Exclusion::MissingSolid;
    };
    if species::lookup_key(key)
        .and_then(|s| s.forms_only_above_k)
        .is_some_and(|floor| temperature_k < floor)
    {
        Exclusion::TemperatureWithheld
    } else {
        Exclusion::RegisteredExcluded
    }
}

fn describe(list: &[(&str, f64)]) -> String {
    let first = list
        .iter()
        .take(3)
        .map(|(name, si)| format!("{name} (SI {si:+.1})"))
        .collect::<Vec<_>>()
        .join(", ");
    match list.len().saturating_sub(3) {
        0 => first,
        n => format!("{first}, and {n} more"),
    }
}

/// Diagnose only unoffered finite indices above the caller's reporting floor.
/// Gas SI uses the database reference fugacity, not the vessel's total pressure;
/// a positive value alone is not a bubble-formation or transfer-rate prediction.
pub(crate) fn events(
    vessel: VesselId,
    temperature_k: f64,
    db_tag: &str,
    saturation: &[(String, f64)],
    offered: &[&str],
    reporting_si: f64,
) -> Vec<Event> {
    let mut result = Vec::new();
    for category in [
        Exclusion::MissingSolid,
        Exclusion::TemperatureWithheld,
        Exclusion::RegisteredExcluded,
        Exclusion::GasBoundary,
        Exclusion::UnknownPhase,
    ] {
        let mut phases: Vec<_> = saturation
            .iter()
            .filter(|(name, si)| {
                si.is_finite()
                    && *si >= reporting_si
                    && !offered.contains(&name.as_str())
                    && classify(name, db_tag, temperature_k) == category
            })
            .map(|(name, si)| (name.as_str(), *si))
            .collect();
        if phases.is_empty() {
            continue;
        }
        phases.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
        let named = describe(&phases);
        let (cause, what) = match category {
            Exclusion::MissingSolid => (
                NotModelledCause::PhaseNotInRegistry,
                format!("the solution is supersaturated against {named}. These solid phases are in {db_tag}.dat but have no matching solid in this lab's registry, so their precipitation is not included. Supersaturation alone does not determine whether or how fast a precipitate forms"),
            ),
            Exclusion::TemperatureWithheld => (
                NotModelledCause::ModelBoundary,
                format!("the solution is supersaturated against {named}, deliberately withheld below the registry's formation-temperature threshold at {:.0} °C. This is a curated metastability boundary, not a computed nucleation or growth rate", temperature_k - 273.15),
            ),
            Exclusion::RegisteredExcluded => (
                NotModelledCause::ModelBoundary,
                format!("the solution is supersaturated against {named}. Matching solids exist in this lab's registry but were not offered in this aqueous equilibrium problem; phase selection, oxidation-state restrictions, or another model's ownership can exclude them. Their formation and its timescale are not predicted by these saturation indices"),
            ),
            Exclusion::GasBoundary => (
                NotModelledCause::ModelBoundary,
                format!("{named} are gas-phase saturation indices relative to the thermodynamic database's reference fugacity, not missing precipitates. Interpret them with this vessel's gas boundary and pressure; these indices alone predict neither bubble formation nor gas-transfer times"),
            ),
            Exclusion::UnknownPhase => (
                NotModelledCause::ModelBoundary,
                format!("{named} have reported saturation indices but no phase definition in the selected {db_tag} diagnostic index; their phase type and exclusion reason cannot be classified"),
            ),
        };
        result.push(Event::NotYetModeled {
            cause,
            vessel,
            what,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_coverage_is_not_candidate_coverage() {
        for tag in ["wateq4f", "minteq.v4", "pitzer"] {
            for (name, phase) in &derived::index_for(tag).phases {
                let expected = if phase.is_gas {
                    Exclusion::GasBoundary
                } else if derived::registry_solid_matching(&phase.composition, phase.waters)
                    .is_some()
                {
                    Exclusion::RegisteredExcluded
                } else {
                    Exclusion::MissingSolid
                };
                assert_eq!(classify(name, tag, 1e6), expected, "{tag}: {name}");
            }
        }
        assert_eq!(
            classify("Hematite", "wateq4f", 298.15),
            Exclusion::RegisteredExcluded
        );
        assert_eq!(classify("N2(g)", "wateq4f", 298.15), Exclusion::GasBoundary);
    }

    #[test]
    fn temperature_boundary_is_checked_not_merely_present() {
        let floor = species::lookup_key("CuO")
            .unwrap()
            .forms_only_above_k
            .unwrap();
        assert_eq!(
            classify("Tenorite", "wateq4f", floor - 1.0),
            Exclusion::TemperatureWithheld
        );
        for temperature in [floor, floor + 1.0, floor + 100.0] {
            assert_eq!(
                classify("Tenorite", "wateq4f", temperature),
                Exclusion::RegisteredExcluded
            );
        }
    }

    #[test]
    fn offered_small_and_invalid_indices_do_not_generate_boundaries() {
        let input = vec![
            ("N2(g)".into(), 5.0),
            ("Hematite".into(), 0.5),
            ("Tenorite".into(), f64::NAN),
            ("unknown".into(), f64::INFINITY),
        ];
        assert!(events(VesselId(0), 298.15, "wateq4f", &input, &["N2(g)"], 1.0).is_empty());
    }

    #[test]
    fn gases_and_registered_solids_are_never_reported_as_missing_precipitates() {
        let input = vec![("N2(g)".into(), 5.0), ("Hematite".into(), 4.0)];
        let output = events(VesselId(0), 298.15, "wateq4f", &input, &[], 1.0);
        assert_eq!(output.len(), 2);
        for event in &output {
            assert!(matches!(
                event,
                Event::NotYetModeled {
                    cause: NotModelledCause::ModelBoundary,
                    ..
                }
            ));
        }
        let serialized = serde_json::to_string(&output).unwrap();
        assert!(serialized.contains("reference fugacity"));
        assert!(serialized.contains("Matching solids exist"));
        assert!(!serialized.contains("would not stay like this"));
    }
}
