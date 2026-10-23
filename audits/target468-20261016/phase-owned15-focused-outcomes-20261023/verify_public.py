"""Verify saved focused outcomes, without executing the engine or archives."""
from pathlib import Path
import hashlib
import json
import re

B = Path(__file__).resolve().parent


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    manifest = read(B / "original-manifest.json")
    plan = read(B / "plan-projection.json")
    commands = read(B / "commands-projection.json")
    parent = read(B / "parent-small-review.json")
    independent = read(B / "independent-small-review.json")
    assert sha(B / "original-manifest.json") == parent["manifest_sha256"] == independent["manifest_sha256"]
    for path in (B / "raw").iterdir():
        assert path.is_file() and not path.name.endswith(".gz")
        assert manifest[path.name] == {"sha256": sha(path), "bytes": path.stat().st_size}
    assert plan["source"] == parent["source"] == independent["source"]
    assert plan["tree"] == parent["tree"] == independent["tree"]
    assert sha(B / "raw/actual-producer-Cargo.lock") == plan["dependency_lock_sha256"]
    done = read(B / "raw/COMPLETED.json")
    assert done["runtime_source"] == plan["source"] and done["runtime_tree"] == plan["tree"]
    expected = {(t["target"], name) for t in plan["targets"] for name in t["tests"]}
    rows = done["actual_outcomes"]
    assert len(expected) == len(rows) == 15
    assert {(r["target"], r["test"]) for r in rows} == expected
    for row in rows:
        key = row["record"]
        record = commands[key]
        assert record["original_record_sha256"] == manifest[key + ".json"]["sha256"]
        assert record["exit_status"] == row["exit_status"] == 0 and row["complete"]
        assert record["argv_projection"][1:] == ["--exact", row["test"], "--nocapture", "--test-threads=1"]
        for suffix in ("stdout", "stderr"):
            assert sha(B / "raw" / (key + "." + suffix)) == record[suffix + "_sha256"]
        out = (B / "raw" / (key + ".stdout")).read_text()
        assert re.findall(r"^test (\S+) \.\.\. (ok|FAILED)$", out, re.M) == [(row["test"], "ok")]
        assert "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out;" in out
    assert parent["actual_observed_passes"] == independent["target"]["count"] == 15
    assert parent["current_affected_controls_pass"] is True
    assert not parent["local_archive_bodies_read"] and not independent["gzip_body_read_or_hashed"]
    for label, argv in [("format-check", ["cargo", "fmt", "--check"]), ("core-clippy", ["cargo", "clippy", "--locked", "-p", "kerotakis-core", "--all-targets", "--", "-D", "warnings"])]:
        record = read(B / "raw" / (label + ".json"))
        assert record["argv"] == argv and record["exit_status"] == 0
        assert independent["strict_gates"][label] == {k: record[k] for k in ("argv", "exit_status", "stdout_sha256", "stderr_sha256")}
    print("Saved15-control projection verified; strict core Clippy/canonical format pass; no full-source or scientific selection claim.")


if __name__ == "__main__":
    main()
