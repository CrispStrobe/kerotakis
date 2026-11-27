//! Colours of dissolved metal–ligand complexes, as data rows.
//!
//! The aqueous solver names the complexes it forms (`Cu(NH3)4+2`,
//! `Fe(SCN)+2`, `CuCl4-2`, `Fe(CN)6-3` ...) but most are not registry
//! species, so the eye had no spectrum for them and the bench printed
//! "no absorption spectrum for Cu(NH3)4+2" over a colourless liquid. This
//! module is one table keyed by the complex's formula: adding a coloured
//! complex is adding a [`ComplexColour`] row. [`crate::solution_optics`]
//! adds the row's bands to the absorbance and stops listing the complex as
//! a spectral gap.
//!
//! Every row says how firmly it is known ([`Basis`]):
//! * `Cited` — the band's peak absorptivity at one wavelength is read from
//!   the named source (the same values the single-wavelength UV-vis method
//!   in [`crate::uvvis_absorptivity`] uses). The Gaussian width is
//!   editorial: it shapes the colour for the eye, it is not a measured
//!   band profile.
//! * `Editorial` — the colour the complex is reported to have, drawn as an
//!   editorial band. A typed note (`look.complex-colour-editorial`, EN/DE/FR)
//!   is shown whenever such a row colours a vessel.
//! * `LikeIon` — an ion pair or hydrolysis product of a coloured aqua ion
//!   drawn with that ion's registry spectrum (editorial, same note).
//!
//! No row is a speciation constant; which complexes exist and how much of
//! each is the solver's answer, never this table's.

use crate::spectrum::{bands, Spectrum};
use crate::stoich;

