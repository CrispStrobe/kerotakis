#!/usr/bin/env bash
# Build, sign and upload the iOS app for TestFlight / the App Store.
#
# Signing is MANUAL against the account's canonical Distribution
# certificate and the "Kerotakis AppStore CI" profile. Automatic signing
# would let Xcode mint certificates against a hard account-wide cap whose
# overflow is resolved by revoking one that other apps depend on, and would
# manage capabilities — which invalidates every profile for the App ID.
# See tools/ios/patch-signing.py.
#
# Order matters and is enforced below:
#   init -> privacy manifest (regenerates the pbxproj)
#        -> signing patch (must be last; a regenerate discards it)
#        -> build
#
# Usage: tools/build-ios-appstore.sh [--no-upload]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAURI="$ROOT/web/app/src-tauri"
GEN="$TAURI/gen/apple"

TEAM_ID="${ASC_TEAM_ID:-N9XSJ4M3GT}"
BUNDLE_ID="${BUNDLE_ID:-com.crispstrobe.kerotakis}"
KEY_ID="${ASC_KEY_ID:-9RMU3C7422}"
ISSUER_ID="${ASC_ISSUER_ID:-5f618ba3-98ef-42ad-835c-fbbef6c76cf5}"
PROFILE_NAME="${IOS_PROFILE_NAME:-Kerotakis AppStore CI}"

UPLOAD=1
[ "${1:-}" = "--no-upload" ] && UPLOAD=0

VERSION="$(python3 -c "import json; print(json.load(open('$TAURI/tauri.conf.json'))['version'])")"
echo "== Kerotakis $VERSION -> iOS App Store"

echo "== the signing identity is present"
security find-identity -v -p codesigning \
    | grep -F "Apple Distribution: Christian Ströbele ($TEAM_ID)" >/dev/null \
    || { echo "missing the canonical Apple Distribution identity; import the .p12"; exit 1; }

echo "== the provisioning profile, where Xcode looks for it"
PROF="$TAURI/ios.mobileprovision"
if [ -n "${ASC_PROFILE_BASE64:-}" ]; then
    printf %s "$ASC_PROFILE_BASE64" | base64 -d > "$PROF"
else
    python3 "$ROOT/tools/asc/fetch-profile.py" "$PROFILE_NAME" "$PROF"
fi
security cms -D -i "$PROF" > /tmp/kero-ios-profile.plist
UUID="$(plutil -extract UUID raw -o - /tmp/kero-ios-profile.plist)"
PROFDIR="$HOME/Library/MobileDevice/Provisioning Profiles"
mkdir -p "$PROFDIR"
cp "$PROF" "$PROFDIR/$UUID.mobileprovision"
echo "   installed $UUID.mobileprovision"

# xcodegen is not preinstalled on a clean macOS runner; `tauri ios init`
# brew-installs it silently, which is a dependency worth making explicit.
command -v xcodegen >/dev/null || brew install xcodegen

cd "$ROOT/web/app"

echo "== tauri ios init"
npx tauri ios init

echo "== the privacy manifest"
python3 "$ROOT/tools/ios/patch-privacy.py"
(cd "$GEN" && xcodegen generate >/dev/null)

echo "== manual signing (last: a regenerate would discard this)"
python3 "$ROOT/tools/ios/patch-signing.py" "$TEAM_ID" "$PROFILE_NAME"

# xcodebuild creates its workspace arena under Xcode's DerivedData
# location, and if that location is unwritable the build fails before
# compiling anything ("Couldn't create workspace arena folder"). A
# `-derivedDataPath` passthrough does not help: `tauri ios build` drops it,
# exactly as it drops the `-authenticationKey*` flags. The one lever that
# does reach xcodebuild is Xcode's own preference, so borrow it and give it
# back. Only when the default really is broken — this is a workaround, not
# a policy.
DD="$HOME/Library/Developer/Xcode/DerivedData"
if ! ( : > "$DD/.kero-write-test" ) 2>/dev/null; then
    DERIVED="${IOS_DERIVED_DATA:-$GEN/DerivedData}"
    mkdir -p "$DERIVED"
    PREV="$(defaults read com.apple.dt.Xcode IDECustomDerivedDataLocation 2>/dev/null || echo __unset__)"
    restore_derived_data() {
        if [ "$PREV" = "__unset__" ]; then
            defaults delete com.apple.dt.Xcode IDECustomDerivedDataLocation 2>/dev/null || true
        else
            defaults write com.apple.dt.Xcode IDECustomDerivedDataLocation -string "$PREV"
        fi
    }
    trap restore_derived_data EXIT
    defaults write com.apple.dt.Xcode IDECustomDerivedDataLocation -string "$DERIVED"
    echo "== $DD is not writable; building into $DERIVED instead"
    echo "   (Xcode's DerivedData preference is restored when this exits)"
else
    rm -f "$DD/.kero-write-test"
fi

# Clear the previous run's leavings, both of which cause real failures:
#
#   Externals/  holds the staticlib per architecture AND configuration, and
#               the target's `sources` scans the whole directory. A `debug`
#               libapp.a left by a simulator build therefore collides with
#               the release one:
#                 error: Multiple commands produce ... Kerotakis.app/libapp.a
#   build/      holds the archive. It must not survive a failed build, or
#               the export below would happily ship the PREVIOUS binary.
rm -rf "$GEN/Externals" "$GEN/build"

