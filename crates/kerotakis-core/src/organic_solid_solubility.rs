//! Reviewed aqueous solubilities for registry ORGANIC solids that the
//! compiled registry carries none for (lane 2026-11-11, D3-AN-004).
//!
//! The defect: ascorbic acid added to water as a solid was said to be
//! "in contact with liquid: no wired solver models this
//! dissolution/reaction" and then, one step later, reacted with iodine
//! straight out of the solid. One route said it could not dissolve, the
//! next consumed it as if it had. The family-wide fix is that a soluble
//! organic solid with a cited aqueous solubility goes through the same
//! dissolution route every other solid takes (`solve::saturation_moves`
//! via [`crate::species::SpeciesData::aqueous_solubility_at`]); an organic
//! solid without one is refused with a typed reason instead of the generic
//! "no solver" sentence, and a curated reaction that still draws on its
//! undissolved solid says so (`solve::solid_phase_reaction_label`).
//!
//! Every row is a measured saturation mole fraction transcribed from a
//! primary source, converted at run time to the registry's unit (g solute
//! per 100 mL of water) with the registry's own molar masses and water
//! density, so no derived number is typed in by hand:
//!
//! `g/100 mL water = x/(1-x) · M_solute/M_water · 100 · ρ_water`.
//!
//! Between measured temperatures the mole fraction is interpolated
//! linearly; outside the measured range it is held at the nearest end,
//! which is the same convention as the registry's two-point rows.
//!
//! What is deliberately NOT here (see `docs/organic-solid-dissolution-coverage-20261111.md`
//! and `tools/organic_solid_dissolution_coverage.py`):
//! - organic solids that are already carried by another route (glucose,
//!   sucrose, fructose, malic and oxalic acid have registry solubilities;
//!   citric acid is speciated as Citrate by the PHREEQC derived route);
//! - organic solids for which no primary measurement was read
//!   (dehydroascorbic acid, cellulose, nylon, paraffin): they are refused
//!   with `not-modeled.organic-solid-no-solubility`, never given a guess.

use crate::species::{self, SpeciesData};

/// One measured solubility series in pure water.
#[derive(Debug, Clone, Copy)]
pub struct OrganicSolubilityRow {
    pub species: &'static str,
    /// (temperature K, saturation mole fraction of the solute) pairs,
    /// ascending in temperature.
    pub mole_fraction: &'static [(f64, f64)],
    pub source: &'static str,
}

/// The reviewed rows.
pub const ROWS: &[OrganicSolubilityRow] = &[
    // L-ascorbic acid in water, gravimetric, 293-323 K: Shalmashi, A.;
    // Eliassi, A. "Solubility of L-(+)-ascorbic acid in water, ethanol,
    // methanol, propan-2-ol, acetone, acetonitrile, ethyl acetate, and
    // tetrahydrofuran from (293 to 323) K." J. Chem. Eng. Data 2008, 53(6),
    // 1332-1334, doi:10.1021/je800056h. Values as deposited in NIST
    // ThermoML for that DOI (expanded uncertainty 2.4e-4 in x at 293 K).
    // x = 0.02896 at 293 K is about 29 g per 100 g water.
    OrganicSolubilityRow {
        species: "ascorbic_acid",
        mole_fraction: &[
            (293.0, 0.02896),
            (298.0, 0.03307),
            (303.0, 0.03779),
            (308.0, 0.04378),
            (313.0, 0.05037),
            (318.0, 0.05794),
            (323.0, 0.06633),
        ],
        source:
            "Shalmashi & Eliassi, J. Chem. Eng. Data 53(6), 1332-1334 (2008), doi:10.1021/je800056h",
    },
    // Maltose in water, gravimetric, 278.2-298.2 K, solid phase D-(+)-maltose
    // monohydrate: Gong, X.; Wang, C.; Zhang, L.; Qu, H. "Solubility of
    // Xylose, Mannose, Maltose Monohydrate, and Trehalose Dihydrate in
    // Ethanol-Water Solutions." J. Chem. Eng. Data 2012, 57(11), 3264-3269,
    // doi:10.1021/je300885g. Read from the NIST ThermoML deposit for that
    // DOI (trc.nist.gov/ThermoML/10.1021/je300885g.xml, data set 5,
    // maltose monohydrate + water): mass fraction of the MONOHYDRATE
    // w = 0.4106 (278.2 K), 0.4424 (288.2 K), 0.4792 (298.2 K; expanded
    // uncertainty 0.0746 at 95 %). The registry's maltose is anhydrous
    // C12H22O11, so the hydrate water counts as solvent:
    // x = (w/360.31) / ((w/360.31) + (1-w)/18.015 + w/360.31),
    // giving the mole fractions below (5 significant figures);
    // `maltose_row_reproduces_the_deposited_mass_fractions` re-derives w.
    // Above 298.2 K (the 37 °C digestion) the 298.2 K value is held, a
    // lower bound since the solubility rises with temperature.
    OrganicSolubilityRow {
        species: "maltose",
        mole_fraction: &[(278.2, 0.032_56), (288.2, 0.036_75), (298.2, 0.042_13)],
        source: "Gong, Wang, Zhang & Qu, J. Chem. Eng. Data 57(11), 3264-3269 (2012), doi:10.1021/je300885g (NIST ThermoML deposit, maltose monohydrate in water)",
    },
];

