//! CAP-4: Phase diagram contour generation.
//!
//! Compute Txy and Pxy boiling-point envelopes using the VLE module
//! and extract contour lines via the marching-squares algorithm.

use contour::ContourBuilder;

/// A point on a phase diagram.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct PhasePoint {
    pub x: f64, // mole fraction
    pub t: f64, // temperature (°C) or pressure (kPa)
}

/// Generate Txy diagram data: bubble and dew temperatures vs composition.
///
/// Returns (bubble_curve, dew_curve) as vectors of (x, T) points.
pub fn txy_envelope(
    bubble_fn: impl Fn(f64) -> Option<(f64, f64)>, // x → (T_bubble, y_vapor)
    n_points: usize,
) -> (Vec<PhasePoint>, Vec<PhasePoint>) {
    let mut bubble = Vec::with_capacity(n_points);
    let mut dew = Vec::with_capacity(n_points);

    for i in 0..=n_points {
        let x = i as f64 / n_points as f64;
        if let Some((t_bubble, y_vapor)) = bubble_fn(x) {
            bubble.push(PhasePoint { x, t: t_bubble });
            dew.push(PhasePoint { x: y_vapor, t: t_bubble });
        }
    }

    (bubble, dew)
}

/// Generate contour lines from a 2D grid of values.
///
/// `grid` is row-major (height × width), values at grid points.
/// Returns contour polygons at the specified threshold levels.
pub fn contour_lines(
    grid: &[f64],
    width: usize,
    height: usize,
    thresholds: &[f64],
) -> Vec<Vec<(f64, f64)>> {
    let builder = ContourBuilder::new(width, height, false);
    let mut all_lines = Vec::new();

    for &threshold in thresholds {
        let contours = builder.contours(grid, &[threshold]);
        if let Ok(features) = contours {
            for feature in features {
                let geom = feature.geometry();
                for ring in geom.0.iter() {
                    let points: Vec<(f64, f64)> = ring
                        .exterior()
                        .points()
                        .map(|p| (p.x(), p.y()))
                        .collect();
                    if !points.is_empty() {
                        all_lines.push(points);
                    }
                }
            }
        }
    }

    all_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn txy_envelope_generates_curves() {
        // Simple linear bubble point: T = 80 + 20*x
        let (bubble, dew) = txy_envelope(
            |x| Some((80.0 + 20.0 * x, x * 0.9)), // toy model
            10,
        );
        assert_eq!(bubble.len(), 11);
        assert!((bubble[0].t - 80.0).abs() < 0.1);
        assert!((bubble[10].t - 100.0).abs() < 0.1);
    }
}