echo "== tauri ios build"
# Its own export step is expected to fail here, and that is not a problem
# worth working around any harder: Tauri generates an ExportOptions.plist
# without a `provisioningProfiles` mapping, which automatic signing does
# not need and manual signing cannot do without —
#   error: exportArchive "Kerotakis.app" requires a provisioning profile.
# The archive it produces first is perfectly good, so take that and export
# it below with options that name the profile. Same split the Xcode-project
# recipes use: archive and export are separate calls.
set +e
npx tauri ios build --export-method app-store-connect
set -e

ARCHIVE="$(find "$GEN/build" -maxdepth 1 -name '*.xcarchive' | head -1)"
[ -n "$ARCHIVE" ] || { echo "no .xcarchive under $GEN/build — the build itself failed"; exit 1; }
# An archive with no app inside is what a half-failed build leaves behind,
# and exporting it produces "Found no compatible export methods" three
# minutes later instead of saying so here.
[ -d "$ARCHIVE/Products/Applications" ] \
    || { echo "the archive has no Products/Applications — the build failed"; exit 1; }
echo "== archive: $ARCHIVE"

echo "== export (manual signing, profile named explicitly)"
EXPORT_DIR="$GEN/build/export"
rm -rf "$EXPORT_DIR"
cat > "$GEN/build/ExportOptions.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key><string>app-store-connect</string>
    <key>destination</key><string>export</string>
    <key>teamID</key><string>$TEAM_ID</string>
    <key>signingStyle</key><string>manual</string>
    <key>signingCertificate</key><string>Apple Distribution</string>
    <key>provisioningProfiles</key>
    <dict>
        <key>$BUNDLE_ID</key><string>$PROFILE_NAME</string>
    </dict>
    <key>uploadSymbols</key><true/>
</dict>
</plist>
PLIST
# /usr/bin first, for rsync. `-exportArchive` shells out to it, and a
# Homebrew rsync ahead of Apple's openrsync makes the export die with a
# bare "error: exportArchive Copy failed" that reads exactly like a signing
# problem and is not one.
PATH="/usr/bin:$PATH" xcodebuild -exportArchive \
    -archivePath "$ARCHIVE" \
    -exportPath "$EXPORT_DIR" \
    -exportOptionsPlist "$GEN/build/ExportOptions.plist" \
    | tail -5

IPA="$(find "$EXPORT_DIR" -name '*.ipa' | head -1)"
[ -n "$IPA" ] || { echo "no .ipa produced under $EXPORT_DIR"; exit 1; }
echo "== produced $IPA ($(du -h "$IPA" | cut -f1))"

echo "== what got signed"
WORK="$(mktemp -d)"
unzip -q "$IPA" -d "$WORK"
APP="$(find "$WORK/Payload" -maxdepth 1 -name '*.app' | head -1)"
codesign --verify --deep --strict --verbose=2 "$APP"
codesign -dv --verbose=4 "$APP" 2>&1 | grep -E "Authority|Identifier|TeamIdentifier"
# "does not satisfy its designated Requirement" here means the upload will
# be rejected; stop rather than spend an altool round trip finding out.
codesign --verify -R="anchor apple generic" --verbose "$APP" 2>&1 | tail -2
[ -f "$APP/PrivacyInfo.xcprivacy" ] \
    || { echo "PrivacyInfo.xcprivacy is missing from the bundle ROOT"; exit 1; }
[ -f "$APP/embedded.mobileprovision" ] \
    || { echo "embedded.mobileprovision is missing"; exit 1; }
/usr/libexec/PlistBuddy -c "Print :ITSAppUsesNonExemptEncryption" "$APP/Info.plist"
/usr/libexec/PlistBuddy -c "Print :UIRequiredDeviceCapabilities" "$APP/Info.plist"
PLIST_VERSION="$(/usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" "$APP/Info.plist")"
[ "$PLIST_VERSION" = "$VERSION" ] \
    || { echo "Info.plist says $PLIST_VERSION, tauri.conf.json says $VERSION"; exit 1; }
# ITMS-90068 is a WARNING on upload, so a build below the floor ships
# silently. Assert it here, where it is an error.
MIN_OS="$(/usr/libexec/PlistBuddy -c "Print :MinimumOSVersion" "$APP/Info.plist")"
echo "   MinimumOSVersion $MIN_OS"
[ "${MIN_OS%%.*}" -ge 15 ] \
    || { echo "MinimumOSVersion $MIN_OS is below 15.0 — Apple's ITMS-90068 floor"; exit 1; }
rm -rf "$WORK"

if [ "$UPLOAD" = 0 ]; then
    echo
    echo "OK (not uploaded): $IPA"
    exit 0
fi

# Everything altool needs is read back out of the artifact by upload.py —
# passing a version the bundle does not carry is a 409, and altool exits 0
# when an upload fails, so the exit code alone cannot be trusted.
python3 "$ROOT/tools/asc/upload.py" "$IPA"
