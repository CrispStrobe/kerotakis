#!/usr/bin/env bash
# Build, sign and wrap the macOS app for the Mac App Store.
#
# Mac App Store submission has two artifacts and two identities, which is
# the whole reason this is a script and not one `tauri build`:
#
#   the .app   signed "Apple Distribution: ...", sandboxed, with the App
#              Store provisioning profile embedded so the identity
#              entitlements it claims are ones it may claim.
#   the .pkg   an installer wrapper around that .app, signed with the
#              separate "3rd Party Mac Developer Installer: ..." identity.
#              `altool --type macos` uploads the .pkg, never the .app.
#
# The profile is fetched from App Store Connect at build time rather than
# committed: it is bound to a certificate and expires. `ASC_PROFILE_BASE64`
# (a CI secret holding the same bytes) is honoured instead when set, so this
# script works on a runner with no API key.
#
# Requires the "Apple Distribution" private key in the keychain — the
# canonical shared certificate, imported from the .p12; never mint a new
# one (appstore.md).
#
# Usage: tools/build-macos-appstore.sh [--no-upload]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAURI="$ROOT/web/app/src-tauri"

TEAM_ID="${ASC_TEAM_ID:-N9XSJ4M3GT}"
BUNDLE_ID="${BUNDLE_ID:-com.crispstrobe.kerotakis}"
KEY_ID="${ASC_KEY_ID:-9RMU3C7422}"
ISSUER_ID="${ASC_ISSUER_ID:-5f618ba3-98ef-42ad-835c-fbbef6c76cf5}"
PROFILE_NAME="${MAC_PROFILE_NAME:-Kerotakis Mac App Store}"

UPLOAD=1
[ "${1:-}" = "--no-upload" ] && UPLOAD=0

VERSION="$(python3 -c "import json,sys; print(json.load(open('$TAURI/tauri.conf.json'))['version'])")"
echo "== Kerotakis $VERSION -> Mac App Store"

echo "== the provisioning profile"
if [ -n "${ASC_PROFILE_BASE64:-}" ]; then
    printf %s "$ASC_PROFILE_BASE64" | base64 -d > "$TAURI/embedded.provisionprofile"
    echo "   from ASC_PROFILE_BASE64"
else
    python3 "$ROOT/tools/asc/fetch-profile.py" "$PROFILE_NAME" \
        "$TAURI/embedded.provisionprofile"
fi
security cms -D -i "$TAURI/embedded.provisionprofile" > /tmp/kero-profile.plist
echo "   $(/usr/libexec/PlistBuddy -c 'Print Name' /tmp/kero-profile.plist)" \
     "expires $(/usr/libexec/PlistBuddy -c 'Print ExpirationDate' /tmp/kero-profile.plist)"

# By fingerprint, never by name. Certificate names are not unique — this
# account has two identically-named installer identities in two keychains
# and only one of them exists in App Store Connect. Given the name,
# productbuild either refuses as ambiguous or silently signs with the dead
# one and Apple rejects the upload.
echo "== the signing identities, matched against what the account says is live"
export APPLE_SIGNING_IDENTITY="$(python3 "$ROOT/tools/asc/resolve-identity.py" DISTRIBUTION)"
INSTALLER_IDENTITY="$(python3 "$ROOT/tools/asc/resolve-identity.py" MAC_INSTALLER_DISTRIBUTION)"

# Universal, because an Apple-Silicon-only build simply cannot be installed
# on an Intel Mac and the store gives no warning about it.
rustup target add x86_64-apple-darwin aarch64-apple-darwin >/dev/null

echo "== tauri build (universal)"
cd "$ROOT/web/app"
npx tauri build --target universal-apple-darwin \
    --config src-tauri/tauri.macos-appstore.conf.json

TARGET_DIR="$(cargo metadata --format-version 1 --no-deps --manifest-path "$TAURI/Cargo.toml" \
    | python3 -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])')"
APP="$TARGET_DIR/universal-apple-darwin/release/bundle/macos/Kerotakis.app"
[ -d "$APP" ] || { echo "no .app at $APP"; exit 1; }

echo "== what got signed"
[ -f "$APP/Contents/embedded.provisionprofile" ] \
    || { echo "the profile is NOT embedded; the upload would be rejected"; exit 1; }
codesign --verify --deep --strict --verbose=2 "$APP"
# ":-" is the modern spelling: XML plist to stdout. The bare "-" prints the
# raw blob, and `--xml` is deprecated.
ENTS="$(codesign -d --entitlements :- "$APP" 2>/dev/null)"
printf '%s\n' "$ENTS" | grep -E "application-identifier|app-sandbox" -A1
# The two keys the store demands and Tauri does not add on its own; a build
# missing them signs fine and is rejected on upload.
printf '%s' "$ENTS" | grep -q "$TEAM_ID.$BUNDLE_ID" \
    || { echo "the signed entitlements do not claim $TEAM_ID.$BUNDLE_ID"; exit 1; }
printf '%s' "$ENTS" | grep -q "com.apple.security.app-sandbox" \
    || { echo "the app is not sandboxed; the Mac App Store requires it"; exit 1; }
# Ask the bundle for its own executable rather than assuming it is named
# after productName: Tauri names it after the Cargo bin target.
EXE="$(/usr/libexec/PlistBuddy -c "Print CFBundleExecutable" "$APP/Contents/Info.plist")"
lipo -archs "$APP/Contents/MacOS/$EXE"
lipo -archs "$APP/Contents/MacOS/$EXE" | grep -q x86_64 \
    || { echo "not universal: an Intel Mac could not install this"; exit 1; }

PKG="$TARGET_DIR/universal-apple-darwin/release/bundle/macos/Kerotakis-$VERSION.pkg"
echo "== the installer package"
productbuild --component "$APP" /Applications \
    --sign "$INSTALLER_IDENTITY" "$PKG"
pkgutil --check-signature "$PKG" | head -3

if [ "$UPLOAD" = 0 ]; then
    echo
    echo "OK (not uploaded): $PKG"
    exit 0
fi

# Everything altool needs is read back out of the artifact by upload.py —
# passing a version the bundle does not carry is a 409, and altool exits 0
# when an upload fails, so the exit code alone cannot be trusted.
python3 "$ROOT/tools/asc/upload.py" "$PKG"
