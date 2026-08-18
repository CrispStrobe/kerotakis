//! The L2 aqueous equilibrator: maps a vessel's contents into a PHREEQC
//! problem, equilibrates against the thermodynamic database, and maps the
//! result back — dissolution, precipitation, pH and ionic strength, derived
//! rather than hardcoded.
//!
//! v1 inventory model: dissolved matter is tracked as element-total ions
//! (Na⁺, Cl⁻, Ag⁺, NO₃⁻) — exact for conservation, approximate as speciation
//! (complexes like AgCl(aq) are inside PHREEQC's solution, and the expert
//! register can surface them later). Solids with a known mineral phase enter
//! as amount-limited `EQUILIBRIUM_PHASES`, so partial dissolution at the
//! solubility limit is computed, not scripted.
//!
//! Known limitation, stated: dissolution/precipitation enthalpy is not yet
//! fed into the vessel's energy balance (curated ΔH arrives with the codex).

use kerotakis_core::{
    species, Equilibrator, Event, Kelvin, Moles, Phase, Portion, SolutionInfo, SolveError,
    SpeciesId, ThermalMode, Vessel,
};

use crate::{databases, Phreeqc, PhreeqcError};

/// How a registry species participates in the aqueous problem.
enum Role {
    Solvent,
    /// Contributes element totals when dissolved or freely soluble.
    Dissolves(&'static [(&'static str, f64)]),
    /// A solid with a mineral phase in the database: enters amount-limited.
    Mineral {
        phase: &'static str,
        elements: &'static [(&'static str, f64)],
    },
}

/// The v1 aqueous-mapping table over the seed registry. Grows with L1.
fn role(key: &str) -> Option<Role> {
    match key {
        "water" => Some(Role::Solvent),
        "NaCl" => Some(Role::Mineral {
            phase: "Halite",
            elements: &[("Na", 1.0), ("Cl", 1.0)],
        }),
        "AgCl" => Some(Role::Mineral {
            phase: "Cerargyrite",
            elements: &[("Ag", 1.0), ("Cl", 1.0)],
        }),
        // Very soluble; no phase needed at teaching concentrations.
        "AgNO3" => Some(Role::Dissolves(&[("Ag", 1.0), ("N(5)", 1.0)])),
        // Strong acid/base: only the counter-ion enters the element totals;
        // the H+/OH- side emerges from PHREEQC's charge balance (`pH 7
        // charge`), which is exactly how a strong-acid solution is defined.
        // The ledger's ion imbalance carries the acidity across steps.
        "HCl" => Some(Role::Dissolves(&[("Cl", 1.0)])),
        "NaOH" => Some(Role::Dissolves(&[("Na", 1.0)])),
        "Na+" => Some(Role::Dissolves(&[("Na", 1.0)])),
        "Cl-" => Some(Role::Dissolves(&[("Cl", 1.0)])),
        "Ag+" => Some(Role::Dissolves(&[("Ag", 1.0)])),
        "NO3-" => Some(Role::Dissolves(&[("N(5)", 1.0)])),
        _ => None,
    }
}

/// Element total → the ion it is booked as in the vessel inventory.
fn element_ion(element: &str) -> Option<&'static str> {
    match element {
        "Na" => Some("Na+"),
        "Cl" => Some("Cl-"),
        "Ag" => Some("Ag+"),
        "N(5)" => Some("NO3-"),
        _ => None,
    }
}

/// Mineral phase → the solid species it is booked as.
fn phase_species(phase: &str) -> Option<&'static str> {
    match phase {
        "Halite" => Some("NaCl"),
        "Cerargyrite" => Some("AgCl"),
        _ => None,
    }
}

/// Phases that can precipitate when their elements are present.
const CANDIDATE_PHASES: &[(&str, &[&str])] =
    &[("Halite", &["Na", "Cl"]), ("Cerargyrite", &["Ag", "Cl"])];

const WATER_MOLAR_MASS: f64 = 18.015;
const TRACE: f64 = 1e-12;

pub struct PhreeqcEquilibrator {
    engine: Phreeqc,
}

impl PhreeqcEquilibrator {
    /// Uses the embedded wateq4f database (the seed species need Ag).
    pub fn new() -> Result<Self, PhreeqcError> {
        Ok(PhreeqcEquilibrator {
            engine: Phreeqc::with_database(databases::WATEQ4F)?,
        })
    }
}

struct Problem {
    kgw: f64,
    /// Element totals in solution, mol.
    totals: Vec<(String, f64)>,
    /// Mineral phases present as solids, mol.
    phases: Vec<(String, f64)>,
    /// Every element to read back: dissolved totals plus the elements of all
    /// involved phases (a dissolving solid puts its elements into solution
    /// even when none started there).
    elements: Vec<String>,
}

