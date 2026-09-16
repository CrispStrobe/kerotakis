//! Perturbation tests: does a number MOVE, in the right direction, when
//! its stated cause moves?
//!
//! This is the sibling of `metamorphic.rs` and the generalisation, across
//! time, of `tools/curiosity-answer-invariance.py`. That tool asks a
//! question across *vessels* — a prompt that distinguishes three metals
//! must produce three different answers — and caught `mat-012`, a row that
//! passed for as long as the corpus existed while weighing five grams of
//! each metal and reading the same number three times.
//!
//! The same question asked across *a change to an input* is the one this
//! file poses: change a cause the bench says it depends on, and the
//! consequence must move; change something the bench says it is
//! independent of, and the consequence must not. Neither half needs a
//! reference value, so nothing here has to be sourced, cited or licensed,
//! and nothing has to be re-blessed when the solver improves.
//!
//! Every expensive defect found here in the last fortnight has that shape
//! and every one was found by hand:
//!
//! | defect | the perturbation that finds it |
//! |---|---|
//! | float-or-sink read a species density, not the recipe's bulk density | move the recipe's bulk density; the verdict must move |
//! | alkalinity treated as a portion, not a charge balance | add acid; it must fall by the charge equivalent |
//! | an element arriving mid-solve was dropped on readback | transfer one in; the totals must change |
//! | a rate law keyed on the bottle rather than the ion | speciate the reagent; the reaction must still fire |
//!
//! **What this file is careful about.** A perturbation that moves an input
//! the engine simply multiplies by always passes and proves nothing. Each
//! test below says, in its own doc comment, what it establishes and what
//! it cannot — and the weak ones say that they are weak rather than being
//! quietly counted as coverage.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Distinguishes concurrent `run` calls — see the note in `metamorphic.rs`:
/// a content-derived directory name once collided between two threads.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script through `kero run --json` and parse the step stream.
fn run(script: &str) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!(
        "kero-perturb-{}-{}",
        std::process::id(),
        CASE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("case.lab");
    std::fs::write(&lab, script).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        out.status.success(),
        "script failed:\n{script}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let steps = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).expect("every line is JSON"))
        .collect();
    std::fs::remove_dir_all(&dir).ok();
    steps
}

/// Named vessel state after the last step.
fn vessel(steps: &[serde_json::Value], index: usize) -> serde_json::Value {
    steps.last().expect("at least one step")["bench"]["vessels"][index].clone()
}

/// The `contents` key the aqueous tail books its analytical BASE
/// equivalents under. Named here rather than imported so this file reads
/// the wire, like any other `--json` client, and would notice a rename.
const BASE_EQUIVALENTS: &str = "base_equivalents";

/// Total moles of one species in a vessel, across phases.
fn moles_of(vessel: &serde_json::Value, species: &str) -> f64 {
    vessel["contents"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["species"] == species)
        .map(|p| p["moles"].as_f64().unwrap())
        .sum()
}

fn ph(vessel: &serde_json::Value) -> f64 {
    vessel["solution"]["ph"]
        .as_f64()
        .unwrap_or_else(|| panic!("no characterised solution: {vessel}"))
}

fn ionic_strength(vessel: &serde_json::Value) -> f64 {
    vessel["solution"]["ionic_strength"].as_f64().unwrap()
}

/// One species' molality out of the reported speciation — the solver's own
/// answer, not the inventory it was handed. 0.0 when the solver did not
/// name it, which is the state the readback defect produced.
fn molality_of(vessel: &serde_json::Value, name: &str) -> f64 {
    vessel["solution"]["species"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|s| s["name"] == name)
                .filter_map(|s| s["molality"].as_f64())
                .sum()
        })
        .unwrap_or(0.0)
}

/// Carbonate alkalinity as a CHARGE: the proton deficiency of the solution,
/// summed over the species the solver reported. Deliberately computed from
/// the readback rather than from what was poured in — that is the whole
/// point of the test below.
fn carbonate_alkalinity(vessel: &serde_json::Value) -> f64 {
    // `base_equivalents` and `H+` are the two nonnegative halves of one
    // signed quantity — the analytical acid/base coordinate the tail books
    // to close its H/O balance — so the `OH⁻ − H⁺` term of the
    // electroneutrality identity is their difference. The base half was
    // published as `OH-` until 2026-09-16; see case 7/7 for why it is not.
    moles_of(vessel, "HCO3-") + 2.0 * moles_of(vessel, "CO3-2")
        + moles_of(vessel, BASE_EQUIVALENTS)
        - moles_of(vessel, "H+")
}

/// Dissolved inorganic carbon, over every carbonate species the readback
/// may carry it in.
fn dissolved_inorganic_carbon(vessel: &serde_json::Value) -> f64 {
    ["CO2(aq)", "H2CO3", "HCO3-", "CO3-2"]
        .iter()
        .map(|s| moles_of(vessel, s))
        .sum()
}

// ===================================================================== 1/6
// Mechanism: acid-base charge balance in the aqueous tail.

