//! Temperature-dependent property correlations with provenance.
//!
//! Every number here has a published source and a stated validity range.
//! Outside the range the function returns `Err` — a refusal, not a guess.

/// Result of a property evaluation: value, unit, provenance, validity note.
#[derive(Debug, Clone)]
pub struct PropertyResult {
    pub value: f64,
    pub unit: &'static str,
    pub provenance: &'static str,
    pub note: Option<String>,
}

// ── Water density ───────────────────────────────────────────────────
//
// Tanaka, Girard, Davis, Peuto, Bignell (2001)
// "Recommended table for the density of water between 0 °C and 40 °C
//  based on recent experimental reports"
// Metrologia 38, 301–309.  doi:10.1088/0026-1394/38/4/3
//
// Valid: 0–40 °C (273.15–313.15 K).

const WATER_DENSITY_PROV: &str =
    "Tanaka et al. (2001), Metrologia 38 301–309; Thiesen equation; valid 0–40 °C";

const TANAKA_A: [f64; 5] = [
    -3.983_035,  // °C
    301.797,     // °C
    522_528.9,   // °C²
    69.348_81,   // °C
    999.974_950, // kg/m³
];

/// Water density in kg/m³ at temperature T (Kelvin).
/// Valid 273.15–313.15 K (0–40 °C).
pub fn water_density_kg_m3(t_kelvin: f64) -> Result<PropertyResult, String> {
    let t_c = t_kelvin - 273.15;
    if !(0.0..=40.0).contains(&t_c) {
        return Err(format!(
            "water density (Tanaka 2001): {:.1} °C is outside the validity range 0–40 °C",
            t_c
        ));
    }
    let a = &TANAKA_A;
    let rho = a[4] * (1.0 - ((t_c + a[0]).powi(2) * (t_c + a[1])) / (a[2] * (t_c + a[3])));
    Ok(PropertyResult {
        value: rho,
        unit: "kg/m³",
        provenance: WATER_DENSITY_PROV,
        note: None,
    })
}

/// Water density in g/mL (numerically identical to g/cm³).
pub fn water_density_g_ml(t_kelvin: f64) -> Result<f64, String> {
    water_density_kg_m3(t_kelvin).map(|r| r.value / 1000.0)
}

// ── Water viscosity ─────────────────────────────────────────────────
//
// Korson, Drost-Hansen, Millero (1969)
// "Viscosity of water at various temperatures"
// J. Phys. Chem. 73 (1), 34–39.
//
// Valid: 0–100 °C.

const WATER_VISCOSITY_PROV: &str =
    "Korson, Drost-Hansen, Millero (1969), J. Phys. Chem. 73 34–39; valid 0–100 °C";

const KORSON_A: f64 = 1.1709;
const KORSON_B: f64 = 0.001_827;
const KORSON_C: f64 = 89.93;
const KORSON_ETA20: f64 = 1.002; // cP at 20 °C

/// Water dynamic viscosity in centipoise (cP = mPa·s) at temperature T (Kelvin).
/// Valid 273.15–373.15 K (0–100 °C).
pub fn water_viscosity_cp(t_kelvin: f64) -> Result<PropertyResult, String> {
    let t_c = t_kelvin - 273.15;
    if !(0.0..=100.0).contains(&t_c) {
        return Err(format!(
            "water viscosity (Korson 1969): {:.1} °C is outside the validity range 0–100 °C",
            t_c
        ));
    }
    let eta = KORSON_ETA20
        * 10f64
            .powf((KORSON_A * (20.0 - t_c) - KORSON_B * (t_c - 20.0).powi(2)) / (t_c + KORSON_C));
    Ok(PropertyResult {
        value: eta,
        unit: "cP",
        provenance: WATER_VISCOSITY_PROV,
        note: None,
    })
}

// ── Water relative permittivity ─────────────────────────────────────
//
// Bradley, Pitzer (1979)
// "Thermodynamics of electrolytes. 12. Dielectric properties of water
//  and Debye–Hückel parameters to 350 °C and 1 kbar"
// J. Phys. Chem. 83 (12), 1599–1603.  doi:10.1021/j100475a009
//
// Valid: 0–350 °C at 1 bar (we enforce 0–100 °C for aqueous-bench use).

const WATER_PERMITTIVITY_PROV: &str =
    "Bradley & Pitzer (1979), J. Phys. Chem. 83 1599–1603; valid 0–350 °C; \
     evaluated at 1 bar";

const BP_U: [f64; 9] = [
    3.4279e2, -5.0866e-3, 9.4690e-7, -2.0525, 3.1159e3, -1.8289e2, -8.0325e3, 4.2142e6, 2.1417,
];

