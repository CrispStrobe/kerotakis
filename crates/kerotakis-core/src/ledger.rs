//! ConservedLedger — shadow-mode conservation auditing (ARCH-003).
//!
//! A `ConservedLedger` snapshots the conserved quantities of a vessel
//! (element totals, total mass, total charge, sensible energy) and can
//! compare two snapshots to detect conservation violations.
//!
//! In shadow mode, the ledger observes but does not change behavior.
//! Violations are collected as diagnostics, not panics.

use std::collections::BTreeMap;

use crate::species;
use crate::stoich;
use crate::vessel::Vessel;

/// A snapshot of conserved quantities for one vessel.
#[derive(Debug, Clone)]
pub struct ConservedLedger {
    /// Element totals in moles, keyed by element symbol.
    pub elements: BTreeMap<String, f64>,
    /// Total mass in grams.
    pub mass: f64,
    /// Total charge in elementary charges (sum of moles * charge per formula).
    pub charge: f64,
    /// Sensible energy in joules (enthalpy relative to standard state).
    pub energy: f64,
}

/// A conservation violation detected between two ledger snapshots.
#[derive(Debug, Clone)]
pub struct Violation {
    pub quantity: String,
    pub before: f64,
    pub after: f64,
    pub delta: f64,
    pub relative: f64,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {:.6e} → {:.6e} (Δ = {:.3e}, {:.1e} relative)",
            self.quantity, self.before, self.after, self.delta, self.relative
        )
    }
}

impl ConservedLedger {
    /// Snapshot the conserved quantities of a vessel.
    pub fn from_vessel(vessel: &Vessel) -> Self {
        let mut elements: BTreeMap<String, f64> = BTreeMap::new();
        let mut charge = 0.0;

        // Main contents
        for portion in &vessel.contents {
            if let Some(data) = species::lookup(&portion.species) {
                if let Ok(formula) = stoich::parse_formula(data.formula) {
                    for (el, count) in &formula.counts {
                        *elements.entry(el.clone()).or_insert(0.0) += count * portion.moles.0;
                    }
                    charge += formula.charge * portion.moles.0;
                }
            }
        }

        // Surface sites (sorbates)
        for surface in &vessel.surfaces {
            for occ in &surface.occupancy {
                let sid = occ.sorbate.species();
                if let Some(data) = species::lookup(&sid) {
                    if let Ok(formula) = stoich::parse_formula(data.formula) {
                        for (el, count) in &formula.counts {
                            *elements.entry(el.clone()).or_insert(0.0) += count * occ.moles.0;
                        }
                        charge += formula.charge * occ.moles.0;
                    }
                }
            }
        }

        // Exchange sites (bound cations)
        for exchanger in &vessel.exchanges {
            for occ in &exchanger.occupancy {
                let sid = occ.ion.species();
                if let Some(data) = species::lookup(&sid) {
                    if let Ok(formula) = stoich::parse_formula(data.formula) {
                        for (el, count) in &formula.counts {
                            *elements.entry(el.clone()).or_insert(0.0) += count * occ.moles.0;
                        }
                        charge += formula.charge * occ.moles.0;
                    }
                }
            }
        }

        // Solid solutions
        for ss in &vessel.solid_solutions {
            for component in &ss.components {
                let sid = component.component.species();
                if let Some(data) = species::lookup(&sid) {
                    if let Ok(formula) = stoich::parse_formula(data.formula) {
                        for (el, count) in &formula.counts {
                            *elements.entry(el.clone()).or_insert(0.0) += count * component.moles.0;
                        }
                        charge += formula.charge * component.moles.0;
                    }
                }
            }
        }

        Self {
            elements,
            mass: vessel.mass().0,
            charge,
            energy: vessel.enthalpy().0,
        }
    }

