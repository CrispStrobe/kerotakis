//! EXP-33: the phase routes that are not the solvent's.
//!
//! `states.rs` is the solvent's story — water freezing and boiling, with the
//! thresholds moved by whatever is dissolved in it. This module is the other
//! ways matter changes state on this bench, and none of them is water's:
//!
//! * **The cryogen route.** Liquid nitrogen boils at 77 K, and what it takes
//!   from the beaker around it while doing so is the whole point of pouring
//!   it: ethanol at room temperature ends up a solid block. Freezing,
//!   melting, boiling and condensing for the substances that carry an
//!   enthalpy for it, with the freezing and the boiling COUPLED — heat
//!   released at a cryogen's boiling point does not raise a temperature, it
//!   boils more cryogen.
//!
//! * **Sublimation.** Ammonium chloride does not melt on a hot plate; it goes
//!   straight to vapour at 338 °C and comes back as a white crust on anything
//!   cool. That is a *separation*: heat a mixture of ammonium chloride and
//!   common salt, and one of them leaves.
//! * **Hydrate bookkeeping.** Blue copper sulfate is not copper sulfate; it is
//!   copper sulfate plus five waters, and the crucible proves it — heat it,
//!   weigh it, and the missing mass is exactly the water. Put a drop back and
//!   the blue returns.
//!
//! All three are curated thresholds rather than computed equilibria, and all
//! three say so. What is *not* curated is the arithmetic: the water driven off a
//! hydrate is counted in moles and reappears as mass on the balance, so the
//! classic mass-before / mass-after lesson closes to the digit rather than to
//! a rounding.
//!
//! ## What this module does not model
//!
//! * **Intermediate hydrates.** Copper sulfate pentahydrate really loses its
//!   waters stepwise (TGA: two near 63 °C, two near 109 °C, the last near
//!   200 °C) through a trihydrate and a monohydrate. Neither intermediate is
//!   in the registry, so this bench does the transition in ONE step at the
//!   final-water temperature and says so. A partially dehydrated hydrate is a
//!   real substance and this bench does not have it.
//! * **Dissociative sublimation.** Ammonium chloride vapour is really ammonia
//!   and hydrogen chloride, which recombine on the cold surface. The bench
//!   moves the intact formula unit, which is what the recovered crust weighs
//!   and what the demonstration shows, but the vapour is not NH₄Cl molecules.
//! * **Rates.** Both routes complete within the step that crosses the
//!   threshold. A real sublimation takes time and a real crucible takes
//!   minutes at temperature; no kinetics is claimed.
//! * **Water activity.** Whether an anhydrous salt takes water back as a
//!   hydrate or simply dissolves is, in truth, a question about water
//!   activity. This bench uses the stoichiometric proxy in
//!   `REHYDRATION_WATER_HEADROOM` below and states it rather than pretending
//!   to a phase diagram it does not have.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Amounts below this are not chemistry, they are float dust.
const TRACE: f64 = 1e-12;

/// How much more water than the crystal formula asks for may be present
/// before the bench stops calling the result a hydrate.
///
/// A stated model choice with a real justification: the school demonstration
/// is a *drop* of water on a spatula of white powder, and the blue that
/// appears is the hydrate, not a solution. Once there is enough water to
/// dissolve the salt, dissolution is the honest answer and the aqueous
/// engine owns it — chalcanthite and epsomite are both phases in the shipped
/// USGS database, so crystallising them back out of solution is a computed
/// solve, not this module's business. The proxy is stoichiometric because
/// the real criterion is water activity and this bench does not compute it.
pub const REHYDRATION_WATER_HEADROOM: f64 = 1.0;

/// A hydrate the bench can take apart and put back together.
///
/// The stoichiometry is not stored: it is read off the registry formula, so
/// a hydrate whose formula says `·5H2O` cannot disagree with a table saying
/// four. Only the *temperature* is curated, because only the temperature is
/// a measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HydratePair {
    /// Registry key of the hydrate, e.g. `chalcanthite`.
    pub hydrate: &'static str,
    /// Registry key of the anhydrous salt, e.g. `CuSO4`.
    pub anhydrous: &'static str,
    /// Waters of crystallisation per formula unit.
    pub waters: f64,
    /// Where this bench drives them all off, K.
    pub dehydration_k: f64,
}

/// Split a hydrate formula into its anhydrous part and its water count.
///
/// `MgSO4·7H2O` → `("MgSO4", 7.0)`. Returns `None` for anything without a
/// hydrate dot, which is most of the registry.
pub fn split_hydrate(formula: &str) -> Option<(&str, f64)> {
    let (anhydrous, waters) = formula
        .split_once('·')
        .or_else(|| formula.split_once('*'))?;
    let waters = waters.trim();
    let rest = waters.strip_suffix("H2O")?;
    let n: f64 = if rest.is_empty() {
        1.0
    } else {
        rest.parse().ok()?
    };
    (n > 0.0).then_some((anhydrous.trim(), n))
}

/// Every hydrate/anhydrous pair the registry can actually take apart: the
/// hydrate carries a dehydration temperature AND its anhydrous partner is a
/// shipped species. A hydrate with no partner is not an error — it is a
/// hydrate this bench will not claim to dehydrate, and the melting-point
/// apparatus still reports its dehydration temperature as data.
pub fn hydrate_pairs() -> Vec<HydratePair> {
    let mut pairs = Vec::new();
    for species in crate::species::registry() {
        let Some(t) = species.transitions else {
            continue;
        };
        let Some(dehydration_k) = t.dehydration_k else {
            continue;
        };
        let Some((anhydrous_formula, waters)) = split_hydrate(species.formula) else {
            continue;
        };
        let Some(partner) = crate::species::registry()
            .iter()
            .find(|candidate| candidate.formula == anhydrous_formula)
        else {
            continue;
        };
        pairs.push(HydratePair {
            hydrate: species.key,
            anhydrous: partner.key,
            waters,
            dehydration_k,
        });
    }
    pairs
}

/// The solids that leave as vapour without melting first.
pub fn sublimes_at(species: &SpeciesId) -> Option<f64> {
    let data = crate::species::lookup(species)?;
    let t = data.transitions?;
    // A substance with a melting point melts; sublimation at 1 atm is for
    // the ones whose vapour pressure reaches an atmosphere while they are
    // still solid, and the registry records that by having no melting point
    // and a sublimation point instead.
    t.melting_k.is_none().then_some(t.sublimation_k)?
}

