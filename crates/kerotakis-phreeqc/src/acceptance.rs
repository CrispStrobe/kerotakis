//! Release-one acceptance scenarios shared by native, Wasm, cache-only, and
//! offline clients.
//!
//! These are application-path checks, not a second chemistry implementation:
//! every aqueous answer comes through the supplied [`Equilibrator`]. A native
//! build supplies linked IPhreeqc, the browser supplies its Emscripten hook,
//! and an offline/cache-only build supplies pre-warmed results through the
//! same `PhreeqcEquilibrator` mapping and readback path.

use std::collections::BTreeMap;

use kerotakis_core::{
    equilibrate_phase_coupled, species, stoich, Bench, CellChain, Equilibrator, Event, ExchangeIon,
    ExchangeOccupancy, ExchangeSites, Grams, Joules, Liters, MobileParcel, Moles, Operator,
    PermissiveScreen, Phase, SolveError, SpeciesId, SurfaceModel, SurfaceSites, SurfaceSorbate,
    Vessel, VesselId, PHASE_COUPLED_TEMPERATURE_TOLERANCE_K,
};

const WATER_MOLES_100_ML: f64 = 5.5509;

/// Stable machine-readable result of the complete R1 gate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct R1AcceptanceReport {
    pub schema: u32,
    pub cases: Vec<R1CaseResult>,
}

impl R1AcceptanceReport {
    pub fn passed(&self) -> bool {
        self.cases.iter().all(|case| case.passed)
    }
}

/// One independently diagnosable release scenario.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct R1CaseResult {
    pub id: String,
    pub passed: bool,
    pub detail: String,
    pub metrics: BTreeMap<String, f64>,
}

/// Run the five R1 scenarios through one supplied aqueous path.
pub fn run_r1_acceptance(solver: &mut dyn Equilibrator) -> R1AcceptanceReport {
    R1AcceptanceReport {
        schema: 1,
        cases: vec![
            capture("limewater", || limewater(solver)),
            capture("carbonated_bottle", || carbonated_bottle(solver)),
            capture("surface_release", || surface_release(solver)),
            capture("softener_breakthrough", || softener_breakthrough(solver)),
            capture("partial_freezing", || partial_freezing(solver)),
        ],
    }
}

fn capture(
    id: &str,
    scenario: impl FnOnce() -> Result<(BTreeMap<String, f64>, Vec<String>), String>,
) -> R1CaseResult {
    match scenario() {
        Ok((metrics, failures)) => R1CaseResult {
            id: id.to_string(),
            passed: failures.is_empty(),
            detail: if failures.is_empty() {
                "passed".to_string()
            } else {
                failures.join("; ")
            },
            metrics,
        },
        Err(detail) => R1CaseResult {
            id: id.to_string(),
            passed: false,
            detail,
            metrics: BTreeMap::new(),
        },
    }
}

fn step(
    bench: &mut Bench,
    solver: &mut dyn Equilibrator,
    operator: Operator,
) -> Result<Vec<Event>, String> {
    let events = bench
        .step_with(operator, solver, &PermissiveScreen)
        .map_err(|error| error.to_string())?;
    if let Some(Event::SolverFailed { solver, detail, .. }) = events
        .iter()
        .find(|event| matches!(event, Event::SolverFailed { .. }))
    {
        return Err(format!("solver '{solver}' failed: {detail}"));
    }
    Ok(events)
}

fn add(
    bench: &mut Bench,
    solver: &mut dyn Equilibrator,
    species: &str,
    moles: f64,
) -> Result<Vec<Event>, String> {
    step(
        bench,
        solver,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(species),
            moles: Moles(moles),
            at: None,
        },
    )
}

fn solid_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Solid && portion.species.0 == key)
        .map(|portion| portion.moles.0)
        .sum()
}

fn gas_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Gas && portion.species.0 == key)
        .map(|portion| portion.moles.0)
        .sum()
}

fn carbon_moles(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .map(|portion| {
            let carbon = species::lookup(&portion.species)
                .and_then(|data| stoich::parse_formula(data.formula).ok())
                .and_then(|formula| formula.counts.get("C").copied())
                .unwrap_or(0.0);
            portion.moles.0 * carbon
        })
        .sum()
}

