//! A split preserves retained and transferred ownership independently.
//!
//! This planning API does not migrate vessel inventory. `Portion::moles`
//! and the vessel writers still own scalar amounts; a lossy projection is
//! refused, never certified as a completed transfer. No still route uses
//! this type until both the solver result and vessel writer can retain it.
use crate::amount::{Amount, AmountError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentSplit {
    initial: Amount,
    retained: Amount,
    transferred: Amount,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    Amount(AmountError),
    Closure,
    StaleDonor,
    ScalarBoundary,
}
impl From<AmountError> for SplitError {
    fn from(error: AmountError) -> Self {
        Self::Amount(error)
    }
}
/// Exact scalar values a legacy writer could use. This is a numeric plan,
/// not authority over species, phase, lots, caches, enthalpy or safety.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarWritePlan {
    pub donor_after: f64,
    pub receiver_after: f64,
    pub transferred: f64,
}
impl ComponentSplit {
    /// Retain the residue as a first-class owner. Subtraction is performed
    /// in the compensated Amount domain, not projected scalar arithmetic.
    /// Inputs are scalars, so this single difference has at most two limbs.
    pub fn from_residue(initial: f64, residue: f64) -> Result<Self, SplitError> {
        let initial = Amount::new(initial)?;
        let retained = Amount::new(residue)?;
        let transferred = initial.checked_sub(retained)?;
        if retained.checked_add(transferred)? != initial {
            return Err(SplitError::Closure);
        }
        Ok(Self {
            initial,
            retained,
            transferred,
        })
    }
    pub fn retained(self) -> Amount {
        self.retained
    }
    pub fn transferred(self) -> Amount {
        self.transferred
    }
    pub fn initial(self) -> Amount {
        self.initial
    }

    /// Refuse every correction a scalar vessel write would discard. The
    /// donor is bound to the same initial inventory used by the split.
    pub fn scalar_write_plan(
        self,
        donor: f64,
        receiver: f64,
    ) -> Result<ScalarWritePlan, SplitError> {
        if Amount::new(donor)? != self.initial {
            return Err(SplitError::StaleDonor);
        }
        let receiver_after = Amount::new(receiver)?.checked_add(self.transferred)?;
        let exact_scalar = |amount: Amount| -> Result<f64, SplitError> {
            let (high, low) = amount.parts();
            if low != 0.0 {
                return Err(SplitError::ScalarBoundary);
            }
            Ok(high)
        };
        let retained = exact_scalar(self.retained)?;
        let moved = exact_scalar(self.transferred)?;
        let received = exact_scalar(receiver_after)?;
        // This checks the legacy writer, not the split's authority. Never
        // derive a positive residual from a rounded overhead projection.
        if donor - moved != retained || receiver + moved != received {
            return Err(SplitError::ScalarBoundary);
        }
        Ok(ScalarWritePlan {
            donor_after: retained,
            receiver_after: received,
            transferred: moved,
        })
    }
}
