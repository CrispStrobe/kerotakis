//! Reducing-sugar tests: Fehling/Benedict-type copper(II) reduction to red
//! copper(I) oxide, and the Tollens silver mirror.
//!
//! An aldose (and a ketose, which hot alkali isomerises to aldoses through
//! the Lobry de Bruyn–Alberda van Ekenstein enediol) reduces alkaline
//! copper(II) to Cu₂O and ammoniacal silver(I) to metallic silver. Sucrose
//! and starch carry no free anomeric carbon and do neither; that negative is
//! the half of the test a learner is usually asked to see, and here it
//! follows from the sugar list rather than from a rule about sucrose.
//!
//! WHAT IS CLAIMED: the qualitative outcome and an atom- and charge-balanced
//! limiting-reagent stoichiometry, two electrons per sugar. WHAT IS NOT: the
//! real oxidation of a sugar by hot alkaline copper is a mixture (sugar acids,
//! chain-cleavage acids, more than two Cu per sugar); this route books one
//! representative product, gluconic acid, held molecular rather than as the
//! gluconate the alkali would make of it. No rate law, no reaction heat and no
//! calibrated yield are claimed, and the onset temperature is an editorial
//! "hot water bath" threshold. Every firing says so through a typed
//! `NotYetModeled` event.
//!
//! Chemistry: the standard textbook equations of the Fehling, Benedict and
//! Tollens tests (e.g. Vogel's Textbook of Practical Organic Chemistry, 5th
//! ed., §9.6.3), balanced here against the registry formulas.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::Phrase;
use crate::species::{Phase, SpeciesData, BASE_EQUIVALENTS};
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

/// The booked oxidation product. Not in the reviewed registry: it exists
/// only so that the copper and silver this route reduces have somewhere to
/// put the electrons' partner atoms. See [`PRODUCT_SPECIES`].
pub const GLUCONIC_ACID: &str = "gluconic_acid";

/// Sugars with a free (or alkali-released) carbonyl that this route admits.
/// Maltose and lactose are reducing too, but their oxidation products
/// (maltobionic and lactobionic acid) are not booked by this lab, so they
/// are left unreacted rather than misbooked as gluconic acid.
const REDUCING_SUGARS: &[&str] = &["glucose", "fructose"];

/// Editorial onset: the test is run in a hot water bath. Not a rate law.
pub const ONSET_K: f64 = 333.15;

const TRACE: f64 = 1e-12;

pub static PRODUCT_SPECIES: &[SpeciesData] = &[SpeciesData {
    key: GLUCONIC_ACID,
    name: "gluconic acid",
    formula: "C6H12O7",
    inchikey: "",
    // C6H12O7 from CIAAW 2021 standard atomic weights.
    molar_mass: 196.155,
    // Not priced: a dissolved product of a route that claims no heat.
    heat_capacity: 0.0,
    heat_capacity_polys: &[],
    // Placeholder for dissolved-volume accounting only, as for the enzyme
    // catalysts; no pure-substance density is claimed.
    density: 1.35,
    standard_phase: Phase::Aqueous,
    appearance: Some("colourless"),
    flame_colour: None,
    colour: None,
    spectrum: None,
    dissolution_enthalpy_kj: None,
    dissolves_without_speciation: true,
    aqueous_solubility_g_per_100_ml: None,
    aqueous_solubility_g_per_100_ml_at_100c: None,
    forms_only_above_k: None,
    magnetic: false,
    transitions: None,
    electrical_resistivity: None,
    provenance: "D-gluconic acid, C6H12O7 (PubChem CID 10690); molar mass from CIAAW 2021 atomic weights. Booked only as the representative product of the qualitative reducing-sugar route; held molecular, not speciated, with no heat capacity or pure-phase density claim",
}];

pub(crate) fn species_data(key: &str) -> Option<&'static SpeciesData> {
    PRODUCT_SPECIES.iter().find(|species| species.key == key)
}

