//! The aqueous heat balance, as one state-function inventory.
//!
//! Three separate heats used to be charged here, each from its own proxy:
//! neutralisation (from how much the solutes' net charge cancelled),
//! dissolution (from a registry enthalpy, per `Dissolved` event), and the
//! carbonate route (nothing at all, because the neutralisation enthalpy was
//! the wrong number for it and there was no other). They disagreed with each
//! other and with the beaker: baking soda and vinegar came out 0.17 K WARMER
//! in one pouring order and unmoved in the other, for a reaction a child can
//! feel go cold.
//!
//! The cause was never the enthalpies. It was that charge cancellation is
//! not the extent of the acid–carbonate reaction — a proton moves from
//! acetic acid to bicarbonate and the counter-ions stay put, so almost no
//! charge cancels while the whole reaction runs.
//!
//! Enthalpy is a state function, so the fix is not a better proxy but the
//! absence of one. Give every species an enthalpy relative to a single
//! basis, add up what the vessel holds before and after, and the difference
//! is the heat. Order independence is then not a property to test for and
//! hope: it is what a state function *is*.
//!
//! # The basis
//!
//! PHREEQC's master species, at the database's own standard state, are
//! zero. Everything else is measured from there:
//!
//! * an **aqueous species** carries the `SOLUTION_SPECIES` reaction
//!   enthalpy that defines it (`H+ + CO3-2 = HCO3-` → −14.6 kJ/mol);
//! * a **gas** carries the negative of its `PHASES` dissolution enthalpy,
//!   because that reaction is written dissolving *into* the basis;
//! * a **solid** carries the sum of its dissolution products' enthalpies
//!   less the registry's curated enthalpy of dissolution — the products
//!   matter, and getting that wrong is the subtlety this module exists to
//!   not repeat (see [`DISSOCIATION`]);
//! * **free acid and base** are carried by charge, not by portions: the
//!   readback never writes `H⁺` or `OH⁻` back (it would double-count
//!   water's own hydrogen and oxygen), so a solute charge of +q means q
//!   moles of unwritten hydroxide, and −q means that much free acid. `H⁺`
//!   is a master species and contributes nothing; `OH⁻` carries the
//!   database's own +55.81 kJ/mol.
//!
//! # What falls out
//!
//! `H⁺ + OH⁻ → H₂O` is not a special case in here. Neutralising a strong
//! acid with a strong base moves one mole of hydroxide (h = +55.81) to
//! water (h = 0), and the sum returns −55.81 kJ/mol without being asked —
//! the same figure `neutralisation_enthalpy` used to read off the engine.
//! Dissolving the alkali first is the other term of the same sum, so
//! `NaOH(s) + HCl → NaCl + H₂O` comes back as −100.31 kJ/mol, being −44.5
//! of dissolution and −55.81 of neutralisation, without either being
//! computed separately.
//!
//! The volcano comes back at +26.8 kJ/mol endothermic against a literature
//! +25 to +30, and by construction it returns the same number whichever way
//! round the beaker was filled.
//!
//! # Where it declines
//!
//! A species whose amount CHANGED and whose enthalpy this lab does not hold
//! stops the whole balance for that step, by name. A species that merely
//! sits there unpriced does not: it appears identically on both sides and
//! cancels, so an exotic spectator cannot silence the heat of a reaction it
//! took no part in.
//!
//! Declining is the deliberate behaviour and not a gap to be filled with
//! the nearest available number. Charging a plausible enthalpy for the
//! wrong substance is exactly how this went wrong before.

use kerotakis_core::{species, stoich, Phase, Portion, SpeciesId};
use std::collections::BTreeMap;

use crate::{dbindex, derived};

/// What each solid with a curated enthalpy of dissolution actually
/// dissolves INTO, in the database's own spelling.
///
/// This table is stoichiometry only — every enthalpy in the calculation
/// still comes from the database or the registry. It exists because
/// `h(solid) = Σ h(products) − ΔH_dissolution` needs the products, and
/// getting that wrong is quiet and plausible: taking `h(NaHCO₃) = −16.7`
/// (the enthalpy alone, as though the products were all masters) rather
/// than `(0 + −14.6) − 16.7 = −31.3` puts the volcano at +12.2 kJ/mol
/// instead of +26.8. Both look like reasonable answers. Only one of them
/// is within a mile of what the world measures.
///
/// A solid absent from this table can only use a fully validated routed
/// native mineral phase with explicit/index-derived reference reaction heat.
/// Otherwise its step declines by name. Splitting a formula to guess products
/// would have to guess for exactly the awkward cases.
pub const DISSOCIATION: &[(&str, &[(&str, f64)])] = &[
    ("NaCl", &[("Na+", 1.0), ("Cl-", 1.0)]),
    ("KCl", &[("K+", 1.0), ("Cl-", 1.0)]),
    ("CaCl2", &[("Ca+2", 1.0), ("Cl-", 2.0)]),
    ("NaOH", &[("Na+", 1.0), ("OH-", 1.0)]),
    ("KOH", &[("K+", 1.0), ("OH-", 1.0)]),
    ("Ca(OH)2", &[("Ca+2", 1.0), ("OH-", 2.0)]),
    // Also flagged: +16.7 matches neither the formation-enthalpy
    // difference (~+18.7) nor a tabulated ~+17.5. The volcano test below
    // asserts the LITERATURE BAND rather than a point precisely so a
    // correction here moves the answer without breaking the test — at
    // +18.7 the reaction comes out near +28.8, still inside 25–30.
    ("NaHCO3", &[("Na+", 1.0), ("HCO3-", 1.0)]),
    // Anhydrous standard dissolution reference, not a hydrated mineral.
    ("Na2CO3", &[("Na+", 2.0), ("CO3-2", 1.0)]),
    ("NaOAc", &[("Na+", 1.0), ("Acetate-", 1.0)]),
    ("Na2SO4", &[("Na+", 2.0), ("SO4-2", 1.0)]),
    ("CuSO4", &[("Cu+2", 1.0), ("SO4-2", 1.0)]),
    ("ZnSO4", &[("Zn+2", 1.0), ("SO4-2", 1.0)]),
    ("AgNO3", &[("Ag+", 1.0), ("NO3-", 1.0)]),
    ("AgCl", &[("Ag+", 1.0), ("Cl-", 1.0)]),
    ("Pb(NO3)2", &[("Pb+2", 1.0), ("NO3-", 2.0)]),
    // SUSPECT VALUE, kept rather than dropped. kerotakis-59's registry
    // agent could not reproduce the registry's +16.2 kJ/mol against a
    // commonly tabulated ~+43.6, and the gap is not a sign, a unit or a
    // hydrate. It is left in because the `Dissolved` loop this replaces
    // charges the same +16.2 today, so keeping it is the status quo and
    // dropping it would silently remove a heat permanganate vessels
    // already get. When the registry row is corrected this follows it
    // without a change here — the number is read, never copied.
    ("KMnO4", &[("K+", 1.0), ("MnO4-", 1.0)]),
    ("NH4Cl", &[("NH4+", 1.0), ("Cl-", 1.0)]),
    ("NH4NO3", &[("NH4+", 1.0), ("NO3-", 1.0)]),
    // Below here the registry names a substance that is ALREADY in
    // solution and that the database spells only as its ions. These carry
    // no enthalpy of dissolution — there is no dissolution left to do —
    // so they are priced as the sum of what they are. `NaOAc` appears in
    // both halves on purpose: as a solid it is these ions less its
    // dissolution enthalpy, and as the aqueous portion a curated reaction
    // deposits it is just the ions.
    ("HCl", &[("H+", 1.0), ("Cl-", 1.0)]),
    ("HBr", &[("H+", 1.0), ("Br-", 1.0)]),
    ("HNO3", &[("H+", 1.0), ("NO3-", 1.0)]),
    ("H2SO4", &[("H+", 2.0), ("SO4-2", 1.0)]),
    // An aqueous analytical equivalent only. This expresses the same
    // conserved master-basis inventory, not a measured heat of dissolving
    // solid KSCN (which has no curated dissolution enthalpy here).
    ("KSCN", &[("K+", 1.0), ("Thiocyanate-", 1.0)]),
];

