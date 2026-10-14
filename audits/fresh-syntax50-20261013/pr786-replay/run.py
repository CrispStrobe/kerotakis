#!/usr/bin/env python3
"""Execute the frozen seven only with a verified own-artifact runtime binding."""
import argparse, datetime, hashlib, json, os, pathlib, shutil, signal, subprocess, sys, time
from analyse import sha, strict_json


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def gate(fast, storage):
    available = int(next(x.split()[1] for x in pathlib.Path("/proc/meminfo").read_text().splitlines() if x.startswith("MemAvailable:"))) // 1024
    r = {"utc": now(), "load1": os.getloadavg()[0], "memory_available_mib": available,
         "fast_free_mib": shutil.disk_usage(fast).free // 1048576,
         "storage_free_mib": shutil.disk_usage(storage).free // 1048576}
    r["permitted"] = r["load1"] <= 3.5 and available >= 3200 and r["fast_free_mib"] >= 4224 and r["storage_free_mib"] >= 8192
    return r


def main():
    if sys.flags.optimize:
        raise SystemExit("optimized Python would disable required checks")
    ap = argparse.ArgumentParser()
    ap.add_argument("--binary", type=pathlib.Path, required=True)
    ap.add_argument("--lock", type=pathlib.Path, required=True)
    ap.add_argument("--binding", type=pathlib.Path, required=True)
    ap.add_argument("--artifact-review", type=pathlib.Path, required=True)
    ap.add_argument("--out", type=pathlib.Path, required=True)
    ap.add_argument("--fast", type=pathlib.Path, required=True)
    ap.add_argument("--storage", type=pathlib.Path, required=True)
    a = ap.parse_args()
    root = pathlib.Path(__file__).resolve().parent
    protocol = strict_json((root / "protocol.json").read_text())
    for name, digest in protocol["files"].items():
        assert sha(root / name) == digest, "frozen inputs changed"
    binding = strict_json(a.binding.read_text())
    assert binding["protocol_sha256"] == sha(root / "protocol.json")
    assert binding["runner_sha256"] == sha(pathlib.Path(__file__))
    assert binding["analyzer_sha256"] == sha(root / "analyse.py")
    assert sha(a.artifact_review) == binding["artifact_review_receipt_sha256"]
    review = strict_json(a.artifact_review.read_text())
    assert review["all_checks_pass"] is True
    assert review["candidate"]["head"] == binding["source_head"] == protocol["source_head"]
    assert review["candidate"]["tree"] == binding["source_tree"] == protocol["source_tree"]
    assert review["run"] == protocol["own_cli_workflow_run"]
    assert review["build_job"] == protocol["own_cli_build_job"]
    assert review["tested_parents"] == [protocol["base"], protocol["source_head"]]
    assert review["candidate"]["binary_sha256"] == binding["binary_sha256"]
    assert review["candidate"]["lock_sha256"] == binding["lock_sha256"]
    assert review["artifact_id"] == binding["artifact_id"]
    assert review["artifact_zip_sha256"] == binding["artifact_zip_sha256"]
    a.out.mkdir(exist_ok=True)
    assert not (a.out / "completed.json").exists(), "completed execution must not repeat"
    def journal(case=None):
        r = gate(a.fast, a.storage)
        with (a.out / "resource-journal.jsonl").open("a") as f:
            f.write(json.dumps(dict(r, case=case)) + "\n")
        if not r["permitted"]:
            raise SystemExit("resource gate closed; untouched controls remain pending")
    journal()
    assert sha(a.binary) == binding["binary_sha256"]
    assert sha(a.lock) == binding["lock_sha256"]
    if (a.out / "binding.json").exists():
        assert strict_json((a.out / "binding.json").read_text()) == binding, "runtime binding changed"
    else:
        (a.out / "binding.json").write_bytes(a.binding.read_bytes())
        (a.out / "artifact-review.json").write_bytes(a.artifact_review.read_bytes())
    assert sha(a.out / "artifact-review.json") == binding["artifact_review_receipt_sha256"]
    cases = sorted(name for name in protocol["files"] if name.endswith(".lab"))
    assert len(cases) == 7
    for i, case in enumerate(cases, 1):
        stem = f"{i:02d}"
        record_path = a.out / (stem + ".result.json")
        stdout = a.out / (stem + ".stdout.jsonl"); stderr = a.out / (stem + ".stderr.txt")
        if record_path.exists():
            old = strict_json(record_path.read_text())
            assert old["case"] == case and old["script_sha256"] == protocol["files"][case]
            assert sha(stdout) == old["stdout_sha256"] and sha(stderr) == old["stderr_sha256"]
            assert not old["timeout"], "timeout needs separate review before continuation"
            continue
        assert not stdout.exists() and not stderr.exists(), "incomplete streams must never be overwritten"
        journal(case)
        assert sha(root / case) == protocol["files"][case]
        start = time.monotonic(); started = now(); timeout = False
        env = {k: v for k, v in os.environ.items() if not k.startswith("KERO_")}
        env.update(KERO_LANG="en", LANG="C.UTF-8")
        with stdout.open("xb") as out, stderr.open("xb") as err:
            process = subprocess.Popen([str(a.binary.resolve()), "run", case, "--json"],
                stdout=out, stderr=err, cwd=root, env=env, start_new_session=True)
            try:
                code = process.wait(timeout=60)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL); code = process.wait(); timeout = True
        record = {"case": case, "ordinal": i, "started_utc": started,
                  "elapsed_seconds": time.monotonic() - start, "exit_code": code, "timeout": timeout,
                  "script_sha256": protocol["files"][case], "stdout_sha256": sha(stdout),
                  "stderr_sha256": sha(stderr), "stdout_bytes": stdout.stat().st_size,
                  "stderr_bytes": stderr.stat().st_size}
        with record_path.open("x") as f:
            f.write(json.dumps(record, indent=2) + "\n")
        print(json.dumps({"case": case, "exit_code": code, "timeout": timeout}), flush=True)
        if timeout:
            raise SystemExit("timeout preserved; review before any continuation")
    with (a.out / "completed.json").open("x") as f:
        f.write(json.dumps({"completed_utc": now(), "cases": 7, "binding_sha256": sha(a.out / "binding.json")}, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