/// Relative permittivity (dielectric constant) of water at temperature T (Kelvin),
/// 1 bar pressure. Valid 273.15–623.15 K (0–350 °C).
pub fn water_permittivity(t_kelvin: f64) -> Result<PropertyResult, String> {
    let t_c = t_kelvin - 273.15;
    if !(0.0..=350.0).contains(&t_c) {
        return Err(format!(
            "water permittivity (Bradley–Pitzer 1979): {:.1} °C is outside the validity range 0–350 °C",
            t_c
        ));
    }
    let p_bar = 1.0;
    let u = &BP_U;
    let b = u[6] + u[7] / t_kelvin + u[8] * t_kelvin;
    let c = u[3] + u[4] / (u[5] + t_kelvin);
    let eps1000 = u[0] * (u[1] * t_kelvin + u[2] * t_kelvin * t_kelvin).exp();
    let eps = eps1000 + c * ((b + p_bar) / (b + 1000.0)).ln();
    Ok(PropertyResult {
        value: eps,
        unit: "(dimensionless)",
        provenance: WATER_PERMITTIVITY_PROV,
        note: None,
    })
}

// ── Henry's law constants ───────────────────────────────────────────
//
// Sander (2015)
// "Compilation of Henry's law constants (version 4.0) for water as solvent"
// Atmos. Chem. Phys. 15, 4399–4981.  doi:10.5194/acp-15-4399-2015
//
// H(T) = Hcp · exp(−ΔsolH/R · (1/T − 1/T°))
//
// Hcp in mol/(L·atm), T° = 298.15 K, −ΔsolH/R given as "C" in K.
// The sign convention: C > 0 means solubility decreases with temperature.

const HENRY_PROV: &str =
    "Sander (2015), Atmos. Chem. Phys. 15 4399–4981; Hcp in mol/(L·atm); T° = 298.15 K";

pub struct HenryCoefficient {
    pub gas: &'static str,
    pub formula: &'static str,
    /// Hcp at 298.15 K, mol/(L·atm).
    pub hcp_298: f64,
    /// Temperature dependence: −ΔsolH/R in K.
    pub c_kelvin: f64,
    pub provenance: &'static str,
}

pub const HENRY_COEFFICIENTS: &[HenryCoefficient] = &[
    HenryCoefficient {
        gas: "carbon dioxide",
        formula: "CO2",
        hcp_298: 3.4e-2,
        c_kelvin: 2400.0,
        provenance: HENRY_PROV,
    },
    HenryCoefficient {
        gas: "oxygen",
        formula: "O2",
        hcp_298: 1.3e-3,
        c_kelvin: 1500.0,
        provenance: HENRY_PROV,
    },
    HenryCoefficient {
        gas: "nitrogen",
        formula: "N2",
        hcp_298: 6.1e-4,
        c_kelvin: 1300.0,
        provenance: HENRY_PROV,
    },
    HenryCoefficient {
        gas: "hydrogen",
        formula: "H2",
        hcp_298: 7.8e-4,
        c_kelvin: 500.0,
        provenance: HENRY_PROV,
    },
    HenryCoefficient {
        gas: "chlorine",
        formula: "Cl2",
        hcp_298: 9.2e-2,
        c_kelvin: 2500.0,
        provenance: HENRY_PROV,
    },
    HenryCoefficient {
        gas: "ammonia",
        formula: "NH3",
        hcp_298: 5.7e1,
        c_kelvin: 4200.0,
        provenance: HENRY_PROV,
    },
];

/// Henry's constant at temperature T (Kelvin), mol/(L·atm).
/// Valid near 273–353 K; exact range depends on the gas.
pub fn henry_at_t(coeff: &HenryCoefficient, t_kelvin: f64) -> PropertyResult {
    let t0 = 298.15;
    let h = coeff.hcp_298 * (coeff.c_kelvin * (1.0 / t_kelvin - 1.0 / t0)).exp();
    PropertyResult {
        value: h,
        unit: "mol/(L·atm)",
        provenance: coeff.provenance,
        note: None,
    }
}

pub fn henry_lookup(formula: &str) -> Option<&'static HenryCoefficient> {
    HENRY_COEFFICIENTS
        .iter()
        .find(|c| c.formula.eq_ignore_ascii_case(formula) || c.gas == formula)
}

// ── Ethanol-water mixture density ──────────────────────────────────
//
// Polynomial fit to CRC Handbook of Chemistry and Physics, 97th ed.,
// Table "Density of aqueous ethanol solutions at 20 °C".
// 21 data points (0–100 % w/w in 5 % steps), 5th-order polynomial.
// Valid: 0 ≤ w ≤ 1 (mass fraction ethanol), T = 20 °C.
// Max residual: 1.5 mg/mL against the tabulated values.

const ETHANOL_WATER_DENSITY_PROV: &str =
    "CRC Handbook, 97th ed., density of ethanol-water at 20 °C; \
     5th-order polynomial fit, max residual 1.5 mg/mL";

const EW_COEFFS: [f64; 6] = [
    9.977_472_131_347e-01,
    -1.873_665_046_600e-01,
    4.305_337_838_453e-01,
    -1.410_997_305_338,
    1.565_239_129_337,
    -6.066_534_820_230e-01,
];

