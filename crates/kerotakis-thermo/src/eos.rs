//! THERMO-007: Equation-of-state backend.
//!
//! Peng-Robinson cubic EOS for gas-phase PVT and fugacity calculations.
//! This is the approved starting point — more complex models (SRK, SAFT)
//! can implement the same trait later.

/// Critical-point properties for a pure component.
#[derive(Debug, Clone, Copy)]
pub struct CriticalProperties {
    /// Critical temperature, K.
    pub tc_k: f64,
    /// Critical pressure, Pa.
    pub pc_pa: f64,
    /// Acentric factor ω (dimensionless).
    pub omega: f64,
}

/// Peng-Robinson EOS: P = RT/(V-b) - a(T)/[V(V+b) + b(V-b)]
#[derive(Debug, Clone, Copy)]
pub struct PengRobinson {
    pub props: CriticalProperties,
}

/// Gas constant, J/(mol·K).
const R: f64 = 8.314_462_618;

impl PengRobinson {
    pub fn new(props: CriticalProperties) -> Self {
        Self { props }
    }

    /// EOS parameter `a(T)` in Pa·m⁶/mol².
    pub fn a(&self, t_k: f64) -> f64 {
        let tc = self.props.tc_k;
        let pc = self.props.pc_pa;
        let omega = self.props.omega;
        let kappa = 0.37464 + 1.54226 * omega - 0.26992 * omega * omega;
        let alpha = (1.0 + kappa * (1.0 - (t_k / tc).sqrt())).powi(2);
        0.45724 * R * R * tc * tc / pc * alpha
    }

    /// EOS parameter `b` in m³/mol.
    pub fn b(&self) -> f64 {
        0.07780 * R * self.props.tc_k / self.props.pc_pa
    }

    /// Compressibility factor Z from the cubic: Z³ - (1-B)Z² + (A-3B²-2B)Z - (AB-B²-B³) = 0
    /// Returns the largest real root (vapour-like).
    pub fn z_vapour(&self, t_k: f64, p_pa: f64) -> f64 {
        let a = self.a(t_k);
        let b = self.b();
        let cap_a = a * p_pa / (R * R * t_k * t_k);
        let cap_b = b * p_pa / (R * t_k);

        // Solve cubic by Newton's method from Z=1 (ideal gas start)
        let mut z = 1.0;
        for _ in 0..100 {
            let f = z * z * z - (1.0 - cap_b) * z * z
                + (cap_a - 3.0 * cap_b * cap_b - 2.0 * cap_b) * z
                - (cap_a * cap_b - cap_b * cap_b - cap_b * cap_b * cap_b);
            let fp = 3.0 * z * z - 2.0 * (1.0 - cap_b) * z
                + (cap_a - 3.0 * cap_b * cap_b - 2.0 * cap_b);
            if fp.abs() < 1e-30 {
                break;
            }
            z -= f / fp;
            if f.abs() < 1e-12 {
                break;
            }
        }
        z.max(cap_b) // Z cannot be less than B
    }

    /// Molar volume in m³/mol.
    pub fn molar_volume(&self, t_k: f64, p_pa: f64) -> f64 {
        self.z_vapour(t_k, p_pa) * R * t_k / p_pa
    }

    /// Fugacity coefficient φ = f/P.
    pub fn fugacity_coefficient(&self, t_k: f64, p_pa: f64) -> f64 {
        let z = self.z_vapour(t_k, p_pa);
        let a = self.a(t_k);
        let b = self.b();
        let cap_a = a * p_pa / (R * R * t_k * t_k);
        let cap_b = b * p_pa / (R * t_k);

        let term1 = z - 1.0 - (z - cap_b).max(1e-30).ln();
        let term2 = cap_a / (2.0 * 2.0f64.sqrt() * cap_b)
            * ((z + (1.0 + 2.0f64.sqrt()) * cap_b) / (z + (1.0 - 2.0f64.sqrt()) * cap_b))
                .max(1e-30)
                .ln();
        (term1 - term2).exp()
    }
}

/// Trait for equation-of-state models.
pub trait EquationOfState {
    fn name(&self) -> &'static str;
    fn z_vapour(&self, t_k: f64, p_pa: f64) -> f64;
    fn molar_volume(&self, t_k: f64, p_pa: f64) -> f64;
    fn fugacity_coefficient(&self, t_k: f64, p_pa: f64) -> f64;
}

impl EquationOfState for PengRobinson {
    fn name(&self) -> &'static str {
        "Peng-Robinson"
    }
    fn z_vapour(&self, t_k: f64, p_pa: f64) -> f64 {
        self.z_vapour(t_k, p_pa)
    }
    fn molar_volume(&self, t_k: f64, p_pa: f64) -> f64 {
        self.molar_volume(t_k, p_pa)
    }
    fn fugacity_coefficient(&self, t_k: f64, p_pa: f64) -> f64 {
        self.fugacity_coefficient(t_k, p_pa)
    }
}

// Common pure-component data
pub const WATER_CRITICAL: CriticalProperties = CriticalProperties {
    tc_k: 647.096,
    pc_pa: 22_064_000.0,
    omega: 0.3443,
};

pub const ETHANOL_CRITICAL: CriticalProperties = CriticalProperties {
    tc_k: 513.92,
    pc_pa: 6_148_000.0,
    omega: 0.6436,
};

pub const NITROGEN_CRITICAL: CriticalProperties = CriticalProperties {
    tc_k: 126.19,
    pc_pa: 3_396_000.0,
    omega: 0.0372,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_gas_limit() {
        // At low pressure, Z → 1
        let pr = PengRobinson::new(NITROGEN_CRITICAL);
        let z = pr.z_vapour(300.0, 100.0); // 300K, 100 Pa
        assert!(
            (z - 1.0).abs() < 0.01,
            "Z should be ~1 at low P, got {}",
            z
        );
    }

    #[test]
    fn compressibility_decreases_with_pressure() {
        let pr = PengRobinson::new(NITROGEN_CRITICAL);
        let z_low = pr.z_vapour(300.0, 101_325.0);
        let z_high = pr.z_vapour(300.0, 10_132_500.0);
        assert!(z_high < z_low, "Z should decrease at higher P");
    }

    #[test]
    fn fugacity_approaches_unity_at_low_pressure() {
        let pr = PengRobinson::new(NITROGEN_CRITICAL);
        let phi = pr.fugacity_coefficient(300.0, 1000.0);
        assert!(
            (phi - 1.0).abs() < 0.01,
            "φ should be ~1 at low P, got {}",
            phi
        );
    }

    #[test]
    fn molar_volume_positive() {
        let pr = PengRobinson::new(WATER_CRITICAL);
        let v = pr.molar_volume(500.0, 101_325.0);
        assert!(v > 0.0);
    }
}
