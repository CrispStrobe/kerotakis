//! ARCH-014: Coverage manifest — for every registered operation and species
//! family, report claimed models, validity, observables, and unsupported
//! dimensions.

use serde::Serialize;

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
}

/// Generate a coverage manifest by querying each solver in the stack.
pub fn coverage_manifest(
    solvers: &[&dyn Equilibrator],
    vessel: &Vessel,
) -> CoverageReport {
    let solver_reports: Vec<SolverCoverage> = solvers
        .iter()
        .map(|s| {
            let cap = s.capability(vessel);
            SolverCoverage {
                solver_name: s.name(),
                applicable: cap.applicability.is_applicable(),
                is_chemistry: cap.is_chemistry,
                validity_notes: cap
                    .validity
                    .map(|v| format!("{v:?}")),
            }
        })
        .collect();

    let any_chemistry = solver_reports.iter().any(|s| s.applicable && s.is_chemistry);
    let mut uncovered = Vec::new();
    if !any_chemistry {
        uncovered.push("No chemistry solver applicable for this vessel state".into());
    }

    CoverageReport {
        species_count: species::REGISTRY.len(),
        solvers: solver_reports,
        uncovered,
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
