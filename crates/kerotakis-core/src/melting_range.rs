//! Melting RANGES for the materials that have no melting point, and the
//! one honesty rule every solid on this bench obeys when heated past where
//! it melts (reviewed-transition-points lane, 2026-11-11).
//!
//! # Why a range and not a point
//!
//! Paraffin wax is a blend of long-chain alkanes and a polyolefin is a
//! semicrystalline polymer with a distribution of lamella thicknesses.
//! Neither has a sharp melting point: each melts over a range, and the
//! range is a property of the grade. A single curated temperature would be
//! a sharper claim than the material supports, so these rows carry the
//! sourced lower and upper bound and nothing else. The melting-point
//! apparatus reports the range (`PurityVerdict::MeltingRange`), never a
//! point.
//!
//! # Melting through a range pays its latent heat (po-009 / th-004, 2026-11-11)
//!
//! A range row that carries a sourced enthalpy of fusion AND a sourced
//! lower bound melts by [`settle_range_melts`]: the liquid fraction rises
//! linearly from 0 at the lower bound to 1 at the upper bound, and the
//! fusion enthalpy is spread evenly over that span. The vessel's
//! temperature and the solid/liquid split are solved together from one
//! energy balance, so heat put into a wax inside its range goes mostly to
//! melting it and the thermometer rises far less than sensible heat alone
//! would say. Cooling back through the range releases the same heat.
//! Linear distribution is a stated model choice: the real shape of a
//! grade's melting endotherm is not sourced here; only the total and the
//! span are.
//!
//! # The rule ("never silently keep it solid")
//!
//! A solid stands above the upper bound of its sourced melting point or
//! range. If [`crate::phase_route::fusion_enthalpy`] or [`fusion_j_per_mol`]
//! carries a latent heat for it, it has already been melted and paid for,
//! and this module says nothing. Otherwise the bench has no latent heat to
//! pay with, so it cannot model the melt — and it says so with a typed
//! refusal (EN/DE/FR) rather than showing a white solid at 190 °C as
//! though that were a prediction: `not-modeled.melting-not-modelled` for
//! a sharp melting point, `not-modeled.melting-range-latent-heat-unsourced`
//! for a range, which also says the thermometer omits the latent heat.
//!
//! # Semicrystalline polymers: grade latent heat = Xc x reference (2026-11-11)
//!
//! A polymer's enthalpy of fusion is only that of its crystalline part:
//! `dHf(grade) = Xc x dHf(100 %)`, with `Xc` the mass crystallinity of
//! the grade and `dHf(100 %)` the reference for a perfect crystal. Each
//! polymer row records BOTH factors from one sourced DSC measurement of a
//! named grade (its own reference value, because papers differ: PA66 is
//! reported against 188, 196 or 226 J/g), and the product is the latent
//! heat the bench pays. The grade's DSC melting-peak temperature is the
//! lower end of the span the heat is paid over; for PE and PP the upper
//! end is the sourced equilibrium melting temperature of the perfect
//! crystal (no crystal survives above it), and where no equilibrium value
//! is sourced (PET, PA66) the whole latent heat is paid AT the measured
//! peak, as a plateau. Both are stated model choices: the shape of a
//! grade's melting endotherm is not sourced, only its total and its peak.
//!
//! Refused by policy (no row, so never paid): poly(vinyl chloride) (it is
//! nearly amorphous and dehydrochlorinates instead of melting cleanly; no
//! crystallinity of a rigid grade was found in a source), unsintered
//! (virgin) PTFE (first-melt crystallinity differs about twofold from the
//! sintered polymer and no open measurement was found), and any grade
//! whose crystallinity is set by an unstated thermal history. See
//! [`REFUSED_FUSION`]; none of those is in the registry today.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase, SpeciesId};
use crate::vessel::Vessel;

