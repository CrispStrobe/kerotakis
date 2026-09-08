//! Heat capacity as a function of temperature, and the enthalpy integral
//! that a bench actually needs.
//!
//! Every heat capacity on this bench used to be one number: the value at
//! 25 °C. That is a good approximation for a beaker of water and a bad one
//! for anything in a crucible. Heating 0.1 mol of chalk to 1500 °C, the open
//! energy balance closed to 93.6% and named the reason: calcite is billed at
//! 82.3 J/(mol·K) and lime at 42.0, while the polynomials the equilibrium
//! solver reads reach about 130 and 53 by 1500 K. The burner is charged at
//! room-temperature prices for a crucible at kiln temperature.
//!
//! So a species may now carry a curve as well as a constant. Two published
//! forms are understood, and both are stored as their own source prints
//! them rather than being converted into one house form:
//!
//! ```text
//! NASA-9    Cp/R = a1/T² + a2/T + a3 + a4·T + a5·T² + a6·T³ + a7·T⁴
//! Shomate   Cp   = A + B·t + C·t² + D·t³ + E/t²        t = T/1000
//! ```
//!
//! Shomate's basis is a strict subset of NASA-9's, so a Shomate row could be
//! rewritten exactly in the other form; a NASA-9 row generally could not,
//! because `a2/T` and `a7·T⁴` have no Shomate counterpart. Dropping them
//! would be a refit, and a refit is a new number that needs its own
//! provenance. Keeping both forms costs one `match` and keeps every
//! coefficient a transcription.
//!
//! # What this module refuses to do
//!
//! **It does not extrapolate.** Outside the tabulated range the heat
//! capacity is held at the endpoint value. This is not timidity: quartz's
//! α-phase polynomial is fitted to 848 K and returns 1007 J/(mol·K) at 1500,
//! sodium carbonate's returns −2038, and lead's −1259. A negative heat
//! capacity does not degrade an energy balance, it destroys it — the vessel
//! cools when heated. Holding the endpoint is wrong by a few percent;
//! extrapolating is wrong by a sign.
//!
//! **It does not carry absolute enthalpy.** NASA-9's `b1` and `b2` fix
//! absolute enthalpy and entropy against a formation reference, and they are
//! deliberately not transcribed. What a bench ledger needs is
//! `enthalpy_between(t0, t1)` — a difference — and a difference does not
//! need a reference state. Formation enthalpies stay where they already
//! live, in `kerotakis-cea`.

use serde::{Deserialize, Serialize};

use crate::species::Phase;

/// The molar gas constant, J/(mol·K) (CODATA, exact since the 2019 SI).
pub const R: f64 = 8.314_462_618_153_24;

/// Which published polynomial a set of coefficients belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CpForm {
    /// `Cp/R = a1/T² + a2/T + a3 + a4·T + a5·T² + a6·T³ + a7·T⁴`, T in
    /// kelvin. NASA TP-2002-211556.
    Nasa9,
    /// `Cp = A + B·t + C·t² + D·t³ + E/t²`, `t = T/1000`, J/(mol·K).
    /// NIST WebBook. The unused trailing slots of `coefficients` are zero.
    Shomate,
}

/// One temperature interval of a curve.
///
/// `coefficients` is a fixed array rather than a slice so the whole table
/// stays a `static` with no relocations and ships unchanged to wasm; a
/// Shomate row uses the first five slots and leaves the rest zero, which is
/// harmless because the two forms are evaluated by different arms.
///
/// `Serialize` only, like every other borrowed-`'static` corner of the
/// registry: the table is generated, never read back into itself.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CpInterval {
    pub t_min: f64,
    pub t_max: f64,
    pub form: CpForm,
    pub coefficients: [f64; 7],
    /// The literature line the source file prints against this interval.
    pub reference: &'static str,
}

impl CpInterval {
    /// Molar heat capacity at `t`, J/(mol·K). Evaluates the polynomial as
    /// written: the caller is responsible for staying in range, and
    /// [`CpPolynomial::cp`] is the entry point that guarantees it.
    pub fn cp(&self, t: f64) -> f64 {
        let c = &self.coefficients;
        match self.form {
            CpForm::Nasa9 => {
                R * (c[0] / (t * t)
                    + c[1] / t
                    + c[2]
                    + c[3] * t
                    + c[4] * t * t
                    + c[5] * t * t * t
                    + c[6] * t * t * t * t)
            }
            CpForm::Shomate => {
                let s = t / 1000.0;
                c[0] + c[1] * s + c[2] * s * s + c[3] * s * s * s + c[4] / (s * s)
            }
        }
    }

