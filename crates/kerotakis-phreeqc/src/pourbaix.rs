//! Predominance (Pourbaix) diagrams, computed cell by cell (CAP-4).
//!
//! Every cell of the pe–pH grid is a real PHREEQC solve: fix pH and pe,
//! offer the curated mineral set at saturation-index zero with nothing to
//! dissolve, and ask what the element actually is under those conditions —
//! a precipitate if one formed and holds the majority of the element, else
//! the dominant aqueous species from the engine's own distribution. The
//! regions of the textbook diagram are where those answers agree with each
//! other; nothing here is a lookup.
//!
//! Cells the engine refuses stay refused: they are counted, labelled
//! unsolved, and never interpolated over — declining to model something
//! must be loud (PLAN.md).

use crate::aqueous::{parse_species_distribution, PhreeqcEquilibrator};
use std::fmt::Write as _;

/// One curated element system: which database, how much of the element,
/// and which minerals are allowed to form. Curation is the honest part —
/// a diagram can only show phases its database and this list both know.
pub struct ElementSystem {
    pub element: &'static str,
    /// mol/kgw of the element in every cell; dilute enough to stay in the
    /// database's activity-model comfort zone, concentrated enough that
    /// precipitation is observable.
    pub molality: f64,
    /// Phases offered for precipitation (SI 0, zero initial moles: they
    /// can form, never dissolve).
    pub minerals: &'static [&'static str],
    pub db_tag: &'static str,
    pub source: &'static str,
}

/// The curated systems. Growing this list is data work: pick the phases
/// the database defines for the element, cite why.
const SYSTEMS: &[ElementSystem] = &[
    ElementSystem {
        element: "Fe",
        molality: 1e-4,
        // wateq4f's iron oxyhydroxides: the amorphous phase is what forms
        // on a bench timescale; goethite and hematite are its aged fates
        // and would blanket the diagram if offered as equilibrium phases,
        // so the bench-honest choice is the phase a beaker actually makes.
        minerals: &["Fe(OH)3(a)", "Siderite"],
        db_tag: "wateq4f",
        source: "phases: WATEQ4F database as shipped (USGS); selection: \
                 amorphous ferric hydroxide is the phase that forms on a \
                 laboratory timescale (aged oxides excluded as kinetically \
                 unreachable in a beaker); Fe total 1e-4 mol/kgw in 0.01 \
                 mol/kgw NaCl background",
    },
    ElementSystem {
        element: "Cu",
        molality: 1e-4,
        minerals: &["Cu(OH)2", "Tenorite", "Cuprite"],
        db_tag: "wateq4f",
        source: "phases: WATEQ4F database as shipped (USGS); Cu total 1e-4 \
                 mol/kgw in 0.01 mol/kgw NaCl background",
    },
];

/// A computed diagram: axes, one dominant-form label per cell (row-major,
/// pe rows over pH columns), and the bookkeeping a caption needs.
pub struct PourbaixDiagram {
    pub element: String,
    pub ph: Vec<f64>,
    pub pe: Vec<f64>,
    /// `labels[i_pe * ph.len() + i_ph]`; `None` where the engine refused.
    pub labels: Vec<Option<String>>,
    pub refused: usize,
    pub t_celsius: f64,
    pub db_tag: String,
    pub provenance: String,
}

impl PourbaixDiagram {
    pub fn label(&self, i_pe: usize, i_ph: usize) -> Option<&str> {
        self.labels[i_pe * self.ph.len() + i_ph].as_deref()
    }

    /// Every distinct dominant form, in first-appearance order — the
    /// legend, and the assertion surface for tests.
    pub fn distinct(&self) -> Vec<String> {
        let mut seen = Vec::new();
        for l in self.labels.iter().flatten() {
            if !seen.iter().any(|s| s == l) {
                seen.push(l.clone());
            }
        }
        seen
    }
}

fn system_for(element: &str) -> Option<&'static ElementSystem> {
    SYSTEMS.iter().find(|s| s.element == element)
}

/// The elements a diagram is curated for.
pub fn curated_elements() -> Vec<&'static str> {
    SYSTEMS.iter().map(|s| s.element).collect()
}

fn cell_input(sys: &ElementSystem, ph: f64, pe: f64) -> String {
    let mut input = String::new();
    writeln!(input, "TITLE pourbaix cell").unwrap();
    writeln!(input, "SOLUTION 1").unwrap();
    writeln!(input, "    temp      25.0").unwrap();
    writeln!(input, "    units     mol/kgw").unwrap();
    writeln!(input, "    pH        {ph:.4}").unwrap();
    writeln!(input, "    pe        {pe:.4}").unwrap();
    // A dilute background electrolyte so activity coefficients are sane;
    // pH and pe are both pinned, so charge is deliberately not balanced —
    // the diagram asks "what is the element at these coordinates", not
    // "is this beaker preparable".
    writeln!(input, "    Na        1e-2").unwrap();
    writeln!(input, "    Cl        1e-2").unwrap();
    writeln!(input, "    {:<9} {:e}", sys.element, sys.molality).unwrap();
    writeln!(input, "EQUILIBRIUM_PHASES 1").unwrap();
    for m in sys.minerals {
        writeln!(input, "    {m}  0.0  0.0").unwrap();
    }
    writeln!(input, "SELECTED_OUTPUT").unwrap();
    writeln!(input, "    -reset    false").unwrap();
    writeln!(input, "    -high_precision true").unwrap();
    writeln!(input, "    -equilibrium_phases {}", sys.minerals.join(" ")).unwrap();
    writeln!(input, "END").unwrap();
    input
}