/// **Alkalinity is a charge, not a portion of something.** Add a strong
/// acid to a bicarbonate solution and the alkalinity must fall by the
/// charge equivalent — one mole per mole of H⁺ — while the sodium that
/// came with it does not move at all, because sodium is a spectator and
/// alkalinity is not a substance it is made of.
///
/// This is the perturbation that finds the defect PLAN.md records as the
/// tail eating NaOH and H₂SO₄: an alkalinity treated as a *portion* of the
/// inventory tracks whatever it was apportioned from, so it moves by the
/// wrong amount, or moves when sodium moves, or does not move at all.
///
/// Three claims, and they are not the same claim three times:
///
/// * **Directional, near-exact.** Alkalinity falls 1:1 with the acid, and
///   keeps falling 1:1 when the acid is quadrupled — so a constant offset,
///   or a proportionality with the wrong constant, fails. Measured: 0.01000
///   → 0.00900 → 0.00800 → 0.00600 mol against 0, 1, 2 and 4 mmol of HCl,
///   exact to eight decimals.
/// * **Invariant.** Sodium does not move. A quantity computed by
///   apportioning the inventory almost always drags a spectator with it.
/// * **Directional, independent of the first two.** Dissolved inorganic
///   carbon *falls* (0.00923 → 0.00569 mol) while the CO₂ partial pressure
///   above the open beaker stays at the room's own 3.90e-4 atm. The acid
///   converts bicarbonate to CO₂ and the room takes it away; the beaker is
///   buffered by the atmosphere, not by its own carbon.
///
/// **What this establishes:** the proton is booked against the carbonate
/// system as charge, in the reported speciation, in the right stoichiometry
/// and with the right spectators left alone; and the open-vessel gas
/// exchange is driven by the acid rather than pinned to the initial state.
///
/// **What it cannot establish:** that any equilibrium constant is right.
/// Every K could be wrong together and all three claims would still hold.
///
/// **How weak is it?** The 1:1 fall is, algebraically, the electroneutrality
/// identity (Na⁺ − Cl⁻ = HCO₃⁻ + 2CO₃²⁻ + OH⁻ − H⁺), so a solver that
/// balances charge cannot fail it by arithmetic — this is the case in this
/// file closest to a tautology, and it is kept because the identity is
/// evaluated over the **reported readback**, which is exactly where both
/// the alkalinity defect and the dropped-element defect lived. The carbon
/// and pCO₂ halves are not implied by charge balance and carry the
/// test's independent weight.
#[test]
fn acid_takes_alkalinity_away_as_charge_and_leaves_the_spectator() {
    let brine = |acid_mol: f64| {
        let mut script = String::from("add v1 water 1000mL\nadd v1 NaHCO3 0.01mol\n");
        if acid_mol > 0.0 {
            script.push_str(&format!("add v1 HCl {acid_mol}mol\n"));
        }
        vessel(&run(&script), 0)
    };

    let baseline = brine(0.0);
    let alkalinity0 = carbonate_alkalinity(&baseline);
    assert!(
        (alkalinity0 - 0.010).abs() < 1e-6,
        "0.01 mol of bicarbonate is 0.01 equivalents of alkalinity: {alkalinity0}"
    );

    for acid in [0.001, 0.002, 0.004] {
        let after = brine(acid);
        let fell = alkalinity0 - carbonate_alkalinity(&after);
        assert!(
            (fell - acid).abs() < 1e-6,
            "{acid} mol of strong acid must take {acid} equivalents of \
             alkalinity, not {fell}"
        );
        // The spectator does not move. A portion-shaped alkalinity drags it.
        let sodium = moles_of(&after, "Na+");
        assert!(
            (sodium - 0.010).abs() < 1e-9,
            "sodium is a spectator; {acid} mol of acid moved it to {sodium}"
        );
        // pH must fall too — weakly, because this is a buffer. Direction
        // only: the size of the fall is case 2's business.
        assert!(
            ph(&after) < ph(&baseline),
            "acid must lower pH: {} -> {}",
            ph(&baseline),
            ph(&after)
        );
    }

    // The carbon leaves; the room's CO2 pressure does not budge. Neither
    // half follows from charge balance.
    let (before, after) = (brine(0.0), brine(0.004));
    let (carbon_before, carbon_after) = (
        dissolved_inorganic_carbon(&before),
        dissolved_inorganic_carbon(&after),
    );
    assert!(
        carbon_after < carbon_before * 0.8,
        "acidifying an open carbonate solution must drive carbon off: \
         {carbon_before} -> {carbon_after} mol"
    );
    let pressure = |v: &serde_json::Value| v["co2_partial_pressure_atm"].as_f64().unwrap();
    assert!(
        (pressure(&after) - pressure(&before)).abs() < 1e-6,
        "an open beaker is held at the room's pCO2, whatever it is: {} -> {}",
        pressure(&before),
        pressure(&after)
    );
}

// ===================================================================== 2/6
// Mechanism: buffering — the same perturbation, two solutions, two
// responses that must differ by a lot.

