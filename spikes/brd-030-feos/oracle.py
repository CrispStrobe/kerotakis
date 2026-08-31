#!/usr/bin/env python3
"""BRD-030 — the independent referee.

Extends the CAP-19 pattern in tools/gen-thermo-fixtures.py to the whole
BRD-030 corpus. The Python `thermo` package (MIT, Caleb Bell) is a third
implementation that shares no code with either kerotakis-thermo or feos, so
when kerotakis and feos disagree it can say which of them moved.

It answers in three voices, and the distinction is the whole point:

  oracle-corr         thermo's own correlations for pure-fluid properties.
                      Independent of both engines. This is the referee for
                      "is feos's PC-SAFT number physically right".

  oracle-unifac-kero  thermo's UNIFAC gammas driven with *kerotakis-thermo's
                      own* Antoine constants (duplicated below on purpose,
                      exactly as tools/gen-thermo-fixtures.py does). A gap
                      against `kerotakis` here is a gamma-implementation
                      difference and nothing else, because the vapour
                      pressures are identical by construction.

  oracle-unifac-corr  thermo's UNIFAC gammas with thermo's own vapour
                      pressures. A full independent answer, available for
                      every binary in the corpus including the ones
                      kerotakis-thermo cannot express at all.

Emits the same six columns as the Rust side:
    engine  case  quantity  status  value  note

Run:  python3 oracle.py > fixtures/oracle.tsv
"""

import json
import math
import os
import sys

from thermo.unifac import UFIP, UFSG, UNIFAC
import thermo
import chemicals

HERE = os.path.dirname(os.path.abspath(__file__))
ATM_KPA = 101.325

# Antoine constants duplicated from crates/kerotakis-thermo/src/vle.rs.
# log10(P/kPa) = a - b/(T_C + c). Duplicated rather than imported so the
# referee solves the *stated* model independently; if the two copies drift
# apart, the fixtures stop matching and say so.
KERO_ANTOINE = {
    "WATER": (7.19621, 1730.63, 233.426, 1.0, 100.0),
    "ETHANOL": (7.32907, 1642.89, 230.300, -57.0, 80.0),
    "ISOPROPANOL": (6.861, 1357.427, 197.336, 56.77, 89.26),
    "METHANOL": (7.20607, 1582.271, 239.726, 15.0, 84.0),
    "PROPANONE": (6.14957, 1161.0, 224.0, -20.0, 77.0),
    "ETHANOIC_ACID": (6.51292, 1533.313, 222.309, 17.0, 157.0),
}


def row(engine, case, quantity, value, note):
    if value is None or (isinstance(value, float) and not math.isfinite(value)):
        print(f"{engine}\t{case}\t{quantity}\tunavailable\t\t{note}")
    else:
        print(f"{engine}\t{case}\t{quantity}\tok\t{value:.10e}\t{note}")


def kero_psat(name, t_c):
    a, b, c, _, _ = KERO_ANTOINE[name]
    return 10.0 ** (a - b / (t_c + c))


def gammas(groups, xs, t_k):
    """thermo's UNIFAC, original published (Fredenslund) parameters."""
    ge = UNIFAC.from_subgroups(
        T=t_k,
        xs=xs,
        chemgroups=[{int(k): int(v) for k, v in g.items()} for g in groups],
        interaction_data=UFIP,
        subgroups=UFSG,
    )
    return ge.gammas()


def bisect_bubble(psat_fns, groups, xs, p_kpa, lo=-150.0, hi=400.0):
    """Bubble temperature of sum(x_i gamma_i(T) Psat_i(T)) = P."""

    def total(t_c):
        g = gammas(groups, xs, t_c + 273.15)
        return sum(x * gi * f(t_c) for x, gi, f in zip(xs, g, psat_fns)) - p_kpa

    try:
        if total(lo) > 0.0 or total(hi) < 0.0:
            return None, None
    except (ValueError, OverflowError, ZeroDivisionError):
        return None, None
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if total(mid) < 0.0:
            lo = mid
        else:
            hi = mid
    t = 0.5 * (lo + hi)
    g = gammas(groups, xs, t + 273.15)
    parts = [x * gi * f(t) for x, gi, f in zip(xs, g, psat_fns)]
    return t, parts[0] / sum(parts)