    /// Check conservation between this ledger (before) and another (after).
    /// Returns a list of violations that exceed the given tolerance.
    ///
    /// `tolerance` is the maximum acceptable relative change for each quantity.
    /// Element amounts below `floor_moles` are excluded (numerical noise).
    pub fn check_against(
        &self,
        after: &ConservedLedger,
        tolerance: f64,
        floor_moles: f64,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check element conservation
        let mut all_elements: Vec<&String> =
            self.elements.keys().chain(after.elements.keys()).collect();
        all_elements.sort();
        all_elements.dedup();

        for el in all_elements {
            let before = self.elements.get(el).copied().unwrap_or(0.0);
            let after_val = after.elements.get(el).copied().unwrap_or(0.0);
            if before.abs() < floor_moles && after_val.abs() < floor_moles {
                continue;
            }
            let delta = after_val - before;
            let denom = before.abs().max(floor_moles);
            let relative = delta.abs() / denom;
            if relative > tolerance {
                violations.push(Violation {
                    quantity: format!("element:{}", el),
                    before,
                    after: after_val,
                    delta,
                    relative,
                });
            }
        }

        // Check mass conservation
        if self.mass > floor_moles {
            let delta = after.mass - self.mass;
            let relative = delta.abs() / self.mass.max(floor_moles);
            if relative > tolerance {
                violations.push(Violation {
                    quantity: "mass".to_string(),
                    before: self.mass,
                    after: after.mass,
                    delta,
                    relative,
                });
            }
        }

        // Check charge conservation
        {
            let delta = after.charge - self.charge;
            let denom = self.charge.abs().max(1e-15);
            let relative = delta.abs() / denom;
            if relative > tolerance && delta.abs() > floor_moles {
                violations.push(Violation {
                    quantity: "charge".to_string(),
                    before: self.charge,
                    after: after.charge,
                    delta,
                    relative,
                });
            }
        }

        violations
    }
}

/// Convenience: snapshot a vessel, run an operation, snapshot again, check.
/// Returns violations (empty = conserved). Does not panic — shadow mode.
pub fn audit_conservation(
    vessel_before: &Vessel,
    vessel_after: &Vessel,
    tolerance: f64,
) -> Vec<Violation> {
    let before = ConservedLedger::from_vessel(vessel_before);
    let after = ConservedLedger::from_vessel(vessel_after);
    before.check_against(&after, tolerance, 1e-15)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::{Phase, SpeciesId};
    use crate::units::{Kelvin, Moles};
    use crate::vessel::{Vessel, VesselId};

    fn test_vessel() -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin(298.15);
        v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
        v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
        v
    }

    #[test]
    fn ledger_from_vessel_has_elements() {
        let v = test_vessel();
        let ledger = ConservedLedger::from_vessel(&v);
        // Water = H2O: 5.5 mol → H: 11.0, O: 5.5
        // NaCl: 0.1 mol → Na: 0.1, Cl: 0.1
        assert!(ledger.elements.contains_key("H"));
        assert!(ledger.elements.contains_key("O"));
        assert!(ledger.elements.contains_key("Na"));
        assert!(ledger.elements.contains_key("Cl"));
        assert!((ledger.elements["H"] - 11.0).abs() < 1e-10);
        assert!((ledger.elements["O"] - 5.5).abs() < 1e-10);
        assert!((ledger.elements["Na"] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn identical_vessels_have_no_violations() {
        let v = test_vessel();
        let violations = audit_conservation(&v, &v, 1e-12);
        assert!(violations.is_empty(), "violations: {:?}", violations);
    }

    #[test]
    fn adding_matter_shows_violation() {
        let before = test_vessel();
        let mut after = before.clone();
        after.deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Aqueous);
        let violations = audit_conservation(&before, &after, 1e-9);
        assert!(
            !violations.is_empty(),
            "should detect element increase from added NaCl"
        );
        assert!(violations.iter().any(|v| v.quantity == "element:Na"));
        assert!(violations.iter().any(|v| v.quantity == "element:Cl"));
    }

    #[test]
    fn kinetics_conserves_elements() {
        // Advance the curated network and verify element conservation
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin(298.15);
        v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
        v.deposit(SpeciesId::new("Na2S2O3"), Moles(0.1), Phase::Aqueous);
        v.solution = Some(crate::vessel::SolutionInfo {
            redox: Vec::new(),
            pe: None,
            ph: 1.7,
            ionic_strength: 0.02,
            species: Vec::new(),
            provenance: None,
        });

        let before = ConservedLedger::from_vessel(&v);
        let _ = crate::kinetics::advance(&mut v, 1.0);
        let after = ConservedLedger::from_vessel(&v);

        // Element conservation should be tight; mass can drift slightly
        // because Vessel::mass() uses SpeciesData.molar_mass (fixed table)
        // while the ledger's element totals use parse_formula atomic weights.
        let violations = before.check_against(&after, 1e-7, 1e-15);
        let real_violations: Vec<_> = violations
            .iter()
            .filter(|v| !v.quantity.starts_with("mass") || v.relative > 1e-6)
            .collect();
        assert!(
            real_violations.is_empty(),
            "kinetics violated conservation: {:?}",
            real_violations
        );
    }
}
