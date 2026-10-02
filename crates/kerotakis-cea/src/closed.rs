//! Boundary contract for the existing open-room CEA thermal route.
//!
//! A rigid adiabatic vessel requires U/V equilibrium, not the open route's
//! H/P balance. A pressure-controlled finite vessel needs its own closed H/P
//! balance with retained products and volume work. A sweep needs an explicit
//! carrier-gas inlet/outlet model. Until those routes have independent energy
//! and pressure validation, none may borrow oxygen or vent finite inventory.
use kerotakis_core::phrase::Phrase;
use kerotakis_core::{ops::NotModelledCause, vessel::Headspace, Event, Vessel};

pub(crate) fn boundary_refusal(vessel: &Vessel) -> Option<Event> {
    let reason = match vessel.headspace {
        Headspace::Open => return None,
        Headspace::Sealed { .. } => Phrase::bare(
            "not-modeled.cea-sealed-energy-boundary",
            "the sealed vessel keeps its gas inventory; the open-flame CEA route cannot compute its fixed-volume energy and pressure balance, so no thermal chemistry has been committed",
        ),
        Headspace::PressureControlled { .. } => Phrase::bare(
            "not-modeled.cea-finite-pressure-boundary",
            "the pressure-controlled vessel keeps a finite gas inventory; the open-flame CEA route has no validated closed enthalpy and volume-work balance, so no thermal chemistry has been committed",
        ),
        Headspace::Swept { .. } => Phrase::bare(
            "not-modeled.cea-swept-boundary",
            "the swept vessel does not admit room oxygen; the open-flame CEA route has no carrier-gas inlet and outlet balance for this boundary, so no thermal chemistry has been committed",
        ),
    };
    Some(Event::not_modeled(
        vessel.id,
        NotModelledCause::BoundaryMismatch,
        reason,
    ))
}
