#!/usr/bin/env python3
"""Replay two frozen alias controls using the verified own CLI, without rebuilding."""
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
    assert not sys.flags.optimize
    assert os.environ["GITHUB_ACTIONS"] == "true"
    assert os.environ["GITHUB_EVENT_NAME"] == "workflow_dispatch"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == os.environ["GITHUB_SHA"]
    root = Path(__file__).resolve().parent
    design = root / "water-alias-cli"
    protocol = strict((design / "protocol.json").read_text())
    review_path = root / "water-alias-artifact-review.json"
    assert sha(review_path) == protocol["artifact_review_sha256"]
    review = strict(review_path.read_text())
    assert review["all_checks_pass"] is True
    for name, digest in protocol["files_sha256"].items():
        assert sha(design / name) == digest
    assert protocol["source_head"] == "e4fb5e33434e9c7d76c0846d9ac995bbf11349f9"
    out = Path("water-alias-cli-evidence")
    out.mkdir(exist_ok=False)
    shutil.copy2(design / "protocol.json", out / "protocol.json")
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
    runtime = fast / "water-alias-runtime"
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
    binding = {"protocol_sha256": sha(design / "protocol.json"), "helper_sha256": sha(Path(__file__)),
               "audit_checkout": os.environ["GITHUB_SHA"], "source_head": protocol["source_head"],
               "binary_sha256": sha(binary), "lock_sha256": sha(runtime / "Cargo.lock"),
               "artifact_id": artifact["id"], "artifact_zip_sha256": sha(archive)}
    (out / "binding.json").write_text(json.dumps(binding, indent=2) + "\n")
    env = dict(os.environ)
    env.pop("GH_TOKEN", None)
    env.pop("GITHUB_TOKEN", None)
    env["KERO_LANG"] = "en"
    streams = []
    for ordinal, name in enumerate(["canonical", "mixed"], 1):
        gate(name)
        cwd = runtime / name
        cwd.mkdir(exist_ok=False)
        shutil.copy2(design / f"scripts/{name}.lab", cwd / "case.lab")
        assert sha(cwd / "case.lab") == protocol["files_sha256"][f"scripts/{name}.lab"]
        started = time.monotonic()
        timeout = False
        try:
            result = subprocess.run([str(binary), "run", "case.lab", "--json"], cwd=cwd, env=env,
                                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
            stdout, stderr, code = result.stdout, result.stderr, result.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr, code, timeout = error.stdout or b"", error.stderr or b"", None, True
        (out / f"{ordinal:02}.stdout.jsonl").write_bytes(stdout)
        (out / f"{ordinal:02}.stderr.txt").write_bytes(stderr)
        record = dict(binding, name=name, script_sha256=sha(cwd / "case.lab"), exit=code, timeout=timeout,
                      elapsed_seconds=time.monotonic() - started,
                      stdout_sha256=hashlib.sha256(stdout).hexdigest(), stderr_sha256=hashlib.sha256(stderr).hexdigest())
        (out / f"{ordinal:02}.result.json").write_text(json.dumps(record, indent=2) + "\n")
        streams.append((record, stdout, stderr))
    cases = []
    semantic = []
    for record, stdout, stderr in streams:
        rows = [strict(v) for v in stdout.decode().splitlines()]
        adds = [v for v in rows if v["operator"]["op"] == "add"]
        assert len(adds) == 3, "all additions must have recorded rows"
        vessel = lambda row: next(v for v in row["bench"]["vessels"] if v["id"] == 0)
        final = vessel(rows[-1])
        water = [v for v in final["contents"] if v["species"] == "water"]
        stock = rows[-1]["bench"]["stock"]
        checks = {"exit_zero": record["exit"] == 0 and not record["timeout"], "stderr_empty": not stderr,
                  "final_water_moles": sum(v["moles"] for v in water) == 0.5,
                  "canonical_inventory": bool(water) and not any(v["species"] == "H2O" for v in final["contents"]),
                  "one_water_bottle": set(stock) == {"water"},
                  "final_stock_water_moles": stock.get("water", {}).get("amount") == 0.5 and stock.get("water", {}).get("unit") == "mole",
                  "overdraw_named": any(v["event"] == "stock_exhausted" and v["key"] == "water" for v in adds[2]["events"]),
                  "overdraw_inventory_unchanged": vessel(adds[1]) == vessel(adds[2])}
        cases.append({"name": record["name"], "checks": checks, "pass": all(checks.values()), "record": record})
        semantic.append([{key: row[key] for key in ["operator", "events", "bench"]} for row in rows])
    report = {"binding": binding, "cases": cases, "semantic_rows_equal": semantic[0] == semantic[1],
              "all_pass": all(v["pass"] for v in cases) and semantic[0] == semantic[1],
              "candidate_integrated": False, "empirical_calibration": False}
    (out / "report.json").write_text(json.dumps(report, indent=2, allow_nan=False) + "\n")
    print("WATER_ALIAS_CLI_JSON=" + json.dumps(dict(report, report_sha256=sha(out / "report.json")), allow_nan=False))
    assert report["all_pass"], "first CLI acceptance failure preserved"


if __name__ == "__main__":
    main()