/// Anhydrous oxides priced through their hydroxide by a REVIEWED hydration
/// enthalpy, read from one vendored database that writes both phases.
///
/// `(oxide, hydroxide, waters, reference database, oxide phase, hydroxide
/// phase)`. The hydration `oxide + n H2O -> hydroxide` is the difference of
/// the two dissolution reactions AS ONE DATABASE WRITES THEM, which is the
/// only way a difference of two `delta_h` lines is meaningful: both must
/// dissolve to exactly the same products with the same (implicit) proton
/// count, which `hydration_enthalpy` checks rather than assumes.
///
/// Why this table exists: wateq4f, the default route, has Portlandite but
/// no Lime. Without an owner for the oxide, quicklime in water either
/// declined or — through the old per-event fallback — charged only the
/// endothermic half (the precipitating portlandite) and cooled the beaker.
/// minteq.v4 writes `CaO + 2H+ = Ca+2 + H2O` at -193.91 kJ and
/// `Ca(OH)2 + 2H+ = Ca+2 + 2H2O` at -128.62 kJ, so the hydration is
/// -65.29 kJ/mol (standard tables: -635.1 - 285.8 -> -986.1 kJ/mol, i.e.
/// -65.2). Domain: the same 25 °C reference reaction enthalpies every
/// other term of this balance uses; no heat-capacity extrapolation.
pub const HYDRATION: &[(&str, &str, f64, &str, &str, &str)] =
    &[("CaO", "Ca(OH)2", 1.0, "minteq.v4", "Lime", "Portlandite")];

/// The reviewed hydration enthalpy, kJ per mole of oxide, for an entry of
/// [`HYDRATION`]; `None` unless both phases exist in the reference database
/// with finite `delta_h`, identical products, and compositions differing by
/// exactly the stated waters.
pub fn hydration_enthalpy(oxide: &str) -> Option<f64> {
    let (_, _, waters, db, oxide_phase, hydroxide_phase) =
        HYDRATION.iter().find(|row| row.0 == oxide)?;
    let idx = derived::index_for(db);
    let anhydrous = idx.phases.get(*oxide_phase)?;
    let hydrated = idx.phases.get(*hydroxide_phase)?;
    if anhydrous.is_gas
        || hydrated.is_gas
        || anhydrous.waters != 0.0
        || hydrated.waters != 0.0
        || anhydrous.products != hydrated.products
    {
        return None;
    }
    let mut expected = anhydrous.composition.clone();
    *expected.entry("H".to_string()).or_default() += 2.0 * waters;
    *expected.entry("O".to_string()).or_default() += waters;
    let same = expected.len() == hydrated.composition.len()
        && expected.iter().all(|(element, n)| {
            hydrated
                .composition
                .get(element)
                .is_some_and(|m| (m - n).abs() < 1e-12)
        });
    if !same {
        return None;
    }
    let h = anhydrous.delta_h_kj? - hydrated.delta_h_kj?;
    h.is_finite().then_some(h)
}

/// A species the balance cannot price, named so the refusal can say which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unpriced {
    pub species: String,
    pub why: &'static str,
}

/// The enthalpy of one aqueous species relative to the master basis.
fn aqueous_enthalpy(key: &str, db_tag: &str) -> Option<f64> {
    let idx = derived::index_for(db_tag);
    // The public ledger spells the conserved ligand physically; its
    // native pseudo-component master is the same reference basis. This
    // must not map metal complexes to the free ligand's zero enthalpy.
    let key = match key {
        "SCN-" => "Thiocyanate-",
        other => other,
    };
    if key == "water" || key == "H2O" {
        return Some(0.0);
    }
    if let Some(dh) = idx.species_delta_h_kj.get(key) {
        return Some(*dh);
    }
    // The registry and the database do not always spell the same substance
    // the same way — `CH3COOH` is `H(Acetate)` in minteq.v4.
    for (_, pairs) in derived::PROTONATION_SPLITS {
        for (db_name, registry_name) in *pairs {
            if *registry_name == key {
                if let Some(dh) = idx.species_delta_h_kj.get(*db_name) {
                    return Some(*dh);
                }
                if idx.species_element.contains_key(*db_name) {
                    return Some(0.0);
                }
            }
        }
    }
    // A master species is the basis.
    idx.species_element.contains_key(key).then_some(0.0)
}

/// A product in a NEW mineral phase fallback needs its own database
/// reference. `species_element` is insufficient: PHREEQC also records
/// Alkalinity pseudo-masters there, which are not zero-enthalpy basis ions.
/// An explicit species reaction heat may price a solid-phase product, but
/// only a real master or water licenses an analytical aqueous equivalent.
fn mineral_product_reference(product: &str, idx: &dbindex::DbIndex) -> Option<(f64, bool)> {
    let genuine_master = idx.masters.values().any(|master| master.species == product);
    let water = product == "H2O";
    let heat = if water {
        0.0
    } else if let Some(explicit) = idx.species_delta_h_kj.get(product) {
        *explicit
    } else if genuine_master {
        0.0
    } else {
        return None;
    };
    heat.is_finite().then_some((heat, water || genuine_master))
}

