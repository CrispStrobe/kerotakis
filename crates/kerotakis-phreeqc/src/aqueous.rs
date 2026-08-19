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
    SpeciesDetail, SpeciesId, ThermalMode, Vessel,
};

use crate::{databases, Phreeqc, PhreeqcError};

use crate::derived::{self, DerivedRole, ATMOSPHERIC};

const WATER_MOLAR_MASS: f64 = 18.015;
const TRACE: f64 = 1e-12;

/// One cached solver result, keyed by database + canonical input.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub rows: Vec<Vec<String>>,
    pub species: Vec<SpeciesDetail>,
}

/// A shippable pre-warmed cache.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CacheData {
    pub entries: Vec<CacheEntry>,
}

pub struct PhreeqcEquilibrator {
    /// wateq4f: inorganic natural-water chemistry, valid to high ionic
    /// strength — the default.
    inorganic: Phreeqc,
    /// minteq.v4: adds organic ligands (acetate), but its activity model is
    /// poor for concentrated brines (halite solubility comes out ~3.7
    /// instead of ~6.1 mol/kgw) — used only when the problem needs organics.
    /// Databases have validity domains; routing by problem is the honest
    /// answer.
    organic: Phreeqc,
    /// pitzer.dat: the specific-ion-interaction model, the right tool for
    /// concentrated brines — but it only knows the major-ion elements.
    brine: Phreeqc,
    /// Content-addressed result cache: same species set, T and P is the
    /// same answer (PLAN.md, P2). Keyed by database + canonical input.
    cache: std::collections::HashMap<String, (Vec<Vec<String>>, Vec<SpeciesDetail>)>,
    cache_hits: usize,
}

impl PhreeqcEquilibrator {
    pub fn new() -> Result<Self, PhreeqcError> {
        Ok(PhreeqcEquilibrator {
            inorganic: Phreeqc::with_database(databases::WATEQ4F)?,
            organic: Phreeqc::with_database(databases::MINTEQ_V4)?,
            brine: Phreeqc::with_database(databases::PITZER)?,
            cache: std::collections::HashMap::new(),
            cache_hits: 0,
        })
    }

    /// Cache hits so far (content-addressed on the canonical PHREEQC input,
    /// which is a deterministic function of the vessel state).
    pub fn cache_hits(&self) -> usize {
        self.cache_hits
    }

    /// Export the cache for shipping (PLAN.md: pre-warmed cache — every
    /// vessel state reachable in the curated lessons computed at build time,
    /// so guided content never waits for a solver, on any device).
    pub fn export_cache(&self) -> CacheData {
        CacheData {
            entries: self
                .cache
                .iter()
                .map(|(k, (rows, species))| CacheEntry {
                    key: k.clone(),
                    rows: rows.clone(),
                    species: species.clone(),
                })
                .collect(),
        }
    }

    /// Load a pre-warmed cache. Entries already present are kept.
    pub fn import_cache(&mut self, data: CacheData) -> usize {
        let before = self.cache.len();
        for e in data.entries {
            self.cache.entry(e.key).or_insert((e.rows, e.species));
        }
        self.cache.len() - before
    }

    /// How many results are cached.
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
}

