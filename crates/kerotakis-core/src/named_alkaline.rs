//! Conserved strong-base reserve for an explicitly bounded named reaction.
//!
//! In this closed pool of certified strong carriers (the shared
//! [`crate::strong_reserve::CARRIERS`] table: Na/K/Ba with halide, nitrate
//! and sulfate) plus acetate, let M be total
//! spectator-cation equivalents, X strong-acid anion equivalents and A total
//! acetate in every admitted protonation/salt form. M-X-A is the strong-base
//! excess after the entire weak-acid reservoir has been neutralised. It is
//! NOT the analytical H/O basis coordinate and NOT instantaneous measured OH.
//! For a charge-closed inventory it also equals owned hydroxide coordinates
//! and literal hydroxides minus the admitted proton reservoirs. The independent
//! inventory bound below checks that identity before anything is consumed.
//!
//! Unknown buffers, bound/surface pools and mixed-solvent ionic states refuse.
//! This is a to-completion stoichiometric request, not kinetics, titration of an
//! arbitrary mixture or a measured activity/enthalpy model.
use crate::{delta::StateDelta, species, Moles, Phase, SpeciesId, Vessel};
const TRACE: f64 = 1e-12; // Existing named-reaction no-conversion threshold.
const RECONCILIATION: f64 = 1e-9; // Relative inventory arithmetic, not physical uncertainty.

#[derive(Clone, Copy)]
pub(crate) struct Applied {
    pub extent: f64,
    pub ionic: bool,
}
struct Capacity {
    moles: f64,
    ionic: bool,
}

fn capacity(v: &Vessel) -> Result<Capacity, String> {
    if !v.unresolved_materials.is_empty()
        || !v.material_objects.is_empty()
        || !v.surfaces.is_empty()
        || !v.electrodes.is_empty()
        || !v.exchanges.is_empty()
        || !v.adsorbed.is_empty()
        || !v.solid_solutions.is_empty()
        || v.pending_co2_transfer_mol != 0.0
    {
        return Err("strong-base capacity is not established for unresolved, bound or pending material reservoirs".into());
    }
    if !v.free_hydroxide.is_finite()
        || v.free_hydroxide < 0.0
        || !v.free_proton.is_finite()
        || v.free_proton < 0.0
    {
        return Err("invalid persisted acid/base measurement cache".into());
    }
    let (mut metal, mut anion, mut acetate, mut hydroxide, mut charge, mut charge_scale) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let mut ionic = false;
    for p in &v.contents {
        let n = p.moles.0;
        if !n.is_finite() || n < 0.0 {
            return Err("strong-base capacity requires finite nonnegative owned amounts".into());
        }
        if n == 0.0 {
            continue;
        }
        let key = p.species.0.as_str();
        let aqueous = matches!(p.phase, Phase::Aqueous);
        let fluid = matches!(p.phase, Phase::Liquid | Phase::Aqueous);
        // The reaction's own pool (solvent, ester, acetate, inert gases) is
        // admitted here; every strong acid/base carrier comes from the one
        // shared certified table, and weak buffers are refused by name.
        let pool = match key {
            "water" => Some(p.phase == Phase::Liquid),
            "ethyl_acetate" | "ethanol" | "CH3COOH" => Some(fluid),
            "NaOAc" => Some(!matches!(p.phase, Phase::Gas)),
            "CH3COO-" | "H+" => Some(aqueous),
            "N2" | "O2" | "Ar" | "He" | "Ne" => {
                Some(matches!(p.phase, Phase::Gas | Phase::Aqueous))
            }
            _ => None,
        };
        let carrier = crate::strong_reserve::carrier(key);
        let admitted = pool
            .or_else(|| carrier.map(|c| c.admit.admits(p.phase)))
            .unwrap_or(false);
        if !admitted {
            if let Some(class) = crate::strong_reserve::weak_buffer_class(key) {
                return Err(format!(
                    "owned {key} is a weak {class} buffer whose acid/base capacity is pH-dependent, not a certified strong-base reserve; no named hydrolysis applied"
                ));
            }
            return Err(format!(
                "strong-base capacity is not established for owned {key} in {:?}; no named hydrolysis applied",
                p.phase
            ));
        }
        let data = species::lookup(&p.species)
            .ok_or_else(|| format!("no molecular inventory definition for {key}"))?;
        let formula = crate::stoich::parse_formula(data.formula)
            .map_err(|_| format!("no molecular inventory formula for {key}"))?;
        charge += formula.charge * n;
        charge_scale += (formula.charge * n).abs();
        if let Some(c) = carrier.filter(|_| pool.is_none()) {
            metal += c.cation_eq * n;
            anion += c.anion_eq * n;
            hydroxide += c.hydroxide_eq * n;
            ionic |= c.ionic;
        }
        match key {
            "NaOAc" => {
                metal += n;
                acetate += n;
            }
            "CH3COOH" | "CH3COO-" => acetate += n,
            "H+" => ionic = true,
            _ => {}
        }
    }
    if ![metal, anion, acetate, hydroxide, charge, charge_scale]
        .iter()
        .all(|n| n.is_finite())
    {
        return Err("strong-base inventory arithmetic is non-finite".into());
    }
    // Preserve the existing explicitly requested literal-NaOH molecular template
    // when it owns all feed. An ionic/basis or KOH route instead needs an aqueous
    // medium; a stale solution cache cannot certify that medium.
    if ionic
        && (!v
            .contents
            .iter()
            .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > TRACE)
            || crate::nonaqueous::water_fraction_among_solvents(v)
                .is_some_and(|f| f < crate::nonaqueous::AQUEOUS_WATER_FRACTION_FLOOR))
    {
        return Err("ionic strong-base capacity requires a water-majority liquid; no named hydrolysis applied".into());
    }
    let tolerance = RECONCILIATION * (metal + anion + acetate + hydroxide + charge_scale);
    let excess = metal - anion - acetate;
    if charge.abs() > tolerance || excess > hydroxide + tolerance {
        return Err(
            "owned charge/proton inventory does not certify the proposed strong-base reserve"
                .into(),
        );
    }
    let moles = if excess <= tolerance {
        0.0
    } else {
        excess.min(hydroxide)
    };
    Ok(Capacity { moles, ionic })
}

