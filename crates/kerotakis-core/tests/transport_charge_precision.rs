//! Supplementary signed extensive-coordinate and exact-endpoint controls.
use kerotakis_core::*;

fn cell(id: usize, charge: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(id), "charge cell");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.solute_charge = charge;
    v
}

#[test]
fn both_signs_of_charge_underflow_and_lost_increment_refuse_atomically() {
    for sign in [-1.0, 1.0] {
        for (held, incoming) in [
            (0.0, sign * f64::from_bits(1)),
            (sign, 1e-20),
            (sign, -1e-20),
        ] {
            let mut chain = CellChain::new(vec![cell(0, held)]).unwrap();
            let before = serde_json::to_value(chain.cells()).unwrap();
            assert!(chain.advance(&cell(9, incoming), 0.5).is_err());
            assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
        }
    }
}

#[test]
fn exact_signed_cancellation_is_a_supported_charge_balance() {
    for sign in [-1.0, 1.0] {
        let mut chain = CellChain::new(vec![cell(0, sign)]).unwrap();
        let step = chain.advance(&cell(9, -sign), 0.5).unwrap();
        assert_eq!(chain.total_solute_charge(), 0.0);
        assert_eq!(step.injected.solute_charge, -0.5 * sign);
        assert_eq!(step.effluent.solute_charge, 0.5 * sign);
    }
}

#[test]
fn full_positive_subnormal_parcels_remain_supported() {
    let tiny = f64::from_bits(1);
    let tracer = SpeciesId::new("passive-tracer");
    let mut old = cell(0, tiny);
    old.deposit(tracer.clone(), Moles(tiny), Phase::Aqueous);
    let mut incoming = cell(9, -tiny);
    incoming.deposit(tracer.clone(), Moles(tiny), Phase::Aqueous);
    let mut chain = CellChain::new(vec![old]).unwrap();
    let step = chain.advance(&incoming, 1.0).unwrap();
    assert_eq!(step.effluent.moles_of(&tracer).0, tiny);
    assert_eq!(step.injected.moles_of(&tracer).0, tiny);
    assert_eq!(chain.cells()[0].moles_of(&tracer).0, tiny);
    assert_eq!(step.effluent.solute_charge, tiny);
    assert_eq!(chain.total_solute_charge(), -tiny);
}
