//! EXP-31: the four classic gas tests — pop, glowing splint, limewater,
//! damp litmus — as observation verbs over a vessel's headspace.

use crate::ops::Event;
use crate::species::{Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::{Headspace, Vessel, VesselId};

/// The four classical bench gas tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GasTest {
    Pop,
    GlowingSplint,
    Limewater,
    DampLitmus,
}

impl std::fmt::Display for GasTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GasTest::Pop => write!(f, "pop test"),
            GasTest::GlowingSplint => write!(f, "glowing splint"),
            GasTest::Limewater => write!(f, "limewater"),
            GasTest::DampLitmus => write!(f, "damp red litmus"),
        }
    }
}

// LEL of hydrogen in air is 4% by volume (NFPA 2/CRC Handbook).
pub const H2_IGNITION_FLOOR: f64 = 0.04;

// A glowing splint relights in oxygen-enriched atmospheres. The
// threshold is well above normal air (~21%); ~25% O₂ by mole fraction
// is the accepted enrichment level at which a glowing splint relights
// (CLEAPSS / Nuffield practical chemistry).
pub const O2_RELIGHT_FLOOR: f64 = 0.25;

// CO₂ above trace turns limewater milky. Any non-negligible mole
// fraction suffices; 0.1% is well above instrument noise and below
// the atmospheric 0.04%.
pub const CO2_TRACE_FLOOR: f64 = 0.001;

// NH₃ above trace turns damp red litmus blue. Same floor as CO₂.
pub const NH3_TRACE_FLOOR: f64 = 0.001;

// Curated Ca(OH)₂ consumed per mole of CO₂ in the limewater test.
// The real reaction is CO₂ + Ca(OH)₂ → CaCO₃↓ + H₂O, 1:1
// stoichiometry. We consume a fixed small aliquot representing the
// limewater's capacity rather than modelling the solution.
const LIMEWATER_CA_OH_2_MOLES: f64 = 0.001;

fn gas_mole_fraction(vessel: &Vessel, species_key: &str) -> f64 {
    let total = vessel.gas_moles().0;
    if total <= 0.0 {
        return 0.0;
    }
    let id = SpeciesId::new(species_key);
    let species_moles: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Gas && p.species == id)
        .map(|p| p.moles.0)
        .sum();
    species_moles / total
}

fn gas_moles_of(vessel: &Vessel, species_key: &str) -> f64 {
    let id = SpeciesId::new(species_key);
    vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Gas && p.species == id)
        .map(|p| p.moles.0)
        .sum()
}

pub fn dispatch(vessel: &mut Vessel, vessel_id: VesselId, test: GasTest) -> Vec<Event> {
    let mut events = Vec::new();

    if matches!(vessel.headspace, Headspace::Open) {
        events.push(Event::NotYetModeled {
            vessel: vessel_id,
            what: format!(
                "nothing to test — gas left as it formed; \
                 collect over a sealed vessel first, then run the {test}"
            ),
        });
        return events;
    }

    if !vessel.owns_headspace_gas() {
        events.push(Event::NotYetModeled {
            vessel: vessel_id,
            what: format!(
                "the vessel's boundary does not retain gas — \
                 seal it first, then run the {test}"
            ),
        });
        return events;
    }

    match test {
        GasTest::Pop => dispatch_pop(vessel, vessel_id, &mut events),
        GasTest::GlowingSplint => dispatch_splint(vessel, vessel_id, &mut events),
        GasTest::Limewater => dispatch_limewater(vessel, vessel_id, &mut events),
        GasTest::DampLitmus => dispatch_litmus(vessel, vessel_id, &mut events),
    }

    events
}

