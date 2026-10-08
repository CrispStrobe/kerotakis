//! Strict opt-in closure for fully represented molecular inventories.
//!
//! Inventory differences are formed before elemental/formal-charge totals, so
//! unchanged solvent or charged background cannot hide a trace reaction. The
//! relative tolerance applies to changed reaction throughput, not the full
//! vessel inventory. There is no absolute trace floor. Historical addition lots
//! and derived solution distributions are not a second material inventory. This
//! certificate does not establish the correctness of derived aqueous speciation
//! or reaction energy; those remain separate scientific contracts.
//!
//! Two-component Amount arithmetic is bounded compensation, not arbitrary
//! precision. Every accumulation must recover both operands on subtraction;
//! otherwise this certificate refuses rather than dropping a third correction.
//! Untyped material owners and unsupported identities are explicit coverage
//! refusals. This policy is separate from legacy native-route element checks.

use crate::{
    amount::Amount, delta::DeltaError, ops::Event, species, stoich, vessel::SolidSolutionComponent,
    Vessel,
};
use std::collections::{BTreeMap, BTreeSet};

/// Formula metadata comes from the owner that admitted the amount. A typed
/// crystal can represent an end member absent from the bottle registry without
/// granting unknown primary contents a formula-string escape hatch.
struct OwnedAmount {
    amount: Amount,
    formula: stoich::Formula,
}
type Inventory = BTreeMap<String, OwnedAmount>;
type Result<T> = std::result::Result<T, String>;

fn zero() -> Amount {
    Amount::new(0.0).expect("zero amount")
}

/// Certify that a bounded expansion addition retained both complete operands.
fn add(a: Amount, b: Amount) -> Result<Amount> {
    let sum = a.checked_add(b).map_err(|e| e.to_string())?;
    let recovered_a = sum.checked_sub(b).map_err(|e| e.to_string())?;
    let recovered_b = sum.checked_sub(a).map_err(|e| e.to_string())?;
    if recovered_a != a || recovered_b != b {
        return Err("two-component inventory accumulation lost an operand correction".into());
    }
    Ok(sum)
}

/// Positive difference with a sign, retaining small corrections before any
/// formula sum. Reversibility is checked against the complete Amount values.
fn difference(a: Amount, b: Amount) -> Result<(bool, Amount)> {
    let (positive, larger, smaller, delta) = match a.checked_sub(b) {
        Ok(delta) => (true, a, b, delta),
        Err(crate::amount::AmountError::Negative) => {
            let delta = b.checked_sub(a).map_err(|e| e.to_string())?;
            (false, b, a, delta)
        }
        Err(error) => return Err(error.to_string()),
    };
    if add(smaller, delta)? != larger
        || larger.checked_sub(delta).map_err(|e| e.to_string())? != smaller
    {
        return Err("two-component inventory subtraction is not reversible".into());
    }
    Ok((positive, delta))
}

fn nonzero(amount: Amount) -> bool {
    amount != zero()
}

fn supported(key: &str) -> Result<(&'static species::SpeciesData, stoich::Formula)> {
    let data = species::lookup_key(key)
        .ok_or_else(|| format!("unrepresented owned molecular identity: {key}"))?;
    let formula = stoich::parse_formula(data.formula)
        .map_err(|_| format!("unparseable owned molecular formula: {key}"))?;
    if formula.counts.is_empty()
        || formula.counts.values().any(|n| !n.is_finite() || *n <= 0.0)
        || !formula.charge.is_finite()
        || !data.molar_mass.is_finite()
        || data.molar_mass <= 0.0
    {
        return Err(format!(
            "incomplete molecular element/charge/mass record: {key}"
        ));
    }
    Ok((data, formula))
}

fn merge_owned(
    inventory: &mut Inventory,
    key: &str,
    amount: Amount,
    formula: stoich::Formula,
) -> Result<()> {
    if let Some(previous) = inventory.get_mut(key) {
        if previous.formula.counts != formula.counts || previous.formula.charge != formula.charge {
            return Err(format!("conflicting owned molecular composition: {key}"));
        }
        previous.amount = add(previous.amount, amount)?;
    } else {
        inventory.insert(key.to_owned(), OwnedAmount { amount, formula });
    }
    Ok(())
}

