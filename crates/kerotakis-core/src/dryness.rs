//! What a solution leaves behind when its solvent is taken to dryness.
//!
//! C1 audit, 2026-11-24: evaporating brine to dryness left "0.0183 mol
//! chloride ion, Aqueous" and "0.0183 mol sodium ion, Aqueous" in a dry
//! beaker, with the sentence that the solids "are not decidable from the
//! ions alone". That is true of SOME residues and false of most of the ones
//! a learner makes. Which salts a mixed solution dries to is genuinely
//! ambiguous only for a *reciprocal* system — two or more cations AND two or
//! more anions, where Na⁺/K⁺/Cl⁻/NO₃⁻ can dry to NaCl + KNO₃ or KCl + NaNO₃.
//! When every anion shares one cation (or every cation shares one anion),
//! electroneutrality fixes the residue uniquely: each ion can only leave as
//! the salt it forms with the common partner. Sodium chloride from brine,
//! sodium chloride + sodium sulfate from a mixed sodium solution, calcium
//! acetate from limewater neutralised with vinegar.
//!
//! A neutral molecular solute (sucrose) is simpler still: when its water is
//! gone it is the solid it was dissolved from, the same registry species.
//!
//! # The rule
//!
//! 1. A neutral aqueous solute whose registry species is a solid at room
//!    conditions is re-filed as that solid.
//! 2. Ions are paired only across a common ion (one cation species, or one
//!    anion species). Each partner ion is matched to the registry solid
//!    whose formula is exactly the charge-balanced combination, anhydrous
//!    and not a condensed gas. The formula is matched by element counts
//!    parsed from the registry, never by name.
//! 3. Everything else — a reciprocal system, an ion with no solid in the
//!    registry, the acid/base basis (H⁺, OH⁻ and base equivalents, which
//!    are analytical coordinates and not free acid or hydroxide), a
//!    dissolved liquid or gas — stays where it was and is named by the
//!    existing stranded-solutes sentence. Nothing is guessed.
//!
//! # What is not modelled, and said
//!
//! The residue is booked ANHYDROUS: the water of crystallisation a real
//! dish keeps (MgCl₂·6H₂O, Na₂SO₄·10H₂O) left with the solvent, because
//! the step that removed the water does not know how much a hydrate would
//! have held back. And no heat of crystallisation is charged at dryness —
//! `evaporate` already books no heat for the water it removes. Both are
//! stated by one typed boundary note every time a residue is booked.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase, SpeciesId};
use crate::stoich::{parse_formula, Formula};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Amounts below this are float dust, not residue.
const TRACE: f64 = 1e-12;

/// The solvent, which is the thing that left.
const SOLVENT: &str = "water";

fn formula_of(key: &str) -> Option<Formula> {
    let data = species::lookup(&SpeciesId::new(key))?;
    parse_formula(data.formula).ok()
}

