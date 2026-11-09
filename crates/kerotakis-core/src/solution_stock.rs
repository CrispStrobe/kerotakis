//! E1: reagents added as aqueous solutions with a stated concentration.
//!
//! `add v1 HCl 10mL 37%` means ten millilitres of 37 % w/w hydrochloric
//! acid — HCl *and its water* — not ten millilitres of pure hydrogen
//! chloride. The concentration is resolved to two pinned mole amounts at
//! parse time, so a replay never depends on today's density table.
//!
//! One generic mechanism for every solute:
//! * a reviewed density table (`data/solutions/stock-solutions.toml`) turns
//!   a solution volume into a mass and a molarity into a mass fraction;
//! * without a table, a molarity of a *liquid* solute uses the ideal-dilution
//!   (additive-volume) approximation and says so in the result;
//! * everything else that needs data we do not have is refused, typed.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;
use crate::species::{self, Phase, SpeciesData};

/// Density of water at 20 °C, g/mL (CRC, see the data file).
pub const WATER_DENSITY_20C: f64 = 0.99820;
const WATER_KEY: &str = "water";

/// What concentration the learner stated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Concentration {
    /// Mass fraction of solute in the solution, 0 < w <= 1.
    MassFraction(f64),
    /// Amount concentration, mol/L of solution.
    Molarity(f64),
}

impl Concentration {
    pub fn label(self) -> String {
        match self {
            Concentration::MassFraction(w) => format!("{}% w/w", trim(w * 100.0)),
            Concentration::Molarity(m) => format!("{} M", trim(m)),
        }
    }
}

fn trim(x: f64) -> String {
    let s = format!("{x:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Where the solution's density came from, carried into the operator and
/// the result event so an approximation is never silent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SolutionDensityBasis {
    /// Interpolated in a reviewed table; `source` is its source id.
    Reviewed { source: String },
    /// Ideal dilution: solute and water volumes taken as additive.
    IdealDilution,
    /// The amount and concentration fixed both masses without a density.
    NotRequired,
}

/// The two pinned portions one solution dose deposits.
#[derive(Debug, Clone, PartialEq)]
pub struct SolutionPortion {
    pub solute_moles: f64,
    pub water_moles: f64,
    pub label: String,
    pub density_basis: SolutionDensityBasis,
}

#[derive(Debug, Deserialize)]
struct DataFile {
    #[serde(default)]
    source: Vec<Source>,
    #[serde(default)]
    density: Vec<DensityTable>,
    #[serde(default)]
    stock: Vec<Stock>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Source {
    pub id: String,
    pub citation: String,
    pub review: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DensityTable {
    pub species: String,
    pub source: String,
    /// `[mass_fraction, density g/mL]`, ascending in mass fraction.
    pub points: Vec<[f64; 2]>,
    /// What this table was checked against, and to what precision; rows
    /// no reproduction confirmed are named as unverified.
    pub review: String,
}

/// One common laboratory stock, offered as a shelf item.
#[derive(Debug, Clone, Deserialize)]
pub struct Stock {
    pub key: String,
    /// The shelf label (`hydrochloric acid 37 %`); the web locales translate
    /// it by this English text.
    pub name: String,
    pub species: String,
    #[serde(default)]
    pub percent: Option<f64>,
    #[serde(default)]
    pub molar: Option<f64>,
    /// The default concentration offered for this solute at lv1/lv2.
    #[serde(default)]
    pub common: bool,
}

impl Stock {
    pub fn concentration(&self) -> Concentration {
        match (self.percent, self.molar) {
            (Some(p), None) => Concentration::MassFraction(p / 100.0),
            (None, Some(m)) => Concentration::Molarity(m),
            _ => panic!("stock {} must state exactly one of percent/molar", self.key),
        }
    }
}

fn data() -> &'static DataFile {
    static DATA: OnceLock<DataFile> = OnceLock::new();
    DATA.get_or_init(|| {
        toml::from_str(include_str!("../../../data/solutions/stock-solutions.toml"))
            .expect("data/solutions/stock-solutions.toml parses")
    })
}

pub fn sources() -> &'static [Source] {
    &data().source
}

pub fn density_tables() -> &'static [DensityTable] {
    &data().density
}

pub fn stocks() -> &'static [Stock] {
    &data().stock
}

/// A shelf stock by its key (`hydrochloric_acid_37_percent`).
pub fn lookup_stock(key: &str) -> Option<&'static Stock> {
    stocks().iter().find(|stock| stock.key == key)
}

