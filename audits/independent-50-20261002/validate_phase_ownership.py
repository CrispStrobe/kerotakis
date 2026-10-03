"""Run the selected audit serially on a hosted CI runner; retain every result.

Local builds use the resource guard on /mnt/storage instead. CI has an
isolated runner and no concurrent Cargo jobs in this audit.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[1]
PREDICTIONS_SHA = "0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064"
CORE_TESTS = """transport transport_verb prepared_kitchen_objects
material_object_state kitchen_biology transfer_trace interface_conservation
solver_transactions conservation phase_heat_capacity pressure_boiling states mix
selected_phase_transfer phase_routes distil atomic_distil_refusal layers
liquid_extraction vegetable_oil_layers phase_without_aqueous thermal_phase_refusal thermal_phase_budget phase_thaw_recovery energy_transfer heat_integral_precision electrolysis_scope solvent_electrolysis_accounting chromatography_coverage chromatograph headspace density_coverage_scaling""".split()
SOURCE_FILES = [
    "crates/kerotakis-cea/src/gibbs.rs",
    "crates/kerotakis-core/src/bench.rs",
    "crates/kerotakis-core/src/transport.rs",
    "crates/kerotakis-core/tests/transport.rs",
    "crates/kerotakis-core/tests/selected_phase_transfer.rs",
    "crates/kerotakis-core/tests/thermal_phase_refusal.rs",
    "crates/kerotakis-core/tests/thermal_phase_budget.rs",
    "crates/kerotakis-core/tests/phase_thaw_recovery.rs",
    "crates/kerotakis-core/tests/energy_transfer.rs",
    "crates/kerotakis-core/src/solve.rs",
    "crates/kerotakis-core/src/heat_capacity.rs",
    "crates/kerotakis-core/tests/heat_integral_precision.rs",
    "audits/thermal-contracts-20261003/integral-references.json",
    "audits/thermal-contracts-20261003/generate_integral_references.py",
    "data/registry/registry-source-v1.json",
    "crates/kerotakis-core/tests/electrolysis_scope.rs",
    "crates/kerotakis-core/src/displacement.rs",
    "crates/kerotakis-core/src/chromatography.rs",
    "crates/kerotakis-core/src/lib.rs",
    "crates/kerotakis-core/src/ops.rs",
    "crates/kerotakis-core/tests/chromatography_coverage.rs",
    "crates/kerotakis-core/tests/solvent_electrolysis_accounting.rs",
    "crates/kerotakis-core/src/render.rs",
    "crates/kerotakis-core/src/buoyancy.rs",
    "crates/kerotakis-core/tests/headspace.rs",
    "crates/kerotakis-core/tests/density_coverage_scaling.rs",
    "crates/kerotakis-phreeqc/src/aqueous.rs",
    "crates/kerotakis-phreeqc/src/inventory.rs",
    "crates/kerotakis-phreeqc/tests/settled_carbonate_transfer.rs",
    "crates/kerotakis-phreeqc/tests/trace_inventory.rs",
    "crates/kerotakis-core/tests/atomic_distil_refusal.rs",
    "crates/kerotakis-thermo/src/vle.rs",
    "crates/kerotakis-thermo/tests/distillation_completion.rs",
    "crates/kerotakis-thermo/tests/distillation_completion.rs",
    ".github/workflows/chemistry-audit.yml",
    "audits/independent-50-20261002/validate_phase_ownership.py",
]


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    if os.environ.get("GITHUB_ACTIONS") != "true":
        parser.error("Use the local resource guard; this entry point is for hosted CI")
    assert digest(ROOT / "predictions.json") == PREDICTIONS_SHA
    env = dict(os.environ, RUSTC_WRAPPER="", CARGO_BUILD_JOBS="1", KERO_WORKERS="1")
    receipt = dict(
        commit=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        started_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        source_hashes={name: digest(REPO / name) for name in SOURCE_FILES},
        predictions_sha256=PREDICTIONS_SHA,
        rustc=subprocess.check_output(["rustc", "--version"], text=True).strip(),
        platform=sys.platform,
        cargo_jobs=1,
        test_threads=1,
        cli_workers=1,
        stages=[],
    )

    def run(name, command):
        started = time.monotonic()
        log = out / f"{name}.log"
        print(f"Starting {name}", flush=True)
        with log.open("w") as stream:
            result = subprocess.run(command, cwd=REPO, env=env, stdout=stream, stderr=subprocess.STDOUT)
        text = log.read_text()
        stage = dict(name=name, command=command, exit_code=result.returncode,
                     seconds=round(time.monotonic() - started, 3), log=log.name,
                     log_sha256=digest(log))
        stage["passed_test_counts"] = [int(n) for n in re.findall(r"test result: ok\. (\d+) passed", text)]
        stage["failed_test_counts"] = [int(n) for n in re.findall(r"test result: FAILED\. \d+ passed; (\d+) failed", text)]
        receipt["stages"].append(stage)
        (out / "validation.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(f"Finished {name}: {result.returncode}", flush=True)
        return result.returncode == 0

    tail = ["-j1", "--", "--test-threads=1"]
    run("thermo", ["cargo", "test", "--no-fail-fast", "-p", "kerotakis-thermo", "--lib", "--tests"] + tail)
    run("codex", ["cargo", "test", "--no-fail-fast", "-p", "kerotakis-codex", "--test", "vertical_slice", "--test", "quest_engine"] + tail)
    run("cea", ["cargo", "test", "--no-fail-fast", "-p", "kerotakis-cea", "--lib", "--tests"] + tail)
    core = ["cargo", "test", "--no-fail-fast", "-p", "kerotakis-core", "--lib"]
    for name in CORE_TESTS:
        core.extend(["--test", name])
    run("core", core + tail)
    run("phreeqc", ["cargo", "test", "--no-fail-fast", "-p", "kerotakis-phreeqc",
                   "--lib", "--test", "exchange_transport", "--test", "surface_transport", "--test", "chromatograph",
                   "--test", "settled_carbonate_transfer", "--test", "trace_inventory",
                   "--test", "order_invariance", "--test", "native_delta_h", "--test", "engine_call_budget"] + tail)
    cli_ok = run("cli", ["cargo", "test", "--no-fail-fast", "-p", "kerotakis-cli", "--bin", "kero",
                         "--test", "thermal_contracts", "--test", "provenance", "--test", "observable_coverage",
                         "--test", "headspace_json", "--test", "json_contract"] + tail)
    run("wasm", ["cargo", "check", "-p", "kerotakis-wasm", "--target", "wasm32-unknown-unknown", "-j1"])
    # This workspace intentionally ignores Cargo.lock. Preserve the exact
    # resolved dependency graph rather than claim it was present at checkout.
    lock = REPO / "Cargo.lock"
    if lock.exists():
        shutil.copyfile(lock, out / "Cargo.lock")
        receipt["cargo_lock_sha256"] = digest(lock)
    if cli_ok:
        binary = REPO / "target/debug/kero"
        receipt["binary_sha256"] = digest(binary)
        shutil.copyfile(binary, out / "kero")
        env.update(KERO_BIN=str(binary), KERO_RESULTS_DIR=str(out / "replay"))
        if run("replay", [sys.executable, str(ROOT / "run.py")]):
            followups = []
            for ident in ["07", "15"]:
                for mode in ["json", "text"]:
                    command = [str(binary), "run", str(ROOT / "followups" / f"{ident}-name-corrected.lab")]
                    if mode == "json":
                        command.append("--json")
                    result = subprocess.run(command, cwd=REPO, env=env, capture_output=True, timeout=90)
                    for label, data in [("stdout", result.stdout), ("stderr", result.stderr)]:
                        (out / "replay" / f"{ident}.corrected.{mode}.{label}").write_bytes(data)
                    followups.append(dict(id=ident, mode=mode, exit_code=result.returncode))
            (out / "replay/explicit-sucrose-execution.json").write_text(json.dumps(followups, indent=2) + "\n")
            receipt["explicit_sucrose"] = followups
            run("assessment", [sys.executable, str(ROOT / "verify_extended.py")])
    receipt["total_rust_tests_passed"] = sum(sum(s["passed_test_counts"]) for s in receipt["stages"])
    receipt["passed"] = all(s["exit_code"] == 0 for s in receipt["stages"]) and cli_ok and all(
        r["exit_code"] == 0 for r in receipt.get("explicit_sucrose", []))
    receipt["completed_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    (out / "validation.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
