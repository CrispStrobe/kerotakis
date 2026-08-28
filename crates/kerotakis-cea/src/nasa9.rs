//! NASA-9 thermochemistry, parsed from NASA CEA's `thermo.inp`
//! (Apache-2.0, `vendor/nasa-cea/`).
//!
//! Each species record carries its own reference citation — "Gurvich,1991
//! pt1 p27", "TPIS,1979", a JANAF table — which we keep as provenance
//! rather than discarding, exactly as we do with the PHREEQC databases.
//!
//! The polynomial form (NASA TP-2002-211556) is, with `a1..a7` and the two
//! integration constants `b1`, `b2`:
//!
//! ```text
//! Cp/R  = a1 T^-2 + a2 T^-1 + a3 + a4 T + a5 T^2 + a6 T^3 + a7 T^4
//! H/RT  = -a1 T^-2 + a2 ln(T)/T + a3 + a4 T/2 + a5 T^2/3 + a6 T^3/4
//!         + a7 T^4/5 + b1/T
//! S/R   = -a1 T^-2/2 - a2 T^-1 + a3 ln(T) + a4 T + a5 T^2/2 + a6 T^3/3
//!         + a7 T^4/4 + b2
//! ```

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The universal gas constant, J/(mol·K).
pub const R: f64 = 8.31446261815324;

/// Standard reference temperature, K.
pub const T_REF: f64 = 298.15;

#[derive(Debug, Clone)]
pub struct Interval {
    pub t_min: f64,
    pub t_max: f64,
    /// a1..a7 followed by b1, b2.
    pub coeffs: [f64; 9],
}

#[derive(Debug, Clone)]
pub struct Species {
    pub name: String,
    /// The record's own literature citation, verbatim.
    pub reference: String,
    /// Element symbol → atom count.
    pub composition: BTreeMap<String, f64>,
    /// 0 = gas; nonzero = condensed (CEA's convention).
    pub phase_code: i32,
    /// g/mol.
    pub molar_mass: f64,
    /// Standard enthalpy of formation at 298.15 K, J/mol.
    pub h_formation: f64,
    pub intervals: Vec<Interval>,
}

impl Species {
    pub fn is_gas(&self) -> bool {
        self.phase_code == 0
    }

    fn interval_for(&self, t: f64) -> Option<&Interval> {
        self.intervals
            .iter()
            .find(|i| t >= i.t_min && t <= i.t_max)
            .or_else(|| {
                // Clamp to the nearest interval rather than refusing: the
                // caller is told the valid range through `t_range()`.
                if let Some(first) = self.intervals.first() {
                    if t < first.t_min {
                        return Some(first);
                    }
                }
                self.intervals.last()
            })
    }

    /// Validity range of the tabulated data, K.
    pub fn t_range(&self) -> Option<(f64, f64)> {
        Some((self.intervals.first()?.t_min, self.intervals.last()?.t_max))
    }

    /// Molar heat capacity at constant pressure, J/(mol·K).
    pub fn cp(&self, t: f64) -> Option<f64> {
        let i = self.interval_for(t)?;
        let a = &i.coeffs;
        Some(
            R * (a[0] / (t * t)
                + a[1] / t
                + a[2]
                + a[3] * t
                + a[4] * t * t
                + a[5] * t * t * t
                + a[6] * t * t * t * t),
        )
    }

    /// Absolute molar enthalpy, J/mol (referenced so that H(298.15) equals
    /// the formation enthalpy, which is CEA's convention).
    pub fn h(&self, t: f64) -> Option<f64> {
        let i = self.interval_for(t)?;
        let a = &i.coeffs;
        Some(
            R * t
                * (-a[0] / (t * t)
                    + a[1] * t.ln() / t
                    + a[2]
                    + a[3] * t / 2.0
                    + a[4] * t * t / 3.0
                    + a[5] * t * t * t / 4.0
                    + a[6] * t * t * t * t / 5.0
                    + a[7] / t),
        )
    }

    /// Standard molar entropy, J/(mol·K).
    pub fn s(&self, t: f64) -> Option<f64> {
        let i = self.interval_for(t)?;
        let a = &i.coeffs;
        Some(
            R * (-a[0] / (2.0 * t * t) - a[1] / t
                + a[2] * t.ln()
                + a[3] * t
                + a[4] * t * t / 2.0
                + a[5] * t * t * t / 3.0
                + a[6] * t * t * t * t / 4.0
                + a[8]),
        )
    }

