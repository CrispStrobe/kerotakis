#!/usr/bin/env python3
"""Collect the exact original applicability baseline; successful capture is not acceptance."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys

SOURCE = "093db6c9a9e64f1d704ded354685cee4ded6a9ec"
TREE = "22ab9d3ad14b824274c6c4d7c00e780808fc857f"
REGULAR_COUNT = 2251
HERE = Path(__file__).resolve().parent
AUDIT = HERE.parents[1]
FROZEN = (
    ("freeze.json", "53566e8135c88580e938d4abddd69f3515ea6b660fe9d773e4da140fb0fc6e64", 24),
    ("runtime-route-freeze.json", "cd7f4a16f4d5b6d8c5f2899fc083ee01e5572cb5b395540474ee3908d9e0a924", 2),
)
GITLINKS = {"sundials-kinetics-rs": "8f2e16e6d6f00926ea28cda9b6ccb3cf2538b5fa",
            "vendor/iphreeqc": "47b459c4a5e3e20b9990fa304e207eca0dcc6ff1"}
ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def json_file(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")


def git(source, *args, input_data=None):
    proc = subprocess.run(["git", "-C", str(source), *args], input=input_data,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          env=dict(os.environ, GIT_OPTIONAL_LOCKS="0"))
    require(proc.returncode == 0, "Git inspection failed: " + " ".join(args) +
            ": " + proc.stderr.decode(errors="replace"))
    return proc.stdout


def frozen_inputs():
    inputs = []
    all_names = []
    for name, expected_hash, count in FROZEN:
        raw = (HERE / name).read_bytes()
        require(digest(raw) == expected_hash, "immutable freeze SHA mismatch: " + name)
        freeze = json.loads(raw)
        require(freeze["source"] == SOURCE and freeze["source_tree"] == TREE, "wrong frozen source")
        relative = PurePosixPath(freeze["fixture_path"])
        require(not relative.is_absolute() and ".." not in relative.parts, "nonportable fixture path")
        fixture = (AUDIT / relative).read_bytes()
        require(digest(fixture) == freeze["fixture_sha256"], "immutable fixture SHA mismatch")
        names = re.findall(r"#\[test\]\s*fn\s+(\w+)\s*\(", fixture.decode())
        require(names == freeze["expected_tests"] and len(names) == count == freeze["expected_count"]
                and len(names) == fixture.count(b"#[test]") == len(set(names)), "wrong frozen identities")
        destination = PurePosixPath(freeze["injected_fixture_path"])
        require(destination.parent == PurePosixPath("crates/kerotakis-core/tests")
                and destination.suffix == ".rs", "wrong fixture destination")
        inputs.append(dict(freeze_name=name, freeze_sha256=expected_hash, fixture_sha256=digest(fixture),
                           fixture=fixture, names=names, destination=str(destination), freeze=freeze))
        all_names += names
    require(len(all_names) == len(set(all_names)) == 26, "duplicate or incomplete combined identities")
    require(inputs[1]["freeze"]["original_freeze_sha256"] == inputs[0]["freeze_sha256"],
            "supplement does not bind original freeze")
    return inputs


def source_inventory(source):
    require(git(source, "rev-parse", "HEAD").decode().strip() == SOURCE, "wrong source HEAD")
    require(git(source, "rev-parse", "HEAD^{tree}").decode().strip() == TREE, "wrong source tree")
    require(Path(git(source, "rev-parse", "--show-toplevel").decode().strip()).resolve() == source,
            "source must name its checkout root")
    status = git(source, "status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none")
    require(not status, "unexpected initial tracked/untracked/submodule source changes")
    entries = []
    links = {}
    for entry in git(source, "ls-tree", "-rz", "HEAD").split(b"\0"):
        if not entry:
            continue
        header, path_raw = entry.split(b"\t", 1)
        mode, kind, oid = header.decode().split()
        path = path_raw.decode()
        if mode == "160000":
            require(kind == "commit", "invalid gitlink kind")
            links[path] = oid
        else:
            require(mode in {"100644", "100755"} and kind == "blob", "unsupported tracked source mode")
            entries.append(dict(path=path, mode=mode, git_blob=oid))
    require(len(entries) == REGULAR_COUNT and links == GITLINKS, "wrong source regular/gitlink inventory")
    objects = git(source, "cat-file", "--batch", input_data="".join(row["git_blob"] + "\n" for row in entries).encode())
    offset = 0
    for row in entries:
        newline = objects.index(b"\n", offset)
        oid, kind, size = objects[offset:newline].decode().split()
        require(oid == row["git_blob"] and kind == "blob", "wrong batch blob response")
        start = newline + 1
        end = start + int(size)
        blob = objects[start:end]
        require(objects[end:end + 1] == b"\n", "truncated batch blob")
        offset = end + 1
        path = source / row["path"]
        require(stat.S_ISREG(path.lstat().st_mode), "tracked source is not a regular file: " + row["path"])
        require(path.read_bytes() == blob, "tracked source bytes differ: " + row["path"])
        row["sha256"] = digest(blob)
    require(offset == len(objects), "unexpected batch trailing data")
    submodule_raw = git(source, "submodule", "status", "--recursive").decode()
    modules = []
    for line in submodule_raw.splitlines():
        match = re.fullmatch(r"([ +\-U])([0-9a-f]{40}) ([^ ]+)(?: \(.*\))?", line)
        require(match is not None and match[1] == " ", "uninitialized/mismatched/conflicted recursive submodule")
        path = match[3]
        commit = git(source / path, "rev-parse", "HEAD").decode().strip()
        require(commit == match[2], "wrong actual submodule HEAD")
        require(not git(source / path, "status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none"),
                "dirty recursive submodule: " + path)
        modules.append(dict(path=path, commit=commit,
                            tree=git(source / path, "rev-parse", "HEAD^{tree}").decode().strip(),
                            status="initialized_at_recorded_commit_and_git_clean"))
    observed = {module["path"]: module["commit"] for module in modules}
    require(all(observed.get(path) == commit for path, commit in GITLINKS.items()), "missing required source gitlink")
    return dict(source=SOURCE, source_tree=TREE, compared_regular_files=REGULAR_COUNT,
                regular_files=entries, gitlinks=links, recursive_submodules=modules,
                recursive_submodule_status=submodule_raw,
                scope="All regular source blobs compared byte-for-byte; recursive gitlinks initialized/HEAD/status checked. "
                      "Submodule contents were not separately blob-compared, and ignored environment/cache contents were not inventoried.")


def run_command(out, name, argv, source, env):
    json_file(out / (name + "-command.json"), dict(argv=argv, cwd=str(source),
        env_overlay={key: env[key] for key in ["CARGO_BUILD_JOBS", "CARGO_TERM_COLOR", "RUST_BACKTRACE"]}))
    with (out / (name + ".log")).open("xb") as stream:
        code = subprocess.run(argv, cwd=source, env=env, stdout=stream, stderr=subprocess.STDOUT).returncode
    json_file(out / (name + "-exit.json"), dict(exit_code=code))
    return code


def test_observations(log, inputs):
    log = ANSI.sub("", log)
    reports = []
    all_headers = list(re.finditer(r"^\s*Running tests/[^\n]+\.rs\s+\(", log, re.M))
    for item in inputs:
        stem = Path(item["destination"]).stem
        headers = list(re.finditer(r"^\s*Running tests/" + re.escape(stem) + r"\.rs\s+\(", log, re.M))
        require(len(headers) == 1, "missing/repeated actual test binary: " + stem)
        start = headers[0].end()
        later_headers = [header.start() for header in all_headers if header.start() >= start]
        block = log[start:min(later_headers) if later_headers else len(log)]
        running = re.findall(r"^\s*running (\d+) tests\s*$", block, re.M)
        require(running == [str(len(item["names"]))], "wrong binary running count: " + stem)
        lines = list(re.finditer(r"^test (\w+) \.\.\. (ok|FAILED)$", block, re.M))
        require(len(lines) == len(item["names"]), "wrong binary test-result line count: " + stem)
        outcomes = {}
        for line in lines:
            require(line[1] not in outcomes, "duplicate binary result identity: " + line[1])
            outcomes[line[1]] = line[2]
        require(set(outcomes) == set(item["names"]), "wrong binary test identities: " + stem)
        records = [dict(name=name, outcome=outcomes[name]) for name in item["names"]]
        summaries = list(re.finditer(
            r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
            r"(\d+) measured; (\d+) filtered out", block))
        require(len(summaries) == 1, "missing/repeated binary test summary: " + stem)
        require(summaries[0].start() > max(line.end() for line in lines),
                "binary summary precedes result identities: " + stem)
        passed = sum(record["outcome"] == "ok" for record in records)
        failed = len(records) - passed
        require(summaries[0].groups() == ("FAILED" if failed else "ok", str(passed), str(failed), "0", "0", "0"),
                "unfiltered binary summary does not match actual identities: " + stem)
        reports.append(dict(target=stem, fixture_sha256=item["fixture_sha256"], tests=records,
                            passed=passed, failed=failed, ignored=0, filtered_out=0))
    require(len(all_headers) == 2, "unexpected/missing actual binary inventory")
    require(sum(len(report["tests"]) for report in reports) == 26, "incomplete original baseline")
    require(len(re.findall(r"^test \w+ \.\.\. (?:ok|FAILED)$", log, re.M)) == 26,
            "unexpected/repeated test result lines")
    require(len(re.findall(r"test result: (?:ok|FAILED)\. \d+ passed;", log)) == 2,
            "unexpected/missing actual test summary inventory")
    return reports


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    require(__debug__, "optimized Python is not supported")
    source = args.source.resolve(strict=True)
    out = args.out.resolve()
    require(not args.out.exists() and not args.out.is_symlink(), "output already exists")
    require(not out.is_relative_to(source), "evidence directory must be outside original source")
    out.mkdir(parents=True, exist_ok=False)
    result = dict(collection_complete=False, acceptance=False, accepted_repair=False, accepted_integration=False,
                  source=SOURCE, source_tree=TREE, started_utc=datetime.now(timezone.utc).isoformat(),
                  collector_sha256=digest(Path(__file__).read_bytes()),
                  python_version=sys.version,
                  hosted_environment={key: os.environ.get(key) for key in
                    ["GITHUB_REPOSITORY", "GITHUB_EVENT_NAME", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_SHA"]},
                  scope="Original frozen baseline observation only. Job success means complete capture, never repair or integration acceptance.")
    try:
        result["audit_revision"] = git(AUDIT, "rev-parse", "HEAD").decode().strip()
        collector_path = str(Path(__file__).resolve().relative_to(AUDIT))
        require(git(AUDIT, "show", result["audit_revision"] + ":" + collector_path)
                == Path(__file__).read_bytes(), "collector bytes differ from recorded audit revision")
        if os.environ.get("GITHUB_SHA"):
            require(os.environ["GITHUB_SHA"] == result["audit_revision"], "hosted audit revision mismatch")
        inputs = frozen_inputs()
        require(not (source / "Cargo.lock").exists(), "existing source lock cannot stand in for fresh hosted resolution")
        for item in inputs:
            destination = source / item["destination"]
            require(not destination.exists() and not destination.is_symlink(), "fixture destination already exists")
        inventory = source_inventory(source)
        source_hashes = {row["path"]: row["sha256"] for row in inventory["regular_files"]}
        for path, wanted in inputs[0]["freeze"]["source_sha256"].items():
            require(source_hashes.get(path) == wanted, "wrong frozen source-file SHA: " + path)
        json_file(out / "source-inventory.json", inventory)
        result["source_inventory_sha256"] = digest((out / "source-inventory.json").read_bytes())
        result["fixtures"] = []
        for item in inputs:
            (out / item["freeze_name"]).write_bytes((HERE / item["freeze_name"]).read_bytes())
            with (source / item["destination"]).open("xb") as stream:
                stream.write(item["fixture"])
            (out / Path(item["destination"]).name).write_bytes(item["fixture"])
            result["fixtures"].append({key: item[key] for key in
                                      ["freeze_name", "freeze_sha256", "fixture_sha256", "destination", "names"]})
        env = dict(os.environ, CARGO_BUILD_JOBS="1", CARGO_TERM_COLOR="never", RUST_BACKTRACE="0")
        result["dependency_environment"] = {key: env.get(key) for key in
                                            ["RUSTFLAGS", "RUSTC_WRAPPER", "CARGO_TARGET_DIR"]}
        require(run_command(out, "cargo-version", ["cargo", "--version", "--verbose"], source, env) == 0,
                "cargo version capture failed")
        require(run_command(out, "rustc-version", ["rustc", "--version", "--verbose"], source, env) == 0,
                "rustc version capture failed")
        require(run_command(out, "generate-lockfile", ["cargo", "generate-lockfile"], source, env) == 0,
                "hosted lock generation failed; no test outcomes inferred")
        lock = (source / "Cargo.lock").read_bytes()
        (out / "Cargo.lock").write_bytes(lock)
        result["hosted_lock_sha256"] = digest(lock)
        require(run_command(out, "fetch-locked", ["cargo", "fetch", "--locked"], source, env) == 0,
                "hosted dependency fetch failed; no test outcomes inferred")
        require((source / "Cargo.lock").read_bytes() == lock, "locked fetch changed lock")
        targets = [Path(item["destination"]).stem for item in inputs]
        argv = ["cargo", "test", "--locked", "--no-fail-fast", "--color", "never", "-p", "kerotakis-core"]
        for target in targets:
            argv += ["--test", target]
        code = run_command(out, "original-baseline", argv, source, env)
        result["cargo_test_exit_code"] = code
        observations = test_observations((out / "original-baseline.log").read_text(errors="replace"), inputs)
        passed = sum(item["passed"] for item in observations)
        failed = sum(item["failed"] for item in observations)
        require(code == (101 if failed else 0), "Cargo exit does not agree with complete test outcomes")
        require((source / "Cargo.lock").read_bytes() == lock, "locked test changed dependency lock")
        for item in inputs:
            require((source / item["destination"]).read_bytes() == item["fixture"], "injected fixture changed during execution")
        post_status = git(source, "status", "--porcelain=v1", "--untracked-files=all", "--ignore-submodules=none").decode()
        allowed = {"?? " + item["destination"] for item in inputs}
        require(set(post_status.splitlines()) == allowed, "unexpected post-execution source changes")
        json_file(out / "post-source-status.json", dict(status=post_status, allowed_untracked_fixture_paths=[item["destination"] for item in inputs]))
        result.update(collection_complete=True, actual_test_count=26, actual_passed=passed, actual_failed=failed,
                      targets=observations, completed_utc=datetime.now(timezone.utc).isoformat())
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        result["collection_error"] = type(error).__name__ + ": " + str(error)
        result["scope"] += " Incomplete dependency/compile/capture paths do not establish test pass/failure counts."
    result["artifact_sha256"] = {path.name: digest(path.read_bytes()) for path in sorted(out.iterdir()) if path.is_file()}
    json_file(out / "result.json", result)
    print(json.dumps(dict(collection_complete=result["collection_complete"], acceptance=False,
                         actual_test_count=result.get("actual_test_count"),
                         collection_error=result.get("collection_error")), allow_nan=False))
    return 0 if result["collection_complete"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