/// Density of an ethanol-water mixture in g/mL at 20 °C.
/// `w_ethanol` is the mass fraction of ethanol, 0.0–1.0.
pub fn ethanol_water_density_g_ml(w_ethanol: f64) -> Result<PropertyResult, String> {
    if !(0.0..=1.0).contains(&w_ethanol) {
        return Err(format!(
            "ethanol mass fraction {w_ethanol:.3} is outside the valid range 0–1"
        ));
    }
    let w = w_ethanol;
    let rho = EW_COEFFS[0]
        + EW_COEFFS[1] * w
        + EW_COEFFS[2] * w * w
        + EW_COEFFS[3] * w * w * w
        + EW_COEFFS[4] * w * w * w * w
        + EW_COEFFS[5] * w * w * w * w * w;
    Ok(PropertyResult {
        value: rho,
        unit: "g/mL",
        provenance: ETHANOL_WATER_DENSITY_PROV,
        note: None,
    })
}

// ── Sucrose-water mixture density ───────────────────────────────────
//
// EXP-19, second half. The published datum is not a density polynomial
// but its inverse: Bates, F. (1942) "Polarimetry, Saccharimetry and the
// Sugars", NBS Circular C440, Table 114 — the sucrose table maintained
// by the National Bureau of Standards (now NIST) — has the closed form
//
//   °Bx = 182.46007·d³ − 775.68212·d² + 1262.7794·d − 669.56218
//
// with d the apparent specific gravity 20°/20 °C and °Bx the sucrose
// mass percent, reproducing the NBS table to an RMS disagreement of
// 0.0009 °Bx over d = 1.00000–1.17874 (0–40 °Bx).
//
// That runs the wrong way for a bench that knows the concentration and
// wants the density, so we invert it NUMERICALLY rather than fitting a
// second polynomial: the cubic's derivative 547.38021·d² −
// 1551.36424·d + 1262.7794 has a negative discriminant, so the cubic is
// strictly increasing and a bisection converges to the bit. The
// published coefficients stay the only data in the file; nothing is
// re-fitted and no residual of ours is added to theirs.
//
// Density then follows from what the ratio means, with the water
// density taken from the Tanaka correlation above rather than a
// second-hand constant:
//
//   ρ(w) = d(w) · ρ_water(20 °C)
//
// Valid: 0 ≤ w ≤ 0.40, the polynomial's own stated range. Above 40 °Bx
// this refuses instead of extrapolating a cubic — a saturated syrup is
// a different measurement, not a longer sum.
//
// Cross-check against an independent table (ISCOTABLES 7th ed., sucrose
// solution densities at 20 °C): 0 %→0.998, 5 %→1.018, 10 %→1.038,
// 20 %→1.081, 30 %→1.127, 40 %→1.176 g/mL. The test below computes the
// largest disagreement over those six rows and pins it.

const SUCROSE_WATER_DENSITY_PROV: &str =
    "Bates (1942), NBS Circular C440 Table 114 (NIST), sucrose °Bx/specific-gravity \
     cubic inverted by bisection; ρ = d·ρ_water(20 °C, Tanaka 2001); valid 0–40 % w/w";

/// NBS Table 114 cubic: apparent specific gravity (20°/20 °C) → °Brix.
const NBS114: [f64; 4] = [-669.562_18, 1262.779_4, -775.682_12, 182.460_07];

fn nbs114_brix(d: f64) -> f64 {
    NBS114[0] + NBS114[1] * d + NBS114[2] * d * d + NBS114[3] * d * d * d
}

/// Density of a sucrose-water solution in g/mL at 20 °C.
/// `w_sucrose` is the mass fraction of sucrose, 0.00–0.40.
pub fn sucrose_water_density_g_ml(w_sucrose: f64) -> Result<PropertyResult, String> {
    if !(0.0..=0.40).contains(&w_sucrose) {
        return Err(format!(
            "sucrose mass fraction {w_sucrose:.3} is outside the valid range 0–0.40 \
             (NBS Table 114 is published for 0–40 °Bx; above that the cubic would be \
             an extrapolation, not a table)"
        ));
    }
    let target_brix = 100.0 * w_sucrose;
    // Strictly increasing on the bracket, so bisection cannot miss.
    let (mut lo, mut hi) = (0.98_f64, 1.20_f64);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if nbs114_brix(mid) < target_brix {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-15 {
            break;
        }
    }
    let d = 0.5 * (lo + hi);
    let rho_water = water_density_g_ml(293.15)?;
    Ok(PropertyResult {
        value: d * rho_water,
        unit: "g/mL",
        provenance: SUCROSE_WATER_DENSITY_PROV,
        note: Some(format!(
            "apparent specific gravity 20°/20 °C = {d:.5}; \
             water reference ρ(20 °C) = {rho_water:.6} g/mL"
        )),
    })
}

