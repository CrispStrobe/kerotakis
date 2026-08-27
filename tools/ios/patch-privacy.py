#!/usr/bin/env python3
"""Put `PrivacyInfo.xcprivacy` into the generated iOS project, exactly once.

Required since iOS 17 for the "Required Reason APIs"; Tauri's core plugins
trigger it even though Kerotakis itself touches none of them directly. There
is no `tauri.conf.json` hook, so it has to be patched in after
`tauri ios init`.

Two placements are wrong in ways that only show up much later:

  gen/apple/<app>_iOS/   that whole directory is listed in the target's
                         `sources`, so xcodegen picks the file up by
                         directory scan AND by the explicit entry, and the
                         build fails with "Multiple commands produce ...".
  nested in the bundle   the file must sit at the built app's ROOT or
                         Apple does not see it at all.

So: copy to `gen/apple/` (which no `sources` entry scans) and declare it
once in project.yml. Run BEFORE the signing patch — this one regenerates
the pbxproj, which would discard signing settings written first.
"""

from __future__ import annotations

import pathlib
import shutil
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
TAURI = ROOT / "web" / "app" / "src-tauri"
GEN = TAURI / "gen" / "apple"

# The anchor is unique in the generated project.yml and belongs to the same
# `sources` list. If a Tauri upgrade changes it, failing here is the point:
# a silently unpatched project ships without the privacy manifest.
ANCHOR = "      - path: LaunchScreen.storyboard\n"
ENTRY = "      - path: PrivacyInfo.xcprivacy\n        buildPhase: resources\n"


def main() -> int:
    src = TAURI / "PrivacyInfo.xcprivacy"
    project = GEN / "project.yml"
    if not project.exists():
        print(f"no generated project at {project} — run `tauri ios init` first",
              file=sys.stderr)
        return 1

    shutil.copy2(src, GEN / "PrivacyInfo.xcprivacy")
    print(f"   copied to {GEN.relative_to(ROOT)}/PrivacyInfo.xcprivacy")

    text = project.read_text()
    if "PrivacyInfo.xcprivacy" in text:
        print("   project.yml: already declared")
        return 0
    if ANCHOR not in text:
        print(f"   project.yml: anchor {ANCHOR.strip()!r} not found — the Tauri "
              f"template changed; re-derive the insertion point", file=sys.stderr)
        return 1
    project.write_text(text.replace(ANCHOR, ANCHOR + ENTRY, 1))
    print("   project.yml: declared once, buildPhase resources")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
