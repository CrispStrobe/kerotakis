//! A large sweep: many states, every invariant we can check, zero excuses.
//!
//! The tests each guard one thing. This drives a *matrix* of vessel states
//! through the whole stack and asserts the properties that must hold
//! everywhere, because the failures that matter have all been the ones
//! nobody thought to write a test for — a solver that created matter, a
//! filter that read as a fact, a rate law that destroyed sodium.
//!
//! Every invariant here is something the engine claims about itself
//! somewhere in its own documentation. Checking a claim is cheaper than
//! believing it.

use kerotakis_core::*;

/// The solvent, named once.
fn solvent() -> SpeciesId {
    SpeciesId::new("water")
}

/// One thing that must be true of every solved state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Invariant {
    /// No amount is negative, NaN or infinite.
    AmountsAreReal,
    /// Elements in equal elements out, across the whole step.
    ElementsConserved,
    /// pH lands somewhere a solution can be.
    PhIsPhysical,
    /// A reported temperature is one the vessel actually reached.
    TemperatureIsReal,
    /// Nothing claims to be liquid below its freezing point.
    StatesAgreeWithTemperature,
    /// Every species present is one the registry can name.
    SpeciesAreNameable,
    /// A vessel that reports a solution reports a finite ionic strength.
    SolutionIsWellFormed,
    /// The redox split accounts for all of the element it describes.
    RedoxSplitIsComplete,
}

pub struct Finding {
    pub invariant: Invariant,
    pub case: String,
    pub detail: String,
}

fn elements_of(v: &Vessel) -> std::collections::BTreeMap<String, f64> {
    let mut totals: std::collections::BTreeMap<String, f64> = Default::default();
    for p in &v.contents {
        let Some(data) = species::lookup(&p.species) else {
            continue;
        };
        let Ok(f) = kerotakis_core::stoich::parse_formula(data.formula) else {
            continue;
        };
        for (el, n) in f.counts {
            *totals.entry(el).or_insert(0.0) += n * p.moles.0;
        }
    }
    totals
}