    /// Standard molar Gibbs energy, J/mol.
    pub fn g(&self, t: f64) -> Option<f64> {
        Some(self.h(t)? - t * self.s(t)?)
    }
}

#[derive(Debug, Default)]
pub struct ThermoDb {
    /// Species CEA permits as equilibrium products.
    pub species: BTreeMap<String, Species>,
    /// Separate feed-only records after `END PRODUCTS`. Keeping these out of
    /// `species` prevents a liquid fuel from being invented as an equilibrium
    /// product while still letting its measured reactant enthalpy enter an
    /// energy balance.
    pub reactants: BTreeMap<String, Species>,
}

static DB: OnceLock<ThermoDb> = OnceLock::new();

/// The embedded NASA CEA thermodynamic database (Apache-2.0, NASA).
pub fn db() -> &'static ThermoDb {
    DB.get_or_init(|| ThermoDb::parse(include_str!("../../../vendor/nasa-cea/thermo.inp")))
}

/// CEA writes Fortran double-precision exponents as `D`.
fn parse_f64(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    t.replace(['D', 'd'], "E").parse().ok()
}

impl ThermoDb {
    pub fn parse(text: &str) -> ThermoDb {
        let mut db = ThermoDb::default();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        // Records live between the `thermo` header and `END PRODUCTS`.
        while i < lines.len() && !lines[i].trim_start().starts_with("thermo") {
            i += 1;
        }
        i += 2; // skip the `thermo` keyword line and its T-range line
        let mut in_reactants = false;
        while i < lines.len() {
            let line = lines[i];
            let marker = line.trim_start().to_ascii_uppercase();
            if marker.starts_with("END PRODUCTS") {
                in_reactants = true;
                i += 1;
                continue;
            }
            if marker.starts_with("END REACTANTS") {
                break;
            }
            if line.starts_with('!') || line.trim().is_empty() {
                i += 1;
                continue;
            }
            // Record header: name in cols 1-18, reference thereafter.
            let name = line.get(..18).unwrap_or(line).trim().to_string();
            if name.is_empty() {
                i += 1;
                continue;
            }
            let reference = line.get(18..).unwrap_or("").trim().to_string();
            let Some(meta) = lines.get(i + 1) else { break };
            let n_intervals: usize = meta
                .get(..2)
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            // Composition: 5 slots of (symbol, count) at cols 11..50.
            let mut composition = BTreeMap::new();
            for slot in 0..5 {
                let start = 10 + slot * 8;
                let Some(chunk) = meta.get(start..start + 8) else {
                    break;
                };
                // CEA writes element symbols upper-case ("CA", "NA"):
                // normalise to standard capitalisation so they match the
                // rest of the world's "Ca", "Na".
                let raw = chunk.get(..2).unwrap_or("").trim();
                let mut chars = raw.chars();
                let sym: String = match chars.next() {
                    Some(first) => {
                        first.to_ascii_uppercase().to_string()
                            + &chars.as_str().to_ascii_lowercase()
                    }
                    None => String::new(),
                };
                let count = chunk.get(2..).and_then(parse_f64).unwrap_or(0.0);
                if !sym.is_empty() && count != 0.0 {
                    *composition.entry(sym).or_insert(0.0) += count;
                }
            }
            let phase_code = meta
                .get(50..52)
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(0);
            let molar_mass = meta.get(52..65).and_then(parse_f64).unwrap_or(0.0);
            let h_formation = meta.get(65..80).and_then(parse_f64).unwrap_or(0.0);

            let mut intervals = Vec::new();
            let mut cursor = i + 2;
            for _ in 0..n_intervals {
                let Some(range) = lines.get(cursor) else {
                    break;
                };
                let t_min = range.get(..11).and_then(parse_f64).unwrap_or(0.0);
                let t_max = range.get(11..22).and_then(parse_f64).unwrap_or(0.0);
                let (Some(c1), Some(c2)) = (lines.get(cursor + 1), lines.get(cursor + 2)) else {
                    break;
                };
                let mut coeffs = [0.0f64; 9];
                for (k, slot) in coeffs.iter_mut().enumerate().take(5) {
                    *slot = c1
                        .get(k * 16..k * 16 + 16)
                        .and_then(parse_f64)
                        .unwrap_or(0.0);
                }
                // Second line: a6, a7, (blank), b1, b2.
                for (k, idx) in [(0usize, 5usize), (1, 6), (3, 7), (4, 8)] {
                    coeffs[idx] = c2
                        .get(k * 16..k * 16 + 16)
                        .and_then(parse_f64)
                        .unwrap_or(0.0);
                }
                intervals.push(Interval {
                    t_min,
                    t_max,
                    coeffs,
                });
                cursor += 3;
            }
            if n_intervals == 0 {
                cursor = i + 2;
            }
            let target = if in_reactants {
                &mut db.reactants
            } else {
                &mut db.species
            };
            target.entry(name.clone()).or_insert(Species {
                name,
                reference,
                composition,
                phase_code,
                molar_mass,
                h_formation,
                intervals,
            });
            i = cursor;
        }
        db
    }

