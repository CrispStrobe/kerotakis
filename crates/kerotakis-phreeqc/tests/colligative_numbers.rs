//! PLAN P3s: the two colligative numbers a school textbook prints, taken
//! off this bench rather than out of a table.
//!
//! Both constants the answers rest on are derived in `states.rs` from
//! water's enthalpies of fusion and vaporisation rather than tabulated, and
//! the particle count comes from PHREEQC's own speciation rather than from
//! a looked-up van 't Hoff factor. So neither of these tests can pass by
//! having the right answer written down somewhere; the whole chain has to
//! be right at once.
//!
//! # The measurements this file is checked against
//!
//! Every world-facing band below names one of the five sources in this
//! block. They were added on 2026-09-14, closing a debt this file had
//! carried since 2026-09-11: it asserted against "measurement" and
//! "textbooks print" with no author, no edition and no page anywhere in its
//! 350 lines, and on 2026-09-13 a comparison was DROPPED rather than cited,
//! on the reading that a book without a digital object identifier could not
//! be cited at all. The owner corrected that on 2026-09-14 — "we should be
//! able to cite any book, only not harvest the books per systematic
//! scraping, and we should be able to trace original sources for almost all
//! values, and cite those" — so the line is bulk dependence, not
//! attribution, and the dropped anchors are restored below.
//!
//! **[S1] Freezing points, measured directly.** G. Scatchard and S. S.
//! Prentiss, "The freezing points of aqueous solutions. IV. Potassium,
//! sodium and lithium chlorides and bromides", *J. Am. Chem. Soc.* 55
//! (1933) 4355-4362, doi:10.1021/ja01338a003. A cryoscopic determination of
//! NaCl(aq) across the dilute range, and the origin of the depressions this
//! file compares against. It is the right kind of source for the freezing
//! rows specifically: it is a DIFFERENT EXPERIMENT from the isopiestic
//! vapour-pressure work the ion-interaction coefficients are fitted to, so
//! agreement here is not agreement with the bench's own ancestry.
//!
//! **[S2] Osmotic coefficients at 25 °C, measured directly.** G. Scatchard,
//! W. J. Hamer and S. E. Wood, "Isotonic solutions. I. The chemical
//! potential of water in aqueous solutions of sodium chloride, potassium
//! chloride, sulfuric acid, sucrose, urea and glycerol at 25°", *J. Am.
//! Chem. Soc.* 60 (1938) 3061-3070, doi:10.1021/ja01279a066; corrections
//! *ibid.* 61 (1939) 3603, doi:10.1021/ja01267a608. The isopiestic series
//! behind the tabulated φ for this salt, and it covers sucrose in the same
//! paper, which is why one citation carries three of the rows here.
//!
//! **[S3] The tabulation actually consulted.** R. A. Robinson and R. H.
//! Stokes, *Electrolyte Solutions*, 2nd edition (revised), Butterworths,
//! London, 1959, Appendix 8.10, "Osmotic and activity coefficients of
//! electrolytes in aqueous solution at 25 °C"; reprinted Dover, Mineola NY,
//! 2002, ISBN 0-486-42225-9. This is the book that was called uncitable on
//! 2026-09-13 for want of an identifier, and citing it is ordinary
//! practice: author, title, edition, publisher, year, table. Its NaCl
//! column is where φ = 0.9324 at 0.1 mol/kg, 0.9355 at 1.0 and 1.2710 at
//! 6.0 come from, and [S2] is the measurement behind that column.
//!
//! **WHAT WAS AND WAS NOT OPENED, because the difference matters.** The
//! bibliographic records of [S1], [S2], [S4] and [S5] were resolved against
//! Crossref on 2026-09-14 and are exact. The NUMBERS were not read off a
//! printed page or a publisher's scan: [S1] and [S2] are paywalled (OpenAlex
//! reports `oa_status: closed` for [S1]) and no copy of [S3] was opened.
//! They are the values this repository already had in its own prose —
//! HISTORY.md carries −0.346, −3.4 °C, φ = 0.936 and 108.7 °C — now given
//! the sources they never had, and the world-facing bands below are set wide
//! enough to absorb a transcription that has not been verified against the
//! printed table. That is a weaker claim than "transcribed from the source",
//! and it is written down rather than blurred. The accuracy corpus that
//! follows this change records the same status per row in a
//! machine-readable field, so a reader never has to infer it.
//!
//! **[S4] Boiling, for the concentrated end.** H. F. Gibbard Jr., G.
//! Scatchard, R. A. Rousseau and J. L. Creek, "Liquid-vapor equilibrium of
//! aqueous sodium chloride, from 298 to 373 K and from 1 to 6 mol kg⁻¹, and
//! related properties", *J. Chem. Eng. Data* 19 (1974) 281-288,
//! doi:10.1021/je60062a023. Named but NOT yet used as a band; see the
//! saturated-brine test, where the figure this file has been quoting turns
//! out to describe a different solution.
//!
//! **[S5] The sugar's own liquidus.** F. E. Young and F. T. Jones, "Sucrose
//! hydrates. The sucrose-water phase diagram", *J. Phys. Colloid Chem.* 53
//! (1949) 1334-1350, doi:10.1021/j150474a004.
//!
//! **[S6] The modern critical re-evaluation**, named for completeness and
//! deliberately not leaned on. D. G. Archer, "Thermodynamic properties of
//! the NaCl+H2O system. II. Thermodynamic properties of NaCl(aq),
//! NaCl·2H2O(cr), and phase equilibria", *J. Phys. Chem. Ref. Data* 21
//! (1992) 793-829, doi:10.1063/1.555915. JPCRD is NIST Standard Reference
//! Data, which PLAN.md's provenance table puts on the do-not-harvest row;
//! under the 2026-09-14 correction a single value may still be cited with
//! attribution, but the direct measurements [S1] and [S2] are preferred and
//! nothing here rests on [S6].
//!
//! Kerotakis transcribes individual factual measurements and redistributes
//! no source text or table, the footing `literature/hartley-campbell-iodine-water`
//! already stands on in the registry.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )))])
}