/// The stocks that stand on the lv1/lv2 shelf as bottles of their own:
/// the common one per solute, in the data file's order.
pub fn shelf_stocks() -> impl Iterator<Item = &'static Stock> {
    stocks().iter().filter(|stock| stock.common)
}

/// The solution density (g/mL) of a listed stock, from the same reviewed
/// table (or disclosed ideal dilution) a dose of it is resolved through.
pub fn stock_density(stock: &Stock) -> Option<f64> {
    let solute = species::lookup_key(&stock.species)?;
    fraction_and_density(solute, stock.concentration())
        .ok()
        .and_then(|(_, density)| density.map(|(rho, _)| rho))
}

/// Where a listed stock's numbers come from: the solute's own provenance
/// and, after it, the source of the solution density the dose is resolved
/// through (a reviewed table's id, or the disclosed ideal dilution). Never
/// empty: every shelf item in the species export carries provenance.
pub fn stock_provenance(stock: &Stock) -> Option<String> {
    let solute = species::lookup_key(&stock.species)?;
    let density = match fraction_and_density(solute, stock.concentration()) {
        Ok((_, Some((_, SolutionDensityBasis::Reviewed { source })))) => source,
        _ => "ideal dilution in water".to_string(),
    };
    Some(format!(
        "{}; solution density: {density}",
        solute.provenance
    ))
}

/// The common (default) stock for a solute, if one is listed.
pub fn common_stock(species_key: &str) -> Option<&'static Stock> {
    stocks()
        .iter()
        .find(|stock| stock.species == species_key && stock.common)
}

fn table(species_key: &str) -> Option<&'static DensityTable> {
    density_tables().iter().find(|t| t.species == species_key)
}

