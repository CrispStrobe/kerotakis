#!/usr/bin/env python3
"""Verify the completed own artifact and replay the frozen seven on a gated worker."""
import base64, hashlib, json, os, pathlib, re, stat, subprocess, sys, zipfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE / "pr786-low-pressure"
sys.path.insert(0, str(ROOT))
import run as producer
from analyse import sha, strict_json


def main():
    if sys.flags.optimize:
        raise SystemExit("optimized Python is unsupported")
    assert os.environ.get("GITHUB_ACTIONS") == "true"
    assert os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch"
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == os.environ["GITHUB_SHA"]
    assert not subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], text=True).strip()
    protocol = strict_json((ROOT / "protocol.json").read_text())
    assert protocol["source_head"] == "10c963e17401b65c947f1f4b89a8d4e35fef5e71"
    for name, digest in protocol["files"].items():
        assert sha(ROOT / name) == digest
    components = strict_json((ROOT / "runtime-binding-fields.json").read_text())
    assert components["producer_sha256"] == sha(ROOT / "run.py")
    assert components["consumer_sha256"] == sha(ROOT / "analyse.py")
    out = pathlib.Path("pr786-low-pressure-evidence")
    out.mkdir(exist_ok=False)
    fast = pathlib.Path(os.environ["RUNNER_TEMP"])
    def gate(stage):
        r = producer.gate(fast, pathlib.Path.cwd())
        with (out / "resource-journal.jsonl").open("a") as f:
            f.write(json.dumps(dict(r, stage=stage)) + "\n")
        assert r["permitted"], "hosted resource gate closed"
    gate("metadata")
    flags = ["--allow-escape-sequences"] if "--allow-escape-sequences" in subprocess.check_output(["gh", "api", "--help"], text=True) else []
    def api(name, endpoint):
        gate(name)
        dest = out / name
        with dest.open("xb") as stdout, dest.with_suffix(dest.suffix + ".stderr").open("xb") as stderr:
            subprocess.run(["gh", "api", *flags, "repos/CrispStrobe/kerotakis/" + endpoint], stdout=stdout, stderr=stderr, check=True, timeout=60)
        return dest
    run_meta = strict_json(api("run.json", "actions/runs/38528139788").read_text())
    job = strict_json(api("job.json", "actions/jobs/115733291584").read_text())
    assert run_meta["head_sha"] == protocol["source_head"] and run_meta["event"] == "pull_request" and run_meta["run_attempt"] == 1
    assert job["run_id"] == 38528139788 and job["conclusion"] == "success" and job["status"] == "completed"
    assert all(s["status"] == "completed" and s["conclusion"] == "success" for s in job["steps"])
    artifact = strict_json(api("artifact.json", "actions/artifacts/11878769254").read_text())
    assert artifact["id"] == 11878769254
    tested = "b9d9ae90d0a0a17b1bee47df5fb54b03496f40b0"
    assert artifact["name"] == "chemistry-audit-cli-" + tested and artifact["expired"] is False
    assert artifact["size_in_bytes"] == 6489404
    assert artifact["digest"] == "sha256:39cbbd687e3d4be92a858a99abc3074fd52f968401f2760c65f177979d036fe3"
    assert artifact["workflow_run"]["id"] == 38528139788 and artifact["workflow_run"]["head_sha"] == protocol["source_head"]
    commit = strict_json(api("tested-commit.json", "git/commits/" + tested).read_text())
    source = strict_json(api("source-commit.json", "git/commits/" + protocol["source_head"]).read_text())
    parents = [x["sha"] for x in commit["parents"]]
    assert parents == [protocol["base"], protocol["source_head"]]
    assert commit["tree"]["sha"] == source["tree"]["sha"] == protocol["source_tree"]
    workflow = strict_json(api("source-workflow.json", "contents/.github/workflows/chemistry-audit.yml?ref=" + protocol["source_head"]).read_text())
    assert workflow["encoding"] == "base64"
    assert hashlib.sha256(base64.b64decode(workflow["content"])).hexdigest() == "57252f0fb26ac203b5420e82a3e1355a20ec0ea6ac925d7078b95bcaf75e1304"
    log = api("build.log", "actions/jobs/115733291584/logs").read_text()
    clean = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log)
    assert "git log -1 --format=%H" in clean and any(line.split() and line.split()[-1] == tested for line in clean.splitlines())
    assert "cargo build --release -p kerotakis-cli" in clean and "name: chemistry-audit-cli-" + tested in clean
    archive = api("artifact.zip", "actions/artifacts/11878769254/zip")
    assert archive.stat().st_size == artifact["size_in_bytes"] and sha(archive) == artifact["digest"].removeprefix("sha256:")
    gate("archive")
    runtime = out / "runtime"; runtime.mkdir()
    expected = {"target/release/kero", "Cargo.lock", "binary-sha256.txt", "submodules.txt"}
    with zipfile.ZipFile(archive) as z:
        infos = z.infolist()
        assert len(infos) == 4 and {i.filename for i in infos} == expected
        assert sum(i.file_size for i in infos) < 32 * 1024 * 1024
        assert all(not i.is_dir() and not stat.S_ISLNK(i.external_attr >> 16) for i in infos)
        assert z.testzip() is None
        for info in infos:
            (runtime / pathlib.Path(info.filename).name).write_bytes(z.read(info))
    binary = runtime / "kero"; lock = runtime / "Cargo.lock"
    elf = binary.read_bytes()
    assert elf[:6] == b"\x7fELF\x02\x01" and int.from_bytes(elf[18:20], "little") == 62
    binary_digest = sha(binary); lock_digest = sha(lock)
    assert (runtime / "binary-sha256.txt").read_text().split() == [binary_digest, "target/release/kero"]
    modules = (runtime / "submodules.txt").read_text().splitlines()
    assert all(line.startswith(" ") for line in modules)
    assert {line.split()[1]: line.split()[0] for line in modules} == {
        "sundials-kinetics-rs": "8f2e16e6d6f00926ea28cda9b6ccb3cf2538b5fa",
        "vendor/iphreeqc": "47b459c4a5e3e20b9990fa304e207eca0dcc6ff1"}
    binary.chmod(0o500); lock.chmod(0o400)
    receipt = {"review_kind": "own_artifact_byte_and_source_association", "all_checks_pass": True,
        "candidate": {"head": protocol["source_head"], "tree": protocol["source_tree"], "binary_sha256": binary_digest, "lock_sha256": lock_digest},
        "base": protocol["base"], "tested_checkout": tested, "tested_parents": parents,
        "run": 38528139788, "run_attempt": 1, "build_job": 115733291584,
        "artifact_id": 11878769254, "artifact_zip_sha256": sha(archive),
        "limitations": ["Top-level module revisions verified; recursive module file bytes and full compiler/native flags are not independently traversed.", "Artifact acceptance does not accept the candidate's currently failed curiosity support contract."]}
    review_path = out / "artifact-review.json"; review_path.write_text(json.dumps(receipt, indent=2) + "\n")
    binding = {"protocol_sha256": sha(ROOT / "protocol.json"), "runner_sha256": sha(ROOT / "run.py"),
        "analyzer_sha256": sha(ROOT / "analyse.py"), "artifact_review_receipt_sha256": sha(review_path),
        "source_head": protocol["source_head"], "source_tree": protocol["source_tree"],
        "binary_sha256": binary_digest, "lock_sha256": lock_digest,
        "artifact_id": 11878769254, "artifact_zip_sha256": sha(archive)}
    binding_path = out / "execution-binding.json"; binding_path.write_text(json.dumps(binding, indent=2) + "\n")
    env = dict(os.environ); env.pop("GH_TOKEN", None); env.pop("GITHUB_TOKEN", None)
    gate("two-controls")
    subprocess.run([sys.executable, str(ROOT / "run.py"), "--binary", str(binary), "--lock", str(lock), "--binding", str(binding_path),
        "--artifact-review", str(review_path), "--out", str(out / "raw"), "--fast", str(fast), "--storage", str(pathlib.Path.cwd())], env=env, check=True)
    subprocess.run([sys.executable, str(ROOT / "analyse.py"), "--raw", str(out / "raw"), "--out", str(out / "report.json")], env=env, check=True)
    report = strict_json((out / "report.json").read_text())
    assert report["all_pass"] is True and len(report["cases"]) == 2
    marker = {"audit_checkout": os.environ["GITHUB_SHA"], "helper_sha256": sha(pathlib.Path(__file__)),
        "source_head": protocol["source_head"], "source_tree": protocol["source_tree"],
        "binary_sha256": binary_digest, "lock_sha256": lock_digest, "artifact_id": 11878769254,
        "artifact_zip_sha256": sha(archive), "protocol_sha256": sha(ROOT / "protocol.json"),
        "artifact_review_sha256": sha(review_path), "report_sha256": sha(out / "report.json"),
        "cases": report["cases"], "all_two_pass": True, "candidate_integrated": False,
        "empirical_calibration": False, "closed_chemistry_implemented": False}
    (out / "marker.json").write_text(json.dumps(marker, indent=2) + "\n")
    print("PR786_LOW_PRESSURE_REPLAY_JSON=" + json.dumps(marker, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