/// **The same acid, into two beakers, must land very differently.** This is
/// the sharpest shape in the file: a DIFFERENTIAL perturbation. A single
/// vessel's "pH fell when I added acid" is passed by any rule that lowers
/// pH on sight of an acid; a rule like that cannot pass this one, because
/// it would move both beakers by the same amount.
///
/// Measured, for 2 mmol of HCl: plain water 6.997 → 2.718, a fall of 4.28;
/// an equimolar acetate buffer 4.663 → 4.628, a fall of 0.035. Two orders
/// of magnitude apart, from one identical perturbation.
///
/// The invariant half rides along in the same runs, and is the one that
/// would catch a buffer implemented by fudging the pH: **total acetate does
/// not change.** Acid protonates acetate; it does not create or destroy it.
/// Measured 0.100000 mol at every acid loading, and the conjugate base
/// falls essentially mole-for-mole with the acid (−0.001998 for 2 mmol,
/// −0.007977 for 8 mmol) — so the pH is being held by a pair that is
/// actually moving, rather than by a number that was clamped.
///
/// **What this establishes:** buffering is computed from the conjugate pair
/// the beaker actually contains, the pair moves by the acid's charge, and
/// the acetate inventory is conserved while it does.
///
/// **What it cannot establish:** that the buffer's pH (4.66) or its
/// capacity is right — only that the two beakers are being solved
/// differently, for a reason located in their contents. The assertion is a
/// ratio between two responses rather than either response's value, which
/// is why it survives a solver change that moves both.
///
/// **Weakness:** low. Nothing here multiplies an input the engine was
/// handed. The 30x threshold is chosen an order of magnitude below the
/// measured 122x so that a genuine improvement in either solver does not
/// need it re-blessed; it is not a measurement and must not be read as one.
#[test]
fn a_buffer_resists_the_acid_that_flattens_plain_water() {
    const BUFFER: &str = "add v1 water 1000mL\nadd v1 CH3COOH 0.05mol\nadd v1 NaOAc 0.05mol\n";
    const WATER: &str = "add v1 water 1000mL\n";
    let with = |base: &str, acid: f64| {
        let mut script = base.to_string();
        if acid > 0.0 {
            script.push_str(&format!("add v1 HCl {acid}mol\n"));
        }
        vessel(&run(&script), 0)
    };

    let water_fall = ph(&with(WATER, 0.0)) - ph(&with(WATER, 0.002));
    let buffer_fall = ph(&with(BUFFER, 0.0)) - ph(&with(BUFFER, 0.002));
    assert!(
        water_fall > 0.0 && buffer_fall > 0.0,
        "acid lowers pH in both beakers: water {water_fall}, buffer {buffer_fall}"
    );
    assert!(
        water_fall > 30.0 * buffer_fall,
        "the same 2 mmol of acid must move unbuffered water far more than a \
         buffer: {water_fall} vs {buffer_fall} pH units"
    );

    // The pair moves; the element does not. Both loadings, so a fixed
    // offset cannot pass as a stoichiometric response.
    let base = with(BUFFER, 0.0);
    let acetate_total = |v: &serde_json::Value| moles_of(v, "CH3COOH") + moles_of(v, "CH3COO-");
    let total0 = acetate_total(&base);
    assert!(
        (total0 - 0.100).abs() < 1e-6,
        "0.05 mol of each is 0.1 mol of acetate: {total0}"
    );
    for acid in [0.002, 0.008] {
        let after = with(BUFFER, acid);
        let total = acetate_total(&after);
        assert!(
            (total - total0).abs() < 1e-6,
            "acid protonates acetate, it does not consume it: {total0} -> {total} mol"
        );
        let conjugate_lost = moles_of(&base, "CH3COO-") - moles_of(&after, "CH3COO-");
        // 1% of the perturbation: the residual is the hydroxide the readback
        // carries as the solution's charge, which is a real equilibrium
        // quantity and moves a little too.
        assert!(
            (conjugate_lost - acid).abs() < 0.01 * acid,
            "{acid} mol of acid must protonate {acid} mol of acetate, not {conjugate_lost}"
        );
    }
}

// ===================================================================== 3/6
// Mechanism: the rate law's temperature dependence, per reaction.