fn dispatch_pop(vessel: &mut Vessel, id: VesselId, events: &mut Vec<Event>) {
    let frac = gas_mole_fraction(vessel, "H2");
    if frac < H2_IGNITION_FLOOR {
        events.push(Event::GasTested {
            vessel: id,
            test: GasTest::Pop,
            positive: false,
            notes: format!(
                "no pop — H₂ mole fraction {:.1}% is below the {:.0}% ignition limit",
                frac * 100.0,
                H2_IGNITION_FLOOR * 100.0
            ),
        });
        return;
    }

    // 2 H₂ + O₂ → 2 H₂O. Extent limited by whichever runs out.
    let h2 = gas_moles_of(vessel, "H2");
    let o2 = gas_moles_of(vessel, "O2");
    // Each mole of O₂ reacts with 2 moles of H₂.
    let extent_by_h2 = h2 / 2.0;
    let extent_by_o2 = o2;
    let extent = extent_by_h2.min(extent_by_o2); // moles of O₂ consumed

    let h2_consumed = extent * 2.0;
    let o2_consumed = extent;
    let h2o_produced = extent * 2.0;

    vessel.withdraw(&SpeciesId::new("H2"), Moles(h2_consumed));
    vessel.withdraw(&SpeciesId::new("O2"), Moles(o2_consumed));
    vessel.deposit(SpeciesId::new("water"), Moles(h2o_produced), Phase::Liquid);
    vessel.refresh_pressure();

    events.push(Event::GasTested {
        vessel: id,
        test: GasTest::Pop,
        positive: true,
        notes: format!(
            "squeaky pop — {:.4} mol H₂ ignited with {:.4} mol O₂, \
             producing {:.4} mol H₂O; 2 H₂ + O₂ → 2 H₂O",
            h2_consumed, o2_consumed, h2o_produced
        ),
    });
}

fn dispatch_splint(vessel: &Vessel, id: VesselId, events: &mut Vec<Event>) {
    let frac = gas_mole_fraction(vessel, "O2");
    if frac >= O2_RELIGHT_FLOOR {
        events.push(Event::GasTested {
            vessel: id,
            test: GasTest::GlowingSplint,
            positive: true,
            notes: format!(
                "the glowing splint relights — O₂ mole fraction {:.1}% \
                 (above the {:.0}% enrichment threshold)",
                frac * 100.0,
                O2_RELIGHT_FLOOR * 100.0
            ),
        });
    } else {
        events.push(Event::GasTested {
            vessel: id,
            test: GasTest::GlowingSplint,
            positive: false,
            notes: format!(
                "the splint does not relight — O₂ mole fraction {:.1}% \
                 is below the {:.0}% enrichment threshold",
                frac * 100.0,
                O2_RELIGHT_FLOOR * 100.0
            ),
        });
    }
}

fn dispatch_limewater(vessel: &mut Vessel, id: VesselId, events: &mut Vec<Event>) {
    let frac = gas_mole_fraction(vessel, "CO2");
    if frac < CO2_TRACE_FLOOR {
        events.push(Event::GasTested {
            vessel: id,
            test: GasTest::Limewater,
            positive: false,
            notes: format!(
                "limewater stays clear — CO₂ mole fraction {:.2}% \
                 is below the {:.1}% detection floor",
                frac * 100.0,
                CO2_TRACE_FLOOR * 100.0
            ),
        });
        return;
    }

    // CO₂ + Ca(OH)₂ → CaCO₃↓ + H₂O (curated, 1:1 stoichiometry).
    // Consume a fixed small aliquot of CO₂ that the limewater reacts with.
    let co2_available = gas_moles_of(vessel, "CO2");
    let co2_consumed = co2_available.min(LIMEWATER_CA_OH_2_MOLES);

    vessel.withdraw(&SpeciesId::new("CO2"), Moles(co2_consumed));
    // The Ca(OH)₂ and CaCO₃ are in the limewater (a separate solution
    // held up to the tube mouth); we don't model the limewater vessel
    // explicitly — we consume a curated amount of CO₂ and state the
    // reaction honestly.
    vessel.refresh_pressure();

    events.push(Event::GasTested {
        vessel: id,
        test: GasTest::Limewater,
        positive: true,
        notes: format!(
            "limewater turns milky — CO₂ detected (mole fraction {:.1}%); \
             {:.4} mol CO₂ consumed; CO₂ + Ca(OH)₂ → CaCO₃↓ + H₂O \
             (curated stoichiometry, limewater not modelled as a vessel)",
            frac * 100.0,
            co2_consumed
        ),
    });
}

fn dispatch_litmus(vessel: &Vessel, id: VesselId, events: &mut Vec<Event>) {
    let frac = gas_mole_fraction(vessel, "NH3");
    if frac >= NH3_TRACE_FLOOR {
        events.push(Event::GasTested {
            vessel: id,
            test: GasTest::DampLitmus,
            positive: true,
            notes: format!(
                "damp red litmus turns blue — NH₃ detected \
                 (mole fraction {:.1}%)",
                frac * 100.0
            ),
        });
    } else {
        events.push(Event::GasTested {
            vessel: id,
            test: GasTest::DampLitmus,
            positive: false,
            notes: format!(
                "litmus stays red — NH₃ mole fraction {:.2}% \
                 is below the {:.1}% detection floor",
                frac * 100.0,
                NH3_TRACE_FLOOR * 100.0
            ),
        });
    }
}