/// The latent heat one phase route has to pay for, in kJ/mol, positive
/// meaning "the vessel supplies it".
///
/// This is a curated table for the same reason `combustion::FUELS` is one:
/// the value is a measurement with a source, not something the registry
/// schema has a slot for. `PhaseTransitions` carries five temperatures and
/// no energies, and widening it would touch the build script, the runtime
/// loader, the export crate and their three fidelity tests for a claim
/// that two rows need.
///
/// **The table is deliberately not total, and that is load-bearing.** A
/// substance with no row here sublimes exactly as it did before this
/// tranche: all of it, in the step that crosses the threshold, at no
/// energy cost. Ammonium chloride is such a substance, and its behaviour
/// is unchanged — the crucible separation is a *separation*, and nobody
/// weighed the heat it took. Adding a row is therefore a deliberate act
/// that changes what a vessel does.
#[derive(Debug, Clone, Copy)]
pub struct LatentHeat {
    /// Registry species key of the CONDENSED phase.
    pub species: &'static str,
    /// kJ per mole of the formula unit.
    pub kj_per_mol: f64,
    pub provenance: &'static str,
}

/// Enthalpies of sublimation, keyed by the solid that leaves.
pub const SUBLIMATION_ENTHALPIES: &[LatentHeat] = &[LatentHeat {
    species: "dry_ice",
    // 25.2 kJ/mol at the 1 atm sublimation point.
    kj_per_mol: 25.2,
    provenance: "Enthalpy of sublimation of carbon dioxide at its 194.7 K normal sublimation point, 25.2 kJ/mol, as commonly tabulated from NIST/CODATA-class evaluated data. PENDING REVIEW: no positively identified page was opened for this row, so no edition-level provenance is claimed and the value stands as the standard tabulated one. Sanity check a reviewer can run without a book: it is the sum of the tabulated 8.65 kJ/mol enthalpy of fusion at the triple point and about 16.7 kJ/mol of vaporisation there, and it is the number that makes 5 g of dry ice cool 100 g of water by about 6.8 K, which is what a kitchen thermometer reads",
}];

/// The enthalpy of sublimation of a solid, J/mol, or `None` where the
/// bench does not claim one.
pub fn sublimation_enthalpy(species: &str) -> Option<f64> {
    SUBLIMATION_ENTHALPIES
        .iter()
        .find(|row| row.species == species)
        .map(|row| row.kj_per_mol * 1000.0)
}

/// The gas a subliming solid becomes.
///
/// Ammonium chloride's vapour is ammonium chloride: nothing else on the
/// shelf has its formula, so the route moves the same key between phases
/// and the crust that comes back is the salt that left. Dry ice's vapour
/// is *carbon dioxide*, which this registry carries as its own gas
/// species — and calling it "dry ice gas" in a vessel would be a
/// contradiction in terms.
///
/// The pair is derived from the formula rather than tabulated, exactly as
/// `hydrate_pairs` derives its stoichiometry from one: a species that
/// claims to be the solid form of a shipped gas cannot disagree with the
/// gas about what it is made of.
pub fn sublimation_product(solid_key: &str) -> &'static str {
    let Some(solid) = crate::species::lookup(&SpeciesId::new(solid_key)) else {
        return "";
    };
    crate::species::registry()
        .iter()
        .find(|candidate| {
            candidate.standard_phase == Phase::Gas
                && candidate.formula == solid.formula
                && candidate.key != solid.key
        })
        .map_or(solid.key, |gas| gas.key)
}

/// The solid a gas deposits as, with the temperature it happens at:
/// the inverse of [`sublimation_product`], resolved over the registry.
pub fn deposition_partner(gas_key: &str) -> Option<(&'static str, f64)> {
    crate::species::registry().iter().find_map(|candidate| {
        let k = sublimes_at(&SpeciesId::new(candidate.key))?;
        (candidate.standard_phase != Phase::Gas && sublimation_product(candidate.key) == gas_key)
            .then_some((candidate.key, k))
    })
}

/// A condensed species that is a phase of a substance this registry also
/// ships as a gas: `dry_ice` for `CO2`.
///
/// Such a key exists so that a bench can HOLD the condensed phase — you
/// cannot put carbon dioxide gas in a beaker and call it dry ice. It is
/// emphatically not a mineral, and anything that pairs registry solids
/// with database phases by composition must skip it, or a carbonate
/// solution acquires a "mineral" with dry ice's formula and precipitates
/// it at 25 °C. `kerotakis_phreeqc::derived` is the caller that matters.
pub fn is_condensed_gas(key: &str) -> bool {
    let Some(data) = crate::species::lookup(&SpeciesId::new(key)) else {
        return false;
    };
    data.standard_phase != Phase::Gas && sublimation_product(key) != key
}

/// The temperature a condensed gas ARRIVES at when it is poured from its
/// bottle: its own sublimation or boiling point, K. `None` for everything
/// else — a reagent that is stable at room temperature arrives at the
/// room's.
///
/// `add` used to deposit every reagent at 298.15 K, and liquid nitrogen at
/// 298.15 K is a state that cannot exist. The route above discarded that
/// superheat, at the cost stated on `ledger`: it could not tell it from
/// heat a `heat` command had honestly put in. Depositing the cryogen cold
/// in the first place is the fix that cost pointed at — the adiabatic
/// mix on `add` then cools the flask the way pouring really does, and the
/// route finds a vessel already at the cryogen's temperature with nothing
/// impossible to discard.
pub fn arrives_at_k(key: &str) -> Option<f64> {
    if !is_condensed_gas(key) {
        return None;
    }
    sublimes_at(&SpeciesId::new(key)).or_else(|| boils_at(key))
}