    /// ∫Cp dT between two temperatures both inside this interval, J/mol.
    ///
    /// Every term is written as a DIFFERENCE OF POWERS carrying `(t1 - t0)`
    /// as an explicit factor, not as an antiderivative evaluated twice and
    /// subtracted. Algebraically the two are the same line; in a double they
    /// are not.
    ///
    /// Liquid water is the curve that shows why. Its NASA-9 fit has huge
    /// alternating coefficients — `a1` is 1.3e9 — so the antiderivative sums
    /// terms of order 1.2e9 J/mol that cancel to about -9.2e8. A double
    /// holds about 2.4e-7 at that magnitude, so the difference of two such
    /// numbers is quantised in steps of a quarter-microjoule NO MATTER HOW
    /// NARROW THE SPAN. A mole of water arriving a millikelvin off standard
    /// gave up the same 2.6e-7 J of noise as a mole boiled. That floor,
    /// not the ledger, is what `conservation.rs` used to carry a
    /// per-mole-of-water term for; ice's fit gives up 4e-11 and nitrogen's
    /// 4e-12, which is the tell that it was one curve's conditioning.
    ///
    /// Factored this way the largest intermediate for a 100 K span of
    /// liquid water is ~2.7e7 rather than 1.2e9, and — the part that
    /// actually matters — every intermediate is proportional to `(t1 - t0)`,
    /// so the error vanishes with the span instead of sitting at a floor.
    /// Measured against a 60-digit reference over the shipped curves, the
    /// worst residue for liquid water falls from 7.9e-7 to 8.0e-8 J/mol on
    /// arbitrary spans and from 7.3e-7 to 5.7e-10 J/mol on spans under a
    /// kelvin.
    ///
    /// Note this deliberately does NOT re-centre on the interval midpoint.
    /// Re-centring would need the coefficients rewritten about `t_mid`,
    /// which is a transformation of published numbers; this is the same
    /// published numbers, regrouped.
    fn enthalpy(&self, t0: f64, t1: f64) -> f64 {
        let c = &self.coefficients;
        let d = t1 - t0;
        match self.form {
            CpForm::Nasa9 => {
                // a1: -1/t1 + 1/t0 = (t1 - t0)/(t0·t1).
                // a2: ln(t1) - ln(t0) = ln1p((t1 - t0)/t0). Forming the
                //     ratio t1/t0 first would round a narrow span's
                //     information away before the logarithm ever saw it.
                // a3..a7: tⁿ⁺¹ differences factored by (t1 - t0).
                R * (c[0] * d / (t0 * t1)
                    + c[1] * (d / t0).ln_1p()
                    + c[2] * d
                    + c[3] / 2.0 * (t1 + t0) * d
                    + c[4] / 3.0 * (t1 * t1 + t1 * t0 + t0 * t0) * d
                    + c[5] / 4.0 * (t1 + t0) * (t1 * t1 + t0 * t0) * d
                    + c[6] / 5.0
                        * (t1 * t1 * t1 * t1
                            + t1 * t1 * t1 * t0
                            + t1 * t1 * t0 * t0
                            + t1 * t0 * t0 * t0
                            + t0 * t0 * t0 * t0)
                        * d)
            }
            CpForm::Shomate => {
                // The same regrouping in `t = T/1000`. `ds` is differenced
                // in kelvin and scaled, not differenced after scaling, so
                // the span keeps every bit it arrived with.
                let s0 = t0 / 1000.0;
                let s1 = t1 / 1000.0;
                let ds = d / 1000.0;
                1000.0
                    * (c[0] * ds
                        + c[1] / 2.0 * (s1 + s0) * ds
                        + c[2] / 3.0 * (s1 * s1 + s1 * s0 + s0 * s0) * ds
                        + c[3] / 4.0 * (s1 + s0) * (s1 * s1 + s0 * s0) * ds
                        + c[4] * ds / (s0 * s1))
            }
        }
    }
}

