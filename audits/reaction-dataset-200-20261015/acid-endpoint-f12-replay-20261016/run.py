#!/usr/bin/env python3
"""Authenticate one existing producer executable, then collect unchanged F12 once.

Validation-only harness. Never deploys, rebuilds or assigns a chemical verdict.
"""
import argparse
import base64
import datetime
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import resource
import signal
import subprocess
import sys
import time
import zipfile

HERE = Path(__file__).resolve().parent
REPOSITORY = "CrispStrobe/kerotakis"
WORKFLOW = ".github/workflows/vercel.yml"
BRANCH = "validation/acid-stock-replay-20261016"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def save(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, allow_nan=False, ensure_ascii=False)
        stream.write("\n")


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def api(path, destination):
    # Only this parent/controller receives GH_TOKEN. No token is written to
    # argv, receipts or the executed CLI's deliberately minimal environment.
    with destination.open("xb") as output, destination.with_suffix(".stderr.txt").open("xb") as errors:
        subprocess.run(["gh", "api", "--method", "GET", path], stdout=output,
                       stderr=errors, check=True, timeout=90)
    return json.loads(destination.read_text())


def freeze():
    design = json.loads((HERE / "design.json").read_text())
    assert digest(HERE / "run.py") == design["runner_sha256"]
    root = HERE.parents[2]
    assert digest(root / WORKFLOW) == design["isolated_workflow_sha256"]
    original = design["original"]
    assert digest(HERE / original["retained_copy"]) == original["sha256"]
    saved = json.loads((HERE / original["retained_copy"]).read_text())
    cases = [c for c in saved["cases"] if c["id"] == "F12-permanganate-peroxide-acid"]
    assert cases == [original["case"]]
    assert cases[0]["script"].encode() == (HERE / "input.lab").read_bytes()
    assert digest(HERE / "input.lab") == cases[0]["script_sha256"] == design["execution"]["script_sha256"]
    assert cases[0]["expected_before_execution"] == design["assessment"]["expected_before_execution"]
    assert design["execution"]["processes"] == 1 and design["execution"]["retries"] == 0
    return design, root


