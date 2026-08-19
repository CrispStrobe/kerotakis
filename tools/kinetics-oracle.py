#!/usr/bin/env python3
"""Differential oracle for the kinetics integrator.

The always-on guard lives in Rust: `kinetics.rs` checks the integrator
against the *closed-form* solution of a first-order decay, which is exact
arithmetic rather than another implementation of the same guess. That test
has already earned its place — it caught a 0.7% drift that no other test
could see, and forced the switch from Euler to the midpoint method.

This script is the second opinion for cases with no closed form: several
reactions competing, or an order that makes the system non-linear. It
integrates the same rate laws with SciPy's adaptive solver and prints the
disagreement.

It is **not** part of the build, by design. Running a program over public
data does not make the output a derivative work, which is what lets an
LGPL/AGPL tool be used this way (PLAN.md, "Stoichiometry: ours, and why not
ChemicalFun"). Keep it that way: no import of this from the crates, and
nothing from here vendored.

    pip install scipy        # or: pip install chempy
    python3 tools/kinetics-oracle.py

Reference values come from `kerotakis-core/src/kinetics.rs`; edit both or
the comparison is meaningless.
"""

import sys

R = 8.314462618

# Mirrors kinetics::REGISTRY. Kept by hand and deliberately so — a
# generated copy would drift silently, while a hand copy that disagrees is
# exactly what this script exists to find.
CASES = [
    {
        "id": "peroxide-decomposition",
        "A": 1.6e10,
        "Ea": 75_000.0,
        "order": 1.0,
        "coefficient": 2.0,
        "c0": 0.5,
        "temperature": 298.15,
        "times": [1.0, 10.0, 120.0, 600.0],
    },
    {
        "id": "peroxide-decomposition + MnO2",
        "A": 1.6e10,
        "Ea": 58_000.0,
        "order": 1.0,
        "coefficient": 2.0,
        "c0": 0.5,
        "temperature": 298.15,
        "times": [1.0, 10.0, 60.0],
    },
    {
        "id": "thiosulfate-acid",
        "A": 2.2e8,
        "Ea": 51_000.0,
        # Pseudo-first-order: the acid term is constant at [H+] = 10^-1.7,
        # which is how the bench treats it too.
        "order": 1.0,
        "coefficient": 1.0,
        "proton": 10 ** -1.7,
        "c0": 0.05,
        "temperature": 298.15,
        "times": [5.0, 20.0, 60.0],
    },
]


def k_of(case, t):
    return case["A"] * pow(2.718281828459045, -case["Ea"] / (R * t))


def main():
    try:
        from scipy.integrate import solve_ivp
    except ImportError:
        print("scipy not installed; nothing to compare against", file=sys.stderr)
        return 1

    worst = 0.0
    for case in CASES:
        k = k_of(case, case["temperature"]) * case.get("proton", 1.0)
        coeff = case["coefficient"]

        def rhs(_t, y):
            c = max(y[0], 0.0)
            return [-coeff * k * pow(c, case["order"])]

        span = (0.0, max(case["times"]))
        sol = solve_ivp(
            rhs, span, [case["c0"]], t_eval=case["times"], rtol=1e-10, atol=1e-16
        )
        print(f"=== {case['id']}  k = {k:.6e}")
        for t, c in zip(case["times"], sol.y[0]):
            exact = case["c0"] * pow(2.718281828459045, -coeff * k * t)
            drift = abs(c - exact) / max(exact, 1e-30)
            worst = max(worst, drift)
            print(f"   t={t:8.1f} s   scipy={c:.8e}   closed form={exact:.8e}   {drift*100:.4f}%")
    print(f"\nworst disagreement between scipy and the closed form: {worst*100:.4f}%")
    print("Compare these against `cargo test -p kerotakis-core --lib kinetics`.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
