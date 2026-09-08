#!/usr/bin/env python3
"""Finish API-editable requirements and attach a platform build."""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
import urllib.parse

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402
import testflight  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
META = json.loads((HERE / "metadata.json").read_text())
APP = META["appId"]
PLATFORM = {"ios": "IOS", "macos": "MAC_OS"}


def query(path: str, **params: str) -> str:
    return f"{path}?{urllib.parse.urlencode(params)}"


def editable_version(platform: str) -> dict:
    versions = client.paged(f"/v1/apps/{APP}/appStoreVersions?limit=50")
    matches = [
        version
        for version in versions
        if version["attributes"].get("platform") == platform
        and version["attributes"].get("appStoreState") == "PREPARE_FOR_SUBMISSION"
    ]
    if len(matches) != 1:
        states = [
            (version["attributes"].get("platform"), version["attributes"].get("appStoreState"))
            for version in versions
        ]
        raise SystemExit(f"expected one editable {platform} version, found {len(matches)}: {states}")
    return matches[0]


def complete_age_rating() -> None:
    infos = client.paged(f"/v1/apps/{APP}/appInfos?limit=50")
    editable = [
        info for info in infos
        if info["attributes"].get("appStoreState") not in ("READY_FOR_SALE", "REPLACED_WITH_NEW_VERSION")
    ]
    if len(editable) != 1:
        raise SystemExit(f"expected one editable app info, found {len(editable)}")
    rating_id = editable[0]["id"]
    empty = {
        "advertising": False,
        "alcoholTobaccoOrDrugUseOrReferences": "NONE",
        "contests": "NONE",
        "gambling": False,
        "gamblingSimulated": "NONE",
        "gunsOrOtherWeapons": "NONE",
        "healthOrWellnessTopics": False,
        "lootBox": False,
        "medicalOrTreatmentInformation": "NONE",
        "messagingAndChat": False,
        "parentalControls": False,
        "profanityOrCrudeHumor": "NONE",
        "ageAssurance": False,
        "sexualContentGraphicAndNudity": "NONE",
        "sexualContentOrNudity": "NONE",
        "socialMedia": False,
        "socialMediaAgeRestricted": False,
        "horrorOrFearThemes": "NONE",
        "matureOrSuggestiveThemes": "NONE",
        "unrestrictedWebAccess": False,
        "userGeneratedContent": False,
        "violenceCartoonOrFantasy": "NONE",
        "violenceRealisticProlongedGraphicOrSadistic": "NONE",
        "violenceRealistic": "NONE",
    }
    client.expect(
        "PATCH",
        f"/v1/ageRatingDeclarations/{rating_id}",
        {"data": {"type": "ageRatingDeclarations", "id": rating_id, "attributes": empty}},
    )
    print("age rating questionnaire: no objectionable content")


def ensure_free_price() -> None:
    path = query(
        f"/v1/apps/{APP}/appPriceSchedule",
        include="manualPrices",
        **{"limit[manualPrices]": "50"},
    )
    status, schedule = client.call("GET", path)
    if status == 404:
        points = client.paged(query(
            f"/v1/apps/{APP}/appPricePoints",
            **{"filter[territory]": "USA", "limit": "200"},
        ))
        free = next(
            (point for point in points
             if str(point["attributes"].get("customerPrice")) in ("0", "0.0", "0.00")),
            None,
        )
        if not free:
            raise SystemExit("Apple returned no free USA app price point")
        client.expect("POST", "/v1/appPriceSchedules", {
            "data": {"type": "appPriceSchedules", "relationships": {
                "app": {"data": {"type": "apps", "id": APP}},
                "baseTerritory": {"data": {"type": "territories", "id": "USA"}},
                "manualPrices": {"data": [{"type": "appPrices", "id": "${price1}"}]},
            }},
            "included": [{
                "type": "appPrices", "id": "${price1}", "attributes": {"startDate": None},
                "relationships": {"appPricePoint": {"data": {
                    "type": "appPricePoints", "id": free["id"],
                }}},
            }],
        })
        print("price schedule: created free")
        return
    if status != 200:
        raise SystemExit(f"read price schedule -> HTTP {status}: {schedule}")

    schedule_id = schedule["data"]["id"]
    prices_status, prices = client.call("GET", query(
        f"/v1/appPriceSchedules/{schedule_id}/manualPrices",
        include="appPricePoint",
        **{
            "fields[appPrices]": "appPricePoint,startDate,endDate",
            "fields[appPricePoints]": "customerPrice",
            "filter[territory]": "USA",
            "limit": "200",
        },
    ))
    if prices_status != 200:
        raise SystemExit(f"read manual prices -> HTTP {prices_status}: {prices}")
    values = [
        item.get("attributes", {}).get("customerPrice")
        for item in prices.get("included", [])
        if item.get("type") == "appPricePoints"
    ]
    if not any(str(value) in ("0", "0.0", "0.00") for value in values):
        raise SystemExit(f"existing price schedule is not free: {values}")
    print("price schedule: verified free", values)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("platform", choices=sorted(PLATFORM))
    parser.add_argument("--build", help="specific build resource id (default: newest live build)")
    args = parser.parse_args()
    platform = PLATFORM[args.platform]
    version = editable_version(platform)
    build = testflight.find_build(APP, platform, args.build)
    if build["attributes"].get("processingState") != "VALID":
        raise SystemExit(f"build {build['id']} is not VALID")
    testflight.export_compliance(build)
    complete_age_rating()
    ensure_free_price()
    client.expect(
        "PATCH",
        f"/v1/appStoreVersions/{version['id']}/relationships/build",
        {"data": {"type": "builds", "id": build["id"]}},
    )
    print(
        f"attached {platform} build {build['attributes'].get('version')} "
        f"to version {version['attributes'].get('versionString')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