/// **Two reactions, the same ten degrees, two different answers.** Warming
/// a vessel and watching a reaction speed up proves almost nothing: the
/// integrator multiplies by an Arrhenius factor, so of course it moves.
/// That is the tautology this file is supposed to be suspicious of, and it
/// is why this test compares TWO mechanisms rather than measuring one.
///
/// The codex (`codex/rates.toml`) states the claim this perturbation is
/// derived from: the thiosulfate clock has a barrier of about 51 kJ/mol and
/// obeys the ten-degree rule, while uncatalysed peroxide decomposition sits
/// near 75 kJ/mol and does not — "the rule is a coincidence of one barrier
/// height, not a law". A steeper barrier is *more* temperature-sensitive,
/// which is the misconception the codex entry exists to correct, and it is
/// checkable without knowing either barrier.
///
/// Measured over 25 → 35 °C: peroxide 7.26e-4 → 1.91e-3 mol of O₂, a factor
/// of 2.64; thiosulfate 4.43e-5 → 8.41e-5 mol of sulfur in two seconds, a
/// factor of 1.90.
///
/// **The two-second wait is load-bearing and is the thing most likely to
/// rot.** The codex's own lv3 note says a finite-time extent stops being a
/// rate proxy once the reagent runs down, and it is right: at the ten
/// seconds the codex scenario uses, thiosulfate's factor has already sagged
/// from 1.90 to 1.74 because the warmer beaker is eating its acid faster.
/// At two seconds about 4% of the acid is gone. If this test ever has to be
/// loosened, the first thing to check is whether the window still sits
/// inside the initial-rate regime — not whether the threshold needs moving.
///
/// **What this establishes:** each reaction's own activation energy reaches
/// the integrator. A global temperature factor, a rate law that lost its
/// Ea, or an Ea shared across the mechanism table all produce two equal
/// factors and fail here.
///
/// **What it cannot establish:** that either activation energy is right,
/// or that either absolute rate is. Both barriers could be wrong by the
/// same ratio and this passes.
///
/// **Weakness: this is the weakest test in the file, and for the reason the
/// brief warns about.** Given that the engine computes exp(−Ea/RT) with a
/// per-reaction Ea, "the bigger Ea gives the bigger factor" is arithmetic,
/// not chemistry — the perturbation is close to the path it tests. It earns
/// its place on plumbing rather than physics: it asserts that two different
/// curated numbers are still reaching two different reactions through the
/// real binary, which is the failure mode of a rate law keyed on the wrong
/// thing. It should not be counted as evidence about temperature dependence.
#[test]
fn a_steeper_barrier_is_the_more_temperature_sensitive_one() {
    let peroxide = |celsius: u32| {
        let script = format!(
            "add v1 water 100mL @ {celsius}C\n\
             add v1 H2O2 0.1mol @ {celsius}C\n\
             wait 30min\n"
        );
        moles_of(&vessel(&run(&script), 0), "O2")
    };
    let clock = |celsius: u32| {
        let script = format!(
            "add v1 water 100mL @ {celsius}C\n\
             add v1 Na2S2O3 0.79g @ {celsius}C\n\
             add v1 HCl 0.002mol @ {celsius}C\n\
             wait 2s\n"
        );
        moles_of(&vessel(&run(&script), 0), "S")
    };

    let (cold_peroxide, warm_peroxide) = (peroxide(25), peroxide(35));
    let (cold_clock, warm_clock) = (clock(25), clock(35));
    for (what, cold) in [("peroxide", cold_peroxide), ("clock", cold_clock)] {
        assert!(
            cold > 0.0,
            "{what} must actually react at 25 C, or there is no rate to compare: {cold}"
        );
    }
    let steep = warm_peroxide / cold_peroxide;
    let shallow = warm_clock / cold_clock;
    assert!(
        shallow > 1.2 && steep > 1.2,
        "ten degrees must speed both reactions up: clock {shallow}x, peroxide {steep}x"
    );
    assert!(
        steep > 1.25 * shallow,
        "peroxide's barrier is the steeper one, so its ten-degree factor must \
         be clearly the larger: {steep}x against the clock's {shallow}x"
    );
}

// ===================================================================== 4/6
// Mechanism: buoyancy as a comparison between two densities.