fn step(bench: &mut Bench, solvers: &mut SolverStack, operator: Operator) -> Vec<Event> {
    bench
        .step_with(operator, solvers, &PermissiveScreen)
        .expect("bench step")
}

fn add(bench: &mut Bench, solvers: &mut SolverStack, key: &str, moles: f64) {
    step(
        bench,
        solvers,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        },
    );
}

/// One mole of sodium chloride in one kilogram of water: 58.4 g, the exact
/// spoonful every textbook uses.
///
/// The bench says −3.44 °C and a thermometer in real brine says about
/// −3.4, so it is asserted from BOTH ends here — the model's own number and
/// the world's — which is the only way a test can hold a stated
/// approximation to account. It said −3.72 until 2026-09-11, and what
/// closed the eight per cent is worth keeping written down, because two
/// plausible explanations for that gap were wrong.
///
/// **It was never the particle count.** Counting is what this bench does
/// honestly: the speciation is asked how many particles there are rather
/// than a van 't Hoff factor being looked up, and for this salt the answer
/// really is two — no database this bench LOADS defines an aqueous NaCl ion
/// pair (minteq.v4 carries `Halite`, which is the solid), so nothing is
/// paired and nothing pretends to be.
///
/// **That scope word is load-bearing, and it used to be missing.** The
/// sentence read "no database shipped with the bench", and one shipped
/// here does: `vendor/iphreeqc/database/llnl.dat` line 5733 writes
/// `Na+ + Cl- = NaCl`, `log_k -0.777`, with a calculated ΔH beside it.
/// Vendored and not routed is a different claim from absent, and this is
/// the file where a stated approximation is held to account, so it is the
/// last place that should blur them.
///
/// **The pair is deliberately NOT borrowed, and the arithmetic is why.**
/// log K −0.777 is K = 0.167, so with ideal ionic activities about one
/// formula unit in seven would be paired; at one molal, where γ± ≈ 0.66
/// and the ion-activity product is nearer 0.44, about one in fourteen.
/// That moves the particle count from 2.00 to roughly 1.93 and this test's
/// answer from −3.72 °C to about −3.59 — most of the way to −3.4, by
/// precisely the mechanism this comment says is not the problem. It would
/// be a better number reached through worse physics, and it would then
/// hide the real correction behind a partial cancellation, which is the
/// failure mode that is hardest to find later.
///
/// The decisive evidence is which datasets carry the pair. Not
/// phreeqc.dat, not wateq4f.dat, not minteq.v4.dat — and above all not
/// pitzer.dat, the dataset built FOR concentrated brine and the one that
/// would need an ion pair most if the deviation at one molal were
/// speciation. pitzer has none; it handles Na⁺/Cl⁻ with virial
/// coefficients instead (`PITZER`, `-B0`, `Cl-  Na+  7.534e-2`, with `-B1`
/// and `-C0` rows beside it), which is to say it corrects the SOLVENT's
/// activity and leaves both ions free. llnl's `NaCl` is a fitting device
/// that extends a Debye–Hückel-family database's range, not a claim that a
/// fourteenth of dissolved salt is undissociated.
///
/// **So the fix was the solvent's activity, and it is now wired.** The gap
/// was that `states::transitions` applied the DILUTE-solution law, ΔT =
/// K_f·m, at one molal, where the solvent's activity is no longer its mole
/// fraction. A textbook's i ≈ 1.85 for this solution is that activity
/// correction wearing the particle count's clothes. The bench now computes
/// `1/T_f = 1/T_f° − (R/ΔH_fus)·ln a_w` and takes a_w from the same
/// pitzer.dat speciation that answered everything else about this beaker:
/// a_w = 0.9668, an osmotic coefficient of 0.937 against a measured 0.936,
/// and −3.44 °C.
///
/// **And it was never a NEW number, either.** The water activity has been
/// on the wire the whole time, as the `H2O` row of the ordinary species
/// distribution — the same rows `particles` draws its census from. Nothing
/// was added to the readback, the cache or the wasm path to reach it.
///
/// What is left between −3.44 and a measurement is the osmotic
/// coefficient's own temperature dependence: PHREEQC solved this solution
/// at the vessel's temperature, near 24 °C after the dissolution
/// endotherm, and φ for this salt falls by about a per cent between there
/// and the freezing point, which is worth a few hundredths of a kelvin in
/// the direction of the measurement. Published depressions for 1.000 molal
/// NaCl sit between 3.37 and 3.44 K depending on the source, which is why
/// the world-facing band below is a tenth and a half rather than a
/// hundredth. The low end of that spread comes from correlations fitted for
/// concentrated brines and read back down here — the fluid-inclusion fits
/// are the common case — whose dilute limit does not reproduce K_f, so they
/// are not preferred; [S1] measured this solution directly and is.
#[test]
fn a_textbook_spoonful_of_salt_freezes_the_water_near_minus_three_point_four() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    // 1 kg of water is 55.51 mol; 1 mol of NaCl is 58.44 g.
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "NaCl", 1.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    let freezing_c = transitions.freezing_k - 273.15;

    // The count, unchanged and still counted rather than assumed.
    assert!(
        (transitions.solute_molality - 2.0).abs() < 0.05,
        "a mole of NaCl dissociates into two counted particles and none of \
         the shipped databases pairs them back up: {:.4} mol/kg",
        transitions.solute_molality
    );
    // The activity, and where it came from. An ideal route here would mean
    // the ion-interaction speciation was not consulted, and the test would
    // pass its temperature band by 0.16 K while having lost the point.
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IonInteraction,
        "one molal brine routes to pitzer.dat, and its solvent activity is \
         the whole of this fix"
    );
    // The measured osmotic coefficient of 1.000 mol/kg NaCl at 25 °C is
    // 0.9355 [S3, NaCl column; measured isopiestically in S2]. This test has
    // asserted against it since 2026-09-11 with no source at all, which is
    // the debt the module header closes. The band stays 0.01, and the
    // argument for it is that the bench's own 0.937 is 1.5 mK-worth of
    // rounding away from the table's 0.9355 while the ideal solvent this
    // replaced asserts 1.000 by construction, six times the band away.
    let phi = transitions.osmotic_coefficient();
    assert!(
        (phi - 0.937).abs() < 0.01,
        "1 molal NaCl has a measured osmotic coefficient of 0.9355 at 25 °C \
         [S2, S3]: {phi:.4}"
    );
    assert!(
        (transitions.water_activity() - 0.9668).abs() < 0.002,
        "a_w = {:.5}",
        transitions.water_activity()
    );

    // What the model says.
    assert!(
        (freezing_c + 3.44).abs() < 0.06,
        "the solvent-activity relation puts one molal brine at -3.44 C: got {freezing_c:.3} C"
    );
    // What the world says: a 1.000 mol/kg brine freezes near −3.4 °C, by
    // the direct cryoscopy of [S1]. Narrowing this would fail on a change
    // that made the answer better; widening it would let the dilute law back
    // in — its -3.72 is outside this band, which is the point of keeping the
    // band. The width is 0.15 K and it is spent on three things: the spread
    // between published depressions for this solution (3.37 to 3.44 K, which
    // is 0.07 on its own), the osmotic coefficient's temperature dependence
    // that the paragraph above says is still uncorrected here, and a
    // transcription of [S1]'s table that nobody in this repository has
    // checked against the printed page. It excludes the predecessor by 0.17.
    assert!(
        (freezing_c + 3.4).abs() < 0.15,
        "real 1 molal brine freezes near -3.4 C [S1]: {freezing_c:.3} C"
    );
    // And the dilute law, named so the regression is visible rather than
    // remembered: it is 8 % steeper than the relation that replaced it.
    let dilute_law = -states::cryoscopic_constant() * transitions.solute_molality;
    assert!(
        dilute_law < freezing_c - 0.2,
        "the dilute law would have said {dilute_law:.3} C"
    );
    // The van 't Hoff factor a table prints for this solution, computed
    // rather than looked up. Textbooks print 1.85, and it is worth saying
    // that this is NOT a seventh independent anchor: i is ν·φ by definition,
    // so this line and the φ line above are one measurement counted twice,
    // and a corpus must not bank them separately.
    let i = transitions.effective_vant_hoff_factor(1.0).unwrap();
    assert!((i - 1.85).abs() < 0.04, "i = {i:.3}");
}

