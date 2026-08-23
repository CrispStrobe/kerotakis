//! The iron Pourbaix diagram, computed — held to the textbook topology.
//!
//! Not the textbook *numbers*: the regions' shapes follow the database's
//! own thermodynamics. What is pinned here is the qualitative map every
//! corrosion chapter draws — acid+oxidising is ferric, acid+reducing is
//! ferrous, alkaline+oxidising is the hydroxide precipitate — because a
//! diagram that gets that topology wrong is worse than none.

#![cfg(feature = "engine")]

use kerotakis_phreeqc::pourbaix::{diagram, outside_water_stability, water_stability_lines};
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn iron_diagram_has_the_textbook_topology() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let d = diagram(&mut eq, "Fe", 15, 13).expect("Fe grid computes");

    let at = |ph: f64, pe: f64| -> Option<&str> {
        let i_ph =
            d.ph.iter()
                .enumerate()
                .min_by(|a, b| (a.1 - ph).abs().total_cmp(&(b.1 - ph).abs()))
                .unwrap()
                .0;
        let i_pe =
            d.pe.iter()
                .enumerate()
                .min_by(|a, b| (a.1 - pe).abs().total_cmp(&(b.1 - pe).abs()))
                .unwrap()
                .0;
        d.label(i_pe, i_ph)
    };

    // Acid and strongly oxidising: dissolved iron(III).
    let ferric = at(1.0, 14.0).expect("acid/oxidising cell solves");
    assert!(
        ferric.starts_with("Fe+3") || ferric.starts_with("FeOH+2"),
        "acid+oxidising should be ferric, got {ferric}"
    );
    // Acid and mildly reducing: dissolved iron(II).
    let ferrous = at(2.0, 0.0).expect("acid/reducing cell solves");
    assert!(
        ferrous.starts_with("Fe+2"),
        "acid+reducing should be Fe+2, got {ferrous}"
    );
    // Alkaline and oxidising: the hydroxide precipitate rules.
    let rust = at(9.0, 8.0).expect("alkaline/oxidising cell solves");
    assert_eq!(
        rust, "Fe(OH)3(a)",
        "alkaline+oxidising should precipitate ferric hydroxide"
    );

    // The map is a map, not a monoculture: at least three distinct forms.
    let distinct = d.distinct();
    assert!(
        distinct.len() >= 3,
        "expected at least ferric/ferrous/hydroxide regions, got {distinct:?}"
    );
    // Refusals outside the water-stability field are physics — water
    // itself decomposes there, and the engine declining is the correct
    // answer. Refusals *inside* the field are genuine holes, and those
    // must be rare.
    let mut inside_refused = 0usize;
    for (j, &pe) in d.pe.iter().enumerate() {
        for (i, &ph) in d.ph.iter().enumerate() {
            if d.label(j, i).is_none() && !outside_water_stability(ph, pe) {
                inside_refused += 1;
            }
        }
    }
    assert!(
        inside_refused * 20 < d.labels.len(),
        "{inside_refused} refusals inside the water-stability field          (of {} cells) — those are real holes",
        d.labels.len()
    );

    // The stability lines bracket the interesting middle of the pe axis.
    let (upper, lower) = water_stability_lines(&d.ph);
    assert!((upper[0].1 - 20.75).abs() < 1e-9);
    assert!((lower[0].1 - 0.0).abs() < 1e-9);
}
