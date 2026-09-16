#!/usr/bin/env python3
"""CAP-19 / tier C: generate differential-oracle fixtures from the Python
`thermo` package (MIT, Caleb Bell) — the build-time second opinion PLAN.md
assigns to kerotakis-thermo.

`provenance/upstreams.toml` clears `thermo-python` as `oracle-only`, with a
sentence this script is bound by: *"only the numbers it PREDICTS from our
own inputs may be compared against."* So nothing here reads thermo's
compound database. Group assignments are stated below, Antoine constants
are duplicated from `vle.rs`, and thermo is asked only to PREDICT from
those. `tools/vle-oracle.py` does read that database (`VaporPressure(
CASRN=...)`, which is backed by the `chemicals` package the same table
marks `avoid`); it is not wired to anything, and it must not be wired
until that is resolved.

Four fixture families, all replayed by tests/thermo_oracle.rs, and a fifth
that is deliberately NOT emitted. Every one states what it is independent
OF, because an oracle sharing a database with the path it checks proves
only that the code reads the database:

  gamma  pair x_a t_kelvin gamma_a gamma_b
      thermo's own UNIFAC (original published interaction parameters) on
      every binary the approved table can form. INDEPENDENT OF: our
      combinatorial and residual arithmetic, our group bookkeeping, and
      our transcription of r, q and a_mn — a mistyped parameter changes
      gamma and this family sees it. NOT INDEPENDENT OF: the parameter
      publication itself (Fredenslund 1975 / Gmehling 1982), which both
      sides read. This is the check that would have caught the
      combinatorial double-division bug on day one.

  hmix   pair x_a t_kelvin he_j_per_mol
      excess enthalpy. Ours is Gibbs-Helmholtz by a central difference in
      T with dT = 1 K (`excess.rs`); thermo's HE() is the analytic
      derivative of the same model. INDEPENDENT OF: our differentiation
      scheme as well as our arithmetic — the row bounds the truncation
      error our comment asserts is "far below the model's own honesty
      bound" instead of asserting it. NOT INDEPENDENT OF: the parameters,
      or of the fact that VLE-fitted parameters give a qualitative hE.

  gammainf solute solvent t_kelvin gamma_inf
      infinite-dilution activity coefficient, the quantity whose ratio
      `bench.rs` turns into a partition coefficient. Same independence as
      `gamma`; listed separately because the consumer is a RATIO of two
      of these and the dilute corner is where our 1e-9 stand-in for
      "infinitely dilute" could differ from thermo's.

  bubble pair x_a p_kpa t_celsius y_a
      bubble points solved HERE, in this script, by plain bisection on
      sum(x_i * gamma_i(T) * Psat_i(T)) = P, with gammas from thermo and
      the same Antoine constants vle.rs curates. INDEPENDENT OF: our
      flash solver and its bracketing. NOT INDEPENDENT OF: the Antoine
      constants, which are duplicated on purpose — the oracle solves the
      same stated model independently, and if the constants drift apart
      the fixtures stop matching and say so. NOTHING here checks whether
      those constants are right; see docs/ORACLE-COVERAGE.md.

  param  kind subgroup_or_pair ...
      NOT EMITTED, deliberately. Echoing thermo's r, q and a_mn into a
      committed fixture would be copying its parameter table into this
      repository under thermo's provenance, when those values already
      live in `approved_table()` under their own. The gamma grid is
      chosen instead so that every one of the 30 interaction parameters
      and all 10 subgroups move at least one emitted number.

Run from the repo root with a venv holding `thermo`:
  ./path/to/python tools/gen-thermo-fixtures.py \
      > crates/kerotakis-thermo/tests/fixtures/thermo_oracle.tsv
"""

from thermo.unifac import UNIFAC, UFIP, UFSG