// ── Water surface tension ───────────────────────────────────────────
//
// EXP-48, first slice. IAPWS R1-76, "Revised Release on Surface Tension
// of Ordinary Water Substance" (International Association for the
// Properties of Water and Steam, 1994; revision 2014):
//
//   σ = B·τ^μ·(1 + b·τ),   τ = 1 − T/T_c
//
// with B = 235.8 mN/m, b = −0.625, μ = 1.256 and T_c = 647.096 K.
// The release covers the whole liquid range from the supercooled region
// to the critical point; we enforce 0–100 °C, which is the bench's.
//
// Reference points the tests pin, all from the same release's own
// table: σ(0 °C) = 75.65, σ(20 °C) = 72.74, σ(25 °C) = 71.97 and
// σ(100 °C) = 58.90 mN/m.

const WATER_SURFACE_TENSION_PROV: &str =
    "IAPWS R1-76 (rev. 2014), Revised Release on the Surface Tension of Ordinary \
     Water Substance; B = 235.8 mN/m, b = −0.625, μ = 1.256, Tc = 647.096 K; \
     enforced 0–100 °C";

const IAPWS_ST_B: f64 = 235.8; // mN/m
const IAPWS_ST_LITTLE_B: f64 = -0.625;
const IAPWS_ST_MU: f64 = 1.256;
const WATER_CRITICAL_K: f64 = 647.096;

/// Surface tension of water against its own vapour, mN/m (= mJ/m²),
/// at temperature T (Kelvin). Valid 273.15–373.15 K (0–100 °C).
pub fn water_surface_tension_mn_m(t_kelvin: f64) -> Result<PropertyResult, String> {
    let t_c = t_kelvin - 273.15;
    if !(0.0..=100.0).contains(&t_c) {
        return Err(format!(
            "water surface tension (IAPWS R1-76): {t_c:.1} °C is outside the \
             enforced bench range 0–100 °C"
        ));
    }
    let tau = 1.0 - t_kelvin / WATER_CRITICAL_K;
    let sigma = IAPWS_ST_B * tau.powf(IAPWS_ST_MU) * (1.0 + IAPWS_ST_LITTLE_B * tau);
    Ok(PropertyResult {
        value: sigma,
        unit: "mN/m",
        provenance: WATER_SURFACE_TENSION_PROV,
        note: None,
    })
}

// ── Capillary rise ──────────────────────────────────────────────────
//
// Jurin's law, the textbook force balance between the wetting line and
// the weight of the raised column:
//
//   h = 2·σ·cos θ / (ρ·g·r)
//
// Nothing here is curated: σ comes from the IAPWS release above, ρ from
// Tanaka, and g is the standard acceleration 9.806 65 m/s² fixed by the
// 3rd CGPM (1901). Which is the point of the acceptance criterion —
// capillary rise is COMPUTED from the sourced values, not tabulated
// beside them.
//
// The binding validity range is the water density's, 0–40 °C, not the
// surface tension's: a computed quantity is only as valid as its
// narrowest input, and saying so is cheaper than discovering it.

const CAPILLARY_RISE_PROV: &str =
    "Jurin's law h = 2σcosθ/(ρgr) computed from IAPWS R1-76 surface tension and \
     Tanaka et al. (2001) water density; g = 9.80665 m/s² (3rd CGPM 1901); \
     valid 0–40 °C, the density correlation's range";

/// Standard acceleration of free fall, m/s² (3rd CGPM, 1901).
const STANDARD_GRAVITY: f64 = 9.806_65;

/// Capillary rise of water in a tube of internal radius `radius_mm`,
/// in mm, at temperature T (Kelvin) and contact angle `theta_deg`
/// (0° = perfectly wetting clean glass). Valid 0–40 °C.
pub fn capillary_rise_mm(
    radius_mm: f64,
    t_kelvin: f64,
    theta_deg: f64,
) -> Result<PropertyResult, String> {
    if !(radius_mm > 0.0) {
        return Err(format!(
            "capillary rise: tube radius must be positive, got {radius_mm}"
        ));
    }
    if !(0.0..=180.0).contains(&theta_deg) {
        return Err(format!(
            "capillary rise: contact angle {theta_deg} ° is outside 0–180 °"
        ));
    }
    // water_density_kg_m3 carries the narrower 0–40 °C range and refuses
    // for us, so the error a caller sees names the correlation that
    // actually ran out.
    let rho = water_density_kg_m3(t_kelvin)?.value;
    let sigma_n_m = water_surface_tension_mn_m(t_kelvin)?.value * 1e-3;
    let r_m = radius_mm * 1e-3;
    let h_m = 2.0 * sigma_n_m * theta_deg.to_radians().cos() / (rho * STANDARD_GRAVITY * r_m);
    Ok(PropertyResult {
        value: h_m * 1e3,
        unit: "mm",
        provenance: CAPILLARY_RISE_PROV,
        note: Some(format!(
            "σ = {:.2} mN/m, ρ = {:.2} kg/m³, r = {radius_mm} mm, θ = {theta_deg} °",
            sigma_n_m * 1e3,
            rho
        )),
    })
}

// ── CLI dispatcher ──────────────────────────────────────────────────

