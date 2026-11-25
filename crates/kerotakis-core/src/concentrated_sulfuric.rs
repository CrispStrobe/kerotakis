//! Concentrated sulfuric acid as a dehydrating agent, a non-volatile acid
//! and, hot, an oxidant — and hydrogen sulfide oxidised by peroxide.
//!
//! Before this route, a beaker of concentrated H2SO4 was inert: the aqueous
//! model rightly refuses the medium (`not-modeled.concentrated-acid-medium`,
//! water a minority of the liquid), and nothing else claimed it. So sugar did
//! not char, table salt gave no hydrogen chloride, and copper, carbon and
//! sulfur sat unchanged in boiling acid (C1 E067, E084, E143, E144, E147,
//! E148, E333, E364).
//!
//! WHAT IS CLAIMED: the qualitative outcome of the standard textbook
//! reactions (e.g. Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed.,
//! §15.2.5 "sulfuric acid"; Holleman-Wiberg, Inorganic Chemistry, 101st ed.,
//! "Schwefelsäure"), each written as an atom- and charge-balanced
//! limiting-reagent stoichiometry:
//!
//! * dehydration of carbohydrates to carbon and water,
//!   C12H22O11 → 12 C + 11 H2O (sucrose; glucose/fructose, starch and
//!   cellulose by the same rule per formula unit);
//! * displacement of the volatile acid, NaCl + H2SO4 → NaHSO4 + HCl↑;
//! * hot oxidation to sulfur dioxide, Cu + 2 H2SO4 → CuSO4 + SO2↑ + 2 H2O,
//!   C + 2 H2SO4 → CO2↑ + 2 SO2↑ + 2 H2O, S + 2 H2SO4 → 3 SO2↑ + 2 H2O;
//! * H2S + H2O2 → S↓ + 2 H2O in water.
//!
//! WHAT IS NOT: no rate, no reaction heat and no measured yield. "Concentrated"
//! is the bench's own existing editorial admission boundary (water below half
//! the liquid, [`crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR`]), and the
//! dehydrating and oxidising rows stop where their own product water would
//! carry the acid across that boundary — the acid is spent as a dehydrating
//! agent when it is diluted, not when a measured capacity says so. "Hot" is an
//! editorial onset, [`HOT_ONSET_K`], not a measured threshold. Every firing
//! says so through a typed `NotYetModeled` event.

use crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR;
use crate::ops::{Event, NotModelledCause};
use crate::phrase::Phrase;
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

/// Editorial onset of the "hot concentrated acid" oxidations: the boiling
/// point of water at one atmosphere, i.e. acid heated well past anything a
/// water bath gives. Not a measured threshold and not a rate law.
pub const HOT_ONSET_K: f64 = 373.15;

const TRACE: f64 = 1e-12;
const ACID: &str = "H2SO4";

/// Carbohydrates the acid chars, with (carbon, water) per formula unit.
const CARBOHYDRATES: &[(&str, f64, f64)] = &[
    ("sucrose", 12.0, 11.0),
    ("maltose", 12.0, 11.0),
    ("glucose", 6.0, 6.0),
    ("fructose", 6.0, 6.0),
    ("starch", 6.0, 5.0),
    ("cellulose", 6.0, 5.0),
];

/// Reducers hot concentrated acid oxidises: (key, H2SO4 per unit, equation,
/// products). Graphite and diamond are not admitted: their resistance to the
/// acid is the reason they are not on the list, not an omission.
struct Oxidation {
    key: &'static str,
    acid: f64,
    water: f64,
    equation: &'static str,
    products: &'static [(&'static str, f64, Phase)],
}

const OXIDATIONS: &[Oxidation] = &[
    Oxidation {
        key: "Cu",
        acid: 2.0,
        water: 2.0,
        equation: "Cu + 2 H₂SO₄ →Δ CuSO₄ + SO₂↑ + 2 H₂O",
        products: &[("CuSO4", 1.0, Phase::Solid), ("SO2", 1.0, Phase::Gas)],
    },
    Oxidation {
        key: "C",
        acid: 2.0,
        water: 2.0,
        equation: "C + 2 H₂SO₄ →Δ CO₂↑ + 2 SO₂↑ + 2 H₂O",
        products: &[("CO2", 1.0, Phase::Gas), ("SO2", 2.0, Phase::Gas)],
    },
    Oxidation {
        key: "activated_charcoal",
        acid: 2.0,
        water: 2.0,
        equation: "C + 2 H₂SO₄ →Δ CO₂↑ + 2 SO₂↑ + 2 H₂O",
        products: &[("CO2", 1.0, Phase::Gas), ("SO2", 2.0, Phase::Gas)],
    },
    Oxidation {
        key: "S",
        acid: 2.0,
        water: 2.0,
        equation: "S + 2 H₂SO₄ →Δ 3 SO₂↑ + 2 H₂O",
        products: &[("SO2", 3.0, Phase::Gas)],
    },
];

