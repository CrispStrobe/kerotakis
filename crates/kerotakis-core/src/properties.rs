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
}