/// The heat capacity of one phase of one species as a function of
/// temperature, with the provenance that stands behind it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CpPolynomial {
    /// Which phase this curve describes. Ice, liquid water and steam are
    /// three curves, not one.
    pub phase: Phase,
    /// Contiguous intervals in ascending temperature order.
    pub intervals: &'static [CpInterval],
    /// The citation the source record carries.
    pub source: &'static str,
    /// How the coefficients got here.
    pub method: &'static str,
    /// What the curve does NOT claim.
    pub boundary: &'static str,
}

impl CpPolynomial {
    /// The tabulated range, K. `None` only for a malformed empty record,
    /// which the data validator rejects.
    pub fn range(&self) -> Option<(f64, f64)> {
        Some((self.intervals.first()?.t_min, self.intervals.last()?.t_max))
    }

    /// The interval covering `t`, or the nearest end of the table.
    fn interval_for(&self, t: f64) -> Option<&CpInterval> {
        if let Some(found) = self
            .intervals
            .iter()
            .find(|i| (i.t_min..=i.t_max).contains(&t))
        {
            return Some(found);
        }
        let first = self.intervals.first()?;
        if t < first.t_min {
            return Some(first);
        }
        self.intervals.last()
    }

    /// Molar heat capacity at `t`, J/(mol·K), held at the endpoint value
    /// outside the tabulated range.
    ///
    /// The clamp is on the TEMPERATURE, not merely on the choice of
    /// interval: evaluating a fit outside its own range is what turns
    /// quartz's 44.6 into 1007 and lead's 26.7 into −1259.
    pub fn cp(&self, t: f64) -> Option<f64> {
        let (lo, hi) = self.range()?;
        let held = t.clamp(lo, hi);
        Some(self.interval_for(held)?.cp(held))
    }

    /// ∫Cp dT from `t0` to `t1`, J/mol. Signed: cooling returns a negative
    /// number, and `enthalpy_between(a, b) == -enthalpy_between(b, a)` to
    /// within rounding.
    ///
    /// Outside the tabulated range the endpoint heat capacity is held
    /// constant, so the integral continues linearly rather than stopping —
    /// a crucible driven past the last tabulated temperature must still cost
    /// something to heat.
    pub fn enthalpy_between(&self, t0: f64, t1: f64) -> Option<f64> {
        if t1 < t0 {
            return self.enthalpy_between(t1, t0).map(|j| -j);
        }
        let (lo, hi) = self.range()?;
        let mut total = 0.0;
        // Below the table: the first interval's own lower endpoint, held.
        if t0 < lo {
            let cp = self.intervals.first()?.cp(lo);
            total += cp * (t1.min(lo) - t0);
        }
        // Above the table: the last interval's upper endpoint, held.
        if t1 > hi {
            let cp = self.intervals.last()?.cp(hi);
            total += cp * (t1 - t0.max(hi));
        }
        // Inside the table: the polynomials, interval by interval.
        let from = t0.max(lo);
        let to = t1.min(hi);
        if to > from {
            for interval in self.intervals {
                let a = from.max(interval.t_min);
                let b = to.min(interval.t_max);
                if b > a {
                    total += interval.enthalpy(a, b);
                }
            }
        }
        Some(total)
    }

