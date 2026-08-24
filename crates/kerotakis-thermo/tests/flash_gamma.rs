//! CAP-16's consistency proofs: with γ(x, T) live in every path, the
//! bubble, dew and flash calculations must agree with *each other* —
//! thermodynamic consistency, not just self-consistency, which is
//! exactly the check the UNIFAC divergence bug taught this crate to
//! demand.

use kerotakis_thermo::vle::*;

/// Boil a liquid, condense its vapour: you must land back on the same
/// temperature and the same liquid. Bubble and dew are inverse
/// questions, and with γ(x, T) on both sides the answers have to close.
#[test]
fn bubble_and_dew_are_inverse_questions() {
    for x in [0.05, 0.3, 0.6] {
        let bp = ethanol_water_bubble_point(x, ATMOSPHERE_KPA).expect("bubble");
        let dp = ethanol_water_dew_point(bp.y[0], ATMOSPHERE_KPA).expect("dew");
        assert!(
            (dp.t_celsius - bp.t_celsius).abs() < 0.05,
            "x = {x}: bubble at {:.3} °C but dew of its vapour at {:.3} °C",
            bp.t_celsius,
            dp.t_celsius
        );
        assert!(
            (dp.x[0] - x).abs() < 0.005,
            "x = {x}: condensing the vapour must recover the liquid, got {:.4}",
            dp.x[0]
        );
    }
}

/// At the azeotrope the roundtrip is a fixed point: the dew of azeotropic
/// vapour is the azeotropic liquid, at the azeotropic temperature.
#[test]
fn the_azeotrope_is_a_fixed_point_of_dew() {
    let dp = ethanol_water_dew_point(0.894, ATMOSPHERE_KPA).expect("dew");
    assert!(
        (dp.x[0] - 0.894).abs() < 0.005,
        "azeotropic vapour condenses to itself, got x = {:.4}",
        dp.x[0]
    );
    assert!(
        (dp.t_celsius - 78.1).abs() < 0.5,
        "azeotrope boils near 78.1 °C, got {:.2}",
        dp.t_celsius
    );
}

/// Between its bubble and dew temperatures a feed splits two-phase, and
/// the split brackets the feed: liquid leaner, vapour richer.
#[test]
fn flash_splits_between_bubble_and_dew() {
    let z = 0.3;
    let bp = ethanol_water_bubble_point(z, ATMOSPHERE_KPA).expect("bubble");
    let dp = ethanol_water_dew_point(z, ATMOSPHERE_KPA).expect("dew");
    assert!(
        dp.t_celsius > bp.t_celsius,
        "dew above bubble for a mixture"
    );

    let mid = 0.5 * (bp.t_celsius + dp.t_celsius);
    let f = ethanol_water_tp_flash(z, ATMOSPHERE_KPA, mid).expect("flash");
    assert!(
        f.vapour_fraction > 0.02 && f.vapour_fraction < 0.98,
        "mid-way between bubble and dew must be two-phase, got V = {:.3}",
        f.vapour_fraction
    );
    assert!(
        f.x[0] < z && z < f.y[0],
        "the split must bracket the feed: x = {:.3}, z = {z}, y = {:.3}",
        f.x[0],
        f.y[0]
    );

    // Approaching the bubble point from above, the vapour fraction
    // vanishes and the first bubble matches the bubble-point vapour.
    let near_bubble =
        ethanol_water_tp_flash(z, ATMOSPHERE_KPA, bp.t_celsius + 0.05).expect("flash near bubble");
    assert!(
        near_bubble.vapour_fraction < 0.05,
        "just above the bubble point almost everything is liquid, got V = {:.3}",
        near_bubble.vapour_fraction
    );
    assert!(
        (near_bubble.y[0] - bp.y[0]).abs() < 0.02,
        "the first bubble matches the bubble-point vapour: {:.3} vs {:.3}",
        near_bubble.y[0],
        bp.y[0]
    );
}