# ---------------------------------------------------------------------------
# Components. Every decomposition below uses ONLY subgroups that
# `crates/kerotakis-thermo/src/unifac.rs::approved_table()` curates, and the
# Rust test carries the same table keyed by the same names — a name either
# side does not know is a hard failure there, not a silently skipped row.
# ---------------------------------------------------------------------------
COMPONENTS = {
    "water": {16: 1},                       # H2O
    "methanol": {15: 1},                    # CH3OH as its own subgroup
    "ethanol": {1: 1, 2: 1, 14: 1},         # CH3 CH2 OH
    "propan-2-ol": {1: 2, 3: 1, 14: 1},     # exercises CH (subgroup 3)
    "propanone": {1: 1, 18: 1},             # CH3 CH3CO
    "pentan-3-one": {1: 2, 2: 1, 19: 1},    # exercises CH2CO (subgroup 19)
    "ethanoic-acid": {1: 1, 42: 1},         # CH3 COOH
    "hexane": {1: 2, 2: 4},
    "2-2-dimethylpropane": {1: 4, 4: 1},    # exercises C (subgroup 4)
    # The decomposition `kerotakis-core/src/bench.rs::partition_groups` feeds
    # the partition path for methanol. It is NOT the original-UNIFAC
    # assignment, which is the single CH3OH subgroup above; it is pinned here
    # so the difference is a number in a fixture rather than a thing nobody
    # looked at. See docs/ORACLE-COVERAGE.md.
    "methanol-as-ch3-oh": {1: 1, 14: 1},
}

# Binaries, chosen so that every ordered interaction parameter in
# `approved_table()` moves at least one emitted gamma. Main groups: 1 CH2,
# 5 OH, 6 CH3OH, 7 H2O, 9 CH2CO, 20 COOH; the comment on each row names the
# unordered main-group pair(s) it is there to reach.
BINARIES = [
    ("ethanol", "water"),               # 1-5, 1-7, 5-7
    ("methanol", "water"),              # 6-7
    ("propan-2-ol", "water"),           # 1-5, 1-7, 5-7 through subgroup 3
    ("propanone", "water"),             # 1-9, 1-7, 7-9   (heat of mixing)
    ("pentan-3-one", "water"),          # 7-9 through subgroup 19
    ("ethanoic-acid", "water"),         # 1-20, 1-7, 7-20
    ("hexane", "water"),                # 1-7           (the LLE pair)
    ("2-2-dimethylpropane", "water"),   # 1-7 through subgroup 4
    ("methanol", "ethanol"),            # 5-6, 1-6
    ("propanone", "ethanol"),           # 5-9, 1-9, 1-5
    ("ethanoic-acid", "ethanol"),       # 5-20, 1-20, 1-5
    ("methanol", "hexane"),             # 1-6
    ("propanone", "hexane"),            # 1-9
    ("ethanoic-acid", "hexane"),        # 1-20
    ("methanol", "propanone"),          # 6-9
    ("methanol", "ethanoic-acid"),      # 6-20
    ("propanone", "ethanoic-acid"),     # 9-20, 1-9, 1-20
]

GAMMA_X = [0.01, 0.05, 0.2, 0.5, 0.8, 0.95, 0.99]
GAMMA_T = [298.15, 333.15]
# The pair whose dilute corners a consumer actually reads: `solve.rs` asks
# for the water-hexane split, which is decided entirely by gamma out where
# both are tiny. A binodal solver of our own is what turns these into
# compositions; that step is NOT oracle-checked and the coverage doc says so.
DILUTE_X = [1e-6, 1e-5, 1e-4, 1e-3, 1e-2]

HMIX_X = [0.1, 0.3, 0.5, 0.7, 0.9]
HMIX_T = [298.15]

# Solute/solvent pairs the partition path in `bench.rs` actually evaluates,
# plus the standard methanol assignment beside the one it uses.
GAMMAINF = [
    ("ethanol", "water"),
    ("ethanol", "hexane"),
    ("methanol", "water"),
    ("methanol", "hexane"),
    ("methanol-as-ch3-oh", "water"),
    ("methanol-as-ch3-oh", "hexane"),
    ("propanone", "water"),
    ("propanone", "hexane"),
]
GAMMAINF_T = [298.15, 310.0]
# What `lle::infinite_dilution_gamma` uses as its stand-in for "infinitely
# dilute". Mirrored rather than imported: if the Rust constant moves and this
# one does not, the fixtures disagree, which is the point.
INFINITE_DILUTION_X = 1e-9