fn limewater(
    solver: &mut dyn Equilibrator,
) -> Result<(BTreeMap<String, f64>, Vec<String>), String> {
    let mut bench = Bench::new();
    add(&mut bench, solver, "water", 55.51)?;
    add(&mut bench, solver, "Ca(OH)2", 0.01)?;
    let first = add(&mut bench, solver, "CO2", 0.01)?;
    let milky = solid_moles(
        bench.vessel(VesselId(0)).map_err(|e| e.to_string())?,
        "CaCO3",
    );
    let second = add(&mut bench, solver, "CO2", 0.05)?;
    let remaining = solid_moles(
        bench.vessel(VesselId(0)).map_err(|e| e.to_string())?,
        "CaCO3",
    );

    let mut failures = Vec::new();
    if milky <= 0.005 {
        failures.push(format!(
            "first CO2 dose formed only {milky:.6e} mol calcite"
        ));
    }
    if remaining >= milky * 0.1 {
        failures.push(format!(
            "excess CO2 left {remaining:.6e} mol from {milky:.6e} mol calcite"
        ));
    }
    if !first.iter().any(|event| {
        matches!(
            event,
            Event::Precipitated { species, .. } if species.0 == "CaCO3"
        )
    }) {
        failures.push("no calcite precipitation event".to_string());
    }
    if !second.iter().any(|event| {
        matches!(
            event,
            Event::Dissolved { species, .. } if species.0 == "CaCO3"
        )
    }) {
        failures.push("no calcite dissolution event".to_string());
    }

    Ok((
        BTreeMap::from([
            ("calcite_after_first_dose_mol".to_string(), milky),
            ("calcite_after_excess_mol".to_string(), remaining),
        ]),
        failures,
    ))
}

fn carbonated_bottle(
    solver: &mut dyn Equilibrator,
) -> Result<(BTreeMap<String, f64>, Vec<String>), String> {
    let mut bench = Bench::new();
    let seal = step(
        &mut bench,
        solver,
        Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(1.0),
        },
    )?;
    let initial_carbon = carbon_moles(bench.vessel(VesselId(0)).map_err(|e| e.to_string())?);
    add(&mut bench, solver, "water", 55.51)?;
    let events = add(&mut bench, solver, "NaHCO3", 0.05)?;
    let vessel = bench.vessel(VesselId(0)).map_err(|e| e.to_string())?;
    let carbon_error = carbon_moles(vessel) - (initial_carbon + 0.05);
    let ph = vessel
        .solution
        .as_ref()
        .map(|solution| solution.ph)
        .unwrap_or(f64::NAN);

    let mut failures = Vec::new();
    if !matches!(&vessel.headspace, kerotakis_core::Headspace::Sealed { .. }) {
        failures.push("bottle is not sealed".to_string());
    }
    if carbon_error.abs() >= 2e-7 {
        failures.push(format!(
            "sealed carbon ledger residual {carbon_error:.6e} mol"
        ));
    }
    if vessel.moles_of(&SpeciesId::new("CO2")).0 <= 0.0 {
        failures.push("no dissolved or gas-phase CO2 remained".to_string());
    }
    if !events
        .iter()
        .chain(seal.iter())
        .any(|event| matches!(event, Event::HeadspaceEquilibrated { .. }))
    {
        failures.push("no headspace-equilibrium event".to_string());
    }
    if events.iter().any(|event| {
        matches!(
            event,
            Event::GasEvolved { species, .. } if species.0 == "CO2"
        )
    }) {
        failures.push("sealed bottle emitted escaped CO2".to_string());
    }

    Ok((
        BTreeMap::from([
            ("carbon_residual_mol".to_string(), carbon_error),
            ("co2_gas_mol".to_string(), gas_moles(vessel, "CO2")),
            ("ph".to_string(), ph),
            ("pressure_pa".to_string(), vessel.pressure.0),
        ]),
        failures,
    ))
}

