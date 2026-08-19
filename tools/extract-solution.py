#!/usr/bin/env python3
"""Pull the last characterised solution out of a `kero run --json` stream.

Exists so CI can record the desktop build's answer to a question and hand it
to the browser test, which then has something to *disagree* with rather than
only a plausibility check. One engine reached by one path is the whole claim;
this is what tests it.
"""
import json
import sys

last = None
for line in sys.stdin:
    try:
        doc = json.loads(line)
    except json.JSONDecodeError:
        continue
    for vessel in doc.get("bench", {}).get("vessels", []):
        if vessel.get("solution"):
            last = vessel["solution"]

if last is None:
    print("no solution was characterised", file=sys.stderr)
    sys.exit(1)
json.dump({"ph": last["ph"], "ionic_strength": last["ionic_strength"]}, sys.stdout)