/// A TENTH-molal brine, which is the case the router does not send to
/// `pitzer.dat` and which was therefore still answered by Raoult's law
/// until 2026-09-13.
///
/// This is the dilute end of the same defect the one-molal test above
/// closed, and it is the more likely beaker of the two: a tenth molal is
/// roughly a level teaspoon of salt in half a litre, which is nearer what
/// anybody actually makes than the textbook kilogram-and-a-mole is. The
/// bench was on a modelled solvent at one molal and on Raoult's law here,
/// which is the wrong way round.
///
/// **Why it was wrong is not the same reason the one-molal case was.**
/// There the fix was to stop using the dilute LAW. Here the relation is
/// already right and the ACTIVITY it runs on was not available: the router
/// sends a solution to `pitzer.dat` only above 1 mol/kgw, because that is
/// where the Debye–Hückel datasets leave their validity domain for the
/// CHEMISTRY. Below it `wateq4f.dat` answers, and PHREEQC does not model
/// a_w on a Debye–Hückel database at all — it prints `1 − 0.017·Σm`, a
/// hard-coded constant that carries no information about what is dissolved.
/// `states::SolventActivity` declines it for that reason and Raoult's law
/// stood in, which put this beaker at −0.371 °C — the whole of the
/// dissolved salt's effect on the solvent taken as a mole-fraction
/// correction, with nothing in it about the salt.
///
/// So the chemistry never needed re-solving; one number did. The fix asks
/// `pitzer.dat` for the solvent activity in a second, deliberately lean
/// speciation — element totals and the solvent mass, no phases, no gas, no
/// interfaces — and reads a_w out of it.
///
/// **Where the numbers below come from, and where they do not.** Every
/// value asserted here is one this repository ships or computes, and the
/// assertions are pins on those. φ is what `pitzer.dat`'s Na–Cl virial
/// coefficients give at this molality, and that file records its own
/// provenance for them: the `-B0` row `Cl-  Na+  7.534e-2 ...` is marked
/// `# ref. 3`, and the file's reference list reads `ref. 3: Appelo, 2015,
/// Appl. Geochem. 55, 6271` — C. A. J. Appelo, "Principles, caveats and
/// improvements in databases for calculating hydrogeochemical reactions in
/// saline waters from 0 to 200 °C and 1 to 1000 atm", Applied Geochemistry
/// 55 (2015) 62-71, doi:10.1016/j.apgeochem.2014.11.007. PHREEQC's
/// databases are on the approved row of PLAN.md's provenance table, so
/// that is a chain this bench may stand on.
///
/// **The world-facing comparison is back, restored 2026-09-14.** It was
/// dropped when this test was written, on the reading that the figures for
/// this solution trace to Robinson and Stokes' tabulation and that a book
/// with no resolvable identifier could not be cited. That reading was
/// wrong, and the owner has said so: a reference work without an identifier
/// is cited by author, title, edition and page like any other book, and the
/// better answer is to name the measurement behind it. Both are done in the
/// module header — [S3] for the tabulation, [S1] and [S2] for the two
/// experiments it compiles — so the two anchors this test declined to make
/// are made below.
///
/// They are worth having, because the model figure and the measured one are
/// NOT the same number and the gap is already explained. φ reads 0.945
/// against a measured 0.9324, and the freezing point −0.3516 against a
/// measured −0.346. Both gaps are the four-significant-figure rounding
/// documented below, in the direction that rounding pushes, and neither is
/// physics. An anchor whose residual is understood is more useful than one
/// that happens to sit on top of the reference.
///
/// **What these two anchors are not.** φ is not an independent check of
/// this path. `pitzer.dat`'s Na-Cl virial coefficients are FITTED to
/// osmotic-coefficient data, which is [S2] and its successors, so comparing
/// the bench's φ against [S3]'s column asks whether the fit was loaded and
/// read correctly, not whether the physics is right. The freezing point is
/// the better anchor of the two: [S1] measured a freezing point directly,
/// which is a different experiment from an isopiestic vapour-pressure one,
/// and the bench reaches it through its own ΔH_fus-derived relation rather
/// than through the fit. Recorded here because a corpus that scored these
/// two rows the same would be overstating one of them.
///
/// **The two solves are visible from the outside**, which is the whole of
/// how a reader tells this vessel apart from a one-solve one:
/// `SolutionInfo::solvent_activity` is `Some` here and `None` on the one
/// molal beaker above, and it names `pitzer.dat` while `provenance.dataset`
/// beside it still names `wateq4f.dat` — because the pH and the speciation
/// really did come from wateq4f and only the activity did not. Both are
/// asserted below, in both directions, so neither the trigger nor the
/// record can drift quietly.
#[test]
fn a_tenth_molal_brine_is_answered_by_a_model_and_not_by_raoult() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "NaCl", 0.1);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let info = vessel.solution.as_ref().expect("a solved solution");

    // The chemistry did NOT route to pitzer — this is the whole premise.
    let provenance = info.provenance.as_ref().expect("provenance");
    assert!(
        !provenance
            .model
            .starts_with(states::ION_INTERACTION_MODEL_PREFIX),
        "a tenth molal solution is inside the Debye-Hückel datasets'          validity domain and the router leaves it there: {}",
        provenance.model
    );

    // …and the activity did, in a second solve that says so.
    let second = info
        .solvent_activity
        .as_ref()
        .expect("a second speciation supplied the solvent activity");
    assert!(
        second.dataset.contains("pitzer"),
        "the activity came from the ion-interaction dataset: {}",
        second.dataset
    );
    assert!(
        second.dataset != provenance.dataset,
        "the point of the record is that these are two different datasets:          {} and {}",
        second.dataset,
        provenance.dataset
    );

    let (transitions, _) = solve::vessel_transitions(vessel);
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IonInteraction,
        "the second opinion is believed on the same terms as a first one"
    );
    assert!(
        (transitions.solute_molality - 0.2).abs() < 0.01,
        "0.1 mol of NaCl in a kilogram of water is 0.2 mol/kgw of counted          particles: {:.4}",
        transitions.solute_molality
    );
    let phi = transitions.osmotic_coefficient();
    eprintln!("0.1 molal NaCl: phi = {phi:.4}");
    // An osmotic coefficient a little under one is the whole content of the
    // ion-interaction correction at this dilution. The band states the
    // physics rather than four printed figures: `from_speciation` already
    // rejects anything outside 0.6 to 2.5 as not having come from an
    // osmotic model at all, and this narrows that to the range a 1:1
    // chloride occupies a tenth molal on pitzer.dat's own coefficients.
    assert!(
        (0.90..=0.96).contains(&phi),
        "a tenth-molal 1:1 chloride's osmotic coefficient sits a little under one: {phi:.4}"
    );
    // It reads 0.945, and the gap between that and the low 0.93s a reader
    // may have in mind is PRECISION, not physics, so it is written down
    // here rather than left to look like an error. PHREEQC's species table
    // prints four significant figures, so a_w arrives as 0.9966 rather than
    // 0.996647, and phi = -ln(a_w)/(M_w*Sum(m)) turns a 5e-5 rounding in
    // a_w into 0.013 in phi at this dilution — 1.4 %, all of it upward,
    // and worth about 0.005 K on the freezing point. That is the accuracy
    // ceiling of this route at a tenth molal and it is why the pin below is
    // a few thousandths of a kelvin wide rather than a few ten-thousandths.
    // Closer would need a_w off the wire at full precision, which is a
    // different piece of work in the readback.
    //
    // And the world. φ = 0.9324 for 0.1 mol/kg NaCl at 25 °C, from [S3]'s
    // NaCl column, measured isopiestically in [S2]. The band is argued from
    // the paragraph above rather than picked: the four-figure rounding is
    // worth 0.013 in φ at this dilution, and although it happens to push
    // upward here nothing guarantees the direction on another engine build,
    // so the band is that figure doubled. It still excludes the predecessor
    // decisively — Raoult's law does not compute an osmotic coefficient at
    // all, it asserts φ = 1 by construction, and 1.000 is 0.068 away, which
    // is 2.7 times this band.
    assert!(
        (phi - 0.9324).abs() < 0.025,
        "0.1 molal NaCl has a measured osmotic coefficient of 0.9324 at 25 °C \
         [S2, S3]; this bench says {phi:.4}"
    );

    let freezing_c = transitions.freezing_k - 273.15;
    eprintln!("0.1 molal NaCl: freezing point {freezing_c:.4} C");
    // Raoult's law, named so the thing that was wrong stays visible, and
    // computed here rather than quoted so it cannot go stale.
    // `SolventActivity::ideal` is the route this beaker used to take.
    let ideal = states::SolventActivity::ideal();
    let ideal_c = states::transitions_with(ideal, transitions.solute_molality, 101.325)
        .0
        .freezing_k
        - 273.15;
    eprintln!("0.1 molal NaCl: Raoult's law would say {ideal_c:.4} C");
    // The correction is in the direction a real solvent moves a freezing
    // point for this salt: phi under one means a_w is HIGHER than the
    // exponential form at phi = 1, so the depression is smaller.
    assert!(
        freezing_c > ideal_c,
        "an osmotic coefficient under one means less depression than Raoult's law: {freezing_c:.4} C against {ideal_c:.4} C"
    );
    // And it is worth having: at least fifteen millikelvin, which is about
    // five per cent of the whole answer. A change that quietly put this
    // beaker back on the ideal route fails here rather than passing by a
    // rounding.
    assert!(
        freezing_c - ideal_c > 0.015,
        "the correction is worth having: {:.4} K",
        freezing_c - ideal_c
    );
    // The pin. This is what THIS bench, on the datasets it ships today,
    // computes for this beaker — not a claim about the world. It moves when
    // the engine, the dataset or the relation moves, and it is meant to:
    // that is what makes it a pin rather than a measurement.
    assert!(
        (freezing_c + 0.3516).abs() < 0.004,
        "pin for the ion-interaction route at 0.1 molal: {freezing_c:.4} C"
    );
    // And what the world says, which is the anchor 2026-09-13 dropped.
    // A tenth-molal NaCl solution freezes at −0.346 °C by direct cryoscopy
    // [S1]. Ten millikelvin, argued three ways and none of them a
    // percentage. It HOLDS the bench's −0.3516, whose 5.6 mK residual is
    // the documented rounding. It SURVIVES the improvement that would close
    // that residual: a_w off the wire at full precision moves this beaker
    // to about −0.3468, which lands almost exactly on the measurement. And
    // it EXCLUDES the predecessor, Raoult's law at −0.371, by 2.5 times the
    // band. Narrower would start testing the readback's print precision
    // instead of the physics; wider would let the ideal solvent back in.
    assert!(
        (freezing_c + 0.346).abs() < 0.010,
        "a tenth-molal brine freezes at −0.346 °C by measurement [S1]: \
         {freezing_c:.4} C"
    );
}