fn hfo() -> SurfaceSites {
    SurfaceSites {
        label: "R1 oxide grains".to_string(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: Vec::new(),
        water_release: Moles(0.0),
    }
}

fn surface_release(
    solver: &mut dyn Equilibrator,
) -> Result<(BTreeMap<String, f64>, Vec<String>), String> {
    let mut bench = Bench::new();
    bench.vessels[0].surfaces.push(hfo());
    add(&mut bench, solver, "water", 55.51)?;
    let neutral_water = bench
        .vessel(VesselId(0))
        .map_err(|e| e.to_string())?
        .moles_of(&SpeciesId::new("water"));
    add(&mut bench, solver, "ZnSO4", 1e-4)?;
    let vessel = bench.vessel(VesselId(0)).map_err(|e| e.to_string())?;
    let surface = &vessel.surfaces[0];
    let bound_zinc = surface.bound(SurfaceSorbate::Zinc).0;
    let bound_sulfate = surface.bound(SurfaceSorbate::Sulfate).0;
    let zinc_inventory = vessel.moles_of(&SpeciesId::new("Zn+2")).0 + bound_zinc;
    let sulfate_inventory = vessel.moles_of(&SpeciesId::new("SO4-2")).0 + bound_sulfate;
    let water_residual =
        vessel.moles_of(&SpeciesId::new("water")).0 - neutral_water.0 - surface.water_release.0;

    let mut failures = Vec::new();
    if bound_zinc <= 0.0 || bound_sulfate <= 0.0 {
        failures.push("HFO did not retain both zinc and sulfate".to_string());
    }
    if surface.water_release.0 <= 0.0 {
        failures.push("ligand exchange released no reference water".to_string());
    }
    if (zinc_inventory - 1e-4).abs() >= 2e-8 || (sulfate_inventory - 1e-4).abs() >= 2e-8 {
        failures.push(format!(
            "surface element ledger: zinc={zinc_inventory:.6e}, sulfate={sulfate_inventory:.6e} mol"
        ));
    }
    if water_residual.abs() >= 1e-10 {
        failures.push(format!(
            "surface reference-water residual {water_residual:.6e} mol"
        ));
    }
    if !surface.has_valid_capacity() {
        failures.push("surface occupancy exceeds finite capacity".to_string());
    }

    Ok((
        BTreeMap::from([
            ("bound_sulfate_mol".to_string(), bound_sulfate),
            ("bound_zinc_mol".to_string(), bound_zinc),
            ("water_release_mol".to_string(), surface.water_release.0),
            ("water_residual_mol".to_string(), water_residual),
        ]),
        failures,
    ))
}

fn resin(cell: usize) -> ExchangeSites {
    ExchangeSites {
        label: format!("R1 sodium resin {cell}"),
        dry_mass: Grams(1.0),
        capacity: Moles(5e-4),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(5e-4),
        }],
    }
}

fn exchange_cell(cell: usize) -> Vessel {
    let mut vessel = Vessel::new(VesselId(cell), format!("R1 softener cell {cell}"));
    vessel.deposit(
        SpeciesId::new("water"),
        Moles(WATER_MOLES_100_ML),
        Phase::Liquid,
    );
    vessel.exchanges.push(resin(cell));
    vessel
}

fn exchange_bound(vessel: &Vessel, ion: ExchangeIon) -> f64 {
    vessel
        .exchanges
        .iter()
        .map(|exchange| exchange.bound(ion).0)
        .sum()
}

fn exchange_inventory(chain: &CellChain, ion: ExchangeIon) -> f64 {
    chain
        .cells()
        .iter()
        .map(|cell| cell.moles_of(&ion.species()).0 + exchange_bound(cell, ion))
        .sum()
}

fn parcel_ion(parcel: &MobileParcel, ion: ExchangeIon) -> f64 {
    parcel.moles_of(&ion.species()).0
}

fn softener_breakthrough(
    solver: &mut dyn Equilibrator,
) -> Result<(BTreeMap<String, f64>, Vec<String>), String> {
    let mut feed = Vessel::new(VesselId(999), "R1 calcium feed");
    feed.deposit(
        SpeciesId::new("water"),
        Moles(WATER_MOLES_100_ML),
        Phase::Liquid,
    );
    feed.deposit(SpeciesId::new("Ca+2"), Moles(2.5e-4), Phase::Aqueous);
    feed.deposit(SpeciesId::new("Cl-"), Moles(5e-4), Phase::Aqueous);
    solver
        .equilibrate(&mut feed)
        .map_err(|error| error.to_string())?;
    let feed_calcium = feed.moles_of(&ExchangeIon::Calcium.species()).0;
    let mut chain =
        CellChain::new((0..4).map(exchange_cell).collect()).map_err(|error| error.to_string())?;
    let mut breakthrough = Vec::new();
    let mut max_ledger_error = 0.0_f64;

    for _ in 1..=12 {
        let calcium_before = exchange_inventory(&chain, ExchangeIon::Calcium);
        let sodium_before = exchange_inventory(&chain, ExchangeIon::Sodium);
        let advance = chain
            .advance_reactive(&feed, 1.0, solver)
            .map_err(|error| error.to_string())?;
        let calcium_error = calcium_before
            + parcel_ion(&advance.transport.injected, ExchangeIon::Calcium)
            - exchange_inventory(&chain, ExchangeIon::Calcium)
            - parcel_ion(&advance.transport.effluent, ExchangeIon::Calcium);
        let sodium_error = sodium_before
            + parcel_ion(&advance.transport.injected, ExchangeIon::Sodium)
            - exchange_inventory(&chain, ExchangeIon::Sodium)
            - parcel_ion(&advance.transport.effluent, ExchangeIon::Sodium);
        max_ledger_error = max_ledger_error
            .max(calcium_error.abs())
            .max(sodium_error.abs());
        breakthrough
            .push(parcel_ion(&advance.transport.effluent, ExchangeIon::Calcium) / feed_calcium);
    }

    let first = breakthrough[0];
    let midpoint = breakthrough[5];
    let last = breakthrough[11];
    let valid_capacity = chain
        .cells()
        .iter()
        .all(|cell| cell.exchanges.iter().all(ExchangeSites::has_valid_capacity));
    let mut failures = Vec::new();
    if first >= 1e-8 {
        failures.push(format!("first-pore-volume calcium fraction {first:.6e}"));
    }
    if last <= 0.8 || last <= midpoint + 0.25 {
        failures.push(format!(
            "breakthrough did not rise sufficiently: midpoint={midpoint:.6}, last={last:.6}"
        ));
    }
    if max_ledger_error >= 2e-8 {
        failures.push(format!(
            "maximum Ca/Na ledger residual {max_ledger_error:.6e} mol"
        ));
    }
    if !valid_capacity {
        failures.push("exchange occupancy exceeds finite capacity".to_string());
    }

    Ok((
        BTreeMap::from([
            ("first_fraction".to_string(), first),
            ("last_fraction".to_string(), last),
            ("max_ledger_residual_mol".to_string(), max_ledger_error),
            ("midpoint_fraction".to_string(), midpoint),
        ]),
        failures,
    ))
}

