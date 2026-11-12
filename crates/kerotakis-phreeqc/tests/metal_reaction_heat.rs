//! A metal dissolving in acid is priced on the aqueous enthalpy basis
//! through the cited formation enthalpy of its master ion (2026-11-12).
use kerotakis_core::species::Phase;
use kerotakis_phreeqc::enthalpy::{species_enthalpy, METAL_MASTER_IONS};

fn h(key: &str, phase: Phase) -> f64 {
    species_enthalpy(key, phase, "wateq4f").unwrap_or_else(|e| panic!("{key}: {e:?}"))
}

#[test]
fn zinc_in_acid_releases_the_formation_enthalpy_of_zinc_ion() {
    // Zn(s) + 2 H+ -> Zn+2 + H2(g): products minus reactants on one basis.
    let dh = h("Zn+2", Phase::Aqueous) + h("H2", Phase::Gas) - h("Zn", Phase::Solid);
    assert!((dh - -153.39).abs() < 0.05, "{dh} kJ/mol");
}

#[test]
fn iron_displacing_copper_is_the_difference_of_the_two_ions() {
    // Fe(s) + Cu+2 -> Fe+2 + Cu(s): dfH(Fe+2) - dfH(Cu+2) = -87.86 - 64.9.
    let dh = h("Fe+2", Phase::Aqueous) + h("Cu", Phase::Solid)
        - h("Fe", Phase::Solid)
        - h("Cu+2", Phase::Aqueous);
    assert!((dh - (-87.86 - 64.9)).abs() < 0.05, "{dh} kJ/mol");
}

#[test]
fn every_metal_row_names_its_source() {
    for (metal, _, ion, dhf, source) in METAL_MASTER_IONS {
        assert!(dhf.is_finite() && !source.is_empty(), "{metal} {ion}");
    }
}