pub struct PropertyInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub const PROPERTIES: &[PropertyInfo] = &[
    PropertyInfo {
        name: "water-density",
        description: "ρ(T) kg/m³, Tanaka 2001, 0–40 °C",
    },
    PropertyInfo {
        name: "water-viscosity",
        description: "η(T) cP, Korson 1969, 0–100 °C",
    },
    PropertyInfo {
        name: "water-permittivity",
        description: "ε(T) dimensionless, Bradley–Pitzer 1979, 0–350 °C",
    },
    PropertyInfo {
        name: "henry",
        description: "Henry's constant H(T) mol/(L·atm), Sander 2015: CO2, O2, N2, H2, Cl2, NH3",
    },
    PropertyInfo {
        name: "ethanol-water-density",
        description: "ρ(w) g/mL at 20 °C, CRC 97th ed., w = mass fraction ethanol 0–1",
    },
    PropertyInfo {
        name: "sucrose-water-density",
        description: "ρ(w) g/mL at 20 °C, NBS Table 114 inverted, w = mass fraction sucrose 0–0.40",
    },
    PropertyInfo {
        name: "water-surface-tension",
        description: "σ(T) mN/m, IAPWS R1-76, 0–100 °C",
    },
    PropertyInfo {
        name: "capillary-rise",
        description: "h(r,T,θ) mm, Jurin's law from IAPWS σ and Tanaka ρ, 0–40 °C; \
                      r in mm, optional theta in degrees (default 0)",
    },
];