    /// The temperature this phase reaches when `joules` per mole are added
    /// at `from`, J/mol in and kelvin out.
    ///
    /// With a constant heat capacity this is `from + j/cp`. With a curve it
    /// is the inverse of `enthalpy_between`, which has no closed form, so it
    /// is bisected: monotone because Cp is positive everywhere a real
    /// substance has one, bracketed by walking outward from the constant-Cp
    /// guess, and stopped when the residual is under a millijoule per mole
    /// or the bracket has collapsed to a microkelvin. Both are far below
    /// anything a thermometer on this bench resolves.
    pub fn temperature_after(&self, from: f64, joules: f64) -> Option<f64> {
        if joules == 0.0 {
            return Some(from);
        }
        let guess_cp = self.cp(from)?.max(1e-9);
        let mut span = (joules / guess_cp).abs().max(1.0);
        let (mut lo, mut hi) = (from, from);
        for _ in 0..60 {
            if joules > 0.0 {
                hi = from + span;
                if self.enthalpy_between(from, hi)? >= joules {
                    break;
                }
                lo = hi;
            } else {
                lo = (from - span).max(0.0);
                if self.enthalpy_between(from, lo)? <= joules {
                    break;
                }
                hi = lo;
                if lo <= 0.0 {
                    break;
                }
            }
            span *= 2.0;
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let delta = self.enthalpy_between(from, mid)? - joules;
            if delta.abs() < 1e-3 || hi - lo < 1e-6 {
                return Some(mid);
            }
            if delta > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(0.5 * (lo + hi))
    }
}

/// The curve for one phase of a species, if the registry carries one.
pub fn polynomial_for(
    polys: &'static [CpPolynomial],
    phase: Phase,
) -> Option<&'static CpPolynomial> {
    polys.iter().find(|p| p.phase == phase)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALCITE: &[CpInterval] = &[
        CpInterval {
            t_min: 300.0,
            t_max: 500.0,
            form: CpForm::Nasa9,
            coefficients: [
                -13298624.25,
                250051.7168,
                -1907.177167,
                7.61666627,
                -0.016558708_6,
                1.879382277e-5,
                -8.72071327e-9,
            ],
            reference: "CaCO3(cr): Hexagonal Gurvich,1996a pt1 p483 pt2 p376.",
        },
        CpInterval {
            t_min: 500.0,
            t_max: 1603.0,
            form: CpForm::Nasa9,
            coefficients: [
                -258355.5736,
                0.0,
                11.97256363,
                0.003263812299,
                0.0,
                0.0,
                0.0,
            ],
            reference: "CaCO3(cr): Hexagonal Gurvich,1996a pt1 p483 pt2 p376.",
        },
    ];

    fn calcite() -> CpPolynomial {
        CpPolynomial {
            phase: Phase::Solid,
            intervals: CALCITE,
            source: "test",
            method: "test",
            boundary: "test",
        }
    }

    #[test]
    fn a_curve_rises_where_the_constant_stood_still() {
        let c = calcite();
        let cold = c.cp(298.15).unwrap();
        let hot = c.cp(1500.0).unwrap();
        assert!(
            (cold - 83.5).abs() < 0.5,
            "calcite at 298 K should be near its registry constant, got {cold}"
        );
        assert!(
            hot > 1.5 * cold,
            "calcite should be half again as expensive to heat at 1500 K, \
             got {hot} against {cold}"
        );
    }

    #[test]
    fn nothing_is_extrapolated_past_the_table() {
        let c = calcite();
        let edge = c.cp(1603.0).unwrap();
        let beyond = c.cp(5000.0).unwrap();
        assert!(
            (edge - beyond).abs() < 1e-9,
            "past the last tabulated temperature the endpoint is held, \
             but {beyond} came back against an endpoint of {edge}"
        );
        assert!(beyond > 0.0, "a held heat capacity is still positive");
    }

    #[test]
    fn the_integral_is_the_area_under_the_curve() {
        let c = calcite();
        // Trapezoid the curve finely and compare: the analytic integral has
        // no business disagreeing with the function it integrates.
        let (t0, t1) = (298.15, 1500.0);
        let n = 20_000;
        let h = (t1 - t0) / n as f64;
        let mut numeric = 0.0;
        for k in 0..n {
            let a = t0 + k as f64 * h;
            numeric += 0.5 * h * (c.cp(a).unwrap() + c.cp(a + h).unwrap());
        }
        let analytic = c.enthalpy_between(t0, t1).unwrap();
        assert!(
            (analytic - numeric).abs() < 1.0,
            "analytic {analytic} J/mol against numeric {numeric} J/mol"
        );
    }

    #[test]
    fn the_integral_reverses_cleanly() {
        let c = calcite();
        let up = c.enthalpy_between(400.0, 1200.0).unwrap();
        let down = c.enthalpy_between(1200.0, 400.0).unwrap();
        assert!(up > 0.0 && (up + down).abs() < 1e-6, "{up} against {down}");
    }

    #[test]
    fn heating_and_asking_where_we_landed_round_trips() {
        let c = calcite();
        let joules = c.enthalpy_between(298.15, 1100.0).unwrap();
        let landed = c.temperature_after(298.15, joules).unwrap();
        assert!(
            (landed - 1100.0).abs() < 1e-3,
            "adding exactly the enthalpy of 298 K to 1100 K should land at \
             1100 K, landed at {landed}"
        );
        let back = c.temperature_after(1100.0, -joules).unwrap();
        assert!(
            (back - 298.15).abs() < 1e-3,
            "and taking it back out should return, got {back}"
        );
    }