fn same_counts(a: &Formula, b: &std::collections::BTreeMap<String, f64>) -> bool {
    a.counts.len() == b.len()
        && b.iter().all(|(el, n)| {
            a.counts
                .get(el)
                .is_some_and(|m| (m - n).abs() <= 1e-9 * n.abs().max(1.0))
        })
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// The registry solid formed by one cation and one anion, with how many of
/// each it holds per formula unit: `(key, cations, anions)`.
///
/// Matched on parsed element counts so that the registry's own formula is
/// the authority: a salt whose formula is not exactly the charge-balanced
/// combination is never offered, and a hydrate (which carries water the
/// dry dish no longer has) is never chosen.
pub fn salt_of(cation: &str, anion: &str) -> Option<(&'static str, f64, f64)> {
    let c = formula_of(cation)?;
    let a = formula_of(anion)?;
    if c.charge <= 0.0 || a.charge >= 0.0 {
        return None;
    }
    let zc = c.charge.round();
    let za = (-a.charge).round();
    if (zc - c.charge).abs() > 1e-9 || (za + a.charge).abs() > 1e-9 {
        return None;
    }
    let g = gcd(zc as u64, za as u64) as f64;
    let (nc, na) = (za / g, zc / g);
    let mut target = std::collections::BTreeMap::<String, f64>::new();
    for (el, n) in &c.counts {
        *target.entry(el.clone()).or_default() += n * nc;
    }
    for (el, n) in &a.counts {
        *target.entry(el.clone()).or_default() += n * na;
    }
    species::registry().iter().find_map(|candidate| {
        if candidate.standard_phase != Phase::Solid
            || crate::phase_route::split_hydrate(candidate.formula).is_some()
            || crate::phase_route::is_condensed_gas(candidate.key)
        {
            return None;
        }
        let f = parse_formula(candidate.formula).ok()?;
        (f.charge == 0.0 && same_counts(&f, &target)).then_some((candidate.key, nc, na))
    })
}

/// What [`crystallise_to_dryness`] did.
#[derive(Debug, Default)]
pub struct DryResidue {
    /// Events describing every solid booked.
    pub events: Vec<Event>,
    /// Display names of the solutes still filed aqueous afterwards: the
    /// ones whose solid is not decidable, or not in the registry.
    pub stranded: Vec<&'static str>,
}

/// Book the residue of a vessel whose solvent has just left.
///
/// Called only by the two passes that removed the water (`evaporate`, and
/// the boiling branch of [`crate::solve::StateEquilibrator`]), for the
/// reason [`crate::solve::stranded_solutes`] gives: only they know the water
/// LEFT rather than never having been there.
pub fn crystallise_to_dryness(vessel: &mut Vessel) -> DryResidue {
    let mut out = DryResidue::default();
    let id = vessel.id;
    let mut booked: Vec<&'static str> = Vec::new();

    // Aqueous inventory by species (a species may sit in several lots).
    let mut aqueous: Vec<(String, f64)> = Vec::new();
    for p in &vessel.contents {
        if p.phase != Phase::Aqueous || p.species.0 == SOLVENT || p.moles.0 <= TRACE {
            continue;
        }
        match aqueous.iter_mut().find(|(k, _)| *k == p.species.0) {
            Some((_, n)) => *n += p.moles.0,
            None => aqueous.push((p.species.0.clone(), p.moles.0)),
        }
    }

    // 1. Neutral molecular solutes that are solids at room conditions.
    let mut cations: Vec<(String, f64)> = Vec::new();
    let mut anions: Vec<(String, f64)> = Vec::new();
    for (key, n) in &aqueous {
        // The acid/base basis (H+, OH-, base equivalents) is an analytical
        // coordinate, not an amount of free acid or base: pairing it would
        // invent a hydroxide or an acid salt. It is left stranded.
        if species::is_acid_base_basis(key) {
            continue;
        }
        let Some(f) = formula_of(key) else { continue };
        if f.charge > 0.0 {
            cations.push((key.clone(), *n));
        } else if f.charge < 0.0 {
            anions.push((key.clone(), *n));
        } else if let Some(data) = species::lookup(&SpeciesId::new(key)) {
            if data.standard_phase == Phase::Solid && !crate::phase_route::is_condensed_gas(key) {
                let sid = SpeciesId::new(key.as_str());
                let moved = vessel.withdraw_phase(&sid, Moles(*n), Phase::Aqueous);
                vessel.deposit(sid.clone(), moved, Phase::Solid);
                out.events.push(Event::Precipitated {
                    vessel: id,
                    species: sid,
                    moles: moved,
                    dry: false,
                });
                booked.push(data.name);
            }
        }
    }

    // 2. Ions, across a common ion only.
    let (shared, partners, shared_is_cation) = match (cations.len(), anions.len()) {
        (1, n) if n >= 1 => (cations[0].clone(), anions.clone(), true),
        (n, 1) if n >= 1 => (anions[0].clone(), cations.clone(), false),
        _ => (Default::default(), Vec::new(), false),
    };
    // (salt, units, partner key, partner moles used, shared moles used)
    let mut plan: Vec<(&'static str, f64, String, f64, f64)> = Vec::new();
    for (partner, n_partner) in &partners {
        let pair = if shared_is_cation {
            salt_of(&shared.0, partner).map(|(s, nc, na)| (s, na, nc))
        } else {
            salt_of(partner, &shared.0)
        };
        // `pair` is (salt, partner count, shared count) per formula unit.
        let Some((salt, per_partner, per_shared)) = pair else {
            continue;
        };
        let units = n_partner / per_partner;
        plan.push((salt, units, partner.clone(), *n_partner, units * per_shared));
    }
    let need: f64 = plan.iter().map(|row| row.4).sum();
    if need > TRACE {
        // The shared ion is the scarcer side only when something else (H⁺,
        // OH⁻, an unpaired ion) carried part of the charge; then every salt
        // is scaled back together rather than one being starved.
        let scale = (shared.1 / need).min(1.0);
        for (salt, units, partner, used_partner, used_shared) in plan {
            let units = units * scale;
            if units <= TRACE {
                continue;
            }
            vessel.withdraw_phase(
                &SpeciesId::new(partner.as_str()),
                Moles(used_partner * scale),
                Phase::Aqueous,
            );
            vessel.withdraw_phase(
                &SpeciesId::new(shared.0.as_str()),
                Moles(used_shared * scale),
                Phase::Aqueous,
            );
            let sid = SpeciesId::new(salt);
            vessel.deposit(sid.clone(), Moles(units), Phase::Solid);
            out.events.push(Event::Precipitated {
                vessel: id,
                species: sid,
                moles: Moles(units),
                dry: false,
            });
            if let Some(data) = species::lookup(&SpeciesId::new(salt)) {
                booked.push(data.name);
            }
        }
    }

    if !booked.is_empty() {
        out.events.push(Event::not_modeled(
            id,
            NotModelledCause::ModelBoundary,
            dry_residue_note(&booked),
        ));
    }
    out.stranded = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Aqueous && p.moles.0 > TRACE)
        .filter_map(|p| species::lookup(&p.species).map(|d| d.name))
        .collect();
    out
}

/// The typed boundary for a dry vessel whose remaining dissolved portions
/// are charged a Kopp's-rule heat capacity (`vessel::residue_molar`), or
/// `None` when nothing is estimated. Said by the same two passes that name
/// stranded solutes, after the decidable residue has been booked.
pub fn residue_heat_capacity_event(vessel: &Vessel) -> Option<Event> {
    let dry = vessel.solvent_gone();
    let mut estimated: Vec<&'static str> = Vec::new();
    for p in &vessel.contents {
        if p.moles.0 <= TRACE {
            continue;
        }
        if let Some(data) = species::lookup(&p.species) {
            if crate::vessel::residue_molar(data, p.phase, dry).is_some()
                && !estimated.contains(&data.name)
            {
                estimated.push(data.name);
            }
        }
    }
    (!estimated.is_empty()).then(|| {
        Event::not_modeled(
            vessel.id,
            NotModelledCause::ModelBoundary,
            residue_heat_capacity_note(&estimated),
        )
    })
}

/// The boundary a dry residue charged by Kopp's rule carries.
pub fn residue_heat_capacity_note(names: &[&str]) -> Phrase {
    Phrase::new(
        "not-modeled.dry-residue-kopp-heat-capacity",
        "with the water gone, {names} are warmed as the solid residue they are, at a heat capacity estimated by Kopp's rule (the sum of tabulated atomic contributions, Kopp 1865) rather than a measured value; heat the residue cannot hold below the source's temperature is reported undelivered",
        vec![(
            "names".to_string(),
            Slot::terms("species", names.iter().copied()),
        )],
    )
}

/// The boundary every booked dry residue carries.
pub fn dry_residue_note(names: &[&str]) -> Phrase {
    Phrase::new(
        "not-modeled.dry-residue-anhydrous",
        "the dry residue is booked as {names}: a dissolved molecule dries to its own solid, and ions that share one common partner can form only these salts; the residue is booked anhydrous, so any water of crystallisation a real dish keeps and the heat of crystallisation at dryness are not modelled",
        vec![(
            "names".to_string(),
            Slot::terms("species", names.iter().copied()),
        )],
    )
}
