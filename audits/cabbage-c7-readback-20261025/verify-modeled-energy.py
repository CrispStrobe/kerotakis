#!/usr/bin/env python3
"""Read-only c7 cabbage model-energy check; no engine or production heat helper.

This prices only quantities the c7 model itself owns. The 1.97% opaque
cabbage-juice fraction is conserved, has no installed heat-capacity role, and
therefore contributes no *modeled* sensible heat. This is not calorimetry.
"""

import hashlib
import json
import math
import subprocess
from pathlib import Path

SOURCE = "c7aa38bddd6e12c694ace4abf2b5f8150cfdd33f"
TREE = "2da5f8e0b57b66c2b4fcbf1106c2b084b73bf17c"
VENDOR = "47b459c4a5e3e20b9990fa304e207eca0dcc6ff1"
PRIMARY = Path(__file__).resolve().parents[2]
PITZER = PRIMARY / "vendor/iphreeqc/database/pitzer.dat"
CAPTURE = Path(__file__).resolve().parent
R = 8.31446261815324
T_REF = 298.15


def git(*args: str) -> bytes:
    return subprocess.check_output(["git", "-C", str(PRIMARY), *args])


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def db_parameters(db: str, equation: str, key: str, count: int) -> list[float]:
    lines = db.splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == equation]
    assert len(starts) == 1, (equation, starts)
    start = starts[0]
    for line in lines[start + 1 : start + 9]:
        stripped = line.strip()
        if stripped.startswith(key):
            values = stripped.split()[1:]
            assert len(values) >= count, (equation, stripped)
            return [float(value) for value in values[:count]]
    raise AssertionError((equation, key))


def delta_h_analytic(coefficients: list[float]) -> float:
    """Independent van't Hoff derivative of the Pitzer printed log10 K row."""
    a1, a2, a3, a4, a5, a6 = coefficients
    t = T_REF
    dlogk = a2 - a3 / t**2 + a4 / (t * math.log(10)) - 2 * a5 / t**3 + 2 * a6 * t
    return R * t**2 * math.log(10) * dlogk / 1000


def water_cp(coefficients: list[float], t: float) -> float:
    """NASA-9 Cp/R as printed; only the 273.15-373.15 K interval is used."""
    terms = [1 / t**2, 1 / t, 1, t, t**2, t**3, t**4]
    return R * math.fsum(a * term for a, term in zip(coefficients, terms))


def water_cp_integral(coefficients: list[float], t: float) -> float:
    """Independent composite Simpson integral, 20000 intervals, not core helper."""
    if t == T_REF:
        return 0.0
    n = 20000
    step = (t - T_REF) / n
    terms = [water_cp(coefficients, T_REF), water_cp(coefficients, t)]
    terms.extend(4 * water_cp(coefficients, T_REF + step * i) for i in range(1, n, 2))
    terms.extend(2 * water_cp(coefficients, T_REF + step * i) for i in range(2, n, 2))
    return math.fsum(terms) * step / 3


def amount(vessel: dict, species: str) -> float:
    return math.fsum(row["moles"] for row in vessel["contents"] if row["species"] == species)


