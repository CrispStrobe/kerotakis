//! Size probe: the smallest program that reaches a feos PC-SAFT bubble
//! point from an exported wasm symbol.
//!
//! Parameters are built in code rather than read from JSON so the probe
//! measures the *engine*, not serde_json plus a filesystem shim that a
//! browser build would not have anyway. The numbers are the two published
//! PC-SAFT records for ethanol and water (Gross & Sadowski 2002,
//! doi:10.1021/ie010954d) — hard-coded here only so the probe links; the
//! comparison corpus reads its parameters from file.

use feos::pcsaft::{PcSaft, PcSaftAssociationRecord, PcSaftParameters, PcSaftRecord};
use feos_core::parameter::{AssociationRecord, Identifier, PureRecord};
use feos_core::{PhaseEquilibrium, SolverOptions};
use nalgebra::DVector;
use quantity::{KELVIN, PASCAL};
use std::sync::Arc;

type Record = PureRecord<PcSaftRecord, PcSaftAssociationRecord>;

fn record(
    name: &str,
    mw: f64,
    m: f64,
    sigma: f64,
    epsilon_k: f64,
    kappa_ab: f64,
    epsilon_k_ab: f64,
) -> Record {
    let id = Identifier {
        name: Some(name.to_string()),
        ..Default::default()
    };
    PureRecord::with_association(
        id,
        mw,
        PcSaftRecord::new(m, sigma, epsilon_k, 0.0, 0.0, None, None, None),
        vec![AssociationRecord::new(
            Some(PcSaftAssociationRecord::new(kappa_ab, epsilon_k_ab)),
            1.0,
            1.0,
            0.0,
        )],
    )
}

/// Bubble-point temperature in kelvin of an ethanol/water liquid at
/// `x_ethanol` and `p_pa`, or a negative number if the solve failed.
#[no_mangle]
pub extern "C" fn bubble_point_k(x_ethanol: f64, p_pa: f64) -> f64 {
    let records = vec![
        record("ethanol", 46.069, 2.3827, 3.1771, 198.24, 0.032384, 2653.4),
        record("water", 18.015, 1.0656, 3.0007, 366.51, 0.034868, 2500.7),
    ];
    let params = match PcSaftParameters::new(records, vec![]) {
        Ok(p) => p,
        Err(_) => return -1.0,
    };
    let eos = Arc::new(PcSaft::new(params));
    let x = DVector::from_vec(vec![x_ethanol, 1.0 - x_ethanol]);
    match PhaseEquilibrium::bubble_point(
        &eos,
        p_pa * PASCAL,
        &x,
        None,
        None,
        (SolverOptions::default(), SolverOptions::default()),
    ) {
        Ok(vle) => vle.vapor().temperature.convert_into(KELVIN),
        Err(_) => -2.0,
    }
}