fn moles_in_phase(vessel: &Vessel, species: &SpeciesId, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == species && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn withdraw_phase(vessel: &mut Vessel, species: &SpeciesId, phase: Phase, moles: f64) {
    let mut remaining = moles;
    for p in vessel.contents.iter_mut() {
        if &p.species == species && p.phase == phase && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    vessel.contents.retain(|p| p.moles.0 > 1e-15);
}

/// Release `moles` of a gas: into the headspace if the vessel owns one,
/// otherwise across the boundary. Either way the balance notices.
fn release_gas(vessel: &mut Vessel, species: SpeciesId, moles: Moles, events: &mut Vec<Event>) {
    let id = vessel.id;
    if vessel.retain_gas(species.clone(), moles) {
        events.push(Event::GasContained {
            vessel: id,
            species,
            moles,
        });
    } else {
        events.push(Event::GasEvolved {
            vessel: id,
            species,
            moles,
        });
    }
}

/// Enthalpies of fusion, keyed by the substance that freezes or melts.
///
/// **Water is deliberately absent and has to stay absent.**
/// `solve::StateEquilibrator` owns the solvent's freezing and boiling,
/// with the colligative shifts `states.rs` computes on top, and two
/// solvers moving the same ice would be a bug rather than a redundancy.
/// This table is for the substances that model was never about.
///
/// ## Why the table is no longer one row
///
/// It used to be one, and the comment beside it said the shortness was the
/// point: a row here is a deliberate act, because [`Ledger`] turns it into
/// a plateau on a thermometer. That has not changed. What changed is the
/// claim the shortness was protecting — that the bench models water's
/// transitions and nobody else's — which PLAN P3s lists as a correctness
/// bug rather than a boundary. A bench that holds a crucible of lead at
/// 700 K and calls it solid is making the same mistake as the one that
/// reported liquid water at −7.95 °C: it is returning the absence of a
/// model as an observation.
///
/// So the route below is now general over the registry's own reviewed
/// melting points, and this table is what decides which of them the bench
/// will actually pay for. Every row is a substance whose melting a learner
/// can reach: the organic liquids a freezing mixture solidifies, the wax-
/// like solid a cooling curve is drawn on, the metals a flame can cast,
/// and the two alkali halides whose molten state is a different substance
/// from their solution.
pub const FUSION_ENTHALPIES: &[LatentHeat] = &[
    LatentHeat {
        species: "ethanol",
        // 4.93 kJ/mol at the 159.01 K melting point already in the registry.
        kj_per_mol: 4.93,
        provenance: "Ethanol enthalpy of fusion 4.93 kJ/mol at its normal melting point: NIST Chemistry WebBook, SRD 69, ethanol (CAS 64-17-5), phase-change data, https://webbook.nist.gov/cgi/cbook.cgi?ID=C64175&Mask=4. The value is roughly a fifth of water's 6.01 kJ/mol per mole and about a ninth per gram, which is why a small pour of liquid nitrogen can freeze ethanol but would barely dent the same mass of water",
    },
    LatentHeat {
        species: "methanol",
        // 3.18 kJ/mol at the 175.62 K melting point in the registry.
        kj_per_mol: 3.18,
        provenance: "Methanol enthalpy of fusion 3.18 kJ/mol at its normal melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "propanone",
        // 5.69 kJ/mol at the 178.45 K melting point in the registry.
        kj_per_mol: 5.69,
        provenance: "Propanone (acetone) enthalpy of fusion 5.69 kJ/mol at its normal melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "isopropanol",
        // 5.37 kJ/mol at the 185.25 K melting point in the registry.
        kj_per_mol: 5.37,
        provenance: "Propan-2-ol enthalpy of fusion 5.37 kJ/mol at its normal melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "hexane",
        // 13.08 kJ/mol at the 177.88 K melting point in the registry.
        kj_per_mol: 13.08,
        provenance: "Hexane enthalpy of fusion 13.08 kJ/mol at its normal melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "ethyl_acetate",
        // 10.48 kJ/mol at the 189.55 K melting point in the registry.
        kj_per_mol: 10.48,
        provenance: "Ethyl acetate enthalpy of fusion 10.48 kJ/mol at its normal melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "CH3COOH",
        // 11.73 kJ/mol at the 289.75 K melting point in the registry.
        kj_per_mol: 11.73,
        provenance: "Acetic acid enthalpy of fusion 11.73 kJ/mol at its 289.75 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance. This is the row that makes \"glacial\" mean something: pure acetic acid at 16.6 degrees Celsius is a solid, and a cold laboratory really does freeze the bottle. It says nothing about vinegar, which is acetic acid dissolved in water and freezes on the solvent's depressed point instead",
    },
    LatentHeat {
        species: "naphthalene",
        // 19.01 kJ/mol at the 353.4 K melting point in the registry.
        kj_per_mol: 19.01,
        provenance: "Naphthalene enthalpy of fusion 19.01 kJ/mol at its 353.4 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance. Naphthalene is the substance school cooling-curve experiments are actually run on, because the plateau sits in a water bath's reach. No enthalpy of vaporisation is claimed for it here: its 491 K boiling point is outside what that experiment goes near, and a row would install a boil this tranche has not checked",
    },
    LatentHeat {
        species: "Pb",
        // 4.77 kJ/mol at the 600.61 K melting point in the registry.
        kj_per_mol: 4.77,
        provenance: "Lead enthalpy of fusion 4.77 kJ/mol at its 600.61 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "Zn",
        // 7.32 kJ/mol at the 692.68 K melting point in the registry.
        kj_per_mol: 7.32,
        provenance: "Zinc enthalpy of fusion 7.32 kJ/mol at its 692.68 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "Mg",
        // 8.48 kJ/mol at the 923.15 K melting point in the registry.
        kj_per_mol: 8.48,
        provenance: "Magnesium enthalpy of fusion 8.48 kJ/mol at its 923.15 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "Al",
        // 10.71 kJ/mol at the 933.47 K melting point in the registry.
        kj_per_mol: 10.71,
        provenance: "Aluminium enthalpy of fusion 10.71 kJ/mol at its 933.47 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "Ag",
        // 11.28 kJ/mol at the 1234.93 K melting point in the registry.
        kj_per_mol: 11.28,
        provenance: "Silver enthalpy of fusion 11.28 kJ/mol at its 1234.93 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "Cu",
        // 13.26 kJ/mol at the 1357.77 K melting point in the registry.
        kj_per_mol: 13.26,
        provenance: "Copper enthalpy of fusion 13.26 kJ/mol at its 1357.77 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "Fe",
        // 13.81 kJ/mol at the 1811.15 K melting point in the registry.
        kj_per_mol: 13.81,
        provenance: "Iron enthalpy of fusion 13.81 kJ/mol at its 1811.15 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance. A laboratory Bunsen tops out at 1773.15 K (crate::apparatus::BUNSEN_CEILING_K), forty kelvin short of this, so the bench melts lead, zinc, aluminium, silver and copper over a flame and declines to melt iron. That is not a gap in the table; it is the reason a blacksmith needs a forge",
    },
    LatentHeat {
        species: "NaCl",
        // 28.16 kJ/mol at the 1073.85 K melting point in the registry.
        kj_per_mol: 28.16,
        provenance: "Sodium chloride enthalpy of fusion 28.16 kJ/mol at its 1073.85 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance. Molten salt, not brine: this is the state a Downs cell electrolyses, and it is a different substance from the solution every other salt row on this bench is about",
    },
    LatentHeat {
        species: "KCl",
        // 26.28 kJ/mol at the 1044.15 K melting point in the registry.
        kj_per_mol: 26.28,
        provenance: "Potassium chloride enthalpy of fusion 26.28 kJ/mol at its 1044.15 K melting point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
];

/// Enthalpies of vaporisation at the normal boiling point.
///
/// This table used to hold one row, liquid nitrogen, and refused ethanol
/// on the ground that "giving ethanol a row would silently install a
/// general boiling route through the back door of a cryogen tranche".
/// The route is no longer installed silently or through a back door: PLAN
/// P3s asks for it in as many words, and ethanol standing liquid at 200 °C
/// because nothing would boil it is exactly the dishonesty that item
/// names. The row is given, and the reason is written down here.
///
/// **Only liquids get a row, and that is a constraint rather than an
/// oversight.** [`condensation_partner`] can find a vapour's way back to
/// the flask only for a species the registry carries with
/// `standard_phase == Liquid`, so a boil given to a standard-phase solid —
/// iodine, naphthalene, molten zinc — would be one-way: the vapour could
/// leave a sealed vessel's liquid behind and never come back on cooling.
/// A transition this bench pays for has to run in both directions or the
/// ledger is not a ledger, so those substances melt here and do not boil,
/// and a reader who wants to know why is reading it.
///
/// **No metal boils here either**, for the same reason plus a sharper one:
/// zinc's 1180 K boiling point is inside a Bunsen's reach and zinc fume is
/// a real hazard with a real name, so it is a claim that wants its own
/// tranche and its own safety row rather than a line in this one.
pub const VAPORISATION_ENTHALPIES: &[LatentHeat] = &[
    LatentHeat {
        species: "liquid_nitrogen",
        // 5.6 kJ/mol at 77 K, at the precision displayed by NIST SRD 69.
        kj_per_mol: 5.6,
        provenance: "Nitrogen enthalpy of vaporisation 5.6 kJ/mol at 77 K: NIST Chemistry WebBook, SRD 69, nitrogen (CAS 7727-37-9), phase-change data, https://webbook.nist.gov/cgi/cbook.cgi?ID=C7727379&Mask=4. It is about a fourteenth of water's 40.65 kJ/mol per mole, which is why liquid nitrogen boils away rapidly",
    },
    LatentHeat {
        species: "ethanol",
        // 38.56 kJ/mol at the 351.39 K boiling point in the registry.
        kj_per_mol: 38.56,
        provenance: "Ethanol enthalpy of vaporisation 38.56 kJ/mol at its normal boiling point: NIST Chemistry WebBook, SRD 69, ethanol (CAS 64-17-5), phase-change data, https://webbook.nist.gov/cgi/cbook.cgi?ID=C64175&Mask=4. Against water's 40.65 kJ/mol it is nearly the same per mole and less than half per gram, which is why a spirit burner empties so much faster than a kettle",
    },
    LatentHeat {
        species: "methanol",
        // 35.21 kJ/mol at the 337.85 K boiling point in the registry.
        kj_per_mol: 35.21,
        provenance: "Methanol enthalpy of vaporisation 35.21 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "propanone",
        // 29.10 kJ/mol at the 329.25 K boiling point in the registry.
        kj_per_mol: 29.10,
        provenance: "Propanone (acetone) enthalpy of vaporisation 29.10 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "isopropanol",
        // 39.85 kJ/mol at the 355.35 K boiling point in the registry.
        kj_per_mol: 39.85,
        provenance: "Propan-2-ol enthalpy of vaporisation 39.85 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "hexane",
        // 28.85 kJ/mol at the 341.88 K boiling point in the registry.
        kj_per_mol: 28.85,
        provenance: "Hexane enthalpy of vaporisation 28.85 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "ethyl_acetate",
        // 31.94 kJ/mol at the 350.21 K boiling point in the registry.
        kj_per_mol: 31.94,
        provenance: "Ethyl acetate enthalpy of vaporisation 31.94 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance.",
    },
    LatentHeat {
        species: "CH3COOH",
        // 23.70 kJ/mol at the 391.05 K boiling point in the registry.
        kj_per_mol: 23.70,
        provenance: "Acetic acid enthalpy of vaporisation 23.70 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th edition, \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables. PENDING REVIEW: the printed edition was not opened for this row, so no page-level provenance is claimed; the value is the standard tabulated one and agrees with NIST Chemistry WebBook SRD 69 phase-change data where that carries the substance. The number looks too small for a hydrogen-bonded liquid and the reason is chemistry rather than error: acetic acid vapour is largely the cyclic dimer, so half the hydrogen bonds survive the boil and are never paid for. This bench releases the vapour as monomeric CH3COOH because that is the only acetic acid the registry carries, and the dimer is not modelled",
    },
];

/// The enthalpy of fusion of a substance, J/mol, or `None` where this
/// bench claims none — which for a solvent means its own model owns it.
pub fn fusion_enthalpy(species: &str) -> Option<f64> {
    FUSION_ENTHALPIES
        .iter()
        .find(|row| row.species == species)
        .map(|row| row.kj_per_mol * 1000.0)
}

/// The enthalpy of vaporisation of a liquid at its normal boiling point,
/// J/mol, or `None` where this bench does not boil it.
pub fn vaporisation_enthalpy(species: &str) -> Option<f64> {
    VAPORISATION_ENTHALPIES
        .iter()
        .find(|row| row.species == species)
        .map(|row| row.kj_per_mol * 1000.0)
}

/// The normal melting point the registry records for a substance, K.
fn melts_at(key: &str) -> Option<f64> {
    crate::species::lookup(&SpeciesId::new(key))?
        .transitions?
        .melting_k
}

/// The normal boiling point the registry records for a substance, K.
fn boils_at(key: &str) -> Option<f64> {
    crate::species::lookup(&SpeciesId::new(key))?
        .transitions?
        .boiling_k
}

/// The liquid a vapour condenses back to, with its boiling point: the
/// inverse of the formula pairing [`sublimation_product`] performs, for
/// the boiling route rather than the subliming one.
fn condensation_partner(gas_key: &str) -> Option<(&'static str, f64)> {
    crate::species::registry().iter().find_map(|candidate| {
        if candidate.standard_phase != Phase::Liquid
            || vaporisation_enthalpy(candidate.key).is_none()
            || sublimation_product(candidate.key) != gas_key
        {
            return None;
        }
        boils_at(candidate.key).map(|boiling| (candidate.key, boiling))
    })
}

/// Heat with nothing left to absorb it warms the vessel.
fn spend_pool(vessel: &mut Vessel, pool: &mut f64, events: &mut Vec<Event>) {
    if *pool <= 0.0 {
        return;
    }
    if vessel.heat_capacity() <= 0.0 {
        *pool = 0.0;
        return;
    }
    let from = vessel.temperature;
    // Not `from + pool/Cp`: over a wide rise the heat capacity the vessel
    // ends with is not the one it started with, and the temperature that
    // absorbs this pool is the one where the INTEGRAL matches it.
    let to = crate::units::Kelvin(vessel.temperature_after(*pool));
    *pool = 0.0;
    if (to.0 - from.0).abs() <= 1e-9 {
        return;
    }
    vessel.temperature = to;
    vessel.refresh_pressure();
    events.push(Event::TemperatureChanged {
        vessel: vessel.id,
        from,
        to,
    });
}

/// What a latent-heat transition costs, and what the vessel can spend.
///
/// `budget` is the heat available to the transition, in joules, measured
/// from the transition temperature; `None` means no enthalpy is claimed
/// for this substance and the route stays athermal, exactly as it was
/// before this tranche.
struct Ledger {
    moles: f64,
    budget: Option<f64>,
    latent: f64,
    forward: bool,
}

/// ## The superheat a cryogen never had
///
/// `add` gives every portion the room temperature nobody asked it about
/// (`bench.rs` defaults `at` to `Kelvin::STANDARD`), and for a condensed
/// gas that is a state which does not exist: a block of dry ice is at
/// 194.7 K, not at 25 °C. Letting that fiction pay for the cooling would
/// be free energy — 5 g of "dry ice at 25 °C" carries 553 J of sensible
/// heat that no real block has.
///
/// A condensed gas arrives from its bottle at its own transition
/// temperature — `add` deposits it there ([`arrives_at_k`]) and the
/// adiabatic mix on the pour cools the rest of the vessel while nominally
/// warming the cryogen. That nominal warming is the state a real cryogen
/// cannot hold: the heat would have boiled (or sublimed) it. So the forward
/// route spends every joule the vessel shows above the threshold on the
/// latent change, the cryogen's own share included, and the vessel lands
/// at the threshold with the inventory the energy allows. A lone sample
/// with nothing to draw on sits at its own transition point and does not
/// leave, which is what an insulated flask of dry ice really does.
///
/// The one approximation left is stated rather than hidden: between the
/// pour and this route the cryogen's sensible heat is carried at its
/// CONDENSED heat capacity (a solid's for dry ice, a liquid's for
/// nitrogen), where the real substance warms as a gas. For 5 g of dry ice
/// in 100 g of water that over-counts the cooling by about half a kelvin.
///
/// Deposition needs no such correction: a vapour really is at the vessel
/// temperature, and the heat it gives back warms everything present.
fn ledger(
    vessel: &Vessel,
    _condensed: &str,
    latent: Option<f64>,
    inventory: f64,
    now: f64,
    threshold: f64,
    forward: bool,
) -> Ledger {
    let Some(latent) = latent else {
        return Ledger {
            moles: inventory,
            budget: None,
            latent: 0.0,
            forward,
        };
    };
    // The whole vessel's heat capacity, the condensed gas's own included.
    // Since `add` deposits a condensed gas at its own transition
    // temperature (`arrives_at_k`), any superheat the vessel shows above
    // that threshold was put there by the rest of the vessel warming the
    // cryogen in the adiabatic mix — heat a real cryogen would have spent
    // boiling — and it is this route's to spend. The earlier form excluded
    // the cryogen's own capacity to discard an arrival superheat that no
    // longer exists; keeping that exclusion here would lose the heat the
    // ethanol gave the nitrogen on the pour.
    // The heat between here and the threshold, integrated: over the span a
    // cryogen or a crucible covers, the area under Cp(T) and the rectangle
    // Cp(now) times the span are different numbers.
    let budget = if forward {
        vessel.energy_between(threshold, now)
    } else {
        vessel.energy_between(now, threshold)
    }
    .max(0.0);
    Ledger {
        moles: (budget / latent).min(inventory),
        budget: Some(budget),
        latent,
        forward,
    }
}

impl Ledger {
    /// Fold heat an exothermic change released in this same pass into an
    /// endothermic one's budget.
    ///
    /// This is what couples freezing to boiling, and without it energy
    /// goes missing. Ethanol dropped into liquid nitrogen freezes, and
    /// the 844 J that releases has to go somewhere — but the vessel is
    /// AT the nitrogen's boiling point, where heat does not raise a
    /// temperature, it boils nitrogen. Letting the freeze warm the flask
    /// to 83 K and then asking the boil-off to spend it would lose most
    /// of it to the superheat correction on [`ledger`], because liquid
    /// nitrogen at 83 K is exactly the impossible state that correction
    /// exists to discard.
    fn draw_on(&mut self, pool: &mut f64, inventory: f64) {
        if let Some(budget) = self.budget.as_mut() {
            *budget += *pool;
            *pool = 0.0;
            self.moles = (*budget / self.latent).min(inventory);
        }
    }
}

/// Spend the ledger and put the vessel at the temperature that leaves.
///
/// One expression covers all three cases the physics has. When the budget
/// is exactly consumed the vessel lands on the transition temperature and
/// stays there — the plateau, with condensed phase still in the flask.
/// When there is heat to spare the remainder warms (or, on deposition,
/// cools) whatever is left, over the heat capacity the vessel has AFTER
/// the move, so matter that left the ledger takes its own sensible heat
/// with it. And when nothing could be afforded at all, the same
/// expression puts the vessel on the transition temperature, which is the
/// correction described on [`ledger`].
fn settle(vessel: &mut Vessel, l: &Ledger, moved: f64, threshold: f64, events: &mut Vec<Event>) {
    let Some(budget) = l.budget else {
        return;
    };
    if vessel.heat_capacity() <= 0.0 {
        return;
    }
    let left = budget - moved * l.latent;
    // Spent from the threshold over the contents the vessel has NOW, and
    // integrated rather than divided: same three cases, one of which is
    // the plateau, where `left` is zero and this lands exactly on the
    // threshold as it always did.
    let to = if l.forward {
        vessel.temperature_after_from(threshold, left)
    } else {
        vessel.temperature_after_from(threshold, -left)
    };
    let to = crate::units::Kelvin(to.max(0.0));
    let from = vessel.temperature;
    if (to.0 - from.0).abs() <= 1e-9 {
        return;
    }
    vessel.temperature = to;
    vessel.refresh_pressure();
    events.push(Event::TemperatureChanged {
        vessel: vessel.id,
        from,
        to,
    });
}

/// Sublimation and hydrate bookkeeping, applied wherever the temperature
/// says they apply.
pub struct PhaseRouteEquilibrator;

impl PhaseRouteEquilibrator {
    /// Freezing, melting, boiling and condensing, for the substances that
    /// carry an enthalpy for it (th-123).
    ///
    /// This is the cryogen route, and it exists because pouring liquid
    /// nitrogen over ethanol is two coupled phase changes running at once:
    /// the nitrogen boils, the vessel falls to 77 K, the ethanol freezes,
    /// and the heat the freezing releases boils *more* nitrogen rather
    /// than warming anything. The pool is what couples them — see
    /// [`Ledger::draw_on`] — and running the exothermic half first in each
    /// pass is what lets the endothermic half spend it in the same pass.
    ///
    /// The sublimation route above deliberately has no pool: only one
    /// substance on this shelf sublimes and nothing exothermic coexists
    /// with it, so there is never anything to couple.
    ///
    /// Honest boundaries this route does NOT model: the Leidenfrost layer
    /// that makes a real pour skitter and slows the heat transfer to a
    /// fraction of what this instantaneous balance assumes; the glass that
    /// cracks when a warm beaker meets a cryogen; the fact that solid
    /// ethanol is a glassy slush before it is a block; and any heat
    /// leaking in from the room, which is the reason a real open dewar
    /// empties itself and this adiabatic one does not.
    fn cryogen(&self, vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
        let mut moved = false;
        // Latent heat an exothermic change has released and nothing has
        // spent yet.
        let mut pool = 0.0;
        for _ in 0..4 {
            let a = self.condensing(vessel, events, &mut pool);
            let b = self.vaporising(vessel, events, &mut pool);
            if !a && !b {
                break;
            }
            moved = true;
        }
        spend_pool(vessel, &mut pool, events);
        moved
    }

    /// The exothermic half: a liquid below its melting point freezes, a
    /// vapour below its boiling point condenses. Neither raises the
    /// temperature here — the heat goes to the pool, and what nothing
    /// absorbs is spent at the end of the pass.
    ///
    /// Both are limited by how much heat the vessel can still take before
    /// it reaches the transition temperature, which is what stops a freeze
    /// warming the flask back above the melting point and melting the same
    /// substance again on the next pass.
    ///
    /// Its endothermic partner admits a candidate sitting EXACTLY on its
    /// transition temperature, which is why the coupling works at all: a
    /// beaker of boiling nitrogen is at 77.36 K, not above it, and if the
    /// boil-off refused to look at it there the freezing heat would have
    /// nowhere to go but the thermometer.
    fn condensing(&self, vessel: &mut Vessel, events: &mut Vec<Event>, pool: &mut f64) -> bool {
        let mut moved = false;
        let freezing: Vec<(SpeciesId, f64, f64)> = vessel
            .contents
            .iter()
            .filter(|p| p.phase == Phase::Liquid && p.moles.0 > TRACE)
            .filter_map(|p| {
                let melting = melts_at(&p.species.0)?;
                let latent = fusion_enthalpy(&p.species.0)?;
                (vessel.temperature.0 < melting).then(|| (p.species.clone(), melting, latent))
            })
            .collect();
        for (species, melting, latent) in freezing {
            let now = vessel.temperature.0;
            if now >= melting {
                continue;
            }
            let inventory = moles_in_phase(vessel, &species, Phase::Liquid);
            let budget = vessel.energy_between(now, melting).max(0.0);
            let n = (budget / latent).min(inventory);
            if n <= TRACE {
                continue;
            }
            withdraw_phase(vessel, &species, Phase::Liquid, n);
            vessel.deposit(species.clone(), Moles(n), Phase::Solid);
            events.push(Event::state_changed(
                vessel.id,
                species,
                Phase::Liquid,
                Phase::Solid,
                crate::units::Kelvin(melting),
                0.0,
                Moles(n),
            ));
            *pool += n * latent;
            moved = true;
        }

        let condensing: Vec<(SpeciesId, &'static str, f64, f64)> = vessel
            .contents
            .iter()
            .filter(|p| p.phase == Phase::Gas && p.moles.0 > TRACE)
            .filter_map(|p| {
                let (liquid, boiling) = condensation_partner(&p.species.0)?;
                let latent = vaporisation_enthalpy(liquid)?;
                (vessel.temperature.0 < boiling)
                    .then(|| (p.species.clone(), liquid, boiling, latent))
            })
            .collect();
        for (gas, liquid, boiling, latent) in condensing {
            let now = vessel.temperature.0;
            if now >= boiling {
                continue;
            }
            let inventory = moles_in_phase(vessel, &gas, Phase::Gas);
            let budget = vessel.energy_between(now, boiling).max(0.0);
            let n = (budget / latent).min(inventory);
            if n <= TRACE {
                continue;
            }
            withdraw_phase(vessel, &gas, Phase::Gas, n);
            vessel.deposit(SpeciesId::new(liquid), Moles(n), Phase::Liquid);
            vessel.refresh_pressure();
            events.push(Event::state_changed(
                vessel.id,
                gas,
                Phase::Gas,
                Phase::Liquid,
                crate::units::Kelvin(boiling),
                0.0,
                Moles(n),
            ));
            *pool += n * latent;
            moved = true;
        }
        moved
    }

    /// The endothermic half: a solid above its melting point melts, a
    /// liquid above its boiling point boils. Both spend the pool first and
    /// the vessel's own sensible heat after, and both take the superheat
    /// correction [`ledger`] describes — a liquid found above its boiling
    /// point is as impossible as a block of dry ice at room temperature.
    fn vaporising(&self, vessel: &mut Vessel, events: &mut Vec<Event>, pool: &mut f64) -> bool {
        let mut moved = false;
        let melting: Vec<(SpeciesId, f64, f64)> = vessel
            .contents
            .iter()
            .filter(|p| p.phase == Phase::Solid && p.moles.0 > TRACE)
            .filter_map(|p| {
                let melting = melts_at(&p.species.0)?;
                let latent = fusion_enthalpy(&p.species.0)?;
                (vessel.temperature.0 >= melting).then(|| (p.species.clone(), melting, latent))
            })
            .collect();
        for (species, point, latent) in melting {
            let now = vessel.temperature.0;
            if now < point {
                continue;
            }
            let inventory = moles_in_phase(vessel, &species, Phase::Solid);
            let mut l = ledger(
                vessel,
                &species.0,
                Some(latent),
                inventory,
                now,
                point,
                true,
            );
            l.draw_on(pool, inventory);
            if l.moles > TRACE {
                withdraw_phase(vessel, &species, Phase::Solid, l.moles);
                vessel.deposit(species.clone(), Moles(l.moles), Phase::Liquid);
                events.push(Event::state_changed(
                    vessel.id,
                    species.clone(),
                    Phase::Solid,
                    Phase::Liquid,
                    crate::units::Kelvin(point),
                    0.0,
                    Moles(l.moles),
                ));
                moved = true;
            }
            settle(vessel, &l, l.moles, point, events);
        }

        let boiling: Vec<(SpeciesId, f64, f64)> = vessel
            .contents
            .iter()
            .filter(|p| p.phase == Phase::Liquid && p.moles.0 > TRACE)
            .filter_map(|p| {
                let boiling = boils_at(&p.species.0)?;
                let latent = vaporisation_enthalpy(&p.species.0)?;
                (vessel.temperature.0 >= boiling).then(|| (p.species.clone(), boiling, latent))
            })
            .collect();
        for (species, point, latent) in boiling {
            let now = vessel.temperature.0;
            if now < point {
                continue;
            }
            let inventory = moles_in_phase(vessel, &species, Phase::Liquid);
            let mut l = ledger(
                vessel,
                &species.0,
                Some(latent),
                inventory,
                now,
                point,
                true,
            );
            l.draw_on(pool, inventory);
            if l.moles > TRACE {
                let vapour = SpeciesId::new(sublimation_product(&species.0));
                withdraw_phase(vessel, &species, Phase::Liquid, l.moles);
                events.push(Event::state_changed(
                    vessel.id,
                    species.clone(),
                    Phase::Liquid,
                    Phase::Gas,
                    crate::units::Kelvin(point),
                    0.0,
                    Moles(l.moles),
                ));
                release_gas(vessel, vapour, Moles(l.moles), events);
                moved = true;
            }
            settle(vessel, &l, l.moles, point, events);
        }
        moved
    }

    fn sublimation(&self, vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
        let now = vessel.temperature.0;
        let mut moved = false;
        // Collect first: the loop mutates `contents`.
        //
        // A solid is a candidate if it has a sublimation point of its own;
        // a gas is a candidate if it has one (ammonium chloride vapour) or
        // if it is the vapour of a solid that does (carbon dioxide over
        // dry ice).
        let candidates: Vec<(SpeciesId, f64, Phase)> = vessel
            .contents
            .iter()
            .filter(|p| matches!(p.phase, Phase::Solid | Phase::Gas) && p.moles.0 > TRACE)
            .filter_map(|p| {
                let k = sublimes_at(&p.species).or_else(|| match p.phase {
                    Phase::Gas => deposition_partner(&p.species.0).map(|(_, k)| k),
                    _ => None,
                })?;
                Some((p.species.clone(), k, p.phase))
            })
            .collect();
        for (species, threshold, phase) in candidates {
            match phase {
                Phase::Solid if now >= threshold => {
                    let inventory = moles_in_phase(vessel, &species, Phase::Solid);
                    if inventory <= TRACE {
                        continue;
                    }
                    let l = ledger(
                        vessel,
                        &species.0,
                        sublimation_enthalpy(&species.0),
                        inventory,
                        now,
                        threshold,
                        true,
                    );
                    if l.moles > TRACE {
                        let vapour = SpeciesId::new(sublimation_product(&species.0));
                        withdraw_phase(vessel, &species, Phase::Solid, l.moles);
                        events.push(Event::state_changed(
                            vessel.id,
                            species.clone(),
                            Phase::Solid,
                            Phase::Gas,
                            crate::units::Kelvin(threshold),
                            0.0,
                            Moles(l.moles),
                        ));
                        release_gas(vessel, vapour, Moles(l.moles), events);
                        moved = true;
                    }
                    settle(vessel, &l, l.moles, threshold, events);
                }
                // Deposition: the cold-finger half of the separation. The
                // vapour only comes back where the vessel kept it.
                Phase::Gas if now < threshold => {
                    let inventory = moles_in_phase(vessel, &species, Phase::Gas);
                    if inventory <= TRACE {
                        continue;
                    }
                    let solid = SpeciesId::new(
                        deposition_partner(&species.0).map_or(species.0.as_str(), |(key, _)| key),
                    );
                    let l = ledger(
                        vessel,
                        &solid.0,
                        sublimation_enthalpy(&solid.0),
                        inventory,
                        now,
                        threshold,
                        false,
                    );
                    if l.moles > TRACE {
                        withdraw_phase(vessel, &species, Phase::Gas, l.moles);
                        vessel.deposit(solid.clone(), Moles(l.moles), Phase::Solid);
                        vessel.refresh_pressure();
                        events.push(Event::state_changed(
                            vessel.id,
                            species,
                            Phase::Gas,
                            Phase::Solid,
                            crate::units::Kelvin(threshold),
                            0.0,
                            Moles(l.moles),
                        ));
                        moved = true;
                    }
                    settle(vessel, &l, l.moles, threshold, events);
                }
                _ => {}
            }
        }
        moved
    }

    fn hydrates(&self, vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
        let now = vessel.temperature.0;
        let water = SpeciesId::new("water");
        let mut moved = false;
        for pair in hydrate_pairs() {
            let hydrate = SpeciesId::new(pair.hydrate);
            let anhydrous = SpeciesId::new(pair.anhydrous);
            if now >= pair.dehydration_k {
                let n = moles_in_phase(vessel, &hydrate, Phase::Solid);
                if n <= TRACE {
                    continue;
                }
                withdraw_phase(vessel, &hydrate, Phase::Solid, n);
                vessel.deposit(anhydrous.clone(), Moles(n), Phase::Solid);
                events.push(Event::Dehydrated {
                    vessel: vessel.id,
                    hydrate: hydrate.clone(),
                    anhydrous: anhydrous.clone(),
                    formula_units: Moles(n),
                    water: Moles(n * pair.waters),
                    at: crate::units::Kelvin(pair.dehydration_k),
                });
                release_gas(vessel, water.clone(), Moles(n * pair.waters), events);
                moved = true;
                continue;
            }
            // Rehydration. Below the threshold, an anhydrous salt in contact
            // with a little water takes it back into the crystal — but only
            // a little: past the headroom, dissolving is what really happens
            // and the aqueous engine owns that.
            let salt = moles_in_phase(vessel, &anhydrous, Phase::Solid);
            if salt <= TRACE {
                continue;
            }
            let free_water = moles_in_phase(vessel, &water, Phase::Liquid);
            if free_water <= TRACE {
                continue;
            }
            let wanted = salt * pair.waters;
            if free_water > wanted * (1.0 + REHYDRATION_WATER_HEADROOM) {
                continue;
            }
            let formula_units = (free_water / pair.waters).min(salt);
            if formula_units <= TRACE {
                continue;
            }
            withdraw_phase(vessel, &anhydrous, Phase::Solid, formula_units);
            withdraw_phase(vessel, &water, Phase::Liquid, formula_units * pair.waters);
            vessel.deposit(hydrate.clone(), Moles(formula_units), Phase::Solid);
            events.push(Event::Hydrated {
                vessel: vessel.id,
                anhydrous,
                hydrate,
                formula_units: Moles(formula_units),
                water: Moles(formula_units * pair.waters),
            });
            moved = true;
        }
        moved
    }
}

impl Equilibrator for PhaseRouteEquilibrator {
    fn name(&self) -> &'static str {
        "phase-routes"
    }

    fn route_kind(&self) -> crate::solve::SolverRouteKind {
        crate::solve::SolverRouteKind::Curated
    }

    fn chemistry_applies(&self, _vessel: &Vessel) -> bool {
        // Neither route is chemistry: no bond is made or broken by
        // sublimation, and a hydrate's water is held by the lattice. They
        // are phase changes, and claiming otherwise would route a beaker
        // away from the aqueous solver that should still see it.
        false
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        // Two passes at most: dehydration can free water that a second
        // salt would take up, and deposition can never trigger sublimation
        // at the same temperature, so the sequence cannot cycle.
        for _ in 0..2 {
            // The cryogen route runs first: it is the one that can move
            // the vessel's temperature by two hundred kelvin, and both
            // routes below are temperature thresholds.
            let a = self.cryogen(vessel, &mut events);
            let b = self.sublimation(vessel, &mut events);
            let c = self.hydrates(vessel, &mut events);
            if !a && !b && !c {
                break;
            }
        }
        // BRD-023: what heat has done to a named plastic. It lives here
        // rather than in a solver of its own for the reason the two routes
        // above share this module: it is a curated threshold on a
        // thermometer, decided by a reviewed temperature and not by an
        // equilibrium, and it moves no matter. It runs after them because
        // a hydrate's water leaving is a change to the vessel and the
        // plastic should be read against the vessel as it ends the step.
        events.extend(crate::plastics::settle(vessel));
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrate_formulas_split_into_salt_and_water() {
        assert_eq!(split_hydrate("MgSO4·7H2O"), Some(("MgSO4", 7.0)));
        assert_eq!(split_hydrate("CaSO4·2H2O"), Some(("CaSO4", 2.0)));
        assert_eq!(split_hydrate("NaCl"), None);
        // A lone water without a count is one water, not zero.
        assert_eq!(split_hydrate("XY·H2O"), Some(("XY", 1.0)));
    }

    #[test]
    fn every_pair_conserves_mass_exactly() {
        // The whole hydrate lesson is a mass ledger, so the molar masses
        // have to be additive to the digit. If a hydrate's molar mass is
        // not its salt plus its water, the crucible cannot balance and no
        // amount of careful arithmetic downstream will fix it.
        for pair in hydrate_pairs() {
            let h = crate::species::lookup(&SpeciesId::new(pair.hydrate)).unwrap();
            let a = crate::species::lookup(&SpeciesId::new(pair.anhydrous)).unwrap();
            let w = crate::species::lookup(&SpeciesId::new("water")).unwrap();
            let sum = a.molar_mass + pair.waters * w.molar_mass;
            assert!(
                (h.molar_mass - sum).abs() < 1e-9,
                "{}: {} != {} + {}×{}",
                pair.hydrate,
                h.molar_mass,
                a.molar_mass,
                pair.waters,
                w.molar_mass
            );
        }
    }

    #[test]
    fn a_substance_that_melts_does_not_also_sublime() {
        // The registry records sublimation only where melting is not what
        // happens; iodine at one atmosphere melts, whatever the demo says.
        for species in crate::species::registry() {
            if let Some(t) = species.transitions {
                if t.sublimation_k.is_some() && t.melting_k.is_some() {
                    assert!(
                        sublimes_at(&SpeciesId::new(species.key)).is_none(),
                        "{} claims both a melting and a sublimation route",
                        species.key
                    );
                }
            }
        }
    }
}
