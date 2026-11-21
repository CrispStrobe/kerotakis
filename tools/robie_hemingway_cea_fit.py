"""Regenerate the NASA-9 records in data/cea/robie-hemingway-1995-supplement.inp.

Source: Robie & Hemingway (1995), USGS Bulletin 2131 (public domain). The
Cp(T) polynomial Cp = A1 + A2 T + A3 T^-2 + A4 T^-0.5 + A5 T^2, S(298.15) and
dfH(298.15) below are transcribed from its coefficient and summary tables; the
printed rows in TABLE are used only to check the fit. The T^-0.5 term has no
NASA-9 counterpart, so the seven Cp coefficients are a least-squares fit to
Cp/R over the source's own range; b1 and b2 then make H(298.15) = dfH and
S(298.15) = S exactly.

    python3 tools/robie_hemingway_cea_fit.py           # rewrite the records
    python3 tools/robie_hemingway_cea_fit.py --check   # exit 1 if stale
"""
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "data/cea/robie-hemingway-1995-supplement.inp"
R = 8.31446261815324  # kerotakis_cea::nasa9::R
T0 = 298.15
SOURCES = {
    "ZnO": dict(A=(43.50, 7.658e-3, -7.573e5, 54.56, 0.0), S298=43.20, H=-350.5e3, Tmax=1800.0,
                M=81.389, comp=[("ZN", 1), ("O", 1)],
                table=[(298.15, 40.42, 43.20, 0.0), (400, 44.56, 55.73, 10.89),
                       (600, 48.22, 74.59, 22.80), (1000, 52.13, 100.20, 33.80),
                       (1400, 55.29, 118.25, 39.50), (1800, 58.34, 132.52, 43.35)]),
    "ZnCO3": dict(A=(148.38, 2.835e-2, 4.796e5, -1.419e3, 0.0), S298=81.19, H=-817.0e3, Tmax=1200.0,
                  M=125.399, comp=[("ZN", 1), ("C", 1), ("O", 3)],
                  table=[(298.15, 80.05, 81.19, 0.0), (400, 91.77, 106.42, 21.93),
                         (600, 108.79, 147.05, 48.21), (800, 121.64, 180.17, 65.02),
                         (1000, 132.34, 208.49, 77.44), (1200, 141.77, 233.47, 87.39)]),
}


def source_cp(a, t):
    return a[0] + a[1] * t + a[2] * t**-2 + a[3] * t**-0.5 + a[4] * t**2


def basis(t):
    t = np.atleast_1d(np.asarray(t, float))
    return np.array([t**-2, t**-1, np.ones_like(t), t, t**2, t**3, t**4]).T


def record(name, d):
    t = np.linspace(T0, d["Tmax"], 600)
    b = basis(t)
    scale = np.abs(b).max(axis=0)
    a, *_ = np.linalg.lstsq(b / scale, source_cp(d["A"], t) / R, rcond=None)
    a = a / scale
    h_nob = lambda t: (-a[0] * t**-2 + a[1] * np.log(t) / t + a[2] + a[3] * t / 2
                       + a[4] * t**2 / 3 + a[5] * t**3 / 4 + a[6] * t**4 / 5)
    s_nob = lambda t: (-a[0] * t**-2 / 2 - a[1] / t + a[2] * np.log(t) + a[3] * t
                       + a[4] * t**2 / 2 + a[5] * t**3 / 3 + a[6] * t**4 / 4)
    b1 = (d["H"] / (R * T0) - h_nob(T0)) * T0
    b2 = d["S298"] / R - s_nob(T0)
    h = lambda t: (h_nob(t) + b1 / t) * R * t
    s = lambda t: (s_nob(t) + b2) * R
    for t_, cp, entropy, h_over_t in d["table"]:
        fitted = float((basis(t_) @ a)[0] * R)
        assert abs(fitted - cp) < 0.01, (name, t_, fitted, cp)
        assert abs(float(s(t_)) - entropy) < 0.01, (name, t_)
        assert abs(float((h(t_) - h(T0)) / t_) - h_over_t) < 0.01, (name, t_)
    sym = "".join(f"{e:<2}{c:6.2f}" for e, c in d["comp"])
    sym += "".join(f"{'':2}{0:6.2f}" for _ in range(5 - len(d["comp"])))
    f = lambda v: f"{v:16.9E}".replace("E", "D")
    return (f"{name + '(cr)':<18}Robie & Hemingway 1995 USGS Bull 2131 Cp(T),S,dfH; NASA-9 fit (Kerotakis)\n"
            f" 1 rh1995 {sym} 1{d['M']:13.7f}{d['H']:15.3f}\n"
            f"{T0:11.3f}{d['Tmax']:11.4f}7 -2.0 -1.0  0.0  1.0  2.0  3.0  4.0  0.0{0.0:17.3f}\n"
            + "".join(f(v) for v in a[:5]) + "\n"
            + f(a[5]) + f(a[6]) + " " * 16 + f(b1) + f(b2) + "\n")


def render():
    text = TARGET.read_text(encoding="utf-8")
    head, _ = text.split("    200.00   1000.00   6000.00  20000.   9/09/04\n", 1)
    body = "".join(record(n, d) for n, d in SOURCES.items())
    return head + "    200.00   1000.00   6000.00  20000.   9/09/04\n" + body + "END PRODUCTS\nEND REACTANTS\n"


if __name__ == "__main__":
    fresh = render()
    if sys.argv[1:] == ["--check"]:
        sys.exit(TARGET.read_text(encoding="utf-8") != fresh)
    TARGET.write_text(fresh, encoding="utf-8")