/// A sourced melting range at 101.325 kPa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeltingRange {
    pub species: &'static str,
    /// Lower end of the span the latent heat is paid over, K, where one is
    /// sourced. For a polymer it is a sourced grade's DSC melting peak.
    /// Equal to `upper_k` for a plateau (the whole heat paid at one
    /// temperature). `None` refuses the melt: there is nowhere to pay from.
    pub lower_k: Option<f64>,
    /// Upper bound, K: no crystal of this material survives above it.
    pub upper_k: f64,
    /// Primary citation with DOI, printed by the apparatus.
    pub source: &'static str,
    /// What the range does not claim.
    pub boundary: &'static str,
    /// Specific enthalpy of fusion of the material as a whole, J/kg, where
    /// one is sourced for it. `None` refuses the melt with a typed reason.
    /// For a semicrystalline polymer this is `crystallinity x
    /// fusion_ref_j_per_g x 1000`, checked by a test.
    pub fusion_j_per_kg: Option<f64>,
    /// Mass crystallinity `Xc` (0-1) of the sourced grade, for a
    /// semicrystalline polymer; `None` for a wax, whose latent heat is
    /// measured on the material as a whole.
    pub crystallinity: Option<f64>,
    /// The 100 %-crystalline reference enthalpy of fusion, J/g, that the
    /// source used to compute `crystallinity`.
    pub fusion_ref_j_per_g: Option<f64>,
}

/// A material whose melting latent heat is refused by policy, with why.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefusedFusion {
    pub material: &'static str,
    pub reason: &'static str,
}

/// Materials this bench must never pay a single grade latent heat for.
/// None is in the registry today; the list keeps a future row honest.
pub const REFUSED_FUSION: &[RefusedFusion] = &[
    RefusedFusion {
        material: "PVC",
        reason: "poly(vinyl chloride) is nearly amorphous and dehydrochlorinates on heating instead of melting cleanly; no sourced crystallinity of a rigid grade was found (its 176 J/g 100 % reference, B. Wunderlich, Thermal Analysis, Academic Press 1990, pp. 417-431 as quoted by R. L. Blaine, TA Instruments TN048, would have nothing to multiply)",
    },
    RefusedFusion {
        material: "PTFE (unsintered)",
        reason: "virgin PTFE powder melts first far more crystalline than the sintered polymer (about twofold); only sintered PTFE has an open crystallinity measurement (39.5-43.1 % against 82 J/g, doi:10.3390/polym17101380)",
    },
    RefusedFusion {
        material: "PLA (unstated thermal history)",
        reason: "poly(lactic acid) crystallinity runs from about 0 to 26 % with moulding and annealing history (Ingeo-type PLA plaques 19-25.7 %, doi:10.1021/acsomega.6c03726; a PLA/PCL blend 0.4 %, Europe PMC PMC12473204), so no grade value can be chosen without the history",
    },
];

