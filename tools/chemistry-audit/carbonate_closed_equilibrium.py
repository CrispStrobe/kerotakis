"""Independent closed-NaHCO3 law, version 1; no solver invocation.

Domain: the four unchanged 300 mL water/NaHCO3 dose protocols, open vessel,
zero elapsed time, WATEQ4F speciation. This checks balances and mass action;
it deliberately imposes no pH ordering. Original fleet-v2 forecasts/results
must remain archived separately when this law supersedes their dose ordering.

Reference data: vendor/iphreeqc at 47b459c4a5e3e20b9990fa304e207eca0dcc6ff1,
database/wateq4f.dat (OH-, HCO3-, CO2, NaCO3-, NaHCO3 entries). The independent
equations use activities, water activity, actual temperature and solvent mass.
USGS PHREEQC manual: https://pubs.usgs.gov/wri/1999/4259/report.pdf
"""
from __future__ import annotations

import math

MODEL = "wateq4f-closed-carbon-v1"
DATASET = ("wateq4f.dat plus USBM IC 9429 reference-temperature complexes, "
           "with the reviewed Sander HBr gas-uptake slice")
ENGINE = "PHREEQC (IPhreeqc, USGS)"
ACTIVITY_MODEL = "WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)"
DOSES = (0.0005, 0.001, 0.002, 0.004)
# registry-source-v1.json phase_thermodynamics entries density/water
# (0.997 g/mL) and molar-mass/water (18.015 g/mol). These are the parser's
# actual input-unit coordinates; the closure law must not silently substitute
# a more precise external molar mass for that independently bound input.
WATER_MOLES = (300 * 0.997) / 18.015
LEDGER_ATOL_MOL = 1e-10
# The source's documented thermal quantisation-cycle ceiling, not a fitted
# chemical tolerance; the normal fixed-point stop is tighter (1e-6 K).
TEMPERATURE_UNCERTAINTY_K = 5e-5
CHARGES = {"Na+": 1, "H+": 1, "OH-": -1, "HCO3-": -1,
           "CO3-2": -2, "NaCO3-": -1}
SPECIES = set(CHARGES) | {"H2O", "O2", "CO2", "NaHCO3"}
CARBON = ("CO2", "HCO3-", "CO3-2", "NaHCO3", "NaCO3-")
SODIUM = ("Na+", "NaHCO3", "NaCO3-")
FORMULAS = {
    "water": {"H": 2, "O": 1}, "Na+": {"Na": 1},
    "CO2(aq)": {"C": 1, "O": 2}, "HCO3-": {"H": 1, "C": 1, "O": 3},
    "CO3-2": {"C": 1, "O": 3},
    # Conservation coordinate OH-, NOT a free hydroxide measurement.
    "base_equivalents": {"H": 1, "O": 1},
}
K_BIC = (107.8871, .03252849, -5151.79, -38.92561, 563713.9)
K_CO2 = (464.1965, .09344813, -26986.16, -165.75951, 2248628.9)
K_WATER = (-283.971, -.05069842, 13323, 102.24447, -1119669)


class LawFailure(ValueError):
    """Missing authority, incomplete evidence, or a violated physical law."""


def require(condition, reason):
    if not condition:
        raise LawFailure(reason)


def finite(value, name):
    require(isinstance(value, (int, float)) and not isinstance(value, bool)
            and math.isfinite(value), f"invalid finite number: {name}")
    return float(value)


def rounded_interval(value):
    """PHREEQC report uses four significant decimal digits (including zeros).

    JSON discards lexical trailing zeros, so infer the quantum from the
    report's known precision, not from the JSON string's decimal count.
    """
    value = finite(value, "reported species")
    require(value > 0, "nonpositive species report")
    half_quantum = .5 * 10 ** (math.floor(math.log10(value)) - 3)
    return (math.nextafter(value-half_quantum, -math.inf),
            math.nextafter(value+half_quantum, math.inf))


def linear_interval(terms):
    low = high = 0.
    for coefficient, interval in terms:
        a, b = (coefficient*x for x in interval)
        low += min(a, b)
        high += max(a, b)
    return math.nextafter(low, -math.inf), math.nextafter(high, math.inf)


def contains(interval, value):
    return interval[0] <= value <= interval[1]


def log_interval(interval):
    require(interval[0] > 0, "nonpositive activity interval")
    return math.log10(interval[0]), math.log10(interval[1])


def polynomial(coefficients, temperature):
    a, b, c, d, e = coefficients
    return a+b*temperature+c/temperature+d*math.log10(temperature)+e/temperature**2