/// **Move the liquid, and the verdict must move with it.** The object never
/// changes: the same potato goes into the same 500 mL of water every time,
/// and only the sugar dissolved in it varies. Floating is a comparison, so
/// perturbing either side of the comparison must be able to flip it.
///
/// This is the perturbation that finds the defect recorded against
/// `float_or_sink`: a buoyancy path that reads the density of the SPECIES a
/// material resolves into rather than the recipe's whole-object bulk
/// density. A potato resolves to starch and cellulose at about 1.5 g/mL, so
/// that path sinks it in every liquid on this sweep and never flips.
///
/// **Nothing here is pinned.** The test reads the object's own declared
/// bulk density out of the scene (`bulk_objects[].bulk_density_g_per_ml`)
/// and the liquid's density off the densitometer, and asserts only that the
/// verdict agrees with the comparison between them — at every point. The
/// threshold is whatever the registry says it is; change the potato's
/// reviewed density and the test follows it rather than failing.
///
/// Measured: 0.997 g/mL sunk, 1.063 sunk, 1.091 floating, 1.159 floating,
/// against a declared 1.08. Two surfaces that could disagree — the
/// instrument and the appearance path — cross at the same place.
///
/// **What this establishes:** whole-object bulk density is what the float
/// comparison uses; the liquid side of it is computed rather than assumed
/// to be water; the flip happens where the two densities cross; and all
/// THREE surfaces agree about it — the densitometer, the scene's
/// `position`, and the sentence a person reads. The third was added after
/// mutating the engine: the comparison is made twice, and hard-coding only
/// the rendered one against water left the scene saying `floating` beside
/// the words "is at the bottom", which the first draft of this test — which
/// read only the scene — passed.
///
/// **What it cannot establish:** that 1.08 g/mL is the right density for a
/// potato, or that a sucrose solution really reaches 1.16 g/mL at this
/// loading. It checks the comparison, not either number in it — which is
/// the point, because the numbers are data and the comparison is code.
///
/// **Weakness: low, with one caveat.** The caveat is that sugar is the only
/// way to raise the liquid's density here: a salt solution reads as pure
/// water, because every ion carries water's density as a structural
/// default, and the bench says so (`ionic_volume_unaccounted`). So this
/// sweep exercises the one route that is fully wired, and a reader should
/// not take it as evidence that the liquid density is right in general.
#[test]
fn the_float_verdict_tracks_the_liquid_it_is_compared_against() {
    let sweep: Vec<(f64, f64, String, String)> = [0u32, 100, 150, 300]
        .iter()
        .map(|grams| {
            let mut script = String::from("add v1 water 500mL\n");
            if *grams > 0 {
                script.push_str(&format!("add v1 sucrose {grams}g\n"));
            }
            script.push_str("add v1 potato 5g\nmeasure v1 density\nlook v1\n");
            let steps = run(&script);
            let liquid = steps
                .iter()
                .flat_map(|step| step["events"].as_array().unwrap())
                .filter(|e| e["event"] == "measured" && e["instrument"] == "densitometer")
                .filter_map(|e| e["value"].as_f64())
                .next_back()
                .expect("the densitometer reported");
            let object = steps.last().unwrap()["scene"]["vessels"][0]["bulk_objects"]
                .as_array()
                .and_then(|list| list.first())
                .unwrap_or_else(|| panic!("no bulk object: the potato is not on the bench"));
            let words = steps.last().unwrap()["scene"]["vessels"][0]["words"]
                .as_str()
                .unwrap()
                .to_string();
            (
                liquid,
                object["bulk_density_g_per_ml"].as_f64().unwrap(),
                object["position"].as_str().unwrap().to_string(),
                words,
            )
        })
        .collect();

    let declared = sweep[0].1;
    for (liquid, object, position, words) in &sweep {
        assert!(
            (object - declared).abs() < 1e-12,
            "the potato's own density cannot depend on the syrup around it: \
             {declared} then {object}"
        );
        let expected = if liquid > object { "floating" } else { "sunk" };
        assert_eq!(
            position, expected,
            "in a liquid of {liquid} g/mL an object of {object} g/mL is {expected}"
        );
        // The SAME comparison is made twice in the engine — once for the
        // scene's `position` and once for the sentence a person reads — and
        // a defect in either alone leaves the other one consistent. Found by
        // mutating the engine rather than the test: hard-coding the rendered
        // comparison against water instead of the liquid actually present
        // left `position: floating` beside the words "is at the bottom", and
        // an earlier draft of this test, which read only the scene, passed.
        let said = if *expected == *"floating" {
            "floats on top"
        } else {
            "is at the bottom"
        };
        assert!(
            words.contains(said),
            "the scene says {position} and the sentence says something else, \
             in a liquid of {liquid} g/mL against an object of {object} g/mL: \
             {words:?}"
        );
    }

    // The sweep must actually straddle the crossing, or every assertion
    // above is vacuously satisfied by one verdict repeated four times —
    // which is precisely the shape `curiosity-answer-invariance.py` exists
    // to catch, and it would be embarrassing to reintroduce it here.
    let densities: Vec<f64> = sweep.iter().map(|(liquid, ..)| *liquid).collect();
    assert!(
        densities.windows(2).all(|pair| pair[1] > pair[0]),
        "more sugar must mean a denser liquid: {densities:?}"
    );
    assert!(
        sweep.first().unwrap().2 == "sunk" && sweep.last().unwrap().2 == "floating",
        "the sweep has to cross the potato's density, or it proves nothing: {sweep:?}"
    );
}

// ===================================================================== 5/6
// Mechanism: an element that arrives after the destination was solved.