/// Partition the vessel into a PHREEQC problem, or None if this vessel is
/// not an aqueous problem this mapper fully understands.
fn partition(vessel: &Vessel) -> Option<Problem> {
    let mut kgw = 0.0;
    let mut totals: Vec<(String, f64)> = Vec::new();
    let mut phases: Vec<(String, f64)> = Vec::new();
    let mut solutes = 0;

    let mut add_total = |el: &str, moles: f64| {
        if let Some(entry) = totals.iter_mut().find(|(e, _)| e == el) {
            entry.1 += moles;
        } else {
            totals.push((el.to_string(), moles));
        }
    };

    let mut elements: Vec<String> = Vec::new();
    let mut note_element = |el: &str| {
        if !elements.iter().any(|e| e == el) {
            elements.push(el.to_string());
        }
    };

    for p in &vessel.contents {
        match role(&p.species.0)? {
            Role::Solvent => kgw += p.moles.0 * WATER_MOLAR_MASS / 1000.0,
            Role::Dissolves(els) => {
                solutes += 1;
                for (el, coeff) in els {
                    add_total(el, p.moles.0 * coeff);
                    note_element(el);
                }
            }
            Role::Mineral {
                phase,
                elements: els,
            } => {
                solutes += 1;
                for (el, _) in els {
                    note_element(el);
                }
                if p.phase == Phase::Solid {
                    if let Some(entry) = phases.iter_mut().find(|(name, _)| name == phase) {
                        entry.1 += p.moles.0;
                    } else {
                        phases.push((phase.to_string(), p.moles.0));
                    }
                } else {
                    for (el, coeff) in els {
                        add_total(el, p.moles.0 * coeff);
                    }
                }
            }
        }
    }

    if kgw <= 0.0 || solutes == 0 {
        return None;
    }
    // Ensure every candidate phase whose elements can reach solution can
    // precipitate, amount 0 if no solid exists yet.
    for (phase, required) in CANDIDATE_PHASES {
        let all_present = required.iter().all(|el| elements.iter().any(|e| e == el));
        let listed = phases.iter().any(|(name, _)| name == phase);
        if all_present && !listed {
            phases.push((phase.to_string(), 0.0));
        }
    }
    Some(Problem {
        kgw,
        totals,
        phases,
        elements,
    })
}

impl Equilibrator for PhreeqcEquilibrator {
    fn name(&self) -> &'static str {
        "phreeqc-aqueous"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        partition(vessel).is_some()
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let Some(problem) = partition(vessel) else {
            return Ok(Vec::new());
        };

        let input = build_input(vessel, &problem);
        self.engine
            .run(&input)
            .map_err(|e| SolveError::NotConverged {
                solver: "phreeqc-aqueous".to_string(),
                detail: e.to_string(),
            })?;

        // Read back: element totals (mol/kgw) and phase amounts (mol).
        // Molalities are per kg of *equilibrated* water (mass_H2O), which
        // differs slightly from the input water mass through speciation.
        let kgw_out = self
            .engine
            .last_value("mass_H2O")
            .ok_or_else(|| missing("mass_H2O"))?;
        let mut new_ions: Vec<(String, f64)> = Vec::new();
        for el in &problem.elements {
            let molality = self.engine.last_value(el).ok_or_else(|| missing(el))?;
            new_ions.push((el.clone(), molality * kgw_out));
        }
        let mut new_phases: Vec<(String, f64)> = Vec::new();
        for (phase, _) in &problem.phases {
            let moles = self
                .engine
                .last_value(phase)
                .ok_or_else(|| missing(phase))?;
            new_phases.push((phase.clone(), moles));
        }
        let ph = self.engine.last_value("pH").ok_or_else(|| missing("pH"))?;
        let mu = self.engine.last_value("mu").ok_or_else(|| missing("mu"))?;

