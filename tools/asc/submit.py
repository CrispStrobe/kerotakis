#!/usr/bin/env python3
"""Submit a prepared App Store version for Apple's review.

This is the step the rest of `tools/asc/` stopped short of. `store.py` finishes
the API-editable requirements and attaches a build; everything after that — the
one action that puts the app in front of a human reviewer — had to be done by
hand in App Store Connect.

The order Apple's API wants is three calls, not one:

  1. POST /v1/reviewSubmissions        open a submission for app + platform
  2. POST /v1/reviewSubmissionItems    put the appStoreVersion inside it
  3. PATCH /v1/reviewSubmissions/{id}  attributes.submitted = true

Step 3 is the irreversible half, so it needs `--submit`. Without it this reads
the state and reports whether the submission *would* go through, which is the
useful thing to run first: a version missing screenshots or a build fails at
step 3 with Apple's own reason, and a rejected submission is visible on the
account.

Idempotent, like everything else here. An open submission for the platform is
reported and reused rather than duplicated — Apple permits only one at a time
and a second POST fails with a less obvious error than saying so plainly.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import client  # noqa: E402
import store  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent
META = json.loads((HERE / "metadata.json").read_text(encoding="utf-8"))
APP = META["appId"]

# A submission in one of these is finished with; anything else is still live
# and must be reused rather than duplicated.
CLOSED = {"COMPLETING", "COMPLETE", "CANCELING", "CANCELED"}


def open_submission(platform: str) -> dict | None:
    for sub in client.paged(f"/v1/apps/{APP}/reviewSubmissions?limit=50"):
        attrs = sub.get("attributes", {})
        if attrs.get("platform") != platform:
            continue
        if attrs.get("state") in CLOSED:
            continue
        return sub
    return None


def attached_build(version: dict) -> dict | None:
    status, doc = client.call("GET", f"/v1/appStoreVersions/{version['id']}/build")
    if status != 200:
        return None
    return doc.get("data")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("platform", choices=sorted(store.PLATFORM))
    parser.add_argument(
        "--submit", action="store_true",
        help="actually submit; without it this only reports readiness",
    )
    args = parser.parse_args()
    platform = store.PLATFORM[args.platform]

    version = store.editable_version(platform)
    version_string = version["attributes"].get("versionString")
    state = version["attributes"].get("appStoreState") or version["attributes"].get("state")
    print(f"{args.platform}: version {version_string}, state {state}")

    build = attached_build(version)
    if not build:
        raise SystemExit(
            f"   no build is attached to {version_string} — run "
            f"`python3 tools/asc/store.py {args.platform}` first"
        )
    print(f"   build {build['attributes'].get('version')} attached")

    existing = open_submission(platform)
    if existing:
        sub_state = existing["attributes"].get("state")
        print(f"   review submission {existing['id']} already open, state {sub_state}")
        if sub_state != "READY_FOR_REVIEW":
            print("   nothing to do: it is already with Apple")
            return 0
        submission = existing
    else:
        if not args.submit:
            print("   no submission open. Re-run with --submit to create one.")
            return 0
        submission = client.expect(
            "POST", "/v1/reviewSubmissions",
            {"data": {
                "type": "reviewSubmissions",
                "attributes": {"platform": platform},
                "relationships": {"app": {"data": {"type": "apps", "id": APP}}},
            }},
        )["data"]
        print(f"   opened review submission {submission['id']}")

    items = client.paged(
        f"/v1/reviewSubmissions/{submission['id']}/items?limit=50"
    )
    if not any(
        (i.get("relationships", {}).get("appStoreVersion", {}).get("data") or {}).get("id")
        == version["id"]
        for i in items
    ):
        if not args.submit:
            print("   the version is not in the submission yet. Re-run with --submit.")
            return 0
        client.expect(
            "POST", "/v1/reviewSubmissionItems",
            {"data": {
                "type": "reviewSubmissionItems",
                "relationships": {
                    "reviewSubmission": {
                        "data": {"type": "reviewSubmissions", "id": submission["id"]}},
                    "appStoreVersion": {
                        "data": {"type": "appStoreVersions", "id": version["id"]}},
                },
            }},
        )
        print(f"   added version {version_string} to the submission")
    else:
        print(f"   version {version_string} is already in the submission")

    if not args.submit:
        print("   ready. Re-run with --submit to send it to Apple.")
        return 0

    client.expect(
        "PATCH", f"/v1/reviewSubmissions/{submission['id']}",
        {"data": {"type": "reviewSubmissions", "id": submission["id"],
                  "attributes": {"submitted": True}}},
    )

    # A 200 is not proof, for the same reason testflight.py re-reads its beta
    # submission: Apple re-validates on submit and can put the whole thing back
    # without telling anyone.
    confirmed = client.expect("GET", f"/v1/reviewSubmissions/{submission['id']}")
    final = confirmed["data"]["attributes"].get("state")
    if final == "READY_FOR_REVIEW":
        raise SystemExit(
            "   still READY_FOR_REVIEW after the PATCH — the submission was not "
            "accepted.\n   Apple's reason is on the version row in App Store "
            "Connect; the API does not expose it."
        )
    print(f"   submitted. state is now {final}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
