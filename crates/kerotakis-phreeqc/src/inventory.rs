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
use std::collections::{BTreeMap, BTreeSet};

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::{
    ledger::ConservedLedger, species, Event, Moles, Phase, Portion, SolveError, SpeciesId, Vessel,
    VesselId,
};

use crate::derived;

/// Carbon already posed in native analytical totals from a clock parcel.
/// This is a virtual material boundary for local basis/certification only:
/// it is neither a native gas phase nor a public gas event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClockCo2Boundary {
    pub(crate) vessel: VesselId,
    pub(crate) applied_mol: f64,
}

impl ClockCo2Boundary {
    pub(crate) fn none(vessel: VesselId) -> Self {
        Self {
            vessel,
            applied_mol: 0.0,
        }
    }
}

/// Opaque recipe mass has no elemental formula in `ConservedLedger`. It can
/// accompany a certified aqueous clock boundary only as an unchanged owner.
/// Check its actual state, including the numerical progress fields, rather
/// than treating the represented-atom ledger as a whole-mixture proof.
pub(crate) fn valid_opaque_clock_stock(vessel: &Vessel) -> bool {
    vessel.unresolved_materials.iter().all(|portion| {
        portion.amount.is_finite()
            && portion.amount >= 0.0
            && portion.protein_denatured_fraction.is_finite()
            && (0.0..=1.0).contains(&portion.protein_denatured_fraction)
            && portion.enzyme_hydrolysis.as_ref().is_none_or(|state| {
                state.converted_fraction.is_finite()
                    && (0.0..=1.0).contains(&state.converted_fraction)
            })
    })
}

/// Only these reviewed, non-native formula units and the typed catalyst may
/// accompany a clock-owned aqueous answer. The carbohydrates have exact
/// registry compositions; pepsin is an immutable catalyst dose whose `C`
/// formula is explicitly a placeholder, not a molecule-count claim.
pub(crate) fn clock_spectator(portion: &Portion) -> bool {
    let key = portion.species.0.as_str();
    let phase_ok = match key {
        "glucose" | "fructose" => matches!(portion.phase, Phase::Solid | Phase::Aqueous),
        "cellulose" => portion.phase == Phase::Solid,
        "pepsin" => portion.phase == Phase::Aqueous,
        _ => false,
    };
    if !phase_ok || !portion.moles.0.is_finite() || portion.moles.0 < 0.0 {
        return false;
    }
    if key == "pepsin" {
        return kerotakis_core::enzyme::family_of(key).is_some()
            && species::lookup(&portion.species).is_some_and(|row| row.formula == "C");
    }
    let expected = if key == "cellulose" {
        "C6H10O5"
    } else {
        "C6H12O6"
    };
    species::lookup(&portion.species).is_some_and(|row| {
        row.formula == expected
            && kerotakis_core::stoich::parse_formula(row.formula)
                .is_ok_and(|formula| formula.charge == 0.0)
    })
}