/// **Pour magnesium into a beaker that has already been solved, and it must
/// still be there when the beaker is read back.** The destination starts as
/// a characterised sodium chloride solution; the magnesium sulfate arrives
/// by `decant`, which is the path on which an element is speciated and then
/// dropped on readback — the defect recorded against the partition's
/// element totals.
///
/// A single transfer would be a thin check, because a dropped element
/// reappears the moment anything re-solves. The perturbation is the
/// FRACTION transferred, and four claims ride on it:
///
/// * **Directional and exactly proportional.** The magnesium in the
///   destination is 0.005 / 0.010 / 0.015 mol for a quarter, a half and
///   three quarters of a 0.02 mol source — linear in the perturbation, so
///   an element that arrives and is partly lost fails.
/// * **Conserved.** What the source loses, the destination gains, exactly.
/// * **Speciated, not merely inventoried.** The magnesium appears in the
///   solver's own reported species list with a non-zero molality, which is
///   the half the readback defect broke. Sitting in `contents` with no
///   speciation is what an unrepresented salt looks like, and it is not
///   good enough.
/// * **Intensive quantities do NOT follow the same line.** The molality
///   rises 0.0137 → 0.0206 → 0.0250 while the moles rise 1:2:3, because the
///   destination's volume grows with the pour and magnesium sulfate ion
///   pairs. Asserting that the amount is linear AND the molality is not is
///   what separates "the element was carried" from "a number was scaled".
///
/// **What this establishes:** an element arriving mid-solve survives into
/// both the inventory and the speciation, in proportion to what was poured,
/// with the source debited the same amount.
///
/// **What it cannot establish:** that the resulting ionic strength or
/// ion-pairing is right. It asserts the shape of the response, not its
/// value.
///
/// **Weakness: low.** The amount transferred is linear in the fraction by
/// construction — that half IS close to the path it tests, and on its own
/// it would be near-tautological. The weight is carried by the two claims
/// that are not: that the solver names the element at all, and that the
/// intensive quantity departs from the extensive one.
#[test]
fn an_element_poured_into_a_solved_beaker_survives_the_readback() {
    let pour = |fraction: f64| {
        let mut script = String::from(
            "add v1 water 200mL\nadd v1 MgSO4 0.02mol\n\
             new\nadd v2 water 200mL\nadd v2 NaCl 0.01mol\n",
        );
        if fraction > 0.0 {
            script.push_str(&format!("decant v1 v2 {fraction}\n"));
        }
        let steps = run(&script);
        (vessel(&steps, 0), vessel(&steps, 1))
    };

    let (_, untouched) = pour(0.0);
    assert!(
        moles_of(&untouched, "Mg+2") == 0.0,
        "the destination starts with no magnesium, or the test proves nothing"
    );
    let baseline_strength = ionic_strength(&untouched);

    let mut molalities = Vec::new();
    let mut strengths = Vec::new();
    for fraction in [0.25, 0.5, 0.75] {
        let (source, destination) = pour(fraction);
        let arrived = moles_of(&destination, "Mg+2");
        assert!(
            (arrived - 0.02 * fraction).abs() < 1e-9,
            "pouring {fraction} of 0.02 mol must deliver {} mol, not {arrived}",
            0.02 * fraction
        );
        let left = moles_of(&source, "Mg+2");
        assert!(
            (left + arrived - 0.02).abs() < 1e-9,
            "magnesium is conserved across the pour: {left} left, {arrived} arrived"
        );
        // The half the readback defect broke: named by the solver, not just
        // carried in the inventory.
        let molality = molality_of(&destination, "Mg+2");
        assert!(
            molality > 0.0,
            "the arriving element must be speciated, not merely inventoried: \
             {arrived} mol of magnesium and no Mg+2 in the species list"
        );
        molalities.push(molality);
        strengths.push(ionic_strength(&destination));
    }

    assert!(
        strengths[0] > baseline_strength && strengths.windows(2).all(|pair| pair[1] > pair[0]),
        "more magnesium sulfate means a stronger solution: {baseline_strength} then {strengths:?}"
    );
    // Extensive rises 1:2:3; intensive does not. If the molality had simply
    // been scaled with the amount, the two gaps below would be equal.
    let first_gap = molalities[1] - molalities[0];
    let second_gap = molalities[2] - molalities[1];
    assert!(
        second_gap < 0.9 * first_gap,
        "the amount is linear in the pour and the molality must not be — the \
         beaker grew and the ions paired: {molalities:?}"
    );
}

// ===================================================================== 6/6
// Mechanism: what is extensive and what is intensive.