/// Paraffin wax and the semicrystalline polymers in the registry (PE, PP,
/// PET, nylon 6,6); every row's citation is in its `source`.
///
/// * Paraffin wax: A. Sharma, V. V. Tyagi, C. R. Chen and D. Buddhi,
///   'Review on thermal energy storage with phase change materials and
///   applications', Renew. Sustain. Energy Rev. 13 (2009) 318-345,
///   doi:10.1016/j.rser.2007.10.005, Table 2a: technical-grade paraffins
///   melting between 42-44 and 66-68 degrees Celsius (315.15-341.15 K
///   across the grades), latent heat 189 kJ/kg for five grades (6106,
///   5838, 6035, 6403, 6499). Sharma et al. compile these rows from the
///   primary measurements of A. Abhat, 'Low temperature latent heat
///   thermal energy storage: heat storage materials', Solar Energy 30
///   (1983) 313-332, doi:10.1016/0038-092X(83)90186-X. The order of
///   magnitude is corroborated by the 150-250 kJ/kg quoted for paraffin
///   waxes in the property table of
///   https://en.wikipedia.org/wiki/Phase-change_material, which cites M.
///   Kenisarin and K. Mahkamov, Renew. Sustain. Energy Rev. 11 (2007)
///   1913-1965, doi:10.1016/j.rser.2006.05.005. Spread over the span of the grades
///   (315.15-341.15 K), because the grade in the beaker is not known.
/// * Polyethylene: U. Gaur and B. Wunderlich, J. Phys. Chem. Ref. Data 10
///   (1981) 119, doi:10.1063/1.555636: equilibrium melting temperature
///   414.6 +/- 0.5 K.
/// * Isotactic polypropylene: U. Gaur and B. Wunderlich, J. Phys. Chem.
///   Ref. Data 10 (1981) 1051, doi:10.1063/1.555650: best-estimate
///   equilibrium melting temperature 460.7 K.
/// * Grade latent heats (Xc x 100 % reference, both from the cited DSC
///   paper, all read in Europe PMC full text 2026-11-11): HDPE
///   doi:10.3390/polym18202546 (67.08 % x 293 J/g, peak 138.83 C); iPP
///   doi:10.3390/polym18192351 (51.9 % x 207 J/g, peak 166.5 C); PET
///   doi:10.1038/s41598-026-43815-y (39.4 % x 140 J/g, peak 247.5 C);
///   PA66 doi:10.3390/polym17202753 (31.9 % x 188 J/g, peak ~263 C).
///   The 100 % references 293/207/140 J/g are B. Wunderlich, Thermal
///   Analysis (Academic Press 1990) pp. 417-431, as tabulated by R. L.
///   Blaine, TA Instruments TN048 'Polymer Heats of Fusion'.
pub const MELTING_RANGES: &[MeltingRange] = &[
    MeltingRange {
        species: "paraffin",
        lower_k: Some(315.15),
        upper_k: 341.15,
        source: "A. Sharma, V. V. Tyagi, C. R. Chen and D. Buddhi, 'Review on thermal energy storage with phase change materials and applications', Renew. Sustain. Energy Rev. 13 (2009) 318-345, doi:10.1016/j.rser.2007.10.005, Table 2a: technical-grade paraffin waxes melt between 42-44 and 66-68 degrees Celsius depending on grade; latent heat of fusion 189 kJ/kg for five grades (compiled from A. Abhat, Solar Energy 30 (1983) 313-332, doi:10.1016/0038-092X(83)90186-X)",
        boundary: "commercial paraffin wax is a blend of long-chain alkanes and melts over a range set by its grade; this is the span of the technical grades in the source, not the range of any one candle. The sourced 189 kJ/kg is paid as the wax is heated through that span, spread evenly over it: the total is sourced, the shape of the melting curve is not",
        fusion_j_per_kg: Some(189_000.0),
        crystallinity: None,
        fusion_ref_j_per_g: None,
    },
    MeltingRange {
        species: "PE",
        lower_k: Some(411.98),
        upper_k: 414.6,
        source: "Upper end: U. Gaur and B. Wunderlich, 'Heat capacity and other thermodynamic properties of linear macromolecules. II. Polyethylene', J. Phys. Chem. Ref. Data 10 (1981) 119-152, doi:10.1063/1.555636: equilibrium melting temperature 414.6 +/- 0.5 K. Grade: S. Karagoz et al., 'Hazelnut Shell-Talc-Carbon Black Hybrid High-Density Polyethylene Biocomposites', Polymers 18 (2026), doi:10.3390/polym18202546, Table 6, neat HDPE (PEHSCB0), second heating: dHm 196.54 J/g, Tm 138.83 degrees Celsius, Xc 67.08 % against the 100 % crystalline reference 293 J/g (4.11 kJ/mol CH2: B. Wunderlich, Thermal Analysis, Academic Press 1990, pp. 417-431, as tabulated by R. L. Blaine, TA Instruments TN048 'Polymer Heats of Fusion'). Grade latent heat 0.6708 x 293 J/g = 196.54 J/g",
        boundary: "a real high-density polyethylene melts over a range set by its crystallinity and lamella thickness. The latent heat is that of one sourced HDPE grade (Xc 67.08 % x 293 J/g), paid evenly between that grade's DSC melting peak (138.83 degrees Celsius) and the equilibrium melting temperature of a perfect crystal (141.45 degrees Celsius); the part of a real endotherm below its peak is not modelled, and another grade's crystallinity (LDPE about 31 %) would pay much less",
        fusion_j_per_kg: Some(196_544.4),
        crystallinity: Some(0.6708),
        fusion_ref_j_per_g: Some(293.0),
    },
    MeltingRange {
        species: "PP",
        lower_k: Some(439.65),
        upper_k: 460.7,
        source: "Upper end: U. Gaur and B. Wunderlich, 'Heat capacity and other thermodynamic properties of linear macromolecules. IV. Polypropylene', J. Phys. Chem. Ref. Data 10 (1981) 1051-1064, doi:10.1063/1.555650: best-estimate equilibrium melting temperature of isotactic polypropylene 460.7 K. Grade: Zhu et al., 'Relationships of Multiphase Structure with Flexibility and DC Insulation Performance in Impact-Resistant Polypropylene Copolymers', Polymers 18 (2026), doi:10.3390/polym18192351, homopolymer HICP, second heating, mean of three: Tm 166.5 +/- 0.8 degrees Celsius, dHm 107.4 +/- 1.9 J/g, Xc 51.9 +/- 0.9 % against the 100 % crystalline reference 207 J/g (8.70 kJ/mol: B. Wunderlich, Thermal Analysis, Academic Press 1990, pp. 417-431, as tabulated by R. L. Blaine, TA Instruments TN048). Grade latent heat 0.519 x 207 J/g = 107.43 J/g",
        boundary: "a real isotactic polypropylene melts over a range set by tacticity and crystallinity. The latent heat is that of one sourced homopolymer grade (Xc 51.9 % x 207 J/g), paid evenly between that grade's DSC melting peak (166.5 degrees Celsius) and the equilibrium melting temperature of a perfect crystal (187.55 degrees Celsius); the 100 % reference is disputed (165-209 J/g in use), which changes the reported Xc, not the measured grade heat",
        fusion_j_per_kg: Some(107_433.0),
        crystallinity: Some(0.519),
        fusion_ref_j_per_g: Some(207.0),
    },
    MeltingRange {
        species: "PET",
        lower_k: Some(520.65),
        upper_k: 520.65,
        source: "W. Chaiwutthinan et al., 'Thermomechanical recycling of post-consumer poly(ethylene terephthalate) bottles and post-industrial polyamide 6 fishing nets reinforced with organo-modified montmorillonite', Sci. Rep. 16 (2026), doi:10.1038/s41598-026-43815-y, Table 2, neat post-consumer bottle rPET, second heating: Tm 247.5 degrees Celsius, dHm 55.2 J/g, Xc 39.4 % against the 100 % crystalline reference 140 J/g (26.9 kJ/mol: B. Wunderlich, Thermal Analysis, Academic Press 1990, pp. 417-431, as tabulated by R. L. Blaine, TA Instruments TN048). Grade latent heat 0.394 x 140 J/g = 55.16 J/g",
        boundary: "one sourced bottle-grade PET (Xc 39.4 % x 140 J/g). No equilibrium melting temperature is sourced here, so the whole latent heat is paid at that grade's DSC melting peak (247.5 degrees Celsius) as a plateau; amorphous PET film (Xc near 0) would pay almost nothing",
        fusion_j_per_kg: Some(55_160.0),
        crystallinity: Some(0.394),
        fusion_ref_j_per_g: Some(140.0),
    },
    MeltingRange {
        species: "nylon",
        lower_k: Some(536.15),
        upper_k: 536.15,
        source: "H. Ceviz et al., 'Investigation of Mechanical, Thermal and Microstructural Properties of Waste Micro-Nano Nichrome (NiCr 80/20) Powder-Reinforced Composites with Polyamide 66 (PA66) Matrix', Polymers 17 (2025), doi:10.3390/polym17202753: neat PA66 'exhibits a sharp endothermic peak at approximately 263 degrees Celsius'; dHm 60 J/g, Xc 31.9 % against the 100 % crystalline reference 188 J/g used by that paper (TN048 tabulates 226 J/g from B. Wunderlich, Thermal Analysis, 1990; the reference choice changes Xc, not the measured heat). Grade latent heat 0.319 x 188 J/g = 59.97 J/g",
        boundary: "one sourced nylon 6,6 grade (Xc 31.9 % x 188 J/g). No equilibrium melting temperature is sourced here, so the whole latent heat is paid at that grade's approximate DSC melting peak (263 degrees Celsius) as a plateau",
        fusion_j_per_kg: Some(59_972.0),
        crystallinity: Some(0.319),
        fusion_ref_j_per_g: Some(188.0),
    },
];