/// A HUNDREDTH-molal brine is left alone, and that is a decision rather
/// than an oversight.
///
/// The second solve is not free, so it is not unconditional. Below about
/// 0.05 mol/kgw of dissolved particles PHREEQC's four-significant-figure
/// species table rounds a_w to 1.000, φ computes as zero,
/// `SolventActivity::from_speciation` rejects it as too coarse to carry
/// information, and the vessel would end up on the ideal route having paid
/// for a speciation to get there. The two models also agree here to about
/// three thousandths of a kelvin, which no thermometer this bench models
/// would show.
///
/// So this beaker takes ONE solve, reports no second activity, and stays
/// on Raoult's law — and the assertion that it does is what stops the
/// trigger widening by accident into "every aqueous step solves twice".
#[test]
fn a_hundredth_molal_brine_is_left_on_raoults_law_and_costs_one_solve() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "NaCl", 0.01);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let info = vessel.solution.as_ref().expect("a solved solution");
    assert!(
        info.solvent_activity.is_none(),
        "no second speciation is worth an engine call at 0.02 mol/kgw of          particles: {:?}",
        info.solvent_activity
    );

    let (transitions, _) = solve::vessel_transitions(vessel);
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IdealSolution,
        "and the route says so rather than implying a model that was never          consulted"
    );
    let freezing_c = transitions.freezing_k - 273.15;
    eprintln!("0.01 molal NaCl: freezing point {freezing_c:.4} C (ideal route)");
    // The two routes agree here, which is the argument for not asking.
    // Computed from the relation rather than quoted: at this particle
    // molality an osmotic coefficient of 0.93 and Raoult's law differ by a
    // few thousandths of a kelvin, which no thermometer this bench models
    // would show and no engine call is worth.
    let modelled = states::transitions_with(
        states::SolventActivity::from_speciation(
            (-0.93 * 0.018015 * transitions.solute_molality).exp(),
            transitions.solute_molality,
            transitions.solute_molality / 2.0,
        ),
        transitions.solute_molality,
        101.325,
    )
    .0
    .freezing_k
        - 273.15;
    eprintln!("0.01 molal NaCl: an ion-interaction route would say {modelled:.4} C");
    assert!(
        (freezing_c - modelled).abs() < 0.005,
        "below the floor the two routes agree to a few thousandths of a kelvin: ideal {freezing_c:.4} C against a modelled {modelled:.4} C"
    );
}