# Antoine, log10(P/kPa) = a - b/(T_C + c) — must match vle.rs. Ethanol's
# high segment is the CC-BY-4.0 experimental fit from
# doi:10.1021/acsomega.6c04827, transformed exactly from kelvin to Celsius.
# Duplicated deliberately; see the `bubble` note in the module docstring.
ANTOINE = {
    "ethanol_low": (7.32907, 1642.89, 230.300),
    "ethanol_high": (6.99161, 1460.701, 214.673),
    "water": (7.19621, 1730.63, 233.426),
    "methanol": (7.20607, 1582.271, 239.726),
    "propan-2-ol": (6.861, 1357.427, 197.336),
    "propanone": (6.14957, 1161.0, 224.0),
    "ethanoic-acid": (6.51292, 1533.313, 222.309),
}
ATM = 101.325

# Binaries with curated Antoine constants on BOTH sides, so a bubble point
# is a thing both implementations can solve.
BUBBLE_BINARIES = [
    ("ethanol", "water"),
    ("methanol", "water"),
    ("propan-2-ol", "water"),
    ("propanone", "water"),
    ("ethanoic-acid", "water"),
]
BUBBLE_X = [0.01, 0.05, 0.1, 0.3, 0.5, 0.7, 0.894]


def model(pair, x_a, t_k):
    a, b = pair
    return UNIFAC.from_subgroups(
        T=t_k,
        xs=[x_a, 1.0 - x_a],
        chemgroups=[COMPONENTS[a], COMPONENTS[b]],
        version=0,
        interaction_data=UFIP,
        subgroups=UFSG,
    )


def gammas(pair, x_a, t_k):
    return model(pair, x_a, t_k).gammas()


def psat(name, t_c):
    if name == "ethanol":
        name = "ethanol_low" if t_c <= 80.0 else "ethanol_high"
    a, b, c = ANTOINE[name]
    return 10.0 ** (a - b / (t_c + c))


def bubble(pair, x, p_kpa):
    a, b = pair

    def total(t_c):
        g = gammas(pair, x, t_c + 273.15)
        return x * g[0] * psat(a, t_c) + (1.0 - x) * g[1] * psat(b, t_c) - p_kpa

    lo, hi = 0.0, 200.0
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if total(mid) < 0.0:
            lo = mid
        else:
            hi = mid
    t = 0.5 * (lo + hi)
    g = gammas(pair, x, t + 273.15)
    pa = x * g[0] * psat(a, t)
    pb = (1.0 - x) * g[1] * psat(b, t)
    return t, pa / (pa + pb)


def main():
    import thermo

    print("# generated by tools/gen-thermo-fixtures.py — do not edit")
    print(f"# thermo {thermo.__version__}, UNIFAC original parameters (UFIP/UFSG)")
    print("# columns: gamma pair x_a T_K g_a g_b | hmix pair x_a T_K hE_J_per_mol")
    print("#          gammainf solute solvent T_K g_inf | bubble pair x_a P_kPa T_C y_a")
    print("# every family's independence claim is in the generator docstring and")
    print("# in docs/ORACLE-COVERAGE.md; a row is not evidence without it.")

    for pair in BINARIES:
        key = f"{pair[0]}+{pair[1]}"
        grid = list(GAMMA_X)
        if pair == ("hexane", "water"):
            grid = sorted(set(DILUTE_X + GAMMA_X + [1.0 - d for d in DILUTE_X]))
        for x in grid:
            for t_k in GAMMA_T:
                ga, gb = gammas(pair, x, t_k)
                print(f"gamma\t{key}\t{x!r}\t{t_k}\t{ga:.10e}\t{gb:.10e}")

    for pair in BINARIES:
        key = f"{pair[0]}+{pair[1]}"
        for x in HMIX_X:
            for t_k in HMIX_T:
                he = model(pair, x, t_k).HE()
                print(f"hmix\t{key}\t{x!r}\t{t_k}\t{he:.10e}")

    for solute, solvent in GAMMAINF:
        for t_k in GAMMAINF_T:
            g = gammas((solute, solvent), INFINITE_DILUTION_X, t_k)[0]
            print(f"gammainf\t{solute}\t{solvent}\t{t_k}\t{g:.10e}")

    for pair in BUBBLE_BINARIES:
        key = f"{pair[0]}+{pair[1]}"
        for x in BUBBLE_X:
            t, y = bubble(pair, x, ATM)
            print(f"bubble\t{key}\t{x!r}\t{ATM}\t{t:.6f}\t{y:.8f}")


if __name__ == "__main__":
    main()
