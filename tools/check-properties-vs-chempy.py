#!/usr/bin/env python3
"""Differential oracle for property correlations (CAP-6).

Compares `kero properties` output against ChemPy's water-property
modules and independent Python arithmetic.

    pip install chempy
    cargo build -p kerotakis-cli
    python3 tools/check-properties-vs-chempy.py

Exits non-zero if any case exceeds tolerance.
"""

import json
import math
import subprocess
import sys
import shutil

KERO = None
failures = []


def find_kero():
    for candidate in [
        "/tmp/kero-basic-target/debug/kero",
        "target/debug/kero",
    ]:
        if shutil.which(candidate) or __import__("os").path.isfile(candidate):
            return candidate
    found = shutil.which("kero")
    return found


def kero_prop(prop, args_dict):
    cmd = [KERO, "properties", prop, "--json"]
    for k, v in args_dict.items():
        cmd.append(f"{k}={v}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"kero properties failed: {result.stderr}")
    return json.loads(result.stdout)


def check(label, kero_val, ref_val, tol, ref_source):
    if ref_val == 0.0:
        rel = abs(kero_val)
    else:
        rel = abs(kero_val - ref_val) / abs(ref_val)
    ok = rel < tol
    status = "OK" if ok else "FAIL"
    print(f"  [{status}] {label}: kero={kero_val:.10e} ref={ref_val:.10e} "
          f"rel_diff={rel:.2e} tol={tol:.0e} ({ref_source})")
    if not ok:
        failures.append(label)


def test_water_density():
    print("\n=== Water density (Tanaka 2001) ===")
    from chempy.properties.water_density_tanaka_2001 import water_density

    temps = [273.15, 277.13, 283.15, 293.15, 298.15, 303.15, 313.15]
    for t in temps:
        r = kero_prop("water-density", {"T": t})
        cp = water_density(t)
        check(f"density T={t:.2f}K", r["value"], cp, 1e-10, "ChemPy/Tanaka")


def test_water_viscosity():
    print("\n=== Water viscosity (Korson 1969) ===")
    from chempy.properties.water_viscosity_korson_1969 import water_viscosity

    temps = [273.15, 283.15, 293.15, 298.15, 313.15, 333.15, 353.15, 373.15]
    for t in temps:
        r = kero_prop("water-viscosity", {"T": t})
        cp = water_viscosity(t)
        check(f"viscosity T={t:.2f}K", r["value"], cp, 1e-10, "ChemPy/Korson")


def test_water_permittivity():
    print("\n=== Water permittivity (Bradley–Pitzer 1979) ===")
    from chempy.properties.water_permittivity_bradley_pitzer_1979 import water_permittivity

    temps = [273.15, 298.15, 323.15, 373.15, 423.15, 523.15]
    for t in temps:
        r = kero_prop("water-permittivity", {"T": t})
        cp = water_permittivity(t)
        check(f"permittivity T={t:.2f}K", r["value"], cp, 1e-10, "ChemPy/Bradley-Pitzer")


def test_henry():
    print("\n=== Henry's constants (Sander 2015) ===")
    from chempy.henry import Henry

    gases = {
        "CO2": Henry(3.4e-2, 2400),
        "O2": Henry(1.3e-3, 1500),
        "N2": Henry(6.1e-4, 1300),
        "H2": Henry(7.8e-4, 500),
        "Cl2": Henry(9.2e-2, 2500),
        "NH3": Henry(5.7e1, 4200),
    }

    for formula, h in gases.items():
        for t in [278.15, 298.15, 323.15]:
            r = kero_prop("henry", {"gas": formula, "T": t})
            cp = h(t)
            check(f"henry {formula} T={t:.2f}K", r["value"], cp, 1e-10, "ChemPy/Sander")


def main():
    global KERO
    KERO = find_kero()
    if KERO is None:
        print("ERROR: kero binary not found.", file=sys.stderr)
        sys.exit(1)
    print(f"Using: {KERO}")

    test_water_density()
    test_water_viscosity()
    test_water_permittivity()
    test_henry()

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