fn deposit(inventory: &mut Inventory, key: &str, value: f64) -> Result<()> {
    let amount = Amount::new(value).map_err(|e| format!("{key}: {e}"))?;
    if !nonzero(amount) {
        return Ok(());
    }
    let (data, formula) = supported(key)?;
    merge_owned(inventory, data.key, amount, formula)
}

fn deposit_crystal_component(
    inventory: &mut Inventory,
    component: SolidSolutionComponent,
    value: f64,
) -> Result<()> {
    let amount = Amount::new(value).map_err(|e| e.to_string())?;
    if !nonzero(amount) {
        return Ok(());
    }
    // This exhaustive mapping is limited to the model's explicit formula-unit
    // owners. It supplies composition, not solubility, kinetics or heat data.
    let formula_text = match component {
        SolidSolutionComponent::CalciumCarbonate => "CaCO3",
        SolidSolutionComponent::StrontiumCarbonate => "SrCO3",
    };
    let formula = stoich::parse_formula(formula_text).map_err(|e| format!("{e:?}"))?;
    let identity = component.species();
    let key = species::lookup(&identity)
        .map(|data| data.key)
        .unwrap_or(&identity.0);
    merge_owned(inventory, key, amount, formula)
}

fn closes(produced: Amount, consumed: Amount, tolerance: f64) -> Result<bool> {
    let (_, imbalance) = difference(produced, consumed)?;
    if !nonzero(imbalance) {
        return Ok(true);
    }
    if tolerance == 0.0 {
        return Ok(false);
    }
    let throughput = produced.to_f64().max(consumed.to_f64());
    let relative = imbalance.to_f64() / throughput;
    if !throughput.is_finite() || throughput <= 0.0 || !relative.is_finite() {
        return Err("reaction-throughput ratio is not representable".into());
    }
    Ok(relative <= tolerance)
}

fn inventory(vessel: &Vessel, tolerance: f64) -> Result<Inventory> {
    // These owners do not supply complete canonical molecular composition.
    // Preserve the explicit strict-policy coverage boundary, even if their
    // unrepresented quantities happen to be unchanged in this particular route.
    if vessel.unresolved_materials.iter().any(|p| p.amount != 0.0) {
        return Err("unresolved named material has no complete molecular inventory".into());
    }
    if !vessel.surfaces.is_empty() {
        return Err(
            "oxide surface support/site inventory is outside strict molecular closure".into(),
        );
    }
    if !vessel.exchanges.is_empty() {
        return Err("exchanger support/site inventory is outside strict molecular closure".into());
    }
    if vessel.soap_scum.as_ref().is_some_and(|s| {
        s.aggregate_mass_g != 0.0 || s.divalent_ion_moles != 0.0 || s.soap_equivalent_moles != 0.0
    }) {
        return Err("soap-scum aggregate has no complete molecular inventory".into());
    }
    if vessel.nuclides.inventory.values().any(|n| *n != 0.0) {
        return Err("nuclide ownership requires its separate nuclear certificate".into());
    }
    let mut inventory = Inventory::new();
    for portion in &vessel.contents {
        deposit(&mut inventory, &portion.species.0, portion.moles.0)?;
    }
    let mut crystal_labels = BTreeSet::new();
    for crystal in &vessel.solid_solutions {
        if !crystal.has_valid_state() || !crystal_labels.insert(&crystal.label) {
            return Err("invalid or ambiguous typed crystal ownership".into());
        }
        for component in &crystal.components {
            deposit_crystal_component(&mut inventory, component.component, component.moles.0)?;
        }
    }
    for bound in &vessel.adsorbed {
        deposit(&mut inventory, &bound.sorbate.0, bound.moles.0)?;
    }
    for electrode in &vessel.electrodes {
        if let Some(amount) = electrode.substrate_moles {
            deposit(&mut inventory, &electrode.material, amount)?;
        }
        for portion in &electrode.deposits {
            deposit(&mut inventory, &portion.species, portion.moles)?;
        }
    }
    for object in &vessel.material_objects {
        let declared = Amount::new(object.mass_g).map_err(|e| e.to_string())?;
        let mut accounted = zero();
        for component in &object.components {
            let amount = Amount::new(component.moles.0).map_err(|e| e.to_string())?;
            if !nonzero(amount) {
                continue;
            }
            let (data, _) = supported(&component.species.0)?;
            let mass = amount
                .checked_scale(data.molar_mass)
                .map_err(|e| e.to_string())?;
            accounted = add(accounted, mass)?;
            deposit(&mut inventory, &component.species.0, component.moles.0)?;
        }
        if !closes(declared, accounted, tolerance)? {
            return Err(format!(
                "prepared object {} has unrepresented declared mass",
                object.material
            ));
        }
    }
    Ok(inventory)
}