struct PhaseCoupled<'a>(&'a mut dyn Equilibrator);

impl Equilibrator for PhaseCoupled<'_> {
    fn name(&self) -> &'static str {
        "R1-phase-coupled"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        self.0.applies(vessel)
    }

    fn chemistry_applies(&self, vessel: &Vessel) -> bool {
        self.0.chemistry_applies(vessel)
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        equilibrate_phase_coupled(self.0, vessel)
    }
}

fn water_moles(vessel: &Vessel, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == "water" && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

fn particle_molality(vessel: &Vessel) -> Option<f64> {
    Some(
        vessel
            .solution
            .as_ref()?
            .species
            .iter()
            .filter(|species| species.name != "H2O")
            .map(|species| species.molality)
            .sum(),
    )
}

fn partial_freezing(
    solver: &mut dyn Equilibrator,
) -> Result<(BTreeMap<String, f64>, Vec<String>), String> {
    let mut bench = Bench::new();
    let events = {
        let mut coupled = PhaseCoupled(solver);
        step(
            &mut bench,
            &mut coupled,
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(WATER_MOLES_100_ML),
                at: None,
            },
        )?;
        step(
            &mut bench,
            &mut coupled,
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("NaCl"),
                moles: Moles(0.05),
                at: None,
            },
        )?;
        step(
            &mut bench,
            &mut coupled,
            Operator::Cool {
                vessel: VesselId(0),
                energy: Joules(20_000.0),
            },
        )?
    };
    let vessel = bench.vessel(VesselId(0)).map_err(|e| e.to_string())?;
    let liquid = water_moles(vessel, Phase::Liquid);
    let ice = water_moles(vessel, Phase::Solid);
    let molality = particle_molality(vessel).unwrap_or(f64::NAN);
    let liquidus = kerotakis_core::states::transitions(molality).freezing_k;
    let temperature_error = vessel.temperature.0 - liquidus;
    let sodium =
        vessel.moles_of(&SpeciesId::new("Na+")).0 + vessel.moles_of(&SpeciesId::new("NaCl")).0;

    let mut failures = Vec::new();
    if liquid <= 0.0 || ice <= 0.0 {
        failures.push(format!(
            "partial freezing produced liquid={liquid:.6e}, ice={ice:.6e} mol"
        ));
    }
    if vessel
        .contents
        .iter()
        .any(|portion| portion.phase == Phase::Solid && portion.species.0 != "water")
    {
        failures.push("the ice compartment contains solute".to_string());
    }
    if temperature_error.abs() > PHASE_COUPLED_TEMPERATURE_TOLERANCE_K {
        failures.push(format!(
            "temperature/liquidus mismatch {temperature_error:.6e} K"
        ));
    }
    if (liquid + ice - WATER_MOLES_100_ML).abs() >= 2e-6 {
        failures.push("water ledger did not close".to_string());
    }
    if (sodium - 0.05).abs() >= 1e-8 {
        failures.push(format!("sodium ledger ended at {sodium:.6e} mol"));
    }
    if !events.iter().any(|event| {
        matches!(
            event,
            Event::StateChanged {
                from: Phase::Liquid,
                to: Phase::Solid,
                ..
            }
        )
    }) {
        failures.push("no liquid-to-solid state event".to_string());
    }

    Ok((
        BTreeMap::from([
            ("ice_mol".to_string(), ice),
            ("liquid_water_mol".to_string(), liquid),
            ("particle_molality".to_string(), molality),
            (
                "temperature_liquidus_delta_k".to_string(),
                temperature_error,
            ),
        ]),
        failures,
    ))
}
