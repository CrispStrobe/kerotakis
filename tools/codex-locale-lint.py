#!/usr/bin/env python3
"""I18N-1 coverage: how much of the experiment catalogue speaks each language.

Translations live in `codex/i18n/<code>.toml`, one file per language, keyed
by `<entry-id>.<path to the English field>`. One file per language is the
point: two translators never edit the same file, and the English source
does not grow a copy per language. Adding French stays ONE new data file
and no code, which is why this gate extends that file rather than opening a
second prose store beside it — two stores writing the same `_de` siblings
would make "add a language" cost two files and leave no answer to which one
wins.

Coverage is per language and per source file, because a partial translation
is the intended state for a language being written — every string falls back
to English on its own — and the useful question is then not "is it done" but
"what is left".

A language in COMPLETE is past that stage: it ships, so a string it lacks is
English prose inside a German page rather than a translation in progress, and
that is a failure. This is what makes a NEW CATALOGUE ENTRY fail preflight
until it is translated. A language not in COMPLETE is only reported on, so a
French translation can be started, committed and grown without the gate
demanding all 1255 strings on its first commit.

Four rules, beyond coverage:

* **Stale.** A key translating nothing means the English moved and the
  translation is describing what is gone — it renders confidently and
  wrongly, which is worse than a gap.
* **Alignment.** `options` is positional: the correct answer is an INDEX
  into it and each diagnosis attaches by index, so a translated list of a
  different length marks the wrong answer correct. `explains` and
  `fails_at` are checked the same way, cheaply and for the same reason.
* **Wording.** No learner is addressed by age (GUI-470). This prose bypasses
  the locale bundles, so `learnerWording.test.ts` never sees it and the check
  is duplicated here — deliberately narrower, see AGE_WORDING.
* **Glossary.** The interface renders `bench` as "Labor" and `shelf` as
  "Regal"; a translator who picks a different word is not wrong in isolation
  but makes the app read as two translations stitched together, which is
  exactly what happened on the first pass ("Werkbank").

    python3 tools/codex-locale-lint.py           # report
    python3 tools/codex-locale-lint.py --check   # non-zero if a rule is broken
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys
import tomllib
import typing

ROOT = pathlib.Path(__file__).resolve().parent.parent
CODEX = ROOT / "codex"
I18N = CODEX / "i18n"

# Prose a learner or teacher reads. Everything else in the catalogue is an
# identifier, a formula, a number or a bibliographic source.
#
# `name`, `power`, `explains` and `fails_at` belong to the MODELS half of the
# catalogue and were missing from this tuple until I18N-1. Their absence is
# why `models.toml` reported 100% German while 325 of its 409 learner-facing
# strings rendered in English: the denominator counted only the three
# registers, so the field that matters most in a model — `fails_at`, the
# stated boundary — was never once counted as untranslated.
TRANSLATABLE = (
    "question", "misconception", "reveals", "next",
    "lv1", "lv2", "lv3", "summary", "options",
    "name", "power", "explains", "fails_at",
)

# Languages that SHIP. A string one of these lacks is English prose on a page
# the reader asked for in their own language, so it fails the gate. A language
# absent from here is being written: it is reported on and never blocks.
COMPLETE = ("de",)

# Lists whose translation must keep the English length.
#
# `options` is the load-bearing one — the catalogue stores the correct answer
# as an INDEX into it and attaches each diagnosis by index, so a list of a
# different length marks a different answer correct and explains a belief the
# learner never expressed. The UI treats a mismatched array as absent, which
# is the safe behaviour and also a silent one: nothing tells the translator.
ALIGNED = ("options", "explains", "fails_at")

# No learner is addressed by age (GUI-470). `learnerWording.test.ts` walks the
# locale bundles; this prose never reaches them, so the rule is duplicated
# here rather than assumed.
#
# It is NARROWER than the bundle regex on purpose. That one forbids `Jahre`
# outright, which is right for an interface label and wrong for prose: "so
# wird seit viertausend Jahren Mörtelkalk hergestellt" and "wie zwei Kinder,
# die dasselbe Seil festhalten" are a fact about the world and a simile about
# chemistry, not a verdict on the reader. Eight such lines are already in the
# shipped German. What is forbidden is an age BAND, or addressing the audience
# as children — the two shapes that put a claim about the reader on the page.
AGE_WORDING = (
    (r"\b(?:ab|für|fuer|von)\s+\d+\s*(?:bis\s*\d+\s*)?Jahren?\b", "an age band"),
    (r"\b\d+\s*[-–—]\s*\d+\s*Jahren?\b", "an age band"),
    (r"\bim Alter von\b", "an age band"),
    (r"\bAlters(?:gruppe|gruppen|band|empfehlung|stufe)\b", "an age band"),
    (r"\b(?:ages?|aged)\s+\d+", "an age band"),
    (r"\b\d+\s*(?:to|–|-)?\s*\d*\s*years?\s+old\b", "an age band"),
    (r"\bKinder(?:labor|versuch|versuche|experiment|experimente|chemie|buch)\b",
     "the reader addressed as a child"),
    (r"\bfür\s+Kinder\b", "the reader addressed as a child"),
    (r"\bfor\s+(?:kids|children)\b", "the reader addressed as a child"),
    (r"\b(?:children|kids)['’]?s?\s+(?:experiment|chemistry|version|edition)\b",
     "the reader addressed as a child"),
)

# Renderings the shell already ships (web/app/src/locales/de.json).
# "Werkbank" is a carpenter's bench; the simulated lab is "Labor" and a
# physical chemistry bench is a "Labortisch". Either is a fix.
#
# "auf der Labor" is the fossil of the migration itself: the German read "auf
# der Bank", `Bank` was replaced with `Labor`, and the feminine article stayed
# behind on a neuter noun. Two of them shipped. A word-for-word glossary pass
# leaves exactly this kind of wreckage, so the wreckage is what gets gated.
GLOSSARY_HARD = {
    "de": {
        "Werkbank": "Labor or Labortisch",
        "Arbeitsplatte": "Labor",
        "auf der Labor": "im Labor (Labor is neuter)",
        "auf dem Labor": "im Labor",
    },
}


class Source(typing.NamedTuple):
    """One translatable English string, and where it came from."""

    file: str
    field: str
    value: str | list[str]


def english_paths() -> dict[str, Source]:
    """Every translatable field in the catalogue, as `<id>.<path>`."""
    out: dict[str, Source] = {}

    def walk(node, path):
        if isinstance(node, dict):
            for k, v in node.items():
                if k in TRANSLATABLE and isinstance(v, (str, list)):
                    out[".".join(path + [k])] = v
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
                for path, value in found.items():
                    out[f"{entry['id']}.{path}"] = Source(
                        f.name, path.rsplit(".", 1)[-1], value
                    )
    return out


def wording_offences(text: str) -> list[str]:
    """Age bands and children's-audience wording, with what was matched."""
    return [
        f"{why}: {m.group(0)!r}"
        for pattern, why in AGE_WORDING
        for m in re.finditer(pattern, text, re.IGNORECASE)
    ]


