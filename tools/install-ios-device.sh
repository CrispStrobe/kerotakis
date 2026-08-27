#!/usr/bin/env bash
#
# Put the current tree on a paired iPad or iPhone, over wifi, in one step.
#
# This is NOT the TestFlight path and does not touch App Store Connect
# beyond reading a profile. It signs for DEVELOPMENT and installs straight
# to a device with `devicectl`, which is the difference between seeing your
# change in two minutes and seeing it after a processing queue and a review.
#
# What it needs, all of which already exist on this account:
#   - the device paired and registered, with Developer Mode enabled
#   - "Team Wildcard Dev iPad" (N9XSJ4M3GT.*), which covers every bundle id
#     in the team, so no per-app development profile has to be minted
#   - the "Apple Development" identity in the keychain
# It mints no certificate and registers no device. If the device is not
# already registered it says so and stops, because registering one consumes
# a slot from the membership year's hundred and that is a decision for a
# human, not a build script.
#
# Usage: tools/install-ios-device.sh [device-name-or-udid]
#
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAURI="$ROOT/web/app/src-tauri"
GEN="$TAURI/gen/apple"

TEAM_ID="${ASC_TEAM_ID:-N9XSJ4M3GT}"
BUNDLE_ID="${BUNDLE_ID:-com.crispstrobe.kerotakis}"
PROFILE_NAME="${IOS_DEV_PROFILE_NAME:-Team Wildcard Dev iPad}"
IDENTITY="Apple Development"
WANT="${1:-}"

echo "== the device"
DEVJSON="$(mktemp)"
xcrun devicectl list devices --json-output "$DEVJSON" >/dev/null 2>&1 || {
    echo "devicectl could not list devices — is Xcode installed and the device paired?"; exit 1; }
read -r DEV_ID DEV_UDID DEV_NAME <<<"$(
  WANT="$WANT" /usr/bin/python3 - "$DEVJSON" <<'PY'
import json, os, sys
want = (os.environ.get("WANT") or "").lower()
devs = json.load(open(sys.argv[1]))["result"]["devices"]
rows = []
for d in devs:
    props = d.get("deviceProperties", {})
    hw = d.get("hardwareProperties", {})
    name = props.get("name", "?")
    udid = hw.get("udid", "")
    state = d.get("connectionProperties", {}).get("tunnelState", "")
    if hw.get("platform") not in ("iOS", "iPadOS", None):
        continue
    rows.append((d["identifier"], udid, name, state))
if want:
    rows = [r for r in rows if want in r[2].lower() or want in r[1].lower()]
if not rows:
    sys.exit("no matching iOS device is paired")
if len(rows) > 1:
    sys.stderr.write("several devices match; name one:\n")
    for r in rows:
        sys.stderr.write(f"   {r[2]}  ({r[1]})\n")
    sys.exit(1)
i, u, n, _ = rows[0]
print(i, u, n.replace(" ", "_"))
PY
)"
DEV_NAME="${DEV_NAME//_/ }"
echo "   $DEV_NAME  udid $DEV_UDID"

echo "== the identity"
security find-identity -v -p codesigning | grep -F "$IDENTITY" >/dev/null \
    || { echo "no \"$IDENTITY\" identity in the keychain"; exit 1; }

echo "== the development profile"
PROF="$TAURI/ios-dev.mobileprovision"
python3 "$ROOT/tools/asc/fetch-profile.py" "$PROFILE_NAME" "$PROF"
security cms -D -i "$PROF" > /tmp/kero-ios-dev-profile.plist
UUID="$(plutil -extract UUID raw -o - /tmp/kero-ios-dev-profile.plist)"
# The profile is what limits WHICH devices may run the build, so a device
# missing from it fails at install time with a message about the signature
# rather than about the device. Say the true thing here instead.
PROF="$PROF" /usr/bin/python3 - "$DEV_UDID" <<'PY' || exit 1
import os, plistlib, subprocess, sys
# Decode the PROFILE, not the plist we already decoded from it — feeding
# `security cms -D` its own output back is an InvalidFileException with a
# traceback that says nothing about profiles.
p = plistlib.loads(subprocess.run(
    ["security", "cms", "-D", "-i", os.environ["PROF"]],
    capture_output=True).stdout)
devices = p.get("ProvisionedDevices") or []
if sys.argv[1] not in devices:
    sys.exit(f"   this device is not in {p['Name']!r} ({len(devices)} device(s)).\n"
             f"   Register it in the developer account first — that spends one of\n"
             f"   the year's 100 device slots, so it is deliberately not automatic.")
print(f"   {p['Name']}: covers this device, expires {p['ExpirationDate']:%Y-%m-%d}")
PY
PROFDIR="$HOME/Library/MobileDevice/Provisioning Profiles"
mkdir -p "$PROFDIR"
cp "$PROF" "$PROFDIR/$UUID.mobileprovision"

command -v xcodegen >/dev/null || brew install xcodegen

cd "$ROOT/web/app"

# The same three leavings that break the App Store build break this one,
# for the same reasons, and only clearing them BEFORE generation helps.
# See tools/build-ios-appstore.sh for the full account: Externals/ is
# scanned by xcodegen so a stale libapp.a becomes a duplicate build
# command; build/ would let a failed build export the previous archive;
# and `tauri ios init` does not rewrite an existing project.yml, so a
# changed minimumSystemVersion silently would not apply.
rm -rf "$GEN/Externals" "$GEN/build" "$GEN/project.yml" "$GEN"/*.xcodeproj

echo "== tauri ios init"
npx tauri ios init

echo "== the privacy manifest"
python3 "$ROOT/tools/ios/patch-privacy.py"
(cd "$GEN" && xcodegen generate >/dev/null)

echo "== manual signing, for development"
python3 "$ROOT/tools/ios/patch-signing.py" "$TEAM_ID" "$PROFILE_NAME" "$IDENTITY"

echo "== build"
# Tauri's own export step wants an ExportOptions.plist naming the profile
# and does not write one, so it fails; the archive it makes first is good.
set +e
npx tauri ios build --export-method debugging
set -e

ARCHIVE="$(find "$GEN/build" -maxdepth 1 -name '*.xcarchive' | head -1)"
[ -n "$ARCHIVE" ] || { echo "no .xcarchive under $GEN/build — the build itself failed"; exit 1; }
APP="$(find "$ARCHIVE/Products/Applications" -maxdepth 1 -name '*.app' | head -1)"
[ -n "$APP" ] || { echo "the archive has no .app — the build failed"; exit 1; }
echo "== archive: $(basename "$ARCHIVE")"

# Prove the thing about to be installed is signed the way we think, and for
# this device. A distribution-signed .app installs and then refuses to
# launch, which is a confusing way to find out.
codesign -d --entitlements :- "$APP" 2>/dev/null | grep -q "get-task-allow" \
    || { echo "the .app is not development-signed — it would install and not launch"; exit 1; }

echo "== install to $DEV_NAME"
xcrun devicectl device install app --device "$DEV_ID" "$APP"

echo
echo "OK: $BUNDLE_ID is on $DEV_NAME."
echo "    Launch it there, or: xcrun devicectl device process launch --device $DEV_ID $BUNDLE_ID"
