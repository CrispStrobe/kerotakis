#!/usr/bin/env python3
"""Keys carried in BOTH `terms` and `messages`, and the two that disagree.

`web/app/src/lib/i18n.svelte.ts` merges the two sections as
`{...terms, ...messages}`, deliberately, so a message overrides a term.
That is a feature: the same English string can be a lesson's short NAME in
one place and a UI label in another. It is also silent, and a collision
whose two sides carry DIFFERENT German means one translation is replacing
another where nobody can see it happen.

This counts the collisions and fails on a disagreeing one that is not
recorded below.

Two things it deliberately does not do:

* It does not fail on a collision whose sides AGREE. 57 of the 59 do, and
  they are harmless — the merge picks one of two identical strings.
* It does not pick a winner for the two that disagree. Which of *Kalkwasser*
  and *Kalkwasserprobe* `limewater` should be is a wording judgement about
  the substance against the test for it, and a lint has no standing to make
  it. They are recorded with what each side means, and a THIRD would fail.

Written after a count of mine was wrong in the other direction: I reported
"59 duplicate keys" by counting key/value pairs across the whole file, when
neither section holds a duplicate key at all. The 59 are cross-section
collisions. A number nobody can reproduce is how #505 happened.
"""

from __future__ import annotations

import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
LOCALES = ROOT / "web/app/src/locales"

# A collision whose two sides carry different text, with what each means.
# An entry here is a known wording question, NOT an exemption on principle:
# a new disagreement fails, and these fail too once somebody decides.
# Empty since 2026-09-23, and an empty list is the claim rather than the
# absence of one: no key in this app means two different things.
#
# The two that were here are worth remembering because they failed in
# opposite directions, and only one of them was a translation problem.
#
#   `limewater` was ONE English word doing two jobs — the substance and
#   the test for it — so no German could be right. `messages` wins the
#   merge, so the test's name reached a shelf and an ingredients list,
#   where you do not pour a test into a beaker. Fixed by giving the TEST
#   its own English ("limewater test", via `gasTests.ts`), not by choosing
#   between two translations of one key.
#
#   `invisible ink boundary` was one meaning with two German renderings,
#   and the one that won dropped the word the lesson is named for. Its
#   five sibling boundary lessons all read "Grenze …", so the `terms`
#   entry was right and the `messages` duplicate was deleted.
#
# A key carrying two meanings is a source-text problem. A key carrying two
# translations is a translation problem. They look identical here and they
# are not fixed the same way.
RECORDED: dict[str, str] = {}


def bundles() -> list[pathlib.Path]:
    """Every shipped language. `_template.json` is the translator's brief,
    not a language, and its values are empty by design."""
    return sorted(p for p in LOCALES.glob("*.json") if not p.name.startswith("_"))


def collisions(doc: dict) -> dict[str, tuple[str, str]]:
    terms = doc.get("terms") or {}
    messages = doc.get("messages") or {}
    return {
        key: (terms[key], messages[key])
        for key in sorted(set(terms) & set(messages))
    }


def main() -> int:
    check = "--check" in sys.argv
    problems: list[str] = []
    for path in bundles():
        doc = json.loads(path.read_text())
        found = collisions(doc)
        disagreeing = {k: v for k, v in found.items() if v[0] != v[1]}
        print(
            f"{path.name}: {len(found)} keys in both sections, "
            f"{len(disagreeing)} carrying different text"
        )
        for key, (term, message) in disagreeing.items():
            if key in RECORDED:
                print(f"   recorded: {key!r} — {RECORDED[key]}")
                continue
            problems.append(
                f"{path.name}: {key!r} is {term!r} as a term and {message!r} as a "
                f"message. One silently replaces the other. Decide which it is, "
                f"or record it in locale-collision-lint.py with the reason."
            )
        # A recorded entry that has been resolved must not stay recorded:
        # a stale exemption is a lint that has quietly stopped checking.
        for key in RECORDED:
            if key in found and found[key][0] == found[key][1]:
                problems.append(
                    f"{path.name}: {key!r} no longer disagrees — delete it from "
                    f"RECORDED and say what settled it."
                )
            elif key not in found:
                problems.append(
                    f"{path.name}: {key!r} is recorded as a collision but is not "
                    f"in both sections any more — delete it from RECORDED."
                )
    for line in problems:
        print(f"   {line}")
    if problems and check:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
