#!/usr/bin/env python3
"""Differential oracle for named relations (CAP-5).

Compares `kero calc` output against ChemPy and independent Python
arithmetic. ChemPy uses R = 8.314472 (CODATA 2006); kerotakis uses the
CODATA 2018 exact value 8.314 462 618 153 24, so ~1e-6 relative
difference is inherent and expected. Tolerance is set accordingly.

    pip install chempy
    cargo build -p kerotakis-cli
    python3 tools/check-relations-vs-chempy.py

Exits non-zero if any case exceeds tolerance. Not part of the build —
see kinetics-oracle.py header for why.
"""

import json
import math
import subprocess
import sys

R_CODATA2018 = 8.314_462_618_153_24
F = 96_485.332_12
k_B = 1.380_649e-23
h_planck = 6.626_070_15e-34

KERO = None
failures = []


def find_kero():
    """Locate the kero binary."""
    import shutil
    for candidate in [
        "/tmp/kero-basic-target/debug/kero",
        "target/debug/kero",
    ]:
        if shutil.which(candidate) or __import__("os").path.isfile(candidate):
            return candidate
    # Fallback: try PATH
    found = shutil.which("kero")
    if found:
        return found
    return None


def kero_calc(relation, args_dict):
    """Run `kero calc` and return the JSON result."""
    cmd = [KERO, "calc", relation, "--json"]
    for k, v in args_dict.items():
        cmd.append(f"{k}={v}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"kero calc failed: {result.stderr}")
    return json.loads(result.stdout)


def check(label, kero_val, ref_val, tol, ref_source):
    """Compare kero against reference; print result."""
    if ref_val == 0.0:
        diff = abs(kero_val)
        rel = diff
    else:
        diff = abs(kero_val - ref_val)
        rel = diff / abs(ref_val)
    ok = rel < tol
    status = "OK" if ok else "FAIL"
    print(f"  [{status}] {label}: kero={kero_val:.10e} ref={ref_val:.10e} "
          f"rel_diff={rel:.2e} tol={tol:.0e} ({ref_source})")
    if not ok:
        failures.append(label)


def test_arrhenius():
    print("\n=== Arrhenius ===")
    cases = [
        {"A": 1e10, "Ea": 50_000, "T": 298.15, "b": 0.0},
        {"A": 1e13, "Ea": 75_000, "T": 350.0, "b": 0.0},
        {"A": 1e8,  "Ea": 30_000, "T": 500.0,  "b": 1.5},
    ]
    from chempy.kinetics.arrhenius import ArrheniusParam
    for c in cases:
        r = kero_calc("arrhenius", c)
        # Python independent calc (CODATA 2018)
        py = c["A"] * c["T"]**c["b"] * math.exp(-c["Ea"] / (R_CODATA2018 * c["T"]))
        check(f"arrhenius A={c['A']:.0e} Ea={c['Ea']} T={c['T']} vs python",
              r["value"], py, 1e-12, "Python/CODATA2018")
        # ChemPy (CODATA 2006, no T exponent support)
        if c["b"] == 0.0:
            cp = ArrheniusParam(c["A"], c["Ea"])(c["T"])
            check(f"arrhenius A={c['A']:.0e} Ea={c['Ea']} T={c['T']} vs chempy",
                  r["value"], cp, 1e-4, "ChemPy/CODATA2006")


def test_eyring():
    print("\n=== Eyring ===")
    cases = [
        {"dG": 50_000, "T": 298.15},
        {"dG": 65_000, "T": 310.0},
        {"dG": 80_000, "T": 400.0},
    ]
    from chempy.kinetics.eyring import EyringParam
    for c in cases:
        r = kero_calc("eyring", c)
        # Python independent calc
        py = (k_B * c["T"] / h_planck) * math.exp(-c["dG"] / (R_CODATA2018 * c["T"]))
        check(f"eyring dG={c['dG']} T={c['T']} vs python",
              r["value"], py, 1e-12, "Python/CODATA2018")
        # ChemPy (EyringParam takes dH, dS; dG = dH - T*dS, so dH=dG, dS=0)
        cp = EyringParam(c["dG"], 0.0)(c["T"])
        check(f"eyring dG={c['dG']} T={c['T']} vs chempy",
              r["value"], cp, 1e-4, "ChemPy/CODATA2006")


def test_nernst():
    print("\n=== Nernst ===")
    cases = [
        {"e0": 0.0, "n": 2, "a": 1.0, "T": 298.15},
        {"e0": 0.3419, "n": 2, "a": 0.01, "T": 298.15},
        {"e0": -0.763, "n": 2, "a": 0.001, "T": 310.0},
    ]
    for c in cases:
        r = kero_calc("nernst", c)
        slope = R_CODATA2018 * c["T"] * math.log(10) / F
        py = c["e0"] + slope / c["n"] * math.log10(c["a"])
        check(f"nernst e0={c['e0']} n={c['n']} a={c['a']} T={c['T']}",
              r["value"], py, 1e-12, "Python/CODATA2018")


def test_henderson_hasselbalch():
    print("\n=== Henderson–Hasselbalch ===")
    cases = [
        {"pKa": 4.76, "cA": 0.1, "cB": 0.1},
        {"pKa": 9.25, "cA": 0.05, "cB": 0.15},
        {"pKa": 7.2, "cA": 0.01, "cB": 0.1},
    ]
    for c in cases:
        r = kero_calc("henderson-hasselbalch", c)
        py = c["pKa"] + math.log10(c["cB"] / c["cA"])
        check(f"HH pKa={c['pKa']} cA={c['cA']} cB={c['cB']}",
              r["value"], py, 1e-12, "Python")


def test_debye_huckel():
    print("\n=== Debye–Hückel limiting law ===")
    A = 0.5091
    cases = [
        {"z": 1, "I": 0.001},
        {"z": 2, "I": 0.005},
        {"z": 1, "I": 0.01},
    ]
    for c in cases:
        r = kero_calc("debye-huckel", c)
        log_gamma = -A * c["z"]**2 * math.sqrt(c["I"])
        gamma = 10**log_gamma
        check(f"DH z={c['z']} I={c['I']}",
              r["value"], gamma, 1e-10, "Python (A=0.5091)")


def test_van_t_hoff():
    print("\n=== Van 't Hoff ===")
    cases = [
        {"dH": 50_000, "K1": 1.0, "T1": 298.15, "T2": 373.15},
        {"dH": -30_000, "K1": 1e5, "T1": 300.0, "T2": 350.0},
        {"dH": 50_000, "K1": 42.0, "T1": 298.15, "T2": 298.15},
    ]
    for c in cases:
        r = kero_calc("van-t-hoff", c)
        py = c["K1"] * math.exp(
            (c["dH"] / R_CODATA2018) * (1.0 / c["T1"] - 1.0 / c["T2"])
        )
        check(f"vH dH={c['dH']} K1={c['K1']} T1={c['T1']} T2={c['T2']}",
              r["value"], py, 1e-12, "Python/CODATA2018")


def test_ionic_strength():
    print("\n=== Ionic strength ===")
    # kero calc ionic-strength uses z:m pairs, not key=value
    cmd = [KERO, "calc", "ionic-strength", "--json", "1:0.1", "-1:0.1"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    r = json.loads(result.stdout)
    py = 0.5 * (0.1 * 1**2 + 0.1 * 1**2)
    check("I(NaCl 0.1m)", r["value"], py, 1e-12, "Python")

    cmd = [KERO, "calc", "ionic-strength", "--json", "2:0.1", "-1:0.2"]
    result = subprocess.run(cmd, capture_output=True, text=True)
    r = json.loads(result.stdout)
    py = 0.5 * (0.1 * 4 + 0.2 * 1)
    check("I(CaCl2 0.1m)", r["value"], py, 1e-12, "Python")


def main():
    global KERO
    KERO = find_kero()
    if KERO is None:
        print("ERROR: kero binary not found. Build with: "
              "CARGO_TARGET_DIR=/tmp/kero-basic-target cargo build -p kerotakis-cli",
              file=sys.stderr)
        sys.exit(1)
    print(f"Using: {KERO}")

    test_arrhenius()
    test_eyring()
    test_nernst()
    test_henderson_hasselbalch()
    test_debye_huckel()
    test_van_t_hoff()
    test_ionic_strength()

    print()
    if failures:
        print(f"FAILED: {len(failures)} case(s)")
        for f in failures:
            print(f"  - {f}")
        sys.exit(1)
    else:
        print("All cases passed.")


if __name__ == "__main__":
    main()
