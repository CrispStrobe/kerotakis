//! Nitrogen oxides after the copper (2026-11-27): NO₂ ⇌ N₂O₄, NO + O₂,
//! NO₂ into water, and nitrous acid. Each rule has a positive control and
//! a negative control (the condition its source requires is withheld).

use kerotakis_core::*;

fn jar(litres: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "flask");
    v.headspace = Headspace::Sealed {
        volume: Liters(litres),
    };
    v
}

fn put(v: &mut Vessel, key: &str, n: f64, phase: Phase) {
    v.deposit(SpeciesId::new(key), Moles(n), phase);
}

fn all(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn gas(v: &Vessel, key: &str) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Gas)
        .map(|p| p.moles.0)
        .sum()
}

fn evolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == key => moles.0,
            _ => 0.0,
        })
        .sum()
}

fn note_keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key.clone()),
            _ => None,
        })
        .collect()
}

fn apply(v: &mut Vessel) -> (bool, Vec<Event>) {
    v.solute_charge = displacement::solute_charge(v);
    let mut events = Vec::new();
    let fired = nitrogen_oxides::apply(v, &mut events);
    (fired, events)
}

/// Sum of the transmitted sRGB channels: larger is paler.
fn brightness(v: &Vessel) -> u32 {
    let [r, g, b] = gas_colour::headspace_colour(v)
        .expect("coloured headspace")
        .srgb
        .expect("cited");
    u32::from(r) + u32::from(g) + u32::from(b)
}

// ── 2 NO2 <-> N2O4 ───────────────────────────────────────────────────

#[test]
fn cooling_a_brown_jar_raises_the_dinitrogen_tetroxide_share_and_fades_it() {
    let mut v = jar(0.5);
    put(&mut v, "NO2", 0.01, Phase::Gas);
    v.temperature = Kelvin(323.15);
    let warm = nitrogen_oxides::dimer_share(&v).unwrap();
    let warm_light = brightness(&v);
    v.temperature = Kelvin(273.15);
    let cold = nitrogen_oxides::dimer_share(&v).unwrap();
    let cold_light = brightness(&v);
    assert!(cold > warm, "dimer share cold {cold} vs warm {warm}");
    assert!(cold_light > warm_light, "{cold_light} vs {warm_light}");
    // The pool is untouched: dimerisation is a split, not a writer.
    assert!((all(&v, "NO2") - 0.01).abs() < 1e-15);
}

#[test]
fn a_dilute_nitrogen_dioxide_pool_is_mostly_monomer() {
    // Le Chatelier on dilution: the same gas in a bigger jar dimerises less.
    let mut small = jar(0.1);
    put(&mut small, "NO2", 0.01, Phase::Gas);
    let mut large = jar(10.0);
    put(&mut large, "NO2", 0.01, Phase::Gas);
    let s = nitrogen_oxides::dimer_share(&small).unwrap();
    let l = nitrogen_oxides::dimer_share(&large).unwrap();
    assert!(l < s && l < 0.25, "{l} {s}");
    // An open vessel owns no headspace, so there is no split to report.
    let mut open = Vessel::new(VesselId(0), "beaker");
    put(&mut open, "NO2", 0.01, Phase::Gas);
    assert!(nitrogen_oxides::dimer_share(&open).is_none());
}

// ── NO + O2 ──────────────────────────────────────────────────────────

#[test]
fn nitrogen_monoxide_turns_brown_with_the_air_in_a_jar() {
    let mut v = jar(0.5);
    put(&mut v, "NO", 0.002, Phase::Gas);
    put(&mut v, "O2", 0.004, Phase::Gas);
    put(&mut v, "N2", 0.016, Phase::Gas);
    assert!(
        gas_colour::headspace_colour(&v).is_none(),
        "NO is colourless"
    );
    let (fired, events) = apply(&mut v);
    assert!(fired);
    assert!(gas(&v, "NO") < 1e-12);
    assert!((gas(&v, "NO2") - 0.002).abs() < 1e-12);
    assert!((gas(&v, "O2") - 0.003).abs() < 1e-12);
    assert!(note_keys(&events).contains(&"not-modeled.nitric-oxide-oxidation".to_string()));
    assert!(gas_colour::headspace_colour(&v)
        .unwrap()
        .gases
        .contains(&"NO2".to_string()));
}

#[test]
fn nitrogen_monoxide_without_oxygen_stays_colourless() {
    // A nitrogen-filled jar, then a nitrogen purge: no oxygen either way.
    let mut v = jar(0.5);
    put(&mut v, "NO", 0.002, Phase::Gas);
    put(&mut v, "N2", 0.02, Phase::Gas);
    assert!(!nitrogen_oxides::fires(&v));
    let (fired, _) = apply(&mut v);
    assert!(!fired);
    assert!((gas(&v, "NO") - 0.002).abs() < 1e-15);
    assert!(gas(&v, "NO2") < 1e-15);
    assert!(gas_colour::headspace_colour(&v).is_none());

    v.headspace = Headspace::Swept {
        pressure: Pascal::ATMOSPHERIC,
    };
    put(&mut v, "O2", 0.004, Phase::Gas);
    assert!(
        !nitrogen_oxides::fires(&v),
        "a swept vessel owns no gas to react"
    );
}