/// Check one bench state against every invariant.
pub fn check(case: &str, before: &Vessel, after: &Vessel, events: &[Event]) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut fail = |invariant: Invariant, detail: String| {
        out.push(Finding {
            invariant,
            case: case.to_string(),
            detail,
        })
    };

    for p in &after.contents {
        if !p.moles.0.is_finite() || p.moles.0 < 0.0 {
            fail(
                Invariant::AmountsAreReal,
                format!("{} is {} mol", p.species.0, p.moles.0),
            );
        }
        if species::lookup(&p.species).is_none() {
            fail(
                Invariant::SpeciesAreNameable,
                format!("{} is not in the registry", p.species.0),
            );
        }
    }

    // Conservation, allowing for what an open vessel legitimately vents and
    // for matter deliberately added by this step.
    //
    // Hydrogen and oxygen are excluded, and that exclusion is an admission
    // rather than a convenience: the aqueous input ends in `pH 7 charge`,
    // so PHREEQC balances the solution's charge by adding or removing
    // protons, and those protons are not in the vessel's inventory. Add
    // hydrochloric acid and its hydrogen leaves the ledger. The engine says
    // as much in the codex — charge is delegated, not checked — and a sweep
    // that pretended otherwise would report 2 000 violations of a rule this
    // design does not claim to follow.
    let (a, b) = (elements_of(before), elements_of(after));
    let mut vented: std::collections::BTreeMap<String, f64> = Default::default();
    for e in events {
        let (species, moles, sign) = match e {
            Event::GasEvolved { species, moles, .. } => (species, moles.0, -1.0),
            Event::Added { species, moles, .. } => (species, moles.0, 1.0),
            // Evaporation and filtration remove matter on purpose; the
            // event carries what left.
            Event::Evaporated { moles, .. } => (&solvent(), moles.0, -1.0),
            _ => continue,
        };
        let Some(d) = species::lookup(species) else {
            continue;
        };
        let Ok(f) = kerotakis_core::stoich::parse_formula(d.formula) else {
            continue;
        };
        for (el, n) in f.counts {
            *vented.entry(el).or_insert(0.0) += sign * n * moles;
        }
    }
    let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        if k == "H" || k == "O" {
            continue;
        }
        let start = a.get(k).copied().unwrap_or(0.0);
        let end = b.get(k).copied().unwrap_or(0.0);
        let expected = start + vented.get(k).copied().unwrap_or(0.0);
        let scale = start.max(end).max(1e-9);
        // 1e-5, not 1e-6, and the reason is written down rather than
        // widened silently: every dissolved amount makes a round trip
        // through `molality × mass_H2O`, and a long path accumulates float
        // residue at about this level. The structural half of that drift is
        // fixed (the solvent is rebuilt on the equilibrated water mass);
        // what is left is arithmetic. The goal is to return this to 1e-6
        // once the readback carries full precision end to end.
        // An absolute floor as well as a relative one, because a relative
        // test alone is unreasonable about small quantities: 3 × 10⁻⁴ mol
        // of carbonate carries the same float residue as 3 × 10⁻¹ and fails
        // a ratio test purely for being small. A nanomole is a thousandth
        // of the amount this bench will even report seeing
        // (OBSERVABLE_MOLES = 1e-6), so a discrepancy below it is
        // arithmetic, not chemistry.
        let drift = (end - expected).abs();
        if drift > 1e-9 && drift / scale > 1e-5 {
            fail(
                Invariant::ElementsConserved,
                format!(
                    "{k}: {start:.6} → {end:.6}, expected {expected:.6} (drift {:.3e}, rel {:.3e})",
                    drift,
                    drift / scale
                ),
            );
        }
    }

    if !after.temperature.0.is_finite() || after.temperature.0 <= 0.0 {
        fail(
            Invariant::TemperatureIsReal,
            format!("{} K", after.temperature.0),
        );
    }
    // Only the *last* announcement has to match: a single step can move the
    // temperature more than once — matter enters at one temperature, then a
    // reaction releases heat — and each of those is a true report of its
    // own moment. It is the final reading that must be the vessel's.
    if let Some(Event::TemperatureChanged { to, .. }) = events
        .iter()
        .rfind(|e| matches!(e, Event::TemperatureChanged { .. }))
    {
        if (to.0 - after.temperature.0).abs() > 0.05 {
            fail(
                Invariant::TemperatureIsReal,
                format!("announced {} K, vessel at {} K", to.0, after.temperature.0),
            );
        }
    }

    // Liquid water below its freezing point is the bug the states model
    // exists for, and it must not come back.
    // From the inventory, not from `solution`: freezing *withdraws* the
    // solution, so reading the depression from there would score every
    // frozen brine against pure water's 273.15 K and cry wolf.
    let kgw_now: f64 = after
        .contents
        .iter()
        .filter(|p| p.species == solvent() && p.phase == Phase::Liquid)
        .map(|p| p.moles.0 * 0.018_015)
        .sum();
    let solutes: f64 = if kgw_now > 0.0 {
        after
            .contents
            .iter()
            .filter(|p| p.phase == Phase::Aqueous)
            .map(|p| p.moles.0)
            .sum::<f64>()
            / kgw_now
    } else {
        0.0
    };
    let t = kerotakis_core::states::transitions(solutes);
    let liquid_water = after
        .contents
        .iter()
        .any(|p| p.species == SpeciesId::new("water") && p.phase == Phase::Liquid);
    // Prefer the engine's own claim to a rival estimate. When it froze
    // something it said where, and that is the number to check; the
    // inventory-based estimate below counts ions where the engine counts
    // *species*, so ion pairing puts the two a few tenths of a kelvin
    // apart. That gap is real information, not noise to be papered over
    // with a margin — it is only used when the engine made no claim at all.
    //
    // The claim to check is the engine's *last* one: partial freezing
    // settles iteratively — freeze, concentrate the residual brine,
    // re-state the transition at the lower liquidus — so earlier events
    // record the onset, not where the vessel ended up.
    let claimed_freezing = events.iter().rev().find_map(|e| match e {
        Event::StateChanged { at, .. } => Some(at.0),
        _ => None,
    });
    let freezing_k = claimed_freezing.unwrap_or(t.freezing_k);
    let allowance = if claimed_freezing.is_some() {
        1e-6
    } else {
        1.5
    };
    if liquid_water && after.temperature.0 < freezing_k - allowance {
        fail(
            Invariant::StatesAgreeWithTemperature,
            format!(
                "liquid water at {:.2} K, freezes at {:.2} K",
                after.temperature.0, freezing_k
            ),
        );
    }

    if let Some(s) = &after.solution {
        if !s.ph.is_finite() || !(-1.0..15.5).contains(&s.ph) {
            fail(Invariant::PhIsPhysical, format!("pH {}", s.ph));
        }
        if !s.ionic_strength.is_finite() || s.ionic_strength < 0.0 {
            fail(
                Invariant::SolutionIsWellFormed,
                format!("I = {}", s.ionic_strength),
            );
        }
        // Every oxidation state of an element must add up to the element.
        let mut by_element: std::collections::BTreeMap<&str, f64> = Default::default();
        for r in &s.redox {
            *by_element.entry(r.element.as_str()).or_insert(0.0) += r.molality;
        }
        for (el, split_total) in by_element {
            let inventory: f64 = after
                .contents
                .iter()
                // The split describes what is *dissolved*. Comparing it
                // against a vessel that also holds a precipitate reports
                // every successful precipitation as a violation: copper
                // sulfate plus lye is 0.01 mol of copper, almost all of it
                // solid Cu(OH)2, and the dissolved split is rightly a
                // millionth of that.
                .filter(|p| p.phase == Phase::Aqueous)
                .filter_map(|p| {
                    let d = species::lookup(&p.species)?;
                    let f = kerotakis_core::stoich::parse_formula(d.formula).ok()?;
                    Some(f.counts.get(el).copied().unwrap_or(0.0) * p.moles.0)
                })
                .sum();
            let kgw = after
                .contents
                .iter()
                .filter(|p| p.species == SpeciesId::new("water") && p.phase == Phase::Liquid)
                .map(|p| p.moles.0 * 0.018015)
                .sum::<f64>();
            if kgw <= 0.0 || inventory <= 0.0 {
                continue;
            }
            let split_moles = split_total * kgw;
            if (split_moles - inventory).abs() / inventory.max(1e-9) > 0.02 {
                fail(
                    Invariant::RedoxSplitIsComplete,
                    format!("{el}: split says {split_moles:.6} mol, vessel holds {inventory:.6}"),
                );
            }
        }
    }

    out
}
