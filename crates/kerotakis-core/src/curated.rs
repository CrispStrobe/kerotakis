//! The seed of L4: curated reactions, hand-verified, applied at the
//! compound level. This is deliberately the *first* chemistry stage in the
//! stack — curated knowledge outranks generic solvers where it exists
//! (PLAN.md, "L4 is a cascade").
//!
//! Gas products leave a reservoir/swept boundary but remain in a materially
//! closed headspace. The balance notices either way, which makes conservation
//! of mass across the boundary explicit. Reaction enthalpies for these entries
//! are not yet curated; no heat effect is applied (honestly).

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

pub struct CuratedReaction {
    /// Shown at student register and above.
    pub equation: &'static str,
    pub reactants: &'static [(&'static str, f64)],
    /// Products with the phase they appear in. `Phase::Gas` products escape
    /// an external boundary or remain in a material-closed headspace.
    pub products: &'static [(&'static str, f64, Phase)],
    /// When set, this reaction fires only in a single-organic-solvent
    /// bench of the named solvent (no water). Extent is computed from
    /// the dissolved fraction only — undissolved solid on the bottom
    /// does not participate.
    pub solvent: Option<&'static str>,
    /// When set, the reaction fires only when the vessel temperature
    /// is at or above this threshold (in kelvin).
    pub min_temp_k: Option<f64>,
    /// When set, this species must be present for the reaction to fire,
    /// but is NOT consumed (enzyme/catalyst).
    pub catalyst: Option<&'static str>,
}

