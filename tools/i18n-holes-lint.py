#!/usr/bin/env python3
"""Every template's placeholders must be supplied by its call (I18N-5).

`Locale::fill` leaves an unknown placeholder as it is written, so a
template asking for `{coupling}` while its call supplies `direction`
renders the literal text `{coupling}` on screen. That is the right runtime
behaviour — a visible, obviously-wrong string beats a silent gap — and it
is exactly why it needs checking here: nothing else notices.

The compiler will not, because both halves are valid Rust. The tests will
not, unless one happens to assert on that sentence. `cargo clippy` will
not. The first time this happened it came from renaming a hole in a
template and forgetting the tuple that fills it, which is a two-line edit
that looks complete.

Also catches the reverse — a value supplied under a name no template asks
for — which is dead weight rather than a bug, and usually the other half
of the same rename.

    python3 tools/i18n-holes-lint.py
    python3 tools/i18n-holes-lint.py --check   # non-zero if a hole is unfilled
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
RENDER = ROOT / "crates/kerotakis-core/src/render.rs"

# locale.fill("key", "template", &[("name", …), …])
FILL = re.compile(
    r'locale\s*\.\s*fill\s*\(\s*"([^"]+)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*&\[', re.S
)


def main() -> int:
    src = RENDER.read_text()
    unfilled: list[tuple[str, list[str], list[str]]] = []
    unused: list[tuple[str, list[str]]] = []

    for m in FILL.finditer(src):
        key = m.group(1)
        # Rust joins a `\` line continuation, so the template is one string.
        template = re.sub(r"\\\n\s*", "", m.group(2))

        # Read the tuple list up to its matching close bracket.
        i, depth = m.end(), 1
        while i < len(src) and depth:
            if src[i] == "[":
                depth += 1
            elif src[i] == "]":
                depth -= 1
            i += 1
        supplied = set(re.findall(r'\(\s*"(\w+)"\s*,', src[m.end() : i]))
        wanted = set(re.findall(r"\{(\w+)\}", template))

        if wanted - supplied:
            unfilled.append((key, sorted(wanted - supplied), sorted(supplied)))
        if supplied - wanted:
            unused.append((key, sorted(supplied - wanted)))

    total = src.count("locale.fill(")
    print(f"fill sites: {total}")
    print(f"placeholders with nothing to fill them: {len(unfilled)}")
    for key, missing, have in unfilled:
        print(f"   {key}")
        print(f"      template wants {missing}")
        print(f"      call supplies  {have[:8]}")

    if unused:
        print(f"\nvalues supplied that no template asks for: {len(unused)}")
        for key, extra in unused[:8]:
            print(f"   {key}: {extra}")

    if unfilled:
        print("\nEach of these renders its placeholder literally on screen.")
    return 1 if (unfilled and "--check" in sys.argv) else 0


if __name__ == "__main__":
    raise SystemExit(main())
