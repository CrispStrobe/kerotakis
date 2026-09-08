#!/usr/bin/env python3
"""Read-only App Store Connect release audit for Kerotakis."""

from __future__ import annotations

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
META = json.loads((HERE / "metadata.json").read_text())
APP = META["appId"]


def attrs(item: dict, *names: str) -> dict:
    source = item.get("attributes", {})
    return {name: source.get(name) for name in names}


def main() -> int:
    app = client.expect("GET", f"/v1/apps/{APP}")["data"]
    print("app", APP, attrs(app, "name", "bundleId", "primaryLocale", "contentRightsDeclaration"))

    print("\nversions")
    versions = client.paged(f"/v1/apps/{APP}/appStoreVersions?limit=50")
    for version in versions:
        platform = version["attributes"].get("platform")
        print(" ", version["id"], attrs(version, "platform", "versionString", "appStoreState", "copyright"))
        status, build = client.call("GET", f"/v1/appStoreVersions/{version['id']}/build")
        print("    build", status, attrs(build.get("data") or {}, "version", "processingState"))
        for loc in client.paged(
            f"/v1/appStoreVersions/{version['id']}/appStoreVersionLocalizations?limit=50"
        ):
            filled = [
                name
                for name in ("description", "keywords", "supportUrl", "marketingUrl", "promotionalText")
                if loc.get("attributes", {}).get(name)
            ]
            sets = client.paged(
                f"/v1/appStoreVersionLocalizations/{loc['id']}/appScreenshotSets?limit=50"
            )
            counts = {}
            for screenshot_set in sets:
                shots = client.paged(
                    f"/v1/appScreenshotSets/{screenshot_set['id']}/appScreenshots?limit=50"
                )
                counts[screenshot_set["attributes"].get("screenshotDisplayType")] = len(shots)
            print("   ", platform, loc["attributes"].get("locale"), "fields", filled, "screenshots", counts)

    print("\nbuilds")
    builds = client.paged(f"/v1/apps/{APP}/builds?limit=50")
    builds.sort(key=lambda item: item["attributes"].get("uploadedDate") or "", reverse=True)
    for build in builds[:12]:
        status, release = client.call("GET", f"/v1/builds/{build['id']}/preReleaseVersion")
        platform = (release.get("data") or {}).get("attributes", {}).get("platform") if status == 200 else None
        print(" ", build["id"], platform, attrs(build, "version", "uploadedDate", "processingState", "usesNonExemptEncryption"))

    print("\napp info")
    for info in client.paged(f"/v1/apps/{APP}/appInfos?limit=50"):
        print(" ", info["id"], attrs(info, "appStoreState"))
        for loc in client.paged(f"/v1/appInfos/{info['id']}/appInfoLocalizations?limit=50"):
            print("   ", attrs(loc, "locale", "name", "subtitle", "privacyPolicyUrl"))
        status, rating = client.call("GET", f"/v1/appInfos/{info['id']}/ageRatingDeclaration")
        print("    age rating", status, attrs(rating.get("data") or {}, "ageRatingOverride"))

    print("\nbeta groups")
    for group in client.paged(f"/v1/apps/{APP}/betaGroups?limit=50"):
        print(" ", group["id"], attrs(group, "name", "isInternalGroup", "publicLinkEnabled", "publicLink"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
