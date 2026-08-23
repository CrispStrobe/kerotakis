//! CAP-8: Statistics — reproducible RNG and distributions.
//!
//! Seeded random number generation for Monte Carlo initial-rates
//! experiments and stochastic kinetics. A seeded run produces
//! bit-identical results across platforms.

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, Normal, Uniform};

/// A reproducible experiment runner with seeded RNG.
pub struct Experiment {
    rng: ChaCha20Rng,
}

impl Experiment {
    /// Create a new experiment with a fixed seed for reproducibility.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha20Rng::seed_from_u64(seed),
        }
    }

    /// Sample n values from a normal distribution N(mean, std_dev).
    pub fn normal_samples(&mut self, mean: f64, std_dev: f64, n: usize) -> Vec<f64> {
        let dist = Normal::new(mean, std_dev).unwrap();
        (0..n).map(|_| dist.sample(&mut self.rng)).collect()
    }

    /// Sample n values from a uniform distribution U(low, high).
    pub fn uniform_samples(&mut self, low: f64, high: f64, n: usize) -> Vec<f64> {
        let dist = Uniform::new(low, high).unwrap();
        (0..n).map(|_| dist.sample(&mut self.rng)).collect()
    }

    /// Compute mean and standard deviation of a sample.
    pub fn mean_std(values: &[f64]) -> (f64, f64) {
        let n = values.len() as f64;
        if n < 1.0 {
            return (f64::NAN, f64::NAN);
        }
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
        (mean, variance.sqrt())
    }

    /// Linear regression: y = a + b*x. Returns (a, b, r²).
    pub fn linear_regression(x: &[f64], y: &[f64]) -> (f64, f64, f64) {
        let n = x.len() as f64;
        let sx: f64 = x.iter().sum();
        let sy: f64 = y.iter().sum();
        let sxx: f64 = x.iter().map(|xi| xi * xi).sum();
        let sxy: f64 = x.iter().zip(y).map(|(xi, yi)| xi * yi).sum();
        let syy: f64 = y.iter().map(|yi| yi * yi).sum();

        let denom = n * sxx - sx * sx;
        if denom.abs() < 1e-30 {
            return (f64::NAN, f64::NAN, f64::NAN);
        }
        let b = (n * sxy - sx * sy) / denom;
        let a = (sy - b * sx) / n;

        let ss_res: f64 = x.iter().zip(y).map(|(xi, yi)| (yi - a - b * xi).powi(2)).sum();
        let ss_tot = n * syy - sy * sy;
        let r2 = if ss_tot.abs() > 1e-30 { 1.0 - n * ss_res / ss_tot } else { f64::NAN };

        (a, b, r2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_rng_is_deterministic() {
        let mut e1 = Experiment::new(42);
        let mut e2 = Experiment::new(42);
        let s1 = e1.normal_samples(0.0, 1.0, 10);
        let s2 = e2.normal_samples(0.0, 1.0, 10);
        assert_eq!(s1, s2, "same seed must give same samples");
    }

    #[test]
    fn mean_std_of_known_data() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, std) = Experiment::mean_std(&data);
        assert!((mean - 5.0).abs() < 1e-10);
        // Sample std with n-1: sqrt(32/7) ≈ 2.138
        assert!((std - 2.138).abs() < 0.01, "sample std = {std}");
    }

    #[test]
    fn linear_regression_perfect_line() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // y = 2x
        let (a, b, r2) = Experiment::linear_regression(&x, &y);
        assert!(a.abs() < 1e-10, "intercept = {a}");
        assert!((b - 2.0).abs() < 1e-10, "slope = {b}");
        assert!((r2 - 1.0).abs() < 1e-10, "R² = {r2}");
    }
}
