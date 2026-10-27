//! Its own test binary on purpose: no dissolution pricer is registered
//! here, so the registry's curated `dissolution_enthalpy_kj` is the only
//! price — and KNO3 has none on main. A curve salt without a price must
//! refuse rather than crystallise heat-free.

use kerotakis_core::crystallisation::{self, Refusal};
use kerotakis_core::*;

#[test]
fn a_curve_salt_with_no_priced_dissolution_heat_refuses() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(
        SpeciesId::new("water"),
        Moles(2.775_464_890_369_136_6),
        Phase::Liquid,
    );
    v.deposit(
        SpeciesId::new("K+"),
        Moles(0.296_726_513_107_399_17),
        Phase::Aqueous,
    );
    v.deposit(
        SpeciesId::new("NO3-"),
        Moles(0.296_726_513_107_399_17),
        Phase::Aqueous,
    );
    v.temperature = Kelvin(321.76);
    assert!(crystallisation::dissolution_enthalpy_kj("KNO3").is_none());
    let from = v.temperature.0;
    let sensible = |t: f64, _n: f64| v.energy_between(t, from);
    assert!(matches!(
        crystallisation::cool_against_curve(
            &v,
            crystallisation::curve_for("KNO3").unwrap(),
            12_000.0,
            &sensible
        ),
        Err(Refusal::Unpriced { salt: "KNO3" })
    ));
}
