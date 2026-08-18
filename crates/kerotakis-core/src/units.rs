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
