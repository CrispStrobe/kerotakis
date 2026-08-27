#!/usr/bin/env python3
"""Write one provisioning profile to disk, by name.

`profileContent` from the API already IS the base64 of the
`.mobileprovision` / `.provisionprofile` bytes, so there is no download
endpoint to chase.

Usage: python3 tools/asc/fetch-profile.py "<profile name>" <output path>
"""

import base64
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402


def main() -> int:
    name, out = sys.argv[1], pathlib.Path(sys.argv[2])
    profiles = client.paged("/v1/profiles?limit=200")
    match = [p for p in profiles if p["attributes"]["name"] == name]
    if not match:
        known = sorted(p["attributes"]["name"] for p in profiles)
        print(f"no profile named {name!r}. Known:", file=sys.stderr)
        for k in known:
            print(f"  {k}", file=sys.stderr)
        return 1
    a = match[0]["attributes"]
    if a["profileState"] != "ACTIVE":
        # An INVALID profile still returns content, and signing with it
        # fails much later with a far worse error message.
        print(f"profile {name!r} is {a['profileState']}, not ACTIVE", file=sys.stderr)
        return 1
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(base64.b64decode(a["profileContent"]))
    print(f"   {name}: {a['profileType']}, expires {a['expirationDate']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
