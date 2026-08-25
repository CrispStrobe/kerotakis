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

echo "== tauri ios build"
npx tauri ios build --export-method app-store-connect

IPA="$(find "$GEN/build" -name '*.ipa' -print0 | xargs -0 ls -t 2>/dev/null | head -1)"
[ -n "$IPA" ] || { echo "no .ipa produced under $GEN/build"; exit 1; }
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
rm -rf "$WORK"

if [ "$UPLOAD" = 0 ]; then
    echo
    echo "OK (not uploaded): $IPA"
    exit 0
fi

# Resolved from the API by bundle id, not taken on trust: uploading
# with the wrong numeric id silently lands the build in another app.
ASC_APP_ID="${ASC_APP_ID:-$(python3 "$ROOT/tools/asc/app-id.py")}"
echo "== app $ASC_APP_ID"
echo "== validate against Apple"
xcrun altool --validate-app -f "$IPA" --type ios \
    --apple-id "$ASC_APP_ID" \
    --bundle-id "$BUNDLE_ID" \
    --api-key "$KEY_ID" --api-issuer "$ISSUER_ID"

echo "== upload"
xcrun altool --upload-package "$IPA" --type ios \
    --apple-id "$ASC_APP_ID" \
    --bundle-id "$BUNDLE_ID" \
    --bundle-version "${BUILD_NUMBER:-1}" \
    --bundle-short-version-string "$VERSION" \
    --api-key "$KEY_ID" --api-issuer "$ISSUER_ID"

echo
echo "OK: uploaded $IPA"
echo "Next: tools/asc/testflight.py ios, once processing reports VALID."
