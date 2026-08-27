#!/usr/bin/env python3
"""Print the SHA-1 of the local signing identity that App Store Connect
still considers live, for a given certificate type.

Signing by NAME is the trap this exists to close. Certificate names are not
unique — this account has two "3rd Party Mac Developer Installer: Christian
Ströbele (N9XSJ4M3GT)" identities in two different keychains, five minutes
apart in issue date, and only one of them exists in App Store Connect at
all. `codesign`/`productbuild` given the name either refuse outright:

    "<name>": ambiguous (matches "<name>" in keychainA and in keychainB)

or, with one keychain on the search list, silently pick the dead one and
produce an artifact Apple rejects on upload. Matching on the fingerprint,
against what the API says is live, cannot do either.

Usage:
    python3 tools/asc/resolve-identity.py DISTRIBUTION
    python3 tools/asc/resolve-identity.py MAC_INSTALLER_DISTRIBUTION
"""

from __future__ import annotations

import base64
import hashlib
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402


def local_identities() -> dict[str, str]:
    """{sha1: display name} for every identity in the keychain search list.

    `-p codesigning` is deliberately NOT passed: installer identities are
    not code-signing identities and would be filtered out.
    """
    out = subprocess.run(["security", "find-identity", "-v"],
                         capture_output=True, text=True).stdout
    return {m[1]: m[2] for m in re.finditer(r'^\s*\d+\)\s+([0-9A-F]{40})\s+"(.+)"$',
                                            out, re.MULTILINE)}


def live_fingerprints(cert_type: str) -> dict[str, str]:
    """{sha1: display name} for every certificate of `cert_type` in the account."""
    found = {}
    for cert in client.paged("/v1/certificates?limit=200"):
        a = cert["attributes"]
        if a["certificateType"] != cert_type:
            continue
        der = base64.b64decode(a["certificateContent"])
        found[hashlib.sha1(der).hexdigest().upper()] = f"{a['displayName']} ({cert['id']})"
    return found


def main() -> int:
    cert_type = sys.argv[1]
    local = local_identities()
    live = live_fingerprints(cert_type)
    usable = {sha: local[sha] for sha in local if sha in live}

    if not usable:
        print(f"no local identity matches a live {cert_type} certificate.", file=sys.stderr)
        print("  live in App Store Connect:", file=sys.stderr)
        for sha, name in live.items():
            print(f"    {sha}  {name}", file=sys.stderr)
        print("  in the local keychains:", file=sys.stderr)
        for sha, name in local.items():
            print(f"    {sha}  {name}", file=sys.stderr)
        print("\n  Import the canonical .p12 — do NOT mint a new certificate; the\n"
              "  account cap is enforced by revoking one another app depends on.",
              file=sys.stderr)
        return 1

    if len(usable) > 1:
        print(f"{len(usable)} local identities match live {cert_type} certificates; "
              f"pick one explicitly:", file=sys.stderr)
        for sha, name in usable.items():
            print(f"    {sha}  {name}", file=sys.stderr)
        return 1

    sha, name = next(iter(usable.items()))
    print(sha)
    print(f"   {name}  [{cert_type}]", file=sys.stderr)
    stale = {s: n for s, n in local.items() if n == name and s != sha}
    for s, n in stale.items():
        print(f"   ignoring a same-named identity App Store Connect does not "
              f"know: {s}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
