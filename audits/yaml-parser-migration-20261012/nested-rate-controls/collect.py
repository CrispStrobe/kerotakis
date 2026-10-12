#!/usr/bin/env python3
"""Execute immutable source-informed CLI controls and preserve every outcome."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent


def digest(blob):
    return hashlib.sha256(blob).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def strict_json(blob):
    def reject_constant(value):
        raise ValueError("non-JSON numeric constant: " + value)
    value = json.loads(blob, parse_constant=reject_constant)
    def finite(item):
        if isinstance(item, float):
            require(math.isfinite(item), "nonfinite JSON value")
        elif isinstance(item, dict):
            for child in item.values():
                finite(child)
        elif isinstance(item, list):
            for child in item:
                finite(child)
    finite(value)
    return value


def keys(value, expected):
    require(isinstance(value, dict) and set(value) == set(expected), "output schema differs")


def check(case, stdout, stderr, reference_json, manifest):
    if case["contract"] == "refusal":
        require(not stdout, "refusal emitted a successful payload")
        message = stderr.decode()
        require(all(text in message for text in case["stderr_contains"]), "refusal diagnostic differs")
        return
    require(not stderr, "successful command emitted stderr")
    require(strict_json(stdout) == case["expected_json"], "complete inspect JSON differs")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--binary-sha256", required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--manifest-sha256", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    require(__debug__, "optimized Python is unsupported")
    manifest_bytes = (HERE / "manifest.json").read_bytes()
    require(digest(manifest_bytes) == args.manifest_sha256, "manifest SHA differs")
    manifest = json.loads(manifest_bytes)
    require(digest(Path(__file__).read_bytes()) == manifest["collector_sha256"], "collector SHA differs")
    require(len(manifest["cases"]) == manifest["expected_count"] == 12 and [c["id"] for c in manifest["cases"]] == manifest["expected_ids"] and len(set(manifest["expected_ids"])) == 12, "frozen case inventory differs")
    binary = args.binary.resolve(strict=True)
    source = args.source.resolve(strict=True)
    require(args.binary_sha256 == manifest["candidate_binary_sha256"], "binary is not the frozen candidate")
    require(digest(binary.read_bytes()) == args.binary_sha256, "binary SHA differs")
    for path, wanted in manifest["source_sha256"].items():
        require(digest((source / path).read_bytes()) == wanted, "source bytes differ: " + path)
    payloads = {}
    for identity, entry in manifest["inputs"].items():
        blob = (source / entry["repository_path"]).read_bytes() if "repository_path" in entry else entry["text"].encode()
        require(digest(blob) == entry["sha256"], "input SHA differs: " + identity)
        payloads[identity] = blob
    require(not args.out.exists() and not args.out.is_symlink(), "output already exists")
    out = args.out.resolve()
    require(not out.is_relative_to(source), "evidence must be outside source")
    out.mkdir(parents=True, exist_ok=False)
    (out / "inputs").mkdir()
    (out / "manifest.json").write_bytes(manifest_bytes)
    (out / "collector.py").write_bytes(Path(__file__).read_bytes())
    for identity, blob in payloads.items():
        (out / "inputs" / (identity + ".yaml")).write_bytes(blob)
    results, references = [], {}
    for case in manifest["cases"]:
        case_dir = out / case["id"]
        case_dir.mkdir()
        argv = [str(binary)] + [str(out / "inputs" / (case["input"] + ".yaml")) if arg == "@input" else arg for arg in case["args"]]
        write_json(case_dir / "command.json", dict(argv=argv, cwd=str(source), timeout_seconds=case["timeout_seconds"]))
        timed_out, launch_error, code = False, None, None
        try:
            process = subprocess.run(argv, cwd=source, capture_output=True, timeout=case["timeout_seconds"])
            stdout, stderr, code = process.stdout, process.stderr, process.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr, timed_out = error.stdout or b"", error.stderr or b"", True
        except OSError as error:
            stdout, stderr, launch_error = b"", b"", str(error)
        (case_dir / "stdout.log").write_bytes(stdout)
        (case_dir / "stderr.log").write_bytes(stderr)
        write_json(case_dir / "exit.json", dict(exit_code=code, timeout=timed_out, launch_error=launch_error))
        failure = None
        try:
            require(not timed_out and launch_error is None, "command timed out or did not launch")
            require(code == case["expected_exit"], "exit differs: " + str(code))
            check(case, stdout, stderr, references, manifest)
            if case["contract"] == "inspect_json":
                references[case["id"]] = strict_json(stdout)
        except Exception as error:
            failure = type(error).__name__ + ": " + str(error)
        results.append(dict(id=case["id"], passed=failure is None, failure=failure, exit_code=code, timeout=timed_out, launch_error=launch_error, input_sha256=manifest["inputs"][case["input"]]["sha256"], stdout_sha256=digest(stdout), stderr_sha256=digest(stderr), command_sha256=digest((case_dir / "command.json").read_bytes()), exit_sha256=digest((case_dir / "exit.json").read_bytes())))
    report = dict(source=manifest["source"], source_tree=manifest["source_tree"], binary_sha256=args.binary_sha256, manifest_sha256=args.manifest_sha256, collector_sha256=manifest["collector_sha256"], total=12, passed=sum(r["passed"] for r in results), failed=sum(not r["passed"] for r in results), cases=results, accepted_integration=False, scope=manifest["scope"], limits=manifest["limits"])
    write_json(out / "results.json", report)
    print(json.dumps({k: report[k] for k in ["total", "passed", "failed", "accepted_integration"]}))
    return 0 if report["failed"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
