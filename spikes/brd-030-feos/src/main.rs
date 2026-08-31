//! BRD-030 — direct feos integration spike.
//!
//! Emits one TSV row per (engine, case, quantity) so the three engines —
//! `kerotakis-thermo`, `feos`, and the Python `thermo` referee — can be
//! joined on the same keys and compared point for point. A row is emitted
//! even when an engine cannot answer: `status=unavailable` with a reason is
//! a *result*, and averaging it away would hide the single most important
//! thing this spike measured.
//!
//! Columns: `engine  case  quantity  status  value  note`
//!
//! Usage:
//!   brd030-feos-spike emit    > fixtures/engines.tsv
//!   brd030-feos-spike bench
//!
//! Run from the spike directory, after `./fetch-parameters.sh`.

use brd030_feos_spike::adapter;

use feos::pcsaft::{PcSaft, PcSaftParameters};
use feos_core::cubic::{PengRobinson, PengRobinsonParameters};
use feos_core::parameter::IdentifierOption;
use feos_core::{
    Contributions, DensityInitialization, FeosResult, PhaseEquilibrium, SolverOptions, State,
};
use kerotakis_thermo::eos::{CriticalProperties, PengRobinson as KeroPr};
use kerotakis_thermo::fluid::FluidModel;
use kerotakis_thermo::unifac;
use kerotakis_thermo::vle::{self, Antoine, Volatile};
use nalgebra::DVector;
use quantity::{JOULE, KELVIN, METER, MOL, PASCAL};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

const PURE_JSON: &str = "parameters/esper2023.json";
const BINARY_JSON: &str = "parameters/rehner2023_binary.json";
/// The alternative parameter set, used on two fluids only, to separate a
/// *parameter* difference from a *model* difference in the discrepancy table.
const GROSS2002_JSON: &str = "parameters/gross2002.json";

// ── output ────────────────────────────────────────────────────────────────

fn ok(engine: &str, case: &str, quantity: &str, value: f64, note: &str) {
    println!("{engine}\t{case}\t{quantity}\tok\t{value:.10e}\t{note}");
}

fn unavailable(engine: &str, case: &str, quantity: &str, why: &str) {
    println!("{engine}\t{case}\t{quantity}\tunavailable\t\t{why}");
}

// ── corpus ────────────────────────────────────────────────────────────────

fn corpus() -> Value {
    let raw = std::fs::read_to_string("corpus.json").expect("corpus.json next to Cargo.toml");
    serde_json::from_str(&raw).expect("corpus.json parses")
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().to_string()
}

fn f(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap_or_default()
}

/// The six fluids `kerotakis-thermo` curates Antoine constants for. Anything
/// else is `None`, and that `None` is the coverage gap this spike measures.
fn kero_antoine(name: &str) -> Option<Antoine> {
    Some(match name {
        "WATER" => vle::WATER,
        "ETHANOL" => vle::ETHANOL,
        "ISOPROPANOL" => vle::ISOPROPANOL,
        "METHANOL" => vle::METHANOL,
        "PROPANONE" => vle::PROPANONE,
        "ETHANOIC_ACID" => vle::ETHANOIC_ACID,
        _ => return None,
    })
}

fn groups(v: &Value) -> Option<unifac::GroupDecomposition> {
    let obj = v.as_object()?;
    let mut g = unifac::GroupDecomposition::new();
    for (k, n) in obj {
        g.insert(k.parse().ok()?, n.as_u64()? as u32);
    }
    Some(g)
}

// ── feos construction ─────────────────────────────────────────────────────

fn pcsaft(names: &[&str], binary: bool, pure_file: &str) -> FeosResult<Arc<PcSaft>> {
    let params = PcSaftParameters::from_json(
        names.to_vec(),
        pure_file,
        if binary { Some(BINARY_JSON) } else { None },
        IdentifierOption::Name,
    )?;
    Ok(Arc::new(PcSaft::new(params)))
}