def authenticate(design, root, out):
    metadata = out / "metadata"
    metadata.mkdir()
    pin = design["producer"]
    assert pin["repository"] == REPOSITORY
    harness = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    assert os.environ["GITHUB_SHA"] == harness
    assert os.environ["GITHUB_REPOSITORY"] == REPOSITORY
    assert os.environ["GITHUB_REF"] == "refs/heads/" + BRANCH
    assert os.environ["GITHUB_RUN_ATTEMPT"] == "1", "reruns cannot repeat the unchanged F12 process"
    assert not subprocess.check_output(["git", "status", "--porcelain"], cwd=root), "harness checkout must be clean"
    current_id = int(os.environ["GITHUB_RUN_ID"])
    current = api(f"repos/{REPOSITORY}/actions/runs/{current_id}", metadata / "replay-run.json")
    assert current["head_sha"] == harness and current["event"] == "workflow_dispatch"
    assert current["path"] == WORKFLOW and current["head_branch"] == BRANCH
    assert current["run_attempt"] == 1
    prior = api(f"repos/{REPOSITORY}/actions/workflows/vercel.yml/runs?branch=validation%2Facid-stock-replay-20261016&event=workflow_dispatch&per_page=100", metadata / "isolated-dispatches.json")
    assert prior["total_count"] == len(prior["workflow_runs"]) == 1
    assert prior["workflow_runs"][0]["id"] == current_id, "another dispatch already owns this frozen process"

    run_id = pin["run_id"]
    run = api(f"repos/{REPOSITORY}/actions/runs/{run_id}", metadata / "producer-run.json")
    assert run["id"] == run_id and run["head_sha"] == pin["source_head"]
    assert run["event"] == pin["event"] and run["path"] == pin["workflow_path"]
    assert run["run_attempt"] == pin["run_attempt"]
    assert run["repository"]["full_name"] == REPOSITORY
    assert run["head_repository"]["full_name"] == REPOSITORY
    # Full producer fleets may still be running. Only the completed successful
    # build job is a prerequisite for collecting this separate raw replay.
    jobs = api(f"repos/{REPOSITORY}/actions/runs/{run_id}/attempts/{pin['run_attempt']}/jobs?per_page=100", metadata / "producer-jobs.json")
    assert jobs["total_count"] == len(jobs["jobs"])
    build = [j for j in jobs["jobs"] if j["name"] == pin["job_name"]]
    assert len(build) == 1 and build[0]["status"] == "completed" and build[0]["conclusion"] == "success"
    assert build[0]["run_id"] == run_id
    assert all(s["conclusion"] == "success" for s in build[0]["steps"] if s["name"] not in ("Complete job",))
    artifacts = api(f"repos/{REPOSITORY}/actions/runs/{run_id}/artifacts?per_page=100", metadata / "producer-artifacts.json")
    assert artifacts["total_count"] == len(artifacts["artifacts"])
    matching = [a for a in artifacts["artifacts"] if a["name"].startswith(pin["artifact_prefix"])]
    assert len(matching) == 1, "require exactly one producer CLI archive"
    artifact = matching[0]
    assert artifact["expired"] is False
    assert artifact["workflow_run"]["id"] == run_id
    assert artifact["workflow_run"]["head_sha"] == pin["source_head"]
    tested = artifact["name"][len(pin["artifact_prefix"]):]
    assert re.fullmatch(r"[0-9a-f]{40}", tested)
    commit = api(f"repos/{REPOSITORY}/git/commits/{tested}", metadata / "tested-commit.json")
    assert commit["sha"] == tested and commit["tree"]["sha"] == pin["source_tree"]
    assert tested == pin["source_head"] or pin["source_head"] in [p["sha"] for p in commit["parents"]]
    source = api(f"repos/{REPOSITORY}/git/commits/{pin['source_head']}", metadata / "source-head.json")
    assert source["sha"] == pin["source_head"] and source["tree"]["sha"] == pin["source_tree"]
    workflow = api(f"repos/{REPOSITORY}/contents/{pin['workflow_path']}?ref={tested}", metadata / "producer-workflow.json")
    assert workflow["encoding"] == "base64"
    producer_bytes = base64.b64decode(workflow["content"])
    assert hashlib.sha256(producer_bytes).hexdigest() == pin["workflow_sha256"]
    (metadata / "producer-workflow.yml").write_bytes(producer_bytes)
    save(metadata / "selected-artifact.json", artifact)

    archive = out / "producer.zip"
    with archive.open("xb") as stream, (metadata / "download.stderr.txt").open("xb") as errors:
        subprocess.run(["gh", "api", f"repos/{REPOSITORY}/actions/artifacts/{artifact['id']}/zip"],
                       stdout=stream, stderr=errors, check=True, timeout=120)
    assert artifact["digest"] == "sha256:" + digest(archive)
    expected = {"target/release/kero", "Cargo.lock", "binary-sha256.txt", "submodules.txt"}
    bundle = out / "producer"
    bundle.mkdir()
    with zipfile.ZipFile(archive) as zipped:
        files = [i for i in zipped.infolist() if not i.is_dir()]
        assert len(files) == len(expected) and {i.filename for i in files} == expected
        assert sum(i.file_size for i in files) <= 1024**3
        for info in zipped.infolist():
            name = PurePosixPath(info.filename)
            assert not name.is_absolute() and ".." not in name.parts and "\\" not in info.filename
        # Extract only the four authenticated regular byte streams. Never
        # honor archive permissions, links or extra paths from the ZIP.
        for info in files:
            destination = bundle / info.filename
            destination.parent.mkdir(parents=True, exist_ok=True)
            with zipped.open(info) as source_stream, destination.open("xb") as destination_stream:
                while block := source_stream.read(1024 * 1024):
                    destination_stream.write(block)
    binary = bundle / "target/release/kero"
    recorded = (bundle / "binary-sha256.txt").read_text().split()
    assert len(recorded) == 2 and recorded[1] == "target/release/kero"
    assert recorded[0] == digest(binary)
    assert (bundle / "Cargo.lock").stat().st_size > 0
    binding = dict(schema=1, verified_utc=utc(), producer_run=run_id, producer_run_attempt=pin["run_attempt"],
                   producer_build_job=build[0]["id"], source_head=pin["source_head"], tested_source=tested,
                   source_tree=pin["source_tree"], artifact_id=artifact["id"], artifact_name=artifact["name"],
                   archive_sha256=digest(archive), binary_sha256=digest(binary),
                   generated_lock_sha256=digest(bundle / "Cargo.lock"), submodules_sha256=digest(bundle / "submodules.txt"),
                   harness_commit=harness, design_sha256=digest(HERE / "design.json"), runner_sha256=digest(HERE / "run.py"),
                   workflow_sha256=digest(root / WORKFLOW), script_sha256=digest(HERE / "input.lab"),
                   original_design_sha256=design["original"]["sha256"],
                   authenticated_metadata_sha256={p.name: digest(p) for p in sorted(metadata.iterdir()) if p.is_file()},
                   acceptance="artifact/build/source binding only; full producer fleets and chemical assessment are separate")
    save(out / "binding.json", binding)
    binary.chmod(0o500)
    return binary, binding