#[derive(Default)]
struct Balance {
    produced: Option<Amount>,
    consumed: Option<Amount>,
}
impl Balance {
    fn note(&mut self, positive: bool, amount: Amount) -> Result<()> {
        let side = if positive {
            &mut self.produced
        } else {
            &mut self.consumed
        };
        *side = Some(add(side.unwrap_or_else(zero), amount)?);
        Ok(())
    }
    fn closes(&self, tolerance: f64) -> Result<bool> {
        closes(
            self.produced.unwrap_or_else(zero),
            self.consumed.unwrap_or_else(zero),
            tolerance,
        )
    }
}

fn validate(before: &Vessel, after: &Vessel, events: &[Event], tolerance: f64) -> Result<()> {
    if !tolerance.is_finite() || !(0.0..1.0).contains(&tolerance) {
        return Err("required conservation tolerance must be finite and below one".into());
    }
    let gas_errors = crate::delta::StateDelta::validate_gas_events(after, events);
    if !gas_errors.is_empty() {
        return Err(format!("invalid gas boundary: {gas_errors:?}"));
    }
    let mut before_inventory = inventory(before, tolerance)?;
    let mut after_inventory = inventory(after, tolerance)?;
    for event in events {
        match event {
            Event::GasEvolved { species, moles, .. } => {
                deposit(&mut after_inventory, &species.0, moles.0)?;
            }
            Event::GasAbsorbed { species, moles, .. } => {
                deposit(&mut before_inventory, &species.0, moles.0)?;
            }
            _ => {}
        }
    }
    let identities: BTreeSet<_> = before_inventory
        .keys()
        .chain(after_inventory.keys())
        .collect();
    let mut elements = BTreeMap::<String, Balance>::new();
    let mut charge = Balance::default();
    for key in identities {
        let before = before_inventory.get(key);
        let after = after_inventory.get(key);
        if let (Some(before), Some(after)) = (before, after) {
            if before.formula.counts != after.formula.counts
                || before.formula.charge != after.formula.charge
            {
                return Err(format!(
                    "owned molecular composition changed identity: {key}"
                ));
            }
        }
        let before_amount = before.map(|entry| entry.amount).unwrap_or_else(zero);
        let after_amount = after.map(|entry| entry.amount).unwrap_or_else(zero);
        let (positive, changed) = difference(after_amount, before_amount)?;
        if !nonzero(changed) {
            continue;
        }
        let formula = &after.or(before).expect("union inventory identity").formula;
        for (element, count) in &formula.counts {
            let count = changed.checked_scale(*count).map_err(|e| e.to_string())?;
            elements
                .entry(element.clone())
                .or_default()
                .note(positive, count)?;
        }
        if formula.charge != 0.0 {
            let charge_amount = changed
                .checked_scale(formula.charge.abs())
                .map_err(|e| e.to_string())?;
            charge.note(positive == (formula.charge > 0.0), charge_amount)?;
        }
    }
    for (element, balance) in elements {
        if !balance.closes(tolerance)? {
            return Err(format!(
                "changed reaction throughput does not conserve element {element}"
            ));
        }
    }
    if !charge.closes(tolerance)? {
        return Err("changed reaction throughput does not conserve formal charge".into());
    }
    Ok(())
}

/// Explicit required-stack policy. Successful return is a bounded certificate
/// for represented elements and formal charge; energy is a separate contract.
pub(crate) fn validate_required_conservation(
    before: &Vessel,
    after: &Vessel,
    events: &[Event],
    tolerance: f64,
) -> Vec<DeltaError> {
    match validate(before, after, events, tolerance) {
        Ok(()) => vec![],
        Err(detail) => vec![DeltaError::InvalidState {
            field: format!("required conservation certificate: {detail}"),
        }],
    }
}
