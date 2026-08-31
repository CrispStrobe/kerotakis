//! The adapter prototype: a Kerotakis-trait-shaped wrapper over feos for
//! ONE calculation family (bubble point), to find out whether the boundary
//! `kerotakis-thermo` already draws is one feos can stand behind.
//!
//! The answer, written here rather than in the report because the code is
//! the evidence: **the seam holds, but the trait leaks its own model.**
//!
//! `FluidModel::bubble_point` takes `&[Volatile]`, and a `Volatile` is
//! `{ antoine, x, gamma }` — a *Raoult-shaped* component. Two of its three
//! fields are the ideal-model's own state: an Antoine correlation and an
//! activity coefficient. A SAFT backend has neither and needs neither; it
//! keys on species identity and mole fraction and derives the rest. So this
//! adapter must ignore `antoine` and `gamma` entirely and take only `x`,
//! matching components positionally against the parameter set it was built
//! with. That works, and it returns a `BubblePoint` the existing callers
//! can consume unchanged — but a trait whose argument type carries the
//! other model's parameters is a trait that will silently accept a
//! mismatched pairing.
//!
//! BRD-031 removed the trait's inherited calculation bodies. This adapter now
//! advertises bubble-point support only and explicitly refuses dew point, TP
//! flash, and saturation pressure. The spike therefore remains honest while
//! the component-identity seam described above is still unresolved.
//!
//! What BRD-032 would need before routing through this:
//!
//! 1. `FluidModel`'s component argument reduced to (species identity, mole
//!    fraction), with Antoine constants moved inside the Raoult implementation
//!    where they belong;
//! 2. the three default method bodies deleted, so a backend that cannot do a
//!    dew point has to say so rather than inherit someone else's answer.
//!
//! Both are small, mechanical changes to a small crate. Together they are the
//! main integration cost this spike found, and they are not large.

use feos::pcsaft::PcSaft;
use feos_core::{PhaseEquilibrium, SolverOptions};
use kerotakis_thermo::fluid::{
    FluidCapabilities, FluidModel, FluidModelError, FluidModelResult, FluidOperation,
};
use kerotakis_thermo::vle::{BubblePoint, DewPoint, FlashResult, VapourPressure, Volatile};
use nalgebra::DVector;
use quantity::{KELVIN, PASCAL};
use std::sync::Arc;

/// `vle.rs` keeps its azeotrope tolerance private, so the adapter restates
/// it rather than reaching into the crate — another small sign that the
/// trait boundary was drawn around one model rather than around the
/// calculation. Kept identical to `vle::AZEOTROPE_TOLERANCE` (1e-3) so the
/// two engines call an azeotrope at the same place.
const AZEOTROPE_TOLERANCE: f64 = 1e-3;

/// A feos PC-SAFT equation of state dressed as a `kerotakis_thermo::FluidModel`.
///
/// `components` records, positionally, which species the parameter set was
/// built for. It exists purely so a caller can be told what this model
/// actually is; the trait gives no way to check the `Volatile`s handed in
/// against it, which is the leak documented above.
pub struct FeosPcSaftFluid {
    eos: Arc<PcSaft>,
    pub components: Vec<String>,
}

impl FeosPcSaftFluid {
    pub fn new(eos: Arc<PcSaft>, components: Vec<String>) -> Self {
        Self { eos, components }
    }
}

impl FluidModel for FeosPcSaftFluid {
    fn name(&self) -> &'static str {
        "feos PC-SAFT (BRD-030 spike adapter)"
    }

    fn capabilities(&self) -> FluidCapabilities {
        FluidCapabilities {
            bubble_point: true,
            ..FluidCapabilities::default()
        }
    }

    fn bubble_point(
        &self,
        components: &[Volatile],
        pressure_kpa: f64,
    ) -> FluidModelResult<BubblePoint> {
        if components.len() != self.components.len() {
            return Ok(None);
        }
        let total: f64 = components.iter().map(|c| c.x).sum();
        if total <= 0.0 || pressure_kpa <= 0.0 {
            return Ok(None);
        }
        // Only `x` crosses the seam. `antoine` and `gamma` are the Raoult
        // model's own state and are deliberately dropped.
        let x = DVector::from_iterator(components.len(), components.iter().map(|c| c.x / total));
        let p = pressure_kpa * 1000.0 * PASCAL;

        // feos wants a temperature to start from and does not always find
        // one unaided. The ladder is the spike's own, not feos's: try
        // unaided first, then a few plausible bench temperatures.
        let opts = (SolverOptions::default(), SolverOptions::default());
        let mut vle = PhaseEquilibrium::bubble_point(&self.eos, p, &x, None, None, opts).ok();
        for t0 in [350.0, 300.0, 400.0, 250.0, 450.0, 200.0, 100.0] {
            if vle.is_some() {
                break;
            }
            vle = PhaseEquilibrium::bubble_point(&self.eos, p, &x, Some(t0 * KELVIN), None, opts)
                .ok();
        }
        let Some(vle) = vle else {
            return Ok(None);
        };

        let t_celsius = vle.vapor().temperature.convert_into(KELVIN) - 273.15;
        let y: Vec<f64> = vle.vapor().molefracs.iter().copied().collect();
        let azeotropic = x
            .iter()
            .zip(&y)
            .all(|(xi, yi)| (xi - yi).abs() < AZEOTROPE_TOLERANCE);
        Ok(Some(BubblePoint {
            t_celsius,
            y,
            azeotropic,
        }))
    }

    fn dew_point(
        &self,
        _components: &[Volatile],
        _pressure_kpa: f64,
    ) -> FluidModelResult<DewPoint> {
        Err(FluidModelError::unsupported(
            self.name(),
            FluidOperation::DewPoint,
        ))
    }

    fn tp_flash(
        &self,
        _components: &[Volatile],
        _pressure_kpa: f64,
        _t_celsius: f64,
    ) -> FluidModelResult<FlashResult> {
        Err(FluidModelError::unsupported(
            self.name(),
            FluidOperation::TpFlash,
        ))
    }

    fn saturation_pressure_kpa(
        &self,
        _correlation: &VapourPressure,
        _t_celsius: f64,
    ) -> FluidModelResult<f64> {
        Err(FluidModelError::unsupported(
            self.name(),
            FluidOperation::SaturationPressure,
        ))
    }
}