/// A named optical reporter is an immutable spectator of a narrowly owned
/// aqueous calculation beside an open gas reservoir. Its colour is a readout, not
/// a licence to model the pigment's acid/base chemistry or the juice's
/// unrepresented buffering. The registry formula must be real and neutral;
/// a placeholder like the pepsin catalogue's `C` cannot earn this path.
pub(crate) fn passive_ladder_reporter(portion: &Portion) -> bool {
    if portion.phase != Phase::Aqueous
        || !portion.moles.0.is_finite()
        || portion.moles.0 < 0.0
        || kerotakis_core::indicator::lookup_ladder(&portion.species.0).is_none()
    {
        return false;
    }
    species::lookup(&portion.species).is_some_and(|row| {
        row.formula != "C"
            && kerotakis_core::stoich::parse_formula(row.formula).is_ok_and(|formula| {
                formula.charge == 0.0
                    && !formula.counts.is_empty()
                    && formula
                        .counts
                        .values()
                        .all(|count| count.is_finite() && *count > 0.0)
            })
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImmutablePolicy {
    Clock,
    ReservoirOptical,
}

#[derive(Clone, Copy, Debug)]
enum ImmutableFailure {
    InvalidOpaque,
    ChangedOpaque,
    InvalidSpectator,
    ChangedSpectator,
}

#[derive(Clone, Debug)]
enum InventoryReason {
    Immutable {
        policy: ImmutablePolicy,
        failure: ImmutableFailure,
    },
    ClockBoundaryInvalid,
    ClockFormulaUnknown,
    AqueousBasisInvalid,
    AqueousBasisDoesNotClose,
    ReservoirBoundaryUnsupported,
    VesselIdentityChanged,
    FormulaUnresolved,
    AmountNonfinite,
    AmountNegative,
    GasCrossingWrongVessel,
    GasCrossingUnowned,
    GasAmountNonfinite,
    GasAmountNegative,
    ReservoirFormulaUnknown,
    ElementLedgerNonfinite,
    ElementLedgerMismatch {
        element: String,
        before: f64,
        after: f64,
    },
    ChargeLedgerNonfinite,
    ChargeLedgerMismatch {
        before: f64,
        after: f64,
    },
}

impl InventoryReason {
    fn phrase(self) -> Phrase {
        let bare = |key, en| Phrase::bare(key, en);
        match self {
            Self::Immutable { policy, failure } => {
                let (key, en) = immutable_phrase(policy, failure);
                bare(key, en)
            }
            Self::ClockBoundaryInvalid => bare(
                "solver-reason.inventory.clock-boundary-invalid",
                "The clock carbon dioxide boundary has an invalid owner or amount.",
            ),
            Self::ClockFormulaUnknown => bare(
                "solver-reason.inventory.clock-formula-unknown",
                "The clock carbon dioxide formula is not registered.",
            ),
            Self::AqueousBasisInvalid => bare(
                "solver-reason.inventory.aqueous-basis-invalid",
                "The nonnegative aqueous hydrogen and oxygen basis is invalid.",
            ),
            Self::AqueousBasisDoesNotClose => bare(
                "solver-reason.inventory.aqueous-basis-does-not-close",
                "The aqueous basis does not close on hydrogen.",
            ),
            Self::ReservoirBoundaryUnsupported => bare(
                "solver-reason.inventory.reservoir-boundary-unsupported",
                "The reservoir opaque owner does not support this clock or finite gas boundary.",
            ),
            Self::VesselIdentityChanged => bare(
                "solver-reason.inventory.vessel-identity-changed",
                "The vessel identity changed.",
            ),
            Self::FormulaUnresolved => bare(
                "solver-reason.inventory.formula-unresolved",
                "A portion has an unresolved formula.",
            ),
            Self::AmountNonfinite => bare(
                "solver-reason.inventory.amount-nonfinite",
                "A portion has a nonfinite amount.",
            ),
            Self::AmountNegative => bare(
                "solver-reason.inventory.amount-negative",
                "A portion has a negative amount.",
            ),
            Self::GasCrossingWrongVessel => bare(
                "solver-reason.inventory.gas-crossing-wrong-vessel",
                "A gas crossing belongs to another vessel.",
            ),
            Self::GasCrossingUnowned => bare(
                "solver-reason.inventory.gas-crossing-unowned",
                "A gas crossing is not owned by a reservoir.",
            ),
            Self::GasAmountNonfinite => bare(
                "solver-reason.inventory.gas-amount-nonfinite",
                "The reservoir gas crossing amount is nonfinite.",
            ),
            Self::GasAmountNegative => bare(
                "solver-reason.inventory.gas-amount-negative",
                "The reservoir gas crossing amount is negative.",
            ),
            Self::ReservoirFormulaUnknown => bare(
                "solver-reason.inventory.reservoir-formula-unknown",
                "A reservoir gas has an unresolved formula.",
            ),
            Self::ElementLedgerNonfinite => bare(
                "solver-reason.inventory.element-ledger-nonfinite",
                "An element ledger contains a nonfinite total.",
            ),
            Self::ElementLedgerMismatch {
                element,
                before,
                after,
            } => Phrase::new(
                "solver-reason.inventory.element-ledger-mismatch",
                "The {element} element ledger does not close: {before} mol became {after} mol.",
                vec![
                    ("element".into(), Slot::text(element)),
                    ("before".into(), Slot::number(format!("{before:.15e}"))),
                    ("after".into(), Slot::number(format!("{after:.15e}"))),
                ],
            ),
            Self::ChargeLedgerNonfinite => bare(
                "solver-reason.inventory.charge-ledger-nonfinite",
                "The charge ledger contains a nonfinite total.",
            ),
            Self::ChargeLedgerMismatch { before, after } => Phrase::new(
                "solver-reason.inventory.charge-ledger-mismatch",
                "The charge ledger does not close: {before} eq became {after} eq.",
                vec![
                    ("before".into(), Slot::number(format!("{before:.15e}"))),
                    ("after".into(), Slot::number(format!("{after:.15e}"))),
                ],
            ),
        }
    }
}

fn valid_immutable_spectators(vessel: &Vessel, policy: ImmutablePolicy) -> bool {
    vessel.contents.iter().all(|portion| {
        if !portion.moles.0.is_finite() || portion.moles.0 < 0.0 {
            return false;
        }
        if portion.moles.0 == 0.0
            || portion.phase == Phase::Gas
            || derived::role(&portion.species.0).is_some()
            || species::is_acid_base_basis(&portion.species.0)
        {
            true
        } else {
            clock_spectator(portion)
                || (policy == ImmutablePolicy::ReservoirOptical && passive_ladder_reporter(portion))
        }
    })
}

fn non_native_spectators(vessel: &Vessel) -> Vec<(String, Phase, u64)> {
    let mut portions = vessel
        .contents
        .iter()
        .filter(|portion| {
            portion.moles.0 != 0.0
                && portion.phase != Phase::Gas
                && derived::role(&portion.species.0).is_none()
                && !species::is_acid_base_basis(&portion.species.0)
        })
        .map(|portion| {
            (
                portion.species.0.clone(),
                portion.phase,
                portion.moles.0.to_bits(),
            )
        })
        .collect::<Vec<_>>();
    portions.sort();
    portions
}

pub(crate) fn valid_clock_immutable_spectators(vessel: &Vessel) -> bool {
    valid_immutable_spectators(vessel, ImmutablePolicy::Clock)
}

pub(crate) fn valid_reservoir_immutable_spectators(vessel: &Vessel) -> bool {
    valid_immutable_spectators(vessel, ImmutablePolicy::ReservoirOptical)
}

/// The generic ledger parses the enzyme catalogue's placeholder `C` as a
/// formula unit. For this narrowly admitted catalyst the exact dose is
/// certified unchanged instead; subtract that placeholder from the physical
/// component comparison. No carbon atom count for pepsin is claimed.
fn clock_known_component_ledger(vessel: &Vessel, exclude_opaque_catalyst: bool) -> ConservedLedger {
    let mut ledger = ConservedLedger::from_vessel(vessel);
    if exclude_opaque_catalyst {
        for portion in &vessel.contents {
            if portion.species.0 == "pepsin" && clock_spectator(portion) {
                *ledger.elements.entry("C".into()).or_default() -= portion.moles.0;
            }
        }
    }
    ledger
}

pub(crate) fn certify_opaque_clock_stock(
    before: &Vessel,
    after: &Vessel,
) -> Result<(), SolveError> {
    certify_immutable_stock(before, after, ImmutablePolicy::Clock)
}

pub(crate) fn certify_opaque_reservoir_stock(
    before: &Vessel,
    after: &Vessel,
) -> Result<(), SolveError> {
    certify_immutable_stock(before, after, ImmutablePolicy::ReservoirOptical)
}

fn certify_immutable_stock(
    before: &Vessel,
    after: &Vessel,
    policy: ImmutablePolicy,
) -> Result<(), SolveError> {
    let same = before.unresolved_materials.len() == after.unresolved_materials.len()
        && before
            .unresolved_materials
            .iter()
            .zip(&after.unresolved_materials)
            .all(|(left, right)| {
                left == right
                    && left.amount.to_bits() == right.amount.to_bits()
                    && left.protein_denatured_fraction.to_bits()
                        == right.protein_denatured_fraction.to_bits()
                    && match (&left.enzyme_hydrolysis, &right.enzyme_hydrolysis) {
                        (Some(a), Some(b)) => {
                            a.converted_fraction.to_bits() == b.converted_fraction.to_bits()
                        }
                        (None, None) => true,
                        _ => false,
                    }
            });
    let detail = match policy {
        ImmutablePolicy::Clock => {
            "opaque clock inventory or immutable spectator changed or is invalid"
        }
        ImmutablePolicy::ReservoirOptical => {
            "reservoir aqueous opaque inventory or immutable spectator changed or is invalid"
        }
    };
    if !valid_opaque_clock_stock(before) || !valid_opaque_clock_stock(after) {
        return Err(inventory_error(
            InventoryReason::Immutable {
                policy,
                failure: ImmutableFailure::InvalidOpaque,
            },
            detail,
        ));
    }
    if !same {
        return Err(inventory_error(
            InventoryReason::Immutable {
                policy,
                failure: ImmutableFailure::ChangedOpaque,
            },
            detail,
        ));
    }
    if !valid_immutable_spectators(before, policy) || !valid_immutable_spectators(after, policy) {
        return Err(inventory_error(
            InventoryReason::Immutable {
                policy,
                failure: ImmutableFailure::InvalidSpectator,
            },
            detail,
        ));
    }
    if non_native_spectators(before) != non_native_spectators(after) {
        return Err(inventory_error(
            InventoryReason::Immutable {
                policy,
                failure: ImmutableFailure::ChangedSpectator,
            },
            detail,
        ));
    }
    Ok(())
}

fn immutable_phrase(
    policy: ImmutablePolicy,
    failure: ImmutableFailure,
) -> (&'static str, &'static str) {
    match (policy, failure) {
        (ImmutablePolicy::Clock, ImmutableFailure::InvalidOpaque) => (
            "solver-reason.inventory.clock-invalid-opaque-stock",
            "The opaque clock stock is invalid.",
        ),
        (ImmutablePolicy::Clock, ImmutableFailure::ChangedOpaque) => (
            "solver-reason.inventory.clock-changed-opaque-stock",
            "The opaque clock stock or progress changed.",
        ),
        (ImmutablePolicy::Clock, ImmutableFailure::InvalidSpectator) => (
            "solver-reason.inventory.clock-invalid-spectator",
            "A clock spectator is invalid.",
        ),
        (ImmutablePolicy::Clock, ImmutableFailure::ChangedSpectator) => (
            "solver-reason.inventory.clock-changed-spectator",
            "A clock spectator changed.",
        ),
        (ImmutablePolicy::ReservoirOptical, ImmutableFailure::InvalidOpaque) => (
            "solver-reason.inventory.reservoir-invalid-opaque-stock",
            "The reservoir opaque stock is invalid.",
        ),
        (ImmutablePolicy::ReservoirOptical, ImmutableFailure::ChangedOpaque) => (
            "solver-reason.inventory.reservoir-changed-opaque-stock",
            "The reservoir opaque stock or progress changed.",
        ),
        (ImmutablePolicy::ReservoirOptical, ImmutableFailure::InvalidSpectator) => (
            "solver-reason.inventory.reservoir-invalid-spectator",
            "A reservoir spectator is invalid.",
        ),
        (ImmutablePolicy::ReservoirOptical, ImmutableFailure::ChangedSpectator) => (
            "solver-reason.inventory.reservoir-changed-spectator",
            "A reservoir spectator changed.",
        ),
    }
}

fn inventory_error(reason: InventoryReason, detail: impl Into<String>) -> SolveError {
    SolveError::NotConverged {
        reason: Some(Box::new(reason.phrase())),
        solver: "phreeqc-inventory".into(),
        detail: detail.into(),
    }
}

fn clock_formula(
    clock: ClockCo2Boundary,
    vessel: VesselId,
) -> Result<Option<kerotakis_core::stoich::Formula>, SolveError> {
    if clock.vessel != vessel || !clock.applied_mol.is_finite() {
        return Err(inventory_error(
            InventoryReason::ClockBoundaryInvalid,
            "invalid clock CO2 boundary owner or amount",
        ));
    }
    if clock.applied_mol == 0.0 {
        return Ok(None);
    }
    let formula = species::lookup_key("CO2")
        .and_then(|row| kerotakis_core::stoich::parse_formula(row.formula).ok())
        .ok_or_else(|| {
            inventory_error(
                InventoryReason::ClockFormulaUnknown,
                "clock CO2 has no known physical formula",
            )
        })?;
    Ok(Some(formula))
}

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
    clock: ClockCo2Boundary,
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
    if let Some(formula) = clock_formula(clock, before.id)? {
        let h = formula.counts.get("H").copied().unwrap_or(0.0);
        let o = formula.counts.get("O").copied().unwrap_or(0.0);
        gas_excess += clock.applied_mol * (h - 2.0 * o);
        for (element, count) in formula.counts {
            *target.entry(element).or_default() += clock.applied_mol * count;
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
        return Err(inventory_error(
            InventoryReason::AqueousBasisInvalid,
            format!(
                "no nonnegative aqueous basis conserves H/O: residual H={hydrogen}, O={oxygen}"
            ),
        ));
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
        return Err(inventory_error(
            InventoryReason::AqueousBasisDoesNotClose,
            format!(
                "the aqueous basis does not close on hydrogen: residual H={hydrogen}, \
                 O={oxygen}, rebuilt H={reconstructed}"
            ),
        ));
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

/// The admitted gas-aware component projection must also close the physical
/// ledger after the H/O basis is rebuilt. The source charge is authoritative:
/// a deliberately charged inlet is not silently neutralised.
pub(crate) fn certify_gas_aware_inventory(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    reservoirs: &[&str],
    clock: ClockCo2Boundary,
) -> Result<(), SolveError> {
    certify_component_inventory(
        before,
        after,
        events,
        reservoirs,
        clock,
        ImmutablePolicy::Clock,
    )
}

/// The optical opaque route shares the existing signed gas-reservoir ledger.
/// The clock remains a separate owner; its parked or spent parcel cannot be
/// converted into an atmospheric crossing by this wrapper.
pub(crate) fn certify_reservoir_opaque_inventory(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    reservoirs: &[&str],
) -> Result<(), SolveError> {
    let unsupported_boundary = || {
        inventory_error(
            InventoryReason::ReservoirBoundaryUnsupported,
            "reservoir opaque aqueous owner has a clock or finite gas boundary",
        )
    };
    if before.unresolved_materials.is_empty()
        || after.unresolved_materials.is_empty()
        || before.pending_co2_transfer_mol != 0.0
        || after.pending_co2_transfer_mol != 0.0
        || !matches!(&before.headspace, kerotakis_core::Headspace::Open)
        || !matches!(&after.headspace, kerotakis_core::Headspace::Open)
        || before.owns_headspace_gas()
        || after.owns_headspace_gas()
        || before
            .contents
            .iter()
            .any(|portion| portion.phase == Phase::Gas)
        || after
            .contents
            .iter()
            .any(|portion| portion.phase == Phase::Gas)
        || events
            .iter()
            .any(|event| matches!(event, Event::GasContained { .. }))
    {
        return Err(unsupported_boundary());
    }
    certify_component_inventory(
        before,
        after,
        events,
        reservoirs,
        ClockCo2Boundary::none(before.id),
        ImmutablePolicy::ReservoirOptical,
    )
}

fn certify_component_inventory(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    reservoirs: &[&str],
    clock: ClockCo2Boundary,
    policy: ImmutablePolicy,
) -> Result<(), SolveError> {
    let prefix = match policy {
        ImmutablePolicy::Clock => "gas-aware physical ledger",
        ImmutablePolicy::ReservoirOptical => "reservoir opaque physical ledger",
    };
    let refused = |reason: InventoryReason, detail: String| {
        inventory_error(reason, format!("{prefix}: {detail}"))
    };
    if before.id != after.id {
        return Err(refused(
            InventoryReason::VesselIdentityChanged,
            "vessel identity changed".into(),
        ));
    }
    if policy == ImmutablePolicy::ReservoirOptical
        || !before.unresolved_materials.is_empty()
        || !after.unresolved_materials.is_empty()
    {
        certify_immutable_stock(before, after, policy)?;
    }
    for vessel in [before, after] {
        for portion in &vessel.contents {
            let formula = species::lookup(&portion.species)
                .and_then(|data| kerotakis_core::stoich::parse_formula(data.formula).ok());
            if formula.is_none() {
                return Err(refused(
                    InventoryReason::FormulaUnresolved,
                    format!("unresolved portion {}", portion.species.0),
                ));
            }
            if !portion.moles.0.is_finite() {
                return Err(refused(
                    InventoryReason::AmountNonfinite,
                    format!("unresolved portion {}", portion.species.0),
                ));
            }
            if portion.moles.0 < 0.0 {
                return Err(refused(
                    InventoryReason::AmountNegative,
                    format!("unresolved portion {}", portion.species.0),
                ));
            }
        }
    }
    let exclude_opaque_catalyst =
        policy == ImmutablePolicy::ReservoirOptical || !before.unresolved_materials.is_empty();
    let mut expected = clock_known_component_ledger(before, exclude_opaque_catalyst);
    for event in events {
        let (event_vessel, species_id, moles, sign) = match event {
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => (vessel, species, moles.0, -1.0),
            Event::GasAbsorbed {
                vessel,
                species,
                moles,
            } => (vessel, species, moles.0, 1.0),
            _ => continue,
        };
        if *event_vessel != before.id {
            return Err(refused(
                InventoryReason::GasCrossingWrongVessel,
                format!("unowned gas crossing {}", species_id.0),
            ));
        }
        if !reservoirs.contains(&species_id.0.as_str()) {
            return Err(refused(
                InventoryReason::GasCrossingUnowned,
                format!("unowned gas crossing {}", species_id.0),
            ));
        }
        if !moles.is_finite() {
            return Err(refused(
                InventoryReason::GasAmountNonfinite,
                format!("unowned gas crossing {}", species_id.0),
            ));
        }
        if moles < 0.0 {
            return Err(refused(
                InventoryReason::GasAmountNegative,
                format!("unowned gas crossing {}", species_id.0),
            ));
        }
        let signed_moles = sign * moles;
        let formula = species::lookup(species_id)
            .and_then(|data| kerotakis_core::stoich::parse_formula(data.formula).ok())
            .ok_or_else(|| {
                refused(
                    InventoryReason::ReservoirFormulaUnknown,
                    format!("unresolved reservoir gas {}", species_id.0),
                )
            })?;
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += count * signed_moles;
        }
        expected.charge += formula.charge * signed_moles;
    }
    if let Some(formula) = clock_formula(clock, before.id)? {
        for (element, count) in formula.counts {
            *expected.elements.entry(element).or_default() += count * clock.applied_mol;
        }
        expected.charge += formula.charge * clock.applied_mol;
    }
    let actual = clock_known_component_ledger(after, exclude_opaque_catalyst);
    let elements: BTreeSet<_> = expected
        .elements
        .keys()
        .chain(actual.elements.keys())
        .collect();
    for element in elements {
        let left = expected.elements.get(element).copied().unwrap_or(0.0);
        let right = actual.elements.get(element).copied().unwrap_or(0.0);
        if !left.is_finite() || !right.is_finite() {
            return Err(refused(
                InventoryReason::ElementLedgerNonfinite,
                format!("{element} changed {left:.15e} -> {right:.15e} mol"),
            ));
        }
        if (right - left).abs() > 1e-10 {
            return Err(inventory_error(
                InventoryReason::ElementLedgerMismatch {
                    element: element.to_string(),
                    before: left,
                    after: right,
                },
                format!("{prefix}: {element} changed {left:.15e} -> {right:.15e} mol"),
            ));
        }
    }
    if !expected.charge.is_finite() || !actual.charge.is_finite() {
        return Err(refused(
            InventoryReason::ChargeLedgerNonfinite,
            format!(
                "charge changed {:.15e} -> {:.15e} eq",
                expected.charge, actual.charge
            ),
        ));
    }
    if (actual.charge - expected.charge).abs() > 1e-10 {
        return Err(inventory_error(
            InventoryReason::ChargeLedgerMismatch {
                before: expected.charge,
                after: actual.charge,
            },
            format!(
                "{prefix}: charge changed {:.15e} -> {:.15e} eq",
                expected.charge, actual.charge
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod reservoir_opaque_immutable_certificate_tests {
    use super::*;
    use kerotakis_core::{
        material::MaterialBasis, vessel::UnresolvedMaterialPortion, Moles, VesselId,
    };

    fn reporter() -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "reservoir reporter mechanism");
        vessel.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        vessel.deposit(
            SpeciesId::new("anthocyanin"),
            Moles(6.690_753_378_830_455e-6),
            Phase::Aqueous,
        );
        vessel.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "red_cabbage_juice".into(),
            recipe_id: "household/red-cabbage-juice-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 0.197,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        vessel
    }

    #[test]
    fn passive_ladder_is_reservoir_only_and_every_spectator_bit_is_immutable() {
        let original = reporter();
        let pigment = original
            .contents
            .iter()
            .find(|portion| portion.species.0 == "anthocyanin")
            .unwrap();
        assert!(passive_ladder_reporter(pigment));
        assert!(valid_reservoir_immutable_spectators(&original));
        assert!(!valid_clock_immutable_spectators(&original));
        assert!(certify_reservoir_opaque_inventory(&original, &original, &[], &[]).is_ok());

        let mut changed = original.clone();
        changed.unresolved_materials[0].amount += 1e-14;
        assert!(certify_opaque_reservoir_stock(&original, &changed).is_err());
        changed = original.clone();
        changed
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "anthocyanin")
            .unwrap()
            .moles
            .0 += 1e-14;
        assert!(certify_opaque_reservoir_stock(&original, &changed).is_err());
        changed = original.clone();
        changed
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "anthocyanin")
            .unwrap()
            .phase = Phase::Solid;
        assert!(!valid_reservoir_immutable_spectators(&changed));
        assert!(certify_opaque_reservoir_stock(&original, &changed).is_err());
    }

    #[test]
    fn reservoir_owner_checks_signed_gas_and_rejects_unknown_ions_or_clock() {
        let original = reporter();
        let mut ionic = original.clone();
        ionic.deposit(SpeciesId::new("KIO3"), Moles(0.001), Phase::Aqueous);
        assert!(!valid_reservoir_immutable_spectators(&ionic));
        assert!(certify_reservoir_opaque_inventory(&ionic, &ionic, &[], &[]).is_err());

        let mut clock = original.clone();
        clock.pending_co2_transfer_mol = 2e-6;
        assert!(certify_reservoir_opaque_inventory(&clock, &clock, &[], &[]).is_err());
        assert!(certify_reservoir_opaque_inventory(
            &original,
            &original,
            &[Event::GasEvolved {
                vessel: original.id,
                species: SpeciesId::new("CO2"),
                moles: Moles(1e-12),
            }],
            &[],
        )
        .is_err());

        let mut carbon = original.clone();
        carbon.deposit(SpeciesId::new("CO2"), Moles(1e-6), Phase::Aqueous);
        let lost = Event::GasEvolved {
            vessel: carbon.id,
            species: SpeciesId::new("CO2"),
            moles: Moles(1e-6),
        };
        assert!(certify_reservoir_opaque_inventory(&carbon, &original, &[lost], &["CO2"],).is_ok());
    }
}

#[cfg(test)]
mod gas_aware_certificate_tests {
    use super::*;
    use kerotakis_core::{
        enzyme::EnzymeFamily,
        material::MaterialBasis,
        vessel::{EnzymeHydrolysisState, UnresolvedMaterialPortion},
        VesselId,
    };

    #[test]
    fn opaque_clock_owner_preserves_every_unresolved_field_and_rejects_invalid_stock() {
        let mut before = Vessel::new(VesselId(0), "opaque owner");
        before.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "whole_milk".into(),
            recipe_id: "household/whole-milk-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 12.0,
            enzyme_hydrolysis: Some(EnzymeHydrolysisState {
                family: EnzymeFamily::Lactase,
                converted_fraction: 0.25,
                carried_enzyme_denatured: false,
            }),
            protein_denatured_fraction: 0.0,
        });
        assert!(certify_opaque_clock_stock(&before, &before).is_ok());
        let mut changed = before.clone();
        changed.unresolved_materials[0].amount = 11.999;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        changed = before.clone();
        changed.unresolved_materials[0].recipe_version = 2;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        changed = before.clone();
        changed.unresolved_materials[0].protein_denatured_fraction = 0.1;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        changed = before.clone();
        changed.unresolved_materials[0]
            .enzyme_hydrolysis
            .as_mut()
            .expect("progress")
            .converted_fraction = 0.26;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        changed = before.clone();
        changed.unresolved_materials[0].amount = f64::NAN;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        changed.unresolved_materials[0].amount = -1.0;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
    }

    #[test]
    fn clock_spectators_are_exact_immutable_stock_not_new_native_components() {
        let mut before = Vessel::new(VesselId(0), "reviewed neutral spectators");
        before.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "cabbage remainder".into(),
            recipe_id: "food/cabbage".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 1.8,
            enzyme_hydrolysis: None,
            protein_denatured_fraction: 0.0,
        });
        before.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        before.deposit(SpeciesId::new("glucose"), Moles(0.01), Phase::Aqueous);
        before.deposit(SpeciesId::new("fructose"), Moles(0.01), Phase::Aqueous);
        before.deposit(SpeciesId::new("cellulose"), Moles(0.01), Phase::Solid);
        before.deposit(SpeciesId::new("pepsin"), Moles(0.001), Phase::Aqueous);
        assert!(valid_clock_immutable_spectators(&before));
        assert!(certify_opaque_clock_stock(&before, &before).is_ok());
        // The catalyst's catalogue formula is a placeholder. Its amount is
        // proved unchanged; the component ledger must not claim its C atom.
        let known = clock_known_component_ledger(&before, true);
        assert!((known.elements["C"] - 0.18).abs() < 1e-12);

        let mut changed = before.clone();
        changed
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "glucose")
            .expect("sugar")
            .moles
            .0 += 1e-13;
        assert!(
            certify_opaque_clock_stock(&before, &changed).is_err(),
            "even sub-budget spectator drift is not a native correction"
        );
        changed = before.clone();
        changed
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "cellulose")
            .expect("fibre")
            .phase = Phase::Aqueous;
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        for (species, phase) in [
            ("KIO3", Phase::Aqueous),
            ("malic_acid", Phase::Aqueous),
            ("unknown_clock_ion", Phase::Solid),
        ] {
            changed = before.clone();
            changed.deposit(SpeciesId::new(species), Moles(0.001), phase);
            assert!(
                !valid_clock_immutable_spectators(&changed),
                "{species} cannot inherit the neutral witness"
            );
            assert!(certify_opaque_clock_stock(&before, &changed).is_err());
        }
        changed = before.clone();
        changed
            .contents
            .iter_mut()
            .find(|p| p.species.0 == "pepsin")
            .expect("catalyst")
            .moles
            .0 = f64::NAN;
        assert!(!valid_clock_immutable_spectators(&changed));
        assert!(certify_opaque_clock_stock(&before, &changed).is_err());
    }

    #[test]
    fn charged_input_is_preserved_and_same_element_charge_change_is_refused() {
        let mut charged = Vessel::new(VesselId(0), "charged input");
        charged.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        charged.deposit(SpeciesId::new("Na+"), Moles(0.01), Phase::Aqueous);
        assert!(certify_gas_aware_inventory(
            &charged,
            &charged,
            &[],
            &[],
            ClockCo2Boundary::none(charged.id)
        )
        .is_ok());

        let mut before = Vessel::new(VesselId(0), "formal charge control");
        before.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        before.deposit(SpeciesId::new("Fe+2"), Moles(0.001), Phase::Aqueous);
        let mut after = before.clone();
        after.contents.retain(|portion| portion.species.0 != "Fe+2");
        after.deposit(SpeciesId::new("Fe+3"), Moles(0.001), Phase::Aqueous);
        let error = certify_gas_aware_inventory(
            &before,
            &after,
            &[],
            &[],
            ClockCo2Boundary::none(before.id),
        )
        .unwrap_err();
        assert!(error.to_string().contains("charge changed"), "{error}");
        assert_eq!(
            error.owned_reason().map(|reason| reason.key.as_str()),
            Some("solver-reason.inventory.charge-ledger-mismatch")
        );
        assert_eq!(
            error.to_string(),
            "phreeqc-inventory could not solve this state: gas-aware physical ledger: charge changed 2.000000000000000e-3 -> 3.000000000000000e-3 eq"
        );
    }

    #[test]
    fn unresolved_formula_reason_keeps_unknown_id_in_raw_detail() {
        let mut vessel = Vessel::new(VesselId(9), "unknown formula");
        vessel.deposit(
            SpeciesId::new("unregistered_mystery_ion"),
            Moles(0.125),
            Phase::Aqueous,
        );
        let error = certify_gas_aware_inventory(
            &vessel,
            &vessel,
            &[],
            &[],
            ClockCo2Boundary::none(vessel.id),
        )
        .unwrap_err();
        assert_eq!(
            error.owned_reason().map(|reason| reason.key.as_str()),
            Some("solver-reason.inventory.formula-unresolved")
        );
        assert_eq!(
            error.to_string(),
            "phreeqc-inventory could not solve this state: gas-aware physical ledger: unresolved portion unregistered_mystery_ion"
        );
    }

    #[test]
    fn gas_boundary_sign_vessel_and_nonnegative_amount_are_checked() {
        let mut before = Vessel::new(VesselId(0), "gas boundary");
        before.deposit(SpeciesId::new("CO2"), Moles(0.001), Phase::Aqueous);
        let after = Vessel::new(VesselId(0), "gas boundary");
        let out = Event::GasEvolved {
            vessel: before.id,
            species: SpeciesId::new("CO2"),
            moles: Moles(0.001),
        };
        assert!(certify_gas_aware_inventory(
            &before,
            &after,
            &[out],
            &["CO2"],
            ClockCo2Boundary::none(before.id)
        )
        .is_ok());
        let invalid = Event::GasEvolved {
            vessel: before.id,
            species: SpeciesId::new("CO2"),
            moles: Moles(-0.001),
        };
        assert!(certify_gas_aware_inventory(
            &before,
            &after,
            &[invalid],
            &["CO2"],
            ClockCo2Boundary::none(before.id)
        )
        .is_err());
        let wrong_vessel = Event::GasEvolved {
            vessel: VesselId(1),
            species: SpeciesId::new("CO2"),
            moles: Moles(0.001),
        };
        assert!(certify_gas_aware_inventory(
            &before,
            &after,
            &[wrong_vessel],
            &["CO2"],
            ClockCo2Boundary::none(before.id)
        )
        .is_err());
    }

    #[test]
    fn virtual_clock_carbon_is_signed_and_independent_of_native_reservoirs() {
        let mut before = Vessel::new(VesselId(7), "clock boundary");
        before.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
        let mut after = before.clone();
        after.deposit(SpeciesId::new("CO2"), Moles(2e-6), Phase::Aqueous);
        let inlet = ClockCo2Boundary {
            vessel: before.id,
            applied_mol: 2e-6,
        };
        assert!(certify_gas_aware_inventory(&before, &after, &[], &[], inlet).is_ok());
        let outlet = ClockCo2Boundary {
            applied_mol: -2e-6,
            ..inlet
        };
        assert!(certify_gas_aware_inventory(&after, &before, &[], &[], outlet).is_ok());
        assert!(certify_gas_aware_inventory(&before, &after, &[], &[], outlet).is_err());
        assert!(certify_gas_aware_inventory(
            &before,
            &after,
            &[],
            &[],
            ClockCo2Boundary {
                vessel: VesselId(8),
                ..inlet
            },
        )
        .is_err());
        assert!(certify_gas_aware_inventory(
            &before,
            &after,
            &[],
            &[],
            ClockCo2Boundary {
                applied_mol: f64::NAN,
                ..inlet
            },
        )
        .is_err());
    }
}
