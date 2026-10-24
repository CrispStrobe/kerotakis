"""Verify saved first outcomes without executing a solver or opening an archive."""
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text())


def sha(name):
    return hashlib.sha256((ROOT / name).read_bytes()).hexdigest()


manifest = read("manifest.json")
assert set(manifest) == {p.name for p in ROOT.iterdir() if p.is_file() and p.name != "manifest.json"}
for name, entry in manifest.items():
    assert (ROOT / name).stat().st_size == entry["bytes"]
    assert sha(name) == entry["sha256"], name
binding = read("binding.json")
done = read("COMPLETED.json")
assert done["runtime_source"] == binding["source"]
assert done["runtime_tree"] == binding["tree"]
assert sha("actual-producer-Cargo.lock") == binding["actual_producer_lock_sha256"]
expected = {(t["target"], n) for t in binding["targets"] for n in t["tests"]}
actual = {(r["target"], r["test"]) for r in done["actual_outcomes"]}
assert len(expected) == len(actual) == len(done["actual_outcomes"]) == 74
assert expected == actual
passes = failures = 0
for target in binding["targets"]:
    name = target["target"]
    binary = read(name + "-binary.json")
    assert binary["runtime_source"] == binding["source"]
    assert binary["runtime_tree"] == binding["tree"]
    assert binary["actual_producer_lock_sha256"] == binding["actual_producer_lock_sha256"]
    listed = re.findall(r"^(\S+): test$", (ROOT / (name + "-list.stdout")).read_text(), re.M)
    for row in [r for r in done["actual_outcomes"] if r["target"] == name]:
        key = row["record"]
        command = read(key + ".json")
        assert command["argv"][1:] == ["--exact", row["test"], "--nocapture", "--test-threads=1"]
        assert command["exit_status"] == row["exit_status"]
        assert command["stdout_sha256"] == sha(key + ".stdout")
        assert command["stderr_sha256"] == sha(key + ".stderr")
        out = (ROOT / (key + ".stdout")).read_text()
        status = "ok" if row["exit_status"] == 0 else "FAILED"
        assert re.findall(r"^test (\S+) \.\.\. (ok|FAILED)$", out, re.M) == [(row["test"], status)]
        result = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;", out)
        assert len(result) == 1
        assert tuple(map(int, result[0])) == (int(status == "ok"), int(status == "FAILED"), 0, 0, len(listed) - 1)
        assert row["complete"]
        passes += int(status == "ok")
        failures += int(status == "FAILED")
assert (passes, failures) == (71, 3)
for key in ["format-check", "affected-clippy", "wasm32-check", "original-cycle-cli", "carbonate-acetic-cli", "bicarbonate-phosphoric-cli", "full-1536-sweep"]:
    command = read(key + ".json")
    assert command["exit_status"] == 0
    if key.endswith("cli") or key == "full-1536-sweep":
        for stream in ["stdout", "stderr"]:
            assert command[stream + "_sha256"] == sha(key + "." + stream)
    if key.endswith("cli"):
        rows = [json.loads(line) for line in (ROOT / (key + ".stdout")).read_text().splitlines() if line.startswith("{")]
        assert rows
        assert not [e for row in rows for e in row.get("events", []) if e.get("event") == "solver_failed"]
sweep = (ROOT / "full-1536-sweep.stdout").read_text()
assert "1536 states" in sweep and "1536 ran" in sweep and "no invariant was violated" in sweep
print("Verified saved outcomes: 71 passed, 3 failed; compiler/CLI/sweep gates passed. No native execution or new qualification.")