    /// Liquid water as the registry ships it: the worst-conditioned curve
    /// on the bench, and the reason `enthalpy` is written in difference
    /// form. Transcribed from `heat-capacity-polynomial/water/liquid`.
    const WATER_LIQUID: CpInterval = CpInterval {
        t_min: 273.15,
        t_max: 373.15,
        form: CpForm::Nasa9,
        coefficients: [
            1326371304.0,
            -24482953.88,
            187942.8776,
            -767.899505,
            1.761556813,
            -0.002151167128,
            1.092570813e-06,
        ],
        reference: "H2O(L): Liquid. Cox,1989. Haar,1984. Keenan,1984. Stimson,1969.",
    };

    #[test]
    fn a_narrow_span_of_water_is_resolved_and_not_quantised() {
        // Over a span this short the curve is a straight line to far
        // better than the tolerances below, so the midpoint rectangle IS
        // the integral and any disagreement is arithmetic, not calculus.
        //
        // Evaluating the antiderivative twice and subtracting cannot pass
        // this. Liquid water's antiderivative is ~-9.2e8 J/mol here, where
        // a double's step is 2.4e-7, so the answer came back quantised: at
        // h = 1e-6 K it was wrong by 0.17 %, and at h = 1e-9 K it was
        // wrong by 100 % because it came back as exactly zero. That
        // quarter-microjoule per mole is the whole of the per-mole-of-water
        // floor `conservation.rs` used to carry.
        let t = 298.15;
        for (h, tolerance) in [(1e-3, 1e-7), (1e-6, 1e-7), (1e-9, 1e-3)] {
            let got = WATER_LIQUID.enthalpy(t, t + h);
            let want = WATER_LIQUID.cp(t + 0.5 * h) * h;
            assert!(
                got != 0.0,
                "a span of {h} K of liquid water is real heat, not zero"
            );
            assert!(
                (got - want).abs() <= tolerance * want.abs(),
                "over {h} K at {t} K: got {got} J/mol against {want} J/mol, \
                 a relative {} outside the {tolerance} this form should hold",
                (got - want).abs() / want.abs()
            );
        }
    }

    #[test]
    fn the_whole_liquid_range_resolves_narrow_spans() {
        // The same claim as above, swept, because 298.15 K could be a
        // lucky point. Nothing about the argument is special to room
        // temperature: the antiderivative is ~1e9 J/mol everywhere on this
        // interval, so everywhere on it the old form quantised the answer.
        for step in 0..=20 {
            let t = 273.15 + 100.0 * step as f64 / 20.0;
            for h in [1e-3, 1e-5] {
                let (a, b) = if t + h > 373.15 {
                    (t - h, t)
                } else {
                    (t, t + h)
                };
                let got = WATER_LIQUID.enthalpy(a, b);
                let want = WATER_LIQUID.cp(0.5 * (a + b)) * h;
                assert!(
                    (got - want).abs() <= 1e-6 * want.abs(),
                    "{h} K at {t} K: {got} J/mol against {want} J/mol"
                );
            }
        }
    }

    #[test]
    fn a_shomate_row_is_the_same_curve_written_differently() {
        // Shomate's basis is a subset of NASA-9's, so the identical curve
        // can be written in both. If the two arms of the evaluator ever
        // disagree about that, one of them is wrong.
        let shomate = CpInterval {
            t_min: 300.0,
            t_max: 1200.0,
            form: CpForm::Shomate,
            coefficients: [40.0, 20.0, -5.0, 1.0, -0.5, 0.0, 0.0],
            reference: "synthetic",
        };
        let a = shomate.coefficients;
        let nasa = CpInterval {
            t_min: 300.0,
            t_max: 1200.0,
            form: CpForm::Nasa9,
            coefficients: [
                a[4] * 1e6 / R,
                0.0,
                a[0] / R,
                a[1] / (1e3 * R),
                a[2] / (1e6 * R),
                a[3] / (1e9 * R),
                0.0,
            ],
            reference: "synthetic",
        };
        for t in [300.0, 500.0, 900.0, 1200.0] {
            let (x, y) = (shomate.cp(t), nasa.cp(t));
            assert!((x - y).abs() < 1e-9, "at {t} K: {x} against {y}");
        }
        let (x, y) = (
            shomate.enthalpy(300.0, 1200.0),
            nasa.enthalpy(300.0, 1200.0),
        );
        assert!((x - y).abs() < 1e-6, "integrals: {x} against {y}");
    }
}
