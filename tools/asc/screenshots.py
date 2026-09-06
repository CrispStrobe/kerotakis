#!/usr/bin/env python3
"""Upload App Store screenshots.

Apple does not take an image in one call. Each file is a four-step
handshake, and every step fails differently:

  1. POST `/v1/appScreenshots` with the file NAME and SIZE only. The
     answer carries `uploadOperations` — a presigned URL, a method, and
     the exact headers to send. The bytes have not moved yet.
  2. PUT the raw bytes to that URL with exactly those headers.
  3. PATCH the resource with `uploaded: true` and the file's MD5. Apple
     checks the checksum; a mismatch fails here, not at the PUT.
  4. Poll until `assetDeliveryState.state` is `UPLOAD_COMPLETE`. Until
     then the screenshot exists and is not usable.

A set holds the images for one device class, hangs off ONE version
localisation, and is therefore per platform: the iPhone shots go on the
iOS version, the desktop shot on the macOS one. Sending an
`APP_IPHONE_67` set to the macOS version is accepted and then rejected at
submission, which is a long way to find out.

`appstore.md` records that `urllib` once died partway through the binary
PUT where `curl` did not, and that later attempts worked. This tries
`urllib` and falls back to `curl`, so a working machine stays fast and a
broken one still finishes.

Usage:
    python3 tools/asc/screenshots.py <dir>            # dir holds manifest.json
    python3 tools/asc/screenshots.py <dir> --replace  # delete existing sets first
    python3 tools/asc/screenshots.py <dir> --dry-run
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import time
import urllib.error
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
META = json.loads((HERE / "metadata.json").read_text())
LOCALE = META["primaryLocale"]
APP = META["appId"]

# Which platform's version localisation each display type belongs on.
PLATFORM_OF = {
    "APP_IPHONE_67": "IOS",
    "APP_IPHONE_65": "IOS",
    "APP_IPAD_PRO_3GEN_129": "IOS",
    "APP_IPAD_PRO_129": "IOS",
    "APP_DESKTOP": "MAC_OS",
}

DRY = False


def localisation_for(platform: str) -> str | None:
    """The version localisation a set for this platform hangs off."""
    for v in client.paged(f"/v1/apps/{APP}/appStoreVersions"):
        if v["attributes"]["platform"] != platform:
            continue
        for loc in client.paged(
            f"/v1/appStoreVersions/{v['id']}/appStoreVersionLocalizations"
        ):
            if loc["attributes"]["locale"] == LOCALE:
                return loc["id"]
    return None


def existing_sets(loc_id: str) -> dict[str, dict]:
    return {
        s["attributes"]["screenshotDisplayType"]: s
        for s in client.paged(f"/v1/appStoreVersionLocalizations/{loc_id}/appScreenshotSets")
    }


def ensure_set(loc_id: str, display_type: str, sets: dict) -> str:
    if display_type in sets:
        return sets[display_type]["id"]
    doc = client.expect("POST", "/v1/appScreenshotSets", {"data": {
        "type": "appScreenshotSets",
        "attributes": {"screenshotDisplayType": display_type},
        "relationships": {"appStoreVersionLocalization": {
            "data": {"type": "appStoreVersionLocalizations", "id": loc_id}}},
    }})
    print(f"   set {display_type}: created")
    return doc["data"]["id"]


def put_bytes(op: dict, blob: bytes) -> None:
    """The one step that is not JSON. urllib first, curl as the fallback."""
    url = op["url"]
    headers = {h["name"]: h["value"] for h in op.get("requestHeaders", [])}
    req = urllib.request.Request(url, data=blob, method=op.get("method", "PUT"))
    for k, v in headers.items():
        req.add_header(k, v)
    try:
        with urllib.request.urlopen(req) as r:
            if r.status not in (200, 201, 204):
                raise urllib.error.HTTPError(url, r.status, "unexpected", None, None)
        return
    except Exception as first:
        print(f"     urllib PUT failed ({type(first).__name__}); falling back to curl")

    cmd = ["curl", "-sS", "-X", op.get("method", "PUT"), "--data-binary", "@-", url]
    for k, v in headers.items():
        cmd += ["-H", f"{k}: {v}"]
    proc = subprocess.run(cmd, input=blob, capture_output=True)
    if proc.returncode != 0:
        raise SystemExit(f"curl PUT failed: {proc.stderr.decode(errors='replace')[:400]}")


def upload_one(set_id: str, path: pathlib.Path) -> None:
    blob = path.read_bytes()
    if DRY:
        print(f"     would upload {path.name} ({len(blob)} bytes)")
        return
    doc = client.expect("POST", "/v1/appScreenshots", {"data": {
        "type": "appScreenshots",
        "attributes": {"fileName": path.name, "fileSize": len(blob)},
        "relationships": {"appScreenshotSet": {
            "data": {"type": "appScreenshotSets", "id": set_id}}},
    }})
    shot_id = doc["data"]["id"]
    ops = doc["data"]["attributes"]["uploadOperations"]
    for op in ops:
        # A large file comes back as several operations, each its own slice.
        start, length = op.get("offset", 0), op.get("length", len(blob))
        put_bytes(op, blob[start:start + length])

    client.expect("PATCH", f"/v1/appScreenshots/{shot_id}", {"data": {
        "type": "appScreenshots", "id": shot_id,
        "attributes": {"uploaded": True,
                       "sourceFileChecksum": hashlib.md5(blob).hexdigest()},
    }})

    # It exists before it is usable, and only the delivery state says which.
    for _ in range(30):
        got = client.expect("GET", f"/v1/appScreenshots/{shot_id}")
        state = got["data"]["attributes"].get("assetDeliveryState", {})
        if state.get("state") == "UPLOAD_COMPLETE":
            print(f"     {path.name}: UPLOAD_COMPLETE")
            return
        if state.get("errors"):
            raise SystemExit(f"     {path.name}: {json.dumps(state['errors'])}")
        time.sleep(4)
    print(f"     {path.name}: still processing — check before submitting")


def main() -> int:
    global DRY
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("directory", help="holds the images and manifest.json")
    ap.add_argument("--replace", action="store_true",
                    help="delete existing sets for these display types first")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    DRY = args.dry_run

    root = pathlib.Path(args.directory)
    manifest = json.loads((root / "manifest.json").read_text())

    by_type: dict[str, list[pathlib.Path]] = {}
    for entry in manifest:
        by_type.setdefault(entry["displayType"], []).append(root / entry["name"])

    print(f"Kerotakis screenshots ({APP}){' — DRY RUN' if DRY else ''}")
    for display_type, paths in by_type.items():
        platform = PLATFORM_OF.get(display_type)
        if not platform:
            print(f"   {display_type}: unknown platform mapping — skipped")
            continue
        loc_id = localisation_for(platform)
        if not loc_id:
            print(f"   {display_type}: no {platform} {LOCALE} localisation — skipped")
            continue
        sets = existing_sets(loc_id)
        if args.replace and display_type in sets and not DRY:
            client.expect("DELETE", f"/v1/appScreenshotSets/{sets[display_type]['id']}",
                          ok=(200, 204))
            print(f"   set {display_type}: deleted")
            sets = existing_sets(loc_id)
        set_id = sets[display_type]["id"] if display_type in sets else (
            "(new)" if DRY else ensure_set(loc_id, display_type, sets))
        print(f"   {display_type} -> {platform}")
        for path in paths:
            upload_one(set_id, path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