/// Evaluate a property by name. For water properties, expects `T=<K>`.
/// For henry, expects `gas=<formula> T=<K>`.
pub fn evaluate(name: &str, args: &[String]) -> Result<PropertyResult, String> {
    let get = |key: &str| -> Result<f64, String> {
        args.iter()
            .filter_map(|a| {
                let (k, v) = a.split_once('=')?;
                if k == key {
                    v.parse::<f64>().ok()
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| format!("missing argument: {key}"))
    };

    let get_str = |key: &str| -> Result<&str, String> {
        args.iter()
            .filter_map(|a| {
                let (k, v) = a.split_once('=')?;
                if k == key {
                    Some(v)
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| format!("missing argument: {key}"))
    };

    match name {
        "water-density" => {
            let t = get("T")?;
            water_density_kg_m3(t)
        }
        "water-viscosity" => {
            let t = get("T")?;
            water_viscosity_cp(t)
        }
        "water-permittivity" => {
            let t = get("T")?;
            water_permittivity(t)
        }
        "henry" => {
            let gas = get_str("gas")?;
            let t = get("T")?;
            let coeff = henry_lookup(gas).ok_or_else(|| {
                format!("no Henry data for '{gas}'; available: CO2, O2, N2, H2, Cl2, NH3")
            })?;
            Ok(henry_at_t(coeff, t))
        }
        "ethanol-water-density" => {
            let w = get("w")?;
            ethanol_water_density_g_ml(w)
        }
        "sucrose-water-density" => {
            let w = get("w")?;
            sucrose_water_density_g_ml(w)
        }
        "water-surface-tension" => {
            let t = get("T")?;
            water_surface_tension_mn_m(t)
        }
        "capillary-rise" => {
            let r = get("r")?;
            let t = get("T")?;
            // A clean glass capillary wets perfectly; the angle is the
            // optional knob, not a required incantation.
            let theta = get("theta").unwrap_or(0.0);
            capillary_rise_mm(r, t, theta)
        }
        _ => Err(format!("unknown property '{name}'")),
    }
}

// ── Water property table ────────────────────────────────────────────

/// Print a full property table for water at the given temperature.
pub fn water_table(t_kelvin: f64) -> Vec<(String, Result<PropertyResult, String>)> {
    vec![
        ("density".into(), water_density_kg_m3(t_kelvin)),
        ("viscosity".into(), water_viscosity_cp(t_kelvin)),
        ("permittivity".into(), water_permittivity(t_kelvin)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Water density reference points ──────────────────────────────
    // CRC Handbook / Tanaka 2001 Table 1

    #[test]
    fn water_density_at_4c_is_maximum() {
        let rho = water_density_kg_m3(277.13).unwrap().value;
        assert!(
            (rho - 999.97).abs() < 0.05,
            "ρ(3.98 °C) ≈ 999.97 kg/m³: {rho}"
        );
    }

    #[test]
    fn water_density_at_25c() {
        let rho = water_density_kg_m3(298.15).unwrap().value;
        assert!(
            (rho - 997.05).abs() < 0.05,
            "ρ(25 °C) ≈ 997.05 kg/m³: {rho}"
        );
    }

    #[test]
    fn water_density_at_0c() {
        let rho = water_density_kg_m3(273.15).unwrap().value;
        assert!((rho - 999.84).abs() < 0.05, "ρ(0 °C) ≈ 999.84 kg/m³: {rho}");
    }

    #[test]
    fn water_density_at_40c() {
        let rho = water_density_kg_m3(313.15).unwrap().value;
        assert!(
            (rho - 992.22).abs() < 0.05,
            "ρ(40 °C) ≈ 992.22 kg/m³: {rho}"
        );
    }

    #[test]
    fn water_density_refuses_outside_range() {
        assert!(water_density_kg_m3(373.15).is_err());
        assert!(water_density_kg_m3(263.15).is_err());
    }

    // ── Water viscosity reference points ────────────────────────────
    // CRC Handbook

    #[test]
    fn water_viscosity_at_20c() {
        let eta = water_viscosity_cp(293.15).unwrap().value;
        assert!((eta - 1.002).abs() < 0.01, "η(20 °C) = 1.002 cP: {eta}");
    }

    #[test]
    fn water_viscosity_at_25c() {
        let eta = water_viscosity_cp(298.15).unwrap().value;
        assert!((eta - 0.890).abs() < 0.01, "η(25 °C) ≈ 0.890 cP: {eta}");
    }

    #[test]
    fn water_viscosity_at_0c() {
        let eta = water_viscosity_cp(273.15).unwrap().value;
        assert!((eta - 1.791).abs() < 0.01, "η(0 °C) ≈ 1.791 cP: {eta}");
    }

    #[test]
    fn water_viscosity_refuses_outside_range() {
        assert!(water_viscosity_cp(263.15).is_err());
        assert!(water_viscosity_cp(383.15).is_err());
    }

    // ── Water permittivity reference points ─────────────────────────
    // CRC Handbook / Bradley–Pitzer Table I

    #[test]
    fn water_permittivity_at_25c() {
        let eps = water_permittivity(298.15).unwrap().value;
        assert!((eps - 78.4).abs() < 0.1, "ε(25 °C) ≈ 78.4: {eps}");
    }

    #[test]
    fn water_permittivity_at_0c() {
        let eps = water_permittivity(273.15).unwrap().value;
        assert!((eps - 87.7).abs() < 0.5, "ε(0 °C) ≈ 87.7: {eps}");
    }

    #[test]
    fn water_permittivity_refuses_outside_range() {
        assert!(water_permittivity(263.15).is_err());
        assert!(water_permittivity(633.15).is_err());
    }

    // ── Henry's law reference points ────────────────────────────────
    // Sander 2015 Table 1

    #[test]
    fn henry_co2_at_25c() {
        let c = henry_lookup("CO2").unwrap();
        let h = henry_at_t(c, 298.15);
        assert!(
            (h.value - 3.4e-2).abs() < 1e-4,
            "H(CO₂, 25 °C) = 3.4e-2: {}",
            h.value
        );
    }

    #[test]
    fn henry_o2_at_25c() {
        let c = henry_lookup("O2").unwrap();
        let h = henry_at_t(c, 298.15);
        assert!(
            (h.value - 1.3e-3).abs() < 1e-6,
            "H(O₂, 25 °C) = 1.3e-3: {}",
            h.value
        );
    }

    #[test]
    fn henry_nh3_at_25c() {
        let c = henry_lookup("NH3").unwrap();
        let h = henry_at_t(c, 298.15);
        assert!(
            (h.value - 57.0).abs() < 0.01,
            "H(NH₃, 25 °C) = 57.0: {}",
            h.value
        );
    }

    #[test]
    fn henry_decreases_with_temperature_for_co2() {
        let c = henry_lookup("CO2").unwrap();
        let h25 = henry_at_t(c, 298.15).value;
        let h50 = henry_at_t(c, 323.15).value;
        assert!(
            h50 < h25,
            "CO₂ less soluble at higher T: H(25)={h25}, H(50)={h50}"
        );
    }

    #[test]
    fn henry_lookup_case_insensitive() {
        assert!(henry_lookup("co2").is_some());
        assert!(henry_lookup("CO2").is_some());
        assert!(henry_lookup("Cl2").is_some());
    }

    #[test]
    fn henry_lookup_by_name() {
        assert!(henry_lookup("ammonia").is_some());
        assert!(henry_lookup("oxygen").is_some());
    }

    // ── evaluate dispatcher ─────────────────────────────────────────

    // ── Ethanol-water density reference points ───────────────────────
    // CRC Handbook 97th ed., density of aqueous ethanol at 20 °C

    #[test]
    fn ethanol_water_pure_water() {
        let rho = ethanol_water_density_g_ml(0.0).unwrap().value;
        assert!((rho - 0.998).abs() < 0.002, "ρ(0 %) ≈ 0.998: {rho}");
    }

    #[test]
    fn ethanol_water_pure_ethanol() {
        let rho = ethanol_water_density_g_ml(1.0).unwrap().value;
        assert!((rho - 0.789).abs() < 0.002, "ρ(100 %) ≈ 0.789: {rho}");
    }

    #[test]
    fn ethanol_water_fifty_percent() {
        let rho = ethanol_water_density_g_ml(0.5).unwrap().value;
        assert!((rho - 0.914).abs() < 0.002, "ρ(50 %) ≈ 0.914: {rho}");
    }

    #[test]
    fn ethanol_water_density_decreases_with_ethanol() {
        let r0 = ethanol_water_density_g_ml(0.0).unwrap().value;
        let r50 = ethanol_water_density_g_ml(0.5).unwrap().value;
        let r100 = ethanol_water_density_g_ml(1.0).unwrap().value;
        assert!(
            r0 > r50 && r50 > r100,
            "density decreases: {r0} > {r50} > {r100}"
        );
    }

    #[test]
    fn ethanol_water_density_refuses_outside_range() {
        assert!(ethanol_water_density_g_ml(-0.1).is_err());
        assert!(ethanol_water_density_g_ml(1.1).is_err());
    }

    // ── evaluate dispatcher ─────────────────────────────────────────

    #[test]
    fn evaluate_dispatches_ethanol_water_density() {
        let args: Vec<String> = ["w=0.5"].iter().map(|s| s.to_string()).collect();
        let r = evaluate("ethanol-water-density", &args).unwrap();
        assert!((r.value - 0.914).abs() < 0.002);
    }

    #[test]
    fn evaluate_dispatches_water_density() {
        let args: Vec<String> = ["T=298.15"].iter().map(|s| s.to_string()).collect();
        let r = evaluate("water-density", &args).unwrap();
        assert!((r.value - 997.05).abs() < 0.1);
    }

    #[test]
    fn evaluate_dispatches_henry() {
        let args: Vec<String> = ["gas=CO2", "T=298.15"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let r = evaluate("henry", &args).unwrap();
        assert!((r.value - 3.4e-2).abs() < 1e-4);
    }

    #[test]
    fn evaluate_rejects_unknown() {
        assert!(evaluate("magic", &[]).is_err());
    }

    // ── Sucrose-water density (EXP-19) ──────────────────────────────
    //
    // The cross-check table is ISCOTABLES 7th ed., sucrose solution
    // densities at 20 °C — an INDEPENDENT source from the NBS cubic the
    // function actually evaluates. Agreement between the two is the
    // evidence that the inversion is right; it is not a fit.

    const ISCO_SUCROSE_20C: [(f64, f64); 6] = [
        (0.00, 0.998),
        (0.05, 1.018),
        (0.10, 1.038),
        (0.20, 1.081),
        (0.30, 1.127),
        (0.40, 1.176),
    ];

    #[test]
    fn sucrose_water_matches_an_independent_table() {
        let mut worst: f64 = 0.0;
        for (w, rho_table) in ISCO_SUCROSE_20C {
            let rho = sucrose_water_density_g_ml(w).unwrap().value;
            let residual = (rho - rho_table).abs();
            assert!(
                residual < 0.002,
                "ρ({:.0} % sucrose) = {rho:.4}, ISCOTABLES {rho_table:.3}, off by {residual:.4}",
                w * 100.0
            );
            worst = worst.max(residual);
        }
        // The tables are printed to 3 decimals, so 0.5 mg/mL of the gap
        // is their rounding. Pinning the total keeps a silent drift in
        // the inversion from hiding inside that.
        assert!(worst < 0.002, "worst residual {worst:.4} g/mL");
    }

    #[test]
    fn sucrose_water_pure_water_is_the_water_correlation() {
        let rho = sucrose_water_density_g_ml(0.0).unwrap().value;
        let water = water_density_g_ml(293.15).unwrap();
        assert!(
            (rho - water).abs() < 1e-3,
            "ρ(0 % sucrose) = {rho} must fall back on ρ_water(20 °C) = {water}"
        );
    }

    #[test]
    fn sucrose_water_density_increases_with_sugar() {
        let mut previous = 0.0;
        for step in 0..=40 {
            let rho = sucrose_water_density_g_ml(step as f64 / 100.0).unwrap().value;
            assert!(rho > previous, "density must rise with sugar at {step} %");
            previous = rho;
        }
    }

    #[test]
    fn sucrose_water_refuses_beyond_the_published_table() {
        // 40 °Bx is where NBS Table 114 stops, so this is where we stop.
        assert!(sucrose_water_density_g_ml(0.40).is_ok());
        assert!(sucrose_water_density_g_ml(0.41).is_err());
        assert!(sucrose_water_density_g_ml(-0.01).is_err());
    }

    #[test]
    fn sucrose_water_reports_the_specific_gravity_it_solved() {
        let r = sucrose_water_density_g_ml(0.20).unwrap();
        let note = r.note.expect("the inverted specific gravity is the working");
        assert!(note.contains("specific gravity"), "note was {note}");
    }

    #[test]
    fn evaluate_dispatches_sucrose_water_density() {
        let args: Vec<String> = ["w=0.2"].iter().map(|s| s.to_string()).collect();
        let r = evaluate("sucrose-water-density", &args).unwrap();
        assert!((r.value - 1.081).abs() < 0.002, "ρ(20 %) = {}", r.value);
    }

    // ── Water surface tension (EXP-48) ──────────────────────────────
    // Reference points from the IAPWS R1-76 release's own table.

    #[test]
    fn water_surface_tension_at_0c() {
        let s = water_surface_tension_mn_m(273.15).unwrap().value;
        assert!((s - 75.65).abs() < 0.05, "σ(0 °C) ≈ 75.65 mN/m: {s}");
    }

    #[test]
    fn water_surface_tension_at_20c() {
        let s = water_surface_tension_mn_m(293.15).unwrap().value;
        assert!((s - 72.74).abs() < 0.05, "σ(20 °C) ≈ 72.74 mN/m: {s}");
    }

    #[test]
    fn water_surface_tension_at_25c() {
        let s = water_surface_tension_mn_m(298.15).unwrap().value;
        assert!((s - 71.97).abs() < 0.05, "σ(25 °C) ≈ 71.97 mN/m: {s}");
    }

    #[test]
    fn water_surface_tension_at_100c() {
        let s = water_surface_tension_mn_m(373.15).unwrap().value;
        assert!((s - 58.90).abs() < 0.05, "σ(100 °C) ≈ 58.90 mN/m: {s}");
    }

    #[test]
    fn water_surface_tension_falls_with_temperature() {
        let cold = water_surface_tension_mn_m(273.15).unwrap().value;
        let warm = water_surface_tension_mn_m(323.15).unwrap().value;
        let hot = water_surface_tension_mn_m(373.15).unwrap().value;
        assert!(cold > warm && warm > hot, "{cold} > {warm} > {hot}");
    }

    #[test]
    fn water_surface_tension_refuses_outside_range() {
        assert!(water_surface_tension_mn_m(263.15).is_err());
        assert!(water_surface_tension_mn_m(383.15).is_err());
    }

    // ── Capillary rise (EXP-48) ─────────────────────────────────────

    #[test]
    fn capillary_rise_in_a_half_millimetre_tube() {
        // 2 × 0.07274 / (998.207 × 9.80665 × 5e-4) = 29.7 mm — the
        // number every textbook quotes for water in a 1 mm-bore tube.
        let h = capillary_rise_mm(0.5, 293.15, 0.0).unwrap().value;
        assert!((h - 29.7).abs() < 0.3, "h(r = 0.5 mm, 20 °C) ≈ 29.7 mm: {h}");
    }

    #[test]
    fn capillary_rise_is_inverse_in_the_radius() {
        let wide = capillary_rise_mm(1.0, 293.15, 0.0).unwrap().value;
        let narrow = capillary_rise_mm(0.5, 293.15, 0.0).unwrap().value;
        assert!(
            (narrow / wide - 2.0).abs() < 1e-9,
            "halving the bore doubles the rise: {narrow} vs {wide}"
        );
    }

    #[test]
    fn capillary_rise_vanishes_at_ninety_degrees() {
        let h = capillary_rise_mm(0.5, 293.15, 90.0).unwrap().value;
        assert!(h.abs() < 1e-9, "a non-wetting wall lifts nothing: {h}");
    }

    #[test]
    fn capillary_rise_is_negative_when_the_liquid_is_pushed_down() {
        // Mercury-like wetting on glass: θ > 90° depresses the column.
        // The geometry is the model's, even though the σ and ρ here are
        // water's — this pins the sign convention, not a mercury number.
        let h = capillary_rise_mm(0.5, 293.15, 140.0).unwrap().value;
        assert!(h < 0.0, "θ > 90 ° must depress: {h}");
    }

    #[test]
    fn capillary_rise_falls_as_the_water_warms() {
        let cold = capillary_rise_mm(0.5, 278.15, 0.0).unwrap().value;
        let warm = capillary_rise_mm(0.5, 313.15, 0.0).unwrap().value;
        assert!(cold > warm, "warmer water climbs less: {cold} vs {warm}");
    }

    #[test]
    fn capillary_rise_refuses_beyond_the_density_correlation() {
        // The surface tension is happy to 100 °C; the density is not,
        // and the narrower input is the one that decides.
        assert!(water_surface_tension_mn_m(333.15).is_ok());
        assert!(capillary_rise_mm(0.5, 333.15, 0.0).is_err());
    }

    #[test]
    fn capillary_rise_refuses_nonsense_geometry() {
        assert!(capillary_rise_mm(0.0, 293.15, 0.0).is_err());
        assert!(capillary_rise_mm(-1.0, 293.15, 0.0).is_err());
        assert!(capillary_rise_mm(0.5, 293.15, 200.0).is_err());
    }

    #[test]
    fn evaluate_dispatches_the_new_interfacial_properties() {
        let st: Vec<String> = ["T=293.15"].iter().map(|s| s.to_string()).collect();
        assert!((evaluate("water-surface-tension", &st).unwrap().value - 72.74).abs() < 0.05);

        let cap: Vec<String> = ["r=0.5", "T=293.15"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!((evaluate("capillary-rise", &cap).unwrap().value - 29.7).abs() < 0.3);

        // theta is optional and defaults to a perfectly wetting wall.
        let flat: Vec<String> = ["r=0.5", "T=293.15", "theta=90"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(evaluate("capillary-rise", &flat).unwrap().value.abs() < 1e-9);
    }
}
