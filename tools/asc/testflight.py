#!/usr/bin/env python3
"""Take an uploaded build all the way to external TestFlight.

Everything here is idempotent: run it again after a second upload and it
re-uses the groups, the contact and the localisations, and only does the
per-build work. The order is not arbitrary — each step is a precondition
Apple enforces with an error that does not name the missing step:

  1. export compliance   a VALID build is "not in an internally testable
                         state" until `usesNonExemptEncryption` is set,
                         and that is NOT an external-only requirement.
  2. beta review detail   PATCH-only, exists as soon as the app does, and
                         rejects the whole request without `contactPhone`.
  3. beta localisation    the app-level description + feedback email +
                         privacy policy URL that Beta App Review reads.
  4. what to test         per build, per locale.
  5. groups               internal needs no review at all; external needs
                         review but gets a public join link for free.
  6. submit               and then re-read the submission, because a 201
                         is not proof: a build that fails re-validation
                         has its submission silently rolled back.

Usage:
    python3 tools/asc/testflight.py ios          # newest iOS build
    python3 tools/asc/testflight.py macos        # newest macOS build
    python3 tools/asc/testflight.py ios --build 12345
    python3 tools/asc/testflight.py ios --internal-only
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
PLATFORM = {"ios": "IOS", "macos": "MAC_OS"}


def find_build(app: str, platform: str, build_id: str | None) -> dict:
    if build_id:
        return client.expect("GET", f"/v1/builds/{build_id}")["data"]
    builds = client.paged(
        f"/v1/builds?filter[app]={app}"
        f"&filter[preReleaseVersion.platform]={platform}"
        f"&sort=-version&limit=200"
    )
    if not builds:
        raise SystemExit(
            f"no {platform} builds for this app yet. Upload one first "
            f"(tools/build-macos-appstore.sh or tools/build-ios-appstore.sh)."
        )
    live = [b for b in builds if b["attributes"]["processingState"] != "EXPIRED"]
    if not live:
        raise SystemExit(
            f"every {platform} build is EXPIRED. Expiry is permanent — "
            f"bump the build number and upload again."
        )
    return live[0]


def export_compliance(build: dict) -> None:
    if build["attributes"].get("usesNonExemptEncryption") is False:
        print("   export compliance: already answered")
        return
    client.expect(
        "PATCH",
        f"/v1/builds/{build['id']}",
        {"data": {"type": "builds", "id": build["id"],
                  "attributes": {"usesNonExemptEncryption": False}}},
    )
    print("   export compliance: usesNonExemptEncryption = false")


def review_detail(app: str) -> None:
    doc = client.expect("GET", f"/v1/apps/{app}/betaAppReviewDetail")
    detail_id = doc["data"]["id"]
    r = META["review"]
    client.expect(
        "PATCH",
        f"/v1/betaAppReviewDetails/{detail_id}",
        {"data": {"type": "betaAppReviewDetails", "id": detail_id, "attributes": {
            "contactFirstName": r["firstName"],
            "contactLastName": r["lastName"],
            "contactEmail": r["email"],
            # Omitting this returns 409 ENTITY_ERROR.ATTRIBUTE.REQUIRED and
            # rejects every other field in the same request.
            "contactPhone": r["phone"],
            "demoAccountRequired": r["demoAccountRequired"],
            "notes": r["notes"],
        }}},
    )
    print("   beta app review contact: set")


def app_localization(app: str) -> None:
    b = META["beta"]
    existing = {
        loc["attributes"]["locale"]: loc
        for loc in client.paged(f"/v1/apps/{app}/betaAppLocalizations")
    }
    attrs = {
        "description": b["description"],
        "feedbackEmail": b["feedbackEmail"],
        "privacyPolicyUrl": META["app"]["privacyPolicyUrl"],
    }
    if LOCALE in existing:
        loc_id = existing[LOCALE]["id"]
        client.expect("PATCH", f"/v1/betaAppLocalizations/{loc_id}",
                      {"data": {"type": "betaAppLocalizations", "id": loc_id,
                                "attributes": attrs}})
        print(f"   beta app localization ({LOCALE}): updated")
    else:
        client.expect("POST", "/v1/betaAppLocalizations",
                      {"data": {"type": "betaAppLocalizations",
                                "attributes": {**attrs, "locale": LOCALE},
                                "relationships": {"app": {"data": {"type": "apps", "id": app}}}}})
        print(f"   beta app localization ({LOCALE}): created")


def build_localization(build_id: str) -> None:
    existing = {
        loc["attributes"]["locale"]: loc
        for loc in client.paged(f"/v1/builds/{build_id}/betaBuildLocalizations")
    }
    whats_new = META["beta"]["whatToTest"]
    if LOCALE in existing:
        loc_id = existing[LOCALE]["id"]
        client.expect("PATCH", f"/v1/betaBuildLocalizations/{loc_id}",
                      {"data": {"type": "betaBuildLocalizations", "id": loc_id,
                                "attributes": {"whatsNew": whats_new}}})
        print(f"   what to test ({LOCALE}): updated")
    else:
        client.expect("POST", "/v1/betaBuildLocalizations",
                      {"data": {"type": "betaBuildLocalizations",
                                "attributes": {"whatsNew": whats_new, "locale": LOCALE},
                                "relationships": {"build": {"data": {"type": "builds", "id": build_id}}}}})
        print(f"   what to test ({LOCALE}): created")


def ensure_group(app: str, name: str, internal: bool) -> dict:
    for g in client.paged(f"/v1/apps/{app}/betaGroups"):
        if g["attributes"]["name"] == name:
            link = g["attributes"].get("publicLink")
            print(f"   group {name!r}: exists{f' — {link}' if link else ''}")
            return g
    attrs = {"name": name, "isInternalGroup": internal}
    if not internal:
        # Free, and immediate: Apple mints the join link on creation.
        attrs["publicLinkEnabled"] = True
    doc = client.expect("POST", "/v1/betaGroups",
                        {"data": {"type": "betaGroups", "attributes": attrs,
                                  "relationships": {"app": {"data": {"type": "apps", "id": app}}}}})
    link = doc["data"]["attributes"].get("publicLink")
    print(f"   group {name!r}: created{f' — {link}' if link else ''}")
    return doc["data"]


def assign(group: dict, build_id: str) -> None:
    name = group["attributes"]["name"]
    assigned = {b["id"] for b in client.paged(f"/v1/betaGroups/{group['id']}/builds")}
    if build_id in assigned:
        print(f"   {name}: build already assigned")
        return
    status, doc = client.call(
        "POST", f"/v1/betaGroups/{group['id']}/relationships/builds",
        {"data": [{"type": "builds", "id": build_id}]},
    )
    if status not in (200, 201, 204):
        for e in doc.get("errors", []):
            print(f"   {name}: {e.get('code')}: {e.get('detail')}")
        raise SystemExit(1)
    print(f"   {name}: build assigned")


def submit_for_beta_review(build_id: str) -> None:
    already = client.paged(f"/v1/betaAppReviewSubmissions?filter[build]={build_id}")
    if already:
        print(f"   beta review: already submitted, state "
              f"{already[0]['attributes']['betaReviewState']}")
        return
    status, doc = client.call(
        "POST", "/v1/betaAppReviewSubmissions",
        {"data": {"type": "betaAppReviewSubmissions",
                  "relationships": {"build": {"data": {"type": "builds", "id": build_id}}}}},
    )
    if status != 201:
        for e in doc.get("errors", []):
            code = e.get("code", "")
            print(f"   beta review: {code}: {e.get('detail')}")
            if "ANOTHER_BUILD_IN_REVIEW" in code:
                print("     (only one build per train may be in beta review "
                      "at a time — this is correct behaviour, not a fault)")
            if "CLOSED_VERSION" in code:
                print("     (that version has already shipped; bump "
                      "CFBundleShortVersionString and upload again)")
        raise SystemExit(1)
    print(f"   beta review: submitted, {doc['data']['attributes']['betaReviewState']}")

    # A 201 is not proof. Submitting kicks off a re-validation, and a build
    # that fails it has its submission rolled back with no notification.
    confirmed = client.paged(f"/v1/betaAppReviewSubmissions?filter[build]={build_id}")
    if not confirmed:
        raise SystemExit(
            "   the submission was ROLLED BACK — Apple re-validated the build "
            "and it failed.\n"
            "   processingState INVALID has no reason exposed over the API; "
            "the cause is only in\n"
            "   the email Apple sends the account holder and on the build row "
            "in TestFlight's web UI."
        )
    print("   beta review: submission confirmed on re-read")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("platform", choices=sorted(PLATFORM))
    ap.add_argument("--build", help="a specific build id (default: the newest live one)")
    ap.add_argument("--internal-only", action="store_true",
                    help="stop after the internal group; no Beta App Review")
    args = ap.parse_args()

    app = client.app_id(META["bundleId"])
    if not app:
        raise SystemExit(
            f"no App Store Connect record for {META['bundleId']}.\n"
            "Creating one is the single step Apple does not expose over the "
            "API (POST /v1/apps returns 403 FORBIDDEN_ERROR for any key, "
            "including Admin).\n"
            "See PACKAGING.md for the exact browser steps."
        )
    print(f"app {META['app']['name']} ({META['bundleId']}) = {app}")

    build = find_build(app, PLATFORM[args.platform], args.build)
    a = build["attributes"]
    print(f"build {build['id']}: version {a['version']}, "
          f"{a['processingState']}, uploaded {a['uploadedDate']}")
    if a["processingState"] != "VALID":
        print(f"   still {a['processingState']} — processing usually takes "
              f"15 to 60 minutes. Nothing below will stick until it is VALID.")
        return 1

    print("\n== the build")
    export_compliance(build)
    build_localization(build["id"])

    print("\n== the app")
    review_detail(app)
    app_localization(app)

    print("\n== groups")
    internal = ensure_group(app, META["groups"]["internal"], internal=True)
    assign(internal, build["id"])
    if not client.paged(f"/v1/betaGroups/{internal['id']}/betaTesters"):
        # Internal groups may only contain existing App Store Connect team
        # members, so the address has to be the one their ASC account uses —
        # which is not derivable from anything here.
        print("   note: the internal group has no testers. Add one with\n"
              "     python3 tools/asc/client.py POST /v1/betaTesters "
              "'{\"data\":{\"type\":\"betaTesters\",\"attributes\":"
              "{\"email\":\"<their ASC account email>\"},\"relationships\":"
              "{\"betaGroups\":{\"data\":[{\"type\":\"betaGroups\",\"id\":\""
              + internal["id"] + "\"}]}}}}'")
    if args.internal_only:
        print("\nOK: internal testing is live (internal builds need no review).")
        return 0

    external = ensure_group(app, META["groups"]["external"], internal=False)
    assign(external, build["id"])

    print("\n== external beta review")
    submit_for_beta_review(build["id"])

    link = external["attributes"].get("publicLink")
    print(f"\nOK. Public link: {link or '(re-read the group once Apple mints it)'}")
    print("Beta App Review is same-day-ish, unlike full App Store review.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
