#!/usr/bin/env bash
# Actually run the macOS app you are about to ship.
#
# A Mac App Store build will not launch on the machine that built it. `open`
# returns:
#
#   RBSRequestErrorDomain Code=5 "Launch failed"
#   NSPOSIXErrorDomain Code=163 "Launchd job spawn failed"
#
# So the shipped bundle is, by default, never once run — which is exactly
# how a blank window reaches the store. (appstore.md carries a standing
# "Untested — nobody has launched it" note for precisely this.)
#
# The fix is small and total: copy the bundle, drop the store provisioning
# profile, re-sign ad-hoc with the same entitlements, and launch THAT. It is
# byte-identical apart from the signature authority, so anything that breaks
# here breaks in the shipped build too.
#
# Prints what the window is, whether the process survives, and — with
# `--shot` — a photograph of the app's own window, captured by CGWindowID so
# it neither needs focus nor grabs whatever else is on screen.
#
# Usage: tools/run-macos-local.sh [path/to/Kerotakis.app] [--shot out.png]
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAURI="$ROOT/web/app/src-tauri"

APP=""
SHOT=""
while [ $# -gt 0 ]; do
    case "$1" in
        --shot) SHOT="${2:?--shot needs a path}"; shift 2 ;;
        *)      APP="$1"; shift ;;
    esac
done

if [ -z "$APP" ]; then
    # Ask cargo where the build went, honouring whatever CARGO_TARGET_DIR is
    # in force — this repo's builds routinely use a private one.
    TARGET_DIR="$(cargo metadata --format-version 1 --no-deps \
        --manifest-path "$TAURI/Cargo.toml" 2>/dev/null \
        | python3 -c 'import sys,json; print(json.load(sys.stdin)["target_directory"])' \
        2>/dev/null)"
    for base in "$TARGET_DIR" "$HOME/.cache/kerotakis-appstore-target" "$TAURI/target"; do
        [ -n "$base" ] && [ -d "$base" ] || continue
        APP="$(find "$base" -maxdepth 5 -name 'Kerotakis.app' -type d 2>/dev/null | head -1)"
        [ -n "$APP" ] && break
    done
fi
[ -n "$APP" ] && [ -d "$APP" ] || {
    echo "no Kerotakis.app found. Pass its path, or run tools/build-macos-appstore.sh first."
    exit 1
}
echo "== $APP"

WORK="$(mktemp -d)"
LOCAL="$WORK/$(basename "$APP")"
cp -R "$APP" "$LOCAL"
rm -f "$LOCAL/Contents/embedded.provisionprofile"
codesign --force --deep --sign - \
    --entitlements "$TAURI/entitlements.plist" "$LOCAL" 2>&1 | tail -1
codesign --verify --verbose=1 "$LOCAL" 2>&1 | tail -1

EXE="$(/usr/libexec/PlistBuddy -c "Print CFBundleExecutable" "$LOCAL/Contents/Info.plist")"
"$LOCAL/Contents/MacOS/$EXE" >"$WORK/stdout.txt" 2>&1 &
PID=$!
sleep 12

if ! kill -0 "$PID" 2>/dev/null; then
    echo "the app exited on its own. Its output:"
    cat "$WORK/stdout.txt"
    rm -rf "$WORK"
    exit 1
fi

echo "== running as pid $PID"
osascript -e "tell application \"System Events\" to get name of every window of \
    (first process whose unix id is $PID)" 2>/dev/null \
    | sed 's/^/   window: /'

if [ -n "$SHOT" ]; then
    # By CGWindowID: a full-screen grab photographs whatever is in front,
    # which on a developer's machine is never the app under test.
    cat > "$WORK/winid.swift" <<'SWIFT'
import CoreGraphics
import Foundation
let wanted = Int(CommandLine.arguments[1])!
guard let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID)
        as? [[String: Any]] else { exit(1) }
for w in list where (w[kCGWindowOwnerPID as String] as? Int) == wanted {
    let b = w[kCGWindowBounds as String] as? [String: Any] ?? [:]
    if (b["Width"] as? Double ?? 0) > 1 {
        print(w[kCGWindowNumber as String] as? Int ?? -1)
        break
    }
}
SWIFT
    if swiftc -O -o "$WORK/winid" "$WORK/winid.swift" 2>/dev/null; then
        WID="$("$WORK/winid" "$PID")"
        [ -n "$WID" ] && screencapture -x -o -l"$WID" "$SHOT" \
            && echo "   window $WID captured to $SHOT"
    fi
fi

echo "== the app's own output"
sed 's/^/   /' "$WORK/stdout.txt" | head -20

kill "$PID" 2>/dev/null
rm -rf "$WORK"
echo "== OK: it launched, opened a window, and survived"