def collect(design, binary, out):
    limits = design["execution"]
    assert limits["cpu_seconds"] == 60 and limits["address_space_bytes"] == 1024**3
    assert limits["wall_seconds"] == 90
    child = out / "child"
    child.mkdir(mode=0o700)
    (child / "input.lab").write_bytes((HERE / "input.lab").read_bytes())
    scratch = child / "private"
    scratch.mkdir(mode=0o700)
    environment = {"PATH": "/usr/bin:/bin", "HOME": str(scratch), "TMPDIR": str(scratch),
                   "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "KERO_WORKERS": "1",
                   "RAYON_NUM_THREADS": "1", "OMP_NUM_THREADS": "1"}
    assert sorted(environment) == sorted(limits["minimal_child_env_keys"])
    assert not any("TOKEN" in key or "SECRET" in key for key in environment)

    def bounded():
        resource.setrlimit(resource.RLIMIT_CPU, (60, 60))
        resource.setrlimit(resource.RLIMIT_AS, (1024**3, 1024**3))
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

    started = utc()
    save(out / "execution-start.json", dict(started_utc=started, command=limits["command"], cwd="child",
          limits=limits, inherited_environment=False, physical_verdict="not_assigned"))
    wall_start = time.monotonic()
    with (out / "stdout.txt").open("xb") as stdout, (out / "stderr.txt").open("xb") as stderr:
        process = subprocess.Popen([str(binary.resolve()), "run", "input.lab", "--json"],
                                   cwd=child, env=environment, stdin=subprocess.DEVNULL,
                                   stdout=stdout, stderr=stderr, close_fds=True,
                                   start_new_session=True, preexec_fn=bounded)
        timed_out = False
        try:
            code = process.wait(timeout=90)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait()
    result = dict(schema=1, case_id="F12-permanganate-peroxide-acid", process_started=True,
                  started_utc=started, finished_utc=utc(), command=limits["command"],
                  exit_code=code, timeout=timed_out, wall_seconds=time.monotonic() - wall_start,
                  stdout_sha256=digest(out / "stdout.txt"), stderr_sha256=digest(out / "stderr.txt"),
                  input_sha256=digest(child / "input.lab"), binding_sha256=digest(out / "binding.json"),
                  limits=limits, execution_status="timeout" if timed_out else "exited",
                  expected_before_execution=design["assessment"]["expected_before_execution"],
                  physical_verdict="not_assigned", original_first_outcome="preserved unchanged")
    save(out / "process.json", result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(mode=0o700, parents=True, exist_ok=False)
    stage = "freeze"
    try:
        design, root = freeze()
        stage = "authenticate-before-execution"
        binary, _ = authenticate(design, root, args.out)
        stage = "one-bounded-unchanged-process"
        result = collect(design, binary, args.out)
        save(args.out / "receipt.json", dict(schema=1, stage="raw process collection complete",
             process_sha256=digest(args.out / "process.json"), physical_verdict="not_assigned",
             workflow_success_means="zero process exit and intact collection; no chemical verdict"))
        return 0 if result["exit_code"] == 0 and not result["timeout"] else 1
    except Exception as error:
        save(args.out / "failure.json", dict(schema=1, stage=stage, error_type=type(error).__name__,
             detail=str(error), recorded_utc=utc(), execution_start_record_present=(args.out / "execution-start.json").exists(),
             physical_verdict="not_assigned", policy="preserve this first failure; no automatic retry"))
        raise


if __name__ == "__main__":
    sys.exit(main())
