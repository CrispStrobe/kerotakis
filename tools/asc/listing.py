#!/usr/bin/env python3
"""Push the App Store listing — the half `testflight.py` does not do.

`testflight.py` takes a build to external beta. This takes the APP to a
submittable state: the copy a shopper reads, the category it is filed
under, and the price. Both read the same `metadata.json`, so the words
live in a diff and not in a web form somebody typed once.

Everything is idempotent: PATCH where the resource exists, POST where it
does not, and re-read afterwards. Run it again after the next upload and
it changes only what moved.

The order matters less here than in TestFlight, but two things are worth
knowing before the first run:

  * Categories live on `appInfos`, not on the app and not on the version.
    They are the same for every platform — one app, one filing.
  * A version localisation is per PLATFORM. iOS and macOS have separate
    `appStoreVersions`, so the same description is written twice, and
    forgetting the second is why one platform sits half-filled.

What this deliberately does NOT do, because no API key can:

  * The App Privacy "nutrition label". `appstore.md` is explicit that it
    needs a browser, and an Admin-role key is refused.
  * Screenshots. They are real images and belong with the tooling that
    makes them (`appstore.md` Step 11, iOS Simulator).
  * Age rating, which wants a questionnaire rather than a field.

Usage:
    python3 tools/asc/listing.py            # both platforms
    python3 tools/asc/listing.py ios
    python3 tools/asc/listing.py --dry-run  # print what would change
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
META = json.loads((HERE / "metadata.json").read_text())
LOCALE = META["primaryLocale"]
APP = META["appId"]
PLATFORM = {"ios": "IOS", "macos": "MAC_OS"}

DRY = False


def change(method: str, path: str, body: dict, what: str) -> None:
    """One write, or one line saying what the write would have been."""
    if DRY:
        print(f"   would {method} {what}")
        return
    client.expect(method, path, body)
    print(f"   {what}")


def versions() -> list[dict]:
    return list(client.paged(f"/v1/apps/{APP}/appStoreVersions"))


def version_localisation(version: dict) -> None:
    """Description, keywords, and the three URLs, per platform."""
    app = META["app"]
    platform = version["attributes"]["platform"]
    attrs = {
        "description": app["description"],
        "keywords": app["keywords"],
        "supportUrl": app["supportUrl"],
        "marketingUrl": app["marketingUrl"],
        "promotionalText": app["promotionalText"],
    }
    existing = {
        loc["attributes"]["locale"]: loc
        for loc in client.paged(
            f"/v1/appStoreVersions/{version['id']}/appStoreVersionLocalizations"
        )
    }
    if LOCALE in existing:
        change(
            "PATCH",
            f"/v1/appStoreVersionLocalizations/{existing[LOCALE]['id']}",
            {"data": {"type": "appStoreVersionLocalizations",
                      "id": existing[LOCALE]["id"], "attributes": attrs}},
            f"{platform} listing ({LOCALE}): updated",
        )
    else:
        change(
            "POST",
            "/v1/appStoreVersionLocalizations",
            {"data": {"type": "appStoreVersionLocalizations",
                      "attributes": {**attrs, "locale": LOCALE},
                      "relationships": {"appStoreVersion": {
                          "data": {"type": "appStoreVersions", "id": version["id"]}}}}},
            f"{platform} listing ({LOCALE}): created",
        )


def version_copyright(version: dict) -> None:
    platform = version["attributes"]["platform"]
    if version["attributes"].get("copyright") == META["app"]["copyright"]:
        print(f"   {platform} copyright: already set")
        return
    change(
        "PATCH",
        f"/v1/appStoreVersions/{version['id']}",
        {"data": {"type": "appStoreVersions", "id": version["id"],
                  "attributes": {"copyright": META["app"]["copyright"]}}},
        f"{platform} copyright: set",
    )


def categories() -> None:
    """Primary and secondary category, on the editable `appInfo`.

    An app has several `appInfos` and only the one that is not yet on the
    store may be edited; the live one rejects the PATCH. Picking by state
    rather than by position is the difference between this working and
    working until the first release.
    """
    infos = list(client.paged(f"/v1/apps/{APP}/appInfos"))
    editable = [
        i for i in infos
        if i["attributes"].get("appStoreState") not in ("READY_FOR_SALE", "REPLACED_WITH_NEW_VERSION")
    ]
    if not editable:
        print("   categories: no editable appInfo (all live) — skipped")
        return
    info = editable[0]
    app = META["app"]
    change(
        "PATCH",
        f"/v1/appInfos/{info['id']}",
        {"data": {"type": "appInfos", "id": info["id"], "relationships": {
            "primaryCategory": {"data": {"type": "appCategories", "id": app["primaryCategory"]}},
            "secondaryCategory": {"data": {"type": "appCategories", "id": app["secondaryCategory"]}},
        }}},
        f"categories: {app['primaryCategory']} / {app['secondaryCategory']}",
    )


def app_localisation() -> None:
    """Name, subtitle and privacy policy URL — app-level, not per version."""
    infos = list(client.paged(f"/v1/apps/{APP}/appInfos"))
    editable = [
        i for i in infos
        if i["attributes"].get("appStoreState") not in ("READY_FOR_SALE", "REPLACED_WITH_NEW_VERSION")
    ]
    if not editable:
        print("   name/subtitle: no editable appInfo — skipped")
        return
    app = META["app"]
    attrs = {
        "name": app["name"],
        "subtitle": app["subtitle"],
        "privacyPolicyUrl": app["privacyPolicyUrl"],
    }
    existing = {
        loc["attributes"]["locale"]: loc
        for loc in client.paged(f"/v1/appInfos/{editable[0]['id']}/appInfoLocalizations")
    }
    if LOCALE in existing:
        change(
            "PATCH",
            f"/v1/appInfoLocalizations/{existing[LOCALE]['id']}",
            {"data": {"type": "appInfoLocalizations",
                      "id": existing[LOCALE]["id"], "attributes": attrs}},
            f"name/subtitle/privacy ({LOCALE}): updated",
        )
    else:
        change(
            "POST",
            "/v1/appInfoLocalizations",
            {"data": {"type": "appInfoLocalizations",
                      "attributes": {**attrs, "locale": LOCALE},
                      "relationships": {"appInfo": {
                          "data": {"type": "appInfos", "id": editable[0]["id"]}}}}},
            f"name/subtitle/privacy ({LOCALE}): created",
        )


def review_detail() -> None:
    """The contact Apple's reviewer calls, distinct from the BETA one."""
    versions_ = versions()
    r = META["review"]
    for v in versions_:
        platform = v["attributes"]["platform"]
        # `call`, not `expect`: a version with no review detail yet answers
        # 404, and that is the answer rather than a failure.
        status, existing = client.call(
            "GET", f"/v1/appStoreVersions/{v['id']}/appStoreReviewDetail"
        )
        attrs = {
            "contactFirstName": r["firstName"],
            "contactLastName": r["lastName"],
            "contactEmail": r["email"],
            "contactPhone": r["phone"],
            "demoAccountRequired": r["demoAccountRequired"],
            "notes": r["notes"],
        }
        if status == 200 and existing.get("data"):
            did = existing["data"]["id"]
            change("PATCH", f"/v1/appStoreReviewDetails/{did}",
                   {"data": {"type": "appStoreReviewDetails", "id": did, "attributes": attrs}},
                   f"{platform} review contact: updated")
        else:
            change("POST", "/v1/appStoreReviewDetails",
                   {"data": {"type": "appStoreReviewDetails", "attributes": attrs,
                             "relationships": {"appStoreVersion": {
                                 "data": {"type": "appStoreVersions", "id": v["id"]}}}}},
                   f"{platform} review contact: created")