def audit(code: str, cat: dict, english: dict[str, Source]) -> list[str]:
    """Everything wrong with one language's file, as printable lines."""
    problems: list[str] = []
    shipping = code in COMPLETE

    for key, value in cat.items():
        src = english.get(key)
        if src is None:
            problems.append(f"STALE: {key!r} translates nothing in the catalogue")
            continue
        if isinstance(src.value, list) != isinstance(value, list):
            kind = "a list" if isinstance(src.value, list) else "a string"
            problems.append(f"SHAPE: {key} translates {kind} as the other")
        elif isinstance(value, list) and src.field in ALIGNED:
            if len(value) != len(src.value):
                problems.append(
                    f"ALIGNMENT: {key} has {len(value)} items, the English has "
                    f"{len(src.value)} — a positional list read by index"
                )
        text = " ".join(value) if isinstance(value, list) else str(value)
        problems += [f"WORDING: {key} carries {o}" for o in wording_offences(text)]
        for bad, good in GLOSSARY_HARD.get(code, {}).items():
            if re.search(rf"\b{bad}\b", text):
                problems.append(
                    f"GLOSSARY: {key} says {bad!r}; the interface says {good!r}"
                )

    if shipping:
        for key in sorted(english):
            if key not in cat:
                problems.append(
                    f"MISSING: {key} is English in a language that ships "
                    f"({english[key].file})"
                )
    return problems


def main() -> int:
    check = "--check" in sys.argv
    problems = 0

    english = english_paths()
    per_file_total = collections.Counter(src.file for src in english.values())

    languages = sorted(p.stem for p in I18N.glob("*.toml")) if I18N.is_dir() else []
    if not languages:
        print("no translations in codex/i18n/")
        return 0

    for code in languages:
        cat = tomllib.load(open(I18N / f"{code}.toml", "rb"))
        ships = " (ships — a gap is a failure)" if code in COMPLETE else " (in progress)"
        print(f"\n== {code}{ships}")
        print(f"{'file':<24} {code:>8} {'English':>8}   coverage")
        have = collections.Counter()
        for key in cat:
            src = english.get(key)
            if src is not None:
                have[src.file] += 1

        total_have = total_all = 0
        for name in sorted(per_file_total):
            n, all_n = have[name], per_file_total[name]
            total_have += n
            total_all += all_n
            pct = 100 * n / all_n if all_n else 100.0
            print(f"{name:<24} {n:>8} {all_n:>8}   {pct:5.1f}% {'#' * int(pct / 5)}")
        pct = 100 * total_have / total_all if total_all else 100.0
        print(f"{'TOTAL':<24} {total_have:>8} {total_all:>8}   {pct:5.1f}%")

        found = audit(code, cat, english)
        for line in found:
            print(f"   {line}")
        problems += len(found)

    if problems:
        print(f"\n{problems} problem(s)")
    return 1 if (problems and check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