/// Solids this route puts at the bottom of the tube.
pub fn solid_product(species: &SpeciesId) -> bool {
    matches!(species.0.as_str(), "Cu2O" | "Ag")
}

fn phase_moles(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn dissolved(vessel: &Vessel, key: &str) -> f64 {
    phase_moles(vessel, key, Phase::Aqueous) + phase_moles(vessel, key, Phase::Liquid)
}

fn has_water(vessel: &Vessel) -> bool {
    phase_moles(vessel, "water", Phase::Liquid) > TRACE
}

/// Hydroxide-bearing medium with no unspent acid: the base the copper test
/// needs. Booked as the analytical basis term after a solve, or as the
/// literal strong base before one.
pub(crate) fn alkaline(vessel: &Vessel) -> bool {
    crate::displacement::unspent_acidity(vessel) <= TRACE
        && ["base_equivalents", "OH-", "NaOH", "KOH"]
            .iter()
            .any(|key| dissolved(vessel, key) > TRACE)
}

fn base_moles(vessel: &Vessel) -> f64 {
    dissolved(vessel, BASE_EQUIVALENTS) + dissolved(vessel, "OH-")
}

fn sugar_moles(vessel: &Vessel) -> f64 {
    REDUCING_SUGARS.iter().map(|s| dissolved(vessel, s)).sum()
}

/// Withdraw `moles` of reducing sugar, glucose first.
fn withdraw_sugar(vessel: &mut Vessel, mut moles: f64) {
    for sugar in REDUCING_SUGARS {
        if moles <= 0.0 {
            break;
        }
        let id = SpeciesId::new(sugar);
        for phase in [Phase::Aqueous, Phase::Liquid] {
            moles -= vessel.withdraw_phase(&id, Moles(moles), phase).0;
        }
    }
}

/// Whether either test can fire in this vessel now.
pub fn fires(vessel: &Vessel) -> bool {
    if vessel.temperature.0 < ONSET_K || !has_water(vessel) || sugar_moles(vessel) <= TRACE {
        return false;
    }
    copper_extent(vessel) > TRACE || silver_extent(vessel) > TRACE
}

fn copper_extent(vessel: &Vessel) -> f64 {
    if !alkaline(vessel) {
        return 0.0;
    }
    let solid =
        phase_moles(vessel, "Cu(OH)2", Phase::Solid) + phase_moles(vessel, "CuO", Phase::Solid);
    let ion = phase_moles(vessel, "Cu+2", Phase::Aqueous).min(base_moles(vessel) / 2.0);
    sugar_moles(vessel).min((solid + ion) / 2.0)
}

fn silver_extent(vessel: &Vessel) -> f64 {
    let silver = phase_moles(vessel, "Ag+", Phase::Aqueous);
    let ammonia = dissolved(vessel, "NH3");
    sugar_moles(vessel).min(silver / 2.0).min(ammonia / 2.0)
}

fn boundary_note(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.reducing-sugar-qualitative",
            "this reducing-sugar test is a qualitative curated route, not calibrated: the sugar is booked as oxidised to gluconic acid, held molecular rather than as gluconate, although the real product is a mixture; the onset is an editorial hot-water-bath threshold, and no rate, no reaction heat and no measured yield are claimed",
        ),
    )
}

