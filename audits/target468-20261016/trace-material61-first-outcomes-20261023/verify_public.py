"""Verify this explicit saved-byte projection; no engine or archive execution."""
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
    assert len(expected) == len(rows) == 61
    assert {(r["target"], r["test"]) for r in rows} == expected
    passes, failures = [], []
    for row in rows:
        key = row["record"]
        record = commands[key]
        assert record["original_record_sha256"] == manifest[key + ".json"]["sha256"]
        assert record["exit_status"] == row["exit_status"]
        assert record["argv_projection"][1:] == ["--exact", row["test"], "--nocapture", "--test-threads=1"]
        for suffix in ("stdout", "stderr"):
            assert sha(B / "raw" / (key + "." + suffix)) == record[suffix + "_sha256"]
        out = (B / "raw" / (key + ".stdout")).read_text()
        actual = re.findall(r"^test (\S+) \.\.\. (ok|FAILED)$", out, re.M)
        assert actual == [(row["test"], "ok" if row["exit_status"] == 0 else "FAILED")]
        assert row["complete"]
        (passes if row["exit_status"] == 0 else failures).append(row["test"])
    assert len(passes) == parent["actual_observed_passes"] == independent["validated_named_passes"] == 60
    assert failures == ["carbonate_weak_acid_microtransfers_settle_and_repeat_without_drift"]
    assert "0.0000008098921674949041" in (B / "raw/particle_census-case-01.stderr").read_text()
    sweep = (B / "raw/full-1536-sweep.stdout").read_text().strip().splitlines()
    assert sweep == ["swept 1536 states — 1536 ran, 8 step(s) stated a refusal", "no invariant was violated."]
    assert commands["full-1536-sweep"]["exit_status"] == 0
    assert parent["local_archive_bodies_read"] is False
    assert independent["gzip_archive_bodies_read_or_hashed"] is False
    assert not parent["full_source_or_product_accepted"]
    print("Saved projection verified:60/61; zero-time drift retained;1536 sweep passed; no scientific selections.")


if __name__ == "__main__":
    main()