fn phase_moles(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && phases.contains(&p.phase))
        .map(|p| p.moles.0)
        .sum()
}

/// Neat (undissolved) sulfuric acid in the liquid.
fn acid(vessel: &Vessel) -> f64 {
    phase_moles(vessel, ACID, &[Phase::Liquid])
}

/// Whether the vessel holds concentrated sulfuric acid by the bench's own
/// admission boundary: liquid H2SO4 present and water below
/// [`AQUEOUS_WATER_FRACTION_FLOOR`] of the competing liquids.
pub fn concentrated(vessel: &Vessel) -> bool {
    acid(vessel) > TRACE && !crate::nonaqueous::aqueous_model_admits(vessel)
}

/// Moles of product water the liquid can take before it crosses the
/// boundary: with water fraction w = W / (W + R), staying below the floor f
/// needs W ≤ R f / (1 − f).
fn water_room(vessel: &Vessel) -> f64 {
    let Some(fraction) = crate::nonaqueous::water_fraction_among_solvents(vessel) else {
        return 0.0;
    };
    let water = phase_moles(vessel, "water", &[Phase::Liquid, Phase::Aqueous]);
    let total = if fraction > 0.0 {
        water / fraction
    } else {
        acid(vessel)
    };
    let rest = (total - water).max(0.0);
    let f = AQUEOUS_WATER_FRACTION_FLOOR;
    (rest * f / (1.0 - f) - water).max(0.0)
}

fn dehydration_extent(vessel: &Vessel, key: &str, water: f64) -> f64 {
    phase_moles(vessel, key, &[Phase::Solid, Phase::Liquid, Phase::Aqueous])
        .min(water_room(vessel) / water)
}

fn chloride_extent(vessel: &Vessel) -> f64 {
    phase_moles(vessel, "NaCl", &[Phase::Solid]).min(acid(vessel))
}

fn oxidation_extent(vessel: &Vessel, row: &Oxidation) -> f64 {
    if vessel.temperature.0 < HOT_ONSET_K {
        return 0.0;
    }
    phase_moles(vessel, row.key, &[Phase::Solid, Phase::Liquid])
        .min(acid(vessel) / row.acid)
        .min(water_room(vessel) / row.water)
}

fn peroxide_extent(vessel: &Vessel) -> f64 {
    if phase_moles(vessel, "water", &[Phase::Liquid]) <= TRACE {
        return 0.0;
    }
    phase_moles(vessel, "H2S(aq)", &[Phase::Aqueous]).min(phase_moles(
        vessel,
        "H2O2",
        &[Phase::Liquid, Phase::Aqueous],
    ))
}

/// Whether any row can fire in this vessel now.
pub fn fires(vessel: &Vessel) -> bool {
    if peroxide_extent(vessel) > TRACE {
        return true;
    }
    if !concentrated(vessel) {
        return false;
    }
    CARBOHYDRATES
        .iter()
        .any(|(key, _, water)| dehydration_extent(vessel, key, *water) > TRACE)
        || chloride_extent(vessel) > TRACE
        || OXIDATIONS
            .iter()
            .any(|row| oxidation_extent(vessel, row) > TRACE)
}

fn withdraw(vessel: &mut Vessel, key: &str, moles: f64, phases: &[Phase]) {
    let id = SpeciesId::new(key);
    let mut left = moles;
    for phase in phases {
        if left <= 0.0 {
            break;
        }
        left -= vessel.withdraw_phase(&id, Moles(left), *phase).0;
    }
}

fn release_gas(vessel: &mut Vessel, key: &str, moles: f64, events: &mut Vec<Event>) {
    let species = SpeciesId::new(key);
    let n = Moles(moles);
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
}

fn concentrated_note(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.concentrated-sulfuric-qualitative",
            "this reaction of concentrated sulfuric acid is a qualitative curated route, not calibrated: the textbook stoichiometry is booked to the limiting reagent, the acid counts as concentrated only while water stays below half of the liquid (an editorial boundary, which also ends the dehydration once the product water has diluted it), \"hot\" is an editorial onset at the boiling point of water, and no rate, no reaction heat and no measured yield are claimed",
        ),
    )
}