        // Rebuild the vessel inventory: water stays; solutes are replaced by
        // the computed state.
        let old_solid = |species: &str| -> f64 {
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == species && p.phase == Phase::Solid)
                .map(|p| p.moles.0)
                .sum()
        };
        let mut events = Vec::new();
        let mut contents = Vec::new();
        for p in &vessel.contents {
            if matches!(role(&p.species.0), Some(Role::Solvent)) {
                contents.push(p.clone());
            }
        }
        for (el, moles) in &new_ions {
            if *moles > TRACE {
                let ion = element_ion(el).expect("mapped element");
                contents.push(Portion {
                    species: SpeciesId::new(ion),
                    moles: Moles(*moles),
                    phase: Phase::Aqueous,
                });
            }
        }
        for (phase, moles) in &new_phases {
            let species = phase_species(phase).expect("mapped phase");
            let before = old_solid(species);
            if *moles > TRACE {
                contents.push(Portion {
                    species: SpeciesId::new(species),
                    moles: Moles(*moles),
                    phase: Phase::Solid,
                });
            }
            let delta = moles - before;
            if delta > TRACE {
                events.push(Event::Precipitated {
                    vessel: vessel.id,
                    species: SpeciesId::new(species),
                    moles: Moles(delta),
                });
            } else if delta < -TRACE {
                events.push(Event::Dissolved {
                    vessel: vessel.id,
                    species: SpeciesId::new(species),
                    moles: Moles(-delta),
                });
            }
        }
        // Freely-soluble solids (no mineral phase) dissolved entirely.
        for p in &vessel.contents {
            if p.phase == Phase::Solid && matches!(role(&p.species.0), Some(Role::Dissolves(_))) {
                events.push(Event::Dissolved {
                    vessel: vessel.id,
                    species: p.species.clone(),
                    moles: p.moles,
                });
            }
        }

        vessel.contents = contents;

        // Reaction heat: curated dissolution enthalpies feed the energy
        // balance (PLAN.md). Dissolution of an endothermic salt cools the
        // vessel; precipitation releases the corresponding heat. v1 applies
        // the temperature change once rather than iterating solver ↔ T; the
        // shifts at teaching concentrations are small against the ~25–100 °C
        // range of the database.
        if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
            let mut q_joules = 0.0; // heat released into the vessel
            for e in &events {
                match e {
                    Event::Dissolved {
                        species: sid,
                        moles,
                        ..
                    } => {
                        if let Some(dh) =
                            species::lookup(sid).and_then(|d| d.dissolution_enthalpy_kj)
                        {
                            q_joules -= dh * 1000.0 * moles.0;
                        }
                    }
                    Event::Precipitated {
                        species: sid,
                        moles,
                        ..
                    } => {
                        if let Some(dh) =
                            species::lookup(sid).and_then(|d| d.dissolution_enthalpy_kj)
                        {
                            q_joules += dh * 1000.0 * moles.0;
                        }
                    }
                    _ => {}
                }
            }
            let cp = vessel.heat_capacity();
            if q_joules.abs() > 1e-9 && cp > 0.0 {
                let from = vessel.temperature;
                let to = Kelvin((from.0 + q_joules / cp).max(0.0));
                vessel.temperature = to;
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from,
                    to,
                });
            }
        }

        let info = SolutionInfo {
            ph,
            ionic_strength: mu,
        };
        let changed = vessel
            .solution
            .map(|prev| (prev.ph - ph).abs() > 0.01 || (prev.ionic_strength - mu).abs() > 1e-4)
            .unwrap_or(true);
        vessel.solution = Some(info);
        if changed {
            events.push(Event::SolutionCharacterized {
                vessel: vessel.id,
                ph,
                ionic_strength: mu,
            });
        }
        Ok(events)
    }
}

fn missing(column: &str) -> SolveError {
    SolveError::NotConverged {
        solver: "phreeqc-aqueous".to_string(),
        detail: format!("selected output lacks column '{column}'"),
    }
}

fn build_input(vessel: &Vessel, problem: &Problem) -> String {
    use std::fmt::Write;
    let mut input = String::new();
    let temp_c = vessel.temperature.to_celsius();
    writeln!(input, "SOLUTION 1").unwrap();
    writeln!(input, "    units     mol/kgw").unwrap();
    writeln!(input, "    temp      {temp_c:.4}").unwrap();
    writeln!(input, "    pH        7  charge").unwrap();
    writeln!(input, "    water     {:.9}", problem.kgw).unwrap();
    for (el, moles) in &problem.totals {
        writeln!(input, "    {el} {:.12e}", moles / problem.kgw).unwrap();
    }
    if !problem.phases.is_empty() {
        writeln!(input, "EQUILIBRIUM_PHASES 1").unwrap();
        for (phase, moles) in &problem.phases {
            writeln!(input, "    {phase} 0 {moles:.12e}").unwrap();
        }
    }
    writeln!(input, "SELECTED_OUTPUT").unwrap();
    writeln!(input, "    -reset    false").unwrap();
    // Default selected-output prints ~5 significant digits, which leaks into
    // the mass balance; high precision prints 12.
    writeln!(input, "    -high_precision true").unwrap();
    writeln!(input, "    -ph       true").unwrap();
    writeln!(input, "    -ionic_strength true").unwrap();
    writeln!(input, "    -water    true").unwrap();
    let elements: Vec<&str> = problem.elements.iter().map(String::as_str).collect();
    writeln!(input, "    -totals   {}", elements.join(" ")).unwrap();
    if !problem.phases.is_empty() {
        let phases: Vec<&str> = problem.phases.iter().map(|(p, _)| p.as_str()).collect();
        writeln!(input, "    -equilibrium_phases {}", phases.join(" ")).unwrap();
    }
    writeln!(input, "END").unwrap();
    input
}
