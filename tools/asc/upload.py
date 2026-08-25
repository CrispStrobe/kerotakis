#!/usr/bin/env python3
"""Validate and upload a signed artifact to App Store Connect.

Three things here are not incidental.

**Every version comes out of the artifact.** `altool` cross-checks the
arguments against the bundle's own Info.plist and refuses the whole upload
on any disagreement:

    Validation failed (409) Info.plist value mismatch. CFBundleVersion
    value of 0.0.1 does not match the value of 1 specified in the request.

Passing a build number the bundle does not carry is the easiest way to
produce that, so nothing is passed that was not read out of the artifact
first. The bundle id is read the same way and used to resolve the numeric
app id, because uploading with the wrong number does not fail — it lands
the build in a different app.

**`altool` exits 0 when the upload fails.** A plain `set -e` pipeline
reports success over "UPLOAD FAILED with 1 error". So the JSON output is
parsed and `product-errors` is what decides.

Usage: python3 tools/asc/upload.py <artifact.ipa|artifact.pkg> [--validate-only]
"""

from __future__ import annotations

import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402


def read_plist(data: bytes) -> dict:
    """Parse a plist through `plutil`, not `plistlib`.

    `plistlib` imports `pyexpat`, and a Homebrew Python whose pyexpat is
    linked against a different libexpat than the one it resolves at runtime
    fails at import with a missing symbol — taking this script down before
    it does anything. `plutil` ships with macOS, handles both the XML and
    binary encodings, and is already a hard dependency of everything else
    here.
    """
    with tempfile.NamedTemporaryFile(suffix=".plist") as tmp:
        tmp.write(data)
        tmp.flush()
        out = subprocess.run(["plutil", "-convert", "json", "-o", "-", tmp.name],
                             capture_output=True, check=True)
    return json.loads(out.stdout)


def plist_from_ipa(path: pathlib.Path) -> dict:
    with zipfile.ZipFile(path) as z:
        name = next(
            n for n in z.namelist()
            if n.count("/") == 2 and n.startswith("Payload/") and n.endswith(".app/Info.plist")
        )
        return read_plist(z.read(name))


def plist_from_pkg(path: pathlib.Path) -> dict:
    tmp = pathlib.Path(tempfile.mkdtemp()) / "expanded"
    subprocess.run(["pkgutil", "--expand-full", str(path), str(tmp)],
                   check=True, capture_output=True)
    try:
        info = next(tmp.rglob("Contents/Info.plist"))
        return read_plist(info.read_bytes())
    finally:
        shutil.rmtree(tmp.parent, ignore_errors=True)


def altool(args: list[str]) -> tuple[bool, str]:
    """Run altool and decide whether it actually worked."""
    proc = subprocess.run(
        ["xcrun", "altool", *args, "--output-format", "json"],
        capture_output=True, text=True,
    )
    raw = proc.stdout.strip() or proc.stderr.strip()
    try:
        doc = json.loads(raw)
    except ValueError:
        # No JSON at all: fall back to the exit code and the raw text.
        return proc.returncode == 0, raw

    errors = doc.get("product-errors") or []
    if errors:
        return False, "\n".join(
            f"  {e.get('code', '')}: {e.get('message', '')}" for e in errors
        )
    if proc.returncode != 0:
        return False, raw
    return True, doc.get("success-message", "ok")


def main() -> int:
    artifact = pathlib.Path(sys.argv[1])
    validate_only = "--validate-only" in sys.argv

    if artifact.suffix == ".ipa":
        kind, info = "ios", plist_from_ipa(artifact)
    elif artifact.suffix == ".pkg":
        kind, info = "macos", plist_from_pkg(artifact)
    else:
        raise SystemExit(f"expected a .ipa or .pkg, got {artifact.name}")

    bundle_id = info["CFBundleIdentifier"]
    short = info["CFBundleShortVersionString"]
    build = info["CFBundleVersion"]
    app_id = client.app_id(bundle_id)
    if not app_id:
        raise SystemExit(f"no App Store Connect record for {bundle_id}")

    print(f"{artifact.name}  ({artifact.stat().st_size / 1e6:.1f} MB)")
    print(f"   bundle          {bundle_id}  ->  app {app_id}")
    print(f"   version         {short}  (build {build})")
    print(f"   platform        {kind}")

    common = [
        "--type", kind,
        "--apple-id", app_id,
        "--bundle-id", bundle_id,
        "--bundle-short-version-string", short,
        "--bundle-version", build,
        "--api-key", client.KEY_ID,
        "--api-issuer", client.ISSUER_ID,
    ]

    print("\n== validate")
    ok, message = altool(["--validate-app", "-f", str(artifact), *common])
    print(message if ok else f"validation FAILED:\n{message}")
    if not ok:
        return 1
    if validate_only:
        print("\nOK (validated, not uploaded)")
        return 0

    print("\n== upload")
    ok, message = altool(["--upload-package", str(artifact), *common])
    print(message if ok else f"upload FAILED:\n{message}")
    if not ok:
        return 1

    print(f"\nOK: uploaded. Processing takes 15-60 minutes; the build is not "
          f"assignable to any TestFlight group until it reports VALID.")
    print(f"Then: python3 tools/asc/testflight.py {kind}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