#[test]
fn copper_in_dilute_nitric_acid_in_an_open_beaker_browns_only_at_the_mouth() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "water", 0.555, Phase::Liquid);
    put(&mut v, "HNO3", 0.02, Phase::Aqueous);
    put(&mut v, "Cu", 0.003, Phase::Solid);
    v.solute_charge = displacement::solute_charge(&v);
    let mut events = Vec::new();
    assert!(concentrated_sulfuric::apply(&mut v, &mut events));
    nitrogen_oxides::apply(&mut v, &mut events);
    assert!((evolved(&events, "NO") - 0.002).abs() < 1e-9);
    // The room air is outside the ledger: said, not booked.
    assert!(note_keys(&events).contains(&"not-modeled.nitric-oxide-meets-air".to_string()));
    assert!(evolved(&events, "NO2") < 1e-12);
    assert!(all(&v, "NO2") < 1e-12);
}

// ── NO2 into water ───────────────────────────────────────────────────

#[test]
fn nitrogen_dioxide_dissolves_in_water_as_nitric_and_nitrous_acid() {
    let mut v = jar(0.5);
    put(&mut v, "water", 5.55, Phase::Liquid);
    put(&mut v, "NO2", 0.002, Phase::Gas);
    let n_before = 0.002;
    let (fired, events) = apply(&mut v);
    assert!(fired);
    assert!(gas(&v, "NO2") < 2e-4, "left {}", gas(&v, "NO2"));
    assert!(all(&v, "NO3-") > 0.0);
    assert!(note_keys(&events).contains(&"not-modeled.nitrogen-dioxide-absorption".to_string()));
    // Nitrogen closes over gas, nitrate, nitrite and any nitrogen monoxide.
    let n = all(&v, "NO2") + all(&v, "NO3-") + all(&v, "NO2-") + all(&v, "NO");
    assert!((n - n_before).abs() < 1e-12, "N {n}");
    // Charge: the acid booked is the nitrate plus nitrite.
    let charge = all(&v, "H+") - all(&v, "NO3-") - all(&v, "NO2-");
    assert!(charge.abs() < 1e-12, "{charge}");
}

#[test]
fn dry_nitrogen_dioxide_is_not_absorbed() {
    let mut v = jar(0.5);
    put(&mut v, "NO2", 0.002, Phase::Gas);
    assert!(!nitrogen_oxides::fires(&v));
    let (fired, _) = apply(&mut v);
    assert!(!fired);
    assert!((gas(&v, "NO2") - 0.002).abs() < 1e-15);
}

// ── nitrous acid ─────────────────────────────────────────────────────

#[test]
fn acidified_nitrite_decomposes_to_nitrate_and_nitrogen_monoxide() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "water", 5.55, Phase::Liquid);
    put(&mut v, "Na+", 0.03, Phase::Aqueous);
    put(&mut v, "NO2-", 0.03, Phase::Aqueous);
    put(&mut v, "H+", 0.03, Phase::Aqueous);
    put(&mut v, "Cl-", 0.03, Phase::Aqueous);
    let (fired, events) = apply(&mut v);
    assert!(fired);
    let no = evolved(&events, "NO");
    assert!(no > 1e-4, "NO {no}");
    // 3 NO2- + 2 H+ -> NO3- + 2 NO + H2O: nitrogen and charge close.
    let n = all(&v, "NO2-") + all(&v, "NO3-") + no;
    assert!((n - 0.03).abs() < 1e-12, "N {n}");
    assert!((all(&v, "NO3-") - no / 2.0).abs() < 1e-12);
    assert!(note_keys(&events).contains(&"not-modeled.nitrous-acid-decomposition".to_string()));
}

#[test]
fn neutral_nitrite_is_stable() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    put(&mut v, "water", 5.55, Phase::Liquid);
    put(&mut v, "Na+", 0.03, Phase::Aqueous);
    put(&mut v, "NO2-", 0.03, Phase::Aqueous);
    assert!(!nitrogen_oxides::fires(&v));
    let (fired, events) = apply(&mut v);
    assert!(!fired);
    assert!(evolved(&events, "NO") < 1e-15);
    assert!((all(&v, "NO2-") - 0.03).abs() < 1e-15);
}