/// Linear interpolation inside the table; `None` outside it.
pub fn density_at(species_key: &str, w: f64) -> Option<(f64, &'static str)> {
    let t = table(species_key)?;
    let pts = &t.points;
    let first = pts.first()?;
    let last = pts.last()?;
    if w < first[0] - 1e-12 || w > last[0] + 1e-12 {
        return None;
    }
    for pair in pts.windows(2) {
        let ([w0, d0], [w1, d1]) = (pair[0], pair[1]);
        if w <= w1 + 1e-12 {
            let f = if w1 > w0 {
                ((w - w0) / (w1 - w0)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            return Some((d0 + f * (d1 - d0), t.source.as_str()));
        }
    }
    Some((last[1], t.source.as_str()))
}

/// Read concentration tokens after the amount: `37%`, `37%w/w`,
/// `37% w/w`, `37% (w/w)`, `1M`, `as 1M`. Returns the concentration and
/// how many words it used; `None` when the words do not start with one.
pub fn parse_concentration(words: &[&str]) -> Result<Option<(Concentration, usize)>, Refusal> {
    let mut used = 0;
    let mut rest = words;
    if rest.first() == Some(&"as") {
        used += 1;
        rest = &rest[1..];
    }
    let Some(token) = rest.first() else {
        return if used > 0 {
            Err(needs_concentration())
        } else {
            Ok(None)
        };
    };
    let (number, suffix) = if let Some(i) = token.find('%') {
        (&token[..i], &token[i..])
    } else if let Some(n) = token.strip_suffix('M') {
        (n, "M")
    } else if used > 0 {
        return Err(needs_concentration());
    } else {
        return Ok(None);
    };
    let value: f64 = number.parse().map_err(|_| needs_concentration())?;
    used += 1;
    let concentration = match suffix {
        "M" => {
            if !value.is_finite() || value <= 0.0 {
                return Err(Refusal::new(
                    "error.solution-molarity-positive",
                    "a solution's molarity must be a positive number (got '{token}')",
                )
                .with("token", *token));
            }
            Concentration::Molarity(value)
        }
        "%" | "%w/w" | "%(w/w)" => {
            if suffix == "%" {
                if let Some(next) = rest.get(1) {
                    if matches!(*next, "w/w" | "(w/w)") {
                        used += 1;
                    }
                }
            }
            if !value.is_finite() || value <= 0.0 || value > 100.0 {
                return Err(Refusal::new(
                    "error.solution-percent-range",
                    "a mass percentage must be above 0 and at most 100 (got '{token}')",
                )
                .with("token", *token));
            }
            Concentration::MassFraction(value / 100.0)
        }
        _ => return Err(needs_concentration()),
    };
    Ok(Some((concentration, used)))
}

fn needs_concentration() -> Refusal {
    Refusal::new(
        "error.solution-needs-concentration",
        "state the solution's concentration as a w/w percentage (37%) or a molarity (1M)",
    )
}

fn needs_data(solute: &SpeciesData, what: &str) -> Refusal {
    let refusal = if what == "density" {
        Refusal::new(
            "error.solution-needs-density",
            "no reviewed density for aqueous {species} at this concentration; add it by mass (g) or by mol",
        )
    } else {
        Refusal::new(
            "error.solution-needs-solubility",
            "no reviewed solubility or density table for aqueous {species}; add it as a pure substance",
        )
    };
    refusal.with("species", solute.key)
}

/// The mass fraction and (when known) solution density for a concentration.
fn fraction_and_density(
    solute: &SpeciesData,
    concentration: Concentration,
) -> Result<(f64, Option<(f64, SolutionDensityBasis)>), Refusal> {
    let reviewed = |w: f64| {
        density_at(solute.key, w).map(|(rho, src)| {
            (
                rho,
                SolutionDensityBasis::Reviewed {
                    source: src.to_string(),
                },
            )
        })
    };
    match concentration {
        Concentration::MassFraction(w) => {
            if table(solute.key).is_none() && solute.standard_phase != Phase::Liquid {
                return Err(needs_data(solute, "solubility"));
            }
            Ok((w, reviewed(w)))
        }
        Concentration::Molarity(m) => {
            let target = m * solute.molar_mass / 1000.0; // g solute per mL solution
            if let Some(t) = table(solute.key) {
                // rho(w)·w rises monotonically over every listed table, so
                // bisection finds the one fraction with that solute load.
                let lo0 = t.points.first().map_or(0.0, |p| p[0]);
                let hi0 = t.points.last().map_or(0.0, |p| p[0]);
                let load = |w: f64| density_at(solute.key, w).map_or(f64::NAN, |(r, _)| r * w);
                if !matches!(
                    target.partial_cmp(&(load(hi0) + 1e-12)),
                    Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
                ) || target < load(lo0) - 1e-12
                {
                    return Err(Refusal::new(
                        "error.solution-beyond-table",
                        "{molarity} M {species} lies outside the reviewed density table",
                    )
                    .with("molarity", trim(m))
                    .with("species", solute.key));
                }
                let (mut lo, mut hi) = (lo0, hi0);
                for _ in 0..200 {
                    let mid = 0.5 * (lo + hi);
                    if load(mid) < target {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let w = 0.5 * (lo + hi);
                return Ok((w, reviewed(w)));
            }
            if solute.standard_phase != Phase::Liquid || solute.density <= 0.0 {
                return Err(needs_data(solute, "solubility"));
            }
            // Ideal dilution per mL of solution: the solute occupies
            // target/rho_solute mL and water fills the rest.
            let water_ml = 1.0 - target / solute.density;
            if water_ml <= 0.0 {
                return Err(Refusal::new(
                    "error.solution-beyond-table",
                    "{molarity} M {species} lies outside the reviewed density table",
                )
                .with("molarity", trim(m))
                .with("species", solute.key));
            }
            let water_g = water_ml * WATER_DENSITY_20C;
            let rho = target + water_g;
            Ok((
                target / rho,
                Some((rho, SolutionDensityBasis::IdealDilution)),
            ))
        }
    }
}

/// Resolve one dose: `amount` is the solution (`g`, `mL`, `L`) or the
/// solute (`mol`).
pub fn resolve(
    solute: &SpeciesData,
    amount_word: &str,
    concentration: Concentration,
) -> Result<SolutionPortion, Refusal> {
    let water = species::lookup_key(WATER_KEY).expect("water is a registry species");
    let digits = amount_word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let unit = &amount_word[digits.len()..];
    let value: f64 = digits.parse().map_err(|_| {
        Refusal::new(
            "error.needs-number-and-unit",
            "'{word}' needs a valid number and unit suffix",
        )
        .with("word", amount_word)
    })?;
    if !value.is_finite() || value <= 0.0 {
        return Err(Refusal::new(
            "error.solution-amount-positive",
            "a solution amount must be positive",
        ));
    }
    let (w, density) = fraction_and_density(solute, concentration)?;
    let volume_ml =
        match unit {
            "mL" | "ml" => Some(value),
            "L" | "l" => Some(value * 1000.0),
            "g" | "mol" => None,
            other => return Err(Refusal::new(
                "error.solution-amount-unit",
                "a solution is measured in g, mL or L of solution, or mol of solute (got '{unit}')",
            )
            .with("unit", other)),
        };
    let (solution_g, basis) = match (unit, volume_ml) {
        (_, Some(ml)) => {
            let (rho, basis) = density.ok_or_else(|| needs_data(solute, "density"))?;
            (ml * rho, basis)
        }
        ("mol", None) => (
            value * solute.molar_mass / w,
            basis_for_mass(&concentration, density),
        ),
        _ => (value, basis_for_mass(&concentration, density)),
    };
    let solute_moles = solution_g * w / solute.molar_mass;
    let water_moles = solution_g * (1.0 - w) / water.molar_mass;
    if !solute_moles.is_finite() || !water_moles.is_finite() {
        return Err(Refusal::new(
            "error.solution-amount-positive",
            "a solution amount must be positive",
        ));
    }
    Ok(SolutionPortion {
        solute_moles,
        water_moles: water_moles.max(0.0),
        label: concentration.label(),
        density_basis: basis,
    })
}

/// A mass or solute-mole amount with a w/w fraction needs no density; a
/// molarity still converted through one, which is then disclosed.
fn basis_for_mass(
    concentration: &Concentration,
    density: Option<(f64, SolutionDensityBasis)>,
) -> SolutionDensityBasis {
    match concentration {
        Concentration::MassFraction(_) => SolutionDensityBasis::NotRequired,
        Concentration::Molarity(_) => density
            .map(|(_, basis)| basis)
            .unwrap_or(SolutionDensityBasis::NotRequired),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hcl() -> &'static SpeciesData {
        species::lookup_key("HCl").unwrap()
    }

    #[test]
    fn data_file_is_self_consistent() {
        for t in density_tables() {
            assert!(species::lookup_key(&t.species).is_some(), "{}", t.species);
            assert!(sources().iter().any(|s| s.id == t.source), "{}", t.source);
            assert!(
                t.review.contains("checked") || t.review.starts_with("unverified"),
                "{} density table states no review",
                t.species
            );
            assert!(
                t.points.windows(2).all(|p| p[1][0] > p[0][0]),
                "{}",
                t.species
            );
            assert!(
                t.points
                    .windows(2)
                    .all(|p| p[1][0] * p[1][1] > p[0][0] * p[0][1]),
                "solute load must rise for {}",
                t.species
            );
        }
        for s in stocks() {
            assert!(species::lookup_key(&s.species).is_some(), "{}", s.key);
            assert!(
                table(&s.species).is_some(),
                "{} has no density table",
                s.key
            );
            let _ = s.concentration();
            let units = if s.molar.is_some() { "1000mL" } else { "100g" };
            resolve(
                species::lookup_key(&s.species).unwrap(),
                units,
                s.concentration(),
            )
            .unwrap_or_else(|e| panic!("{} does not resolve: {e:?}", s.key));
        }
    }

    /// Every common stock is a shelf bottle: one per solute, named with its
    /// concentration, with a density for the fluid overlay, and resolvable
    /// by its own key exactly as `add v1 <key> 10mL` resolves it.
    #[test]
    fn common_stocks_are_shelf_bottles() {
        let shelf: Vec<&Stock> = shelf_stocks().collect();
        assert!(shelf
            .iter()
            .any(|s| s.key == "hydrochloric_acid_37_percent"));
        let mut solutes: Vec<&str> = shelf.iter().map(|s| s.species.as_str()).collect();
        let listed = solutes.len();
        solutes.sort_unstable();
        solutes.dedup();
        assert_eq!(listed, solutes.len(), "one common stock per solute");
        for stock in &shelf {
            let digits = match (stock.percent, stock.molar) {
                (Some(p), None) => format!("{} %", trim(p)),
                (None, Some(m)) => format!("{} M", trim(m)),
                _ => unreachable!(),
            };
            assert!(
                stock.name.ends_with(&digits),
                "{}: {}",
                stock.key,
                stock.name
            );
            let rho = stock_density(stock).unwrap_or_else(|| panic!("{} density", stock.key));
            assert!((0.8..2.0).contains(&rho), "{} {rho}", stock.key);
            let provenance = stock_provenance(stock).unwrap();
            assert!(
                !provenance.trim().is_empty() && !provenance.starts_with(';'),
                "{} provenance {provenance:?}",
                stock.key
            );
        }
        let hcl37 = lookup_stock("hydrochloric_acid_37_percent").unwrap();
        assert_eq!(hcl37.name, "hydrochloric acid 37 %");
        assert!((stock_density(hcl37).unwrap() - 1.18365).abs() < 1e-4);
        // Not common: never its own bottle.
        assert!(shelf.iter().all(|s| s.key != "glacial_acetic_acid"));
    }

    #[test]
    fn parses_every_form() {
        let mf = |w: &[&str]| parse_concentration(w).unwrap().unwrap();
        assert_eq!(mf(&["37%"]), (Concentration::MassFraction(0.37), 1));
        assert_eq!(mf(&["37%w/w"]), (Concentration::MassFraction(0.37), 1));
        assert_eq!(mf(&["37%", "w/w"]), (Concentration::MassFraction(0.37), 2));
        assert_eq!(
            mf(&["37%", "(w/w)"]),
            (Concentration::MassFraction(0.37), 2)
        );
        assert_eq!(mf(&["1M"]), (Concentration::Molarity(1.0), 1));
        assert_eq!(mf(&["as", "1M"]), (Concentration::Molarity(1.0), 2));
        assert_eq!(parse_concentration(&["@", "60C"]).unwrap(), None);
        assert_eq!(parse_concentration(&[]).unwrap(), None);
    }

    #[test]
    fn rejects_out_of_range() {
        for bad in [["120%"], ["0%"], ["-5%"], ["-1M"], ["0M"], ["xM"]] {
            assert!(parse_concentration(&bad).is_err(), "{bad:?}");
        }
        assert!(parse_concentration(&["as"]).is_err());
        assert!(parse_concentration(&["as", "60C"]).is_err());
    }

    #[test]
    fn ten_ml_of_37_percent_hcl() {
        let p = resolve(hcl(), "10mL", Concentration::MassFraction(0.37)).unwrap();
        let rho = density_at("HCl", 0.37).unwrap().0;
        let water = species::lookup_key("water").unwrap();
        assert!((p.solute_moles - 10.0 * rho * 0.37 / hcl().molar_mass).abs() < 1e-9);
        assert!((p.water_moles - 10.0 * rho * 0.63 / water.molar_mass).abs() < 1e-9);
        assert!(matches!(
            p.density_basis,
            SolutionDensityBasis::Reviewed { .. }
        ));
    }

    #[test]
    fn molarity_by_volume_and_by_moles_agree() {
        let by_volume = resolve(hcl(), "100mL", Concentration::Molarity(1.0)).unwrap();
        assert!((by_volume.solute_moles - 0.1).abs() < 1e-9);
        let by_moles = resolve(hcl(), "0.1mol", Concentration::Molarity(1.0)).unwrap();
        assert!((by_moles.solute_moles - 0.1).abs() < 1e-12);
        assert!((by_moles.water_moles - by_volume.water_moles).abs() < 1e-9);
    }

    #[test]
    fn mass_by_grams_needs_no_density() {
        let h2o2 = species::lookup_key("H2O2").unwrap();
        let p = resolve(h2o2, "20g", Concentration::MassFraction(0.30)).unwrap();
        assert!((p.solute_moles - 6.0 / h2o2.molar_mass).abs() < 1e-12);
        assert_eq!(p.density_basis, SolutionDensityBasis::NotRequired);
    }

    #[test]
    fn refuses_what_it_cannot_know() {
        // Beyond the HCl table (40 % w/w max): volume needs a density.
        assert!(resolve(hcl(), "10mL", Concentration::MassFraction(0.45)).is_err());
        // Molarity beyond the table.
        assert!(resolve(hcl(), "10mL", Concentration::Molarity(20.0)).is_err());
        // Kitchen measures are not solution amounts.
        assert!(resolve(hcl(), "1cup", Concentration::Molarity(1.0)).is_err());
        assert!(resolve(hcl(), "-1mL", Concentration::Molarity(1.0)).is_err());
        // A solid with no reviewed table has no known solubility.
        let solid = crate::species::registry()
            .iter()
            .find(|s| s.standard_phase == Phase::Solid && table(s.key).is_none())
            .unwrap();
        let err = resolve(solid, "10mL", Concentration::Molarity(1.0)).unwrap_err();
        assert_eq!(err.key, "error.solution-needs-solubility");
    }

    #[test]
    fn liquid_without_table_uses_disclosed_ideal_dilution() {
        let liquid = crate::species::registry()
            .iter()
            .find(|s| {
                s.standard_phase == Phase::Liquid && table(s.key).is_none() && s.key != "water"
            })
            .unwrap();
        let p = resolve(liquid, "100mL", Concentration::Molarity(0.1)).unwrap();
        assert_eq!(p.density_basis, SolutionDensityBasis::IdealDilution);
        assert!((p.solute_moles - 0.01).abs() < 1e-9);
        // w/w by volume without a table is refused, not approximated.
        assert!(resolve(liquid, "100mL", Concentration::MassFraction(0.1)).is_err());
    }
}
