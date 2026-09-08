//! Conservative change from full speciation to the analytical inventory.
//!
//! PHREEQC's solvent belongs to its full species distribution. Analytical
//! component totals require their own H/O balance. Complete that basis with
//! H2O and acid/base equivalents, not a correction to the scale. These
//! equivalents are not free-ion measurements: SolutionInfo remains authoritative
//! for activities, pH, and free_proton/free_hydroxide.
use kerotakis_core::{
    ledger::ConservedLedger, species, Event, Moles, Phase, Portion, SolveError, SpeciesId, Vessel,
};

pub(crate) fn complete_basis(
    before: &Vessel,
    after: &mut Vessel,
    events: &[Event],
    reservoirs: &[&str],
) -> Result<(), SolveError> {
    let mut target = ConservedLedger::from_vessel(before).elements;
    for event in events {
        let (id, amount) = match event {
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
            for (element, count) in formula.counts {
                *target.entry(element).or_default() += amount * count;
            }
        }
    }
    after.contents.retain(|p| {
        !(p.phase == Phase::Aqueous && matches!(p.species.0.as_str(), "H+" | "OH-")
            || p.phase == Phase::Liquid && p.species.0 == "water")
    });
    let booked = ConservedLedger::from_vessel(after).elements;
    let remaining =
        |el: &str| target.get(el).copied().unwrap_or(0.0) - booked.get(el).copied().unwrap_or(0.0);
    let oxygen = remaining("O");
    let hydrogen = remaining("H");
    // w+b=O; 2w+a+b=H. Choose the nonnegative acid/base representation.
    // Autoionization is in speciation and is not duplicated in this basis.
    let excess = hydrogen - 2.0 * oxygen;
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
    for (key, amount, phase) in [
        ("water", water.max(0.0), Phase::Liquid),
        ("H+", acid, Phase::Aqueous),
        ("OH-", base, Phase::Aqueous),
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