#[test]
fn a_sealed_jar_holds_back_nitrous_acid_decomposition() {
    // Nitrogen monoxide pressure builds up and the equilibrium stops short.
    let mk = |sealed: bool| {
        let mut v = if sealed {
            jar(0.01)
        } else {
            Vessel::new(VesselId(0), "beaker")
        };
        put(&mut v, "water", 5.55, Phase::Liquid);
        put(&mut v, "Na+", 0.3, Phase::Aqueous);
        put(&mut v, "NO2-", 0.3, Phase::Aqueous);
        put(&mut v, "H+", 0.3, Phase::Aqueous);
        put(&mut v, "Cl-", 0.3, Phase::Aqueous);
        apply(&mut v);
        all(&v, "NO3-")
    };
    let (open, sealed) = (mk(false), mk(true));
    assert!(sealed < open, "sealed {sealed} open {open}");
}

// ── headspace pressure counts molecules ──────────────────────────────

#[test]
fn a_dimerised_jar_reads_the_molecule_pressure_not_the_pool_pressure() {
    // 0.04 mol NO2 units in 0.5 L at 25 C: pool pressure 1.98 bar, but at
    // Kp 6.7 bar^-1 roughly two thirds of the pool is dimer, so the real
    // pressure is n_M + n_D, between half and all of the pool count.
    let mut v = jar(0.5);
    put(&mut v, "NO2", 0.04, Phase::Gas);
    v.temperature = Kelvin(298.15);
    v.refresh_pressure();
    let pool_pa = 0.04 * 8_314.462_618 * 298.15 / 0.5;
    let m = nitrogen_oxides::monomer_moles(0.04, 298.15, 0.5);
    let molecules = m + 0.5 * (0.04 - m);
    let expect = molecules * 8_314.462_618 * 298.15 / 0.5;
    assert!(
        (v.pressure.0 - expect).abs() / expect < 1e-9,
        "{:?}",
        v.pressure
    );
    assert!(v.pressure.0 < 0.8 * pool_pa && v.pressure.0 > 0.5 * pool_pa);
    // Negative control: a non-associating gas reads its full pool count.
    let mut n2 = jar(0.5);
    put(&mut n2, "N2", 0.04, Phase::Gas);
    n2.refresh_pressure();
    assert!((n2.pressure.0 - pool_pa).abs() / pool_pa < 1e-12);
}

#[test]
fn a_piston_jar_volume_counts_each_dimer_once() {
    let mut v = Vessel::new(VesselId(0), "syringe");
    v.headspace = Headspace::PressureControlled {
        pressure: Pascal(101_325.0),
        volume: Liters(1.0),
    };
    put(&mut v, "NO2", 0.04, Phase::Gas);
    v.temperature = Kelvin(298.15);
    v.refresh_pressure();
    let Headspace::PressureControlled { volume, .. } = v.headspace else {
        unreachable!()
    };
    // Self-consistent: the molecule count at that volume fills it at 1 atm.
    let n = v.gas_molecules_in(298.15, volume.0);
    let back = n * 8_314.462_618 * 298.15 / 101_325.0;
    assert!(
        (back - volume.0).abs() / volume.0 < 1e-9,
        "{volume:?} {back}"
    );
    // Bounded by the all-dimer and all-monomer volumes.
    let full = 0.04 * 8_314.462_618 * 298.15 / 101_325.0;
    assert!(volume.0 < full && volume.0 > 0.5 * full);
}

// ── N2O4 condensation caveat ─────────────────────────────────────────

#[test]
fn saturation_line_passes_through_the_boiling_point() {
    let p = nitrogen_oxides::n2o4_saturation_bar(nitrogen_oxides::N2O4_BOILING_K).unwrap();
    assert!((p - 1.01325).abs() < 1e-12);
    // Colder means lower vapour pressure; the line refuses above its range.
    assert!(nitrogen_oxides::n2o4_saturation_bar(273.15).unwrap() < 0.5);
    assert!(nitrogen_oxides::n2o4_saturation_bar(400.0).is_none());
}

#[test]
fn a_concentrated_jar_in_ice_flags_condensing_dinitrogen_tetroxide() {
    // 0.1 mol NO2 units in 0.5 L at 0 C: nearly all dimer at ~2.2 bar,
    // above the ~0.41 bar saturation line, so liquid N2O4 would form.
    let mut v = jar(0.5);
    put(&mut v, "NO2", 0.1, Phase::Gas);
    v.temperature = Kelvin(273.15);
    assert!(nitrogen_oxides::n2o4_supersaturation(&v).is_some());
    let mut events = Vec::new();
    nitrogen_oxides::apply(&mut v, &mut events);
    assert!(
        note_keys(&events).contains(&"not-modeled.dinitrogen-tetroxide-condensation".to_string())
    );
    // Negative control: the same jar warm, or a dilute jar in ice, stays gas.
    v.temperature = Kelvin(323.15);
    assert!(nitrogen_oxides::n2o4_supersaturation(&v).is_none());
    let mut dilute = jar(5.0);
    put(&mut dilute, "NO2", 0.002, Phase::Gas);
    dilute.temperature = Kelvin(273.15);
    assert!(nitrogen_oxides::n2o4_supersaturation(&dilute).is_none());
}
