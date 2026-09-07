#!/usr/bin/env python3
"""Replay the preserved and repaired catalog entries through the real CLI.

Inputs are loaded from the catalog, not maintained as a second recipe copy.
Use the recorder's --out fresh-directory and --binary options. This script is
an evidence harness only; it is never part of the chemistry implementation.
"""
import tomllib

import run as recorder

entries = []
for path in sorted((recorder.ROOT / "codex").glob("*.toml")):
    entries.extend(tomllib.loads(path.read_text()).get("reaction", []))
selected = {
    "endpoint-is-not-a-full-drop", "equilibrium-can-run-backward",
    "cold-from-baking-soda",
}
recorder.CASES = [
    recorder.case(entry["id"], entry.get("summary") or entry.get("equation") or entry["id"],
                  entry["setup"]["script"])
    for entry in entries
    if entry["id"] in selected or "Na2S2O3" in entry["setup"]["script"]
]

if __name__ == "__main__":
    recorder.main()