/// How a row's colour is established.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Basis {
    /// Peak (centre nm, ε L mol⁻¹ cm⁻¹) cited; width editorial.
    Cited(&'static str),
    /// Reported colour drawn as an editorial band; the string names the
    /// colour and the reference that reports it.
    Editorial(&'static str),
    /// Drawn with the registry spectrum of this aqua ion (editorial).
    LikeIon(&'static str),
}

/// One complex and its visible bands `(centre nm, peak ε, width nm)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexColour {
    pub species: &'static str,
    pub bands: &'static [(f64, f64, f64)],
    pub basis: Basis,
}

const CU_AMMINE: &str = "epsilon(640 nm) = 77 L mol-1 cm-1 for Cu(NH3)4 2+, Anal. Chim. Acta \
(2002) S0003267002001356 (snippet; the uvvis_absorptivity row)";
const FESCN: &str = "Frank & Oswalt, J. Am. Chem. Soc. 69 (1947) 1321, doi:10.1021/ja01198a026: \
447 nm, 4700 L mol-1 cm-1 (the uvvis_absorptivity row)";
const FERRICYANIDE: &str = "Dalton Trans. (2025), doi:10.1039/d4dt02916j: ferricyanide band \
centred near 420 nm, epsilon about 1040 M-1 cm-1 (snippet)";
const GE_CU: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 28 \
(copper(II): ammines blue to deep blue-violet, chloro complexes yellow-green to yellow, cuprate \
deep blue)";
const GE_CO_NI: &str =
    "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 26/27 \
(cobalt(II) octahedral complexes pink, tetrahedral CoCl4 2- deep blue; nickel ammines blue-violet)";
const CO_AQUA: &str = "Davis & Gibson, NBS Misc. Publ. 114 (1931), Table 6: Co(H2O)6 2+ \
epsilon(510 nm) = 4.75 L mol-1 cm-1 (the uvvis_absorptivity row)";
const GE_FE3: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 25 \
(aqueous iron(III) salts are yellow to brown from charge-transfer bands of the hydroxo and chloro \
complexes rising into the violet; the band is drawn editorially, not from a measured epsilon)";
const GE_FE: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 25 \
(hexacyanoferrate(II) pale yellow, thiocyanato complexes blood red)";
const GE_CR6: &str = "Greenwood & Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 23, \
section 23.3.3 (chromate yellow, dichromate orange; hydrogen chromate, dominant in dilute \
acidified dichromate, between them). The reviewed absorptivities (Burke, Deardorff & Menis, \
J. Res. NBS 76A (1972) 469) peak below 405 nm, so the visible tail is drawn editorially";
const BR2_AQ: &str = "Dissolved bromine (bromine water) is yellow-orange to brown, from the Br2 \
band near 390 nm ('Oxidation of Thiourea by Aqueous Bromine', snippet via lookchem PDF: Br2 171 \
mol-1 dm3 cm-1 at 390 nm, the uvvis_absorptivity row); band width as for the other cited rows";

/// The table. Append rows; never change a cited number without its source.
pub const TABLE: &[ComplexColour] = &[
    // --- copper(II) ammines
    ComplexColour {
        species: "Cu(NH3)4+2",
        bands: &[(640.0, 77.0, 70.0)],
        basis: Basis::Cited(CU_AMMINE),
    },
    ComplexColour {
        species: "Cu(NH3)3+2",
        bands: &[(660.0, 60.0, 75.0)],
        basis: Basis::Editorial(GE_CU),
    },
    ComplexColour {
        species: "Cu(NH3)2+2",
        bands: &[(690.0, 45.0, 80.0)],
        basis: Basis::Editorial(GE_CU),
    },
    ComplexColour {
        species: "CuNH3+2",
        bands: &[(740.0, 30.0, 85.0)],
        basis: Basis::Editorial(GE_CU),
    },
    // --- copper(II) hydroxo and cuprate
    ComplexColour {
        species: "Cu(OH)4-2",
        bands: &[(620.0, 50.0, 70.0)],
        basis: Basis::Editorial(GE_CU),
    },
    ComplexColour {
        species: "Cu(OH)3-",
        bands: &[(640.0, 40.0, 75.0)],
        basis: Basis::Editorial(GE_CU),
    },
    ComplexColour {
        species: "CuOH+",
        bands: &[],
        basis: Basis::LikeIon("Cu+2"),
    },
    ComplexColour {
        species: "Cu2(OH)2+2",
        bands: &[],
        basis: Basis::LikeIon("Cu+2"),
    },
    ComplexColour {
        species: "Cu(OH)2",
        bands: &[],
        basis: Basis::LikeIon("Cu+2"),
    },
    // --- copper(II) chloro complexes: a charge-transfer band in the violet
    // makes them yellow; mixed with the blue aqua ion the eye sees green.
    ComplexColour {
        species: "CuCl+",
        bands: &[],
        basis: Basis::LikeIon("Cu+2"),
    },
    ComplexColour {
        species: "CuCl2",
        bands: &[(385.0, 300.0, 30.0)],
        basis: Basis::Editorial(GE_CU),
    },
    ComplexColour {
        species: "CuCl3-",
        bands: &[(385.0, 500.0, 30.0)],
        basis: Basis::Editorial(GE_CU),
    },
    ComplexColour {
        species: "CuCl4-2",
        bands: &[(390.0, 700.0, 30.0)],
        basis: Basis::Editorial(GE_CU),
    },
    // --- iron(III) thiocyanate
    ComplexColour {
        species: "Fe(SCN)+2",
        bands: &[(447.0, 4700.0, 55.0)],
        basis: Basis::Cited(FESCN),
    },
    ComplexColour {
        species: "Fe(SCN)2+",
        bands: &[(460.0, 4700.0, 60.0)],
        basis: Basis::Editorial(GE_FE),
    },
    // --- hexacyanoferrates
    ComplexColour {
        species: "Fe(CN)6-3",
        bands: &[(420.0, 1040.0, 40.0)],
        basis: Basis::Cited(FERRICYANIDE),
    },
    ComplexColour {
        species: "Fe(CN)6-4",
        bands: &[(330.0, 300.0, 35.0)],
        basis: Basis::Editorial(GE_FE),
    },
    ComplexColour {
        species: "HFe(CN)6-3",
        bands: &[(330.0, 300.0, 35.0)],
        basis: Basis::Editorial(GE_FE),
    },
    ComplexColour {
        species: "H2Fe(CN)6-2",
        bands: &[(330.0, 300.0, 35.0)],
        basis: Basis::Editorial(GE_FE),
    },
    // --- iron(III) aqua, hydroxo and chloro species (yellow-brown)
    ComplexColour {
        species: "Fe+3",
        bands: &[(350.0, 2000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "FeOH+2",
        bands: &[(350.0, 2000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "Fe(OH)2+",
        bands: &[(350.0, 2000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "Fe2(OH)2+4",
        bands: &[(350.0, 4000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "Fe3(OH)4+5",
        bands: &[(350.0, 6000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "FeCl+2",
        bands: &[(350.0, 2000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "FeCl2+",
        bands: &[(350.0, 2000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    ComplexColour {
        species: "FeCl3",
        bands: &[(350.0, 2000.0, 45.0)],
        basis: Basis::Editorial(GE_FE3),
    },
    // --- iron(II) ion pairs: the pale green aqua ion's own spectrum
    ComplexColour {
        species: "FeCl+",
        bands: &[],
        basis: Basis::LikeIon("Fe+2"),
    },
    ComplexColour {
        species: "FeOH+",
        bands: &[],
        basis: Basis::LikeIon("Fe+2"),
    },
    // --- cobalt(II) and nickel(II)
    ComplexColour {
        species: "Co+2",
        bands: &[(510.0, 4.75, 55.0)],
        basis: Basis::Cited(CO_AQUA),
    },
    ComplexColour {
        species: "CoCl+",
        bands: &[(510.0, 5.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "CoCl2",
        bands: &[(560.0, 60.0, 70.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "CoCl3-",
        bands: &[(640.0, 200.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "CoCl4-2",
        bands: &[(690.0, 600.0, 50.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Co(NH3)+2",
        bands: &[(510.0, 5.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Co(NH3)2+2",
        bands: &[(510.0, 5.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Co(NH3)3+2",
        bands: &[(510.0, 5.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Co(NH3)4+2",
        bands: &[(510.0, 5.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Co(NH3)5+2",
        bands: &[(510.0, 5.0, 60.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Ni(NH3)2+2",
        bands: &[(600.0, 6.0, 80.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    ComplexColour {
        species: "Ni(NH3)6+2",
        bands: &[(580.0, 7.0, 80.0)],
        basis: Basis::Editorial(GE_CO_NI),
    },
    // --- chromium(VI): registry ions with no reviewed visible spectrum
    ComplexColour {
        species: "CrO4-2",
        bands: &[(370.0, 2000.0, 35.0)],
        basis: Basis::Editorial(GE_CR6),
    },
    ComplexColour {
        species: "HCrO4-",
        bands: &[(395.0, 1000.0, 40.0)],
        basis: Basis::Editorial(GE_CR6),
    },
    ComplexColour {
        species: "Cr2O7-2",
        bands: &[(420.0, 1500.0, 45.0)],
        basis: Basis::Editorial(GE_CR6),
    },
    // --- dissolved bromine: not a native species of the routed databases,
    // so its aqueous portion takes this row (`solution_optics`).
    ComplexColour {
        species: "Br2",
        bands: &[(390.0, 171.0, 55.0)],
        basis: Basis::Cited(BR2_AQ),
    },
];

/// The row for a native species name (formula-compared, so `CuNH3+2` and
/// `Cu(NH3)+2` are one row).
pub fn row_for(name: &str) -> Option<&'static ComplexColour> {
    let parsed = stoich::parse_formula(name).ok()?;
    TABLE.iter().find(|row| {
        row.species == name
            || stoich::parse_formula(row.species)
                .ok()
                .is_some_and(|f| f.counts == parsed.counts && f.charge == parsed.charge)
    })
}

/// The row's spectrum, ε per band.
pub fn spectrum(row: &ComplexColour) -> Option<Spectrum> {
    match row.basis {
        Basis::LikeIon(ion) => crate::species::lookup_key(ion)
            .and_then(|data| data.spectrum)
            .copied(),
        _ => Some(bands(row.bands)),
    }
}

/// Whether the row's colour is editorial (shown with the typed note).
pub fn is_editorial(row: &ComplexColour) -> bool {
    !matches!(row.basis, Basis::Cited(_))
}

/// The typed note shown when an editorial row colours a vessel.
pub fn editorial_note(species: Vec<String>) -> crate::phrase::Phrase {
    crate::phrase::Phrase::new(
        "look.complex-colour-editorial",
        "The colour of {species} is drawn from its reported colour as an editorial band, not from a measured spectrum.",
        vec![(
            "species".to_string(),
            crate::phrase::Slot::terms("species", species),
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_parses_and_has_a_spectrum() {
        for row in TABLE {
            assert!(
                stoich::parse_formula(row.species).is_ok(),
                "{}",
                row.species
            );
            let s = spectrum(row).unwrap_or_else(|| panic!("{} has no spectrum", row.species));
            assert!(s.iter().all(|e| e.is_finite() && *e >= 0.0));
            assert!(s.iter().any(|e| *e > 0.0), "{} draws nothing", row.species);
            if let Basis::Cited(source) = row.basis {
                assert!(!source.is_empty());
            }
        }
    }

    #[test]
    fn formula_spellings_find_one_row() {
        assert_eq!(row_for("CuNH3+2").unwrap().species, "CuNH3+2");
        assert_eq!(row_for("Cu(NH3)4+2").unwrap().species, "Cu(NH3)4+2");
        assert!(row_for("Cu+2").is_none(), "the aqua ion is the registry's");
        assert!(
            row_for("Zn(NH3)4+2").is_none(),
            "d10 zinc ammine is colourless"
        );
    }

    #[test]
    fn tetraammine_copper_is_blue_and_thiocyanate_is_red() {
        let colour = |name: &str, molar: f64| {
            let s = spectrum(row_for(name).unwrap()).unwrap();
            let a = s.map(|e| e * molar * crate::spectrum::BEAKER_PATH_CM);
            crate::spectrum::transmitted_colour(&a)
        };
        let cu = colour("Cu(NH3)4+2", 0.05);
        assert!(cu.b > cu.r, "tetraammine copper is blue: {cu:?}");
        let fe = colour("Fe(SCN)+2", 2e-4);
        assert!(fe.r > fe.b, "iron thiocyanate is red: {fe:?}");
        let pb = colour("Fe(CN)6-3", 0.01);
        assert!(pb.r > pb.b && pb.g > pb.b, "ferricyanide is yellow: {pb:?}");
    }

    #[test]
    fn chromium_vi_and_bromine_water_take_their_reported_colours() {
        let colour = |name: &str, molar: f64| {
            let row = row_for(name).unwrap();
            // Br2 has a cited peak (like Fe(SCN)+2); the Cr(VI) rows are editorial.
            assert_eq!(is_editorial(row), name != "Br2", "{name} basis");
            let s = spectrum(row).unwrap();
            let a = s.map(|e| e * molar * crate::spectrum::BEAKER_PATH_CM);
            crate::spectrum::transmitted_colour(&a)
        };
        let chromate = colour("CrO4-2", 1e-3);
        assert!(
            chromate.r > chromate.b && chromate.g > chromate.b,
            "chromate is yellow: {chromate:?}"
        );
        let dichromate = colour("Cr2O7-2", 1e-3);
        assert!(
            dichromate.r > dichromate.g && dichromate.g > dichromate.b,
            "dichromate is orange: {dichromate:?}"
        );
        let hydrogen = colour("HCrO4-", 1e-3);
        assert!(
            hydrogen.r > hydrogen.b && hydrogen.g > hydrogen.b,
            "hydrogen chromate is orange-yellow: {hydrogen:?}"
        );
        let bromine = colour("Br2", 0.02);
        assert!(bromine.r > bromine.b, "bromine water is yellow-orange to brown: {bromine:?}");
    }
}
