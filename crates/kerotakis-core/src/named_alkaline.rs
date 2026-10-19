//! Conserved strong-base reserve for an explicitly bounded named reaction.
//!
//! In this closed Na/K + halide/nitrate + acetate pool, let M be total
//! alkali-metal equivalents, X strong-acid anion equivalents and A total
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
        let admitted = match key {
            "water" => p.phase == Phase::Liquid,
            "ethyl_acetate" | "ethanol" | "CH3COOH" | "HCl" | "HBr" | "HNO3" => fluid,
            "NaOH" | "KOH" | "NaOAc" | "NaCl" | "KCl" | "NaBr" | "KBr" | "NaNO3" | "KNO3" => {
                !matches!(p.phase, Phase::Gas)
            }
            "Na+"
            | "K+"
            | "Cl-"
            | "Br-"
            | "NO3-"
            | "CH3COO-"
            | "H+"
            | "OH-"
            | species::BASE_EQUIVALENTS => aqueous,
            "N2" | "O2" | "Ar" | "He" | "Ne" => matches!(p.phase, Phase::Gas | Phase::Aqueous),
            _ => false,
        };
        if !admitted {
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
        if matches!(
            key,
            "Na+"
                | "K+"
                | "NaOH"
                | "KOH"
                | "NaOAc"
                | "NaCl"
                | "KCl"
                | "NaBr"
                | "KBr"
                | "NaNO3"
                | "KNO3"
        ) {
            metal += n;
        }
        if matches!(
            key,
            "Cl-"
                | "Br-"
                | "NO3-"
                | "HCl"
                | "HBr"
                | "HNO3"
                | "NaCl"
                | "KCl"
                | "NaBr"
                | "KBr"
                | "NaNO3"
                | "KNO3"
        ) {
            anion += n;
        }
        if matches!(key, "CH3COOH" | "CH3COO-" | "NaOAc") {
            acetate += n;
        }
        if matches!(key, "NaOH" | "KOH" | "OH-" | species::BASE_EQUIVALENTS) {
            hydroxide += n;
        }
        ionic |= matches!(
            key,
            "Na+"
                | "K+"
                | "H+"
                | "OH-"
                | "Cl-"
                | "Br-"
                | "NO3-"
                | "CH3COO-"
                | "KOH"
                | species::BASE_EQUIVALENTS
        );
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
    for key in [species::BASE_EQUIVALENTS, "OH-", "NaOH", "KOH"] {
        let needed = (requested - spent).max(0.0);
        let take = needed.min(next.moles_of(&SpeciesId::new(key)).0);
        if take <= 0.0 {
            continue;
        }
        let actual = next.withdraw(&SpeciesId::new(key), Moles(take)).0;
        if key == "NaOH" {
            // This sodium was in the withdrawn molecule, not already a spectator.
            next.deposit(SpeciesId::new("NaOAc"), Moles(actual), Phase::Aqueous);
        } else {
            next.deposit(SpeciesId::new("CH3COO-"), Moles(actual), Phase::Aqueous);
            if key == "KOH" {
                next.deposit(SpeciesId::new("K+"), Moles(actual), Phase::Aqueous);
            }
        }
        spent += actual;
    }
    if (requested - spent).abs() > RECONCILIATION * requested {
        return Err("owned hydroxide withdrawal cannot represent the requested extent".into());
    }
    let taken = next
        .withdraw(&SpeciesId::new("ethyl_acetate"), Moles(spent))
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
