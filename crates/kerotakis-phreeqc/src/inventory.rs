//! Conservative change from full speciation to the analytical inventory.
//!
//! PHREEQC's solvent belongs to its full species distribution. Analytical
//! component totals require their own H/O balance. Complete that basis with
//! H2O and acid/base equivalents, not a correction to the scale. These
//! equivalents are not free-ion measurements: SolutionInfo remains authoritative
//! for activities, pH, and free_proton/free_hydroxide.
//!
//! The base half of those equivalents is published as
//! [`species::BASE_EQUIVALENTS`] and not as `OH-`. It was `OH-` until
//! 2026-09-16, and under that name it was read — reasonably — as an amount
//! of hydroxide, which it is not: it is `2·O − H` left over after every
//! other portion is booked, the same number as the solution's residual
//! cation charge, and it equals the measured hydroxide only where the
//! charge really is carried by free base. An equimolar acetate buffer
//! published 5.17e-4 mol of it at a pH that can hold 4.6e-10, and moved it
//! 1.30x under a perturbation that must move hydroxide 2.33x. The
//! measurement was never missing — `Vessel::free_hydroxide` had it, and
//! moved by exactly 2.33x — so the fix is the name.
use std::collections::BTreeMap;

use kerotakis_core::{
    ledger::ConservedLedger, species, Event, Moles, Phase, Portion, SolveError, SpeciesId, Vessel,
};

/// A portion that is a coordinate of the acid/base/water basis rather than
/// an amount of a substance. `complete_basis` strips these and rewrites
/// them, on both sides of the balance.
fn is_basis_portion(portion: &Portion) -> bool {
    portion.phase == Phase::Aqueous && species::is_acid_base_basis(&portion.species.0)
        || portion.phase == Phase::Liquid && portion.species.0 == "water"
}

/// `H − 2·O` for one species: its coordinate along the acid/base axis of
/// the `{H2O, H+, base}` basis, and zero for water by construction.
fn acid_base_coordinate(id: &SpeciesId) -> f64 {
    species::lookup(id)
        .and_then(|data| kerotakis_core::stoich::parse_formula(data.formula).ok())
        .map(|formula| {
            let h = formula.counts.get("H").copied().unwrap_or(0.0);
            let o = formula.counts.get("O").copied().unwrap_or(0.0);
            h - 2.0 * o
        })
        .unwrap_or(0.0)
}

fn amount(totals: &BTreeMap<String, f64>, element: &str) -> f64 {
    totals.get(element).copied().unwrap_or(0.0)
}

pub(crate) fn complete_basis(
    before: &Vessel,
    after: &mut Vessel,
    events: &[Event],
    reservoirs: &[&str],
) -> Result<(), SolveError> {
    let before_ledger = ConservedLedger::from_vessel(before);
    let mut target = before_ledger.elements;
    // What the gas that left or arrived took with it, both as element
    // totals and as its own acid/base coordinate.
    let mut gas_excess = 0.0;
    for event in events {
        let (id, moles) = match event {
            Event::GasEvolved { species, moles, .. } => (species, -moles.0),
            Event::GasAbsorbed { species, moles, .. }
                if reservoirs.contains(&species.0.as_str()) =>
            {
                (species, moles.0)
            }
            _ => continue,
        };
        if let Some(formula) =
            species::lookup(id).and_then(|s| kerotakis_core::stoich::parse_formula(s.formula).ok())
        {
            let h = formula.counts.get("H").copied().unwrap_or(0.0);
            let o = formula.counts.get("O").copied().unwrap_or(0.0);
            gas_excess += moles * (h - 2.0 * o);
            for (element, count) in formula.counts {
                *target.entry(element).or_default() += moles * count;
            }
        }
    }
    // The before vessel, split the same way the after vessel is about to
    // be: the basis portions it already holds, kept as themselves, and
    // everything else.
    let mut core_before = before.clone();
    let mut basis_excess = 0.0;
    core_before.contents.retain(|portion| {
        if is_basis_portion(portion) {
            basis_excess += portion.moles.0 * acid_base_coordinate(&portion.species);
            return false;
        }
        true
    });
    let core = ConservedLedger::from_vessel(&core_before).elements;
    after.contents.retain(|portion| !is_basis_portion(portion));
    let after_ledger = ConservedLedger::from_vessel(after);
    let booked = after_ledger.elements;
    let remaining = |el: &str| amount(&target, el) - amount(&booked, el);
    let oxygen = remaining("O");
    let hydrogen = remaining("H");
    // w+b=O; 2w+a+b=H. Choose the nonnegative acid/base representation.
    // Autoionization is in speciation and is not duplicated in this basis.
    //
    // `a − b` is `H − 2·O`, and it is NOT computed that way. Both residuals
    // are about 11 mol for 100 g of water and they differ by about 3e-4, so
    // subtracting them is a difference of two large nearly-equal numbers:
    // every part in 1e9 the engine's ten-significant-figure molalities leave
    // in an 11 mol sum arrives in the answer magnified by 3e4. That is what
    // made `aq-023` publish two different `base_equivalents` for one final
    // state (PLAN.md, 2026-09-18).
    //
    // So the same quantity is assembled from small ones instead. `H − 2·O`
    // is linear over portions, and water's own coordinate is exactly zero —
    // every mole of solvent contributes nothing to it. Dropping the solvent
    // from both sides therefore changes no value and removes the whole of
    // the cancellation: what is left is the before vessel's own acid/base
    // portions, carried across exactly, plus the coordinate of the species
    // on each side, all of them small.
    let excess = basis_excess + (amount(&core, "H") - 2.0 * amount(&core, "O")) + gas_excess
        - (amount(&booked, "H") - 2.0 * amount(&booked, "O"));
    let acid = excess.max(0.0);
    let base = (-excess).max(0.0);
    let water = oxygen - base;
    if !water.is_finite() || water < -1e-9 || !acid.is_finite() || !base.is_finite() {
        return Err(SolveError::NotConverged {
            solver: "phreeqc-inventory".into(),
            detail: format!(
                "no nonnegative aqueous basis conserves H/O: residual H={hydrogen}, O={oxygen}"
            ),
        });
    }
    // Oxygen closes exactly — `w + b` is `oxygen` by construction, as it
    // always was. Hydrogen no longer does, because `a − b` no longer comes
    // from it: it closes to whatever the two routes agree to. They are the
    // same expression, so that is rounding and nothing else, and the
    // tolerance is set far above rounding on purpose. A residue this large
    // would mean the two sides disagree about what is in the vessel, which
    // is the failure the original difference could never report because it
    // defined hydrogen to be whatever made it balance.
    let reconstructed = 2.0 * water.max(0.0) + acid + base;
    let residue = reconstructed - hydrogen;
    if !residue.is_finite() || residue.abs() > 1e-9 + 1e-6 * hydrogen.abs() {
        return Err(SolveError::NotConverged {
            solver: "phreeqc-inventory".into(),
            detail: format!(
                "the aqueous basis does not close on hydrogen: residual H={hydrogen}, \
                 O={oxygen}, rebuilt H={reconstructed}"
            ),
        });
    }
    for (key, amount, phase) in [
        ("water", water.max(0.0), Phase::Liquid),
        ("H+", acid, Phase::Aqueous),
        (species::BASE_EQUIVALENTS, base, Phase::Aqueous),
    ] {
        if amount > 1e-12 {
            after.contents.push(Portion {
                species: SpeciesId::new(key),
                moles: Moles(amount),
                phase,
            });
        }
    }
    Ok(())
}