/// The sourced melting range of a material, where it has one.
pub fn range(species: &str) -> Option<&'static MeltingRange> {
    MELTING_RANGES.iter().find(|row| row.species == species)
}

/// The temperature above which a solid of this species must no longer be
/// shown as solid: its sourced melting point, or the top of its sourced
/// melting range.
pub fn melted_above_k(species: &str) -> Option<f64> {
    species::lookup(&SpeciesId::new(species))
        .and_then(|data| data.transitions)
        .and_then(|t| t.melting_k)
        .or_else(|| range(species).map(|row| row.upper_k))
}

/// A pair of materials whose melts are sourced as mutually immiscible.
///
/// Melting-point depression is a property of a SOLUTION: a mixture melts
/// below its lowest-melting component only where the components dissolve
/// in each other's melt (a solution or eutectic system). Components that
/// do not mix when molten each melt over their own range, as they would
/// alone, so the depression law must not be claimed for them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImmiscibleMelt {
    pub a: &'static str,
    pub b: &'static str,
    pub source: &'static str,
}

/// * Polyethylene / polypropylene: S. Teh, A. Rudin and J. C. Keung, 'A
///   review of polyethylene-polypropylene blends and their
///   compatibilization', Adv. Polym. Technol. 13 (1994) 1-23,
///   doi:10.1002/adv.1994.060130101 (abstract: "the incompatible nature of
///   polyethylenes and polypropylenes"; blends are phase-separated and
///   show the separate melting endotherms of both components).
pub const IMMISCIBLE_MELTS: &[ImmiscibleMelt] = &[ImmiscibleMelt {
    a: "PE",
    b: "PP",
    source: "S. Teh, A. Rudin and J. C. Keung, 'A review of polyethylene-polypropylene blends and their compatibilization', Adv. Polym. Technol. 13 (1994) 1-23, doi:10.1002/adv.1994.060130101",
}];

