#!/usr/bin/env python3
"""Reproduce the reviewed HBr standard-state conversion; not a runtime model.

Sander (2023), CC BY 4.0, doi:10.5194/acp-23-10901-2023, note 146:
Hs' = 8.2e9 exp[10000 K (1/T - 1/298.15)] mol²/(m⁶ Pa).
Changes: convert concentration to molality and derive the local enthalpy.
Density calculation adapted from USGS Phreeqc::calc_rho_0, already approved
under vendor/iphreeqc/phreeqc3-doc/NOTICE.TXT; preserve that rights notice.
See provenance/sander-2023-hbr-review.md and NOTICE for source attribution.
"""

import hashlib
import json
import math
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
USGS_SOURCE = ROOT / "vendor/iphreeqc/src/phreeqcpp/utilities.cpp"
USGS_SHA256 = "a93c82d158df1ef0bf8d3ee426614382a0dbfe5ab6431b6cb4bfefddfc7bfc28"
SANDER_PDF_SHA256 = "8c059bc311cacb0b4626f1935a06f990303d9af0af50721dd5fb864ddbc88bad"
GAS_DATA = ROOT / "data/thermo/sander-2023-hbr.dat"
TEMPERATURE_K = 298.15
R_J_MOL_K = 8.31446261815324
HS_PRIME_SI = 8.2e9
INVERSE_T_SLOPE_K = 10000.0
PA_PER_ATM = 101325.0


def pure_water_density(tc: float, pressure_atm: float = 1.0) -> float:
    """USGS calc_rho_0 expression, returning kg/m³ (not its final g/cm³).

    Pure-water activity is 1. This audit evaluates only near 25 °C and 1 atm,
    so native high-temperature clamps and solution-state effects do not apply.
    """
    temperature = tc + 273.15
    theta = 1.0 - temperature / 647.096
    rho_sat = 322.0 * (
        1.0
        + 1.99274064 * theta ** (1.0 / 3.0)
        + 1.09965342 * theta ** (2.0 / 3.0)
        - 0.510839303 * theta ** (5.0 / 3.0)
        - 1.75493479 * theta ** (16.0 / 3.0)
        - 45.5170352 * theta ** (43.0 / 3.0)
        - 6.7469445e5 * theta ** (110.0 / 3.0)
    )
    p0 = 5.1880000e-2 + tc * (
        -4.1885519e-4 + tc * (6.6780748e-6 + tc * (-3.6648699e-8 + tc * 8.3501912e-11))
    )
    p1 = -6.0251348e-6 + tc * (
        3.6696407e-7 + tc * (-9.2056269e-9 + tc * (6.7024182e-11 - tc * 1.5947241e-13))
    )
    p2 = -2.2983596e-9 + tc * (
        -4.0133819e-10 + tc * (1.2619821e-11 + tc * (-9.8952363e-14 + tc * 2.3363281e-16))
    )
    p3 = 7.0517647e-11 + tc * (
        6.8566831e-12 + tc * (-2.2829750e-13 + tc * (1.8113313e-15 - tc * 4.2475324e-18))
    )
    p_sat = math.exp(11.6702 - 3816.44 / (temperature - 46.13))
    pressure = max(pressure_atm, p_sat) - (p_sat - 1e-6)
    return rho_sat + pressure * (
        p0 + pressure * (p1 + pressure * (p2 + math.sqrt(pressure) * p3))
    )


def main() -> None:
    actual_sha = hashlib.sha256(USGS_SOURCE.read_bytes()).hexdigest()
    if actual_sha != USGS_SHA256:
        raise SystemExit("USGS density source changed: re-review the transcription before use")

    tc = TEMPERATURE_K - 273.15
    density = pure_water_density(tc)
    # At infinite dilution c=rho*b. Two ion concentrations imply rho².
    equilibrium = HS_PRIME_SI * PA_PER_ATM / density**2
    log_k = math.log10(equilibrium)
    convergence = []
    for step in (0.01, 0.001, 0.0001):
        derivative = (pure_water_density(tc + step) - pure_water_density(tc - step)) / (2 * step)
        enthalpy = (
            -R_J_MOL_K * INVERSE_T_SLOPE_K
            - 2 * R_J_MOL_K * TEMPERATURE_K**2 * derivative / density
        ) / 1000.0
        convergence.append({"step_K": step, "density_derivative": derivative, "delta_h_kj_mol": enthalpy})

    fields = {}
    for line in GAS_DATA.read_text().splitlines():
        words = line.split()
        if words and words[0] in ("log_k", "delta_h"):
            fields[words[0]] = float(words[1])
    assert abs(fields["log_k"] - log_k) < 1e-12
    # Platform libm rounding can perturb the numerical derivative; this
    # tolerance remains far tighter than the two-digit experimental input.
    assert abs(fields["delta_h"] - convergence[1]["delta_h_kj_mol"]) < 1e-7
    assert max(row["delta_h_kj_mol"] for row in convergence) - min(
        row["delta_h_kj_mol"] for row in convergence
    ) < 1e-7
    print(json.dumps({
        "density_source_sha256": actual_sha,
        "reviewed_sander_pdf_sha256": SANDER_PDF_SHA256,
        "temperature_K": TEMPERATURE_K,
        "pressure_atm": 1.0,
        "density_kg_m3": density,
        "K_molal_atm": equilibrium,
        "log10_K": log_k,
        "derivative_convergence": convergence,
        "runtime_data_fields_verified": True,
    }, indent=2))


if __name__ == "__main__":
    main()