/// A saturated chloride brine boils where a cook finds it, and that is the
/// other half of the same correction.
///
/// Boiling-point elevation does NOT need its own treatment — it is the same
/// derivation with the vapour on the other side of the equality, and it
/// takes the same a_w — but it does need its own test, because the
/// correction changes SIGN, and a relation pinned only in the dilute half
/// would be pinned only where the two laws agree in direction.
///
/// Two crossings, and they are not the same crossing. A chloride's osmotic
/// coefficient passes one near 2.3 molal (0.955 at 1.5, 0.983 at 2.0, 1.013
/// at 2.5 by measurement), which is where a_w stops falling faster than the
/// ideal solvent's and starts falling slower. The relation itself overtakes
/// ΔT = K·m a little later, because the logarithm's own curvature holds it
/// back: at 2 molal NaCl the depression is 7.12 K against the law's 7.44,
/// and it is not until about 2.9 molal that it passes. So below roughly
/// three molal the dilute law OVER-states both shifts and above it
/// UNDER-states them, which is why one test at each end is the least this
/// can be pinned with.
///
/// Six molal NaCl is the second case: pitzer.dat reports φ = 1.27, the bench
/// says 108.0 °C, and the dilute law said 106.1. A correction worth nearly
/// two degrees, in the opposite direction from the one the freezing test
/// pins.
///
/// **This test deliberately has NO world-facing band, and the reason is a
/// correction rather than a gap in the sources.** This file and HISTORY.md
/// have both been quoting 108.7 °C as the measurement this beaker should be
/// compared against, in this file's own words "a measurement of a saturated
/// brine". But the beaker below is 6.000 mol/kg, and a saturated chloride
/// brine at its BOILING point is more concentrated than that: the test
/// itself records halite saturating near 6.11 mol/kgw at the vessel's
/// temperature, and a chloride's solubility rises as it is heated. So 108.7
/// and 108.0 are not two answers for one solution, they are answers for two
/// solutions, and the 0.7 K between them is mostly composition. Adding that
/// comparison as a band would have been the exact defect this bench has a
/// name for — a quantity checked against a proxy that moves for a reason
/// nobody controlled.
///
/// A real anchor for this row exists and has not been bought: [S4] measured
/// the vapour pressure of NaCl(aq) from 298 to 373 K over 1 to 6 mol/kg,
/// which is this beaker exactly, at this temperature, to this molality.
/// Until someone reads its table, one further question stays open with it:
/// the bench took φ from a speciation solved near the vessel's ambient
/// temperature and used it at the boil, and whether φ for this salt rises or
/// falls over that interval is precisely what [S4] settles. The accuracy
/// corpus carries this as an open row rather than a passing one.
#[test]
fn a_saturated_brine_boils_hotter_than_the_dilute_law_allows() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "NaCl", 6.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IonInteraction
    );
    assert!(
        transitions.osmotic_coefficient() > 1.0,
        "a concentrated chloride's osmotic coefficient passes one, and that \
         is why the dilute law under-states this end: {:.3}",
        transitions.osmotic_coefficient()
    );
    let boiling_c = transitions.boiling_k - 273.15;
    assert!(
        (boiling_c - 108.0).abs() < 0.4,
        "a saturated brine boils near 108 C: {boiling_c:.2} C"
    );
    assert!(
        transitions.boiling_elevation()
            > states::ebullioscopic_constant() * transitions.solute_molality,
        "above three molal the relation is STEEPER than the dilute law, not \
         shallower: {:.3} K against the law's {:.3} K",
        transitions.boiling_elevation(),
        states::ebullioscopic_constant() * transitions.solute_molality
    );
    // And it is still inside the dataset's stated range. This brine's
    // ionic strength is about 6.2 mol/kgw — halite saturates near 6.11 in
    // this dataset — against the ion-interaction ceiling of 20, which
    // `pitzer.dat`'s own evaporite sequence justifies: it carries
    // `MgCl2_4H2O` and `Carnallite`, and bischofite saturates near I = 17.
    assert!(
        transitions.within_model_range(),
        "a saturated chloride brine is the case pitzer.dat exists for"
    );

    // …and the stage shows the boil while withholding the freeze, because
    // the two boundaries are different boundaries. The activity model is
    // inside its range here, so the boil is an ordinary answer; the
    // liquidus this relation computes is −25.7 °C, which is past the 252 K
    // eutectic boundary `StateEquilibrator` refuses to cross. Drawing that
    // plateau would promise a temperature the solver has already declined
    // to reach. The dilute law printed −22.3 °C for the same beaker and was
    // wrong the same way, only by less.
    let picture = scene::scene_vessel(vessel);
    assert!(
        picture.melting_point_k.is_none(),
        "a saturated brine's liquidus is below the declared boundary and \
         must not be drawn: {:?}",
        picture.melting_point_k
    );
    assert!(
        picture
            .boiling_point_k
            .is_some_and(|k| (k - transitions.boiling_k).abs() < 1e-9),
        "the boil is unaffected and is the same number the state layer \
         computed: {:?}",
        picture.boiling_point_k
    );
    assert!(transitions.freezing_k < states::BRINE_MODEL_MIN_K);
}