/// Hand-verified seed set. Grows into the codex (P4).
pub const REACTIONS: &[CuratedReaction] = &[
    // Keep calcium acetate as the installed aqueous ions rather than
    // inventing a molecular pseudo-species. A later aqueous solver can then
    // speciate the same conserved inventory without replaying this reaction.
    // The route claims limiting-reagent matter and gas, not an instantaneous
    // rate or reaction heat; particle area and an audited enthalpy remain open.
    CuratedReaction {
        equation: "CaCO₃ + 2 CH₃COOH → Ca²⁺ + 2 CH₃COO⁻ + H₂O + CO₂↑",
        reactants: &[("CaCO3", 1.0), ("CH3COOH", 2.0)],
        products: &[
            ("Ca+2", 1.0, Phase::Aqueous),
            ("CH3COO-", 2.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
            ("CO2", 1.0, Phase::Gas),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // ── familiar carbonate fizz (BRD-014) ─────────────────────────
    // Molecular bookkeeping for the observable household reaction. Sodium
    // acetate remains dissolved while carbon dioxide crosses the open vessel
    // boundary (or enters a sealed headspace). No heat effect is claimed
    // until a reviewed reaction enthalpy is installed.
    CuratedReaction {
        equation: "NaHCO₃ + CH₃COOH → CH₃COONa + H₂O + CO₂↑",
        reactants: &[("NaHCO3", 1.0), ("CH3COOH", 1.0)],
        products: &[
            ("NaOAc", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
            ("CO2", 1.0, Phase::Gas),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // The same fizz, written in the names the beaker holds once the soda
    // has dissolved — the `MnO₄⁻`-for-`KMnO4` pattern, one row down.
    //
    // Renaming is symmetric, and the acetate protonation split only fixed
    // the acid end of it. Put water in the vessel first and the bicarbonate
    // dissolves; the readback then books its carbon as `HCO3-` (the
    // documented teaching-pH protonation choice) and its sodium as `Na+`,
    // so the reactant named `NaHCO3` above is no longer in the vessel and
    // its reaction is unreachable from that end. Add the acid first and it
    // fires; add the soda first and it did not. Order dependence is worse
    // than absence: "put the solid in first" is a workaround somebody finds
    // by accident and never understands.
    //
    // Written on the ion rather than the salt because that is what is
    // actually there, and the sodium is a spectator — which is why it does
    // not appear on either side. Any bicarbonate reaches this row, not only
    // bicarbonate that arrived as baking soda, and that is correct: acid
    // poured into a bicarbonate solution fizzes however the bicarbonate got
    // there.
    //
    // No heat effect claimed, on the same terms as the row above. The
    // enthalpy of the acid-carbonate reaction is not held anywhere in this
    // lab; the aqueous tail used to supply one by mistake, charging it at
    // the strong-acid-strong-base figure, and that has been withdrawn
    // rather than replaced.
    CuratedReaction {
        equation: "HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑",
        reactants: &[("HCO3-", 1.0), ("CH3COOH", 1.0)],
        products: &[
            ("CH3COO-", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
            ("CO2", 1.0, Phase::Gas),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "NH3 + NaOCl → NH2Cl↑ + NaOH",
        reactants: &[("NH3", 1.0), ("NaOCl", 1.0)],
        products: &[("NH2Cl", 1.0, Phase::Gas), ("NaOH", 1.0, Phase::Aqueous)],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "NaOCl + 2 HCl → Cl2↑ + NaCl + H2O",
        reactants: &[("NaOCl", 1.0), ("HCl", 2.0)],
        products: &[
            ("Cl2", 1.0, Phase::Gas),
            ("NaCl", 1.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "4 KMnO4 + 3 C₂H₅OH → 4 MnO₂↓ + 3 CH₃COOH + 4 KOH + H₂O",
        reactants: &[("KMnO4", 4.0), ("ethanol", 3.0)],
        products: &[
            ("MnO2", 4.0, Phase::Solid),
            ("CH3COOH", 3.0, Phase::Liquid),
            ("KOH", 4.0, Phase::Liquid),
            ("water", 1.0, Phase::Liquid),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "4 MnO₄⁻ + 3 C₂H₅OH → 4 MnO₂↓ + 3 CH₃COOH + 4 OH⁻ + H₂O",
        reactants: &[("MnO4-", 4.0), ("ethanol", 3.0)],
        products: &[
            ("MnO2", 4.0, Phase::Solid),
            ("CH3COOH", 3.0, Phase::Aqueous),
            ("OH-", 4.0, Phase::Aqueous),
            ("water", 1.0, Phase::Liquid),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // ── EXP-39: the permanganate–oxalate standardisation ──────────
    //
    // The classic primary standard, and the one titration in the school
    // canon whose endpoint you can see without an indicator. It rides a
    // curated row rather than the aqueous engine for a stated reason:
    // NO shipped PHREEQC database speciates oxalate. phreeqc.dat,
    // wateq4f.dat, minteq.v4.dat and pitzer.dat carry no oxalate master
    // species, no solution species, nothing — the only vendored database
    // that spells C2O4-2 at all is llnl_organics.dat, which is not
    // shipped and which writes oxalate as a redox reaction off acetate
    // and O2 with no master species of its own. So the coupled solve
    // cannot find this reaction, and pretending otherwise would be worse
    // than curating it.
    //
    // The textbook equation is the acidic one:
    //
    //     2 MnO4- + 5 H2C2O4 + 6 H+ -> 2 Mn2+ + 10 CO2 + 8 H2O
    //
    // and the six protons are the boundary this row states. A vessel on
    // this bench has no proton portion — protons live in PHREEQC's
    // charge balance, not in the inventory — so a curated aqueous row
    // cannot consume H+ as matter. It is written in the equivalent basic
    // form instead, which is the same reaction with 6 H2O added to both
    // sides: the six protons leave as six hydroxides, and the sulfuric
    // acid the practical puts in the flask neutralises them on the next
    // solve. Same electrons, same mass, same acid consumed. It is the
    // convention the aqueous permanganate–ethanol row above already
    // uses, for the same reason.
    //
    // What this row therefore does NOT claim: that the reaction stops
    // without acid. It fires on contact, where the real one needs the
    // acidic medium and ~60 C to start. Both are conditions this bench
    // does not gate on, and stating that is cheaper than a wrong gate.
    CuratedReaction {
        equation: "2 MnO₄⁻ + 5 H₂C₂O₄ → 2 Mn²⁺ + 10 CO₂↑ + 2 H₂O + 6 OH⁻",
        reactants: &[("MnO4-", 2.0), ("H2C2O4", 5.0)],
        products: &[
            ("Mn+2", 2.0, Phase::Aqueous),
            ("CO2", 10.0, Phase::Gas),
            ("water", 2.0, Phase::Liquid),
            ("OH-", 6.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // The same reaction reached from the solid in the burette: the
    // titrant is dosed as KMnO4 and the curated stage runs before the
    // aqueous one, so the first thing this reaction ever sees is the
    // undissolved salt. Potassium spectates and is booked as the ion the
    // databases would have given it.
    CuratedReaction {
        equation: "2 KMnO₄ + 5 H₂C₂O₄ → 2 Mn²⁺ + 2 K⁺ + 10 CO₂↑ + 2 H₂O + 6 OH⁻",
        reactants: &[("KMnO4", 2.0), ("H2C2O4", 5.0)],
        products: &[
            ("Mn+2", 2.0, Phase::Aqueous),
            ("K+", 2.0, Phase::Aqueous),
            ("CO2", 10.0, Phase::Gas),
            ("water", 2.0, Phase::Liquid),
            ("OH-", 6.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // ── silver metathesis in ethanol (CAP-23 rung 2b) ────────────
    // PHREEQC handles these in water; the curated entries fire only
    // in the organic solvent, drawing from the dissolved fraction.
    CuratedReaction {
        equation: "AgNO₃ + NaCl → AgCl↓ + NaNO₃",
        reactants: &[("AgNO3", 1.0), ("NaCl", 1.0)],
        products: &[("AgCl", 1.0, Phase::Solid), ("NaNO3", 1.0, Phase::Solid)],
        solvent: Some("ethanol"),
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "AgNO₃ + KCl → AgCl↓ + KNO₃",
        reactants: &[("AgNO3", 1.0), ("KCl", 1.0)],
        products: &[("AgCl", 1.0, Phase::Solid), ("KNO3", 1.0, Phase::Solid)],
        solvent: Some("ethanol"),
        min_temp_k: None,
        catalyst: None,
    },
    // ── iodine decolorisation (EXP-13: Vitamin C) ─────────────
    // Ascorbic acid reduces molecular iodine to iodide; this is
    // the basis of the iodometric vitamin C assay.
    CuratedReaction {
        equation: "C₆H₈O₆ + I₂ → C₆H₆O₆ + 2 HI",
        reactants: &[("ascorbic_acid", 1.0), ("I2", 1.0)],
        products: &[
            ("dehydroascorbic_acid", 1.0, Phase::Aqueous),
            ("HI", 2.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    // ── thermal decomposition (EXP-2: Backpulver) ───────────────
    // Onset ~50 °C, classroom-observable above ~80 °C (CRC Handbook
    // 97th ed.; Merck Index 15th ed.). Threshold set at 353 K.
    CuratedReaction {
        equation: "2 NaHCO₃ →Δ Na₂CO₃ + H₂O + CO₂↑",
        reactants: &[("NaHCO3", 2.0)],
        products: &[
            ("Na2CO3", 1.0, Phase::Solid),
            ("water", 1.0, Phase::Liquid),
            ("CO2", 1.0, Phase::Gas),
        ],
        solvent: None,
        min_temp_k: Some(353.0),
        catalyst: None,
    },
    // ── enzymatic hydrolysis (EXP-14: Das süße Brot) ────────────
    // Amylase catalyses starch hydrolysis to maltose. The enzyme
    // is not consumed. Simplified: 2 monomer units + H₂O → maltose.
    CuratedReaction {
        equation: "2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁",
        reactants: &[("starch", 2.0), ("water", 1.0)],
        products: &[("maltose", 1.0, Phase::Aqueous)],
        solvent: None,
        min_temp_k: None,
        catalyst: Some("amylase"),
    },
    // ── EXP-5: hypochlorite bleaching of dyes ──────────────────────
    CuratedReaction {
        equation: "betanin + NaOCl → betanin(ox) + NaCl",
        reactants: &[("betanin", 1.0), ("NaOCl", 1.0)],
        products: &[
            ("betanin_ox", 1.0, Phase::Aqueous),
            ("NaCl", 1.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "curcumin + NaOCl → curcumin(ox) + NaCl",
        reactants: &[("curcumin", 1.0), ("NaOCl", 1.0)],
        products: &[
            ("curcumin_ox", 1.0, Phase::Aqueous),
            ("NaCl", 1.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
    CuratedReaction {
        equation: "indigo carmine + NaOCl → isatin sulfonate + NaCl",
        reactants: &[("indigo_carmine", 1.0), ("NaOCl", 1.0)],
        products: &[
            ("indigo_carmine_ox", 1.0, Phase::Aqueous),
            ("NaCl", 1.0, Phase::Aqueous),
        ],
        solvent: None,
        min_temp_k: None,
        catalyst: None,
    },
];

/// A named organic transformation the `react` verb applies on command.
///
/// Deliberately NOT registered with `CuratedEquilibrator`: acid and
/// alcohol standing in one beaker at room temperature do not visibly
/// esterify — the reaction wants its conditions and its push, which is
/// what makes it a *verb*. The stoichiometry here is proven at the
/// identity level against the atom-mapped SMIRKS templates in
/// `kerotakis-org` (its differential test maps each product SMILES to
/// an InChIKey and requires the registry species named here to carry
/// that same key), so this table and the templates cannot drift apart.
pub struct OrgReaction {
    /// The name the verb takes: `react v1 esterification`.
    pub name: &'static str,
    pub equation: &'static str,
    pub reactants: &'static [(&'static str, f64)],
    pub products: &'static [(&'static str, f64, Phase)],
    /// What this entry does NOT claim — stated at lv3, because an
    /// equilibrium driven to completion on command is a modelling
    /// choice, not a measurement.
    pub boundary: &'static str,
    pub source: &'static str,
}

pub const ORG_REACTIONS: &[OrgReaction] = &[
    OrgReaction {
        name: "esterification",
        equation: "CH3COOH + C2H5OH ⇌ CH3COOC2H5 + H2O",
        reactants: &[("CH3COOH", 1.0), ("ethanol", 1.0)],
        products: &[
            ("ethyl_acetate", 1.0, Phase::Liquid),
            ("water", 1.0, Phase::Liquid),
        ],
        boundary: "Fischer esterification is an equilibrium (K ≈ 4 for this                    pair); the verb drives the requested extent to completion                    and says so — no yield claim is made, and the acid                    catalyst and heat the real reaction wants are assumed,                    not modelled",
        source: "Fischer esterification, March's Advanced Organic Chemistry;                  stoichiometry proven against the kerotakis-org SMIRKS                  template at the InChIKey level",
    },
    OrgReaction {
        name: "saponification",
        equation: "CH3COOC2H5 + NaOH → NaOAc + C2H5OH",
        reactants: &[("ethyl_acetate", 1.0), ("NaOH", 1.0)],
        products: &[
            ("NaOAc", 1.0, Phase::Aqueous),
            ("ethanol", 1.0, Phase::Liquid),
        ],
        boundary: "alkaline hydrolysis is driven by the carboxylate sink and                    goes to completion honestly; the heat of reaction is not                    yet curated and no thermal effect is applied",
        source: "Alkaline ester hydrolysis, March's Advanced Organic                  Chemistry; stoichiometry proven against the kerotakis-org                  SMIRKS template at the InChIKey level",
    },
];

const TRACE: f64 = 1e-12;

fn extent(vessel: &Vessel, reaction: &CuratedReaction) -> f64 {
    reaction
        .reactants
        .iter()
        .map(|(key, coeff)| vessel.moles_of(&SpeciesId::new(key)).0 / coeff)
        .fold(f64::INFINITY, f64::min)
}

/// Moles of a species available in solution for a solvent-gated
/// reaction: the liquid (already-dissolved) fraction plus whatever
/// solid could still dissolve up to the handbook solubility limit.
fn available_dissolved(vessel: &Vessel, key: &str, solvent: &str) -> f64 {
    let id = SpeciesId::new(key);
    let liquid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species == id && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum();
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species == id && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    if solid <= 0.0 {
        return liquid;
    }
    if let Some(row) = crate::nonaqueous::ORGANIC_SOLUBILITY
        .iter()
        .find(|r| r.solute == key && r.solvent == solvent)
    {
        let solvent_id = SpeciesId::new(solvent);
        let solvent_data =
            crate::species::lookup(&solvent_id).expect("known solvents are registry species");
        let solvent_moles: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species == solvent_id && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum();
        let solvent_ml = solvent_moles * solvent_data.molar_mass / solvent_data.density;
        let species_data = match crate::species::lookup(&id) {
            Some(d) => d,
            None => return liquid,
        };
        let limit_mol = row.g_per_100ml * (solvent_ml / 100.0) / species_data.molar_mass;
        let room = (limit_mol - liquid).max(0.0);
        liquid + solid.min(room)
    } else {
        liquid + solid
    }
}

fn extent_in_solvent(vessel: &Vessel, reaction: &CuratedReaction, solvent: &str) -> f64 {
    reaction
        .reactants
        .iter()
        .map(|(key, coeff)| available_dissolved(vessel, key, solvent) / coeff)
        .fold(f64::INFINITY, f64::min)
}

/// Withdraw from liquid phase first, then solid — ensures only the
/// dissolved (or would-dissolve) fraction is consumed.
fn withdraw_from_solution(vessel: &mut Vessel, species: &SpeciesId, moles: Moles) {
    let mut remaining = moles.0;
    for p in vessel.contents.iter_mut() {
        if &p.species == species && p.phase == Phase::Liquid && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    for p in vessel.contents.iter_mut() {
        if &p.species == species && p.phase == Phase::Solid && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    vessel.contents.retain(|p| p.moles.0 > 1e-15);
}

/// Applies every curated reaction whose reactants are present, to
/// completion, before the generic solvers run.
pub struct CuratedEquilibrator;

impl Equilibrator for CuratedEquilibrator {
    fn name(&self) -> &'static str {
        "curated-reactions"
    }

    fn route_kind(&self) -> crate::solve::SolverRouteKind {
        crate::solve::SolverRouteKind::Curated
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        let solvent = crate::nonaqueous::single_organic_solvent(vessel);
        REACTIONS.iter().any(|r| {
            if let Some(min_t) = r.min_temp_k {
                if vessel.temperature.0 < min_t {
                    return false;
                }
            }
            if let Some(cat) = r.catalyst {
                if vessel.moles_of(&SpeciesId::new(cat)).0 <= TRACE {
                    return false;
                }
            }
            if let Some(req) = r.solvent {
                solvent == Some(req) && extent_in_solvent(vessel, r, req) > TRACE
            } else {
                extent(vessel, r) > TRACE
            }
        })
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        // Solvent-gated reactions fire at most once per equilibration:
        // the dissolved fraction reacts, and that is all until the next
        // step re-equilibrates (Le Chatelier pull from the solid is a
        // multi-step kinetic, not an instant cascade).
        let mut solvent_gated_done = [false; REACTIONS.len()];
        for _ in 0..8 {
            let mut progressed = false;
            let solvent = crate::nonaqueous::single_organic_solvent(vessel);
            for (i, reaction) in REACTIONS.iter().enumerate() {
                if let Some(min_t) = reaction.min_temp_k {
                    if vessel.temperature.0 < min_t {
                        continue;
                    }
                }
                if let Some(cat) = reaction.catalyst {
                    if vessel.moles_of(&SpeciesId::new(cat)).0 <= TRACE {
                        continue;
                    }
                }
                let x = if let Some(req) = reaction.solvent {
                    if solvent != Some(req) || solvent_gated_done[i] {
                        continue;
                    }
                    extent_in_solvent(vessel, reaction, req)
                } else {
                    extent(vessel, reaction)
                };
                if x <= TRACE {
                    continue;
                }
                progressed = true;
                if reaction.solvent.is_some() {
                    solvent_gated_done[i] = true;
                }
                for (key, coeff) in reaction.reactants {
                    if reaction.solvent.is_some() {
                        withdraw_from_solution(vessel, &SpeciesId::new(key), Moles(x * coeff));
                    } else {
                        vessel.withdraw(&SpeciesId::new(key), Moles(x * coeff));
                    }
                }
                events.push(Event::ReactionOccurred {
                    vessel: vessel.id,
                    equation: reaction.equation.to_string(),
                });
                for (key, coeff, phase) in reaction.products {
                    let n = Moles(x * coeff);
                    if *phase == Phase::Gas {
                        let species = SpeciesId::new(key);
                        if vessel.retain_gas(species.clone(), n) {
                            events.push(Event::GasContained {
                                vessel: vessel.id,
                                species,
                                moles: n,
                            });
                        } else {
                            events.push(Event::GasEvolved {
                                vessel: vessel.id,
                                species,
                                moles: n,
                            });
                        }
                    } else {
                        vessel.deposit(SpeciesId::new(key), n, *phase);
                    }
                }
            }
            if !progressed {
                break;
            }
        }
        Ok(events)
    }
}
