//! COLUMN-001: breakthrough conservation and dispersion characterisation.
//!
//! The acceptance criteria from the coupling review: matter closes exactly
//! over a whole open-column run at any Courant fraction, and the scheme's
//! numerical behaviour is characterised rather than assumed. Two facts
//! about first-order upwind shape these tests: at Courant 1 the update is
//! exact plug flow (zero numerical diffusion), and below Courant 1 the
//! scheme diffuses, with the smearing shrinking as the SPATIAL grid is
//! refined at the same physical flow. This is a numerical check of the
//! engine's own transport maths, NOT independent experimental validation.

use kerotakis_core::*;

const COLUMN_WATER_MOLES: f64 = 4.0 * 5.5509;

fn column_cell(id: usize, water_moles: f64, tracer_moles: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(id), format!("cell {id}"));
    vessel.deposit(SpeciesId::new("water"), Moles(water_moles), Phase::Liquid);
    vessel.deposit(
        SpeciesId::new("passive-tracer"),
        Moles(tracer_moles),
        Phase::Aqueous,
    );
    vessel.temperature = Kelvin(295.0);
    vessel.solute_charge = tracer_moles * 0.25;
    vessel.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: Vec::new(),
        ph: 7.0,
        ionic_strength: 0.0,
        species: Vec::new(),
        provenance: None,
    });
    vessel
}

/// An inlet matched to one cell of the column: same liquid volume, tracer
/// at 1 mole per 5.5509 mol of water (one mole per original cell).
fn inlet_for_cell(water_moles: f64) -> Vessel {
    let mut inlet = Vessel::new(VesselId(99), "inlet");
    inlet.deposit(SpeciesId::new("water"), Moles(water_moles), Phase::Liquid);
    inlet.deposit(
        SpeciesId::new("passive-tracer"),
        Moles(water_moles / 5.5509),
        Phase::Aqueous,
    );
    inlet.temperature = Kelvin(295.0);
    inlet.solute_charge = 0.25 * water_moles / 5.5509;
    inlet.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: Vec::new(),
        ph: 7.0,
        ionic_strength: 0.0,
        species: Vec::new(),
        provenance: None,
    });
    inlet
}

/// Run `steps` transport steps of the given Courant fraction over a
/// uniform `cells`-cell column and return
/// (chain-held tracer, total effluent tracer, total injected tracer).
fn run_column(cells: usize, fraction: f64, steps: usize) -> (f64, f64, f64) {
    // Fixed column volume and inlet concentration across grid refinements.
    let cell_water = COLUMN_WATER_MOLES / cells as f64;
    let chain_cells: Vec<Vessel> = (0..cells)
        .map(|id| column_cell(id, cell_water, 0.0))
        .collect();
    let mut chain = transport::CellChain::new(chain_cells).unwrap();
    let inlet = inlet_for_cell(cell_water);
    let tracer = SpeciesId::new("passive-tracer");
    let mut total_effluent = 0.0;
    let mut total_injected = 0.0;
    for _ in 0..steps {
        let before = chain.total_moles(&tracer).0;
        let step = chain.advance(&inlet, fraction).unwrap();
        let after = chain.total_moles(&tracer).0;
        let effluent = step.effluent.moles_of(&tracer).0;
        // `TransportStep::injected` already records the Courant-scaled
        // fraction actually injected (see the existing transport.rs ledger
        // test); scaling again here would double-count it.
        let injected = step.injected.moles_of(&tracer).0;
        total_effluent += effluent;
        total_injected += injected;
        assert!(
            (before + injected - after - effluent).abs() < 1e-12,
            "tracer ledger must close at every step"
        );
    }
    (chain.total_moles(&tracer).0, total_effluent, total_injected)
}

#[test]
fn breakthrough_conserves_mass_at_every_courant_fraction() {
    for fraction in [1.0, 0.5, 0.25, 0.125] {
        let (in_chain, effluent, injected) = run_column(4, fraction, 40);
        let total = in_chain + effluent;
        assert!(
            (total - injected).abs() < 1e-9,
            "fraction {fraction}: chain {in_chain} + effluent {effluent} != injected {injected}"
        );
    }
}

#[test]
fn courant_one_is_exact_plug_flow() {
    // After exactly one column volume has been displaced at Courant 1, the
    // front stands at the outlet having lost nothing: every injected mole
    // is still in the chain, and the effluent is exactly zero.
    let cells = 4;
    let (in_chain, effluent, injected) = run_column(cells, 1.0, cells);
    assert!(effluent.abs() < 1e-12, "effluent {effluent}");
    assert!(
        (injected - cells as f64).abs() < 1e-9,
        "injected {injected}"
    );
    assert!(
        (in_chain - injected).abs() < 1e-9,
        "chain {in_chain} vs injected {injected}: plug flow must hold everything"
    );
}

#[test]
fn refining_the_spatial_grid_shrinks_numerical_dispersion() {
    // At half a column volume displaced the ideal front is mid-column, and
    // no grid size may leak any tracer past the outlet: the smeared front
    // cannot arrive that early. (Verified analytically alongside this
    // file; the assertion is exact, not a tolerance.)
    for cells in [4, 8, 16] {
        let steps = (cells as f64 * 0.5 / 0.5).round() as usize;
        let (_, effluent, _) = run_column(cells, 0.5, steps);
        assert!(
            effluent.abs() < 1e-12,
            "early breakthrough at {cells} cells: {effluent}"
        );
    }

    // At one full column volume displaced the ideal answer is still zero
    // effluent, but the front is centred on the outlet, so first-order
    // upwind's smearing lets a tail through. The meaningful invariant is
    // the FRACTION of injected tracer that breaks through: it must shrink
    // with each spatial refinement (measured 0.137 -> 0.098 -> 0.070 on
    // this problem).
    let breakthrough_fraction = |cells: usize| {
        let steps = (cells as f64 / 0.5).round() as usize;
        let (_, effluent, injected) = run_column(cells, 0.5, steps);
        effluent / injected
    };
    let coarse = breakthrough_fraction(4);
    let mid = breakthrough_fraction(8);
    let fine = breakthrough_fraction(16);
    assert!(
        coarse > mid && mid > fine,
        "spatial refinement must shrink the breakthrough fraction: {coarse} -> {mid} -> {fine}"
    );
}