/// One mole of sucrose in one kilogram of water: 342 g, and it boils at
/// 100.5 °C.
///
/// Sucrose is the case that has to be got right by counting rather than by
/// speciating: it is a non-electrolyte, no shipped database carries it, and
/// reading that silence as "nothing is dissolved" is what used to make a
/// sugar solution boil at exactly 100.0 °C.
///
/// It is also the case that shows what the 2026-09-11 activity route does
/// NOT do. No database carries sucrose, so no speciation computed a solvent
/// activity for this beaker and Raoult's law stands in: a_w is the
/// solvent's mole fraction, the route says `IdealSolution` rather than
/// pretending otherwise, and the answers move by the curvature of the
/// logarithm alone — the depression from 1.859 K to 1.831, the elevation
/// from 0.513 K to 0.509. Measurements of one molal sucrose put the
/// depression at about 1.86, because this sugar's own osmotic coefficient
/// is 1.02 rather than 1 and pushes back almost exactly as far as the
/// linearisation did. The bench does not know that number and does not
/// pretend to: a two per cent shortfall, declared ideal, is the honest
/// form of not knowing, and it is inside this test's band from both sides.
///
/// **The measured side now has a source.** One molal sucrose is 25.5 per
/// cent by mass, and the ice liquidus of the sucrose-water system at that
/// composition is [S5]; sucrose's osmotic coefficient at 25 °C is in [S2],
/// the same isotonic series that carries this file's chloride numbers,
/// which is why one paper covers three of the six rows here. The 0.05 K
/// band below is the honest width for a row whose model and reference agree
/// for DIFFERENT reasons: the bench is short by the 2 % it declares, the
/// reference is above K_f by about the 2 % sucrose's own non-ideality adds,
/// and the two nearly cancel. A tighter band would be reading precision
/// into a coincidence.
#[test]
fn a_mole_of_sugar_raises_the_boiling_point_by_half_a_degree() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "sucrose", 1.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IdealSolution,
        "no database carries sucrose, so nothing computed a solvent activity \
         for it and the bench must say so rather than assume one"
    );
    let boiling_c = transitions.boiling_k - 273.15;
    assert!(
        (boiling_c - 100.51).abs() < 0.1,
        "342 g of sucrose in a kilogram of water boils at 100.5 C; this bench \
         says {boiling_c:.3} C from {:.4} mol/kg of particles",
        transitions.solute_molality
    );
    assert!(
        (transitions.boiling_elevation() - 0.512).abs() < 0.02,
        "the elevation is K_b itself at one molal: {:.4} K",
        transitions.boiling_elevation()
    );
    // And the other end of the same particle count: a non-electrolyte
    // depresses the freezing point by K_f, four times as far.
    assert!(
        (transitions.freezing_depression() - 1.86).abs() < 0.05,
        "the same one molal depresses freezing by K_f = 1.86 K: {:.4} K",
        transitions.freezing_depression()
    );
}

