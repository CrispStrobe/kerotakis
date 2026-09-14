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
    // 25.23 kJ/mol.
    kj_per_mol: 25.23,
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
///
/// ## Where the numbers come from
///
/// The rule is a **preference, not a prohibition**: a primary measurement
/// is better than a compilation, and a compilation is a perfectly good
/// citation where no primary can be had. A measured value is a fact;
/// citing a book is not redistributing it. Twenty-three of these
/// twenty-six rows now name a primary measurement or a public-domain
/// government compilation; three name the CRC Handbook and say so.
///
/// The one source this table refuses is the **NIST Chemistry WebBook /
/// JANAF-online**, which asserts copyright over its compilation under the
/// Standard Reference Data Act — a different kind of objection from
/// preferring a primary. Three rows cited it and no longer do. The
/// sentence "agrees with NIST SRD 69" that thirteen rows carried is gone
/// for a second reason that has nothing to do with licensing: nobody ever
/// checked it, so it was reassurance rather than evidence.
///
/// ## The mistake that is worth keeping
///
/// Seven of the nine inorganic rows cite a public-domain United States
/// Government document that PRINTS the enthalpy of fusion: US Bureau of
/// Mines Bulletin 672 (Pankratz, 1982) for the metals, and NSRDS-NBS 37 —
/// the 1971 JANAF second edition, the escape hatch PLAN.md's audit names —
/// for the two alkali halides. Neither bears a copyright notice. Magnesium
/// and copper cite nothing, because the documents that could source them
/// disagree with the value; their rows say so.
///
/// They briefly cited something else, and the correction is the useful
/// part. On 2026-09-13 all nine were re-sourced by differencing NASA CEA's
/// vendored `thermo.inp`: `H(liquid, Tm) - H(crystal, Tm)`. CEA is
/// Apache-2.0, already on disk, and already this repository's primary
/// thermochemistry source, so it looked like the cheapest possible answer.
/// Six values moved by a fraction of a per cent and the derived numbers
/// landed on round figures — 4812, 7300, 8400, 10700, 28200 J/mol — which
/// read as the derivation recovering the tables the fitters started from.
///
/// Silver moved 11.28 to 11.00 and that was written up as a disagreement
/// between evaluations. It was a mistake. Bulletin 672 prints 2.700
/// kcal/mol, 11.297, and corroborates it from its own enthalpy-increment
/// column; CEA's `Ag(cr)` agrees with Bulletin 672 to 0.2 per cent while
/// its `Ag(L)` sits about 350 J/mol low.
///
/// **A difference of two independently fitted polynomials is not a
/// tabulated transition enthalpy.** Both fits can be excellent over their
/// own intervals and their difference at the shared boundary still carries
/// both residuals, because nothing in the fitting makes them meet at the
/// evaluated ΔH. `thermo.inp` exists to compute thermodynamic functions,
/// not to tabulate phase changes. The tell was there to be read and was
/// read the wrong way round: CEA's silver records cite Cox 1989, the CODATA
/// Key Values, which does not publish enthalpies of fusion at all, so that
/// citation could never have supported an 11.00.
///
/// CEA is still consulted. `kerotakis-cea`'s
/// `latent_heats_are_the_vendored_file` differences it against every row as
/// an INDEPENDENT cross-check — which it now genuinely is, the values
/// coming from elsewhere — inside a three per cent band that catches a
/// transposed digit while leaving room for the residuals. Silver's gap is
/// pinned as a gap rather than tolerated.
///
/// ## The organic rows
///
/// CEA carries no condensed methanol, acetone, propan-2-ol, hexane, ethyl
/// acetate, acetic acid or naphthalene, so nothing on disk reaches them and
/// they were upgraded one at a time. Two public-domain NBS compilations
/// carry some outright — **Circular 500** (Rossini, 1952) and **Circular
/// 461** (the 1947 NBS/GPO edition of API Research Project 44, which is not
/// the copyrighted TRC product of the same name) — and the rest cite the
/// primary calorimetry by DOI. Several of those papers are from 1928–1929
/// and entered the United States public domain on 1 January 2026 under the
/// 95-year term; the publishers' paywalls on them are a business decision,
/// not a rights one.
///
/// **Acetone's boil is the one row that could not be upgraded.** It keeps
/// the handbook citation it always had, and its provenance records what was
/// tried so nobody repeats the search.
///
/// ## Why a row is never deleted for having a weak source
///
/// [`PhaseRoute::vaporising`] and [`PhaseRoute::condensing`] reach this
/// table through a `filter_map` with `?`. A liquid whose boiling point is
/// in the registry but whose latent heat is missing is therefore not
/// refused — it is skipped, and the vessel goes on holding it above its
/// boiling point and calling it liquid. That is the P3s correctness bug
/// quoted higher up this comment: returning the absence of a model as an
/// observation.
///
/// So downgrading a citation and removing a row are very different acts,
/// and only the first is ever the right answer to a provenance concern. An
/// honest shrink would have to take the registry's transition temperature
/// with it, so that `boils_at` returns `None` and the bench has no
/// transition to be silently wrong about — a change to the phase-transition
/// tranche, not to this table.
pub const FUSION_ENTHALPIES: &[LatentHeat] = &[
    LatentHeat {
        species: "ethanol",
        // 4.93 kJ/mol.
        kj_per_mol: 4.93,
        provenance: "Ethanol enthalpy of fusion 4.93 kJ/mol at its 159.0 K melting point. T. Haida, H. Suga and S. Seki, 'Calorimetric study of the glassy state XII: Plural glass-transition phenomena of ethanol', J. Chem. Thermodyn. 9 (1977) 1133-1148, doi:10.1016/0021-9614(77)90115-X: 4931 J/mol for crystal I to liquid at 159.00 K. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. WHICH CRYSTAL MATTERS HERE. Ethanol is polymorphic and has a glassy state as well as a crystal, so 'the enthalpy of fusion of ethanol' is ambiguous until the phase is named; this is the crystal-I row, which is the one a freezing mixture makes. The older public-domain alternative - Kelley, J. Am. Chem. Soc. 51 (1929) 779, doi:10.1021/ja01378a016, and NBS Circular 500 after it - gives 1.200 kcal/mol at 158.6 K, which is 5.02 and 1.8 per cent higher, and it is not preferred despite being the cleaner licence: it predates the polymorphism being understood. The value is roughly a fifth of water's 6.01 kJ/mol per mole and about a ninth per gram, which is why a small pour of liquid nitrogen can freeze ethanol but would barely dent the same mass of water",
    },
    LatentHeat {
        species: "methanol",
        // 3.17 kJ/mol.
        kj_per_mol: 3.17,
        provenance: "Methanol enthalpy of fusion 3.17 kJ/mol at its 175.26 K melting point. NBS Circular 500, F. D. Rossini et al., Selected Values of Chemical Thermodynamic Properties, National Bureau of Standards, 1952; retrieved 2026-09-13 from https://archive.org/details/circularofbureau500ross. A United States Government work and not Standard Reference Data - the Standard Reference Data Act notice that governs the NIST WebBook is absent from it - so it carries no copyright and is not on PLAN.md's avoid row. Its own Preface states the scope limit that makes it reach some of these rows and not others: it covers carbon compounds of one and two carbon atoms only. Table 23-2 prints 0.757 kcal/mol at 175.26 K, which is 3167 J/mol at 1 cal = 4.184 J. The primary measurement behind it is K. K. Kelley, J. Am. Chem. Soc. 51 (1929) 180, doi:10.1021/ja01376a022, 757 cal/mol at 175.2 K, which is itself public domain now: works published in 1930 or earlier entered the United States public domain on 1 January 2026 under the 95-year term. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. THE VALUE MOVED, 3.18 to 3.17, which is the handbook's rounding replaced by the source's. NOT CLAIMED HERE: methanol also has a solid-solid transition at 157.4 K worth 0.154 kcal/mol, which Circular 500 carries and this bench does not model. This row is fusion only, and a cooling curve run below 157 K would miss a plateau it does not know about",
    },
    LatentHeat {
        species: "propanone",
        // 5.69 kJ/mol.
        kj_per_mol: 5.69,
        provenance: "Propanone (acetone) enthalpy of fusion 5.69 kJ/mol at its 177.6 K melting point. G. S. Parks and K. K. Kelley, 'Thermal data on organic compounds II', J. Phys. Chem. 32 (1928) 734-737, doi:10.1021/j150287a006: 1360 cal/mol at 177.6 K, which is 5690 J/mol. Published 1928, so in the United States public domain since 1 January 2026 under the 95-year term - the paywall on the publisher's scan is a business decision and not a rights one. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry.",
    },
    LatentHeat {
        species: "isopropanol",
        // 5.37 kJ/mol.
        kj_per_mol: 5.37,
        provenance: "Propan-2-ol enthalpy of fusion 5.37 kJ/mol at its 184.67 K melting point. K. K. Kelley, 'The heat capacities of isopropyl alcohol and acetone from 16 to 298 K', J. Am. Chem. Soc. 51 (1929) 1145-1150, doi:10.1021/ja01379a022: 1284 cal/mol at 184.67 K, which is 5372 J/mol. Published 1929, United States public domain since 1 January 2026. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry.",
    },
    LatentHeat {
        species: "hexane",
        // 13.08 kJ/mol.
        kj_per_mol: 13.08,
        provenance: "Hexane enthalpy of fusion 13.08 kJ/mol at its 177.84 K melting point. D. R. Douslin and H. M. Huffman, 'Low-temperature thermal data on the five isomeric hexanes', J. Am. Chem. Soc. 68 (1946) 1704-1708, doi:10.1021/ja01213a006: 3126 cal/mol at 177.84 K, which is 13079 J/mol. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. INDEPENDENTLY CORROBORATED BY A PUBLIC-DOMAIN DOCUMENT that was read in full: US Atomic Energy Commission report K-550 (1950), doi:10.2172/4430842, gives 3127 +/- 2 cal/mol at 177.844 K. NOT NBS CIRCULAR 461 FOR THIS ROW, although that document is public domain and is cited for hexane's boil below. Its Table 2z prints 3.114 kcal/mol, which is the superseded 1931 Huffman/Parks/Barmore determination: the table is dated 1945, a year before Douslin and Huffman measured it again. Preferring the open document there would have shipped a known-superseded number for the sake of a licence",
    },
    LatentHeat {
        species: "ethyl_acetate",
        // 10.48 kJ/mol.
        kj_per_mol: 10.48,
        provenance: "Ethyl acetate enthalpy of fusion 10.48 kJ/mol at its 189.3 K melting point. G. S. Parks, H. M. Huffman and S. B. Thomas... in the series's usual form, Parks, Huffman and Barmore, 'Thermal data on organic compounds XI', J. Am. Chem. Soc. 55 (1933) 2733-2740, doi:10.1021/ja01334a016: 2505.0 cal/mol at 189.3 K, which is 10481 J/mol. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. A DISCREPANCY A REVIEWER SHOULD SETTLE, recorded here rather than quietly fixed: the registry carries this species' melting point as 189.55 K, and both this calorimetry and the tabulated triple point put it at 189.3. A quarter of a kelvin does not matter to the latent heat but it is the temperature the bench freezes at, and the two numbers should not disagree",
    },
    LatentHeat {
        species: "CH3COOH",
        // 11.72 kJ/mol.
        kj_per_mol: 11.72,
        provenance: "Acetic acid enthalpy of fusion 11.72 kJ/mol at its 289.77 K melting point. NBS Circular 500, F. D. Rossini et al., Selected Values of Chemical Thermodynamic Properties, National Bureau of Standards, 1952; retrieved 2026-09-13 from https://archive.org/details/circularofbureau500ross. A United States Government work and not Standard Reference Data - the Standard Reference Data Act notice that governs the NIST WebBook is absent from it - so it carries no copyright and is not on PLAN.md's avoid row. Its own Preface states the scope limit that makes it reach some of these rows and not others: it covers carbon compounds of one and two carbon atoms only. It prints 2.80 kcal/mol at 289.77 K, which is 11715 J/mol. Acetic acid is a two-carbon compound, so it falls inside Circular 500's stated scope. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. THE VALUE MOVED, 11.73 to 11.72, which is the handbook's third figure replaced by the source's. This is the row that makes 'glacial' mean something: pure acetic acid at 16.6 degrees Celsius is a solid, and a cold laboratory really does freeze the bottle. It says nothing about vinegar, which is acetic acid dissolved in water and freezes on the solvent's depressed point instead",
    },
    LatentHeat {
        species: "naphthalene",
        // 18.98 kJ/mol.
        kj_per_mol: 18.98,
        provenance: "Naphthalene enthalpy of fusion 18.98 kJ/mol at its 353.40 K melting point. J. P. McCullough, H. L. Finke, J. F. Messerly, S. S. Todd, T. C. Kincheloe and G. Waddington, 'The low-temperature thermodynamic properties of naphthalene...', J. Phys. Chem. 61 (1957) 1105-1116, doi:10.1021/j150554a016: 18.98 kJ/mol at 353.40 K. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. INDEPENDENTLY CORROBORATED BY A PUBLIC-DOMAIN DOCUMENT read in full: NIPER-678 (1993), doi:10.2172/10181459, reports its own measurement of 18.99 kJ/mol at 353.40 K. THE VALUE MOVED, 19.01 to 18.98, and the reason to distrust where 19.01 came from is worth writing down: the NIST WebBook's row for this very paper prints 18.226 kJ/mol, which is a 4536-to-4356 digit transposition and sits 4.2 per cent below every determination since 1926 - and its companion entropy of fusion is derived from the same bad number, so the page corroborates itself. A source that can agree with itself while being wrong is a source this bench should not be reading. Naphthalene is the substance school cooling-curve experiments are actually run on, because the plateau sits in a water bath's reach. No enthalpy of vaporisation is claimed for it here: its 491 K boiling point is outside what that experiment goes near, and a row would install a boil this tranche has not checked",
    },
    LatentHeat {
        species: "Pb",
        // 4.80 kJ/mol.
        kj_per_mol: 4.80,
        provenance: "Lead enthalpy of fusion 4.80 kJ/mol at its 600.65 K melting point. L. B. Pankratz, Thermodynamic Properties of Elements and Oxides, United States Department of the Interior, Bureau of Mines Bulletin 672 (1982), Superintendent of Documents no. I 28.23:672; retrieved 2026-09-13 from https://stacks.cdc.gov/view/cdc/219421. A United States Government work: the document bears no copyright notice anywhere in its 518 pages, and CDC Stacks, which holds the legacy Bureau of Mines collection, records its rights as Public Domain. Kerotakis transcribes one printed phase-change line per row and redistributes no table. Bulletin 672's Pb(c,l) table prints \"600.65 K, melting point of Pb; delta-H = 1.147 kcal/mol\", which is 4799 J/mol at 1 cal = 4.184 J.",
    },
    LatentHeat {
        species: "Zn",
        // 7.32 kJ/mol.
        kj_per_mol: 7.32,
        provenance: "Zinc enthalpy of fusion 7.32 kJ/mol at its 692.73 K melting point. L. B. Pankratz, Thermodynamic Properties of Elements and Oxides, United States Department of the Interior, Bureau of Mines Bulletin 672 (1982), Superintendent of Documents no. I 28.23:672; retrieved 2026-09-13 from https://stacks.cdc.gov/view/cdc/219421. A United States Government work: the document bears no copyright notice anywhere in its 518 pages, and CDC Stacks, which holds the legacy Bureau of Mines collection, records its rights as Public Domain. Kerotakis transcribes one printed phase-change line per row and redistributes no table. Bulletin 672's Zn(c,l,g) table prints \"692.73 K, melting point of Zn; delta-H = 1.750 kcal/mol\" = 7322 J/mol, and the same table's enthalpy-increment column corroborates it internally: H-H(298) is 2.580 kcal/mol for the crystal and 4.330 for the liquid at that temperature, and the difference is the 1.750 printed above it.",
    },
    LatentHeat {
        species: "Mg",
        // 8.48 kJ/mol.
        kj_per_mol: 8.48,
        provenance: "Magnesium enthalpy of fusion 8.48 kJ/mol at its 923.15 K melting point. CRC Handbook of Chemistry and Physics, 97th ed., \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables - a COMMERCIAL COMPILATION, which this project cites happily but ranks below a primary measurement. PENDING REVIEW: no positively identified copy was opened for this row, so no edition-level page provenance is claimed. THIS ROW IS WHERE THREE EVALUATIONS DISAGREE, which is why no primary source is cited even though several documents carry a number. US Bureau of Mines Bulletin 672 prints 2.139 kcal/mol at 922 K, which is 8.95 and five and a half per cent above this; differencing NASA CEA's Mg(L) against its Mg(cr) gives 8.40. Bulletin 672 would be the better-licensed citation and is NOT taken, because it rests on Hultgren 1973 and is here the OLDER evaluation rather than the better one - citing it would buy provenance with accuracy. The handbook's 8.48 is the modern consensus, and upgrading this row means finding the modern evaluation's own primary, not swapping compilations.",
    },
    LatentHeat {
        species: "Al",
        // 10.80 kJ/mol.
        kj_per_mol: 10.80,
        provenance: "Aluminium enthalpy of fusion 10.80 kJ/mol at its 933.61 K melting point. L. B. Pankratz, Thermodynamic Properties of Elements and Oxides, United States Department of the Interior, Bureau of Mines Bulletin 672 (1982), Superintendent of Documents no. I 28.23:672; retrieved 2026-09-13 from https://stacks.cdc.gov/view/cdc/219421. A United States Government work: the document bears no copyright notice anywhere in its 518 pages, and CDC Stacks, which holds the legacy Bureau of Mines collection, records its rights as Public Domain. Kerotakis transcribes one printed phase-change line per row and redistributes no table. Bulletin 672 prints 2.580 kcal/mol at 933.61 K, which is 10795 J/mol.",
    },
    LatentHeat {
        species: "Ag",
        // 11.30 kJ/mol.
        kj_per_mol: 11.30,
        provenance: "Silver enthalpy of fusion 11.30 kJ/mol at its 1235.08 K melting point. L. B. Pankratz, Thermodynamic Properties of Elements and Oxides, United States Department of the Interior, Bureau of Mines Bulletin 672 (1982), Superintendent of Documents no. I 28.23:672; retrieved 2026-09-13 from https://stacks.cdc.gov/view/cdc/219421. A United States Government work: the document bears no copyright notice anywhere in its 518 pages, and CDC Stacks, which holds the legacy Bureau of Mines collection, records its rights as Public Domain. Kerotakis transcribes one printed phase-change line per row and redistributes no table. Bulletin 672 p. 32, Ag(c,l), prints \"1235.08 K, melting point of Ag; delta-H = 2.700 kcal/mol\" = 11297 J/mol, and the table's own enthalpy increments corroborate it: H-H(298) is 6.315 kcal/mol for the crystal and 9.015 for the liquid at 1235.08 K, differing by exactly the 2.700 printed. Its data are from Hultgren's Selected Values of the Thermodynamic Properties of the Elements, corrected to IPTS-68. THIS ROW IS A CORRECTION. It shipped at 11.28 with a handbook citation, was moved to 11.00 on 2026-09-13 by differencing NASA CEA's Ag(L) and Ag(cr) polynomials, and is moved back here. The CEA number was wrong and the way it was wrong is the lesson: CEA's Ag(cr) record agrees with Bulletin 672 to 0.2 per cent, but its Ag(L) record sits about 350 J/mol low, so the DIFFERENCE carries both fits' residuals even where each fit is good. CEA's Ag records cite Cox 1989, the CODATA Key Values, which does not publish enthalpies of fusion at all - so that citation could never have been the provenance of an 11.00, and the 2.7 per cent was a fitting artefact rather than a rival evaluation.",
    },
    LatentHeat {
        species: "Cu",
        // 13.26 kJ/mol.
        kj_per_mol: 13.26,
        provenance: "Copper enthalpy of fusion 13.26 kJ/mol at its 1357.77 K melting point. CRC Handbook of Chemistry and Physics, 97th ed., \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables - a COMMERCIAL COMPILATION, which this project cites happily but ranks below a primary measurement. PENDING REVIEW: no positively identified copy was opened for this row, so no edition-level page provenance is claimed. Three evaluations again, as for magnesium above: Bureau of Mines Bulletin 672 prints 3.120 kcal/mol at 1357.6 K, which is 13.05, and NASA CEA's difference gives 13.14. Bulletin 672 is the oldest of the three. The handbook's 13.26 is the modern consensus and stays until someone finds the primary behind it.",
    },
    LatentHeat {
        species: "Fe",
        // 13.81 kJ/mol.
        kj_per_mol: 13.81,
        provenance: "Iron enthalpy of fusion 13.81 kJ/mol at its 1811 K melting point. L. B. Pankratz, Thermodynamic Properties of Elements and Oxides, United States Department of the Interior, Bureau of Mines Bulletin 672 (1982), Superintendent of Documents no. I 28.23:672; retrieved 2026-09-13 from https://stacks.cdc.gov/view/cdc/219421. A United States Government work: the document bears no copyright notice anywhere in its 518 pages, and CDC Stacks, which holds the legacy Bureau of Mines collection, records its rights as Public Domain. Kerotakis transcribes one printed phase-change line per row and redistributes no table. Bulletin 672 prints 3.300 kcal/mol at 1811 K = 13807 J/mol. Iron melts out of delta-iron, not the alpha the bench starts from. A laboratory Bunsen tops out at 1773.15 K (crate::apparatus::BUNSEN_CEILING_K), forty kelvin short of this, so the bench melts lead, zinc, aluminium, silver and copper over a flame and declines to melt iron. That is not a gap in the table; it is the reason a blacksmith needs a forge.",
    },
    LatentHeat {
        species: "NaCl",
        // 28.16 kJ/mol.
        kj_per_mol: 28.16,
        provenance: "Sodium chloride enthalpy of fusion 28.16 kJ/mol at its 1073.8 K melting point. JANAF Thermochemical Tables, second edition, D. R. Stull and H. Prophet, NSRDS-NBS 37, National Bureau of Standards, June 1971, doi:10.6028/NBS.NSRDS.37; retrieved 2026-09-13 from https://nvlpubs.nist.gov/nistpubs/Legacy/NSRDS/nbsnsrds37.pdf. PLAN.md's provenance audit names this edition as the public-domain escape hatch from the NIST SRD row, and the document itself bears no copyright notice - its copyright page carries nothing but a Library of Congress card number. This is emphatically NOT the 1985 third or 1998 fourth edition, whose copyright is secured under 15 U.S.C. 290e and assigned to the American Institute of Physics and the American Chemical Society. The ClNa table, Sodium Chloride (NaCl) (Crystal), prints T(m) = 1073.8 +/- 1.0 K and delta-H(m) = 6.73 +/- 0.04 kcal/mol, which is 28158 J/mol. Its own note records the competing measurement rather than hiding it: Dworkin and Bredig, J. Phys. Chem. 64, 269 (1960), reported 1073 K and 6.69 +/- 0.06 kcal/mol. Molten salt, not brine: this is the state a Downs cell electrolyses, and it is a different substance from the solution every other salt row on this bench is about.",
    },
    LatentHeat {
        species: "KCl",
        // 26.28 kJ/mol.
        kj_per_mol: 26.28,
        provenance: "Potassium chloride enthalpy of fusion 26.28 kJ/mol at its 1044 K melting point. JANAF Thermochemical Tables, second edition, D. R. Stull and H. Prophet, NSRDS-NBS 37, National Bureau of Standards, June 1971, doi:10.6028/NBS.NSRDS.37; retrieved 2026-09-13 from https://nvlpubs.nist.gov/nistpubs/Legacy/NSRDS/nbsnsrds37.pdf. PLAN.md's provenance audit names this edition as the public-domain escape hatch from the NIST SRD row, and the document itself bears no copyright notice - its copyright page carries nothing but a Library of Congress card number. This is emphatically NOT the 1985 third or 1998 fourth edition, whose copyright is secured under 15 U.S.C. 290e and assigned to the American Institute of Physics and the American Chemical Society. The ClK table, Potassium Chloride (KCl) (Crystal), prints T(m) = 1044 K and delta-H(m) = 6.282 kcal/mol, which is 26284 J/mol. Its note records alternatives of 6.34, 6.4 and 6.5 kcal/mol from other workers, so the spread on this row is about three per cent and the fourth figure here is the table's, not a measurement's.",
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
        // 5.58 kJ/mol.
        kj_per_mol: 5.58,
        provenance: "Nitrogen enthalpy of vaporisation 5.58 kJ/mol at its 77.364 K normal boiling point. T. R. Strobridge, The Thermodynamic Properties of Nitrogen from 64 to 300 K between 0.1 and 200 Atmospheres, NBS Technical Note 129, NBS Cryogenic Engineering Laboratory, January 1962, doi:10.6028/NBS.TN.129, Table 2 (Thermodynamic Properties of Nitrogen at Saturation), the 1.000 atm row: delta-H(vap) = 199.260 J/g, which at that document's own stated basis of M = 28.016 g/mol is 5582 J/mol, and 5582 with the modern 28.0134 as well. Retrieved 2026-09-13 from https://nvlpubs.nist.gov/nistpubs/Legacy/TN/nbstechnicalnote129.pdf. NBS Technical Notes are works of United States Government employees, are not Standard Reference Data, and this one carries no copyright notice. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. Corroborated by two further public-domain or age-expired determinations: NACA TN 2969 (Furukawa and McCoskey, NBS, 1953) gives 5592.2 J/mol at 77.395 K, and Giauque and Clayton, J. Am. Chem. Soc. 55 (1933) 4875, doi:10.1021/ja01339a024, gives 1332.9 cal/mol = 5577 J/mol. Four anchors inside half a per cent. THE VALUE MOVED, 5.6 to 5.58: the retired figure cited the NIST Chemistry WebBook SRD 69 and was quoted to two significant figures because that is what the WebBook displayed. NASA CEA was the other candidate and is wrong here for a reason worth keeping: its N2(L) record is an assigned enthalpy with no polynomial, and paired with CEA's IDEAL-gas N2 record it gives 5.68 kJ/mol. TN 129's is a real-gas saturated-vapour enthalpy, so the 1.7 per cent between them IS the vapour's non-ideality at 77 K, and it is the reason no vaporisation row on this bench cites CEA. It is about a fourteenth of water's 40.65 kJ/mol per mole, which is why liquid nitrogen boils away rapidly",
    },
    LatentHeat {
        species: "ethanol",
        // 38.58 kJ/mol.
        kj_per_mol: 38.58,
        provenance: "Ethanol enthalpy of vaporisation 38.58 kJ/mol at its normal boiling point. NBS Circular 500, F. D. Rossini et al., Selected Values of Chemical Thermodynamic Properties, National Bureau of Standards, 1952; retrieved 2026-09-13 from https://archive.org/details/circularofbureau500ross. A United States Government work and not Standard Reference Data - the Standard Reference Data Act notice that governs the NIST WebBook is absent from it - so it carries no copyright and is not on PLAN.md's avoid row. Its own Preface states the scope limit that makes it reach some of these rows and not others: it covers carbon compounds of one and two carbon atoms only. It prints 9.22 kcal/mol at 78.5 degrees Celsius, which is 38576 J/mol. Ethanol is a two-carbon compound, inside Circular 500's stated scope. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. THE VALUE MOVED, 38.56 to 38.58. The retired figure traces through the NIST WebBook to Majer and Svoboda's 1985 IUPAC compilation, which is a copyrighted book. Against water's 40.65 kJ/mol it is nearly the same per mole and less than half per gram, which is why a spirit burner empties so much faster than a kettle",
    },
    LatentHeat {
        species: "methanol",
        // 35.27 kJ/mol.
        kj_per_mol: 35.27,
        provenance: "Methanol enthalpy of vaporisation 35.27 kJ/mol at its normal boiling point. E. F. Fiock, D. C. Ginnings and W. B. Holton, 'Calorimetric determinations of thermal properties of methyl alcohol, ethyl alcohol and benzene', J. Research NBS 6 (1931) 881-900 (RP312), doi:10.6028/jres.006.054: 1100.7 international joules per gram at 64.7 degrees Celsius, which is 35.27 kJ/mol. The Journal of Research of the NBS is a United States Government publication and is not Standard Reference Data; RP312 carries no copyright notice anywhere. Read in full 2026-09-13. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. NBS Circular 500 independently prints 8.43 kcal/mol at 337.9 K, the same 35.27. THE VALUE MOVED, 35.21 to 35.27. The retired figure was the handbook's, and PubChem's entry for this quantity was checked as a replacement and rejected: it gives 37.34 kJ/mol at 25 degrees Celsius citing 'Haynes, W.M. (ed.). CRC Handbook of Chemistry and Physics. 95th Edition' through HSDB, which would have laundered the handbook rather than replaced it",
    },
    LatentHeat {
        species: "propanone",
        // 29.10 kJ/mol.
        kj_per_mol: 29.10,
        provenance: "Propanone (acetone) enthalpy of vaporisation 29.10 kJ/mol at its normal boiling point. CRC Handbook of Chemistry and Physics, 97th ed., \"Enthalpy of Fusion\" and \"Enthalpy of Vaporization\" tables - a COMMERCIAL COMPILATION, which this project cites happily but ranks below a primary measurement. PENDING REVIEW: no positively identified copy was opened for this row, so no edition-level page provenance is claimed. THIS IS THE ONE ROW IN THE TABLE THAT SEARCHING FAILED TO UPGRADE, and it is worth recording what was tried so nobody repeats it. The primary publication is J. Pennington and K. A. Kobe, J. Am. Chem. Soc. 79 (1957) 300, whose DOI resolves - but the value could not be read, and the pointer to it is itself inconsistent: the NIST WebBook attributes that paper's value to 338 K, about nine kelvin above the boiling point, while quoting a figure identical to the 329 K compilation value, which is thermodynamically impossible. J. H. Mathews, J. Am. Chem. Soc. 48 (1926) 562, doi:10.1021/ja01414a002, measured enthalpies of vaporisation at boiling points and is United States public domain by age, but no copy is reachable online. NBS Circular 500 cannot help: its own Preface limits it to compounds of one and two carbon atoms, and acetone has three. NASA CEA cannot help either - it carries no condensed acetone. An interlibrary scan of either paper is the whole of what stands between this row and a primary citation.",
    },
    LatentHeat {
        species: "isopropanol",
        // 39.85 kJ/mol.
        kj_per_mol: 39.85,
        provenance: "Propan-2-ol enthalpy of vaporisation 39.85 kJ/mol at its normal boiling point. J. L. Hales, J. D. Cox and E. B. Lees, 'Thermodynamic properties of propan-1-ol and propan-2-ol', Trans. Faraday Soc. 59 (1963) 1544-1555, doi:10.1039/TF9635901544. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. VERIFICATION IS INCOMPLETE AND THE ROW SAYS SO. The publication is the right one and its DOI resolves, but the value was read through the NIST WebBook's rendering of it rather than off the paper, so this citation names the work the number came from without yet having been checked against it. That is the same position kerotakis-thermo's ISOPROPANOL_PROVENANCE takes on this substance's Antoine constants, with the same lane: primary literature, pending review. An independent determination agreeing to 0.1 kJ/mol exists if a reviewer wants a second anchor - Berman, Larkam and McKetta, J. Chem. Eng. Data 9 (1964) 218, doi:10.1021/je60021a020",
    },
    LatentHeat {
        species: "hexane",
        // 28.85 kJ/mol.
        kj_per_mol: 28.85,
        provenance: "Hexane enthalpy of vaporisation 28.85 kJ/mol at its 68.742 degrees Celsius boiling point. NBS Circular 461, F. D. Rossini et al., Selected Values of Properties of Hydrocarbons, National Bureau of Standards, 1947; retrieved 2026-09-13 from https://archive.org/details/circularofbureau461ross. This is the edition of American Petroleum Institute Research Project 44 that the NBS issued through the Government Printing Office - masthead 'American Petroleum Institute Research Project 44 / National Bureau of Standards' - and the whole document contains no occurrence of the word copyright. The later Thermodynamics Research Center editions of API RP-44 are a separate and copyrighted product; this is not one of them. Table 2m prints 80.03 cal/g, equivalently 6.896 kcal/mol, at 68.742 degrees Celsius, which is 28853 J/mol - agreeing with the shipped value to one part in ten thousand. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. This is the cleanest row in the table: a public-domain United States Government document that prints the quantity at the temperature the bench uses it at, and no value change at all",
    },
    LatentHeat {
        species: "ethyl_acetate",
        // 31.94 kJ/mol.
        kj_per_mol: 31.94,
        provenance: "Ethyl acetate enthalpy of vaporisation 31.94 kJ/mol at its normal boiling point. J. E. Connett, J. F. Counsell and D. A. Lee, 'Thermodynamic properties of organic oxygen compounds: enthalpy of vaporization of ethyl acetate', J. Chem. Thermodyn. 8 (1976) 1199-1203, doi:10.1016/0021-9614(76)90129-4. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. VERIFICATION IS INCOMPLETE AND THE ROW SAYS SO, in the same way as propan-2-ol above: the publication is right and its DOI resolves, but the value was read through the NIST WebBook's rendering rather than off the paper. A public-domain corroboration was read in full and DISAGREES by two per cent, which is why it is recorded rather than adopted: A. C. Brown, J. Chem. Soc. Trans. 83 (1903) 987, doi:10.1039/CT9038300987, measured 88.37 cal/g at 77.3 degrees Celsius, which is 32.58 kJ/mol. A 1903 determination two per cent above a 1976 one is the expected direction and not a reason to move the value, but a reviewer should know both numbers exist",
    },
    LatentHeat {
        species: "CH3COOH",
        // 24.39 kJ/mol.
        kj_per_mol: 24.39,
        provenance: "Acetic acid enthalpy of vaporisation 24.39 kJ/mol at its 118.2 degrees Celsius boiling point. NBS Circular 500, F. D. Rossini et al., Selected Values of Chemical Thermodynamic Properties, National Bureau of Standards, 1952; retrieved 2026-09-13 from https://archive.org/details/circularofbureau500ross. A United States Government work and not Standard Reference Data - the Standard Reference Data Act notice that governs the NIST WebBook is absent from it - so it carries no copyright and is not on PLAN.md's avoid row. Its own Preface states the scope limit that makes it reach some of these rows and not others: it covers carbon compounds of one and two carbon atoms only. It prints 5.83 kcal/mol, which is 24392 J/mol. Kerotakis transcribes one factual measurement and redistributes no source text or table, the footing `literature/hartley-campbell-iodine-water` already stands on in the registry. INDEPENDENTLY CONFIRMED by a second document read in full and licensed CC0 by age: A. C. Brown, J. Chem. Soc. Trans. 83 (1903) 987, doi:10.1039/CT9038300987, measured 97.05 cal/g at the boiling point, which is also 24.39 kJ/mol. THE VALUE MOVED, 23.70 to 24.39, and this is the largest correction in the table at 2.9 per cent. The retired 23.70 was not a rounding of anything here: it traces to the CRC Handbook, and PubChem proves the chain rather than breaking it - PubChem's acetic acid record prints '23.70 kJ/mol at 117.9 degrees Celsius' citing 'Haynes, W.M. (ed.). CRC Handbook of Chemistry and Physics. 94th Edition' through HSDB. That is the shipped value, verbatim, with the avoid-row source named. The number still looks too small for a hydrogen-bonded liquid and the reason is chemistry rather than error: acetic acid vapour is largely the cyclic dimer, so half the hydrogen bonds survive the boil and are never paid for. Its Trouton entropy is about 62 J/(mol.K) against a normal 85, which is the tell. WHAT THIS NUMBER IS NOT: it is the enthalpy per mole of monomer to the REAL, largely dimeric vapour. Vaporisation to ideal MONOMER gas is about 51.6 kJ/mol (Konicek and Wadso, Acta Chem. Scand. 24 (1970) 2612, doi:10.3891/acta.chem.scand.24-2612, open access). This bench releases the vapour as monomeric CH3COOH because that is the only acetic acid the registry carries, and the dimer is not modelled - so the energy is right for the boil and the species is not, and any future path that treats the vapour as ideal monomer needs the larger number",
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
