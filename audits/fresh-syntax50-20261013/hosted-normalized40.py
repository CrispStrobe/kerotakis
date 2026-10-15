#!/usr/bin/env python3
"""Execute the original frozen forty naming controls using their verified old-source CLI."""
import base64
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import time
import zipfile


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pairs(items):
    result = {}
    for key, value in items:
        assert key not in result, "duplicate JSON key"
        result[key] = value
    return result


def finite(value):
    number = float(value)
    assert math.isfinite(number), "nonfinite JSON number"
    return number


def strict(text):
    def constant(value):
        raise ValueError("nonfinite JSON constant: " + value)
    return json.loads(text, object_pairs_hook=pairs, parse_float=finite, parse_constant=constant)


def main():
    if sys.flags.optimize:
        raise SystemExit("optimized Python is unsupported")
    assert os.environ["GITHUB_ACTIONS"] == "true"
    assert os.environ["GITHUB_EVENT_NAME"] == "workflow_dispatch"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == os.environ["GITHUB_SHA"]
    root = Path(__file__).resolve().parent
    design = root / "normalized40"
    protocol = strict((root / "normalized40-hosted/protocol.json").read_text())
    review_path = root.parent / "release-performance-20261013/candidate-artifact-independent-review-v2.json"
    assert sha(review_path) == protocol["artifact_review_sha256"]
    review = strict(review_path.read_text())
    assert review["all_checks_pass"] is True
    for name, digest in protocol["files_sha256"].items():
        assert sha(root / name) == digest
    assert protocol["source_head"] == "d06ffe3a1740728656878c4aea0e9507ca83c808"
    out = Path("normalized40-evidence")
    out.mkdir(exist_ok=False)
    shutil.copy2(root / "normalized40-hosted/protocol.json", out / "protocol.json")
    fast = Path(os.environ["RUNNER_TEMP"])

    def gate(stage):
        mem = int(next(v for v in Path("/proc/meminfo").read_text().splitlines() if v.startswith("MemAvailable:")).split()[1]) // 1024
        r = {"stage": stage, "load1": os.getloadavg()[0], "mem_available_mib": mem,
             "fast_free_mib": shutil.disk_usage(fast).free // 2**20,
             "storage_free_mib": shutil.disk_usage(out).free // 2**20}
        limits = protocol["resource_thresholds"]
        r["permitted"] = (r["load1"] <= limits["load1_max"] and mem >= limits["mem_available_mib_min"]
                          and r["fast_free_mib"] >= limits["fast_free_mib_min"]
                          and r["storage_free_mib"] >= limits["storage_free_mib_min"])
        with (out / "resource-journal.jsonl").open("a") as f:
            f.write(json.dumps(r) + "\n")
        assert r["permitted"], "worker resource gate closed"

    flags = ["--allow-escape-sequences"] if "--allow-escape-sequences" in subprocess.check_output(["gh", "api", "--help"], text=True) else []

    def api(name, endpoint):
        gate(name)
        path = out / name
        with path.open("xb") as stdout, path.with_suffix(path.suffix + ".stderr").open("xb") as stderr:
            subprocess.run(["gh", "api", *flags, "repos/CrispStrobe/kerotakis/" + endpoint],
                           stdout=stdout, stderr=stderr, check=True, timeout=60)
        return path

    run = strict(api("run.json", f"actions/runs/{protocol['own_run']}").read_text())
    job = strict(api("job.json", f"actions/jobs/{protocol['build_job']}").read_text())
    assert run["head_sha"] == protocol["source_head"] and run["event"] == "pull_request" and run["run_attempt"] == 1
    assert job["run_id"] == protocol["own_run"] and job["status"] == "completed" and job["conclusion"] == "success"
    assert all(v["status"] == "completed" and v["conclusion"] == "success" for v in job["steps"])
    artifact = strict(api("artifact.json", f"actions/artifacts/{protocol['artifact_id']}").read_text())
    assert artifact["id"] == protocol["artifact_id"] and not artifact["expired"]
    assert artifact["name"] == "chemistry-audit-cli-" + review["tested_checkout"]
    assert artifact["workflow_run"]["id"] == protocol["own_run"] and artifact["workflow_run"]["head_sha"] == protocol["source_head"]
    assert artifact["digest"] == "sha256:" + protocol["artifact_zip_sha256"]
    tested = strict(api("tested.json", "git/commits/" + review["tested_checkout"]).read_text())
    source = strict(api("source.json", "git/commits/" + protocol["source_head"]).read_text())
    assert [v["sha"] for v in tested["parents"]] == review["tested_parents"]
    assert tested["tree"]["sha"] == source["tree"]["sha"] == protocol["source_tree"]
    workflow = strict(api("source-workflow.json", "contents/.github/workflows/chemistry-audit.yml?ref=" + protocol["source_head"]).read_text())
    assert hashlib.sha256(base64.b64decode(workflow["content"])).hexdigest() == "57252f0fb26ac203b5420e82a3e1355a20ec0ea6ac925d7078b95bcaf75e1304"
    log = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", api("build.log", f"actions/jobs/{protocol['build_job']}/logs").read_text())
    assert "git log -1 --format=%H" in log and any(v.split() and v.split()[-1] == tested["sha"] for v in log.splitlines())
    assert "cargo build --release -p kerotakis-cli" in log and "name: chemistry-audit-cli-" + tested["sha"] in log
    archive = api("artifact.zip", f"actions/artifacts/{protocol['artifact_id']}/zip")
    assert archive.stat().st_size == artifact["size_in_bytes"] and sha(archive) == protocol["artifact_zip_sha256"]
    gate("archive")
    runtime = fast / "normalized40-runtime"
    runtime.mkdir(exist_ok=False)
    with zipfile.ZipFile(archive) as z:
        infos = z.infolist()
        assert len(infos) == 4 and {v.filename for v in infos} == {"target/release/kero", "Cargo.lock", "binary-sha256.txt", "submodules.txt"}
        assert sum(v.file_size for v in infos) < 32 * 2**20
        assert all(not v.is_dir() and not stat.S_ISLNK(v.external_attr >> 16) for v in infos) and z.testzip() is None
        for info in infos:
            (runtime / Path(info.filename).name).write_bytes(z.read(info))
    binary = runtime / "kero"
    assert sha(binary) == protocol["binary_sha256"] and sha(runtime / "Cargo.lock") == protocol["lock_sha256"]
    elf = binary.read_bytes()
    assert elf[:6] == b"\x7fELF\x02\x01" and int.from_bytes(elf[18:20], "little") == 62
    assert (runtime / "binary-sha256.txt").read_text().split() == [sha(binary), "target/release/kero"]
    modules = (runtime / "submodules.txt").read_text().splitlines()
    assert all(v.startswith(" ") for v in modules)
    assert {v.split()[1]: v.split()[0] for v in modules} == {"sundials-kinetics-rs": "8f2e16e6d6f00926ea28cda9b6ccb3cf2538b5fa", "vendor/iphreeqc": "47b459c4a5e3e20b9990fa304e207eca0dcc6ff1"}
    binary.chmod(0o500)
    shutil.copy2(runtime / "Cargo.lock", out / "Cargo.lock")
    binding = {"protocol_sha256": sha(root / "normalized40-hosted/protocol.json"), "helper_sha256": sha(Path(__file__)),
               "audit_checkout": os.environ["GITHUB_SHA"], "source_head": protocol["source_head"],
               "binary_sha256": sha(binary), "lock_sha256": sha(runtime / "Cargo.lock"),
               "artifact_id": artifact["id"], "artifact_zip_sha256": sha(archive)}
    (out / "binding.json").write_text(json.dumps(binding, indent=2) + "\n")
    env = dict(os.environ)
    env.pop("GH_TOKEN", None)
    env.pop("GITHUB_TOKEN", None)
    env["KERO_LANG"] = "en"
    gate("frozen-forty")
    runner = design / "run-frozen.py"
    assert sha(design / "execution-binding.json") == protocol["original_execution_binding_sha256"]
    assert strict((design / "execution-binding.json").read_text())["experiment_count"] == 40
    result = subprocess.run([sys.executable, str(runner), "--design", str(design / "design"),
        "--binary", str(binary), "--binary-sha256", protocol["binary_sha256"],
        "--out", str(out / "cases"), "--fast", str(fast), "--storage", str(out)], env=env, timeout=2700)
    if result.returncode:
        raise SystemExit(result.returncode)
    completed = strict((out / "cases/completed.json").read_text())
    assert completed["cases"] == 40
    records = [strict(p.read_text()) for p in sorted((out / "cases").glob("*.result.json"))]
    assert len(records) == 40 and not any(r["timeout"] for r in records)
    report = {"cases_executed": 40, "process_exit_codes": [r["exit_code"] for r in records],
        "binding": binding, "original_execution_binding_sha256": protocol["original_execution_binding_sha256"],
        "physical_verdicts_pending_saved_observation_review": True,
        "original_forecasts_changed": False, "new_blind_designs": False}
    (out / "execution-summary.json").write_text(json.dumps(report, indent=2) + "\n")
    print("NORMALIZED40_EXECUTION_JSON=" + json.dumps(report))


if __name__ == "__main__":
    main()
