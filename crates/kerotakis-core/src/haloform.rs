//! The iodoform (haloform) test, table-driven over its substrates
//! (2026-11-28 sweep, gap S4).
//!
//! A methyl ketone `CH₃CO–R`, or an alcohol that iodine in alkali first
//! oxidises to one (`CH₃CH(OH)–R`: ethanol, propan-2-ol), is iodinated three
//! times on the methyl group and then cleaved by hydroxide. The products are
//! triiodomethane, CHI₃ — the yellow, antiseptic-smelling precipitate the test
//! is named after — and the carboxylate of the rest of the molecule. Methanol
//! has no carbon to put next to the methyl group, so it is the classical
//! negative, and here it is negative because it has no row, not because of a
//! rule about methanol.
//!
//! Each row is the substrate's balanced overall equation (Vogel's Textbook of
//! Practical Organic Chemistry, 5th ed.; R. C. Fuson and B. A. Bull,
//! "The Haloform Reaction", Chem. Rev. 15 (1934) 275,
//! doi:10.1021/cr60052a001), written against the registry formulas and
//! checked by a test for atoms and charge. A new substrate is a new row.
//!
//! WHAT IS CLAIMED: the qualitative outcome and a limiting-reagent
//! stoichiometry. WHAT IS NOT: no rate, no reaction heat and no yield; the
//! onset is an editorial warm-water-bath threshold; the hypoiodite
//! intermediate is not booked; the formate from ethanol is held as molecular
//! formic acid (the lab has no formate ion), and the solid's solubility is not
//! modelled. Every firing says so through a typed `NotYetModeled` event.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::Phrase;
use crate::species::{Phase, SpeciesData, BASE_EQUIVALENTS};
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

/// The precipitate, a registry species.
pub const IODOFORM: &str = "CHI3";

/// Ethanol's cleavage product. Not in the reviewed registry: booked so the
/// carbon atom the cleavage releases has somewhere to go. See
/// [`PRODUCT_SPECIES`].
pub const FORMIC_ACID: &str = "formic_acid";

/// Editorial onset: the test is warmed in a water bath. Not a rate law.
pub const ONSET_K: f64 = 323.15;

const TRACE: f64 = 1e-12;

/// One haloform substrate and its balanced overall equation
/// `substrate + iodine I₂ + hydroxide OH⁻ → CHI₃ + carboxylate + iodide I⁻ + water H₂O`.
pub struct Substrate {
    /// Registry key of the substrate.
    pub key: &'static str,
    pub iodine: f64,
    pub hydroxide: f64,
    pub iodide: f64,
    pub water: f64,
    /// The cleavage product, booked aqueous.
    pub carboxylate: &'static str,
    pub equation: &'static str,
}

pub const SUBSTRATES: &[Substrate] = &[
    // Methyl ketone: three iodinations, then cleavage to acetate.
    Substrate {
        key: "propanone",
        iodine: 3.0,
        hydroxide: 4.0,
        iodide: 3.0,
        water: 3.0,
        carboxylate: "CH3COO-",
        equation: "CH₃COCH₃ + 3 I₂ + 4 OH⁻ → CHI₃↓ + CH₃COO⁻ + 3 I⁻ + 3 H₂O",
    },
    // Secondary alcohol: one further I₂ and two OH⁻ to oxidise it to
    // propanone first.
    Substrate {
        key: "isopropanol",
        iodine: 4.0,
        hydroxide: 6.0,
        iodide: 5.0,
        water: 5.0,
        carboxylate: "CH3COO-",
        equation: "CH₃CH(OH)CH₃ + 4 I₂ + 6 OH⁻ → CHI₃↓ + CH₃COO⁻ + 5 I⁻ + 5 H₂O",
    },
    // Primary alcohol oxidised to acetaldehyde, then cleaved to formate,
    // booked here as molecular formic acid (one OH⁻ fewer).
    Substrate {
        key: "ethanol",
        iodine: 4.0,
        hydroxide: 5.0,
        iodide: 5.0,
        water: 4.0,
        carboxylate: FORMIC_ACID,
        equation: "C₂H₅OH + 4 I₂ + 5 OH⁻ → CHI₃↓ + HCOOH + 5 I⁻ + 4 H₂O",
    },
];