/// Reference enthalpies for a registry mineral on the exact phase selected
/// by the routed database. `products` is the analytical aqueous equivalent;
/// `solid` additionally reverses the phase's written dissolution reaction.
/// Neither number is a guessed formation heat or a temperature-dependent Cp.
fn mineral_reference(key: &str, db_tag: &str) -> Result<(Option<f64>, f64), Unpriced> {
    let decline = |why: &'static str| Unpriced {
        species: key.to_string(),
        why,
    };
    let Some(derived::DerivedRole::Mineral {
        phase: role_phase, ..
    }) = derived::role(key)
    else {
        return Err(decline("no exact derived mineral owner for this solid"));
    };
    let Some(routed) = derived::phase_in_db(role_phase, db_tag) else {
        return Err(decline("the routed database has no phase for this mineral"));
    };
    if routed != role_phase.as_str() {
        return Err(decline(
            "the routed phase substitutes a different mineral polymorph",
        ));
    }
    let idx = derived::index_for(db_tag);
    let phase = idx
        .phases
        .get(routed)
        .ok_or_else(|| decline("the routed database does not define the selected mineral phase"))?;
    let registry = species::lookup(&SpeciesId::new(key))
        .filter(|row| row.standard_phase == Phase::Solid)
        .ok_or_else(|| decline("the selected mineral has no solid registry identity"))?;
    let (base_formula, hydrate_waters) = dbindex::split_hydrate(registry.formula);
    // The index currently omits written H2O products from phase.products.
    // Until that water has an explicit stoichiometric owner, a hydrate's
    // dissolution heat cannot be balanced by the anhydrous-product check.
    if hydrate_waters != 0.0 || phase.waters != 0.0 {
        return Err(decline(
            "hydration water is not balanced by this mineral reference path",
        ));
    }
    let registry_atoms = dbindex::parse_formula(&base_formula)
        .ok_or_else(|| decline("the registry mineral formula cannot be balanced"))?;
    if phase.is_gas
        || !phase.waters.is_finite()
        || hydrate_waters != phase.waters
        || registry_atoms != phase.composition
        || derived::phase_by_name(routed).is_none_or(|owner| owner.species != key)
        || derived::registry_solid_for(&phase.composition, phase.waters) != Some(key)
    {
        return Err(decline(
            "the selected phase does not exactly match the registry mineral and hydrate",
        ));
    }
    let solid_formula = stoich::parse_formula(&base_formula)
        .map_err(|_| decline("the registry mineral has no physical formula"))?;
    if solid_formula.charge != 0.0 || phase.products.is_empty() {
        return Err(decline(
            "the selected mineral has no neutral, complete dissolution reaction",
        ));
    }
    let dissolution = phase
        .delta_h_kj
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            decline("the selected routed mineral states no finite reference dissolution enthalpy")
        })?;
    let mut product_atoms = BTreeMap::<String, f64>::new();
    let mut product_charge = 0.0;
    let mut product_enthalpy = 0.0;
    let mut analytical_master_basis = true;
    for (product, coefficient) in &phase.products {
        if !coefficient.is_finite() || *coefficient <= 0.0 {
            return Err(decline(
                "the selected mineral has an invalid dissolution coefficient",
            ));
        }
        let data = species::lookup(&SpeciesId::new(product.as_str()))
            .ok_or_else(|| decline("a dissolution product has no registry formula"))?;
        let formula = stoich::parse_formula(data.formula)
            .map_err(|_| decline("a dissolution product has no physical formula"))?;
        for (element, count) in formula.counts {
            *product_atoms.entry(element).or_default() += coefficient * count;
        }
        product_charge += coefficient * formula.charge;
        let (h, genuine_master_or_water) =
            mineral_product_reference(product, idx).ok_or_else(|| {
                decline("a dissolution product lacks a genuine master or explicit routed enthalpy")
            })?;
        product_enthalpy += coefficient * h;
        // A registry analytical amount denotes the dissociated component,
        // never a native ion pair that happens to share its name. Only a
        // complete master-species/water basis justifies that equivalence.
        analytical_master_basis &= genuine_master_or_water;
    }
    if product_charge.abs() > 1e-12
        || product_atoms
            .keys()
            .any(|key| !registry_atoms.contains_key(key))
        || registry_atoms.iter().any(|(element, amount)| {
            (product_atoms.get(element).copied().unwrap_or(0.0) - amount).abs() > 1e-12
        })
        || !product_enthalpy.is_finite()
    {
        return Err(decline(
            "the routed dissolution products do not balance the mineral formula",
        ));
    }
    let solid = product_enthalpy - dissolution;
    if !solid.is_finite() {
        return Err(decline(
            "the routed mineral reference enthalpy is nonfinite",
        ));
    }
    Ok((analytical_master_basis.then_some(product_enthalpy), solid))
}