/// Bubble point with an initial-temperature ladder. feos's solver takes an
/// optional starting temperature and does not always find one unaided; the
/// ladder is the spike's, not feos's, and the report says how often it was
/// needed.
fn feos_bubble(eos: &Arc<PcSaft>, x: &DVector<f64>, p_kpa: f64) -> Option<(f64, Vec<f64>, usize)> {
    let p = p_kpa * 1000.0 * PASCAL;
    let opts = (SolverOptions::default(), SolverOptions::default());
    let mut tries = 0;
    let mut vle = PhaseEquilibrium::bubble_point(eos, p, x, None, None, opts).ok();
    for t0 in [350.0, 300.0, 400.0, 250.0, 450.0, 200.0, 100.0, 500.0] {
        if vle.is_some() {
            break;
        }
        tries += 1;
        vle = PhaseEquilibrium::bubble_point(eos, p, x, Some(t0 * KELVIN), None, opts).ok();
    }
    let vle = vle?;
    Some((
        vle.vapor().temperature.convert_into(KELVIN) - 273.15,
        vle.vapor().molefracs.iter().copied().collect(),
        tries,
    ))
}

// ── pure fluids ───────────────────────────────────────────────────────────

fn pures(c: &Value) {
    let p_atm = f(c, "pressure_kpa");
    for fl in c["pures"].as_array().unwrap() {
        let key = s(fl, "key");
        let feos_name = s(fl, "feos");
        let kero_name = fl["kerotakis"].as_str();

        // ---- kerotakis-thermo: Antoine, inside its stated range only ----
        match kero_name.and_then(kero_antoine) {
            None => {
                for t in fl["temps_c"].as_array().unwrap() {
                    let case = format!("{key}@{}C", t.as_f64().unwrap());
                    unavailable(
                        "kerotakis",
                        &case,
                        "psat_kpa",
                        "no Antoine constants curated in kerotakis-thermo",
                    );
                    unavailable(
                        "kerotakis",
                        &case,
                        "rho_liq_mol_m3",
                        "kerotakis-thermo has no liquid-density model",
                    );
                    unavailable(
                        "kerotakis",
                        &case,
                        "dhvap_kj_mol",
                        "kerotakis-thermo has no enthalpy model for pure fluids",
                    );
                }
                unavailable(
                    "kerotakis",
                    &key,
                    "tboil_c",
                    "no Antoine constants curated in kerotakis-thermo",
                );
                for q in ["tcrit_k", "pcrit_kpa"] {
                    unavailable(
                        "kerotakis",
                        &key,
                        q,
                        "kerotakis-thermo curates critical properties for 3 fluids only (eos.rs)",
                    );
                }
            }
            Some(a) => {
                for t in fl["temps_c"].as_array().unwrap() {
                    let tc = t.as_f64().unwrap();
                    let case = format!("{key}@{tc}C");
                    match a.pressure_kpa(tc) {
                        Some(p) => ok("kerotakis", &case, "psat_kpa", p, a.source),
                        None => unavailable(
                            "kerotakis",
                            &case,
                            "psat_kpa",
                            &format!(
                                "outside fitted Antoine range {:.1}..{:.1} C",
                                a.valid_c.0, a.valid_c.1
                            ),
                        ),
                    }
                    // No liquid density and no enthalpy of vaporisation
                    // exist in kerotakis-thermo at all: Antoine is a
                    // pressure correlation and nothing more.
                    unavailable(
                        "kerotakis",
                        &case,
                        "rho_liq_mol_m3",
                        "kerotakis-thermo has no liquid-density model",
                    );
                    unavailable(
                        "kerotakis",
                        &case,
                        "dhvap_kj_mol",
                        "kerotakis-thermo has no enthalpy model for pure fluids",
                    );
                }
                let mix = [Volatile {
                    antoine: a,
                    x: 1.0,
                    gamma: 1.0,
                }];
                match vle::bubble_point(&mix, p_atm) {
                    Some(bp) => ok("kerotakis", &key, "tboil_c", bp.t_celsius, a.source),
                    None => unavailable("kerotakis", &key, "tboil_c", "bisection did not bracket"),
                }
                for q in ["tcrit_k", "pcrit_kpa"] {
                    unavailable(
                        "kerotakis",
                        &key,
                        q,
                        "kerotakis-thermo curates critical properties for 3 fluids only (eos.rs)",
                    );
                }
            }
        }

        // ---- feos PC-SAFT with esper2023 ----
        let eos = match pcsaft(&[feos_name.as_str()], false, PURE_JSON) {
            Ok(e) => e,
            Err(e) => {
                unavailable("feos-pcsaft", &key, "all", &format!("parameters: {e}"));
                continue;
            }
        };
        for t in fl["temps_c"].as_array().unwrap() {
            let tc = t.as_f64().unwrap();
            let case = format!("{key}@{tc}C");
            emit_pure_at_t(&eos, &case, tc, "esper2023");
        }
        // Normal boiling point.
        match PhaseEquilibrium::pure(
            &eos,
            p_atm * 1000.0 * PASCAL,
            None,
            SolverOptions::default(),
        ) {
            Ok(v) => ok(
                "feos-pcsaft",
                &key,
                "tboil_c",
                v.vapor().temperature.convert_into(KELVIN) - 273.15,
                "esper2023",
            ),
            Err(e) => unavailable("feos-pcsaft", &key, "tboil_c", &e.to_string()),
        }
        // Critical point.
        //
        // The binding is annotated because `()` implements `Composition<D, N>`
        // for both `U1` and `Dyn` and for every `D: DualNum<f64>`, so nothing
        // in the call pins `State`'s `D`. Naming the concrete state — the
        // dynamic-size, plain-`f64` one — is the whole fix.
        let critical: FeosResult<State<Arc<PcSaft>>> =
            State::critical_point(&eos, (), None, None, SolverOptions::default());
        match critical {
            Ok(cp) => {
                ok(
                    "feos-pcsaft",
                    &key,
                    "tcrit_k",
                    cp.temperature.convert_into(KELVIN),
                    "esper2023",
                );
                ok(
                    "feos-pcsaft",
                    &key,
                    "pcrit_kpa",
                    cp.pressure(Contributions::Total).convert_into(PASCAL) / 1000.0,
                    "esper2023",
                );
            }
            Err(e) => unavailable("feos-pcsaft", &key, "tcrit_k", &e.to_string()),
        }

        // ---- feos PC-SAFT with the ORIGINAL gross2002 set, where it has
        // the fluid. Same code, same model family, different published
        // parameters: this row is what separates "feos disagrees" from
        // "the parameter set disagrees". ----
        if let Ok(e2) = pcsaft(&[feos_name.as_str()], false, GROSS2002_JSON) {
            for t in fl["temps_c"].as_array().unwrap() {
                let tc = t.as_f64().unwrap();
                let case = format!("{key}@{tc}C");
                emit_pure_at_t_named(&e2, "feos-pcsaft-gross2002", &case, tc, "gross2002");
            }
        }
    }
}