/// The reviewed row for `key`, if any.
pub fn row(key: &str) -> Option<&'static OrganicSolubilityRow> {
    ROWS.iter().find(|row| row.species == key)
}

fn mole_fraction_at(row: &OrganicSolubilityRow, t_k: f64) -> Option<f64> {
    let points = row.mole_fraction;
    let first = points.first()?;
    let last = points.last()?;
    if !t_k.is_finite() || t_k <= first.0 {
        return Some(first.1);
    }
    if t_k >= last.0 {
        return Some(last.1);
    }
    points.windows(2).find_map(|pair| {
        let (t0, x0) = pair[0];
        let (t1, x1) = pair[1];
        (t_k >= t0 && t_k <= t1).then(|| x0 + (x1 - x0) * (t_k - t0) / (t1 - t0))
    })
}

/// Grams of `key` that 100 mL of pure water holds at `t_k`, from a
/// reviewed row; `None` where there is no row (never a guess).
pub fn g_per_100_ml(key: &str, t_k: f64) -> Option<f64> {
    let row = row(key)?;
    let x = mole_fraction_at(row, t_k)?;
    if !(0.0..1.0).contains(&x) {
        return None;
    }
    let solute = species::lookup_key(key)?;
    let water = species::lookup_key("water")?;
    if water.molar_mass <= 0.0 || water.density <= 0.0 {
        return None;
    }
    Some(x / (1.0 - x) * solute.molar_mass / water.molar_mass * 100.0 * water.density)
}

/// A registry organic molecular solid: neutral, carbon AND hydrogen, and
/// built only from non-metals. Salts of organic acids (sodium acetate) and
/// metal complexes (chlorophyll) are not in this family — their
/// dissolution belongs to the ionic routes.
pub fn is_organic_molecular(data: &SpeciesData) -> bool {
    const NON_METALS: &[&str] = &["C", "H", "O", "N", "S", "P", "F", "Cl", "Br", "I"];
    let Ok(parsed) = crate::stoich::parse_formula(data.formula) else {
        return false;
    };
    parsed.charge == 0.0
        && parsed.counts.contains_key("C")
        && parsed.counts.contains_key("H")
        && parsed
            .counts
            .keys()
            .all(|element| NON_METALS.contains(&element.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascorbic_acid_row_converts_to_about_29_g_per_100_ml_at_293_k() {
        let g = g_per_100_ml("ascorbic_acid", 293.0).expect("reviewed row");
        // x/(1-x)·M/Mw·100 = 0.02896/0.97104·176.124/18.015·100 ≈ 29.2 g/100 g,
        // times the registry water density (≈1 g/mL).
        assert!(g > 27.0 && g < 31.0, "{g}");
        let warm = g_per_100_ml("ascorbic_acid", 310.0).unwrap();
        assert!(warm > g);
    }

    #[test]
    fn maltose_row_reproduces_the_deposited_mass_fractions() {
        // Back from x (anhydrous maltose with hydrate water as solvent) to
        // the deposited monohydrate mass fraction w.
        let (m_hydrate, m_water) = (360.31, 18.015);
        for ((_, x), w) in row("maltose")
            .unwrap()
            .mole_fraction
            .iter()
            .zip([0.4106, 0.4424, 0.4792])
        {
            let n = *x; // per mole of solution
            let water_free = 1.0 - x - n; // solvent water minus hydrate water
            let mass_hydrate = n * m_hydrate;
            let back = mass_hydrate / (mass_hydrate + water_free * m_water);
            assert!((back - w).abs() < 2e-4, "{back} vs {w}");
        }
        // Far above the 0.2 g in 50 mL of the starch-digestion control.
        let g = g_per_100_ml("maltose", 298.2).unwrap();
        assert!(g > 70.0 && g < 95.0, "{g}");
    }

    #[test]
    fn a_species_without_a_row_has_no_number() {
        assert!(g_per_100_ml("dehydroascorbic_acid", 298.15).is_none());
        assert!(g_per_100_ml("cellulose", 298.15).is_none());
    }
}
