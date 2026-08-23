//! The ideal layer, checked against things a bench can verify.

use kerotakis_thermo::vle::*;

/// Pure water boils at 100 °C at one atmosphere. If this is wrong, nothing
/// downstream is worth reading.
#[test]
fn water_boils_where_the_thermometer_says() {
    let mix = [Volatile {
        antoine: WATER,
        x: 1.0,
        gamma: 1.0,
    }];
    let bp = bubble_point(&mix, ATMOSPHERE_KPA).expect("water boils");
    assert!(
        (bp.t_celsius - 100.0).abs() < 0.3,
        "water should boil at 100 °C, got {:.3}",
        bp.t_celsius
    );
}

/// And ethanol at 78.4 °C.
#[test]
fn ethanol_boils_where_the_thermometer_says() {
    let mix = [Volatile {
        antoine: ETHANOL,
        x: 1.0,
        gamma: 1.0,
    }];
    let bp = bubble_point(&mix, ATMOSPHERE_KPA).expect("ethanol boils");
    assert!(
        (bp.t_celsius - 78.4).abs() < 0.5,
        "ethanol should boil at 78.4 °C, got {:.3}",
        bp.t_celsius
    );
}

/// Pressure moves the boiling point, which is the whole of why a pressure
/// cooker works and why water boils cold on a mountain.
#[test]
fn lower_pressure_boils_water_cooler() {
    let mix = [Volatile {
        antoine: WATER,
        x: 1.0,
        gamma: 1.0,
    }];
    // Roughly the pressure on top of Mont Blanc.
    let high = bubble_point(&mix, ATMOSPHERE_KPA).expect("sea level");
    let low = bubble_point(&mix, 55.0).expect("altitude");
    assert!(
        low.t_celsius < high.t_celsius - 15.0,
        "boiling should fall a long way by 55 kPa: {:.1} against {:.1}",
        low.t_celsius,
        high.t_celsius
    );
}

/// An ideal mixture has no azeotrope, and it is important that we can show
/// that rather than assert it.
///
/// Raoult's law makes y₁ − x₁ strictly one sign across the whole
/// composition axis: the more volatile component always enriches in the
/// vapour, so a tall enough column always reaches a pure product. Every
/// azeotrope in the world is a statement that γ ≠ 1, which is why this test
/// and the UNIFAC one are the same experiment run twice.
#[test]
fn an_ideal_mixture_separates_all_the_way() {
    let found = azeotrope(ETHANOL, WATER, ATMOSPHERE_KPA, |_, _| (1.0, 1.0));
    assert!(
        found.is_none(),
        "Raoult's law cannot produce an azeotrope, and got {found:?}"
    );
}

/// Ethanol enriches in the vapour under the ideal model — correctly, up to
/// the point where the real mixture stops, which the ideal model cannot
/// know about.
#[test]
fn the_more_volatile_component_comes_over_first() {
    let mix = [
        Volatile {
            antoine: ETHANOL,
            x: 0.1,
            gamma: 1.0,
        },
        Volatile {
            antoine: WATER,
            x: 0.9,
            gamma: 1.0,
        },
    ];
    let bp = bubble_point(&mix, ATMOSPHERE_KPA).expect("mixture boils");
    assert!(
        bp.y[0] > 0.1,
        "ethanol should enrich in the vapour: y = {:.3} from x = 0.1",
        bp.y[0]
    );
    assert!(!bp.azeotropic, "an ideal mixture is never azeotropic");
}

/// Antoine outside its fitted range answers `None` rather than a number.
#[test]
fn extrapolation_is_refused_rather_than_returned() {
    assert!(WATER.pressure_kpa(50.0).is_some());
    assert!(
        WATER.pressure_kpa(300.0).is_none(),
        "300 °C is far outside the 1–100 °C fit and must not be extrapolated"
    );
}
