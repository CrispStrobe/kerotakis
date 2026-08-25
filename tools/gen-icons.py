#!/usr/bin/env python3
"""Generate every icon Kerotakis ships, from one vector mark.

The mark is `web/icon.svg`'s flask — an outline in the bench's hot orange
over its blue liquid — and it is the *only* artwork; everything below is
that same geometry placed differently, because each destination masks it
differently:

  full-bleed  opaque square, no alpha, flask at ~76% of the canvas.
              iOS, Android and the App Store marketing icon all apply
              their own mask on top; anything we round or inset survives
              as a dark ring in the corners (appstore.md, "the artwork
              must be FULL-BLEED").
  squircle    the rounded shape IS the artwork, on a transparent margin.
              macOS does not mask, so a full-bleed square ships as a
              hard-edged square that looks wrong next to every other icon
              in the Dock.
  maskable    the PWA `purpose: maskable` variant. Android may crop to a
              circle, so the mark lives inside the inner-80% safe zone.
  rounded     the browser-tab / `purpose: any` look — the original
              rounded rect, which is what the favicon has always been.

Outputs are committed; this script is the record of how to redo them, not
a build step. It needs `rsvg-convert` (librsvg) and Pillow, neither of
which the app itself depends on.

Usage: python3 tools/gen-icons.py [--check]

`--check` regenerates into a temporary directory and diffs, so CI (or a
reviewer) can prove the committed PNGs still match this source.
"""

from __future__ import annotations

import argparse
import filecmp
import math
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
ICONS = ROOT / "web" / "app" / "src-tauri" / "icons"
WEB = ROOT / "web"

# The palette is the bench's own (web/index.html's :root).
INK = "#14120f"  # the dark ground
HOT = "#d98a4a"  # glassware outline
COOL = "#6fa8c7"  # liquid

# The mark in the 64-unit space `web/icon.svg` draws it in. Its ink extends
# from x 17→48 and y 9→53.6 once the 3-unit stroke is accounted for, so the
# centre is not the canvas centre and placing it needs these numbers.
MARK_X0, MARK_X1 = 15.5, 48.5
MARK_Y0, MARK_Y1 = 7.5, 55.1
MARK_W = MARK_X1 - MARK_X0
MARK_H = MARK_Y1 - MARK_Y0
MARK_CX = (MARK_X0 + MARK_X1) / 2
MARK_CY = (MARK_Y0 + MARK_Y1) / 2

MARK = f"""\
  <g>
    <path d="M27 9h10v5l-1.5 1.5V26l11.5 21a4.5 4.5 0 0 1-4 6.6H21a4.5 4.5 0 0 1-4-6.6L28.5 26V15.5L27 14z"
          fill="none" stroke="{HOT}" stroke-width="3" stroke-linejoin="round"/>
    <path d="M24 38.5h16l5.3 9.7a2.2 2.2 0 0 1-1.9 3.3H20.6a2.2 2.2 0 0 1-1.9-3.3z"
          fill="{COOL}"/>
    <circle cx="30" cy="34" r="1.6" fill="{COOL}"/>
    <circle cx="35" cy="30" r="1.2" fill="{COOL}"/>
  </g>"""


def placed(size: float, cx: float, cy: float, scale: float) -> str:
    """The mark, scaled and centred on (cx, cy) in a `size` canvas."""
    tx = cx - MARK_CX * scale
    ty = cy - MARK_CY * scale
    body = MARK.replace("\n", "\n  ")
    return f'  <g transform="translate({tx:.3f} {ty:.3f}) scale({scale:.5f})">\n{body}\n  </g>'


def squircle_path(size: float, side: float, n: float = 5.0, steps: int = 240) -> str:
    """A superellipse |x|^n + |y|^n = 1 — Apple's continuous corner.

    A plain `rx` rounded rect has a visible curvature discontinuity where
    the arc meets the straight edge; at icon sizes that reads as a subtly
    wrong shape beside real macOS icons. Sampling the superellipse costs
    one path and gets the corner right.
    """
    a = side / 2
    c = size / 2
    pts = []
    for i in range(steps):
        t = 2 * math.pi * i / steps
        ct, st = math.cos(t), math.sin(t)
        x = math.copysign(abs(ct) ** (2 / n), ct)
        y = math.copysign(abs(st) ** (2 / n), st)
        pts.append(f"{c + a * x:.3f},{c + a * y:.3f}")
    return "M" + "L".join(pts) + "Z"


def svg(size: int, background: str, mark_scale: float) -> str:
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" '
        f'viewBox="0 0 {size} {size}">\n'
        f"{background}\n"
        f"{placed(size, size / 2, size / 2, mark_scale)}\n"
        f"</svg>\n"
    )