fn emit_pure_at_t(eos: &Arc<PcSaft>, case: &str, t_celsius: f64, note: &str) {
    emit_pure_at_t_named(eos, "feos-pcsaft", case, t_celsius, note)
}

fn emit_pure_at_t_named(eos: &Arc<PcSaft>, engine: &str, case: &str, t_celsius: f64, note: &str) {
    let t = (t_celsius + 273.15) * KELVIN;
    match PhaseEquilibrium::pure(eos, t, None, SolverOptions::default()) {
        Ok(v) => {
            ok(
                engine,
                case,
                "psat_kpa",
                v.vapor()
                    .pressure(Contributions::Total)
                    .convert_into(PASCAL)
                    / 1000.0,
                note,
            );
            ok(
                engine,
                case,
                "rho_liq_mol_m3",
                v.liquid().density.convert_into(MOL / METER.powi::<3>()),
                note,
            );
            // The ideal-gas contribution is identical in both phases at the
            // same temperature, so the residual difference IS the enthalpy
            // of vaporisation — no ideal-gas heat-capacity model needed.
            let dh = v.vapor().residual_molar_enthalpy() - v.liquid().residual_molar_enthalpy();
            ok(
                engine,
                case,
                "dhvap_kj_mol",
                dh.convert_into(JOULE / MOL) / 1000.0,
                note,
            );
        }
        Err(e) => {
            for q in ["psat_kpa", "rho_liq_mol_m3", "dhvap_kj_mol"] {
                unavailable(engine, case, q, &e.to_string());
            }
        }
    }
}

// ── binaries ──────────────────────────────────────────────────────────────