    pub fn get(&self, name: &str) -> Option<&Species> {
        self.species.get(name)
    }

    pub fn get_reactant(&self, name: &str) -> Option<&Species> {
        self.reactants.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_database() {
        let db = db();
        assert!(
            db.species.len() > 1000,
            "expected the full CEA species set, got {}",
            db.species.len()
        );
        assert!(
            db.reactants.contains_key("C2H5OH(L)"),
            "feed-only liquid ethanol is parsed separately from products"
        );
        assert!(!db.species.contains_key("C2H5OH(L)"));
        for name in ["CO2", "H2O", "O2", "N2", "CH4"] {
            assert!(db.get(name).is_some(), "{name} missing");
        }
    }

    #[test]
    fn co2_matches_the_reference_values() {
        let co2 = db().get("CO2").expect("CO2");
        assert_eq!(co2.composition["C"], 1.0);
        assert_eq!(co2.composition["O"], 2.0);
        assert!(co2.is_gas());
        assert!((co2.molar_mass - 44.0095).abs() < 1e-3);
        // Formation enthalpy: -393.51 kJ/mol, the textbook value.
        assert!(
            (co2.h_formation + 393_510.0).abs() < 50.0,
            "ΔHf(CO2) = {}",
            co2.h_formation
        );
        // Cp(298.15) = 37.13 J/(mol·K).
        let cp = co2.cp(T_REF).expect("cp");
        assert!((cp - 37.13).abs() < 0.2, "Cp(CO2, 298) = {cp}");
        // S°(298.15) = 213.8 J/(mol·K).
        let s = co2.s(T_REF).expect("s");
        assert!((s - 213.8).abs() < 0.5, "S(CO2, 298) = {s}");
        // H(298.15) must reproduce the formation enthalpy.
        let h = co2.h(T_REF).expect("h");
        assert!(
            (h - co2.h_formation).abs() < 100.0,
            "H(298) = {h} vs ΔHf = {}",
            co2.h_formation
        );
        // And it carries its citation.
        assert!(!co2.reference.is_empty(), "CO2 cites its source");
    }

    #[test]
    fn heat_capacity_rises_with_temperature() {
        let n2 = db().get("N2").expect("N2");
        let cold = n2.cp(300.0).unwrap();
        let hot = n2.cp(2000.0).unwrap();
        assert!(cold > 28.0 && cold < 30.0, "Cp(N2, 300) = {cold}");
        assert!(hot > cold, "diatomic Cp rises with T: {cold} → {hot}");
    }

    #[test]
    fn two_letter_elements_are_normalised() {
        // CEA writes "CA"; the rest of chemistry writes "Ca".
        let calcite = db().get("CaCO3(cr)").expect("calcite");
        assert_eq!(calcite.composition["Ca"], 1.0);
        assert_eq!(calcite.composition["C"], 1.0);
        assert_eq!(calcite.composition["O"], 3.0);
        assert!(!calcite.is_gas());
        // ΔHf(calcite) = −1206.6 kJ/mol.
        assert!((calcite.h_formation + 1_206_600.0).abs() < 500.0);
    }

    #[test]
    fn condensed_phases_are_marked() {
        // Water's condensed records exist alongside the gas.
        let liquid = db()
            .species
            .values()
            .find(|s| s.name.starts_with("H2O(") && !s.is_gas());
        assert!(liquid.is_some(), "a condensed water phase is present");
    }
}