/// **A claimed independence, and the sensitivity beside it.** The codex
/// entry `moles-from-mass` states this outright, as the cure for its own
/// named misconception: "the same 5.844 g into 100 mL and into 200 mL, and
/// both vessels `inspect` to 0.1000 mol. Only the mol-per-litre moved — the
/// count you weighed is not negotiable by water."
///
/// So the bench has told us what must not move, and a perturbation test can
/// simply take it at its word. Adding water is the perturbation; the amount
/// of substance must not move, and the molality must halve each time the
/// water doubles.
///
/// The pairing is the whole test. A quantity asserted only to be constant
/// is passed by a constant, and a quantity asserted only to move is passed
/// by anything that moves — but the two together pin down which of them the
/// engine thinks depends on the beaker. Measured over a 100 → 800 mL sweep:
/// sodium 0.0999948667932679 mol at every volume, agreeing to 1.3e-14 —
/// not bit-identical, because the solve is re-run in a different beaker,
/// but four orders of magnitude tighter than the 1e-12 asserted here;
/// molality 1.0030 → 0.5015 → 0.2507 → 0.1254.
///
/// **What this establishes:** amount of substance is stored, not recomputed
/// from a concentration and a volume. The classic bug this catches is the
/// inverse: `n = c · V` recovered on readback, which makes the count follow
/// the water and quietly agrees with the misconception the codex entry is
/// written to correct.
///
/// **What it cannot establish:** that 0.09999487 mol is the right answer
/// for 5.844 g of NaCl. It says the number does not depend on the water,
/// not that it is correct — and the 5e-5 shortfall against a textbook
/// 0.1 mol is a molar-mass question this test deliberately does not ask.
///
/// **Weakness: low, but it is the most likely of these to be already
/// covered.** `metamorphic.rs` asserts scale invariance of pH and ionic
/// strength, which is the same family; this differs in perturbing ONLY the
/// solvent, so the extensive quantity is held fixed rather than doubled
/// alongside it, and it is the amount rather than an intensive property
/// that is pinned.
#[test]
fn water_changes_the_concentration_and_not_the_count() {
    let mut amounts = Vec::new();
    let mut molalities = Vec::new();
    for millilitres in [100, 200, 400, 800] {
        let after = vessel(
            &run(&format!(
                "add v1 water {millilitres}mL\nadd v1 NaCl 5.844g\n"
            )),
            0,
        );
        amounts.push(moles_of(&after, "Na+"));
        molalities.push(molality_of(&after, "Na+"));
    }

    for amount in &amounts {
        assert!(
            (amount - amounts[0]).abs() < 1e-12,
            "the count you weighed is not negotiable by water: {amounts:?}"
        );
    }
    assert!(
        amounts[0] > 0.09 && amounts[0] < 0.11,
        "and it has to be the amount actually weighed out, not zero: {}",
        amounts[0]
    );
    // Eight times the water, an eighth of the molality. 2% covers the
    // activity and density corrections; it is a shape check, not a
    // measurement of either.
    for (index, molality) in molalities.iter().enumerate() {
        let expected = molalities[0] / 2f64.powi(index as i32);
        assert!(
            (molality - expected).abs() < 0.02 * expected,
            "doubling the water must halve the molality: {molalities:?}"
        );
    }
}

// ===================================================================== 7/7
// Mechanism: a quantity that carries a species' NAME must behave like that
// species.

