#!/usr/bin/env python3
"""How much of the ENGINE's own prose is translatable yet (I18N-5).

`tools/codex-locale-lint.py` measures the catalogue. This measures the
other half: the sentences `crates/kerotakis-core/src/render.rs` composes
itself, which no amount of `_de` keys in the codex can reach.

Two different numbers, and the second is the one that matters:

  reachable  — literals that go through `locale.t` / `locale.fill`, so a
               catalogue CAN translate them
  translated — of those, how many each shipped language actually carries

A literal still inside a bare `format!` is not merely untranslated; it is
untranslatABLE, and no amount of work in a .toml will change that. That is
why the two are counted separately: the first is a code change, the second
is a translation.

    python3 tools/engine-locale-lint.py
    python3 tools/engine-locale-lint.py --check   # non-zero if a key is orphaned

The check is deliberately NOT "fail below N% translated". A partial
translation is the intended state — every string falls back to English on
its own — so a coverage floor would only encourage machine-filling the
catalogue to make a number go green.
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
RENDER = ROOT / "crates/kerotakis-core/src/render.rs"
CATALOGUES = ROOT / "crates/kerotakis-core/i18n"

# `locale.t("vessel.open", ", open to atmosphere")` and the fill() form.
CALL = re.compile(r'locale\s*\.\s*(?:t|fill)\s*\(\s*"([^"]+)"\s*,\s*"((?:[^"\\]|\\.)*)"')
# A bare user-facing literal. Newlines are excluded from the character
# class on purpose: `[^"\\]` matches them, so the pattern would span from
# the end of one string to the start of the next across a multi-line
# format! and report the code between them as prose.
LITERAL = re.compile(r'"((?:[^"\\\n]|\\.){6,})"')

# `locale.lookup(&format!("glassware.{}", …))`: keys under a prefix looked
# up this way never appear as a literal at a call site, so they are used
# even though nothing names them. Reporting them as orphans is the lint
# being wrong about a legitimate pattern, and a lint that cries wolf on a
# legitimate pattern is one people learn to ignore.
DYNAMIC = re.compile(r'locale\s*\.\s*lookup\s*\(\s*&?\s*format!\s*\(\s*"([\w.-]+)\.\{')

# A dotted key named anywhere in the file, which covers the case where the
# key is chosen by a match arm rather than passed literally:
#
#   locale.t(match p.phase { Phase::Gas => "phase.gas", … }, …)
#
# Weaker than "named at a call site", deliberately. The match is the
# clearest way to write a four-way choice, and a lint that is wrong about
# correct code is a lint people stop reading.
MENTIONED = re.compile(r'"([a-z][\w-]*(?:\.[\w -]+)+)"')
PROSE = re.compile(r"[A-Za-z]{3,}\s+[A-Za-z]{2,}")


def load_catalogue(path: pathlib.Path) -> dict[str, str]:
    doc = tomllib.load(open(path, "rb"))
    out: dict[str, str] = {}
    for section, body in doc.items():
        if isinstance(body, dict):
            for k, v in body.items():
                if isinstance(v, str):
                    out[f"{section}.{k}"] = v
        elif isinstance(body, str):
            out[section] = body
    return out


def main() -> int:
    src = RENDER.read_text()
    used = {m.group(1): m.group(2) for m in CALL.finditer(src)}
    dynamic = {m.group(1) for m in DYNAMIC.finditer(src)}
    mentioned = {m.group(1) for m in MENTIONED.finditer(src)}

    # Everything that looks like prose, minus what already goes through a
    # call. Rough by design: it over-reports rather than under-reports,
    # because a missed line is one nobody knows is English.
    reachable_texts = set(used.values())
    bare = set()
    for m in LITERAL.finditer(src):
        text = m.group(1)
        if text in reachable_texts or not PROSE.search(text):
            continue
        # Doc comments and attributes are not output.
        line_start = src.rfind("\n", 0, m.start()) + 1
        line = src[line_start : m.start()]
        if line.lstrip().startswith(("//", "#[", "///")):
            continue
        bare.add(text)

    # A key used by two DIFFERENT templates is the worst failure this
    # file can have: not a missing translation but a wrong sentence, since
    # whichever German lands in the catalogue renders for both. Each bulk
    # converter checked its own output for collisions and neither checked
    # the file, so `event.smelled.lv3` was created twice.
    per_key = collections.defaultdict(set)
    for m in CALL.finditer(src):
        per_key[m.group(1)].add(m.group(2))
    shared = {k: v for k, v in per_key.items() if len(v) > 1}
    if shared:
        print("KEY USED BY TWO DIFFERENT SENTENCES:")
        for k, texts in sorted(shared.items()):
            print(f"   {k}")
            for x in sorted(texts):
                print(f"      {x[:70]}")

    print(f"{'engine prose in render.rs':<34}")
    print(f"   reachable by a catalogue : {len(used):>4} keys")
    print(f"   still inside a bare format!: {len(bare):>4} literals")

    problems = len(shared)
    print()
    print(f"{'language':<12} {'translated':>10} {'of':>4} {'reachable':>10}   coverage")
    for path in sorted(CATALOGUES.glob("*.toml")):
        code = path.stem
        cat = load_catalogue(path)
        hit = sum(1 for k in used if k in cat)
        pct = 100 * hit / len(used) if used else 100.0
        dyn = sum(1 for k in cat if k.split(".")[0] in dynamic)
        arm = sum(1 for k in cat if k not in used and k.split(".")[0] not in dynamic
                  and k in mentioned)
        parts = []
        if dyn:
            parts.append(f"+{dyn} looked up by value")
        if arm:
            parts.append(f"+{arm} chosen by a match arm")
        extra = f"   ({', '.join(parts)})" if parts else ""
        print(f"{code:<12} {hit:>10} {'/':>4} {len(used):>10}   {pct:5.1f}%{extra}")

        # A key in the catalogue that no call site asks for is dead weight,
        # and usually a rename nobody finished.
        for k in sorted(set(cat) - set(used)):
            if k.split(".")[0] in dynamic or k in mentioned:
                continue
            print(f"   ORPHAN: {code}.toml has '{k}', which render.rs never asks for")
            problems += 1

    if bare:
        print(f"\nnot yet reachable — each needs a locale.t() at its call site:")
        for text in sorted(bare)[:12]:
            print(f"   {text[:76]}")
        if len(bare) > 12:
            print(f"   … and {len(bare) - 12} more")

    if problems:
        print(f"\n{problems} problem(s)")
    return 1 if (problems and "--check" in sys.argv) else 0


if __name__ == "__main__":
    raise SystemExit(main())
