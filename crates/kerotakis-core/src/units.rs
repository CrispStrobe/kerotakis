//! Lightweight unit newtypes for the public API.
//!
//! These are serde-friendly f64 wrappers, not a dimensional-analysis system;
//! migration to `uom` at the API boundary is planned once the API surface
//! settles (PLAN.md, crate table). Internal canonical units: K, Pa, L, mol,
//! g, J.

use serde::{Deserialize, Serialize};

macro_rules! unit {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub f64);

        impl $name {
            pub fn value(self) -> f64 {
                self.0
            }
        }

        impl std::ops::Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl std::ops::Mul<f64> for $name {
            type Output = Self;
            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }
    };
}

unit!(
    /// Thermodynamic temperature in kelvin.
    Kelvin
);
unit!(
    /// Pressure in pascal.
    Pascal
);
unit!(
    /// Volume in litres.
    Liters
);
unit!(
    /// Amount of substance in moles.
    Moles
);
unit!(
    /// Mass in grams.
    Grams
);
unit!(
    /// Energy in joules.
    Joules
);

// ── ARCH-002: typed quantity gaps ──────────────────────────────────

unit!(
    /// Power in watts (J/s).
    Watts
);
unit!(
    /// Electric current in amperes.
    Amperes
);
unit!(
    /// Electric potential in volts.
    Volts
);
unit!(
    /// Area in square metres.
    SquareMeters
);
unit!(
    /// Amount-of-substance concentration in mol/L.
    Molarity
);
unit!(
    /// Volumetric flow rate in L/s.
    VolumeFlow
);
unit!(
    /// Molar flow rate in mol/s.
    MolarFlow
);
unit!(
    /// Photon flux in mol·photons/s (einstein/s).
    PhotonFlux
);
unit!(
    /// Time duration in seconds.
    Seconds
);

impl Kelvin {
    pub const STANDARD: Kelvin = Kelvin(298.15);

    pub fn from_celsius(c: f64) -> Self {
        Kelvin(c + 273.15)
    }

    pub fn to_celsius(self) -> f64 {
        self.0 - 273.15
    }
}

impl Pascal {
    pub const ATMOSPHERIC: Pascal = Pascal(101_325.0);
}

impl Molarity {
    /// Compute concentration from moles and volume.
    pub fn from_moles_and_litres(moles: Moles, litres: Liters) -> Self {
        if litres.0 <= 0.0 {
            Molarity(0.0)
        } else {
            Molarity(moles.0 / litres.0)
        }
    }
}

impl Watts {
    /// Compute power from energy and time.
    pub fn from_joules_per_second(joules: Joules, seconds: Seconds) -> Self {
        if seconds.0 <= 0.0 {
            Watts(0.0)
        } else {
            Watts(joules.0 / seconds.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! round_trip {
        ($name:ident, $ty:ident, $val:expr) => {
            #[test]
            fn $name() {
                let original = $ty($val);
                let json = serde_json::to_string(&original).unwrap();
                let recovered: $ty = serde_json::from_str(&json).unwrap();
                assert_eq!(original, recovered);
            }
        };
    }

    round_trip!(kelvin_round_trip, Kelvin, 373.15);
    round_trip!(pascal_round_trip, Pascal, 101325.0);
    round_trip!(liters_round_trip, Liters, 0.5);
    round_trip!(moles_round_trip, Moles, 0.001);
    round_trip!(grams_round_trip, Grams, 58.44);
    round_trip!(joules_round_trip, Joules, 4184.0);
    round_trip!(watts_round_trip, Watts, 100.0);
    round_trip!(amperes_round_trip, Amperes, 0.5);
    round_trip!(volts_round_trip, Volts, 1.23);
    round_trip!(square_meters_round_trip, SquareMeters, 0.01);
    round_trip!(molarity_round_trip, Molarity, 0.1);
    round_trip!(volume_flow_round_trip, VolumeFlow, 0.001);
    round_trip!(molar_flow_round_trip, MolarFlow, 1e-6);
    round_trip!(photon_flux_round_trip, PhotonFlux, 1e-3);
    round_trip!(seconds_round_trip, Seconds, 3600.0);

    #[test]
    fn kelvin_celsius_conversion() {
        let k = Kelvin::from_celsius(100.0);
        assert!((k.0 - 373.15).abs() < 1e-10);
        assert!((k.to_celsius() - 100.0).abs() < 1e-10);
    }

    #[test]
    fn molarity_from_moles_and_litres() {
        let m = Molarity::from_moles_and_litres(Moles(0.1), Liters(1.0));
        assert!((m.0 - 0.1).abs() < 1e-15);
    }

    #[test]
    fn watts_from_energy_and_time() {
        let w = Watts::from_joules_per_second(Joules(1000.0), Seconds(10.0));
        assert!((w.0 - 100.0).abs() < 1e-15);
    }

    #[test]
    fn unit_arithmetic() {
        let a = Watts(100.0);
        let b = Watts(50.0);
        assert_eq!((a + b).0, 150.0);
        assert_eq!((a - b).0, 50.0);
        assert_eq!((a * 2.0).0, 200.0);
    }
}