struct Problem {
    kgw: f64,
    /// Element totals in solution, mol.
    totals: Vec<(String, f64)>,
    /// Phases: (name, initial moles, target saturation index).
    phases: Vec<(String, f64, f64)>,
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
    let mut phases: Vec<(String, f64, f64)> = Vec::new();
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
        match derived::role(&p.species.0)? {
            DerivedRole::Solvent => kgw += p.moles.0 * WATER_MOLAR_MASS / 1000.0,
            DerivedRole::Dissolves(els) => {
                solutes += 1;
                for (el, coeff) in els {
                    add_total(el, p.moles.0 * coeff);
                    note_element(el);
                }
            }
            DerivedRole::Mineral {
                phase,
                elements: els,
            } => {
                solutes += 1;
                for (el, _) in els {
                    note_element(el);
                }
                if p.phase == Phase::Solid {
                    if let Some(entry) = phases.iter_mut().find(|(name, ..)| name == phase) {
                        entry.1 += p.moles.0;
                    } else {
                        phases.push((phase.clone(), p.moles.0, 0.0));
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
    // Every derived candidate phase whose elements can reach solution can
    // precipitate, amount 0 if no solid exists yet.
    for cand in derived::candidate_phases() {
        let all_present = cand
            .elements
            .iter()
            .all(|(el, _)| elements.iter().any(|e| e == el));
        let listed = phases.iter().any(|(name, ..)| name == &cand.name);
        if all_present && !listed {
            phases.push((cand.name.clone(), 0.0, 0.0));
        }
    }
    // Atmospheric venting: a gas phase joins when its non-water elements
    // are present (derived from the gas formula itself).
    for (phase, _, target_si, _) in ATMOSPHERIC {
        let gas_formula = phase.trim_end_matches("(g)");
        let required = crate::dbindex::parse_formula(gas_formula).unwrap_or_default();
        let all_present = required
            .keys()
            .filter(|el| *el != "O" && *el != "H")
            .all(|el| elements.iter().any(|e| e == el));
        let listed = phases.iter().any(|(name, ..)| name == phase);
        if all_present && !listed {
            phases.push((phase.to_string(), 0.0, *target_si));
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
        let Some(mut problem) = partition(vessel) else {
            return Ok(Vec::new());
        };

        // Route by validity domain: minteq.v4 when its extended chemistry
        // is needed — organics (it alone has acetate) or phosphate (wateq4f
        // lacks the free H3PO4 species, so the first proton would come out
        // artificially strong); pitzer for concentrated major-ion brines
        // (the ion-interaction model is built for them: halite saturates at
        // the textbook 6.13 mol/kgw where wateq4f gives 6.50); wateq4f
        // otherwise.
        // "Extended": any element wateq4f does not know (derived — e.g.
        // Acetate), plus phosphate as a documented exception (wateq4f has
        // the P element but no free H3PO4 species).
        let needs_extended = problem
            .elements
            .iter()
            .any(|e| !derived::index_for("wateq4f").has_element(e) || e == "P");
        // Rough concentration estimate: dissolved totals plus what the
        // solid phases could dissolve (each formula unit ~2 ions).
        let potential_molality = (problem.totals.iter().map(|(_, n)| n).sum::<f64>()
            + 2.0 * problem.phases.iter().map(|(_, n, _)| n).sum::<f64>())
            / problem.kgw;
        let pitzer_capable = problem
            .elements
            .iter()
            .all(|el| derived::index_for("pitzer").has_element(el));
        let (engine, db_tag) = if needs_extended {
            (&mut self.organic, "minteq.v4")
        } else if potential_molality > 1.0 && pitzer_capable {
            (&mut self.brine, "pitzer")
        } else {
            (&mut self.inorganic, "wateq4f")
        };
        // Phases the routed database does not define must not reach the
        // input. Zero-amount candidates are dropped; a solid-backed
        // anhydrous phase (e.g. solid KCl on the wateq4f route — Sylvite is
        // pitzer-only) dissolves into element totals instead, i.e. is
        // treated as freely soluble in that database's domain. Hydrate
        // phases are kept so the engine errors honestly rather than the
        // ledger losing their crystal water silently.
        let idx = derived::index_for(db_tag);
        let mut freed: Vec<(String, f64)> = Vec::new();
        problem.phases.retain(|(name, moles, _)| {
            if idx.has_phase(name) {
                return true;
            }
            if *moles > 0.0 {
                if let Some(p) = derived::phase_by_name(name) {
                    if p.waters == 0.0 {
                        for (el, c) in &p.elements {
                            freed.push((el.clone(), c * moles));
                        }
                        return false;
                    }
                }
                return true;
            }
            false
        });
        for (el, n) in freed {
            if let Some(entry) = problem.totals.iter_mut().find(|(e, _)| *e == el) {
                entry.1 += n;
            } else {
                problem.totals.push((el, n));
            }
        }
        let input = build_input(vessel, &problem);
        let key = format!("#{db_tag}\n{input}");

        // Content-addressed cache: database + input string is a
        // deterministic canonicalisation of (species set, amounts, T) — same
        // state, same answer, no engine call.
        let (rows, speciation) = if let Some(hit) = self.cache.get(&key) {
            self.cache_hits += 1;
            hit.clone()
        } else {
            engine.run(&input).map_err(|e| SolveError::NotConverged {
                solver: "phreeqc-aqueous".to_string(),
                detail: e.to_string(),
            })?;
            let rows = engine.selected_output();
            let speciation = parse_species_distribution(&engine.output_string());
            if self.cache.len() >= 10_000 {
                self.cache.clear(); // simple bound; refine when profiling says so
            }
            self.cache.insert(key, (rows.clone(), speciation.clone()));
            (rows, speciation)
        };
        let value = |column: &str| -> Option<f64> {
            let idx = rows.first()?.iter().position(|h| h == column)?;
            rows.last()?.get(idx)?.parse().ok()
        };

        // Read back: element totals (mol/kgw) and phase amounts (mol).
        // Molalities are per kg of *equilibrated* water (mass_H2O), which
        // differs slightly from the input water mass through speciation.
        let kgw_out = value("mass_H2O").ok_or_else(|| missing("mass_H2O"))?;
        let mut new_ions: Vec<(String, f64)> = Vec::new();
        for el in &problem.elements {
            let molality = value(el).ok_or_else(|| missing(el))?;
            new_ions.push((el.clone(), molality * kgw_out));
        }
        let mut new_phases: Vec<(String, f64)> = Vec::new();
        for (phase, ..) in &problem.phases {
            let moles = value(phase).ok_or_else(|| missing(phase))?;
            new_phases.push((phase.clone(), moles));
        }
        let ph = value("pH").ok_or_else(|| missing("pH"))?;
        let mu = value("mu").ok_or_else(|| missing("mu"))?;

        // Rebuild the vessel inventory: water stays; solutes are replaced by
        // the computed state.

        let mut events = Vec::new();
        let mut contents = Vec::new();
        for p in &vessel.contents {
            if matches!(derived::role(&p.species.0), Some(DerivedRole::Solvent)) {
                contents.push(p.clone());
            }
        }
        for (el, moles) in &new_ions {
            if *moles > TRACE {
                let ion = derived::booking_ion(el).expect("booking ion covered by tests");
                contents.push(Portion {
                    species: SpeciesId::new(ion),
                    moles: Moles(*moles),
                    phase: Phase::Aqueous,
                });
            }
        }
        for (phase, moles) in &new_phases {
            if let Some(dp) = derived::phase_by_name(phase) {
                let (species, waters) = (dp.species, dp.waters);
                // Baseline is the phase's INPUT amount, not the vessel's
                // solids: a freely-soluble solid (e.g. KCl) contributes to
                // the totals, not the phase, and comparing against vessel
                // solids double-counted its dissolution (and its heat).
                let before = problem
                    .phases
                    .iter()
                    .find(|(name, ..)| name == phase)
                    .map(|(_, m, _)| *m)
                    .unwrap_or(0.0);
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
                // Waters of crystallisation move between the liquid and the
                // solid (gypsum: 2 H2O per formula unit).
                if waters != 0.0 && delta.abs() > TRACE {
                    if let Some(w) = contents
                        .iter_mut()
                        .find(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
                    {
                        w.moles = Moles((w.moles.0 - waters * delta).max(0.0));
                    }
                }
            } else if let Some((_, gas, _, water_coproduct)) =
                ATMOSPHERIC.iter().find(|(name, ..)| name == phase)
            {
                // Escaped the open vessel: reported, not booked — the
                // balance notices the loss. The water co-product of the
                // gas-forming reaction stays behind.
                if *moles > TRACE {
                    events.push(Event::GasEvolved {
                        vessel: vessel.id,
                        species: SpeciesId::new(gas),
                        moles: Moles(*moles),
                    });
                    let formed_water = moles * water_coproduct;
                    if formed_water > TRACE {
                        if let Some(w) = contents
                            .iter_mut()
                            .find(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
                        {
                            w.moles = Moles(w.moles.0 + formed_water);
                        }
                    }
                }
            }
        }
        // Freely-soluble solids (no mineral phase) dissolved entirely.
        for p in &vessel.contents {
            if p.phase == Phase::Solid
                && matches!(derived::role(&p.species.0), Some(DerivedRole::Dissolves(_)))
            {
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
            species: speciation,
        };
        let changed = vessel
            .solution
            .as_ref()
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
        for (phase, moles, target_si) in &problem.phases {
            writeln!(input, "    {phase} {target_si} {moles:.12e}").unwrap();
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
        let phases: Vec<&str> = problem.phases.iter().map(|(p, ..)| p.as_str()).collect();
        writeln!(input, "    -equilibrium_phases {}", phases.join(" ")).unwrap();
    }
    writeln!(input, "END").unwrap();
    input
}

/// Parse the last "Distribution of species" block of a PHREEQC output
/// report into (name, molality, activity) triples, molality > 1e-9,
/// descending. The block's shape is stable across PHREEQC 3.x: a header,
/// element-total lines (2 columns), and species lines (>= 6 columns:
/// name, molality, activity, log m, log a, log gamma[, volume]).
fn parse_species_distribution(output: &str) -> Vec<SpeciesDetail> {
    let Some(start) = output.rfind("Distribution of species") else {
        return Vec::new();
    };
    let mut result: Vec<SpeciesDetail> = Vec::new();
    for line in output[start..].lines().skip(1) {
        let trimmed = line.trim();
        // The next report section begins with a dashed rule.
        if trimmed.starts_with("-----") {
            break;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 6 {
            continue;
        }
        let (Ok(molality), Ok(activity)) = (tokens[1].parse::<f64>(), tokens[2].parse::<f64>())
        else {
            continue;
        };
        // Log columns must also parse, or this is a header/stray line.
        if tokens[3].parse::<f64>().is_err() {
            continue;
        }
        // A species appears once per element section it contains (AgCl is
        // listed under both Ag and Cl); keep it once.
        if molality > 1e-9 && !result.iter().any(|r| r.name == tokens[0]) {
            result.push(SpeciesDetail {
                name: tokens[0].to_string(),
                molality,
                activity,
            });
        }
    }
    result.sort_by(|a, b| b.molality.total_cmp(&a.molality));
    result
}