/// Whether two materials are sourced as immiscible in the melt.
pub fn immiscible(a: &str, b: &str) -> bool {
    IMMISCIBLE_MELTS
        .iter()
        .any(|row| (row.a == a && row.b == b) || (row.a == b && row.b == a))
}

/// Whether every pair of these components is sourced as immiscible in the
/// melt — the only case in which the melting-point apparatus drops the
/// depression law for a mixture. Fewer than two components is not a blend.
pub fn all_immiscible(components: &[&str]) -> bool {
    components.len() > 1
        && components
            .iter()
            .enumerate()
            .all(|(i, a)| components[i + 1..].iter().all(|b| immiscible(a, b)))
}

/// The enthalpy of fusion of a range material, J/mol, where it can be
/// paid: the row has a sourced specific enthalpy of fusion AND a sourced
/// lower bound to spread it from, and the registry knows the molar mass.
pub fn fusion_j_per_mol(species: &str) -> Option<f64> {
    let row = range(species)?;
    let per_kg = row.fusion_j_per_kg?;
    row.lower_k?;
    let molar_mass = species::lookup(&SpeciesId::new(species))?.molar_mass;
    (molar_mass > 0.0).then(|| per_kg * molar_mass / 1000.0)
}

/// The liquid fraction of a range material at `t_k`: 0 at or below the
/// lower bound, 1 at or above the upper, linear between.
pub fn melted_fraction(row: &MeltingRange, t_k: f64) -> f64 {
    // No lower end, or a plateau (lower end equal to the upper): a step.
    // `settle_range_melts` resolves the partial melt ON a plateau.
    let Some(lower) = row.lower_k.filter(|low| *low < row.upper_k) else {
        return if t_k > row.upper_k { 1.0 } else { 0.0 };
    };
    ((t_k - lower) / (row.upper_k - lower)).clamp(0.0, 1.0)
}

const TRACE: f64 = 1e-12;

/// A row whose whole latent heat is paid at one temperature.
fn is_plateau(row: &MeltingRange) -> bool {
    row.lower_k.is_some_and(|low| low >= row.upper_k)
}

