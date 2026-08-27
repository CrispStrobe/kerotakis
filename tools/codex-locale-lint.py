#!/usr/bin/env python3
"""I18N-1 coverage: how much of the experiment catalogue speaks each language.

Translations live in `codex/i18n/<code>.toml`, one file per language, keyed
by `<entry-id>.<path to the English field>`. One file per language is the
point: two translators never edit the same file, and the English source
does not grow a copy per language.

Coverage is per language and per source file, because a partial translation
is the intended state — every string falls back to English on its own — and
the useful question is not "is it done" but "what is left".

Also enforces the glossary. The interface renders `bench` as "Labor" and
`shelf` as "Regal"; a translator who picks a different word is not wrong in
isolation but makes the app read as two translations stitched together,
which is exactly what happened on the first pass ("Werkbank").

    python3 tools/codex-locale-lint.py           # report
    python3 tools/codex-locale-lint.py --check   # non-zero if a rule is broken
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
CODEX = ROOT / "codex"
I18N = CODEX / "i18n"

# Prose a learner or teacher reads. Everything else in the catalogue is an
# identifier, a formula, a number or a bibliographic source.
TRANSLATABLE = (
    "question", "misconception", "reveals", "next",
    "lv1", "lv2", "lv3", "summary", "options",
)

# Renderings the shell already ships (web/app/src/locales/de.json).
# "Werkbank" is a carpenter's bench; the simulated lab is "Labor" and a
# physical chemistry bench is a "Labortisch". Either is a fix.
GLOSSARY_HARD = {
    "de": {"Werkbank": "Labor or Labortisch", "Arbeitsplatte": "Labor"},
}


def english_paths() -> dict[str, str]:
    """Every translatable field in the catalogue, as `<id>.<path>`."""
    out: dict[str, str] = {}

    def walk(node, path):
        if isinstance(node, dict):
            for k, v in node.items():
                if k in TRANSLATABLE and isinstance(v, (str, list)):
                    out[".".join(path + [k])] = k
                elif isinstance(v, (dict, list)):
                    walk(v, path + [k])
        elif isinstance(node, list):
            for i, v in enumerate(node):
                if isinstance(v, (dict, list)):
                    walk(v, path + [str(i)])

    for f in sorted(CODEX.glob("*.toml")):
        if f.name == "concepts.toml":
            continue
        doc = tomllib.load(open(f, "rb"))
        for section in ("reaction", "model"):
            for entry in doc.get(section, []):
                found: dict[str, str] = {}
                sub = out
                out = {}
                walk(entry, [])
                found, out = out, sub
                for path, field in found.items():
                    out[f"{entry['id']}.{path}"] = f.name
    return out


def main() -> int:
    check = "--check" in sys.argv
    problems = 0

    english = english_paths()
    per_file_total = collections.Counter(english.values())

    languages = sorted(p.stem for p in I18N.glob("*.toml")) if I18N.is_dir() else []
    if not languages:
        print("no translations in codex/i18n/")
        return 0

    for code in languages:
        cat = tomllib.load(open(I18N / f"{code}.toml", "rb"))
        print(f"\n== {code}")
        print(f"{'file':<24} {code:>8} {'English':>8}   coverage")
        have = collections.Counter()
        for key in cat:
            src = english.get(key)
            if src is None:
                print(f"   STALE: {key!r} translates nothing in the catalogue")
                problems += 1
            else:
                have[src] += 1

        total_have = total_all = 0
        for name in sorted(per_file_total):
            n, all_n = have[name], per_file_total[name]
            total_have += n
            total_all += all_n
            pct = 100 * n / all_n if all_n else 100.0
            print(f"{name:<24} {n:>8} {all_n:>8}   {pct:5.1f}% {'#' * int(pct / 5)}")
        pct = 100 * total_have / total_all if total_all else 100.0
        print(f"{'TOTAL':<24} {total_have:>8} {total_all:>8}   {pct:5.1f}%")

        for bad, good in GLOSSARY_HARD.get(code, {}).items():
            for key, value in cat.items():
                text = " ".join(value) if isinstance(value, list) else str(value)
                if re.search(rf"\b{bad}\b", text):
                    print(f"   GLOSSARY: {key} says {bad!r}; the interface says {good!r}")
                    problems += 1

    if problems:
        print(f"\n{problems} problem(s)")
    return 1 if (problems and check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