def reference_interval(coefficients, temperature):
    """Bound log K over the documented temperature uncertainty by calculus."""
    a, b, c, d, e = coefficients
    tlo = temperature-TEMPERATURE_UNCERTAINTY_K
    thi = temperature+TEMPERATURE_UNCERTAINTY_K
    # Triangle bound on |d(log K)/dT| over the whole interval, not merely
    # endpoint sampling (which could miss an internal stationary point).
    slope = abs(b)+abs(c)/tlo**2+abs(d)/(tlo*math.log(10))+2*abs(e)/tlo**3
    center = polynomial(coefficients, temperature)
    error = slope*TEMPERATURE_UNCERTAINTY_K + 32*math.ulp(
        abs(a)+abs(b*thi)+abs(c/tlo)+abs(d*math.log10(thi))+abs(e/tlo**2))
    return center-error, center+error


def _ledger(vessel):
    result = {e: 0. for e in ("Na", "C", "H", "O")}
    require(len(vessel.get("contents", [])) > 0, "empty physical ledger")
    for portion in vessel["contents"]:
        name = portion.get("species")
        require(name in FORMULAS, f"unreviewed physical species: {name}")
        require(portion.get("phase") == ("liquid" if name == "water" else "aqueous"),
                "unexpected owned phase")
        amount = finite(portion.get("moles"), "physical moles")
        require(amount >= 0, "negative physical inventory")
        for element, number in FORMULAS[name].items():
            result[element] += number*amount
    return result