fn moles_in(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn withdraw(vessel: &mut Vessel, key: &str, phase: Phase, moles: f64) {
    let mut remaining = moles;
    for p in vessel.contents.iter_mut() {
        if p.species.0 == key && p.phase == phase && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = crate::units::Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    vessel.contents.retain(|p| p.moles.0 > 1e-15);
}

/// Melt (or freeze) every range material with a payable enthalpy of
/// fusion to the state the vessel's energy allows, paying the latent heat
/// out of (or back into) the vessel's sensible heat.
///
/// The vessel arrives at `T0` with each material at liquid fraction `f0`
/// (whatever the last step left). The new temperature `T` solves
///
/// ```text
/// H_sensible(T0 -> T) + sum_i L_i n_i (f_i(T) - f0_i) = 0
/// ```
///
/// which is monotone increasing in `T` (every heat capacity is positive
/// and every `f_i` is non-decreasing), so it is bisected on the bracket
/// `[min(T0, lowest lower bound), max(T0, highest upper bound)]`, on which
/// it changes sign. The split is then set to `f_i(T)`.
///
/// The sensible term is integrated over the phases the vessel holds on
/// arrival; the solid/liquid heat-capacity difference over the moved
/// fraction is a second-order term this does not iterate on.
pub fn settle_range_melts(vessel: &mut Vessel) -> Vec<Event> {
    struct Melter {
        key: String,
        row: &'static MeltingRange,
        latent: f64,
        total: f64,
        liquid0: f64,
    }
    let mut melters: Vec<Melter> = Vec::new();
    for portion in &vessel.contents {
        if !matches!(portion.phase, Phase::Solid | Phase::Liquid) || portion.moles.0 <= TRACE {
            continue;
        }
        let key = portion.species.0.as_str();
        if melters.iter().any(|m| m.key == key) {
            continue;
        }
        let (Some(row), Some(latent)) = (range(key), fusion_j_per_mol(key)) else {
            continue;
        };
        let liquid0 = moles_in(vessel, key, Phase::Liquid);
        let total = moles_in(vessel, key, Phase::Solid) + liquid0;
        melters.push(Melter {
            key: key.to_string(),
            row,
            latent,
            total,
            liquid0,
        });
    }
    if melters.is_empty() || vessel.heat_capacity() <= 0.0 {
        return Vec::new();
    }
    let t0 = vessel.temperature.0;
    let balance = |t: f64| -> f64 {
        vessel.energy_between(t0, t)
            + melters
                .iter()
                .map(|m| m.latent * (m.total * melted_fraction(m.row, t) - m.liquid0))
                .sum::<f64>()
    };
    let at_t0 = balance(t0);
    // Already consistent: nothing to melt or freeze at this temperature.
    let scale: f64 = melters.iter().map(|m| m.latent * m.total).sum();
    if at_t0.abs() <= 1e-9 * scale.max(1.0) {
        return Vec::new();
    }
    let mut lo = melters
        .iter()
        .filter_map(|m| m.row.lower_k)
        .fold(t0, f64::min);
    let mut hi = melters.iter().map(|m| m.row.upper_k).fold(t0, f64::max);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if balance(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
        if hi - lo <= 1e-10 {
            break;
        }
    }
    let mut t1 = (0.5 * (lo + hi)).max(0.0);

    // A plateau row (PET, PA66) melts in a step, so the balance jumps
    // there and the bisection lands ON the plateau with energy left over.
    // Pin the vessel to the plateau and melt the fraction that energy pays
    // for, shared in proportion to each plateau material's total heat.
    let mut fractions: Vec<f64> = melters.iter().map(|m| melted_fraction(m.row, t1)).collect();
    let plateau = melters
        .iter()
        .find(|m| is_plateau(m.row) && (t1 - m.row.upper_k).abs() <= 1e-6)
        .map(|m| m.row.upper_k);
    if let Some(point) = plateau {
        t1 = point;
        let pinned: Vec<usize> = (0..melters.len())
            .filter(|&i| is_plateau(melters[i].row) && melters[i].row.upper_k == point)
            .collect();
        for (i, m) in melters.iter().enumerate() {
            fractions[i] = melted_fraction(m.row, t1);
        }
        for &i in &pinned {
            fractions[i] = 0.0;
        }
        let residual = vessel.energy_between(t0, t1)
            + melters
                .iter()
                .zip(&fractions)
                .map(|(m, f)| m.latent * (m.total * f - m.liquid0))
                .sum::<f64>();
        let capacity: f64 = pinned
            .iter()
            .map(|&i| melters[i].latent * melters[i].total)
            .sum();
        let share = if capacity > 0.0 {
            (-residual / capacity).clamp(0.0, 1.0)
        } else {
            0.0
        };
        for &i in &pinned {
            fractions[i] = share;
        }
    }

    let mut events = Vec::new();
    for (m, fraction) in melters.iter().zip(&fractions) {
        let target = m.total * fraction;
        // The moved fraction changed phase INSIDE the material's range, not
        // at wherever the vessel ended up: wax cooled from 48 °C to 20 °C
        // finished freezing at the bottom of its range, not at 20 °C
        // (th-004). The event reports the vessel temperature clamped into
        // the range, which is the last temperature at which this linear
        // model moves any of it.
        let at =
            crate::units::Kelvin(t1.clamp(m.row.lower_k.unwrap_or(m.row.upper_k), m.row.upper_k));
        let delta = target - m.liquid0;
        let id = SpeciesId::new(&m.key);
        if delta > TRACE {
            let n = delta.min(moles_in(vessel, &m.key, Phase::Solid));
            withdraw(vessel, &m.key, Phase::Solid, n);
            vessel.deposit(id.clone(), crate::units::Moles(n), Phase::Liquid);
            events.push(Event::state_changed(
                vessel.id,
                id,
                Phase::Solid,
                Phase::Liquid,
                at,
                0.0,
                crate::units::Moles(n),
            ));
        } else if delta < -TRACE {
            let n = (-delta).min(moles_in(vessel, &m.key, Phase::Liquid));
            withdraw(vessel, &m.key, Phase::Liquid, n);
            vessel.deposit(id.clone(), crate::units::Moles(n), Phase::Solid);
            events.push(Event::state_changed(
                vessel.id,
                id,
                Phase::Liquid,
                Phase::Solid,
                at,
                0.0,
                crate::units::Moles(n),
            ));
        }
    }
    let from = vessel.temperature;
    if (t1 - from.0).abs() > 1e-9 {
        let to = crate::units::Kelvin(t1);
        vessel.temperature = to;
        vessel.refresh_pressure();
        events.push(Event::TemperatureChanged {
            vessel: vessel.id,
            from,
            to,
        });
    }
    events
}

/// A solid standing above where it melts with no latent heat on the bench
/// to melt it with: the species key, the temperature it melts above (K),
/// and whether that is a sharp melting point (`true`) or the top of a
/// melting range (`false`).
#[derive(Debug, Clone, PartialEq)]
pub struct UnpaidMelt {
    pub species: String,
    pub melts_above_k: f64,
    pub sharp_point: bool,
}

/// Every solid species in `vessel` that stands above its sourced melting
/// point or range with no latent heat the bench can pay: neither
/// [`crate::phase_route::fusion_enthalpy`] nor [`fusion_j_per_mol`] has a
/// value for it. Water's ice is owned by the solvent model and skipped.
///
/// This one predicate drives both the step-level refusal
/// ([`unmodelled_melts`]) and the inline qualifier on a thermometer reading
/// ([`thermometer_qualifier`]), so the two can never disagree.
pub fn unpaid_melts(vessel: &Vessel) -> Vec<UnpaidMelt> {
    let now = vessel.temperature.0;
    let mut out: Vec<UnpaidMelt> = Vec::new();
    for portion in &vessel.contents {
        if portion.phase != Phase::Solid || portion.moles.0 <= 1e-12 {
            continue;
        }
        let key = portion.species.0.as_str();
        if out.iter().any(|u| u.species == key) {
            continue;
        }
        // A latent heat on the bench means the phase route melts it; water's
        // ice belongs to the solvent model (`solve::StateEquilibrator`).
        if key == "water"
            || crate::phase_route::fusion_enthalpy(key).is_some()
            || fusion_j_per_mol(key).is_some()
        {
            continue;
        }
        let Some(melts) = melted_above_k(key) else {
            continue;
        };
        if now <= melts {
            continue;
        }
        let sharp_point = species::lookup(&SpeciesId::new(key))
            .and_then(|data| data.transitions)
            .and_then(|t| t.melting_k)
            .is_some();
        out.push(UnpaidMelt {
            species: key.to_string(),
            melts_above_k: melts,
            sharp_point,
        });
    }
    out
}

/// One typed refusal per solid species standing above where it melts
/// without a latent heat to melt it with.
pub fn unmodelled_melts(vessel: &Vessel) -> Vec<Event> {
    let mut events = Vec::new();
    for unpaid in unpaid_melts(vessel) {
        let slots = vec![
            (
                "name".to_string(),
                Slot::term("species", species::english_name(&unpaid.species)),
            ),
            (
                "melting".to_string(),
                Slot::number(format!("{:.0}", unpaid.melts_above_k - 273.15)),
            ),
        ];
        // A sharp melting point and a melting range say different things:
        // past a range, the thermometer has also skipped a latent heat
        // whose size this bench does not know.
        let phrase = if unpaid.sharp_point {
            Phrase::new(
                "not-modeled.melting-not-modelled",
                "{name} heated above its melting point ({melting} °C): melting is not modelled here, so the solid shown is not a prediction that it stays solid",
                slots,
            )
        } else {
            Phrase::new(
                "not-modeled.melting-range-latent-heat-unsourced",
                "{name} heated past its melting range (up to {melting} °C): its enthalpy of fusion is not sourced, so the latent heat is not paid — the temperature shown overstates the warming, and the solid shown is not a prediction that it stays solid",
                slots,
            )
        };
        events.push(Event::not_modeled(
            vessel.id,
            NotModelledCause::NoReviewedDatum,
            phrase,
        ));
    }
    events
}

/// The inline qualifier a thermometer reading carries when the vessel's
/// heat accounting skipped a latent heat (po-009, 2026-11-11).
///
/// A solid standing above where it melts, with no enthalpy of fusion on the
/// bench, was warmed by sensible heat alone: the energy a real sample
/// would have spent melting went into the thermometer instead. The number
/// is therefore an upper bound on the warming, not a prediction, and the
/// reading says so on itself rather than relying on a refusal filed by an
/// earlier step. `None` when every solid's melt is paid (or none stands
/// above where it melts): the reading is then unqualified.
pub fn thermometer_qualifier(vessel: &Vessel) -> Option<Phrase> {
    let unpaid = unpaid_melts(vessel);
    if unpaid.is_empty() {
        return None;
    }
    let names: Vec<String> = unpaid
        .iter()
        .map(|u| species::english_name(&u.species).to_string())
        .collect();
    Some(Phrase::new(
        "measured.thermometer-latent-heat-unpaid",
        "{names} stands above where it melts and no enthalpy of fusion was paid for it: this temperature counts sensible heat only, so it overstates the warming and is not a prediction of what a real thermometer would read",
        vec![("names".to_string(), Slot::terms("species", names))],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_range_is_ordered_and_cited() {
        for row in MELTING_RANGES {
            assert!(
                row.lower_k.is_none_or(|low| low <= row.upper_k),
                "{}",
                row.species
            );
            assert!(row.source.contains("doi"), "{} has no DOI", row.species);
            if let Some(per_kg) = row.fusion_j_per_kg {
                assert!(per_kg > 0.0, "{}", row.species);
                assert!(
                    row.source.contains("kJ/kg") || row.source.contains("J/g"),
                    "{}: latent heat uncited",
                    row.species
                );
            }
            // A polymer's grade heat is exactly Xc x its recorded reference.
            match (row.crystallinity, row.fusion_ref_j_per_g) {
                (Some(xc), Some(reference)) => {
                    assert!(xc > 0.0 && xc < 1.0, "{}", row.species);
                    let per_kg = row.fusion_j_per_kg.expect("a grade heat");
                    let expected = xc * reference * 1000.0;
                    assert!(
                        (per_kg - expected).abs() <= 1e-6 * expected,
                        "{}: {per_kg} vs {expected}",
                        row.species
                    );
                }
                (None, None) => {}
                _ => panic!("{}: Xc and its reference come together", row.species),
            }
            assert!(
                species::lookup(&SpeciesId::new(row.species)).is_some(),
                "{} is not in the registry",
                row.species
            );
        }
    }

    #[test]
    fn every_immiscible_pair_is_cited_and_known() {
        for row in IMMISCIBLE_MELTS {
            assert!(row.source.contains("doi"), "{}/{}", row.a, row.b);
            assert!(range(row.a).is_some() || species::lookup(&SpeciesId::new(row.a)).is_some());
            assert!(range(row.b).is_some() || species::lookup(&SpeciesId::new(row.b)).is_some());
            assert!(immiscible(row.b, row.a), "symmetric");
        }
    }
}