def main() -> None:
    assert git("rev-parse", SOURCE).decode().strip() == SOURCE
    assert git("rev-parse", f"{SOURCE}^{{tree}}").decode().strip() == TREE
    assert git("rev-parse", f"{SOURCE}:vendor/iphreeqc").decode().strip() == VENDOR
    registry_bytes = git("show", f"{SOURCE}:data/registry/registry-source-v1.json")
    registry = json.loads(registry_bytes)
    db_bytes = subprocess.check_output(["git", "-C", str(PITZER.parents[1]), "show", VENDOR + ":database/pitzer.dat"])
    db = db_bytes.decode("latin-1")  # vendored USGS file contains legacy non-UTF8 comments
    water_rows = [row for row in registry["heat_capacity_polynomials"] if row["id"] == "heat-capacity-polynomial/water/liquid"]
    assert len(water_rows) == 1
    water_interval = water_rows[0]["intervals"][0]
    assert water_interval["t_min_k"] <= T_REF <= water_interval["t_max_k"]
    assert water_rows[0]["form"] == "nasa9"
    water_coefficients = water_interval["coefficients"]
    soda_rows = [row for row in registry["phase_thermodynamics"] if row["id"] == "dissolution-enthalpy/Na2CO3"]
    assert len(soda_rows) == 1
    soda_dh = soda_rows[0]["quantity"]["value"]
    assert soda_dh == -26.656 and soda_rows[0]["quantity"]["unit"]["symbol"] == "kJ/mol"
    anthocyanin_rows = [row for row in registry["phase_thermodynamics"] if row["id"] == "heat-capacity/anthocyanin"]
    assert len(anthocyanin_rows) == 1
    pigment_cp = anthocyanin_rows[0]["quantity"]["value"]
    assert pigment_cp == 700.0
    juice_rows = [row for row in registry["material_recipes"] if row["id"] == "household/red-cabbage-juice-surrogate"]
    assert len(juice_rows) == 1
    juice = juice_rows[0]
    assert juice["roles"] == [{"kind": "surface_colourant", "srgb": [107, 63, 160]}]
    assert juice["unresolved_fraction"]["lower"] == juice["unresolved_fraction"]["upper"] == 0.0197
    h_oh = delta_h_analytic(db_parameters(db, "H2O = OH- + H+", "-analytic", 6))
    h_hco3 = delta_h_analytic(db_parameters(db, "CO3-2 + H+ = HCO3-", "-analytic", 5) + [0.0])
    h_co2 = db_parameters(db, "CO3-2 + 2 H+ = CO2 + H2O", "delta_h", 1)[0] * 4.184
    assert "delta_h -5.738 kcal" in db and h_co2 == -5.738 * 4.184

    lines = CAPTURE.joinpath("stdout").read_bytes().splitlines()
    records = [json.loads(line) for line in lines]
    assert records[16]["operator"]["op"] == "add"
    assert records[17]["operator"]["material"] == "washing_soda"
    assert records[18]["operator"]["material"] == "red_cabbage_juice"
    for record in records[16:19]:
        assert not any(event["event"] in ("gas_evolved", "gas_absorbed", "heat_of_mixing") for event in record["events"])
    vessels = [records[i]["bench"]["vessels"][3] for i in (16, 17, 18)]
    assert all(vessel["id"] == 3 and vessel["thermal_mode"] == "adiabatic" for vessel in vessels)
    assert vessels[1]["solution"]["provenance"]["dataset"].startswith("pitzer.dat")
    assert vessels[2]["solution"]["provenance"]["dataset"].startswith("pitzer.dat")

    def state_energy(vessel: dict) -> dict:
        t = vessel["temperature"]
        assert water_interval["t_min_k"] <= t <= water_interval["t_max_k"]
        chemical_j = 1000 * (
            h_oh * vessel["free_hydroxide"]
            + h_hco3 * amount(vessel, "HCO3-")
            + h_co2 * amount(vessel, "CO2(aq)")
        )
        water_sensible_j = amount(vessel, "water") * water_cp_integral(water_coefficients, t)
        pigment_sensible_j = amount(vessel, "anthocyanin") * pigment_cp * (t - T_REF)
        return {
            "temperature_k": t,
            "chemical_j": chemical_j,
            "water_sensible_j": water_sensible_j,
            "pigment_sensible_j": pigment_sensible_j,
            "modeled_total_j": chemical_j + water_sensible_j + pigment_sensible_j,
        }

    values = [state_energy(vessel) for vessel in vessels]
    soda_moles = 4.0 / 105.988
    solid_j = soda_moles * (-soda_dh) * 1000
    initial_j = values[0]["modeled_total_j"] + solid_j
    residuals = [values[1]["modeled_total_j"] - initial_j, values[2]["modeled_total_j"] - initial_j]
    assert abs(residuals[0]) <= 0.001 and abs(residuals[1]) <= 0.001, residuals
    capture_result = json.loads(CAPTURE.joinpath("RESULT.json").read_text())
    assert capture_result["exit_status"] == 1  # invalid appended CLI wait 0s suffix; original prefix completed
    assert capture_result["source"] == SOURCE
    assert capture_result["stdout_sha256"] == sha(CAPTURE.joinpath("stdout").read_bytes())
    report = {
        "kind": "independent_modeled_energy_check_not_calorimetry",
        "source": SOURCE,
        "tree": TREE,
        "vendor_gitlink": VENDOR,
        "registry_sha256": sha(registry_bytes),
        "pitzer_raw_sha256": sha(db_bytes),
        "capture_stdout_sha256": sha(CAPTURE.joinpath("stdout").read_bytes()),
        "source_references_kj_per_mol": {"solid_Na2CO3": -soda_dh, "OH_minus": h_oh, "HCO3_minus": h_hco3, "CO2_aq": h_co2},
        "water_cp_at_298_15_j_per_mol_k": water_cp(water_coefficients, T_REF),
        "water_cp_at_final_j_per_mol_k": water_cp(water_coefficients, vessels[2]["temperature"]),
        "states": values,
        "soda_solid_initial_reference_j": solid_j,
        "initial_modeled_energy_j": initial_j,
        "post_soda_residual_j": residuals[0],
        "post_juice_residual_j": residuals[1],
        "unchanged_numerical_budget_j": 0.001,
        "limits": [
            "The native route is Pitzer, not WATEQ; final ionic strength exceeds 1 mol/kgw.",
            "The juice surrogate includes 0.197 g conserved unresolved material with no modeled Cp; no finite-concentration calorimetric calibration is inferred.",
            "The appended CLI wait 0s suffix was rejected by the parser; zero-time re-equilibration was not reached.",
        ],
    }
    print(json.dumps(report, sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
