#!/usr/bin/env python3
"""Extract the PC-SAFT records Kerotakis uses from FeOs parameter files.

Downloads (or reads from --upstream-dir) esper2023.json and
rehner2023_binary.json at the pinned feos-org/feos commit, verifies the
full-file sha256 values, and writes only the methanol/ethanol/water pure
records and the methanol-water / ethanol-water binary records to
crates/kerotakis-feos/data/*.subset.json. Each subset is
{"_source": {...}, "records": [...]}, with the records copied unchanged.

Usage: python3 tools/extract_feos_params.py [--upstream-dir DIR]
"""
import argparse
import hashlib
import json
import pathlib
import sys
import urllib.request

COMMIT = "aabc02bbf6c469cf702206cfa68bbb7b2105a324"
FILES = {
    "esper2023.json": "87fdcd8733245d90cd6ad9e9d7d0583bd3f7ffb3c4d67a25197190f289e2a679",
    "rehner2023_binary.json": "8b7d008b2bdf7ec49542eb443e36078fbe46e2737775126589c1718dfa13c335",
}
PURES = ("methanol", "ethanol", "water")
OUT = pathlib.Path(__file__).resolve().parent.parent / "crates" / "kerotakis-feos" / "data"


def load(name, upstream_dir):
    if upstream_dir:
        data = (pathlib.Path(upstream_dir) / name).read_bytes()
    else:
        url = f"https://raw.githubusercontent.com/feos-org/feos/{COMMIT}/parameters/pcsaft/{name}"
        with urllib.request.urlopen(url, timeout=60) as r:
            data = r.read()
    digest = hashlib.sha256(data).hexdigest()
    if digest != FILES[name]:
        sys.exit(f"{name}: sha256 {digest} != pinned {FILES[name]}")
    return json.loads(data)


def is_pair(r):
    names = {r["id1"].get("name"), r["id2"].get("name")}
    return "water" in names and bool(names & {"methanol", "ethanol"})


def write(name, records, upstream):
    body = {
        "_source": {
            "upstream": f"feos-org/feos@{COMMIT}:parameters/pcsaft/{upstream}",
            "upstream_sha256": FILES[upstream],
            "extracted_by": "tools/extract_feos_params.py",
            "note": "records copied unchanged; do not edit by hand",
        },
        "records": records,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / name
    path.write_text(json.dumps(body, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(path, hashlib.sha256(path.read_bytes()).hexdigest())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--upstream-dir")
    a = ap.parse_args()
    pures = [r for r in load("esper2023.json", a.upstream_dir) if r["identifier"].get("name") in PURES]
    binaries = [r for r in load("rehner2023_binary.json", a.upstream_dir) if is_pair(r)]
    if sorted(r["identifier"]["name"] for r in pures) != sorted(PURES) or len(binaries) != 2:
        sys.exit(f"unexpected record counts: {len(pures)} pure, {len(binaries)} binary")
    write("esper2023.subset.json", pures, "esper2023.json")
    write("rehner2023_binary.subset.json", binaries, "rehner2023_binary.json")


if __name__ == "__main__":
    main()