def report() -> None:
    """Re-read, because a 201 is not proof — the same rule as TestFlight."""
    print("\n   after:")
    for v in versions():
        platform = v["attributes"]["platform"]
        locs = list(client.paged(
            f"/v1/appStoreVersions/{v['id']}/appStoreVersionLocalizations"))
        for loc in locs:
            a = loc["attributes"]
            filled = [k for k in ("description", "keywords", "supportUrl",
                                  "marketingUrl", "promotionalText") if a.get(k)]
            print(f"     {platform} {a['locale']}: {', '.join(filled) or 'empty'}")
        print(f"     {platform} copyright: {v['attributes'].get('copyright') or 'unset'}")


def main() -> int:
    global DRY
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("platform", nargs="?", choices=["ios", "macos"],
                    help="only this platform's version (default: both)")
    ap.add_argument("--dry-run", action="store_true",
                    help="print what would change and write nothing")
    args = ap.parse_args()
    DRY = args.dry_run

    print(f"Kerotakis listing ({APP}){' — DRY RUN' if DRY else ''}")
    app_localisation()
    categories()

    wanted = PLATFORM.get(args.platform) if args.platform else None
    for v in versions():
        if wanted and v["attributes"]["platform"] != wanted:
            continue
        version_localisation(v)
        version_copyright(v)
    review_detail()
    if not DRY:
        report()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
