#!/usr/bin/env python3
"""Write manual App Store signing into the generated Xcode project.

Manual, not automatic, and deliberately so. Automatic signing does work —
Tauri's own `xcodebuild` calls pick up `APPLE_API_KEY` / `APPLE_API_ISSUER`
/ `APPLE_API_KEY_PATH` — but on this account it is the wrong trade twice
over: it "manages iOS capabilities changes", and a capability change
invalidates *every* provisioning profile for the App ID; and it mints
certificates, against a hard account-wide cap where the overflow is
resolved by revoking somebody else's. Manual signing contacts Apple at
build time not at all.

None of this can be done from `tauri.conf.json`. `bundle.iOS.
developmentTeam` does not reach the pbxproj, and the `IOS_CERTIFICATE` /
`IOS_MOBILE_PROVISION` environment variables only import a certificate into
a temporary keychain — they write nothing into the project. A freshly
generated project has exactly one signing key in it:

    CODE_SIGN_IDENTITY = "iPhone Developer";

so three of the four keys below must be INSERTED; a sed-style replace
matches nothing and reports success.

Run this LAST. Anything that re-runs `xcodegen generate` afterwards
discards every setting written here.

Usage: python3 tools/ios/patch-signing.py <team-id> <profile name>
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
GEN = ROOT / "web" / "app" / "src-tauri" / "gen" / "apple"


def main() -> int:
    team, profile = sys.argv[1], sys.argv[2]
    # "Apple Distribution" for the store, "Apple Development" for a
    # build that has to launch on a device you are holding.
    identity = sys.argv[3] if len(sys.argv) > 3 else "Apple Distribution"
    projects = sorted(GEN.glob("*.xcodeproj"))
    if not projects:
        print(f"no .xcodeproj under {GEN} — run `tauri ios init` first", file=sys.stderr)
        return 1
    pbxproj = projects[0] / "project.pbxproj"
    text = pbxproj.read_text()

    # Drop the generated identity first: leaving "iPhone Developer" in place
    # makes the block ambiguous, and an sdk-scoped override outranks the
    # value inserted below.
    text = re.sub(r'^\s*CODE_SIGN_IDENTITY(\[sdk=[^\]]*\])? = .*\n', "", text,
                  flags=re.MULTILINE)

    settings = (
        f'\t\t\t\tCODE_SIGN_IDENTITY = "{identity}";\n'
        f'\t\t\t\tCODE_SIGN_STYLE = Manual;\n'
        f'\t\t\t\tDEVELOPMENT_TEAM = {team};\n'
        f'\t\t\t\tPROVISIONING_PROFILE_SPECIFIER = "{profile}";\n'
    )
    # Every buildSettings block, because the app target's settings are split
    # across the project-level and target-level Debug/Release blocks and
    # picking "the right one" out of a generated file is guesswork.
    for key in ("CODE_SIGN_STYLE", "DEVELOPMENT_TEAM", "PROVISIONING_PROFILE_SPECIFIER"):
        text = re.sub(rf'^\s*{key} = .*\n', "", text, flags=re.MULTILINE)
    text, count = re.subn(r'(buildSettings = \{\n)', r'\1' + settings, text)

    pbxproj.write_text(text)
    print(f"   {pbxproj.relative_to(ROOT)}: manual signing in {count} buildSettings blocks")
    print(f"   team {team}, profile {profile!r}, identity {identity!r}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