/// Road salt, put the way a learner asks it: does the ice on the path
/// actually melt, and does the bench agree that sugar would be a worse buy?
///
/// Equal MASSES, because that is what is in the bag. 58.4 g of salt is one
/// mole and dissociates; 58.4 g of sucrose is 0.17 mol and does not. The
/// factor of about twelve between the two depressions is the entire
/// argument for why the gritter carries salt.
#[test]
fn salt_beats_sugar_on_the_path_by_about_a_factor_of_twelve() {
    let depression = |key: &str, moles: f64| {
        let mut bench = Bench::new();
        let mut solvers = stack();
        add(&mut bench, &mut solvers, "water", 55.51);
        add(&mut bench, &mut solvers, key, moles);
        let vessel = bench.vessel(VesselId(0)).unwrap();
        solve::vessel_transitions(vessel).0.freezing_depression()
    };
    // 58.44 g of each: 1.0 mol of NaCl, 0.1707 mol of sucrose.
    let salt = depression("NaCl", 1.0);
    let sugar = depression("sucrose", 0.1707);
    assert!(
        salt / sugar > 9.0 && salt / sugar < 15.0,
        "equal masses are not equal particle counts: salt {salt:.3} K against \
         sugar {sugar:.3} K, a ratio of {:.1}",
        salt / sugar
    );
}