/// The enthalpy of one portion's species, kJ/mol, relative to the basis.
pub fn species_enthalpy(key: &str, phase: Phase, db_tag: &str) -> Result<f64, Unpriced> {
    let idx = derived::index_for(db_tag);
    match phase {
        // A gas is priced by where it goes, less what the trip costs —
        // the same rule as a solid, and for the same reason.
        //
        // `delta_h` is the enthalpy of the dissolution reaction AS THE
        // DATABASE WRITES IT, and the databases do not write the same
        // reaction. minteq.v4 dissolves CO2(g) to `2 H+ + CO3-2`, which are
        // master species and cost nothing, so the gas is just the reaction
        // reversed. wateq4f dissolves it to an aqueous `CO2` that carries
        // about -24 kJ/mol of its own, and ignoring that put the same gas
        // leaving the same beaker at +19.98 kJ/mol on one route and -4.06
        // on the other. The volcano then cooled by 1.70 K poured one way
        // and 1.28 K poured the other, and the 1.28 was the correct one.
        Phase::Gas => {
            let phase = idx.phases.get(&format!("{key}(g)")).ok_or(Unpriced {
                species: key.to_string(),
                why: "the routed database defines no gas phase of this name",
            })?;
            let dissolution = phase.delta_h_kj.ok_or(Unpriced {
                species: key.to_string(),
                why: "the routed database states no enthalpy for this gas dissolving",
            })?;
            let mut sum = 0.0;
            for (product, n) in &phase.products {
                let h = aqueous_enthalpy(product, db_tag).ok_or(Unpriced {
                    species: product.clone(),
                    why: "a dissolution product the routed database does not define",
                })?;
                sum += h * n;
            }
            Ok(sum - dissolution)
        }
        // A solid is where its ions go, LESS what the trip costs.
        //
        // The enthalpy of dissolution belongs in this sum and not beside
        // it. It used to be charged separately, per `Dissolved` event, and
        // that survived as the last second path in the heat balance — with
        // exactly the failure a second path always has. Pouring baking soda
        // into water dissolves it through the aqueous tail, the event
        // fires, +16.7 kJ/mol is charged. Pouring it into vinegar lets the
        // curated row consume the solid outright: no dissolution event, no
        // dissolution heat, and the same reaction came out cooling by 0.49 K
        // one way round and 1.70 K the other.
        //
        // Here it is one term of one sum, so both routes begin from the
        // same solid in the same state and reach +26.8 kJ/mol either way.
        //
        // The subtlety, and it cost a factor of two before it was caught:
        // the products are NOT all master species. `NaHCO3 -> Na+ + HCO3-`
        // and the bicarbonate carries -14.6 of its own, so the solid is
        // `(0 + -14.6) - 16.7 = -31.3`, not `-16.7`. Both look reasonable.
        // Only one is within a mile of what the world measures.
        Phase::Solid => {
            let products = DISSOCIATION
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, p)| *p);
            if let Some((_, hydroxide, waters, ..)) = HYDRATION.iter().find(|row| row.0 == key) {
                // h(oxide) + n h(water) + dH_hydration = h(hydroxide).
                let hydration = hydration_enthalpy(key).ok_or(Unpriced {
                    species: key.to_string(),
                    why: "the reviewed hydration reference does not validate",
                })?;
                let h_hydroxide = species_enthalpy(hydroxide, Phase::Solid, db_tag)?;
                let h_water = species_enthalpy("water", Phase::Liquid, db_tag)?;
                return Ok(h_hydroxide - waters * h_water - hydration);
            }
            let Some(products) = products else {
                return mineral_reference(key, db_tag).map(|(_, solid)| solid);
            };
            let dissolution = species::lookup(&SpeciesId::new(key))
                .and_then(|d| d.dissolution_enthalpy_kj)
                .ok_or(Unpriced {
                    species: key.to_string(),
                    why: "the registry holds no enthalpy of dissolution for this solid",
                })?;
            let mut sum = 0.0;
            for (product, n) in products {
                let h = aqueous_enthalpy(product, db_tag).ok_or(Unpriced {
                    species: (*product).to_string(),
                    why: "a dissociation product the routed database does not define",
                })?;
                sum += h * n;
            }
            Ok(sum - dissolution)
        }
        // Water is the solvent and the basis; any other miscible liquid is
        // priced as what it becomes in solution, which leaves its enthalpy
        // of MIXING to `hmix`, where it already lives, rather than counting
        // it twice.
        Phase::Liquid | Phase::Aqueous => {
            // `CaCO3` here is an analytical amount of the mineral already
            // dissolved. A native ion pair with the same spelling is not
            // automatically that amount, and no solid dissolution remains
            // to be charged. Curated mappings keep their existing priority.
            if phase == Phase::Aqueous
                && !DISSOCIATION.iter().any(|(name, _)| *name == key)
                && !idx.species_delta_h_kj.contains_key(key)
                && matches!(
                    derived::role(key),
                    Some(derived::DerivedRole::Mineral { .. })
                )
            {
                return mineral_reference(key, db_tag).and_then(|(products, _)| {
                    products.ok_or(Unpriced {
                        species: key.to_string(),
                        why: "analytical mineral products are not a complete master-species basis",
                    })
                });
            }
            if let Some(h) = aqueous_enthalpy(key, db_tag) {
                return Ok(h);
            }
            // Already dissolved, and the database spells it only as its
            // ions: it is the sum of them, with no dissolution term
            // because there is no dissolution left to do.
            let products = DISSOCIATION
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, p)| *p)
                .ok_or(Unpriced {
                    species: key.to_string(),
                    why: "the routed database does not define this species in solution",
                })?;
            let mut sum = 0.0;
            for (product, n) in products {
                let h = aqueous_enthalpy(product, db_tag).ok_or(Unpriced {
                    species: (*product).to_string(),
                    why: "a dissociation product the routed database does not define",
                })?;
                sum += h * n;
            }
            Ok(sum)
        }
    }
}

/// `Phase` is not `Ord`, and this map wants a stable key rather than a
/// derive on a core type that nothing else asks for.
fn phase_key(phase: Phase) -> u8 {
    match phase {
        Phase::Solid => 0,
        Phase::Liquid => 1,
        Phase::Aqueous => 2,
        Phase::Gas => 3,
    }
}

fn phase_of(key: u8) -> Phase {
    match key {
        0 => Phase::Solid,
        1 => Phase::Liquid,
        2 => Phase::Aqueous,
        _ => Phase::Gas,
    }
}

/// Everything the vessel holds, as moles keyed by (species, phase).
fn tally(contents: &[Portion]) -> BTreeMap<(String, u8), f64> {
    let mut t = BTreeMap::new();
    for p in contents {
        // Analytical H/O equivalents are not free-ion activities. Hydroxide
        // enthalpy is supplied separately from the measured species state.
        if p.phase == Phase::Aqueous && species::is_acid_base_basis(&p.species.0) {
            continue;
        }
        *t.entry((p.species.0.clone(), phase_key(p.phase)))
            .or_insert(0.0) += p.moles.0;
    }
    t
}

/// Below this, a change is not a reaction — it is the solver's last digit.
const NEGLIGIBLE: f64 = 1e-12;