/// Dominant form at one (pH, pe): the precipitate if one holds the
/// majority of the element, else the top element-bearing aqueous species.
fn classify(eq: &mut PhreeqcEquilibrator, sys: &ElementSystem, ph: f64, pe: f64) -> Option<String> {
    let input = cell_input(sys, ph, pe);
    let out = eq.run_raw(sys.db_tag, &input).ok()?;
    // Phase moles from selected output: header row names the phase, the
    // last data row carries the moles now present.
    if let (Some(header), Some(row)) = (out.selected.first(), out.selected.last()) {
        if out.selected.len() >= 2 {
            let mut best_phase: Option<(&str, f64)> = None;
            for m in sys.minerals {
                if let Some(idx) = header.iter().position(|h| h.trim() == *m) {
                    if let Some(moles) = row.get(idx).and_then(|v| v.trim().parse::<f64>().ok()) {
                        if best_phase.is_none_or(|(_, b)| moles > b) {
                            best_phase = Some((m, moles));
                        }
                    }
                }
            }
            if let Some((phase, moles)) = best_phase {
                if moles > 0.5 * sys.molality {
                    return Some(phase.to_string());
                }
            }
        }
    }
    // No majority precipitate: the dominant dissolved form. The substring
    // test is sound for the curated symbols (two-letter element symbols
    // cannot appear inside another species name by accident); a curated
    // system with a one-letter symbol would need a real formula parse
    // here, which is why none is curated yet.
    let speciation = parse_species_distribution(&out.report);
    speciation
        .into_iter()
        .filter(|s| s.name.contains(sys.element))
        .max_by(|a, b| a.molality.total_cmp(&b.molality))
        .map(|s| s.name)
}

/// Compute the full grid. `nx`/`ny` are the pH/pe cell counts; cost is one
/// engine solve per cell (no redox bisection — pe is the axis, not an
/// unknown), so a 48x40 grid is ~2k solves.
pub fn diagram(
    eq: &mut PhreeqcEquilibrator,
    element: &str,
    nx: usize,
    ny: usize,
) -> Result<PourbaixDiagram, String> {
    let sys = system_for(element).ok_or_else(|| {
        format!(
            "no curated Pourbaix system for '{element}' — curated: {} \
             (adding one is data work: pick the database's phases for the \
             element and cite why)",
            curated_elements().join(", ")
        )
    })?;
    if !eq.can_solve() {
        return Err("a Pourbaix grid needs a live engine (or attached solver); \
                    pre-warmed results cannot cover a fresh grid"
            .to_string());
    }
    let (nx, ny) = (nx.max(4), ny.max(4));
    // pH spans the bench-real range; pe spans the water-stability window
    // plus a margin so the O2 and H2 lines are visibly *inside* the frame.
    let ph: Vec<f64> = (0..nx).map(|i| 14.0 * i as f64 / (nx - 1) as f64).collect();
    let pe: Vec<f64> = (0..ny)
        .map(|j| -12.0 + 34.0 * j as f64 / (ny - 1) as f64)
        .collect();
    let mut labels = Vec::with_capacity(nx * ny);
    let mut refused = 0usize;
    for &pe_v in &pe {
        for &ph_v in &ph {
            let label = classify(eq, sys, ph_v, pe_v);
            if label.is_none() {
                refused += 1;
            }
            labels.push(label);
        }
    }
    Ok(PourbaixDiagram {
        element: sys.element.to_string(),
        ph,
        pe,
        labels,
        refused,
        t_celsius: 25.0,
        db_tag: sys.db_tag.to_string(),
        provenance: sys.source.to_string(),
    })
}

/// A polyline in (pH, pe) coordinates.
pub type PePhLine = Vec<(f64, f64)>;

/// The water-stability lines at 25 °C, for drawing over the grid.
///
/// Upper: O2(g) at 1 atm, O2 + 4 H+ + 4 e− = 2 H2O, pe = 20.75 − pH.
/// Lower: H2(g) at 1 atm, 2 H+ + 2 e− = H2, pe = −pH.
/// The 20.75 is log K/4 for the oxygen half-reaction as tabulated in the
/// PHREEQC databases themselves (O2 log_k 83.1 per 4 e− at 25 °C → 20.775,
/// conventionally quoted 20.75); the hydrogen line is exact by the
/// definition of pe.
pub fn water_stability_lines(ph: &[f64]) -> (PePhLine, PePhLine) {
    let upper = ph.iter().map(|&x| (x, 20.75 - x)).collect();
    let lower = ph.iter().map(|&x| (x, -x)).collect();
    (upper, lower)
}

/// Whether a grid point lies outside the water-stability field (with half
/// a pe unit of margin). Out there the engine's refusal *is* the answer:
/// water itself is oxidised or reduced before the element's chemistry can
/// happen, so a refused cell beyond these lines is physics, and a refused
/// cell inside them is a genuine hole worth counting.
pub fn outside_water_stability(ph: f64, pe: f64) -> bool {
    pe > (20.75 - ph) + 0.5 || pe < -ph - 0.5
}