def masters(size: int = 1024) -> dict[str, str]:
    """The four masters, each named for the mask it expects."""
    # Full-bleed: the mark at 76% of the canvas height. The OS mask eats
    # the corners, so nothing but ground may live there.
    full_bleed = svg(
        size,
        f'  <rect width="{size}" height="{size}" fill="{INK}"/>',
        size * 0.76 / MARK_H,
    )

    # macOS: Apple's icon grid puts the shape at 824/1024 of the canvas,
    # the rest transparent margin for the Dock's shadow and bounce.
    side = size * 824 / 1024
    squircle = svg(
        size,
        f'  <path d="{squircle_path(size, side)}" fill="{INK}"/>',
        side * 0.72 / MARK_H,
    )

    # Maskable: the safe zone is the inner 80% *circle*, so the mark's
    # diagonal — not its height — is what has to fit.
    diagonal = math.hypot(MARK_W, MARK_H)
    maskable = svg(
        size,
        f'  <rect width="{size}" height="{size}" fill="{INK}"/>',
        size * 0.80 / diagonal,
    )

    # Rounded: the favicon's own look, kept for `purpose: any` where no
    # mask is applied and a bare square would be the odd one out.
    radius = size * 12 / 64  # the same 12/64 the original icon.svg uses
    rounded = svg(
        size,
        f'  <rect width="{size}" height="{size}" rx="{radius:.3f}" fill="{INK}"/>',
        size * 0.74 / MARK_H,
    )

    return {
        "master-full-bleed.svg": full_bleed,
        "master-macos.svg": squircle,
        "master-maskable.svg": maskable,
        "master-rounded.svg": rounded,
    }


def rasterize(src: pathlib.Path, dst: pathlib.Path, size: int, *, opaque: bool) -> None:
    from PIL import Image

    subprocess.run(
        ["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(dst), str(src)],
        check=True,
    )
    # Normalise the channel count explicitly rather than inheriting whatever
    # rsvg-convert decided: it drops the alpha channel when a master happens
    # to be fully opaque, and `tauri::generate_context!` then panics with
    # "icon ... is not RGBA" — a build failure two steps removed from its
    # cause. `opaque` is the other side of the same coin: Apple rejects an
    # alpha channel in the 1024 marketing icon.
    with Image.open(dst) as im:
        im.convert("RGB" if opaque else "RGBA").save(dst)


# (master, output path relative to ROOT, size, opaque)
#
# `opaque` drops the alpha channel. The two consumers want opposite things
# and it is worth being explicit about which is which: `tauri::generate_
# context!` panics with "icon ... is not RGBA" on a channel-less PNG, while
# Apple rejects an alpha channel in the 1024 marketing icon. So the Tauri
# source keeps its (entirely opaque) alpha and the App Store gets its own
# flattened copy.
OUTPUTS = [
    # The `tauri icon` source: RGBA, or the build macro refuses it.
    ("master-full-bleed.svg", "web/app/src-tauri/icons/icon.png", 1024, False),
    ("master-macos.svg", "web/app/src-tauri/icons/icon-macos.png", 1024, False),
    # The App Store marketing icon: RGB, or the upload is rejected.
    ("master-full-bleed.svg", "web/app/src-tauri/icons/appstore-1024.png", 1024, True),
    # The PWA payload. 192 and 512 are what an installable manifest needs;
    # the maskable 512 is what keeps Android from framing us in a white box.
    ("master-rounded.svg", "web/icon-192.png", 192, False),
    ("master-rounded.svg", "web/icon-512.png", 512, False),
    ("master-maskable.svg", "web/icon-maskable-512.png", 512, False),
    # iOS Home Screen: Safari masks this itself, so it wants the full-bleed
    # master, and it must be opaque or the corners go black.
    ("master-full-bleed.svg", "web/apple-touch-icon.png", 180, True),
]


def generate(icons_dir: pathlib.Path, out_root: pathlib.Path) -> None:
    icons_dir.mkdir(parents=True, exist_ok=True)
    for name, body in masters().items():
        (icons_dir / name).write_text(body)
    for master, rel, size, opaque in OUTPUTS:
        dst = out_root / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        rasterize(icons_dir / master, dst, size, opaque=opaque)
        print(f"  {rel}  {size}px{' opaque' if opaque else ''}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check",
        action="store_true",
        help="regenerate into a temp tree and diff against what is committed",
    )
    args = ap.parse_args()

    if not shutil.which("rsvg-convert"):
        print("gen-icons: rsvg-convert not found (brew install librsvg)", file=sys.stderr)
        return 2

    if not args.check:
        print("== icons")
        generate(ICONS, ROOT)
        return 0

    with tempfile.TemporaryDirectory() as tmp:
        tmp_root = pathlib.Path(tmp)
        generate(tmp_root / "masters", tmp_root)
        stale = []
        for name in masters():
            if not (ICONS / name).exists() or (ICONS / name).read_text() != (
                tmp_root / "masters" / name
            ).read_text():
                stale.append(f"web/app/src-tauri/icons/{name}")
        for _, rel, _, _ in OUTPUTS:
            if not (ROOT / rel).exists() or not filecmp.cmp(
                ROOT / rel, tmp_root / rel, shallow=False
            ):
                stale.append(rel)
        if stale:
            print("gen-icons --check: stale, rerun `python3 tools/gen-icons.py`:")
            for s in sorted(set(stale)):
                print(f"  {s}")
            return 1
        print("gen-icons --check: every icon matches its source")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