def check_rows(rows, expected_dose):
    """Return evidence, or raise LawFailure; expected dose comes from manifest.

    The supplied dose is independent of the observed output. No caller may
    infer expected_dose from an output it is about to certify.
    """
    require(expected_dose in DOSES, "dose outside versioned law domain")
    require(len(rows) == 4, "expected water, bicarbonate, measure, inspect rows")
    require([r.get("operator", {}).get("op") for r in rows]
            == ["add", "add", "measure", "inspect"], "unexpected protocol operations")
    first, dose_op = rows[0]["operator"], rows[1]["operator"]
    require(first.get("species") == "water" and first.get("vessel") == 0
            and abs(finite(first.get("moles"), "water dose")-WATER_MOLES) <= LEDGER_ATOL_MOL,
            "water dose differs from unchanged 300 mL protocol")
    require(dose_op.get("species") == "NaHCO3" and dose_op.get("vessel") == 0
            and finite(dose_op.get("moles"), "bicarbonate dose") == expected_dose,
            "bicarbonate dose differs from manifest")
    require(rows[2]["operator"].get("instrument") == "ph_meter", "wrong measurement")
    expected = {"Na": expected_dose, "C": expected_dose,
                "H": 2*WATER_MOLES+expected_dose,
                "O": WATER_MOLES+3*expected_dose}
    evidence = []
    for index, row in enumerate(rows):
        vessels = row.get("bench", {}).get("vessels", [])
        require(len(vessels) == 1 and vessels[0].get("id") == 0, "wrong vessel ownership")
        v = vessels[0]
        require(v.get("headspace") == {"boundary": "open"}, "wrong gas boundary")
        require(finite(v.get("elapsed_seconds"), "elapsed time") == 0, "elapsed time is nonzero")
        require(finite(v.get("pending_co2_transfer_mol"), "pending transfer") == 0,
                "unspent external carbon transfer")
        require(not any(e.get("event") in ("gas_evolved", "gas_absorbed", "solver_failed",
                                           "not_modeled", "not_yet_modeled", "safety_veto")
                        for e in row.get("events", [])),
                "external transfer or unsupported/failed execution")
        ledger = _ledger(v)
        physical_charge = sum(
            portion["moles"] * {"Na+": 1, "HCO3-": -1, "CO3-2": -2,
                                 "base_equivalents": -1}.get(portion["species"], 0)
            for portion in v["contents"])
        require(abs(physical_charge) <= LEDGER_ATOL_MOL,
                "canonical physical charge does not close")
        target = expected if index else {"Na": 0., "C": 0., "H": 2*WATER_MOLES, "O": WATER_MOLES}
        require(all(abs(ledger[e]-target[e]) <= LEDGER_ATOL_MOL for e in target),
                "Na/C/H/O physical inventory does not close")
        if not index:
            continue
        solution = v.get("solution") or {}
        provenance = solution.get("provenance") or {}
        require(provenance.get("dataset") == DATASET and provenance.get("engine") == ENGINE,
                "unbound or changed thermodynamic provenance")
        require(provenance.get("model") == ACTIVITY_MODEL
                and provenance.get("routing") == "the default inorganic aqueous dataset",
                "unreviewed activity model or routing")
        # vessel.rs SolutionInfo.scope uses serde(default,
        # skip_serializing_if=SolutionScope::is_complete). Absence therefore
        # means Complete in this bound schema; explicit null/unknown values
        # do not. Complete provenance and the full domain census are still
        # mandatory below, so absence alone cannot confer acceptance.
        require("scope" not in solution or solution["scope"] == "complete",
                "incomplete solution scope")
        temperature = finite(v.get("temperature"), "temperature")
        require(273.15 < temperature < 323.15, "temperature outside bounded aqueous domain")
        kg = finite(solution.get("solvent_kg"), "solvent mass")
        require(kg > 0, "invalid solvent mass")
        details = solution.get("species", [])
        require(len(details) == len(SPECIES) and {x.get("name") for x in details} == SPECIES,
                "missing, duplicate or unreviewed aqueous species")
        m = {x["name"]: rounded_interval(x.get("molality")) for x in details}
        activities = {x["name"]: rounded_interval(x.get("activity")) for x in details}
        # H2O activity is replaced from precise numerical selected output by
        # apply_numerical_water_activity; it is not a rounded report scalar.
        aw = finite(next(x["activity"] for x in details if x["name"] == "H2O"), "water activity")
        require(0 < aw <= 1, "invalid water activity")
        activities["H2O"] = (math.nextafter(aw, 0.), math.nextafter(aw, math.inf))
        for names in (CARBON, SODIUM):
            total = linear_interval((kg, m[name]) for name in names)
            require(contains(total, expected_dose), "aqueous component census does not close")
        charge = linear_interval((z, m[name]) for name, z in CHARGES.items())
        require(contains(charge, 0.), "aqueous charge interval does not include zero")
        strength = linear_interval((.5*z*z, m[name]) for name, z in CHARGES.items())
        require(contains(strength, finite(solution.get("ionic_strength"), "ionic strength")),
                "ionic strength inconsistent with species charges")
        ph = finite(solution.get("ph"), "pH")
        if index == 2:
            measured = [e for e in row.get("events", []) if e.get("event") == "measured"]
            require(len(measured) == 1
                    and finite(measured[0].get("value"), "measured pH") == ph,
                    "reported pH measurement differs from the settled solution")
        # Final selected output serializes pH to twelve decimals in these
        # records. Compare its proton activity to the report's uncertainty.
        require(contains(activities["H+"], 10**-ph), "pH inconsistent with H+ activity")
        loga = {name: log_interval(value) for name, value in activities.items()}
        def quotient(terms):
            return linear_interval((power, loga[name]) for name, power in terms.items())
        equations = {
            "K2": (quotient({"H+": 1, "CO3-2": 1, "HCO3-": -1}),
                   linear_interval([(-1, reference_interval(K_BIC, temperature))])),
            "CO2-association": (quotient({"CO2": 1, "H2O": 1, "CO3-2": -1, "H+": -2}),
                                reference_interval(K_CO2, temperature)),
            "Kw": (quotient({"H+": 1, "OH-": 1, "H2O": -1}),
                   reference_interval(K_WATER, temperature)),
            "NaHCO3": (quotient({"NaHCO3": 1, "Na+": -1, "HCO3-": -1}), (-.25, -.25)),
        }
        # NaCO3- log K(T): reference log K=1.27; delta H=8.91 kcal/mol.
        def sodium_k(t):
            return 1.27+(8.91*4184/(8.31446261815324*math.log(10)))*(1/298.15-1/t)
        equations["NaCO3"] = (quotient({"NaCO3-": 1, "Na+": -1, "CO3-2": -1}),
                              (sodium_k(temperature-TEMPERATURE_UNCERTAINTY_K),
                               sodium_k(temperature+TEMPERATURE_UNCERTAINTY_K)))
        for name, (observed, reference) in equations.items():
            require(observed[0] <= reference[1] and reference[0] <= observed[1],
                    f"mass action interval excludes reference: {name}")
        evidence.append({"row": index, "physical_ledger_mol": ledger,
                         "physical_charge_eq": physical_charge,
                         "provenance": provenance,
                         "scope_serialization": solution.get("scope", "omitted Complete variant"),
                         "charge_interval_molal": charge, "log_equations": equations})
    return {"model": MODEL, "dose_mol": expected_dose, "rows": evidence}