/// The heat released, in JOULES, by everything that changed between two
/// states of one vessel. Positive warms the beaker.
///
/// `gas_out` is what left the liquid during this step and is therefore no
/// longer in `after`; it is priced as gas and counted on the after side.
/// `oh_before`/`oh_after` are MOLES of free hydroxide, read from PHREEQC's
/// own species distribution. They are not inferred from the solutes' net
/// charge, and that distinction is load-bearing: a positive solute charge
/// means free base only when the vessel's inputs were charge-balanced, and
/// a beaker handed a bare cation is not. Reading 6.4 mol of `Ag+` as 6.4
/// mol of hydroxide charged 355 kJ of imaginary neutralisation and drove a
/// vessel to MINUS 198 K. Charge correlates with free base; it is not free
/// base, and the engine already reports the real thing.
///
/// Only species whose amount actually MOVED are priced, so an unpriceable
/// spectator costs nothing. If something that moved cannot be priced, the
/// whole step declines and says which species stopped it.
pub fn heat_released_j(
    before: &[Portion],
    oh_before: f64,
    after: &[Portion],
    oh_after: f64,
    gas_out: &[(String, f64)],
    db_tag: &str,
) -> Result<f64, Unpriced> {
    let mut delta = tally(after);
    for (k, n) in tally(before) {
        *delta.entry(k).or_insert(0.0) -= n;
    }
    for (species, moles) in gas_out {
        *delta
            .entry((species.clone(), phase_key(Phase::Gas)))
            .or_insert(0.0) += moles;
    }
    // The unwritten hydroxide. Free acid is H+, a master species, and
    // contributes nothing, so only the base side appears here.
    let d_oh = oh_after - oh_before;
    // Hydroxide has to come from somewhere nameable.
    //
    // A beaker handed bare `Na+` — 3.9 mol of cation with no anion — is
    // charge-balanced by the engine with 3.9 mol of hydroxide, and that
    // hydroxide is real as far as the speciation is concerned. But nothing
    // in the ledger supplied it: the sodium is priced as the master
    // species it is, and the hydroxide then appears out of the basis
    // carrying +55.81 kJ/mol of enthalpy each. 3.9 mol of that is 218 kJ
    // of invented cooling, and it drove a vessel to MINUS 27 K.
    //
    // A real alkali does not do this, because it arrives as a portion that
    // is priced with its hydroxide in it — `NaOH(s)` is Na+ plus OH-, so
    // the hydroxide appearing on one side is paid for by the solid
    // disappearing on the other, and the two cancel. So the rule is simply
    // that: if free hydroxide GREW, something that contains hydroxide must
    // have been consumed to make it. Nothing did, and the vessel was never
    // charge-coherent to begin with, so no heat may be drawn from it.
    //
    // Hydroxide SHRINKING needs no such licence — that is neutralisation,
    // and what consumed it is the acid, which is priced already.
    if d_oh.abs() > NEGLIGIBLE {
        let h = aqueous_enthalpy("OH-", db_tag).ok_or(Unpriced {
            species: "OH-".to_string(),
            why: "the routed database does not define hydroxide",
        })?;
        *delta
            .entry(("__free_hydroxide".to_string(), phase_key(Phase::Aqueous)))
            .or_insert(0.0) += d_oh;
        // Priced directly rather than through species_enthalpy, since the
        // key is ours and not a registry name.
        let _ = h;
    }

    // How much hydroxide has to appear before something must account for
    // it. The two cases this separates are six orders of magnitude apart,
    // so the line is not delicate:
    //
    // * a bicarbonate solution sits at pH 8.3 and makes about 9 µmol of
    //   hydroxide by hydrolysing — genuinely supplied, by a weak base this
    //   table does not list as a hydroxide source, and worth 0.5 J;
    // * a beaker handed 3.9 mol of bare `Na+` is charge-balanced by the
    //   engine with 3.9 MOL of hydroxide, worth 218 kJ, and reached minus
    //   27 K before this check existed.
    //
    // A millimole is comfortably above every hydrolysis this bench sees
    // and a factor of ~4000 below the pathology. What slips under it is
    // worth at most 56 J, which is 0.13 K in a 100 g beaker and is charged
    // rather than refused — declining a real reaction's heat to avoid a
    // rounding error would be the worse trade.
    const HYDROXIDE_FROM_A_REAGENT: f64 = 1e-3;
    if d_oh > HYDROXIDE_FROM_A_REAGENT {
        let carries_hydroxide = |key: &str| {
            DISSOCIATION
                .iter()
                .any(|(k, products)| *k == key && products.iter().any(|(p, _)| *p == "OH-"))
        };
        let supplied = delta.iter().any(|((key, _), moved)| {
            *moved < -NEGLIGIBLE
                && (carries_hydroxide(key)
                    || HYDRATION
                        .iter()
                        .any(|row| row.0 == key && carries_hydroxide(row.1)))
        });
        if !supplied {
            return Err(Unpriced {
                species: "OH-".to_string(),
                why: "free hydroxide appeared with nothing in the vessel to supply it — \
                      the charge was balanced by the engine, not by a reagent, so this \
                      beaker was never charge-coherent",
            });
        }
    }

    let mut d_h_kj = 0.0;
    for ((key, phase), moved) in &delta {
        if moved.abs() <= NEGLIGIBLE {
            continue;
        }
        let h = if key == "__free_hydroxide" {
            aqueous_enthalpy("OH-", db_tag).ok_or(Unpriced {
                species: "OH-".to_string(),
                why: "the routed database does not define hydroxide",
            })?
        } else {
            species_enthalpy(key, phase_of(*phase), db_tag)?
        };
        d_h_kj += h * moved;
        if std::env::var("KERO_BAL").is_ok() {
            eprintln!(
                "    [d] {key:<12} ph={phase} moved={moved:+.9} h={h:+.3} -> {:+.4} kJ",
                h * moved
            );
        }
    }
    // Heat released is the negative of the enthalpy the contents gained.
    Ok(-d_h_kj * 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::{Moles, SpeciesId};

    fn portion(key: &str, moles: f64, phase: Phase) -> Portion {
        Portion {
            species: SpeciesId::new(key),
            moles: Moles(moles),
            phase,
        }
    }

    const DB: &str = "minteq.v4";

    #[test]
    fn aqueous_thiocyanate_relabelling_preserves_known_heat_not_unknown_properties() {
        for db in ["wateq4f", "minteq.v4"] {
            let neutralisation =
                heat_released_j(&[], 0.01, &[], 0.0, &[], db).expect("known hydroxide heat");
            assert!(neutralisation > 0.0);
            for moles in [1e-6, 0.001, 0.1] {
                let before = [portion("KSCN", moles, Phase::Aqueous)];
                let after = [
                    portion("K+", moles, Phase::Aqueous),
                    portion("SCN-", moles, Phase::Aqueous),
                ];
                let relabel = heat_released_j(&before, 0.0, &after, 0.0, &[], db)
                    .expect("same aqueous master-basis inventory");
                assert!(relabel.abs() < 1e-12, "{db}: {relabel}");
                let combined = heat_released_j(&before, 0.01, &after, 0.0, &[], db)
                    .expect("analytical relabelling cannot suppress known reaction heat");
                assert!((combined - neutralisation).abs() < 1e-10);
            }
            assert!(species_enthalpy("KSCN", Phase::Solid, db).is_err());
            for complex in [
                "Fe(SCN)+2",
                "Fe(SCN)2+",
                "Fe(Thiocyanate)+2",
                "Fe(Thiocyanate)2+",
                "Cu(NH3)4+2",
            ] {
                assert!(species_enthalpy(complex, Phase::Aqueous, db).is_err());
            }
        }
        assert!(species_enthalpy("SCN-", Phase::Aqueous, "pitzer").is_err());
        assert!(species_enthalpy("KSCN", Phase::Aqueous, "pitzer").is_err());
    }

    /// A solid is where its ions go, less what the trip costs. Both terms
    /// belong in this one number: charging the dissolution separately, per
    /// event, was the last second path in the balance and it made the
    /// answer depend on which solver happened to consume the solid.
    #[test]
    fn a_solids_enthalpy_is_its_ions_less_the_cost_of_getting_there() {
        // NaHCO3 -> Na+ (0) + HCO3- (-14.6), less +16.7 of dissolution.
        // Taking the dissolution alone gives -16.7 and puts the volcano at
        // +12.2 kJ/mol instead of +26.8. Both look reasonable; one is not.
        let h = species_enthalpy("NaHCO3", Phase::Solid, DB).expect("priced");
        assert!((h - (-31.3)).abs() < 1e-9, "{h}");
        // NaOH -> Na+ (0) + OH- (+55.81), less -44.5 of dissolution. Both
        // halves of `NaOH + HCl` are in this one number.
        let h = species_enthalpy("NaOH", Phase::Solid, DB).expect("priced");
        assert!((h - 100.31).abs() < 1e-9, "{h}");
        // NaCl -> two master species, so it is only its dissolution.
        let h = species_enthalpy("NaCl", Phase::Solid, DB).expect("priced");
        assert!((h - (-3.88)).abs() < 1e-9, "{h}");
    }

    /// The anhydrous reagent has its own standard reference, independent of
    /// any hydrated carbonate phase in the native databases. Both intermediate
    /// dissolution and direct precipitation must use that same reference.
    #[test]
    fn sodium_carbonate_reference_closes_dissolution_and_calcium_cycle_on_each_basis() {
        let data = species::lookup_key("Na2CO3").expect("anhydrous reagent");
        assert_eq!(data.dissolution_enthalpy_kj, Some(-26.656));
        let formula = kerotakis_core::stoich::parse_formula(data.formula).unwrap();
        let products = DISSOCIATION
            .iter()
            .find(|(key, _)| *key == "Na2CO3")
            .unwrap()
            .1;
        let mut dissolved = kerotakis_core::stoich::Formula::default();
        for (key, n) in products {
            let ion = species::lookup_key(key).expect("registered dissociation product");
            let f = kerotakis_core::stoich::parse_formula(ion.formula).unwrap();
            for (element, count) in f.counts {
                *dissolved.counts.entry(element).or_default() += count * n;
            }
            dissolved.charge += f.charge * n;
        }
        assert_eq!(
            formula, dissolved,
            "dissociation preserves atoms and charge"
        );

        for db in ["wateq4f", "minteq.v4", "pitzer"] {
            let before = [portion("Na2CO3", 0.01, Phase::Solid)];
            let ions = [
                portion("Na+", 0.02, Phase::Aqueous),
                portion("CO3-2", 0.01, Phase::Aqueous),
            ];
            let q = heat_released_j(&before, 0.0, &ions, 0.0, &[], db)
                .expect("anhydrous dissolution is priced");
            assert!((q - 266.56).abs() < 1e-9, "{db}: {q} J");
            let reverse = heat_released_j(&ions, 0.0, &before, 0.0, &[], db).unwrap();
            assert!((q + reverse).abs() < 1e-9, "{db}: reverse sign");

            let salts = [
                portion("Na2CO3", 0.01, Phase::Solid),
                portion("CaCl2", 0.01, Phase::Solid),
            ];
            let intermediate = [
                portion("Na2CO3", 0.01, Phase::Solid),
                portion("Ca+2", 0.01, Phase::Aqueous),
                portion("Cl-", 0.02, Phase::Aqueous),
            ];
            let precipitated = [
                portion("CaCO3", 0.01, Phase::Solid),
                portion("Na+", 0.02, Phase::Aqueous),
                portion("Cl-", 0.02, Phase::Aqueous),
            ];
            let direct = heat_released_j(&salts, 0.0, &precipitated, 0.0, &[], db)
                .expect("complete salt and mineral heat");
            let first = heat_released_j(&salts, 0.0, &intermediate, 0.0, &[], db).unwrap();
            let second = heat_released_j(&intermediate, 0.0, &precipitated, 0.0, &[], db).unwrap();
            assert!((direct - first - second).abs() < 1e-9, "{db}: Hess cycle");
            assert!(species_enthalpy("unregistered_carbonate", Phase::Solid, db).is_err());
            assert!(species_enthalpy("KSCN", Phase::Solid, db).is_err());
        }
    }

    /// The native phase reaction, not a nearest curated salt or a registry
    /// heat-capacity guess, supplies the solid reference. The pre-existing
    /// exact native aqueous-complex heat keeps priority where the database
    /// gives one; only its absence admits the analytical master-ion fallback.
    /// This preserves a legacy reference, not a new molecular-state claim.
    #[test]
    fn calcite_reference_prices_solid_and_analytical_aqueous_on_each_routed_basis() {
        for (db, dissolution, native_aqueous) in [
            ("wateq4f", -9.610_648, 3.545 * 4.184),
            ("minteq.v4", -8.0, 16.0),
        ] {
            let index = derived::index_for(db);
            let phase = &index.phases["Calcite"];
            assert_eq!(derived::phase_in_db("Calcite", db), Some("Calcite"));
            assert_eq!(phase.products.len(), 2);
            assert!((phase.delta_h_kj.expect("phase delta H") - dissolution).abs() < 1e-9);
            assert_eq!(
                index.species_delta_h_kj.get("CaCO3").copied(),
                Some(native_aqueous),
                "{db}: explicit aqueous complex reference"
            );
            let solid = species_enthalpy("CaCO3", Phase::Solid, db).expect("Calcite solid");
            let native =
                species_enthalpy("CaCO3", Phase::Aqueous, db).expect("dissolved CaCO3 amount");
            assert!((solid + dissolution).abs() < 1e-9, "{db}: {solid}");
            assert!((native - native_aqueous).abs() < 1e-12, "{db}: {native}");
        }
        // Pitzer's inline -delta_h is attached to the log_k line; the
        // source parser instead takes the slope of its separate -analytic
        // expression. Differentiate the literal source coefficients by a
        // symmetric finite difference, independently of dbindex's formula.
        let log_k = |t: f64| -67.278 - 5.0767e-2 * t + 29.8956 * t.log10();
        let t = 298.15_f64;
        let dt = 1e-3;
        let slope = (log_k(t + dt) - log_k(t - dt)) / (2.0 * dt);
        let independently_derived =
            8.314_462_618_153_24 * t * t * std::f64::consts::LN_10 * slope / 1000.0;
        assert!((independently_derived + 12.287_557_49).abs() < 1e-5);
        assert!((independently_derived + 12.300_96).abs() > 0.01);
        let indexed = derived::index_for("pitzer").phases["Calcite"]
            .delta_h_kj
            .expect("Pitzer analytic phase reference");
        assert!((indexed - independently_derived).abs() < 1e-6);
        let solid = species_enthalpy("CaCO3", Phase::Solid, "pitzer").expect("Pitzer Calcite");
        assert!((solid + indexed).abs() < 1e-9);
        assert!(
            species_enthalpy("CaCO3", Phase::Aqueous, "pitzer")
                .expect("Pitzer analytical CaCO3")
                .abs()
                < 1e-12
        );
        // Existing curated values and honest unknowns retain their paths.
        assert!((species_enthalpy("NaCl", Phase::Solid, DB).unwrap() + 3.88).abs() < 1e-9);
        assert!(species_enthalpy("KSCN", Phase::Solid, DB).is_err());
        assert!(species_enthalpy("CaCO3", Phase::Solid, "unknown-db").is_err());
    }

    #[test]
    fn pseudo_master_alias_is_not_a_new_mineral_reference_and_hydrates_decline() {
        let mut index = dbindex::DbIndex::default();
        // The parser records Alkalinity's HCO3- alias here even though it
        // does not put it in masters. That mapping cannot invent zero heat.
        index.species_element.insert("HCO3-".into(), "C".into());
        assert_eq!(mineral_product_reference("HCO3-", &index), None);
        index.species_delta_h_kj.insert("HCO3-".into(), -14.6);
        assert_eq!(
            mineral_product_reference("HCO3-", &index),
            Some((-14.6, false))
        );
        index.masters.insert(
            "C".into(),
            dbindex::MasterSpecies {
                element: "C".into(),
                species: "HCO3-".into(),
            },
        );
        assert_eq!(
            mineral_product_reference("HCO3-", &index),
            Some((-14.6, true))
        );
        assert_eq!(mineral_product_reference("H2O", &index), Some((0.0, true)));
        assert!(mineral_product_reference("unknown", &index).is_none());
        let gypsum =
            mineral_reference("gypsum", "wateq4f").expect_err("hydrate water is not priced");
        assert_eq!(
            gypsum.why,
            "hydration water is not balanced by this mineral reference path"
        );
    }

    /// The strong-acid/strong-base heat is not a special case in here; it
    /// is one hydroxide going to water, and the sum says so.
    #[test]
    fn neutralisation_falls_out_of_the_basis() {
        // A beaker holding 0.1 mol of free base, neutralised to nothing.
        let before = [portion("Na+", 0.1, Phase::Aqueous)];
        let after = [
            portion("Na+", 0.1, Phase::Aqueous),
            portion("Cl-", 0.1, Phase::Aqueous),
        ];
        let q = heat_released_j(&before, 0.1, &after, 0.0, &[], DB).expect("priced");
        // 0.1 mol x -55.81 kJ/mol of enthalpy lost = that much heat out.
        assert!(
            (q - 5581.0).abs() < 1e-6,
            "expected the database's own 55.81 kJ/mol, got {} J",
            q
        );
    }

    /// The reaction this module was written for, and the number the world
    /// already knows: +25 to +30 kJ/mol endothermic.
    #[test]
    fn the_volcano_is_endothermic_and_the_right_size() {
        let before = [
            portion("NaHCO3", 0.02, Phase::Solid),
            portion("CH3COOH", 0.02, Phase::Liquid),
        ];
        let after = [
            portion("Na+", 0.02, Phase::Aqueous),
            portion("CH3COO-", 0.02, Phase::Aqueous),
        ];
        let q = heat_released_j(&before, 0.0, &after, 0.0, &[("CO2".into(), 0.02)], DB)
            .expect("priced");
        let kj_per_mol = -q / 1000.0 / 0.02;
        // The whole reaction now, dissolution included, against the
        // literature's +25 to +30 for baking soda and vinegar. Pinned to
        // the BAND rather than to our own figure, so a correction to the
        // registry's enthalpy of dissolution (its +16.7 is flagged as not
        // reproducible) moves the answer without breaking the test.
        assert!(
            (25.0..=30.0).contains(&kj_per_mol),
            "expected +25..30 kJ/mol endothermic, got {kj_per_mol}"
        );
    }

    /// A spectator nobody can price must not silence a reaction it took no
    /// part in — it is on both sides and cancels.
    #[test]
    fn an_unpriceable_spectator_that_does_not_move_is_harmless() {
        let before = [
            portion("Na+", 0.1, Phase::Aqueous),
            portion("glitter", 1.0, Phase::Solid),
        ];
        let after = [
            portion("Na+", 0.1, Phase::Aqueous),
            portion("Cl-", 0.1, Phase::Aqueous),
            portion("glitter", 1.0, Phase::Solid),
        ];
        assert!(heat_released_j(&before, 0.1, &after, 0.0, &[], DB).is_ok());
    }

    /// But one that MOVES stops the balance, by name, rather than being
    /// charged the nearest number that happens to be to hand.
    #[test]
    fn something_unpriceable_that_moves_declines_by_name() {
        let before = [portion("glitter", 1.0, Phase::Solid)];
        let after = [portion("glitter", 0.5, Phase::Solid)];
        let err = heat_released_j(&before, 0.0, &after, 0.0, &[], DB).expect_err("declines");
        assert_eq!(err.species, "glitter");
    }

    /// The basis must not be able to collapse to all-zeros.
    ///
    /// Every fallback in `aqueous_enthalpy` ends at "it is a master
    /// species, so it is zero" — which is correct, and is also what a
    /// BROKEN PARSE looks like. If `species_delta_h_kj` came back empty,
    /// HCO3- would not be found as a species, would be found as
    /// Alkalinity's master, and would price at 0.0 instead of -14.6. Every
    /// test above would still pass except the two that pin numbers, and
    /// every heat in the bench would quietly go to zero without a single
    /// refusal, because zero is a number and the balance would sum it
    /// happily.
    ///
    /// So assert the corpus is really there before trusting anything drawn
    /// from it: a population of enthalpies, and non-zero ones at that.
    #[test]
    fn the_basis_is_populated_and_not_silently_zero() {
        let idx = derived::index_for(DB);
        assert!(
            idx.species_delta_h_kj.len() > 100,
            "only {} species enthalpies parsed — suspect the parser, not the file",
            idx.species_delta_h_kj.len()
        );
        let nonzero = idx
            .species_delta_h_kj
            .values()
            .filter(|v| **v != 0.0)
            .count();
        assert!(nonzero > 100, "only {nonzero} of them are non-zero");
        let phases = idx
            .phases
            .values()
            .filter(|p| p.delta_h_kj.is_some_and(|d| d != 0.0))
            .count();
        assert!(phases > 50, "only {phases} phase enthalpies");
        // And the three the carbonate and neutralisation answers actually
        // rest on, by name and value — a populated map is not the same as
        // a map containing what this module reads out of it.
        assert!((aqueous_enthalpy("HCO3-", DB).unwrap() + 14.6).abs() < 1e-9);
        assert!((aqueous_enthalpy("OH-", DB).unwrap() - 55.81).abs() < 1e-9);
        assert!(
            (species_enthalpy("CO2", Phase::Gas, DB).unwrap() + 4.06).abs() < 1e-9,
            "the CO2(g) phase enthalpy"
        );
    }

    /// Every row prices, and a solid prices as its ions LESS its enthalpy
    /// of dissolution — both terms, once each. If the dissolution ever
    /// leaves this number it becomes an event-driven second path, and the
    /// same reaction then costs different amounts depending on which
    /// solver consumed the solid.
    ///
    /// Compared against the products directly rather than against the
    /// same key priced as aqueous, because those are not always the same
    /// substance: minteq.v4 defines an aqueous ION PAIR called `NaHCO3`
    /// (`Na+ + H+ + CO3-2 = NaHCO3`, -28.33 kJ/mol) which is a different
    /// thing from dissolved sodium bicarbonate, and comparing the two
    /// would be comparing a salt with a complex that shares its name.
    #[test]
    fn a_solid_prices_as_its_ions_less_its_dissolution() {
        for (key, products) in DISSOCIATION {
            let expected: f64 = products
                .iter()
                .map(|(product, n)| {
                    aqueous_enthalpy(product, DB)
                        .unwrap_or_else(|| panic!("{key}: no h for product {product}"))
                        * n
                })
                .sum();
            // The rows below the solids arrive ALREADY dissolved and have
            // no dissolution to charge — they only have to price in
            // solution, as the sum of what they are.
            let Some(dissolution) =
                species::lookup(&SpeciesId::new(key)).and_then(|d| d.dissolution_enthalpy_kj)
            else {
                let aqueous = species_enthalpy(key, Phase::Aqueous, DB).unwrap_or_else(|e| {
                    panic!(
                        "{key} arrives dissolved and does not price: {} ({})",
                        e.species, e.why
                    )
                });
                assert!(
                    (aqueous - expected).abs() < 1e-9,
                    "{key} in solution: priced {aqueous}, its ions come to {expected}"
                );
                continue;
            };
            let solid = species_enthalpy(key, Phase::Solid, DB)
                .unwrap_or_else(|e| panic!("{key} as a solid: {} ({})", e.species, e.why));
            assert!(
                (solid - (expected - dissolution)).abs() < 1e-9,
                "{key}: priced {solid}, expected its ions ({expected}) less \
                 its dissolution ({dissolution})"
            );
            // And it must price in solution too, by whichever route.
            species_enthalpy(key, Phase::Aqueous, DB)
                .unwrap_or_else(|e| panic!("{key} in solution: {} ({})", e.species, e.why));
        }
    }

    /// TH-009. Quicklime's hydration comes from ONE database that writes
    /// both phases with the same convention (minteq.v4 Lime and
    /// Portlandite), and is close to the standard-table -65.2 kJ/mol.
    #[test]
    fn lime_hydration_is_reviewed_and_validated_against_its_own_convention() {
        let h = hydration_enthalpy("CaO").expect("validated hydration");
        assert!((h - (-193.91 - -128.62)).abs() < 1e-9, "{h}");
        assert!((h - (-65.2)).abs() < 0.5, "{h}");
        // Nothing outside the table is invented.
        for oxide in ["MgO", "Na2O", "CaCO3", "glitter"] {
            assert!(hydration_enthalpy(oxide).is_none(), "{oxide}");
        }
    }

    /// TH-009 on every route, including wateq4f which has no Lime phase:
    /// slaking 0.01 mol releases about 0.65 kJ, and dissolving part of the
    /// portlandite adds its own (exothermic) solution heat rather than the
    /// old fallback's lone endothermic precipitation term.
    #[test]
    fn slaking_quicklime_warms_on_every_route() {
        for db in ["wateq4f", "minteq.v4"] {
            let before = [
                portion("water", 5.5509, Phase::Liquid),
                portion("CaO", 0.01, Phase::Solid),
            ];
            let slaked = [
                portion("water", 5.5409, Phase::Liquid),
                portion("Ca(OH)2", 0.01, Phase::Solid),
            ];
            let q = heat_released_j(&before, 0.0, &slaked, 0.0, &[], db).expect("owned");
            assert!((q - 652.9).abs() < 1e-6, "{db}: {q} J");
            // The production state: part of the hydroxide dissolved.
            let partly = [
                portion("water", 5.5409, Phase::Liquid),
                portion("Ca(OH)2", 0.007967, Phase::Solid),
                portion("Ca+2", 0.002033, Phase::Aqueous),
            ];
            let q = heat_released_j(&before, 0.0, &partly, 0.004066, &[], db)
                .expect("free hydroxide is supplied by the oxide");
            assert!(q > 652.9 && q < 720.0, "{db}: {q} J");
        }
    }

    /// TH-015. Neutralising 5 mmol of carried hydroxide with strong acid
    /// releases ~55.8 kJ per mole of water formed, ~0.28 kJ here.
    #[test]
    fn carried_hydroxide_neutralised_by_acid_releases_its_heat() {
        for db in ["wateq4f", "minteq.v4"] {
            let mixed = [
                portion("water", 5.5509, Phase::Liquid),
                portion("Na+", 0.005, Phase::Aqueous),
                portion("Cl-", 0.005, Phase::Aqueous),
            ];
            let q = heat_released_j(&mixed, 0.005, &mixed, 1e-7, &[], db).expect("priced");
            assert!(q > 270.0 && q < 285.0, "{db}: {q} J");
        }
    }

    /// Negative: table salt dissolving is small and endothermic (+3.88
    /// kJ/mol), not a reaction heat.
    #[test]
    fn table_salt_dissolution_is_near_zero_and_endothermic() {
        for db in ["wateq4f", "minteq.v4"] {
            let before = [portion("NaCl", 0.01, Phase::Solid)];
            let after = [
                portion("Na+", 0.01, Phase::Aqueous),
                portion("Cl-", 0.01, Phase::Aqueous),
            ];
            let q = heat_released_j(&before, 0.0, &after, 0.0, &[], db).expect("priced");
            assert!((q - (-38.8)).abs() < 1e-9, "{db}: {q} J");
        }
    }

    /// Negative: potassium nitrate has no reviewed dissolution enthalpy in
    /// the registry and no Niter phase on the shipped routes, so the whole
    /// step declines by name (TH-007) instead of being charged partially.
    #[test]
    fn an_unsupported_solid_is_refused_whole() {
        for db in ["wateq4f", "minteq.v4"] {
            let before = [
                portion("water", 2.775, Phase::Liquid),
                portion("KNO3", 0.2967, Phase::Solid),
            ];
            let after = [
                portion("water", 2.775, Phase::Liquid),
                portion("K+", 0.2967, Phase::Aqueous),
                portion("NO3-", 0.2967, Phase::Aqueous),
            ];
            let err = heat_released_j(&before, 0.0, &after, 0.0, &[], db).expect_err("refused");
            assert_eq!(err.species, "KNO3", "{db}");
        }
    }
}
