"""Reassess saved native observations; never launches or substitutes an engine."""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent


def assess():
    design = json.loads((HERE / "frozen-design.json").read_text())
    execution = json.loads((HERE / "execution.json").read_text())
    raw = (HERE / "stdout.jsonl").read_bytes()
    assert hashlib.sha256(raw).hexdigest() == execution["stdout_sha256"]
    assert execution["engine"] == design["engine"]
    assert execution["script_sha256"] == design["script_sha256"]
    assert hashlib.sha256(design["script"].encode()).hexdigest() == design["script_sha256"]
    rows = [json.loads(line) for line in raw.splitlines()]
    assert len(rows) == execution["observations"] == 9
    inspections = [row for row in rows if row["operator"]["op"] == "inspect"]
    explanations = [row for row in rows if row["operator"]["op"] == "explain"]
    assert len(inspections) == 3 and len(explanations) == 2
    states = []
    for row in inspections:
        vessel = row["bench"]["vessels"][0]
        contents = vessel["contents"]
        assert all(p["species"] in {"Fe+2", "Fe+3", "Fe(OH)2", "Fe(OH)3"}
                   for p in contents if p["species"].startswith("Fe"))
        def amount(species, phase):
            return sum(p["moles"] for p in contents
                       if p["species"] == species and p["phase"] == phase)
        states.append({
            "elapsed_s": vessel["elapsed_seconds"],
            "ph": vessel["solution"]["ph"],
            "temperature_K": vessel["temperature"],
            "boundary": vessel["headspace"],
            "excess_enthalpy_j": vessel["excess_enthalpy_j"],
            "solute_charge": vessel["solute_charge"],
            "FeOH2_mol": amount("Fe(OH)2", "solid"),
            "ferric_mol": amount("Fe+3", "aqueous") + amount("Fe(OH)3", "solid"),
            "represented_Fe_mol": sum(p["moles"] for p in contents if p["species"].startswith("Fe")),
            "water_mol": amount("water", "liquid"),
            "base_equivalents_mol": amount("base_equivalents", "aqueous"),
        })
    comparisons = []
    for row, state in zip(explanations, states[1:]):
        lines = [line.strip() for line in row["text"].splitlines() if " pH " in line]
        same = [line for line in lines if line.startswith("wateq4f.dat ")]
        assert len(same) == 1
        ph = float(re.search(r" pH ([0-9.]+)", same[0])[1])
        solid = float(re.search(r"Fe\(OH\)2 ([0-9.]+) mol", same[0])[1])
        comparisons.append({"lines": lines, "same_dataset_ph_residual": abs(ph - state["ph"]),
                            "same_dataset_displayed_FeOH2_mol": solid})
    def span(key):
        return max(s[key] for s in states) - min(s[key] for s in states)
    limits = design["repair_acceptance"]
    metrics = {
        "max_FeOH2_difference_mol": span("FeOH2_mol"),
        "max_ph_difference": span("ph"),
        "represented_iron_max_residual_mol": max(abs(s["represented_Fe_mol"] - limits["represented_iron_each_state_expected_mol"]) for s in states),
        "max_ferric_mol": max(s["ferric_mol"] for s in states),
        "solver_failed_events": sum(e["event"] == "solver_failed" for r in rows for e in r["events"]),
        "water_span_mol": span("water_mol"),
        "charge_baseline_span_mol": span("solute_charge"),
        "base_equivalent_span_mol": span("base_equivalents_mol"),
    }
    criteria = {
        "completed": execution["status"] == "completed" and execution["exit_code"] == 0,
        "retained_solid": min(s["FeOH2_mol"] for s in states) >= limits["positive_solid_after_dose_and_both_waits_min_mol"],
        "solid_invariance": metrics["max_FeOH2_difference_mol"] <= limits["solid_between_unchanged_waits_max_difference_mol"],
        "ph_invariance": metrics["max_ph_difference"] <= limits["ph_between_unchanged_waits_max_difference"],
        "represented_iron": metrics["represented_iron_max_residual_mol"] <= limits["represented_iron_max_difference_mol"],
        "no_artificial_ferric_inventory": metrics["max_ferric_mol"] <= limits["ferric_inventory_without_partner_max_mol"],
        "same_dataset_ph": all(c["same_dataset_ph_residual"] <= limits["same_dataset_explain_inspect_ph_max_difference_including_display_rounding"] for c in comparisons),
        "no_solver_failure": metrics["solver_failed_events"] == limits["solver_failed_events"],
    }
    assert [s["elapsed_s"] for s in states] == [0, 1, 2]
    assert all(s["temperature_K"] == 298.15 and s["boundary"] == {"boundary": "open"}
               and s["excess_enthalpy_j"] == 0 for s in states)
    return {"status": "PASS" if all(criteria.values()) else "FAIL", "criteria": criteria,
            "metrics": metrics, "states": states, "comparisons": comparisons,
            "execution": execution, "original_first_results_unchanged": True,
            "qualification": "Bounded regression consistency only. Not complete H/O/charge/energy closure, empirical calibration, REST/deployment acceptance, full-fleet acceptance or a catalog selection.",
            "runtime_metadata_qualification": "Raw manifest build_image/rustc/target are inherited snapshot metadata, not attested producer build metadata. Binary and actual producer lock bytes, tested merge/tree and replacement runtime-library hashes are separately bound. SDK destruction returned; independent deletion is not attested."}


if __name__ == "__main__":
    print(json.dumps(assess(), indent=2))
