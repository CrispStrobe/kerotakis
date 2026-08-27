#!/usr/bin/env python3
"""I18N-1 coverage: how much of the experiment catalogue speaks German yet.

The catalogue is translated field by field into `*_de` siblings, so a
partially translated file degrades to English per string rather than
failing. That is the right behaviour and it is also silent, which is why
progress needs measuring rather than remembering.

Also enforces the glossary. The interface already renders `bench` as
"Labor", `shelf` as "Regal" and so on; a translator who picks a different
word is not wrong in isolation but makes the app read as two translations
stitched together, and that is exactly what happened on the first pass
("Werkbank").

    python3 tools/codex-locale-lint.py           # report
    python3 tools/codex-locale-lint.py --check   # non-zero if a rule is broken
"""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
CODEX = ROOT / "codex"

# Prose the learner or teacher reads. Everything else in the catalogue is
# an identifier, a formula, a number or a bibliographic source.
TRANSLATABLE = (
    "question", "misconception", "reveals", "next",
    "lv1", "lv2", "lv3", "summary",
)

# Renderings the shell already ships (web/app/src/lib/i18n.svelte.ts).
# left = what must NOT appear, right = what the shell says instead.
FORBIDDEN = {
    "Werkbank": "Labor",
    "Werkbank": "Labor",
    "Arbeitsplatte": "Labor",
    "Glas": None,          # too vague where Becherglas is meant; advisory only
}
GLOSSARY_HARD = {"Werkbank": "Labor", "Arbeitsplatte": "Labor"}


def leaves(node, prefix=""):
    if isinstance(node, dict):
        for k, v in node.items():
            yield from leaves(v, f"{prefix}.{k}" if prefix else k)
    elif isinstance(node, list):
        for i, v in enumerate(node):
            yield from leaves(v, f"{prefix}[{i}]")
    else:
        yield prefix, node


def main() -> int:
    check = "--check" in sys.argv
    problems = 0
    total_en = total_de = 0

    print(f"{'file':<24} {'German':>8} {'English':>8}   coverage")
    for path in sorted(CODEX.glob("*.toml")):
        try:
            doc = tomllib.load(open(path, "rb"))
        except Exception as e:
            print(f"{path.name:<24}   DOES NOT PARSE: {e}")
            problems += 1
            continue

        flat = dict(leaves(doc))
        en = {k for k in flat if k.rsplit(".", 1)[-1] in TRANSLATABLE}
        de = {k for k in flat
              if k.rsplit(".", 1)[-1].endswith("_de")
              and k.rsplit(".", 1)[-1][:-3] in TRANSLATABLE}
        total_en += len(en)
        total_de += len(de)
        pct = (100 * len(de) / len(en)) if en else 100.0
        bar = "#" * int(pct / 5)
        print(f"{path.name:<24} {len(de):>8} {len(en):>8}   {pct:5.1f}% {bar}")

        # A translation with no English original is a typo in the key.
        for k in de:
            if k[:-3] not in flat:
                print(f"   ORPHAN: {k} has no English counterpart")
                problems += 1

        # Glossary.
        for k, v in flat.items():
            if not isinstance(v, str) or not k.rsplit(".", 1)[-1].endswith("_de"):
                continue
            for bad, good in GLOSSARY_HARD.items():
                if re.search(rf"\b{bad}\b", v):
                    print(f"   GLOSSARY: {k} says {bad!r}; the interface says {good!r}")
                    problems += 1

    pct = (100 * total_de / total_en) if total_en else 100.0
    print(f"\n{'TOTAL':<24} {total_de:>8} {total_en:>8}   {pct:5.1f}%")
    if problems:
        print(f"\n{problems} problem(s)")
    if check and problems:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