/// Propose on a clone, retire actual owned hydroxide carriers, then commit atomically.
/// The generic analytical reserve never adds measured hydroxide a second time.
pub(crate) fn saponify(v: &mut Vessel) -> Result<Applied, String> {
    let ester = v.moles_of(&SpeciesId::new("ethyl_acetate")).0;
    if !ester.is_finite() || ester < 0.0 {
        return Err("invalid ester inventory".into());
    }
    if ester <= TRACE {
        return Ok(Applied {
            extent: 0.0,
            ionic: false,
        });
    }
    let cap = capacity(v)?;
    let requested = ester.min(cap.moles);
    if requested <= TRACE {
        return Ok(Applied {
            extent: 0.0,
            ionic: cap.ionic,
        });
    }
    let mut next = v.clone();
    let mut spent = 0.0;
    // Spend owned hydroxide carriers from the shared certified table, each
    // from its admitted phases only (never a gas portion of the same key).
    for key in [species::BASE_EQUIVALENTS, "OH-", "NaOH", "KOH", "Ba(OH)2"] {
        let c = crate::strong_reserve::carrier(key)
            .ok_or_else(|| format!("{key} is not a certified strong-base carrier"))?;
        for phase in [Phase::Aqueous, Phase::Liquid, Phase::Solid] {
            if !c.admit.admits(phase) {
                continue;
            }
            let needed = (requested - spent).max(0.0);
            let owned: f64 = next
                .contents
                .iter()
                .filter(|p| p.species.0 == key && p.phase == phase)
                .map(|p| p.moles.0)
                .sum();
            let take = needed.min(owned * c.hydroxide_eq);
            if take <= 0.0 {
                continue;
            }
            let molecules = next
                .withdraw_phase(&SpeciesId::new(key), Moles(take / c.hydroxide_eq), phase)
                .0;
            let equivalents = molecules * c.hydroxide_eq;
            if key == "NaOH" {
                // This sodium was in the withdrawn molecule, not already a spectator.
                next.deposit(SpeciesId::new("NaOAc"), Moles(molecules), Phase::Aqueous);
            } else {
                next.deposit(
                    SpeciesId::new("CH3COO-"),
                    Moles(equivalents),
                    Phase::Aqueous,
                );
                if let Some((cation, per)) = c.cation {
                    next.deposit(
                        SpeciesId::new(cation),
                        Moles(molecules * per),
                        Phase::Aqueous,
                    );
                }
            }
            spent += equivalents;
        }
    }
    if (requested - spent).abs() > RECONCILIATION * requested {
        return Err("owned hydroxide withdrawal cannot represent the requested extent".into());
    }
    // The admitted ester phases are liquid and dissolved; never a vapour.
    let ester_id = SpeciesId::new("ethyl_acetate");
    let mut taken = next
        .withdraw_phase(&ester_id, Moles(spent), Phase::Liquid)
        .0;
    taken += next
        .withdraw_phase(&ester_id, Moles(spent - taken), Phase::Aqueous)
        .0;
    if (taken - spent).abs() > RECONCILIATION * spent {
        return Err("owned ester withdrawal cannot represent the requested extent".into());
    }
    next.deposit(SpeciesId::new("ethanol"), Moles(spent), Phase::Liquid);
    let before = crate::ledger::ConservedLedger::from_vessel(v);
    let after = crate::ledger::ConservedLedger::from_vessel(&next);
    if (after.charge - before.charge).abs() > RECONCILIATION * spent {
        return Err("named hydrolysis failed its charge ledger".into());
    }
    next.solute_charge = crate::displacement::solute_charge(&next);
    next.solution = None;
    // Match the shared family applier: retire the hydroxide already consumed
    // before the aqueous tail establishes the next actual measurement. This
    // cache is never added to capacity and must not bill cleavage as acid/base
    // neutralisation or erase a retained strong-base excess.
    next.free_hydroxide = (next.free_hydroxide - spent).max(0.0);
    StateDelta::new("named-alkaline-capacity")
        .with_snapshot(v, &next)
        .commit_conserved(v, RECONCILIATION)
        .map_err(|e| format!("named hydrolysis refused its atomic inventory proposal: {e:?}"))?;
    Ok(Applied {
        extent: spent,
        ionic: cap.ionic,
    })
}