fn peroxide_note(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.sulfide-peroxide-qualitative",
            "hydrogen peroxide oxidising dissolved hydrogen sulfide to sulfur is a qualitative curated route, not calibrated: it is booked to completion of the limiting reagent as elemental sulfur, although further oxidation to sulfate is possible, and no rate, no reaction heat and no measured yield are claimed",
        ),
    )
}

/// Apply every row to its limiting reagent. Returns whether anything reacted.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let mut concentrated_fired = false;
    let mut peroxide_fired = false;

    let x = peroxide_extent(vessel);
    if x > TRACE {
        withdraw(vessel, "H2S(aq)", x, &[Phase::Aqueous]);
        withdraw(vessel, "H2O2", x, &[Phase::Aqueous, Phase::Liquid]);
        vessel.deposit(SpeciesId::new("S"), Moles(x), Phase::Solid);
        vessel.deposit(SpeciesId::new("water"), Moles(2.0 * x), Phase::Liquid);
        events.push(Event::ReactionOccurred {
            vessel: vessel.id,
            equation: "H₂S + H₂O₂ → S↓ + 2 H₂O".to_string(),
        });
        peroxide_fired = true;
    }

    if concentrated(vessel) {
        for (key, carbon, water) in CARBOHYDRATES {
            let x = dehydration_extent(vessel, key, *water);
            if x <= TRACE {
                continue;
            }
            withdraw(
                vessel,
                key,
                x,
                &[Phase::Solid, Phase::Liquid, Phase::Aqueous],
            );
            vessel.deposit(SpeciesId::new("C"), Moles(carbon * x), Phase::Solid);
            vessel.deposit(SpeciesId::new("water"), Moles(water * x), Phase::Liquid);
            let formula = crate::species::lookup_key(key)
                .map(|d| d.formula)
                .unwrap_or(key);
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: format!("{formula} →(H₂SO₄) {carbon} C + {water} H₂O"),
            });
            concentrated_fired = true;
        }

        let x = chloride_extent(vessel);
        if x > TRACE {
            withdraw(vessel, "NaCl", x, &[Phase::Solid]);
            withdraw(vessel, ACID, x, &[Phase::Liquid]);
            vessel.deposit(SpeciesId::new("NaHSO4"), Moles(x), Phase::Solid);
            release_gas(vessel, "HCl", x, events);
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: "NaCl + H₂SO₄ → NaHSO₄ + HCl↑".to_string(),
            });
            concentrated_fired = true;
        }

        for row in OXIDATIONS {
            let x = oxidation_extent(vessel, row);
            if x <= TRACE {
                continue;
            }
            withdraw(vessel, row.key, x, &[Phase::Solid, Phase::Liquid]);
            withdraw(vessel, ACID, row.acid * x, &[Phase::Liquid]);
            vessel.deposit(SpeciesId::new("water"), Moles(row.water * x), Phase::Liquid);
            for (key, coeff, phase) in row.products {
                if *phase == Phase::Gas {
                    release_gas(vessel, key, coeff * x, events);
                } else {
                    vessel.deposit(SpeciesId::new(key), Moles(coeff * x), *phase);
                }
            }
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: row.equation.to_string(),
            });
            concentrated_fired = true;
        }
    }

    if peroxide_fired {
        events.push(peroxide_note(vessel));
    }
    if concentrated_fired {
        events.push(concentrated_note(vessel));
    }
    peroxide_fired || concentrated_fired
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::Kelvin;
    use crate::VesselId;

    fn beaker() -> Vessel {
        Vessel::new(VesselId(0), "beaker")
    }

    fn noted(events: &[Event], key: &str) -> bool {
        events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { reason: Some(r), .. } if r.key == key))
    }

    fn element_total(v: &Vessel, el: &str) -> f64 {
        v.contents
            .iter()
            .map(|p| {
                let formula = crate::species::lookup_key(&p.species.0)
                    .map(|d| d.formula)
                    .unwrap_or_else(|| panic!("{}", p.species.0));
                p.moles.0
                    * crate::stoich::parse_formula(formula)
                        .unwrap()
                        .counts
                        .get(el)
                        .copied()
                        .unwrap_or(0.0)
            })
            .sum()
    }

    #[test]
    fn concentrated_acid_chars_sugar_until_its_own_water_dilutes_it() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("sucrose"), Moles(0.005), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.0375), Phase::Liquid);
        let before = ["C", "H", "O", "S"].map(|e| element_total(&v, e));
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        // 0.0375 mol acid takes at most 0.0375 mol water before water is half
        // the liquid: 0.0375 / 11 of sucrose chars, the rest is left.
        let charred = 0.0375 / 11.0;
        assert!((phase_moles(&v, "C", &[Phase::Solid]) - 12.0 * charred).abs() < 1e-12);
        assert!((phase_moles(&v, "sucrose", &[Phase::Solid]) - (0.005 - charred)).abs() < 1e-12);
        assert!(noted(
            &events,
            "not-modeled.concentrated-sulfuric-qualitative"
        ));
        let after = ["C", "H", "O", "S"].map(|e| element_total(&v, e));
        for (b, a) in before.iter().zip(after) {
            assert!((b - a).abs() < 1e-12, "{before:?} {after:?}");
        }
        // Now diluted to the boundary: nothing more fires.
        assert!(!fires(&v));
    }

    #[test]
    fn dilute_acid_neither_chars_nor_releases_hydrogen_chloride() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        v.deposit(SpeciesId::new("sucrose"), Moles(0.005), Phase::Solid);
        v.deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.02), Phase::Liquid);
        v.temperature = Kelvin(380.0);
        v.deposit(SpeciesId::new("Cu"), Moles(0.01), Phase::Solid);
        assert!(!fires(&v));
    }

    #[test]
    fn salt_and_concentrated_acid_give_hydrogen_chloride() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("NaCl"), Moles(0.05), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.03), Phase::Liquid);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        let hcl: f64 = events
            .iter()
            .map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == "HCl" => moles.0,
                _ => 0.0,
            })
            .sum();
        assert!((hcl - 0.03).abs() < 1e-12);
        assert!((phase_moles(&v, "NaHSO4", &[Phase::Solid]) - 0.03).abs() < 1e-12);
        assert!((phase_moles(&v, "NaCl", &[Phase::Solid]) - 0.02).abs() < 1e-12);
    }

    #[test]
    fn copper_needs_hot_acid_and_gives_sulfur_dioxide() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("Cu"), Moles(0.005), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        assert!(!fires(&v), "cold concentrated acid leaves copper");
        v.temperature = Kelvin(420.0);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!(phase_moles(&v, "Cu", &[Phase::Solid]) < 1e-12);
        assert!((phase_moles(&v, "CuSO4", &[Phase::Solid]) - 0.005).abs() < 1e-12);
        let so2: f64 = events
            .iter()
            .map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == "SO2" => moles.0,
                _ => 0.0,
            })
            .sum();
        assert!((so2 - 0.005).abs() < 1e-12);
        assert!((acid(&v) - 0.04).abs() < 1e-12);
    }

    #[test]
    fn hot_acid_oxidises_carbon_and_sulfur_but_not_graphite() {
        for (key, so2) in [("C", 2.0), ("activated_charcoal", 2.0), ("S", 3.0)] {
            let mut v = beaker();
            v.deposit(SpeciesId::new(key), Moles(0.002), Phase::Solid);
            v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
            v.temperature = Kelvin(420.0);
            let mut events = Vec::new();
            assert!(apply(&mut v, &mut events), "{key}");
            let made: f64 = events
                .iter()
                .map(|e| match e {
                    Event::GasEvolved { species, moles, .. } if species.0 == "SO2" => moles.0,
                    _ => 0.0,
                })
                .sum();
            assert!((made - so2 * 0.002).abs() < 1e-12, "{key}: {made}");
        }
        let mut v = beaker();
        v.deposit(SpeciesId::new("graphite"), Moles(0.002), Phase::Solid);
        v.deposit(SpeciesId::new(ACID), Moles(0.05), Phase::Liquid);
        v.temperature = Kelvin(420.0);
        assert!(!fires(&v));
    }

    #[test]
    fn peroxide_oxidises_dissolved_hydrogen_sulfide_to_sulfur() {
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(0.55), Phase::Liquid);
        v.deposit(SpeciesId::new("H2S(aq)"), Moles(0.001), Phase::Aqueous);
        v.deposit(SpeciesId::new("H2O2"), Moles(0.005), Phase::Liquid);
        let mut events = Vec::new();
        assert!(apply(&mut v, &mut events));
        assert!((phase_moles(&v, "S", &[Phase::Solid]) - 0.001).abs() < 1e-12);
        assert!((phase_moles(&v, "H2O2", &[Phase::Liquid]) - 0.004).abs() < 1e-12);
        assert!(noted(&events, "not-modeled.sulfide-peroxide-qualitative"));
        // No peroxide, no sulfur.
        let mut v = beaker();
        v.deposit(SpeciesId::new("water"), Moles(0.55), Phase::Liquid);
        v.deposit(SpeciesId::new("H2S(aq)"), Moles(0.001), Phase::Aqueous);
        assert!(!fires(&v));
    }
}