fn binaries(c: &Value) {
    let p_atm = f(c, "pressure_kpa");
    let xs: Vec<f64> = c["binary_x1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let mut pure_by_key: BTreeMap<String, &Value> = BTreeMap::new();
    for fl in c["pures"].as_array().unwrap() {
        pure_by_key.insert(s(fl, "key"), fl);
    }
    let flash_t = f(&c["flash"], "t_celsius");
    let flash_p = f(&c["flash"], "p_kpa");
    let flash_z = f(&c["flash"], "z1");

    for b in c["binaries"].as_array().unwrap() {
        let key = s(b, "key");
        let (ka, kb) = (s(b, "a"), s(b, "b"));
        let fa = pure_by_key[&ka];
        let fb = pure_by_key[&kb];

        // ---- kerotakis-thermo: Antoine + UNIFAC, or a named refusal ----
        let kero = kero_binary(fa, fb);
        for &x1 in &xs {
            let case = format!("{key}@x1={x1}");
            match &kero {
                Err(why) => {
                    unavailable("kerotakis", &case, "tbub_c", why);
                    unavailable("kerotakis", &case, "y1", why);
                }
                Ok((aa, ab, ga, gb)) => {
                    let table = unifac::approved_table();
                    let res = vle::bubble_point_with(&[*aa, *ab], &[x1, 1.0 - x1], p_atm, |t_k| {
                        unifac::activity_coefficients(
                            &table,
                            &[(ga.clone(), x1), (gb.clone(), 1.0 - x1)],
                            t_k,
                        )
                    });
                    match res {
                        Some(bp) => {
                            ok("kerotakis", &case, "tbub_c", bp.t_celsius, "Antoine+UNIFAC");
                            ok("kerotakis", &case, "y1", bp.y[0], "Antoine+UNIFAC");
                        }
                        None => {
                            unavailable("kerotakis", &case, "tbub_c", "bisection did not bracket");
                            unavailable("kerotakis", &case, "y1", "bisection did not bracket");
                        }
                    }
                }
            }
        }
        // TP flash on the same binary.
        let fcase = format!("{key}@flash");
        match &kero {
            Err(why) => unavailable("kerotakis", &fcase, "vapour_fraction", why),
            Ok((aa, ab, ga, gb)) => {
                let table = unifac::approved_table();
                let res = vle::tp_flash_with(
                    &[*aa, *ab],
                    &[flash_z, 1.0 - flash_z],
                    flash_p,
                    flash_t,
                    &mut |x, t_k| {
                        unifac::activity_coefficients(
                            &table,
                            &[(ga.clone(), x[0]), (gb.clone(), x[1])],
                            t_k,
                        )
                    },
                );
                match res {
                    Some(fr) => ok(
                        "kerotakis",
                        &fcase,
                        "vapour_fraction",
                        fr.vapour_fraction,
                        "Antoine+UNIFAC",
                    ),
                    None => unavailable("kerotakis", &fcase, "vapour_fraction", "flash refused"),
                }
            }
        }

        // ---- feos PC-SAFT ----
        let na = s(fa, "feos");
        let nb = s(fb, "feos");
        let eos = match pcsaft(&[na.as_str(), nb.as_str()], true, PURE_JSON) {
            Ok(e) => e,
            Err(e) => {
                for &x1 in &xs {
                    let case = format!("{key}@x1={x1}");
                    unavailable("feos-pcsaft", &case, "tbub_c", &format!("parameters: {e}"));
                    unavailable("feos-pcsaft", &case, "y1", &format!("parameters: {e}"));
                }
                continue;
            }
        };
        for &x1 in &xs {
            let case = format!("{key}@x1={x1}");
            let x = DVector::from_vec(vec![x1, 1.0 - x1]);
            match feos_bubble(&eos, &x, p_atm) {
                Some((t, y, tries)) => {
                    let note = if tries == 0 {
                        "esper2023+rehner2023".to_string()
                    } else {
                        format!("esper2023+rehner2023; needed {tries} restart(s)")
                    };
                    ok("feos-pcsaft", &case, "tbub_c", t, &note);
                    ok("feos-pcsaft", &case, "y1", y[0], &note);
                }
                None => {
                    unavailable(
                        "feos-pcsaft",
                        &case,
                        "tbub_c",
                        "bubble point did not converge",
                    );
                    unavailable("feos-pcsaft", &case, "y1", "bubble point did not converge");
                }
            }
        }
        // TP flash.
        let z = DVector::from_vec(vec![flash_z, 1.0 - flash_z]);
        match PhaseEquilibrium::tp_flash(
            &eos,
            (flash_t + 273.15) * KELVIN,
            flash_p * 1000.0 * PASCAL,
            &z,
            None,
            SolverOptions::default(),
            None,
        ) {
            Ok(fr) => ok(
                "feos-pcsaft",
                &fcase,
                "vapour_fraction",
                fr.vapor_phase_fraction(),
                "esper2023+rehner2023",
            ),
            Err(e) => unavailable("feos-pcsaft", &fcase, "vapour_fraction", &e.to_string()),
        }

        // ---- the adapter, on the same case, proving the seam ----
        if let (Some(aa), Some(ab)) = (
            fa["kerotakis"].as_str().and_then(kero_antoine),
            fb["kerotakis"].as_str().and_then(kero_antoine),
        ) {
            let model = adapter::FeosPcSaftFluid::new(eos.clone(), vec![na.clone(), nb.clone()]);
            for &x1 in &xs {
                let case = format!("{key}@x1={x1}");
                let mix = [
                    Volatile {
                        antoine: aa,
                        x: x1,
                        gamma: 1.0,
                    },
                    Volatile {
                        antoine: ab,
                        x: 1.0 - x1,
                        gamma: 1.0,
                    },
                ];
                match model.bubble_point(&mix, p_atm) {
                    Some(bp) => ok("adapter", &case, "tbub_c", bp.t_celsius, model.name()),
                    None => unavailable("adapter", &case, "tbub_c", "adapter refused"),
                }
            }
        }
    }
}

type KeroBinary = (
    Antoine,
    Antoine,
    unifac::GroupDecomposition,
    unifac::GroupDecomposition,
);

/// Whether `kerotakis-thermo` can speak about a binary at all, and why not
/// when it cannot. Both halves matter: an Antoine set AND a UNIFAC group
/// decomposition every group of which is in the approved table.
fn kero_binary(fa: &Value, fb: &Value) -> Result<KeroBinary, String> {
    let aa = fa["kerotakis"]
        .as_str()
        .and_then(kero_antoine)
        .ok_or_else(|| format!("no Antoine constants for {}", s(fa, "key")))?;
    let ab = fb["kerotakis"]
        .as_str()
        .and_then(kero_antoine)
        .ok_or_else(|| format!("no Antoine constants for {}", s(fb, "key")))?;
    let ga = groups(&fa["groups"]).ok_or("no group decomposition")?;
    let gb = groups(&fb["groups"]).ok_or("no group decomposition")?;
    let table = unifac::approved_table();
    for (g, which) in [(&ga, fa), (&gb, fb)] {
        for id in g.keys() {
            if table.group(*id).is_none() {
                return Err(format!(
                    "UNIFAC group {id} of {} is not in the approved table",
                    s(which, "key")
                ));
            }
        }
    }
    Ok((aa, ab, ga, gb))
}

// ── Peng-Robinson, both implementations, identical parameters ─────────────

fn peng_robinson(c: &Value) {
    let pr = &c["peng_robinson"];
    for fl in pr["fluids"].as_array().unwrap() {
        let key = s(fl, "key");
        let (tc, pc, w, mw) = (f(fl, "tc_k"), f(fl, "pc_pa"), f(fl, "omega"), f(fl, "mw"));
        let kero = KeroPr::new(CriticalProperties {
            tc_k: tc,
            pc_pa: pc,
            omega: w,
        });
        let params = match PengRobinsonParameters::new_simple(&[tc], &[pc], &[w], &[mw]) {
            Ok(p) => p,
            Err(e) => {
                unavailable("feos-pr", &key, "all", &e.to_string());
                continue;
            }
        };
        let eos = Arc::new(PengRobinson::new(params));

        for st in pr["states"].as_array().unwrap() {
            let (t, p) = (f(st, "t_k"), f(st, "p_pa"));
            let case = format!("{key}@{t}K,{p}Pa");
            ok(
                "kerotakis-pr",
                &case,
                "z_vapour",
                kero.z_vapour(t, p),
                "THERMO-007",
            );
            ok(
                "kerotakis-pr",
                &case,
                "molar_volume_m3_mol",
                kero.molar_volume(t, p),
                "THERMO-007",
            );
            ok(
                "kerotakis-pr",
                &case,
                "phi",
                kero.fugacity_coefficient(t, p),
                "THERMO-007",
            );

            match State::new_npt(
                &eos,
                t * KELVIN,
                p * PASCAL,
                (),
                Some(DensityInitialization::Vapor),
            ) {
                Ok(state) => {
                    ok(
                        "feos-pr",
                        &case,
                        "z_vapour",
                        state.compressibility(Contributions::Total),
                        "feos_core::cubic",
                    );
                    ok(
                        "feos-pr",
                        &case,
                        "molar_volume_m3_mol",
                        state.molar_volume.convert_into(METER.powi::<3>() / MOL),
                        "feos_core::cubic",
                    );
                    ok(
                        "feos-pr",
                        &case,
                        "phi",
                        state.ln_phi()[0].exp(),
                        "feos_core::cubic",
                    );
                }
                Err(e) => {
                    for q in ["z_vapour", "molar_volume_m3_mol", "phi"] {
                        unavailable("feos-pr", &case, q, &e.to_string());
                    }
                }
            }
        }
    }
}

// ── timings ───────────────────────────────────────────────────────────────

fn bench(c: &Value) {
    let p_atm = f(c, "pressure_kpa");
    let t0 = Instant::now();
    let eos = pcsaft(&["ethanol", "water"], true, PURE_JSON).expect("ethanol-water parameters");
    eprintln!(
        "parameter load (esper2023 1842 records + rehner2023_binary 7848 records): {:?}",
        t0.elapsed()
    );

    let t0 = Instant::now();
    let pure = pcsaft(&["water"], false, PURE_JSON).expect("water parameters");
    eprintln!(
        "parameter load (esper2023 only, 1 component): {:?}",
        t0.elapsed()
    );

    let n = 100;
    let t0 = Instant::now();
    for i in 0..n {
        let t = (300.0 + i as f64 * 0.5) * KELVIN;
        let _ = PhaseEquilibrium::pure(&pure, t, None, SolverOptions::default());
    }
    eprintln!("feos PC-SAFT vapour pressure: {:?}/call", t0.elapsed() / n);

    let t0 = Instant::now();
    for i in 0..n {
        let x = DVector::from_vec(vec![0.1 + 0.008 * i as f64, 0.9 - 0.008 * i as f64]);
        let _ = feos_bubble(&eos, &x, p_atm);
    }
    eprintln!(
        "feos PC-SAFT binary bubble point: {:?}/call",
        t0.elapsed() / n
    );

    let t0 = Instant::now();
    for i in 0..n {
        let _ = vle::ethanol_water_bubble_point(0.1 + 0.008 * i as f64, p_atm);
    }
    eprintln!(
        "kerotakis Antoine+UNIFAC binary bubble point: {:?}/call",
        t0.elapsed() / n
    );

    let kero = KeroPr::new(CriticalProperties {
        tc_k: 647.096,
        pc_pa: 22_064_000.0,
        omega: 0.3443,
    });
    let t0 = Instant::now();
    for i in 0..n {
        let _ = kero.z_vapour(300.0 + i as f64, 101_325.0);
    }
    eprintln!("kerotakis Peng-Robinson Z: {:?}/call", t0.elapsed() / n);

    let params =
        PengRobinsonParameters::new_simple(&[647.096], &[22_064_000.0], &[0.3443], &[18.01528])
            .unwrap();
    let fpr = Arc::new(PengRobinson::new(params));
    let t0 = Instant::now();
    for i in 0..n {
        let _ = State::new_npt(
            &fpr,
            (300.0 + i as f64) * KELVIN,
            101_325.0 * PASCAL,
            (),
            Some(DensityInitialization::Vapor),
        )
        .map(|s| s.compressibility(Contributions::Total));
    }
    eprintln!("feos Peng-Robinson Z: {:?}/call", t0.elapsed() / n);
}

fn main() {
    let c = corpus();
    match std::env::args().nth(1).as_deref() {
        Some("bench") => bench(&c),
        _ => {
            println!("engine\tcase\tquantity\tstatus\tvalue\tnote");
            pures(&c);
            binaries(&c);
            peng_robinson(&c);
        }
    }
}