/// Apply both tests to completion of the limiting reagent. Returns whether
/// anything reacted.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !fires(vessel) {
        return false;
    }
    let mut reacted = false;

    // Copper(II), solids first: hydroxide, then the oxide hot alkali makes
    // of it, then dissolved (e.g. citrate- or tartrate-held) Cu²⁺.
    let mut left = copper_extent(vessel);
    if left > TRACE {
        reacted = true;
        for (source, water_out, equation) in [
            (
                "Cu(OH)2",
                2.0,
                "C₆H₁₂O₆ + 2 Cu(OH)₂ → C₆H₁₂O₇ + Cu₂O↓ + 2 H₂O",
            ),
            ("CuO", 0.0, "C₆H₁₂O₆ + 2 CuO → C₆H₁₂O₇ + Cu₂O↓"),
        ] {
            let x = left.min(phase_moles(vessel, source, Phase::Solid) / 2.0);
            if x <= TRACE {
                continue;
            }
            withdraw_sugar(vessel, x);
            vessel.withdraw_phase(&SpeciesId::new(source), Moles(2.0 * x), Phase::Solid);
            vessel.deposit(SpeciesId::new(GLUCONIC_ACID), Moles(x), Phase::Aqueous);
            vessel.deposit(SpeciesId::new("Cu2O"), Moles(x), Phase::Solid);
            if water_out > 0.0 {
                vessel.deposit(SpeciesId::new("water"), Moles(water_out * x), Phase::Liquid);
            }
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: equation.to_string(),
            });
            left -= x;
        }
        let x = left
            .min(phase_moles(vessel, "Cu+2", Phase::Aqueous) / 2.0)
            .min(base_moles(vessel) / 4.0);
        if x > TRACE {
            withdraw_sugar(vessel, x);
            vessel.withdraw_phase(&SpeciesId::new("Cu+2"), Moles(2.0 * x), Phase::Aqueous);
            let mut base = 4.0 * x;
            for key in [BASE_EQUIVALENTS, "OH-"] {
                base -= vessel
                    .withdraw_phase(&SpeciesId::new(key), Moles(base), Phase::Aqueous)
                    .0;
            }
            vessel.deposit(SpeciesId::new(GLUCONIC_ACID), Moles(x), Phase::Aqueous);
            vessel.deposit(SpeciesId::new("Cu2O"), Moles(x), Phase::Solid);
            vessel.deposit(SpeciesId::new("water"), Moles(2.0 * x), Phase::Liquid);
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: "C₆H₁₂O₆ + 2 Cu²⁺ + 4 OH⁻ → C₆H₁₂O₇ + Cu₂O↓ + 2 H₂O".to_string(),
            });
        }
    }

    // Tollens: ammoniacal silver(I) to a silver mirror. The ammonia takes
    // the protons the oxidation releases.
    let x = silver_extent(vessel);
    if x > TRACE {
        reacted = true;
        withdraw_sugar(vessel, x);
        vessel.withdraw_phase(&SpeciesId::new("Ag+"), Moles(2.0 * x), Phase::Aqueous);
        let mut nh3 = 2.0 * x;
        for phase in [Phase::Aqueous, Phase::Liquid] {
            nh3 -= vessel
                .withdraw_phase(&SpeciesId::new("NH3"), Moles(nh3), phase)
                .0;
        }
        vessel.withdraw_phase(&SpeciesId::new("water"), Moles(x), Phase::Liquid);
        vessel.deposit(SpeciesId::new(GLUCONIC_ACID), Moles(x), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Ag"), Moles(2.0 * x), Phase::Solid);
        vessel.deposit(SpeciesId::new("NH4+"), Moles(2.0 * x), Phase::Aqueous);
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: "C₆H₁₂O₆ + 2 Ag⁺ + 2 NH₃ + H₂O → C₆H₁₂O₇ + 2 Ag↓ + 2 NH₄⁺".to_string(),
        });
    }

    if reacted {
        events.push(boundary_note(vessel));
    }
    reacted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VesselId;

    fn hot(mut vessel: Vessel) -> Vessel {
        vessel.temperature = crate::units::Kelvin(363.15);
        vessel
    }

    fn fehling(sugar: &str) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "test tube");
        v.deposit(SpeciesId::new("water"), Moles(0.5), Phase::Liquid);
        v.deposit(SpeciesId::new(sugar), Moles(0.001), Phase::Aqueous);
        v.deposit(SpeciesId::new("Cu(OH)2"), Moles(0.0005), Phase::Solid);
        v.deposit(SpeciesId::new("Na+"), Moles(0.003), Phase::Aqueous);
        v.deposit(
            SpeciesId::new(BASE_EQUIVALENTS),
            Moles(0.002),
            Phase::Aqueous,
        );
        v.deposit(SpeciesId::new("SO4-2"), Moles(0.0005), Phase::Aqueous);
        hot(v)
    }

    #[test]
    fn glucose_and_fructose_reduce_copper_to_cuprous_oxide() {
        for sugar in ["glucose", "fructose"] {
            let mut v = fehling(sugar);
            let mut events = Vec::new();
            assert!(apply(&mut v, &mut events), "{sugar}");
            assert!((phase_moles(&v, "Cu2O", Phase::Solid) - 0.00025).abs() < 1e-12);
            assert!(phase_moles(&v, "Cu(OH)2", Phase::Solid) < 1e-12);
            assert!(events
                .iter()
                .any(|e| matches!(e, Event::NotYetModeled { .. })));
        }
    }

    #[test]
    fn sucrose_and_starch_are_negative() {
        for sugar in ["sucrose", "starch", "maltose"] {
            let mut v = fehling("glucose");
            v.contents.retain(|p| p.species.0 != "glucose");
            v.deposit(SpeciesId::new(sugar), Moles(0.001), Phase::Aqueous);
            let mut events = Vec::new();
            assert!(!apply(&mut v, &mut events), "{sugar}");
            assert_eq!(phase_moles(&v, "Cu2O", Phase::Solid), 0.0);
        }
    }

    #[test]
    fn cold_or_acidic_copper_does_not_react() {
        let mut cold = fehling("glucose");
        cold.temperature = crate::units::Kelvin(298.15);
        assert!(!fires(&cold));
        let mut neutral = fehling("glucose");
        neutral.contents.retain(|p| p.species.0 != BASE_EQUIVALENTS);
        assert!(!fires(&neutral));
    }

    #[test]
    fn dissolved_copper_draws_base_and_conserves_charge() {
        let mut v = fehling("glucose");
        v.contents.retain(|p| p.species.0 != "Cu(OH)2");
        v.deposit(SpeciesId::new("Cu+2"), Moles(0.0005), Phase::Aqueous);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!((phase_moles(&v, "Cu2O", Phase::Solid) - 0.00025).abs() < 1e-12);
        assert!((phase_moles(&v, BASE_EQUIVALENTS, Phase::Aqueous) - 0.001).abs() < 1e-12);
    }

    #[test]
    fn tollens_mirror_needs_ammonia_and_a_reducing_sugar() {
        let tollens = |sugar: &str, nh3: f64| {
            let mut v = Vessel::new(VesselId(0), "test tube");
            v.deposit(SpeciesId::new("water"), Moles(0.15), Phase::Liquid);
            v.deposit(SpeciesId::new("Ag+"), Moles(0.0003), Phase::Aqueous);
            v.deposit(SpeciesId::new("NO3-"), Moles(0.0003), Phase::Aqueous);
            if nh3 > 0.0 {
                v.deposit(SpeciesId::new("NH3"), Moles(nh3), Phase::Aqueous);
            }
            v.deposit(SpeciesId::new(sugar), Moles(0.0005), Phase::Aqueous);
            hot(v)
        };
        let mut v = tollens("glucose", 0.002);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!((phase_moles(&v, "Ag", Phase::Solid) - 0.0003).abs() < 1e-12);
        assert!((phase_moles(&v, "NH4+", Phase::Aqueous) - 0.0003).abs() < 1e-12);
        assert!(!fires(&tollens("sucrose", 0.002)));
        assert!(!fires(&tollens("glucose", 0.0)));
    }

    #[test]
    fn product_formula_balances_the_written_equations() {
        let data = species_data(GLUCONIC_ACID).unwrap();
        assert_eq!(data.formula, "C6H12O7");
        let glucose = crate::species::lookup_key("glucose").unwrap().molar_mass;
        let o = 15.999;
        assert!((data.molar_mass - (glucose + o)).abs() < 0.01);
    }
}
