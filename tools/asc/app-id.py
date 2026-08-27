#!/usr/bin/env python3
"""Print the numeric App Store Connect app id for Kerotakis.

`altool --apple-id` wants this number, not the bundle id, and uploading
with the wrong one does not fail — it silently lands the build in whatever
app that number names. So it is resolved from the API by bundle id every
time rather than read from a secret that can go stale, and the value
recorded in metadata.json is treated as a cross-check, not a source.
"""

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402

META = json.loads((pathlib.Path(__file__).resolve().parent / "metadata.json").read_text())
bundle = META["bundleId"]

resolved = client.app_id(bundle)
if not resolved:
    print(
        f"no App Store Connect record for {bundle}.\n"
        "Creating one is the single step Apple does not expose over the API "
        "(POST /v1/apps returns 403 for any key, Admin included).\n"
        "See PACKAGING.md for the browser steps.",
        file=sys.stderr,
    )
    raise SystemExit(1)

recorded = META.get("appId")
if recorded and recorded != resolved:
    print(
        f"metadata.json records appId {recorded}, but {bundle} is actually "
        f"{resolved}. Fix metadata.json before uploading anything.",
        file=sys.stderr,
    )
    raise SystemExit(1)

print(resolved)