pub static PRODUCT_SPECIES: &[SpeciesData] = &[SpeciesData {
    key: FORMIC_ACID,
    name: "formic acid",
    formula: "HCOOH",
    inchikey: "",
    // CH2O2 from CIAAW 2021 standard atomic weights.
    molar_mass: 46.025,
    // Not priced: a dissolved product of a route that claims no heat.
    heat_capacity: 0.0,
    heat_capacity_polys: &[],
    // Placeholder for dissolved-volume accounting only (as for gluconic
    // acid); no pure-substance density is claimed.
    density: 1.22,
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
    provenance: "formic acid, HCOOH (PubChem CID 284, InChIKey BDAGIHXWWSANSR-UHFFFAOYSA-N); molar mass from CIAAW 2021 atomic weights. Booked only as the cleavage product of ethanol in the qualitative haloform route; held molecular, not speciated as formate, with no heat capacity or pure-phase density claim",
}];

pub(crate) fn species_data(key: &str) -> Option<&'static SpeciesData> {
    PRODUCT_SPECIES.iter().find(|species| species.key == key)
}

pub fn substrate(key: &str) -> Option<&'static Substrate> {
    SUBSTRATES.iter().find(|s| s.key == key)
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

fn iodine(vessel: &Vessel) -> f64 {
    dissolved(vessel, "I2") + phase_moles(vessel, "I2", Phase::Solid)
}

fn base(vessel: &Vessel) -> f64 {
    dissolved(vessel, BASE_EQUIVALENTS) + dissolved(vessel, "OH-")
}

/// The yellow solid at the bottom of the tube is this route's.
pub fn produced_solid(vessel: &Vessel, species: &SpeciesId) -> bool {
    species.0 == IODOFORM && phase_moles(vessel, IODOFORM, Phase::Solid) > TRACE
}

fn extent(vessel: &Vessel, row: &Substrate) -> f64 {
    dissolved(vessel, row.key)
        .min(iodine(vessel) / row.iodine)
        .min(base(vessel) / row.hydroxide)
}

/// Whether any substrate can react in this vessel now.
pub fn fires(vessel: &Vessel) -> bool {
    vessel.temperature.0 >= ONSET_K
        && phase_moles(vessel, "water", Phase::Liquid) > TRACE
        && crate::displacement::unspent_acidity(vessel) <= TRACE
        && SUBSTRATES.iter().any(|row| extent(vessel, row) > TRACE)
}

/// Withdraw up to `moles` across `phases` in order; returns what was taken.
fn withdraw_any(vessel: &mut Vessel, key: &str, moles: f64, phases: &[Phase]) -> f64 {
    let id = SpeciesId::new(key);
    let mut taken = 0.0;
    for phase in phases {
        if moles - taken <= 0.0 {
            break;
        }
        taken += vessel.withdraw_phase(&id, Moles(moles - taken), *phase).0;
    }
    taken
}

fn boundary_note(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.haloform-qualitative",
            "this iodoform test is a qualitative curated route, not calibrated: the substrate is booked to completion of the limiting reagent as iodoform and its carboxylate (ethanol's formate held as molecular formic acid), the hypoiodite intermediate is not booked, the onset is an editorial warm-water-bath threshold, and no rate, no reaction heat and no measured yield are claimed",
        ),
    )
}

