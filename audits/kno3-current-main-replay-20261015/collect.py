#!/usr/bin/env python3
"""Hosted-only source-pinned build and observation collector; no physical verdict."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

SOURCE = "d791d3062886e4a06dbc7c9d1635383da3e05359"


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1048576), b""):
            digest.update(chunk)
    return digest.hexdigest()


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def git(source, *args):
    return subprocess.check_output(["git", "-C", str(source), *args], text=True).strip()


def write(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    # Refuse accidental local invocation before creating any output.
    if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_OS") != "Linux":
        raise RuntimeError("hosted GitHub Actions Linux only")
    source = args.source.resolve()
    evidence = args.out.resolve()
    evidence.mkdir(parents=True, exist_ok=False)
    here = Path(__file__).resolve().parent
    records = []

    def gate(stage, disk_mib):
        available = int(next(line.split()[1] for line in Path("/proc/meminfo").read_text().splitlines()
                             if line.startswith("MemAvailable:"))) // 1024
        row = {"utc": utc(), "stage": stage, "load1": os.getloadavg()[0],
               "memory_available_mib": available,
               "source_free_mib": shutil.disk_usage(source).free // 1048576,
               "evidence_free_mib": shutil.disk_usage(evidence).free // 1048576}
        minimum_memory = 4736 if stage in ("prepare", "fetch-sundials", "resolve-lock", "release-build") else 3400
        row["minimum_memory_mib"] = minimum_memory
        row["minimum_disk_mib"] = disk_mib
        row["permitted"] = (row["load1"] <= 3.5 and available >= minimum_memory
                            and min(row["source_free_mib"], row["evidence_free_mib"]) >= disk_mib)
        with (evidence / "resources.jsonl").open("a") as stream:
            stream.write(json.dumps(row, allow_nan=False) + "\n")
        if not row["permitted"]:
            raise RuntimeError("resource gate closed: " + stage)

    def guard():
        head = git(source, "rev-parse", "HEAD")
        status = git(source, "status", "--porcelain", "--untracked-files=no")
        modules = subprocess.check_output(
            ["git", "-C", str(source), "submodule", "status", "--recursive"], text=True).rstrip("\n")
        if head != SOURCE or status or any(line[0] != " " for line in modules.splitlines()):
            raise RuntimeError("source identity/cleanliness/submodule guard failed")
        return {"head": head, "tree": git(source, "rev-parse", "HEAD^{tree}"),
                "tracked_status": status, "recursive_submodules": modules}

    def run(label, command, env, timeout, cwd=source):
        gate(label, 8192 if label in ("fetch-sundials", "resolve-lock", "release-build") else 1024)
        before = guard()
        started = utc()
        clock = time.monotonic()
        timed_out = False
        with (evidence / (label + ".stdout")).open("xb") as out, (evidence / (label + ".stderr")).open("xb") as err:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdout=out, stderr=err,
                                       start_new_session=True)
            try:
                code = process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                code = process.wait()
                timed_out = True
        row = {"stage": label, "argv": command, "cwd": "source" if cwd == source else "audit/design",
               "started_utc": started, "elapsed_seconds": time.monotonic() - clock,
               "exit_code": code, "timeout": timed_out, "source_before": before,
               "stdout_sha256": sha(evidence / (label + ".stdout")),
               "stderr_sha256": sha(evidence / (label + ".stderr")),
               "stdout_bytes": (evidence / (label + ".stdout")).stat().st_size,
               "stderr_bytes": (evidence / (label + ".stderr")).stat().st_size}
        # Preserve process outcome even if the after guard rejects.
        write(evidence / (label + ".result.json"), row)
        records.append(row)
        after = guard()
        write(evidence / (label + ".source-after.json"), after)
        if timed_out or code:
            raise RuntimeError("preserved unsuccessful stage: " + label)

    try:
        audit = here.parents[1]
        audit_head = git(audit, "rev-parse", "HEAD")
        workflow = audit / ".github/workflows/kno3-current-main-replay.yml"
        caller = audit / ".github/workflows/chemistry-audit.yml"
        write(evidence / "routing-witness.json", {
            "audit_head": audit_head, "dispatched_head": os.environ.get("GITHUB_SHA"),
            "github_workflow_ref": os.environ.get("GITHUB_WORKFLOW_REF"),
            "github_workflow_sha": os.environ.get("GITHUB_WORKFLOW_SHA"),
            "caller": ".github/workflows/chemistry-audit.yml", "caller_sha256": sha(caller),
            "callee": ".github/workflows/kno3-current-main-replay.yml", "callee_sha256": sha(workflow),
            "producer_sha256": sha(__file__), "physical_verdict": "not_assigned"})
        gate("prepare", 8192)
        binding = json.loads((here / "design-binding.json").read_text())
        if binding["source_head"] != SOURCE or binding["selected_experiment"] != "15":
            raise RuntimeError("unexpected frozen selection")
        for name, digest in binding["copied_files"].items():
            file = here / "design" / name
            if not file.resolve().is_relative_to((here / "design").resolve()) or sha(file) != digest:
                raise RuntimeError("changed frozen design member: " + name)
        forecast = json.loads((here / "design/expectations.json").read_text())
        if len([case for case in forecast["experiments"] if case["id"] == "15"]) != 1:
            raise RuntimeError("selected forecast missing or duplicate")
        if audit_head != os.environ["GITHUB_SHA"] or git(audit, "status", "--porcelain", "--untracked-files=no"):
            raise RuntimeError("audit checkout must equal clean dispatched head")
        if os.environ.get("GITHUB_WORKFLOW_SHA") != audit_head:
            raise RuntimeError("routing workflow revision must equal dispatched audit head")
        if os.environ.get("GITHUB_WORKFLOW_REF", "").split("@", 1)[0].split("/", 2)[-1] not in (
                ".github/workflows/chemistry-audit.yml", ".github/workflows/kno3-current-main-replay.yml"):
            raise RuntimeError("unexpected dispatch route")
        inputs = evidence / "audit-inputs"
        inputs.mkdir()
        shutil.copytree(here / "design", inputs / "design")
        for file in [here / "design-binding.json", Path(__file__), workflow, caller]:
            shutil.copyfile(file, inputs / file.name)
        identity = {"schema_version": 1, "source": guard(), "audit_head": audit_head,
                    "producer_sha256": sha(__file__), "workflow_sha256": sha(workflow),
                    "routing_witness_sha256": sha(evidence / "routing-witness.json"),
                    "design_binding_sha256": sha(here / "design-binding.json"), "design": binding,
                    "run_id": os.environ["GITHUB_RUN_ID"], "run_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
                    "repository": os.environ["GITHUB_REPOSITORY"], "event": os.environ["GITHUB_EVENT_NAME"],
                    "physical_verdict": "not_assigned", "comparison_to_original": "requires_manual_review"}
        if identity["event"] != "workflow_dispatch":
            raise RuntimeError("dispatch only")
        write(evidence / "identity.json", identity)
        # Explicit build environment: avoid inherited RUSTFLAGS, wrappers or credentials.
        env = {key: os.environ[key] for key in ["PATH", "HOME", "RUSTUP_HOME", "CARGO_HOME", "TMPDIR"]
               if key in os.environ}
        env.update({"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC",
                    "CARGO_TARGET_DIR": str(source / "target"), "CARGO_BUILD_JOBS": "2",
                    "RUSTFLAGS": "", "RUSTDOCFLAGS": "", "CC": "gcc", "CXX": "g++",
                    "CFLAGS": "", "CXXFLAGS": "", "CPPFLAGS": "", "LDFLAGS": "",
                    "CMAKE_BUILD_PARALLEL_LEVEL": "2", "KERO_LANG": "en"})
        write(evidence / "build-environment.json", env)
        toolchain = {}
        for name, command in [("rustc", ["rustc", "-vV"]), ("cargo", ["cargo", "-Vv"]),
                              ("cc", ["gcc", "--version"]), ("cxx", ["g++", "--version"]),
                              ("cmake", ["cmake", "--version"])]:
            gate("toolchain-" + name, 8192)
            path = shutil.which(command[0], path=env["PATH"])
            toolchain[name] = {"argv": command, "executable_sha256": sha(path),
                               "version": subprocess.check_output(command, env=env, text=True)}
        for name in ["rustc", "cargo"]:
            path = subprocess.check_output(["rustup", "which", name], env=env, text=True).strip()
            toolchain[name]["actual_tool_sha256"] = sha(path)
        write(evidence / "toolchain.json", toolchain)
        if (source / "target").exists():
            raise RuntimeError("target must be fresh; no build cache admitted")
        run("fetch-sundials", ["bash", "tools/fetch-sundials.sh"], env, 300)
        if not (source / "Cargo.lock").exists():
            run("resolve-lock", ["cargo", "generate-lockfile"], env, 300)
        shutil.copyfile(source / "Cargo.lock", evidence / "Cargo.lock")
        lock_sha = sha(evidence / "Cargo.lock")
        write(evidence / "lock-binding.json", {"sha256": lock_sha, "tracked_in_source":
              bool(git(source, "ls-files", "Cargo.lock")), "build_uses_locked": True})
        run("release-build", ["cargo", "build", "--release", "--locked", "-p", "kerotakis-cli", "-vv"], env, 2400)
        if sha(source / "Cargo.lock") != lock_sha:
            raise RuntimeError("dependency lock changed during build")
        binary = source / "target/release/kero"
        gate("preserve-binary", 1024)
        shutil.copyfile(binary, evidence / "kero")
        binary_sha = sha(binary)
        if sha(evidence / "kero") != binary_sha:
            raise RuntimeError("preserved binary differs from built binary")
        write(evidence / "binary-binding.json", {"sha256": binary_sha, "lock_sha256": lock_sha,
              "source_head": SOURCE, "source_tree": identity["source"]["tree"]})
        if git(audit, "status", "--porcelain", "--untracked-files=no"):
            raise RuntimeError("audit source changed during build")
        if any(sha(here / "design" / name) != digest for name, digest in binding["copied_files"].items()):
            raise RuntimeError("frozen design changed before observation")
        # Same CLI syntax as normalized40/run-frozen.py, exact frozen script bytes.
        run("experiment15", [str(binary), "run", "scripts/15.lab", "--json"], env, 60, here / "design")
        if sha(binary) != binary_sha or sha(source / "Cargo.lock") != lock_sha:
            raise RuntimeError("binary or lock changed during observation")
        write(evidence / "completed.json", {"completed_utc": utc(), "experiment_ids": ["15"],
              "binary_sha256": binary_sha, "lock_sha256": lock_sha,
              "identity_sha256": sha(evidence / "identity.json"), "physical_verdict": "not_assigned"})
    except BaseException as error:
        write(evidence / "error.json", {"utc": utc(), "type": type(error).__name__, "message": str(error),
              "completed_stages": [row["stage"] for row in records], "physical_verdict": "not_assigned"})
        raise


if __name__ == "__main__":
    main()
