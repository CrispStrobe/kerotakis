//! The baseline for the size probe: the same exported symbol reaching
//! kerotakis-thermo's existing Antoine + UNIFAC bubble point, and nothing
//! else. `wasm-probe-feos.wasm` minus this is the code size feos adds.

use kerotakis_thermo::vle;

/// Bubble-point temperature in kelvin of an ethanol/water liquid at
/// `x_ethanol` and `p_pa`, or a negative number if the solve failed.
///
/// # Safety
/// Exported for the size probe only; it takes no pointers.
#[no_mangle]
pub extern "C" fn bubble_point_k(x_ethanol: f64, p_pa: f64) -> f64 {
    match vle::ethanol_water_bubble_point(x_ethanol, p_pa / 1000.0) {
        Some(bp) => bp.t_celsius + 273.15,
        None => -2.0,
    }
}