/// Run every row to completion of its limiting reagent, in table order.
/// Returns whether anything reacted.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    if !fires(vessel) {
        return false;
    }
    let mut reacted = false;
    for row in SUBSTRATES {
        let x = extent(vessel, row);
        if x <= TRACE {
            continue;
        }
        reacted = true;
        let _ = withdraw_any(vessel, row.key, x, &[Phase::Aqueous, Phase::Liquid]);
        let _ = withdraw_any(
            vessel,
            "I2",
            row.iodine * x,
            &[Phase::Aqueous, Phase::Liquid, Phase::Solid],
        );
        let mut oh = row.hydroxide * x;
        for key in [BASE_EQUIVALENTS, "OH-"] {
            oh -= withdraw_any(vessel, key, oh, &[Phase::Aqueous, Phase::Liquid]);
        }
        vessel.deposit(SpeciesId::new(IODOFORM), Moles(x), Phase::Solid);
        vessel.deposit(SpeciesId::new(row.carboxylate), Moles(x), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("I-"), Moles(row.iodide * x), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("water"), Moles(row.water * x), Phase::Liquid);
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: row.equation.to_string(),
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

    fn tube(substrate: &str, kelvin: f64, base_moles: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "test tube");
        v.deposit(SpeciesId::new("water"), Moles(0.55), Phase::Liquid);
        v.deposit(SpeciesId::new(substrate), Moles(0.002), Phase::Liquid);
        v.deposit(SpeciesId::new("I2"), Moles(0.003), Phase::Aqueous);
        v.deposit(SpeciesId::new("K+"), Moles(0.006), Phase::Aqueous);
        v.deposit(SpeciesId::new("I-"), Moles(0.006), Phase::Aqueous);
        if base_moles > 0.0 {
            v.deposit(SpeciesId::new("Na+"), Moles(base_moles), Phase::Aqueous);
            v.deposit(
                SpeciesId::new(BASE_EQUIVALENTS),
                Moles(base_moles),
                Phase::Aqueous,
            );
        }
        v.temperature = crate::units::Kelvin(kelvin);
        v
    }

    fn formula_of(key: &str) -> crate::stoich::Formula {
        let text = match key {
            "OH-" => "OH-",
            "I-" => "I-",
            _ => {
                crate::species::lookup_key(key)
                    .unwrap_or_else(|| panic!("{key} is a known species"))
                    .formula
            }
        };
        crate::stoich::parse_formula(text).unwrap()
    }

    /// Every row balances atoms and charge against the registry formulas.
    #[test]
    fn every_row_balances_atoms_and_charge() {
        for row in SUBSTRATES {
            let mut net: std::collections::BTreeMap<String, f64> = Default::default();
            let mut charge = 0.0;
            let mut add = |key: &str, n: f64| {
                let f = formula_of(key);
                for (el, c) in &f.counts {
                    *net.entry(el.clone()).or_default() += n * c;
                }
                charge += n * f.charge;
            };
            add(row.key, -1.0);
            add("I2", -row.iodine);
            add("OH-", -row.hydroxide);
            add(IODOFORM, 1.0);
            add(row.carboxylate, 1.0);
            add("I-", row.iodide);
            add("water", row.water);
            for (el, n) in &net {
                assert!(n.abs() < 1e-9, "{}: {el} off by {n}", row.key);
            }
            assert!(charge.abs() < 1e-9, "{}: charge off by {charge}", row.key);
        }
    }

    #[test]
    fn ethanol_propanone_and_propan_2_ol_give_iodoform() {
        for (substrate, iodine_per) in [("ethanol", 4.0), ("propanone", 3.0), ("isopropanol", 4.0)]
        {
            let mut v = tube(substrate, 333.15, 0.01);
            let mut events = Vec::new();
            assert!(apply(&mut v, &mut events), "{substrate}");
            // Iodine limits: 3 mmol over the row's I2 coefficient.
            let expected = 0.003 / iodine_per;
            let got = phase_moles(&v, IODOFORM, Phase::Solid);
            assert!((got - expected).abs() < 1e-12, "{substrate}: {got}");
            assert!(produced_solid(&v, &SpeciesId::new(IODOFORM)));
            assert!(events
                .iter()
                .any(|e| matches!(e, Event::NotYetModeled { .. })));
        }
    }

    /// Methanol has no CH₃–C unit: no row, so no iodoform.
    #[test]
    fn methanol_is_negative() {
        let mut v = tube("methanol", 333.15, 0.01);
        assert!(!fires(&v));
        let mut events = Vec::new();
        assert!(!apply(&mut v, &mut events));
        assert_eq!(phase_moles(&v, IODOFORM, Phase::Solid), 0.0);
    }

    #[test]
    fn cold_or_base_free_ethanol_does_not_react() {
        assert!(!fires(&tube("ethanol", 298.15, 0.01)));
        assert!(!fires(&tube("ethanol", 333.15, 0.0)));
    }

    #[test]
    fn iodoform_is_a_cited_yellow_registry_solid() {
        let data = crate::species::lookup_key(IODOFORM).expect("CHI3 registry row");
        assert_eq!(data.formula, "CHI3");
        assert_eq!(data.inchikey, "OKJPEAGHQZHRQV-UHFFFAOYSA-N");
        assert!((data.molar_mass - 393.732).abs() < 1e-9);
        assert!((data.density - 4.008).abs() < 1e-9);
        assert_eq!(data.appearance, Some("yellow"));
        assert_eq!(data.standard_phase, Phase::Solid);
    }
}
