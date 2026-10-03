//! Supplementary independent quantum cases: positive is not always accurate.
use kerotakis_core::*;

fn cell(id: usize, tracer: f64, charge: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(id), "quantum cell");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.deposit(
        SpeciesId::new("passive-tracer"),
        Moles(tracer),
        Phase::Aqueous,
    );
    v.solute_charge = charge;
    v
}

#[test]
fn positive_but_quantized_subnormal_parcels_refuse_atomically() {
    let tiny = f64::from_bits(1);
    for (quanta, fraction) in [(5.0, 0.5), (22.0, 0.25)] {
        for in_inlet in [false, true] {
            let (held, arriving) = if in_inlet {
                (0.0, quanta * tiny)
            } else {
                (quanta * tiny, 0.0)
            };
            let mut chain = CellChain::new(vec![cell(0, held, 0.0)]).unwrap();
            let before = serde_json::to_value(chain.cells()).unwrap();
            assert!(chain.advance(&cell(9, arriving, 0.0), fraction).is_err());
            assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
        }
        for sign in [-1.0, 1.0] {
            let mut chain = CellChain::new(vec![cell(0, 0.0, sign * quanta * tiny)]).unwrap();
            let before = serde_json::to_value(chain.cells()).unwrap();
            assert!(chain.advance(&cell(9, 0.0, 0.0), fraction).is_err());
            assert_eq!(serde_json::to_value(chain.cells()).unwrap(), before);
        }
    }
}

#[test]
fn exactly_divisible_subnormal_parcels_conserve_flux_and_signed_charge() {
    let tiny = f64::from_bits(1);
    let tracer = SpeciesId::new("passive-tracer");
    for sign in [-1.0, 1.0] {
        let mut chain = CellChain::new(vec![cell(0, 8.0 * tiny, sign * 8.0 * tiny)]).unwrap();
        let step = chain.advance(&cell(9, 0.0, 0.0), 0.5).unwrap();
        assert_eq!(step.effluent.moles_of(&tracer).0, 4.0 * tiny);
        assert_eq!(chain.cells()[0].moles_of(&tracer).0, 4.0 * tiny);
        assert_eq!(step.effluent.solute_charge, sign * 4.0 * tiny);
        assert_eq!(chain.total_solute_charge(), sign * 4.0 * tiny);
    }
}
