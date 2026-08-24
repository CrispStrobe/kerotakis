//! Phase behaviour: what boils, at what temperature, and what comes over.
//!
//! Distillation is the one place where school chemistry's tidy story breaks
//! in a way a learner can *see*. Heat a mixture, collect the vapour, and
//! you are told the more volatile component comes over first — which is
//! true right up until ethanol and water reach 95.6% by mass and stop
//! separating altogether, no matter how tall the column. That azeotrope is
//! why spirits are 96% and not 100%, why "absolute alcohol" needs a
//! different trick entirely, and it is the teaching moment most simulators
//! skip because an ideal model cannot produce it.
//!
//! An ideal model *cannot*, and that is the point of building this in two
//! layers rather than one:
//!
//! ```text
//! Raoult:   y·P = x · P°(T)                    ideal, no azeotrope, ever
//! Modified: y·P = x · γ(x, T) · P°(T)          γ ≠ 1 is where it comes from
//! ```
//!
//! The first layer is honest and wrong, and the bench can show it being
//! wrong. Everything an ideal mixture does — Raoult's law, a smooth
//! boiling-point curve, the more volatile one always enriching — falls out
//! of `vle` with γ pinned at 1. Everything the real one does needs
//! `unifac`, and the difference between the two curves *is* the lesson.
//!
//! What is curated and what is computed: vapour pressures come from Antoine
//! constants with their own sources and stated validity ranges, activity
//! coefficients from group contributions with published parameters, and the
//! flash itself is arithmetic on top of both. Nothing here is a lookup of
//! the answer.

pub mod eos;
pub mod excess;
pub mod fluid;
pub mod lle;
pub mod phase_diagram;
pub mod unifac;
pub mod vle;