/// **A slot called `OH-` that did not move like hydroxide.** Found by this
/// suite, on 2026-09-16, while writing case 2 — which is why it is here
/// rather than in a report. It was `#[ignore]`d for one day, asserting the
/// behaviour that was wanted rather than the behaviour that shipped; the
/// name changed on 2026-09-16 and the assertions below are now the ones
/// that hold.
///
/// **What was found.** The vessel's `contents` — the lv3 machine contract,
/// and what every `--json` client reads — carried an entry whose `species`
/// was `OH-`. In a strong base it was hydroxide. In an acetate buffer it
/// was not: it was the solution's residual cation charge wearing
/// hydroxide's name.
///
/// Measured, across an acid sweep of an equimolar acetate buffer:
///
/// ```text
/// acid    pH      contents OH-   free_hydroxide   10^(pH-14)·kgw
/// 0.000   4.6630   5.174e-04       5.761e-10        4.589e-10
/// 0.020   4.2955   3.972e-04       2.472e-10        1.969e-10
/// ```
///
/// `free_hydroxide` — the field the aqueous solver writes on its way out —
/// is right, and moves by 2.33x for the 0.3675 pH units it fell, which is
/// 10^0.3675 to three figures. The entry named `OH-` on the wire moved by
/// 1.30x, and sat 1.1e6 above the hydroxide a pH of 4.66 can hold.
///
/// It was not a buffer-only artefact. The same slot read 7x high for
/// bicarbonate (the carbonate alkalinity booked as hydroxide), 1.28x for
/// sodium acetate, 1.12x for sodium hydroxide — the ratio was 1 only when
/// the solution's charge really is carried by free base. **The engine
/// already knew this rule and stated it**, in the doc comment on
/// `Vessel::free_hydroxide`: "Net charge is free base only in a vessel of
/// strong electrolytes: a bicarbonate solution carries its charge excess as
/// carbonate alkalinity... Reading either as hydroxide invents a
/// neutralisation that never happened, at 55.81 kJ for every mole of it."
/// That is the rule the slot broke, under hydroxide's own name.
///
/// This is the defect shape the suite was built for, and it is the same one
/// as the alkalinity defect in case 1 seen from the other side: there, a
/// charge was mistaken for a portion; here, a charge was published as a
/// species. It is exactly what a single-run check cannot see — 5.17e-4 mol
/// is a perfectly plausible number — and what one perturbation makes
/// obvious, because a quantity that carries a name must move the way the
/// thing it is named after moves.
///
/// **What shipped.** The carrier was renamed, not re-split: it is
/// `base_equivalents` on the wire, and `species::BASE_EQUIVALENTS` in the
/// engine. Nothing about the arithmetic changed — it is still the base half
/// of the analytical acid/base equivalents that close a solved vessel's H/O
/// balance, still `2·O − H` over everything else booked, still the residual
/// cation charge. What changed is that it no longer claims to be a
/// measurement of hydroxide. `OH-` is left free to mean hydroxide, which it
/// genuinely does elsewhere in `contents` — the chloralkali cell deposits
/// the caustic soda it is for under exactly that key — and the measured
/// hydroxide is where it always was, in `free_hydroxide` and in the
/// reported speciation.
///
/// The acid half is still published as `H+` and has the same shape of
/// defect: it is titratable acid, not free protons, and a beaker of vinegar
/// holds two hundred times fewer of the second than the first. It is
/// recorded here and not fixed here; `kerotakis-safety` already declines to
/// read it as a strong-acid bottle, which is what makes it the less urgent
/// half.
///
/// **What this establishes, in two claims that are not the same claim.**
///
/// * **A name means the thing.** Across the same acid sweep of the same
///   buffer, no `contents` entry named `OH-` holds more hydroxide than the
///   pH can hold. It is the assertion that failed by six orders of
///   magnitude the day before this shipped, and it is checked at both pHs
///   rather than at one, so a slot that merely happened to coincide at one
///   of them would not pass. It passes today because nothing is published
///   under that name in a buffer — and that is the point: the key is now
///   free for hydroxide to use, and anything that puts a number there will
///   be held to it.
/// * **The renamed key answers to charge.** Take a strong base and remove
///   its charge imbalance a known amount at a time: `base_equivalents`
///   falls one mole per mole of strong acid, over a fourfold range of acid.
///   That is the definition it now carries, tested as a definition — a
///   constant offset fails it, and so does a proportionality with the wrong
///   constant.
///
/// **What it cannot establish:** that the pH is right. The first claim
/// compares two of the engine's own surfaces against each other; if both
/// were wrong together it would still pass. That is the price of needing no
/// reference value, and it is the right price here, because the failure was
/// a disagreement rather than an error. Nor does it establish that
/// `base_equivalents` is the right quantity for the tail to book — only
/// that it is honestly named and moves the way its name says.
#[test]
fn a_slot_named_hydroxide_moves_like_hydroxide() {
    let buffer = |acid: f64| {
        let mut script =
            String::from("add v1 water 1000mL\nadd v1 CH3COOH 0.05mol\nadd v1 NaOAc 0.05mol\n");
        if acid > 0.0 {
            script.push_str(&format!("add v1 HCl {acid}mol\n"));
        }
        vessel(&run(&script), 0)
    };
    // What a pH of this value can hold, in moles, over the solver's own
    // solvent mass. No reference value: the engine supplies both sides.
    let implied_hydroxide = |v: &serde_json::Value| {
        10f64.powf(ph(v) - 14.0) * v["solution"]["solvent_kg"].as_f64().unwrap()
    };

    let (before, after) = (buffer(0.0), buffer(0.02));
    for v in [&before, &after] {
        let implied = implied_hydroxide(v);
        for (surface, what) in [
            (moles_of(v, "OH-"), "contents[OH-]"),
            (v["free_hydroxide"].as_f64().unwrap(), "free_hydroxide"),
        ] {
            assert!(
                surface < 10.0 * implied,
                "{what} claims {surface} mol of hydroxide at pH {:.4}, which can \
                 hold {implied} mol",
                ph(v)
            );
        }
    }
    // The carrier did not vanish with its old name: the buffer still needs
    // it, and it is still the number that used to be published as `OH-`.
    // Without this the claim above could be satisfied by dropping the slot.
    assert!(
        moles_of(&before, BASE_EQUIVALENTS) > 100.0 * implied_hydroxide(&before),
        "the analytical basis carrier is still published, and is still not \
         hydroxide: {} mol against {} mol of hydroxide at pH {:.4}",
        moles_of(&before, BASE_EQUIVALENTS),
        implied_hydroxide(&before),
        ph(&before)
    );

    // And it must move like what it is now called: a charge coordinate.
    // A mole of strong acid is a mole of anion the cations no longer
    // outnumber, so the residual falls by one mole for each.
    let lye = |acid: f64| {
        let mut script = String::from("add v1 water 1000mL\nadd v1 NaOH 0.05mol\n");
        if acid > 0.0 {
            script.push_str(&format!("add v1 HCl {acid}mol\n"));
        }
        vessel(&run(&script), 0)
    };
    let base0 = moles_of(&lye(0.0), BASE_EQUIVALENTS);
    // A sanity bound, not the claim: 0.05 mol of a fully dissociated strong
    // base leaves its sodium with nothing but the solvent to balance it, so
    // the residual is that sodium and nothing else. Loose because the
    // readback is free to book some of that sodium in another form; the
    // claim below survives it either way.
    assert!(
        (base0 - 0.05).abs() < 0.2 * 0.05,
        "0.05 mol of sodium hydroxide leaves about 0.05 equivalents of \
         residual cation charge, not {base0}"
    );
    // This is the claim. Every mole of strong acid brings a mole of
    // chloride, which carries neither hydrogen nor oxygen into the basis,
    // so the residual falls by exactly one mole for each — over a fourfold
    // range, which is what rules out a constant offset and a wrong constant.
    for acid in [0.005, 0.010, 0.020] {
        let fell = base0 - moles_of(&lye(acid), BASE_EQUIVALENTS);
        assert!(
            (fell - acid).abs() < 1e-5 + 0.02 * acid,
            "{acid} mol of strong acid must take {acid} equivalents of \
             residual cation charge, not {fell}"
        );
    }
}