def main():
    corpus = json.load(open(os.path.join(HERE, "corpus.json")))
    print("engine\tcase\tquantity\tstatus\tvalue\tnote")

    by_key = {p["key"]: p for p in corpus["pures"]}

    # ── pure fluids ──────────────────────────────────────────────────
    for fl in corpus["pures"]:
        key, cas = fl["key"], fl["cas"]
        try:
            vp = thermo.VaporPressure(CASRN=cas)
            vl = thermo.VolumeLiquid(CASRN=cas)
            hv = thermo.EnthalpyVaporization(CASRN=cas)
        except Exception as exc:  # pragma: no cover - referee availability
            for t_c in fl["temps_c"]:
                row("oracle-corr", f"{key}@{t_c}C", "psat_kpa", None, f"thermo: {exc}")
            continue

        for t_c in fl["temps_c"]:
            case = f"{key}@{t_c}C"
            t_k = t_c + 273.15
            p = vp.T_dependent_property(t_k)
            row(
                "oracle-corr", case, "psat_kpa",
                None if p is None else p / 1000.0,
                f"thermo.VaporPressure method={vp.method}",
            )
            v = vl.T_dependent_property(t_k)
            row(
                "oracle-corr", case, "rho_liq_mol_m3",
                None if not v else 1.0 / v,
                f"thermo.VolumeLiquid method={vl.method}",
            )
            h = hv.T_dependent_property(t_k)
            row(
                "oracle-corr", case, "dhvap_kj_mol",
                None if h is None else h / 1000.0,
                f"thermo.EnthalpyVaporization method={hv.method}",
            )

        tb = chemicals.Tb(cas)
        row("oracle-corr", key, "tboil_c", None if tb is None else tb - 273.15,
            "chemicals.Tb (experimental normal boiling point)")
        tc = chemicals.Tc(cas)
        row("oracle-corr", key, "tcrit_k", tc, "chemicals.Tc (experimental)")
        pc = chemicals.Pc(cas)
        row("oracle-corr", key, "pcrit_kpa", None if pc is None else pc / 1000.0,
            "chemicals.Pc (experimental)")

    # ── binaries ─────────────────────────────────────────────────────
    xs_grid = corpus["binary_x1"]
    for b in corpus["binaries"]:
        key = b["key"]
        fa, fb = by_key[b["a"]], by_key[b["b"]]
        ga, gb = fa.get("groups"), fb.get("groups")

        vpa = thermo.VaporPressure(CASRN=fa["cas"])
        vpb = thermo.VaporPressure(CASRN=fb["cas"])

        for x1 in xs_grid:
            case = f"{key}@x1={x1}"
            xs = [x1, 1.0 - x1]

            # voice 3: fully independent
            if ga and gb:
                try:
                    t, y1 = bisect_bubble(
                        [lambda t_c: vpa.T_dependent_property(t_c + 273.15) / 1000.0,
                         lambda t_c: vpb.T_dependent_property(t_c + 273.15) / 1000.0],
                        [ga, gb], xs, ATM_KPA,
                    )
                except Exception as exc:
                    t, y1 = None, None
                    note = f"UNIFAC: {exc}"
                else:
                    note = (f"thermo UNIFAC + thermo Psat "
                            f"({vpa.method}/{vpb.method})")
                row("oracle-unifac-corr", case, "tbub_c", t, note)
                row("oracle-unifac-corr", case, "y1", y1, note)
            else:
                why = "no UNIFAC group decomposition (supercritical/inorganic)"
                row("oracle-unifac-corr", case, "tbub_c", None, why)
                row("oracle-unifac-corr", case, "y1", None, why)

            # voice 2: kerotakis's own Antoine, thermo's gammas
            ka, kb = fa.get("kerotakis"), fb.get("kerotakis")
            if ka and kb and ga and gb:
                try:
                    t, y1 = bisect_bubble(
                        [lambda t_c, n=ka: kero_psat(n, t_c),
                         lambda t_c, n=kb: kero_psat(n, t_c)],
                        [ga, gb], xs, ATM_KPA,
                    )
                    note = "thermo UNIFAC + kerotakis Antoine (isolates gamma)"
                except Exception as exc:
                    t, y1, note = None, None, f"UNIFAC: {exc}"
                row("oracle-unifac-kero", case, "tbub_c", t, note)
                row("oracle-unifac-kero", case, "y1", y1, note)
            else:
                why = "kerotakis-thermo has no Antoine set for one component"
                row("oracle-unifac-kero", case, "tbub_c", None, why)
                row("oracle-unifac-kero", case, "y1", None, why)

    # ── Peng-Robinson referee ────────────────────────────────────────
    # Same Tc/Pc/omega as both implementations, so this is a third
    # independent Peng-Robinson rather than a different model.
    pr = corpus["peng_robinson"]
    for fl in pr["fluids"]:
        key = fl["key"]
        for st in pr["states"]:
            t, p = st["t_k"], st["p_pa"]
            case = f"{key}@{t}K,{p}Pa"
            try:
                eos = thermo.eos.PR(Tc=fl["tc_k"], Pc=fl["pc_pa"],
                                    omega=fl["omega"], T=t, P=p)
                v = eos.V_g if getattr(eos, "V_g", None) is not None else eos.V_l
                z = p * v / (8.314462618 * t)
                phi = eos.phi_g if getattr(eos, "phi_g", None) is not None else eos.phi_l
            except Exception as exc:
                row("oracle-pr", case, "z_vapour", None, f"thermo.eos.PR: {exc}")
                continue
            row("oracle-pr", case, "z_vapour", z, "thermo.eos.PR")
            row("oracle-pr", case, "molar_volume_m3_mol", v, "thermo.eos.PR")
            row("oracle-pr", case, "phi", phi, "thermo.eos.PR")

    print(f"# thermo {thermo.__version__}, chemicals {chemicals.__version__}",
          file=sys.stderr)


if __name__ == "__main__":
    main()
